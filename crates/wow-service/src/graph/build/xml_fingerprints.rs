//! Explicit bounded XML observations; decoded selections cannot supply native authority.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_core::{
    CanonicalResult, ContentDigest, EvidenceRecord, ProjectGenerationId, SourceHandle, SourceSpan,
    StableHandleId,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphCoverageState, GraphLocalAssertion, GraphNodeId, GraphProducerLookup, GraphResolvedEntity,
};
use wow_project::{
    graph::{
        PLATFORM_RAW_INVENTORY_PARTITION, PLATFORM_RAW_MEMBER_KIND, PlatformGraphProvenance,
        ProjectScopedXmlClassification, ProjectXmlClassificationOccurrence,
        XmlClassificationCatalogSupport, XmlClassificationCatalogWitness,
        XmlClassificationMembership,
        xml_fingerprints::{
            XML_SCHEMA_CLASS_COMPONENT_RECIPE, XmlSchemaClassComponentValue,
            build_selected_xml_schema_class_components,
        },
    },
    load::schema::classification::{XmlClassificationBinding, XmlOccurrenceClassification},
};

use super::{checkpoint, error, graph_error, project_error};
use crate::{ServiceErrorCode, ServiceResult};

const SELECTION_SCHEMA: &str = "wow-service/xml-fingerprint-selection/1";
const RESULT_SCHEMA: &str = "wow-service/xml-fingerprint-manifest/1";
const OCCURRENCE_KIND: &str = "xml_source_occurrence";
const CLASS_KIND: &str = "xml_schema_class";
const CLASSIFICATION_RELATION: &str = "xml_occurrence_schema_class";

/// Exact native proposal IDs, not tag names, unscoped index IDs or guessed roots.
/// An absent class selects an unclassified native occurrence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct XmlFingerprintTarget {
    occurrence_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    class_id: Option<Box<str>>,
}
impl XmlFingerprintTarget {
    #[must_use]
    pub fn new(occurrence_id: Box<str>, class_id: Option<Box<str>>) -> Self {
        Self {
            occurrence_id,
            class_id,
        }
    }
}

