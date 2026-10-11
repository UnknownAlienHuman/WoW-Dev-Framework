//! Native catalog coordinates use admitted raw whole-file support, never Main eligibility.
use super::*;
use crate::load::XmlSourceSpan;
use crate::load::schema::{
    AdmittedXmlSchema, XmlSchemaReference, classification::XmlClassificationWitness,
};
use crate::platform_source::PlatformFileKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassificationCatalogWitness {
    pub component_id: ContentDigest<CanonicalResult>,
    pub component_kind: XmlSchemaComponentKind,
    pub component_state: XmlSchemaComponentState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_parent: Option<ContentDigest<CanonicalResult>>,
    pub document: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub span: XmlSourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<XmlSchemaReference>,
    pub raw_member_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

/// Typed coordinates are observations; graph support is the admitted raw member.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlClassificationCatalogSupport {
    catalog_digest: ContentDigest<CanonicalResult>,
    witnesses: BTreeMap<ContentDigest<CanonicalResult>, XmlClassificationCatalogWitness>,
    references_by_witness:
        BTreeMap<ContentDigest<CanonicalResult>, Vec<ContentDigest<CanonicalResult>>>,
}

impl XmlClassificationCatalogSupport {
    pub fn witnesses(&self) -> impl Iterator<Item = &XmlClassificationCatalogWitness> {
        self.witnesses.values()
    }

    pub(super) fn component(
        &self,
        id: ContentDigest<CanonicalResult>,
    ) -> ProjectResult<&XmlClassificationCatalogWitness> {
        let native = self.witnesses.get(&id).ok_or_else(invalid)?;
        if native.component_id != id || native.reference.is_some() {
            return Err(invalid());
        }
        Ok(native)
    }

    pub(super) fn resolve_witness(
        &self,
        witness: &XmlClassificationWitness,
        supports: &mut BTreeSet<(StableHandleId, EvidenceId)>,
        raw_inputs: &mut BTreeSet<GraphLocalAssertion>,
        budget: &mut ProducerBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        let mut ids = Vec::new();
        match witness {
            XmlClassificationWitness::Component { component } => {
                self.component(*component)?;
                ids.push(*component);
            }
            XmlClassificationWitness::Parent { child, parent } => {
                if self.component(*child)?.component_parent != Some(*parent) {
                    return Err(invalid());
                }
                self.component(*parent)?;
                ids.extend([*child, *parent]);
            }
            XmlClassificationWitness::Reference { source, target, .. } => {
                self.component(*source)?;
                self.component(*target)?;
                ids.extend([*source, *target]);
                let key = classifier_witness_id(witness)?;
                ids.extend(
                    self.references_by_witness
                        .get(&key)
                        .ok_or_else(invalid)?
                        .iter()
                        .copied(),
                );
            }
        }
        for id in ids {
            crate::analyzer::checkpoint(stop)?;
            let native = self.witnesses.get(&id).ok_or_else(invalid)?;
            budget.charge_serialized(
                &(
                    "xml-classification-proof-support",
                    id,
                    native.source_handle_id,
                    native.evidence_id,
                    &native.raw_member_proposal_id,
                ),
                stop,
            )?;
            supports.insert((native.source_handle_id, native.evidence_id));
            raw_inputs.insert(local_entity(&native.raw_member_proposal_id));
        }
        Ok(())
    }

