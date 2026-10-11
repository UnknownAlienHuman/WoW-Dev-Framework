//! Selected schema-class projection over retained scoped indexes, never source discovery.
mod inclusion;
mod registry;
mod support;

use super::producer_budget::ProducerBudget;
use super::*;
use crate::load::schema::{
    XmlExpandedName, XmlSchemaComponentKind, XmlSchemaComponentState,
    classification::{
        ReviewedXmlClassificationProfile, XmlClassOutcome, XmlClassificationBinding,
        XmlClassificationBudget, XmlClassificationUnknown, XmlDocumentClassification,
        XmlSchemaClass,
    },
};
use std::collections::BTreeSet;
use wow_core::{CanonicalResult, ContentDigest, GenerationContextId, SourceContent};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphLocalAssertion,
    GraphPartitionSnapshot,
};

pub(super) use inclusion::membership;
pub use inclusion::{
    XmlClassificationInclusionManifest, XmlClassificationMembership,
    XmlClassificationPartitionIdentity, XmlClassificationProposalDisposition,
};
pub(super) use registry::{CLASS_KIND, CLASSIFICATION_RELATION, extend as extend_registry};
pub use support::{XmlClassificationCatalogSupport, XmlClassificationCatalogWitness};

// This is the existing combined direct assertion ceiling, not a new allowance.
const MAX_ASSERTIONS: usize = 200_000;
type ScopedKey<'a> = (&'a str, &'a str, Option<&'a str>, &'a str, &'a str);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectScopedXmlClassification {
    pub scope: ProjectXmlFactScope,
    pub document: String,
    pub report: XmlDocumentClassification,
    pub occurrences: Vec<ProjectXmlClassificationOccurrence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlClassificationOccurrence {
    pub occurrence: String,
    pub occurrence_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_proposal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_proposal_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlUnmatchedNameCount {
    pub name: XmlExpandedName,
    pub occurrences: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct ClassificationDerivation {
    pub output: GraphLocalAssertion,
    pub inputs: Vec<GraphLocalAssertion>,
}

/// Graph-generation support differs from the leaf's schema observations.
/// Serialize-only: this report cannot recreate a reviewed profile or native owner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlClassification {
    context_id: GenerationContextId,
    binding: XmlClassificationBinding,
    profile_digest: ContentDigest<CanonicalResult>,
    catalog_support: XmlClassificationCatalogSupport,
    documents: Vec<ProjectScopedXmlClassification>,
    counts_by_class: BTreeMap<XmlSchemaClass, u64>,
    unmatched_names: Vec<XmlUnmatchedNameCount>,
    proposals: Vec<GraphLocalAssertion>,
    derivations: Vec<ClassificationDerivation>,
    digest: ContentDigest<CanonicalResult>,
}

impl ProjectXmlClassification {
    #[must_use]
    pub const fn digest(&self) -> ContentDigest<CanonicalResult> {
        self.digest
    }
    #[must_use]
    pub fn documents(&self) -> &[ProjectScopedXmlClassification] {
        &self.documents
    }
    #[must_use]
    pub const fn binding(&self) -> &XmlClassificationBinding {
        &self.binding
    }
    #[must_use]
    pub const fn profile_digest(&self) -> ContentDigest<CanonicalResult> {
        self.profile_digest
    }
    #[must_use]
    pub fn unmatched_names(&self) -> &[XmlUnmatchedNameCount] {
        &self.unmatched_names
    }
    #[must_use]
    pub fn counts_by_class(&self) -> &BTreeMap<XmlSchemaClass, u64> {
        &self.counts_by_class
    }
    pub const fn catalog_support(&self) -> &XmlClassificationCatalogSupport {
        &self.catalog_support
    }
    pub(super) fn derivation(
        &self,
        output: &GraphLocalAssertion,
    ) -> Option<&[GraphLocalAssertion]> {
        self.derivations
            .binary_search_by(|record| record.output.cmp(output))
            .ok()
            .map(|index| self.derivations[index].inputs.as_slice())
    }
}

pub(super) struct PreparedXmlClassification {
    reviewed_digest: ContentDigest<CanonicalResult>,
    pub(super) catalog_support: XmlClassificationCatalogSupport,
    documents: Vec<ProjectScopedXmlClassification>,
}

/// Prepare native raw support and typed coordinates; catalog coordinates never
/// enter captured Main/package-file support maps.
pub(super) fn prepare(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    reviewed: &ReviewedXmlClassificationProfile,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<PreparedXmlClassification> {
    crate::analyzer::checkpoint(stop)?;
    if project.configuration().project_kind() != ProjectKind::BlizzardUiPlatformSource
        || source.context() != project.snapshot().generation_context()
        || source.project_snapshot_id != project.snapshot_id()
        || source.analyzer_snapshot_id != project.analyzer_snapshot_id()
    {
        return Err(invalid());
    }
    let catalog_support = support::prepare(project, source, reviewed.catalog(), budget, stop)?;
    let mut containment = BTreeMap::new();
    for row in source.xml_containment() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(
            &("xml-classification-containment-index", &row.fact_id),
            stop,
        )?;
        if row.context_id != source.context().context_id()
            || containment
                .insert(key(&row.scope, &row.document, &row.occurrence_id), row)
                .is_some()
        {
            return Err(invalid());
        }
    }
    let mut facts = BTreeMap::<ScopedKey<'_>, Vec<&ProjectXmlFact>>::new();
    let mut fact_ids = BTreeSet::new();
    for fact in source.xml_facts() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("xml-classification-fact-index", &fact.fact_id), stop)?;
        if fact.context_id != source.context().context_id() || !fact_ids.insert(&fact.fact_id) {
            return Err(invalid());
        }
        facts
            .entry(key(&fact.scope, &fact.document, &fact.occurrence_id))
            .or_default()
            .push(fact);
    }
    let mut documents = Vec::new();
    let mut classifier_budget = XmlClassificationBudget::new();
    let mut visited = 0usize;
    let mut visited_facts = 0usize;
    for input in load_inputs::scopes(project, stop)? {
        crate::analyzer::checkpoint(stop)?;
        let plan = input.plan();
        let scope = ProjectXmlFactScope {
            selected_toc: input.qualified_path(plan.selected_toc())?,
            flavor: project
                .configuration()
                .selected_profile()
                .flavor_id()
                .to_owned(),
            package: input.package().map(str::to_owned),
        };
        budget.charge_serialized(&("xml-classification-scope", &scope), stop)?;
        let scope_text = xml_roles::canonical_text(&scope, budget, stop)?;
        for (local, index) in plan.xml_documents() {
            crate::analyzer::checkpoint(stop)?;
            let mapped = input.mapped_source(local)?;
            if index.document() != local || index.source_digest() != mapped.content_digest {
                return Err(invalid());
            }
            let report = reviewed.classify_document(index, &mut classifier_budget, stop)?;
            budget.charge_serialized(&report, stop)?;
            if report.document() != local
                || report.source_digest() != mapped.content_digest
                || report.index_digest() != index.digest()
                || report.profile_digest() != reviewed.digest()
                || report.catalog_digest() != reviewed.catalog().receipt().digest()
                || report.occurrences().len() != index.elements().len()
                || report.coverage().occurrences() != index.elements().len()
            {
                return Err(invalid());
            }
            let mut occurrences = Vec::new();
            for (element, classified) in index.elements().iter().zip(report.occurrences()) {
                crate::analyzer::checkpoint(stop)?;
                let row = containment
                    .get(&key(&scope, &mapped.path, &element.occurrence_id))
                    .ok_or_else(invalid)?;
                let span = xml::source_span(plan, local, &element.span)?;
                if row.document_digest != index.digest()
                    || row.content_digest != mapped.content_digest
                    || row.element_name != element.qualified_name
                    || row.span != span
                    || row.parent_occurrence_id != element.parent_occurrence_id
                    || classified.occurrence() != element.occurrence_id
                    || classified.span() != &element.span
                {
                    return Err(invalid());
                }
                let handle = source
                    .source_handles()
                    .get(&row.source_handle_id)
                    .ok_or_else(invalid)?;
                let evidence = source
                    .evidence()
                    .get(&row.evidence_id)
                    .ok_or_else(invalid)?;
                if project.source_handle(&mapped.path, span, None)? != *handle
                    || *handle.content_digest() != mapped.content_digest
                    || evidence.context_id() != source.context().context_id()
                    || evidence.source_handle_ids() != [row.source_handle_id]
                {
                    return Err(invalid());
                }
                if let Some(rows) = facts.get(&key(&scope, &mapped.path, &element.occurrence_id)) {
                    for fact in rows {
                        crate::analyzer::checkpoint(stop)?;
                        if fact.document_digest != index.digest()
                            || fact.content_digest != mapped.content_digest
                            || fact.element_name != element.qualified_name
                        {
                            return Err(invalid());
                        }
                        visited_facts = visited_facts.checked_add(1).ok_or_else(exhausted)?;
                    }
                }
                let occurrence_key: BTreeMap<Box<str>, GraphProposalValue> = BTreeMap::from([
                    (
                        "scope".into(),
                        GraphProposalValue::String(scope_text.clone()),
                    ),
                    (
                        "document".into(),
                        GraphProposalValue::String(mapped.path.as_str().into()),
                    ),
                    (
                        "document_digest".into(),
                        GraphProposalValue::String(index.digest().canonical().into()),
                    ),
                    (
                        "occurrence".into(),
                        GraphProposalValue::String(element.occurrence_id.as_str().into()),
                    ),
                    (
                        "role".into(),
                        GraphProposalValue::String(xml_roles::role_name(element.role).into()),
                    ),
                    (
                        "source_handle".into(),
                        GraphProposalValue::String(row.source_handle_id.canonical().into()),
                    ),
                ]);
                let id = crate::identity::canonical_id(
                    "xml-role:",
                    "wow-project/platform-xml-role-entity/1",
                    &("xml_source_occurrence", &occurrence_key),
                    ProjectPhase::View,
                )?;
                budget.charge_serialized(
                    &("xml-classification-occurrence", &id, &occurrence_key),
                    stop,
                )?;
                let occurrence = ProjectXmlClassificationOccurrence {
                    occurrence: element.occurrence_id.clone(),
                    occurrence_proposal_id: id.into(),
                    source_handle_id: row.source_handle_id,
                    evidence_id: row.evidence_id,
                    class_proposal_id: None,
                    relation_proposal_id: None,
                };
                budget.charge_serialized(&occurrence, stop)?;
                occurrences.push(occurrence);
                visited = visited
                    .checked_add(1)
                    .filter(|count| *count <= MAX_ASSERTIONS)
                    .ok_or_else(exhausted)?;
            }
            let document = ProjectScopedXmlClassification {
                scope: scope.clone(),
                document: mapped.path,
                report,
                occurrences,
            };
            budget.charge_serialized(&document, stop)?;
            documents.push(document);
        }
    }
    if visited != source.xml_containment().len() || visited_facts != source.xml_facts().len() {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(PreparedXmlClassification {
        reviewed_digest: reviewed.digest(),
        catalog_support,
        documents,
    })
}

/// Called immediately after xml_roles::append in the selected XmlStructure recipe.
pub(super) fn append(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    prepared: PreparedXmlClassification,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<ProjectXmlClassification> {
    crate::analyzer::checkpoint(stop)?;
    let reviewed = project
        .configuration()
        .xml_classification_profile()
        .ok_or_else(invalid)?;
    if reviewed.digest() != prepared.reviewed_digest {
        return Err(invalid());
    }
    let mut native_occurrences = BTreeMap::new();
    for draft in entities
        .iter()
        .filter(|draft| draft.proposal.entity_kind_id() == "xml_source_occurrence")
    {
        crate::analyzer::checkpoint(stop)?;
        if draft.producer != PlatformGraphProducer::XmlStructure
            || draft.proposal.confidence() != GraphConfidence::Proven
        {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "xml-classification-native-occurrence-index",
                draft.proposal.proposal_id(),
            ),
            stop,
        )?;
        if native_occurrences
            .insert(draft.proposal.proposal_id(), &draft.proposal)
            .is_some()
        {
            return Err(invalid());
        }
    }
    for document in &prepared.documents {
        for row in &document.occurrences {
            crate::analyzer::checkpoint(stop)?;
            let proposal = native_occurrences
                .get(row.occurrence_proposal_id.as_str())
                .ok_or_else(invalid)?;
            if proposal.source_handle_ids() != [row.source_handle_id]
                || proposal.evidence_ids() != [row.evidence_id]
                || string(proposal.semantic_key(), "occurrence")? != row.occurrence
                || string(proposal.semantic_key(), "document")? != document.document
                || string(proposal.semantic_key(), "document_digest")?
                    != document.report.index_digest().canonical()
                || proposal.semantic_key().len() != 6
            {
                return Err(invalid());
            }
        }
    }
    drop(native_occurrences);
    let raw_count = source.raw_inventory().ok_or_else(invalid)?.members().len();
    let mut output = Output {
        entities,
        relations,
        budget,
        stop,
        rows: raw_count,
        classes: BTreeMap::new(),
        proposals: Vec::new(),
        derivations: Vec::new(),
    };
    output.rows = output
        .rows
        .checked_add(output.entities.len())
        .and_then(|count| count.checked_add(output.relations.len()))
        .filter(|count| *count <= MAX_ASSERTIONS)
        .ok_or_else(exhausted)?;
    let mut documents = prepared.documents;
    let mut counts_by_class = BTreeMap::new();
    let mut unmatched = BTreeMap::<XmlExpandedName, u64>::new();
    for document in &mut documents {
        for (row, classified) in document
            .occurrences
            .iter_mut()
            .zip(document.report.occurrences())
        {
            crate::analyzer::checkpoint(stop)?;
            // Retain support for failed/ambiguous paths too; never turn it into a class edge.
            let mut supports = BTreeSet::new();
            let mut raw_inputs = BTreeSet::new();
            for proof in classified.proofs() {
                for witness in proof.witnesses() {
                    prepared.catalog_support.resolve_witness(
                        witness,
                        &mut supports,
                        &mut raw_inputs,
                        output.budget,
                        stop,
                    )?;
                }
            }
            for blocker in classified.blockers() {
                for witness in blocker.witnesses() {
                    prepared.catalog_support.resolve_witness(
                        witness,
                        &mut supports,
                        &mut raw_inputs,
                        output.budget,
                        stop,
                    )?;
                }
            }
            match classified.outcome() {
                XmlClassOutcome::Classified(class) => {
                    let [declaration] = classified.candidates() else {
                        return Err(invalid());
                    };
                    let native = prepared.catalog_support.component(*declaration)?;
                    if native.component_kind != XmlSchemaComponentKind::GlobalElement
                        || native.component_state != XmlSchemaComponentState::Observed
                        || classified.proofs().is_empty()
                        || classified
                            .proofs()
                            .iter()
                            .any(|proof| proof.class() != class)
                    {
                        return Err(invalid());
                    }
                    supports.insert((native.source_handle_id, native.evidence_id));
                    raw_inputs.insert(local_entity(&native.raw_member_proposal_id));
                    let class_id = output.class(
                        reviewed,
                        *declaration,
                        class,
                        classified.proofs(),
                        &supports,
                        &raw_inputs,
                    )?;
                    supports.insert((row.source_handle_id, row.evidence_id));
                    raw_inputs.insert(local_entity(&row.occurrence_proposal_id));
                    raw_inputs.insert(local_entity(&class_id));
                    let relation_id = output.classifies(
                        &row.occurrence_proposal_id,
                        &class_id,
                        &supports,
                        raw_inputs,
                    )?;
                    row.class_proposal_id = Some(class_id.into());
                    row.relation_proposal_id = Some(relation_id.into());
                    let count = counts_by_class.entry(class).or_insert(0_u64);
                    *count = count.checked_add(1).ok_or_else(exhausted)?;
                }
                XmlClassOutcome::Unknown(XmlClassificationUnknown::MissingGlobalDeclaration) => {
                    let name = classified.qname().name().ok_or_else(invalid)?;
                    output
                        .budget
                        .charge_serialized(&("xml-classification-unmatched-name", name), stop)?;
                    let count = unmatched.entry(name.clone()).or_insert(0);
                    *count = count.checked_add(1).ok_or_else(exhausted)?;
                }
                XmlClassOutcome::Unknown(_) | XmlClassOutcome::Conflict(_) => (),
            }
        }
    }
    let mut unmatched_names = Vec::new();
    for (name, occurrences) in unmatched {
        crate::analyzer::checkpoint(stop)?;
        let row = XmlUnmatchedNameCount { name, occurrences };
        output.budget.charge_serialized(&row, stop)?;
        unmatched_names.push(row);
    }
    output.proposals.sort();
    output
        .derivations
        .sort_by(|left, right| left.output.cmp(&right.output));
    let identity = (
        source.context().context_id(),
        reviewed.binding(),
        reviewed.digest(),
        &prepared.catalog_support,
        &documents,
        &counts_by_class,
        &unmatched_names,
        &output.proposals,
        &output.derivations,
    );
    output.budget.charge_serialized(&identity, stop)?;
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-classification-projection/1",
        &identity,
        ProjectPhase::View,
    )?;
    let result = ProjectXmlClassification {
        context_id: source.context().context_id(),
        binding: reviewed.binding().clone(),
        profile_digest: reviewed.digest(),
        catalog_support: prepared.catalog_support,
        documents,
        counts_by_class,
        unmatched_names,
        proposals: output.proposals,
        derivations: output.derivations,
        digest,
    };
    output.budget.charge_serialized(&result, stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(result)
}

struct Output<'a> {
    entities: &'a mut Vec<EntityDraft>,
    relations: &'a mut Vec<RelationDraft>,
    budget: &'a mut ProducerBudget,
    stop: &'a AtomicBool,
    rows: usize,
    classes: BTreeMap<Box<str>, usize>,
    proposals: Vec<GraphLocalAssertion>,
    derivations: Vec<ClassificationDerivation>,
}
impl Output<'_> {
    fn next(&mut self) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        self.rows = self
            .rows
            .checked_add(1)
            .filter(|count| *count <= MAX_ASSERTIONS)
            .ok_or_else(exhausted)?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn class(
        &mut self,
        profile: &ReviewedXmlClassificationProfile,
        declaration: ContentDigest<CanonicalResult>,
        class: XmlSchemaClass,
        proofs: &[crate::load::schema::classification::XmlClassProof],
        supports: &BTreeSet<(StableHandleId, EvidenceId)>,
        inputs: &BTreeSet<GraphLocalAssertion>,
    ) -> ProjectResult<Box<str>> {
        let proof_digest = crate::identity::canonical_digest(
            "wow-project/xml-schema-class-proof/1",
            &proofs,
            ProjectPhase::View,
        )?;
        let key = BTreeMap::from([
            (
                "catalog_digest".into(),
                GraphProposalValue::String(profile.binding().catalog_digest.canonical().into()),
            ),
            (
                "profile_digest".into(),
                GraphProposalValue::String(profile.digest().canonical().into()),
            ),
            (
                "declaration".into(),
                GraphProposalValue::String(declaration.canonical().into()),
            ),
            (
                "class".into(),
                GraphProposalValue::String(xml_roles::canonical_text(
                    &class,
                    self.budget,
                    self.stop,
                )?),
            ),
            (
                "proof_digest".into(),
                GraphProposalValue::String(proof_digest.canonical().into()),
            ),
        ]);
        self.budget
            .charge_serialized(&(CLASS_KIND, &key, supports, inputs), self.stop)?;
        let id = crate::identity::canonical_id(
            "xml-schema-class:",
            "wow-project/xml-schema-class-entity/1",
            &(CLASS_KIND, &key),
            ProjectPhase::View,
        )?;
        let proposal = GraphEntityProposal::new(
            id.clone(),
            CLASS_KIND,
            key,
            GraphConfidence::Derived,
            supports.iter().map(|(handle, _)| *handle).collect(),
            supports.iter().map(|(_, evidence)| *evidence).collect(),
            Vec::new(),
        )
        .map_err(|_| invalid())?;
        if let Some(index) = self.classes.get(&id) {
            if self.entities.get(*index).ok_or_else(invalid)?.proposal != proposal {
                return Err(invalid());
            }
            return Ok(id);
        }
        self.next()?;
        let draft = EntityDraft::new(PlatformGraphProducer::XmlStructure, proposal);
        self.budget.charge_serialized(&draft, self.stop)?;
        let output = local_entity(&id);
        self.record(output.clone(), inputs.iter().cloned().collect())?;
        self.budget
            .charge_serialized(&("xml-classification-class-address", &id), self.stop)?;
        self.classes.insert(id.clone(), self.entities.len());
        self.entities.push(draft);
        self.proposals.push(output);
        Ok(id)
    }

    fn classifies(
        &mut self,
        occurrence: &str,
        class: &str,
        supports: &BTreeSet<(StableHandleId, EvidenceId)>,
        inputs: BTreeSet<GraphLocalAssertion>,
    ) -> ProjectResult<Box<str>> {
        self.next()?;
        self.budget.charge_serialized(
            &(CLASSIFICATION_RELATION, occurrence, class, supports),
            self.stop,
        )?;
        let id = crate::identity::canonical_id(
            "xml-schema-class-edge:",
            "wow-project/xml-schema-class-relation/1",
            &(CLASSIFICATION_RELATION, occurrence, class),
            ProjectPhase::View,
        )?;
        let draft = RelationDraft::new(
            PlatformGraphProducer::XmlStructure,
            id.clone(),
            CLASSIFICATION_RELATION,
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed(occurrence.into()),
                target: GraphProposalEndpoint::Proposed(class.into()),
                confidence: GraphConfidence::Derived,
                source_handle_ids: supports
                    .iter()
                    .map(|(handle, _)| *handle)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect(),
                evidence_ids: supports.iter().map(|(_, evidence)| *evidence).collect(),
                coverage_ids: Vec::new(),
            },
        )
        .map_err(|_| invalid())?;
        self.budget.charge_serialized(&draft, self.stop)?;
        let output = GraphLocalAssertion {
            kind: GraphAssertionKind::Relation,
            proposal_id: id.clone(),
        };
        self.record(output.clone(), inputs.into_iter().collect())?;
        self.relations.push(draft);
        self.proposals.push(output);
        Ok(id)
    }

    fn record(
        &mut self,
        output: GraphLocalAssertion,
        inputs: Vec<GraphLocalAssertion>,
    ) -> ProjectResult<()> {
        let record = ClassificationDerivation { output, inputs };
        self.budget.charge_serialized(&record, self.stop)?;
        crate::analyzer::checkpoint(self.stop)?;
        self.derivations.push(record);
        Ok(())
    }
}

fn local_entity(id: &str) -> GraphLocalAssertion {
    GraphLocalAssertion {
        kind: GraphAssertionKind::Entity,
        proposal_id: id.into(),
    }
}
fn key<'a>(
    scope: &'a ProjectXmlFactScope,
    document: &'a str,
    occurrence: &'a str,
) -> ScopedKey<'a> {
    (
        &scope.selected_toc,
        &scope.flavor,
        scope.package.as_deref(),
        document,
        occurrence,
    )
}
fn string<'a>(
    key: &'a BTreeMap<Box<str>, GraphProposalValue>,
    name: &str,
) -> ProjectResult<&'a str> {
    let Some(GraphProposalValue::String(value)) = key.get(name) else {
        return Err(invalid());
    };
    Ok(value)
}