/// Targets must be strictly ordered by exact occurrence proposal ID.
/// This query does not imply that any unselected occurrence or class is absent.
#[derive(Debug, Clone, Serialize)]
pub struct XmlFingerprintSelection {
    schema: &'static str,
    targets: Vec<XmlFingerprintTarget>,
    max_records: usize,
    max_metadata_bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSelection {
    schema: String,
    targets: Vec<XmlFingerprintTarget>,
    max_records: usize,
    max_metadata_bytes: usize,
}
impl XmlFingerprintSelection {
    pub fn new(
        targets: Vec<XmlFingerprintTarget>,
        max_records: usize,
        max_metadata_bytes: usize,
    ) -> ServiceResult<Self> {
        if max_records == 0 || max_metadata_bytes == 0 {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        if targets.len() > max_records {
            return Err(error(ServiceErrorCode::BudgetExceeded));
        }
        if targets.iter().any(|target| {
            target.occurrence_id.is_empty()
                || target.class_id.as_ref().is_some_and(|id| id.is_empty())
        }) || targets
            .windows(2)
            .any(|pair| pair[0].occurrence_id >= pair[1].occurrence_id)
        {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        let selection = Self {
            schema: SELECTION_SCHEMA,
            targets,
            max_records,
            max_metadata_bytes,
        };
        super::preflight_cancellable(
            &selection,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            &AtomicBool::new(false),
        )?;
        Ok(selection)
    }
    pub fn from_json_slice(bytes: &[u8]) -> ServiceResult<Self> {
        Self::from_json_slice_cancellable(bytes, &AtomicBool::new(false))
    }
    pub fn from_json_slice_cancellable(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        checkpoint(stop)?;
        let wire: WireSelection = super::super::input::decode(
            bytes,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GRAPH_REQUEST_MAX_BYTES,
            super::super::GraphReadStage::Request,
            stop,
        )
        .map_err(|failure| error(failure.code))?;
        if wire.schema != SELECTION_SCHEMA {
            return Err(error(ServiceErrorCode::InvalidRequest));
        }
        let selection = Self::new(wire.targets, wire.max_records, wire.max_metadata_bytes)?;
        checkpoint(stop)?;
        Ok(selection)
    }

    pub(super) fn build(
        &self,
        direct: &PlatformGraphProvenance<'_>,
        stop: &AtomicBool,
    ) -> ServiceResult<XmlFingerprintManifest> {
        checkpoint(stop)?;
        let inclusion = direct
            .xml_classification_inclusion()
            .ok_or_else(|| error(ServiceErrorCode::IdentityMismatch))?;
        let classification = inclusion.classification();
        if inclusion.membership().scope() != direct.scope()
            || direct.source().context().context_id() != direct.scope().source_context_id
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        let mut budget = MetadataBudget::new(self.max_metadata_bytes, stop);
        budget.charge(self)?;
        // Charge the complete binding before any owned custody copies.
        let project = direct.project();
        let binding_view = (
            project.snapshot_id(),
            project.project_generation(),
            project.configuration().configuration_digest(),
            direct.scope(),
            classification.binding(),
            classification.profile_digest(),
            classification.digest(),
            inclusion.membership(),
            classification.catalog_support(),
        );
        budget.charge(&binding_view)?;

        let mut occurrences = BTreeMap::new();
        let mut max_occurrence_id_bytes = 0_usize;
        let mut max_class_id_bytes = 0_usize;
        for document in classification.documents() {
            checkpoint(stop)?;
            if document.report.occurrences().len() != document.occurrences.len() {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            for (offset, (native, projected)) in document
                .report
                .occurrences()
                .iter()
                .zip(&document.occurrences)
                .enumerate()
            {
                checkpoint(stop)?;
                if native.occurrence() != projected.occurrence {
                    return Err(error(ServiceErrorCode::InternalContractViolation));
                }
                budget.charge(&(
                    "xml-fingerprint-occurrence-index",
                    &projected.occurrence_proposal_id,
                ))?;
                max_occurrence_id_bytes =
                    max_occurrence_id_bytes.max(projected.occurrence_proposal_id.len());
                max_class_id_bytes = max_class_id_bytes
                    .max(projected.class_proposal_id.as_ref().map_or(0, String::len));
                if occurrences
                    .insert(
                        projected.occurrence_proposal_id.as_str(),
                        (document, offset),
                    )
                    .is_some()
                {
                    return Err(error(ServiceErrorCode::InternalContractViolation));
                }
            }
        }
        // Bound strings against genuine IDs before string comparison or lookup.
        for target in &self.targets {
            checkpoint(stop)?;
            if target.occurrence_id.is_empty()
                || target.occurrence_id.len() > max_occurrence_id_bytes
                || target
                    .class_id
                    .as_ref()
                    .is_some_and(|id| id.is_empty() || id.len() > max_class_id_bytes)
            {
                return Err(error(ServiceErrorCode::InvalidRequest));
            }
            let (document, offset) = occurrences
                .get(target.occurrence_id.as_ref())
                .ok_or_else(|| error(ServiceErrorCode::InvalidRequest))?;
            if document.occurrences[*offset].class_proposal_id.as_deref()
                != target.class_id.as_deref()
            {
                return Err(error(ServiceErrorCode::InvalidRequest));
            }
        }
        for pair in self.targets.windows(2) {
            checkpoint(stop)?;
            if pair[0].occurrence_id >= pair[1].occurrence_id {
                return Err(error(ServiceErrorCode::InvalidRequest));
            }
        }
        let lookup = direct.graph().producer_lookup(stop).map_err(graph_error)?;
        if lookup.scope() != direct.scope() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        // One representative per genuine whole-file handle; coordinates stay in
        // the original sealed catalog retained below, not a new source registry.
        let mut catalog = BTreeMap::<StableHandleId, &XmlClassificationCatalogWitness>::new();
        for witness in classification.catalog_support().witnesses() {
            checkpoint(stop)?;
            if let Some(previous) = catalog.get(&witness.source_handle_id) {
                if previous.raw_member_proposal_id != witness.raw_member_proposal_id
                    || previous.document != witness.document
                    || previous.content_digest != witness.content_digest
                    || previous.evidence_id != witness.evidence_id
                {
                    return Err(error(ServiceErrorCode::InternalContractViolation));
                }
            } else {
                budget.charge(&("xml-fingerprint-catalog-index", witness.source_handle_id))?;
                catalog.insert(witness.source_handle_id, witness);
            }
        }
        let mut records = Vec::new();
        let mut selected_ids = Vec::new();
        for target in &self.targets {
            checkpoint(stop)?;
            let &(document, offset) = occurrences
                .get(target.occurrence_id.as_ref())
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
            let projected = &document.occurrences[offset];
            let observation = &document.report.occurrences()[offset];
            let support = join(
                direct,
                &lookup,
                document,
                projected,
                observation,
                &catalog,
                &mut budget,
                stop,
            )?;
            budget.charge(&(
                &document.scope,
                document.document.as_str(),
                projected,
                observation,
            ))?;
            let record = XmlFingerprintRecord {
                scope: document.scope.clone(),
                document: document.document.clone(),
                projection: projected.clone(),
                observation: observation.clone(),
                support,
            };
            budget.charge(&(
                "xml-fingerprint-selected-component-id",
                &projected.occurrence_proposal_id,
            ))?;
            selected_ids.push(projected.occurrence_proposal_id.as_str());
            records.push(record);
        }
        // The native recipe continues the same cumulative ledger and returns
        // owned values in the exact selected occurrence order.
        let (components, native_used) = build_selected_xml_schema_class_components(
            &inclusion,
            &selected_ids,
            budget.used,
            self.max_metadata_bytes,
            stop,
        )
        .map_err(project_error)?;
        if components.len() != records.len()
            || native_used < budget.used
            || native_used > budget.limit
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        budget.used = native_used;
        for ((component_id, _), record) in components.iter().zip(&records) {
            checkpoint(stop)?;
            if component_id != &record.projection.occurrence_proposal_id {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
        }
        let manifest = XmlFingerprintManifest {
            schema: RESULT_SCHEMA,
            component_recipe: XML_SCHEMA_CLASS_COMPONENT_RECIPE,
            max_records: self.max_records,
            max_metadata_bytes: self.max_metadata_bytes,
            // Observation selection never upgrades the admitted universe's coverage.
            coverage: GraphCoverageState::Partial,
            binding: XmlFingerprintBinding {
                project_snapshot_id: project.snapshot_id().into(),
                project_generation: project.project_generation(),
                configuration_digest: project.configuration().configuration_digest(),
                graph_scope: direct.scope().clone(),
                source: classification.binding().clone(),
                reviewed_profile_digest: classification.profile_digest(),
                classification_digest: classification.digest(),
                membership: inclusion.membership().clone(),
                catalog_support: classification.catalog_support().clone(),
            },
            records,
            components,
        };
        // Charge the complete encoded envelope, including framing, to the same
        // cumulative native/service ledger before returning an owned result.
        budget.charge(&manifest)?;
        checkpoint(stop)?;
        Ok(manifest)
    }
}

#[derive(Debug, Serialize)]
pub struct XmlFingerprintBinding {
    project_snapshot_id: Box<str>,
    project_generation: ProjectGenerationId,
    configuration_digest: ContentDigest<CanonicalResult>,
    graph_scope: GraphAssertionRecordScope,
    source: XmlClassificationBinding,
    reviewed_profile_digest: ContentDigest<CanonicalResult>,
    classification_digest: ContentDigest<CanonicalResult>,
    membership: XmlClassificationMembership,
    catalog_support: XmlClassificationCatalogSupport,
}
#[derive(Debug, Serialize)]
pub struct XmlFingerprintSupport {
    occurrence_assertion: GraphAssertionRef,
    occurrence_node_id: GraphNodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    class_assertion: Option<GraphAssertionRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    class_node_id: Option<GraphNodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classification_assertion: Option<GraphAssertionRef>,
    observed_source: SourceHandle,
    observed_evidence: EvidenceRecord,
    raw_catalog: Vec<XmlFingerprintRawSupport>,
}
#[derive(Debug, Serialize)]
pub struct XmlFingerprintRawSupport {
    assertion: GraphAssertionRef,
    node_id: GraphNodeId,
    source: SourceHandle,
    evidence: EvidenceRecord,
}
#[derive(Debug, Serialize)]
pub struct XmlFingerprintRecord {
    scope: wow_project::graph::ProjectXmlFactScope,
    document: String,
    projection: ProjectXmlClassificationOccurrence,
    observation: XmlOccurrenceClassification,
    support: XmlFingerprintSupport,
}
/// Serialize-only observation result: it cannot be reopened as native authority.
/// components[i] belongs exactly to records[i]; neither list is a source universe.
#[derive(Debug, Serialize)]
pub struct XmlFingerprintManifest {
    schema: &'static str,
    component_recipe: &'static str,
    max_records: usize,
    max_metadata_bytes: usize,
    coverage: GraphCoverageState,
    binding: XmlFingerprintBinding,
    records: Vec<XmlFingerprintRecord>,
    components: Vec<(String, XmlSchemaClassComponentValue)>,
}

impl XmlFingerprintManifest {
    #[must_use]
    pub fn records(&self) -> &[XmlFingerprintRecord] {
        &self.records
    }
    #[must_use]
    pub fn components(&self) -> &[(String, XmlSchemaClassComponentValue)] {
        &self.components
    }
    #[must_use]
    pub const fn coverage(&self) -> GraphCoverageState {
        self.coverage
    }
}
impl XmlFingerprintRecord {
    #[must_use]
    pub const fn projection(&self) -> &ProjectXmlClassificationOccurrence {
        &self.projection
    }
    #[must_use]
    pub const fn observation(&self) -> &XmlOccurrenceClassification {
        &self.observation
    }
}

#[allow(clippy::too_many_arguments)]
fn join(
    direct: &PlatformGraphProvenance<'_>,
    lookup: &GraphProducerLookup<'_>,
    document: &ProjectScopedXmlClassification,
    projected: &ProjectXmlClassificationOccurrence,
    observation: &XmlOccurrenceClassification,
    catalog: &BTreeMap<StableHandleId, &XmlClassificationCatalogWitness>,
    budget: &mut MetadataBudget<'_>,
    stop: &AtomicBool,
) -> ServiceResult<XmlFingerprintSupport> {
    checkpoint(stop)?;
    let occurrence_ref = reference(
        direct,
        GraphAssertionKind::Entity,
        &projected.occurrence_proposal_id,
    )?;
    let occurrence = lookup
        .entity(direct.scope(), occurrence_ref, stop)
        .map_err(graph_error)?;
    if occurrence.proposal().entity_kind_id() != OCCURRENCE_KIND
        || occurrence.proposal().confidence() != GraphConfidence::Proven
        || occurrence.proposal().source_handle_ids() != [projected.source_handle_id]
        || occurrence.proposal().evidence_ids() != [projected.evidence_id]
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let handle = direct
        .evidence_catalog()
        .source_handle(&projected.source_handle_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let evidence = direct
        .evidence_catalog()
        .evidence(&projected.evidence_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let span = observation.span();
    if handle.path().as_str() != document.document
        || *handle.content_digest() != document.report.source_digest()
        || handle.span().byte_start() != Some(span.byte_start)
        || handle.span().byte_end() != Some(span.byte_end)
        || handle.project_generation() != Some(direct.project().project_generation())
        || handle.reference_generation()
            != Some(direct.project().configuration().reference_generation())
        || evidence.context_id() != direct.scope().source_context_id
        || !evidence
            .source_handle_ids()
            .contains(&projected.source_handle_id)
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let mut raw_catalog = Vec::new();
    let (class_assertion, class_node_id, classification_assertion) = match (
        &projected.class_proposal_id,
        &projected.relation_proposal_id,
    ) {
        (None, None) => (None, None, None),
        (Some(class_id), Some(relation_id)) => {
            let class_ref = reference(direct, GraphAssertionKind::Entity, class_id)?;
            let class = lookup
                .entity(direct.scope(), class_ref, stop)
                .map_err(graph_error)?;
            let relation_ref = reference(direct, GraphAssertionKind::Relation, relation_id)?;
            let relation = lookup
                .relation(direct.scope(), relation_ref, stop)
                .map_err(graph_error)?;
            if class.proposal().entity_kind_id() != CLASS_KIND
                || class.proposal().confidence() != GraphConfidence::Derived
                || relation.proposal().relation_kind_id() != CLASSIFICATION_RELATION
                || relation.proposal().confidence() != GraphConfidence::Derived
                || relation.accepted().edge().from() != occurrence.accepted().node().node_id()
                || relation.accepted().edge().to() != class.accepted().node().node_id()
                || !relation
                    .proposal()
                    .source_handle_ids()
                    .contains(&projected.source_handle_id)
                || !relation
                    .proposal()
                    .evidence_ids()
                    .contains(&projected.evidence_id)
            {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            for id in class.proposal().source_handle_ids() {
                checkpoint(stop)?;
                let witness = catalog
                    .get(id)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                if !class
                    .proposal()
                    .evidence_ids()
                    .contains(&witness.evidence_id)
                    || !relation.proposal().source_handle_ids().contains(id)
                    || !relation
                        .proposal()
                        .evidence_ids()
                        .contains(&witness.evidence_id)
                {
                    return Err(error(ServiceErrorCode::InternalContractViolation));
                }
                raw_catalog.push(raw_support(direct, lookup, witness, budget, stop)?);
            }
            if raw_catalog.is_empty() {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            budget.charge(&(class_ref, class.accepted().node().node_id(), relation_ref))?;
            (
                Some(class_ref.clone()),
                Some(class.accepted().node().node_id().clone()),
                Some(relation_ref.clone()),
            )
        }
        _ => return Err(error(ServiceErrorCode::InternalContractViolation)),
    };
    budget.charge(&(
        occurrence_ref,
        occurrence.accepted().node().node_id(),
        handle,
        evidence,
    ))?;
    checkpoint(stop)?;
    Ok(XmlFingerprintSupport {
        occurrence_assertion: occurrence_ref.clone(),
        occurrence_node_id: occurrence.accepted().node().node_id().clone(),
        class_assertion,
        class_node_id,
        classification_assertion,
        observed_source: handle.clone(),
        observed_evidence: evidence.clone(),
        raw_catalog,
    })
}
fn raw_support(
    direct: &PlatformGraphProvenance<'_>,
    lookup: &GraphProducerLookup<'_>,
    witness: &XmlClassificationCatalogWitness,
    budget: &mut MetadataBudget<'_>,
    stop: &AtomicBool,
) -> ServiceResult<XmlFingerprintRawSupport> {
    checkpoint(stop)?;
    let raw_ref = reference(
        direct,
        GraphAssertionKind::Entity,
        &witness.raw_member_proposal_id,
    )?;
    let raw: GraphResolvedEntity<'_> = lookup
        .entity(direct.scope(), raw_ref, stop)
        .map_err(graph_error)?;
    let handle = direct
        .evidence_catalog()
        .source_handle(&witness.source_handle_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let evidence = direct
        .evidence_catalog()
        .evidence(&witness.evidence_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    if raw.partition().partition_id() != PLATFORM_RAW_INVENTORY_PARTITION
        || raw.proposal().entity_kind_id() != PLATFORM_RAW_MEMBER_KIND
        || raw.proposal().source_handle_ids() != [witness.source_handle_id]
        || raw.proposal().evidence_ids() != [witness.evidence_id]
        || handle.path().as_str() != witness.document
        || *handle.content_digest() != witness.content_digest
        || handle.span() != SourceSpan::whole_file()
        || handle.project_generation() != Some(direct.project().project_generation())
        || handle.reference_generation()
            != Some(direct.project().configuration().reference_generation())
        || evidence.context_id() != direct.scope().source_context_id
        || !evidence
            .source_handle_ids()
            .contains(&witness.source_handle_id)
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    // finish() already checked exact raw admission, member bytes and each typed
    // catalog span. Reuse that capability; never re-admit XSD as a Main file.
    budget.charge(&(raw_ref, raw.accepted().node().node_id(), handle, evidence))?;
    checkpoint(stop)?;
    Ok(XmlFingerprintRawSupport {
        assertion: raw_ref.clone(),
        node_id: raw.accepted().node().node_id().clone(),
        source: handle.clone(),
        evidence: evidence.clone(),
    })
}
fn reference<'a>(
    direct: &'a PlatformGraphProvenance<'_>,
    kind: GraphAssertionKind,
    proposal_id: &str,
) -> ServiceResult<&'a GraphAssertionRef> {
    direct
        .assertion(&GraphLocalAssertion {
            kind,
            proposal_id: proposal_id.into(),
        })
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))
}

/// One operation ledger: no per-record allowance, cap reset, or charge discount.
struct MetadataBudget<'a> {
    used: usize,
    limit: usize,
    stop: &'a AtomicBool,
    failure: Option<ServiceErrorCode>,
}
impl<'a> MetadataBudget<'a> {
    fn new(limit: usize, stop: &'a AtomicBool) -> Self {
        Self {
            used: 0,
            limit,
            stop,
            failure: None,
        }
    }
    fn charge(&mut self, value: &impl Serialize) -> ServiceResult<()> {
        checkpoint(self.stop)?;
        let encoded = serde_json::to_writer(&mut *self, value);
        checkpoint(self.stop)?;
        encoded.map_err(|_| {
            error(
                self.failure
                    .unwrap_or(ServiceErrorCode::CanonicalizationFailed),
            )
        })
    }
}
impl Write for MetadataBudget<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.stop.load(Ordering::Acquire) {
            self.failure = Some(ServiceErrorCode::Cancelled);
            return Err(std::io::Error::other("XML fingerprint metadata cancelled"));
        }
        if bytes.len() > self.limit.saturating_sub(self.used) {
            self.failure = Some(ServiceErrorCode::BudgetExceeded);
            return Err(std::io::Error::other(
                "XML fingerprint metadata exceeds its limit",
            ));
        }
        self.used += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