    /// Called after the existing raw-membership pass in producer_evidence.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::graph) fn validate_native(
        &self,
        project: &ProjectView,
        source: &ProjectGraphProvenance,
        owner: &GraphPartitionSnapshot,
        lookup: &wow_graph::GraphProducerLookup<'_>,
        scope: &GraphAssertionRecordScope,
        addresses: &BTreeMap<GraphLocalAssertion, GraphAssertionRef>,
        budget: &mut ProducerBudget,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        crate::analyzer::checkpoint(stop)?;
        let admitted = project
            .configuration()
            .platform_packages()
            .ok_or_else(invalid)?
            .source();
        let raw = source.raw_inventory().ok_or_else(invalid)?;
        let reviewed = project
            .configuration()
            .xml_classification_profile()
            .ok_or_else(invalid)?;
        if self.catalog_digest != reviewed.catalog().receipt().digest() || lookup.scope() != scope {
            return Err(invalid());
        }
        reviewed.catalog().validate_source(admitted, stop)?;
        let mut bindings = BTreeMap::new();
        for witness in self.witnesses.values() {
            crate::analyzer::checkpoint(stop)?;
            if !bindings.contains_key(witness.document.as_str()) {
                let reference = addresses
                    .get(&local_entity(&witness.raw_member_proposal_id))
                    .ok_or_else(invalid)?;
                // Native capability: exact retained raw partition, batch, member and generation.
                let binding =
                    raw_inventory::bind_platform_raw_member(project, owner, reference, stop)?;
                budget.charge_serialized(
                    &(
                        "xml-classification-native-binding",
                        &witness.document,
                        binding.source_handle(),
                    ),
                    stop,
                )?;
                if binding.kind() != PlatformFileKind::Schema
                    || binding.source_snapshot_id() != raw.source_snapshot_id()
                    || binding.admission_digest() != raw.admission_digest()
                    || binding.source_handle().project_generation()
                        != Some(project.project_generation())
                    || binding.source_handle().reference_generation()
                        != Some(project.configuration().reference_generation())
                {
                    return Err(invalid());
                }
                bindings.insert(witness.document.as_str(), binding);
            }
            let binding = bindings
                .get(witness.document.as_str())
                .ok_or_else(invalid)?;
            let reference = addresses
                .get(&local_entity(&witness.raw_member_proposal_id))
                .ok_or_else(invalid)?;
            let resolved = lookup
                .entity(scope, reference, stop)
                .map_err(platform_producers::graph_error)?;
            if binding.path() != witness.document
                || binding.content_digest() != witness.content_digest
                || binding.source_handle().handle_id() != witness.source_handle_id
                || binding.source_handle().span() != SourceSpan::whole_file()
                || binding.reference() != reference
                || resolved.partition().partition_id() != PLATFORM_RAW_INVENTORY_PARTITION
                || resolved.proposal().entity_kind_id() != PLATFORM_RAW_MEMBER_KIND
                || resolved.proposal().source_handle_ids() != [witness.source_handle_id]
                || resolved.proposal().evidence_ids() != [witness.evidence_id]
            {
                return Err(invalid());
            }
            // Coordinates came from the sealed catalog, never source_file admission.
            let member = admitted.raw_member(&witness.document, stop)?;
            validate_range(member.bytes(), &witness.span)?;
        }
        crate::analyzer::checkpoint(stop)
    }
}

pub(super) fn prepare(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    catalog: &AdmittedXmlSchema,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlClassificationCatalogSupport> {
    crate::analyzer::checkpoint(stop)?;
    let admitted = project
        .configuration()
        .platform_packages()
        .ok_or_else(invalid)?
        .source();
    catalog.validate_source(admitted, stop)?;
    if source.context() != project.snapshot().generation_context() {
        return Err(invalid());
    }
    let mut result = XmlClassificationCatalogSupport {
        catalog_digest: catalog.receipt().digest(),
        witnesses: BTreeMap::new(),
        references_by_witness: BTreeMap::new(),
    };
    let mut raw_by_path = BTreeMap::new();
    for record in source.raw_inventory().ok_or_else(invalid)?.members() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &("xml-classification-raw-index", record.path.as_str()),
            stop,
        )?;
        if raw_by_path.insert(record.path.as_str(), record).is_some() {
            return Err(invalid());
        }
    }
    let mut components = BTreeMap::new();
    for component in catalog.components() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &("xml-classification-component-index", component.id()),
            stop,
        )?;
        if components.insert(component.id(), component).is_some() {
            return Err(invalid());
        }
        let record = raw_by_path
            .get(component.document())
            .copied()
            .ok_or_else(invalid)?;
        let witness = XmlClassificationCatalogWitness {
            component_id: component.id(),
            component_kind: component.kind(),
            component_state: component.state(),
            component_parent: component.parent(),
            document: component.document().into(),
            content_digest: component.content_digest(),
            span: component.span().clone(),
            reference: None,
            raw_member_proposal_id: record.proposal_id.clone(),
            source_handle_id: record.source_handle.handle_id(),
            evidence_id: record.evidence.evidence_id(),
        };
        insert(
            project,
            &mut result,
            component.id(),
            witness,
            record,
            budget,
            stop,
        )?;
    }
    for reference in catalog.references() {
        crate::analyzer::checkpoint(stop)?;
        let component = components.get(&reference.source()).ok_or_else(invalid)?;
        let record = raw_by_path
            .get(component.document())
            .copied()
            .ok_or_else(invalid)?;
        let id = reference_id(reference)?;
        let witness = XmlClassificationCatalogWitness {
            component_id: component.id(),
            component_kind: component.kind(),
            component_state: component.state(),
            component_parent: component.parent(),
            document: component.document().into(),
            content_digest: component.content_digest(),
            span: reference.value_span().clone(),
            reference: Some(reference.clone()),
            raw_member_proposal_id: record.proposal_id.clone(),
            source_handle_id: record.source_handle.handle_id(),
            evidence_id: record.evidence.evidence_id(),
        };
        insert(project, &mut result, id, witness, record, budget, stop)?;
        for &target in reference.candidates() {
            crate::analyzer::checkpoint(stop)?;
            let native = XmlClassificationWitness::Reference {
                source: reference.source(),
                reference_kind: reference.kind(),
                target,
                value_span: reference.value_span().clone(),
                decoded_value_digest: reference.decoded_value_digest(),
            };
            let key = classifier_witness_id(&native)?;
            budget.charge_serialized(&("xml-classification-reference-index", key, id), stop)?;
            let ids = result.references_by_witness.entry(key).or_default();
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    budget.charge_serialized(&result, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(result)
}

fn insert(
    project: &ProjectView,
    result: &mut XmlClassificationCatalogSupport,
    id: ContentDigest<CanonicalResult>,
    witness: XmlClassificationCatalogWitness,
    record: &ProjectRawInventoryMember,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    if result.witnesses.len() >= MAX_ASSERTIONS {
        return Err(exhausted());
    }
    let member = project
        .configuration()
        .platform_packages()
        .ok_or_else(invalid)?
        .source()
        .raw_member(&witness.document, stop)?;
    if member.kind() != PlatformFileKind::Schema
        || record.kind != member.kind()
        || record.content_digest != witness.content_digest
        || member.content_digest() != witness.content_digest
        || record.byte_length != member.byte_length()
        || record.source_handle.span() != SourceSpan::whole_file()
    {
        return Err(invalid());
    }
    validate_range(member.bytes(), &witness.span)?;
    budget.charge_serialized(&witness, stop)?;
    if let Some(previous) = result.witnesses.get(&id) {
        if previous != &witness {
            return Err(invalid());
        }
    } else {
        result.witnesses.insert(id, witness);
    }
    Ok(())
}

fn validate_range(bytes: &[u8], span: &XmlSourceSpan) -> ProjectResult<()> {
    let start = usize::try_from(span.byte_start).map_err(|_| invalid())?;
    let end = usize::try_from(span.byte_end).map_err(|_| invalid())?;
    std::str::from_utf8(bytes)
        .map_err(|_| invalid())?
        .get(start..end)
        .ok_or_else(invalid)?;
    Ok(())
}
fn reference_id(reference: &XmlSchemaReference) -> ProjectResult<ContentDigest<CanonicalResult>> {
    crate::identity::canonical_digest(
        "wow-project/xml-classification-catalog-reference/1",
        reference,
        ProjectPhase::View,
    )
}
fn classifier_witness_id(
    witness: &XmlClassificationWitness,
) -> ProjectResult<ContentDigest<CanonicalResult>> {
    crate::identity::canonical_digest(
        "wow-project/xml-classification-catalog-witness/1",
        witness,
        ProjectPhase::View,
    )
}
