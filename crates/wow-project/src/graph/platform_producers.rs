//! Native stage plans over one retained project projection, never a wire owner.
use super::*;
use diagnostics::capture_scratch;
use producer_budget::ProducerBudget;
use std::collections::BTreeSet;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphEvidenceCatalog,
    GraphLocalAssertion, GraphPartitionSnapshot, GraphProducerLookup, GraphSnapshot,
};

pub const PLATFORM_DIRECT_GRAPH_PROFILE: &str = "wow-project/platform-direct-producers/1";
pub const PLATFORM_CAPABILITIES_GRAPH_PROFILE: &str = "wow-project/platform-capabilities/1";

#[derive(Clone, Copy)]
pub(super) enum DirectRecipe {
    Original,
    Capabilities(crate::PlatformGraphCapabilities),
}
impl DirectRecipe {
    const fn capabilities(self) -> Option<crate::PlatformGraphCapabilities> {
        match self {
            Self::Original => None,
            Self::Capabilities(capabilities) => Some(capabilities),
        }
    }
    const fn profile(self) -> &'static str {
        match self {
            Self::Original => PLATFORM_DIRECT_GRAPH_PROFILE,
            Self::Capabilities(_) => PLATFORM_CAPABILITIES_GRAPH_PROFILE,
        }
    }
    fn producer_version(self, producer: PlatformGraphProducer) -> Option<String> {
        match self {
            Self::Original => None,
            Self::Capabilities(capabilities) => {
                let mut version = format!("{}:v1", producer.partition_id());
                for capability in capabilities.iter() {
                    let affected = match capability {
                        crate::PlatformGraphCapability::InventorySpansV1 => {
                            producer == PlatformGraphProducer::Inventory
                        }
                        crate::PlatformGraphCapability::StructuralRolesV1 => matches!(
                            producer,
                            PlatformGraphProducer::Inventory
                                | PlatformGraphProducer::TocLoad
                                | PlatformGraphProducer::XmlStructure
                        ),
                        crate::PlatformGraphCapability::XmlSourceMapsV1
                        | crate::PlatformGraphCapability::XmlSchemaClassificationV1 => matches!(
                            producer,
                            PlatformGraphProducer::Inventory | PlatformGraphProducer::XmlStructure
                        ),
                    };
                    if affected {
                        version.push(':');
                        version.push_str(capability.id());
                    }
                }
                Some(version)
            }
        }
    }
    pub(super) const fn inventory_spans(self) -> bool {
        matches!(self, Self::Capabilities(capabilities) if capabilities.contains(crate::PlatformGraphCapability::InventorySpansV1))
    }
    pub(super) const fn structural_roles(self) -> bool {
        matches!(self, Self::Capabilities(capabilities) if capabilities.contains(crate::PlatformGraphCapability::StructuralRolesV1))
    }
    pub(super) const fn xml_source_maps(self) -> bool {
        matches!(self, Self::Capabilities(capabilities) if capabilities.contains(crate::PlatformGraphCapability::XmlSourceMapsV1))
    }
    pub(super) const fn xml_schema_classification(self) -> bool {
        matches!(self, Self::Capabilities(capabilities) if capabilities.contains(crate::PlatformGraphCapability::XmlSchemaClassificationV1))
    }
}
const ORDER: [PlatformGraphProducer; 4] = [
    PlatformGraphProducer::Inventory,
    PlatformGraphProducer::TocLoad,
    PlatformGraphProducer::AnalyzerStructure,
    PlatformGraphProducer::XmlStructure,
];
const MAX_DIRECT_ASSERTIONS: usize = 200_000;

/// Constructed only from an immutable native platform project. Each stage waits
/// for exact native admission of its predecessor; retries retain the same batch.
pub struct PlatformGraphProposalPlan<'a> {
    recipe: DirectRecipe,
    inventory_span_omissions: Option<usize>,
    xml_source_maps: Option<XmlSourceMapSummary>,
    project: &'a ProjectView,
    collected: CollectedSourceGraph,
    scope: GraphAssertionRecordScope,
    foundation: GraphSnapshot,
    stages: Vec<PlatformGraphProducerProposals>,
    budget: ProducerBudget,
}

#[derive(Clone, Serialize)]
pub struct PlatformGraphProducerProposals {
    #[serde(skip_serializing_if = "Option::is_none")]
    producer_version: Option<String>,
    producer: PlatformGraphProducer,
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
}
impl PlatformGraphProducerProposals {
    #[must_use]
    pub const fn producer(&self) -> PlatformGraphProducer {
        self.producer
    }
    #[must_use]
    pub fn producer_version(&self) -> &str {
        self.producer_version.as_deref().unwrap_or("1")
    }
    pub fn into_parts(self) -> (GraphProposalBatch, Vec<GraphCoverageRecord>) {
        (self.batch, self.coverage)
    }
}

/// Exact addresses and source support bound to the original held native owners.
/// Serialized reports cannot construct this capability.
pub struct PlatformGraphProvenance<'a> {
    recipe: DirectRecipe,
    inventory_span_omissions: Option<usize>,
    xml_source_maps: Option<XmlSourceMapSummary>,
    project: &'a ProjectView,
    graph: &'a GraphPartitionSnapshot,
    xml_classification_membership: Option<xml_classification::XmlClassificationMembership>,
    scope: GraphAssertionRecordScope,
    source: ProjectGraphProvenance,
    addresses: BTreeMap<GraphLocalAssertion, GraphAssertionRef>,
    evidence: GraphEvidenceCatalog,
}
impl PlatformGraphProvenance<'_> {
    #[must_use]
    pub const fn capabilities(&self) -> Option<crate::PlatformGraphCapabilities> {
        self.recipe.capabilities()
    }
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.recipe.profile()
    }
    /// Count of retained Unknown spans omitted by native /2; /1 has no span role.
    #[must_use]
    pub const fn inventory_span_omissions(&self) -> Option<usize> {
        self.inventory_span_omissions
    }
    /// Mapping metadata belongs only to the native /4 recipe.
    #[must_use]
    pub const fn xml_source_maps(&self) -> Option<&XmlSourceMapSummary> {
        self.xml_source_maps.as_ref()
    }
    #[must_use]
    pub const fn project(&self) -> &ProjectView {
        self.project
    }
    #[must_use]
    pub const fn graph(&self) -> &GraphPartitionSnapshot {
        self.graph
    }
    #[must_use]
    pub const fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    #[must_use]
    pub const fn source(&self) -> &ProjectGraphProvenance {
        &self.source
    }
    /// Consumes native authority and returns the same owned source report.
    /// The returned report cannot recreate the native project/graph capability.
    #[must_use]
    pub fn into_source(self) -> ProjectGraphProvenance {
        self.source
    }
    /// Move the descriptive admission handoff after all native consumers finish.
    #[must_use]
    pub fn into_source_and_xml_classification_membership(
        self,
    ) -> (ProjectGraphProvenance, Option<XmlClassificationMembership>) {
        (self.source, self.xml_classification_membership)
    }
    #[must_use]
    pub fn assertion(&self, assertion: &GraphLocalAssertion) -> Option<&GraphAssertionRef> {
        self.addresses.get(assertion)
    }
    #[must_use]
    pub fn xml_classification_inclusion(&self) -> Option<XmlClassificationInclusionManifest<'_>> {
        Some(XmlClassificationInclusionManifest::new(
            self.source.xml_classification()?,
            self.xml_classification_membership.as_ref()?,
        ))
    }
    #[must_use]
    pub const fn evidence_catalog(&self) -> &GraphEvidenceCatalog {
        &self.evidence
    }
}

pub fn build_platform_graph_proposal_plan<'a>(
    project: &'a ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<PlatformGraphProposalPlan<'a>> {
    build_platform_graph_proposal_plan_diagnosed(
        project,
        stop,
        &mut SourceGraphDiagnostics::default(),
    )
}

/// Diagnose this operation without changing admission, configuration or bytes.
pub fn build_platform_graph_proposal_plan_diagnosed<'a>(
    project: &'a ProjectView,
    stop: &AtomicBool,
    diagnostics: &mut SourceGraphDiagnostics,
) -> ProjectResult<PlatformGraphProposalPlan<'a>> {
    diagnostics.reset();
    let recipe = match project.configuration().platform_graph_profile() {
        Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(capabilities)) => {
            DirectRecipe::Capabilities(capabilities)
        }
        _ => DirectRecipe::Original,
    };
    build_plan(project, recipe, stop, diagnostics)
}

/// Native preparation with an explicit exact set. A selected application cannot
/// substitute a different set. Legacy native inspection retains its own registry.
pub fn build_platform_graph_proposal_plan_for_capabilities<'a>(
    project: &'a ProjectView,
    capabilities: crate::PlatformGraphCapabilities,
    stop: &AtomicBool,
) -> ProjectResult<PlatformGraphProposalPlan<'a>> {
    crate::analyzer::checkpoint(stop)?;
    if let Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(selected)) =
        project.configuration().platform_graph_profile()
        && selected != capabilities
    {
        return Err(invalid());
    }
    build_plan(
        project,
        DirectRecipe::Capabilities(capabilities),
        stop,
        &mut SourceGraphDiagnostics::default(),
    )
}

fn build_plan<'a>(
    project: &'a ProjectView,
    recipe: DirectRecipe,
    stop: &AtomicBool,
    diagnostics: &mut SourceGraphDiagnostics,
) -> ProjectResult<PlatformGraphProposalPlan<'a>> {
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if config.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || config.platform_graph_profile().is_none()
        || (recipe.structural_roles() && !raw_inventory::selected(config))
    {
        return Err(ProjectError::new(
            ProjectErrorCode::DeferredCapability,
            ProjectPhase::View,
            "direct platform stages require a selected genuine platform project",
        ));
    }
    let mut collected =
        collect_source_graph_proposals_for_recipe(project, recipe, stop, diagnostics)?;
    if recipe.xml_source_maps() || recipe.xml_schema_classification() {
        if collected.provenance.profile != source_graph_profile(config) {
            return Err(invalid());
        }
        // New piece support belongs to this native report, never source /22.
        collected.provenance.profile = recipe.profile();
    }
    if collected
        .entities
        .len()
        .saturating_add(collected.relations.len())
        > MAX_DIRECT_ASSERTIONS
    {
        return Err(resource_exhausted(
            "direct assertions",
            collected.entities.len() as u128 + collected.relations.len() as u128,
            MAX_DIRECT_ASSERTIONS,
        ));
    }
    // Refine the collector's provisional tally with complete borrowed metadata.
    // Shared raw/provenance bodies are counted once, under the same operation cap.
    let mut exact = ProducerBudget::for_configuration(config, 0)?;
    diagnostics.measure(
        DiagnosticPhase::CollectorEncoding,
        &mut exact,
        collected
            .entities
            .len()
            .checked_add(collected.relations.len()),
        CountingMethod::SerializedBytes,
        |budget| raw_inventory::charge_serialized(budget, &collected, stop),
    )?;
    let mut budget =
        ProducerBudget::for_configuration(config, exact.used().max(collected.text_bytes.used()))?;
    diagnostics.measure(
        DiagnosticPhase::DirectPreparation,
        &mut budget,
        None,
        CountingMethod::SerializedBytes,
        |budget| {
            // New source pieces must enter the native catalog before Inventory spans.
            let prepared_maps = if recipe.xml_source_maps() {
                Some(xml_source_maps::prepare(
                    project,
                    &mut collected.provenance,
                    budget,
                    stop,
                )?)
            } else {
                None
            };
            let prepared_classification = if recipe.xml_schema_classification() {
                let reviewed = config.xml_classification_profile().ok_or_else(invalid)?;
                let prepared = xml_classification::prepare(
                    project,
                    &collected.provenance,
                    reviewed,
                    budget,
                    stop,
                )?;
                Some(prepared)
            } else {
                None
            };
            let inventory_span_omissions = if recipe.inventory_spans() {
                let omitted = inventory_roles::append_spans(
                    project,
                    &collected.provenance,
                    &mut collected.entities,
                    &mut collected.relations,
                    budget,
                    stop,
                )?;
                if collected
                    .entities
                    .len()
                    .checked_add(
                        collected
                            .inventory_batch
                            .as_ref()
                            .map_or(0, |batch| batch.entity_proposals().len()),
                    )
                    .ok_or_else(exhausted)?
                    > MAX_NODES
                    || collected.relations.len() > MAX_EDGES
                    || collected
                        .entities
                        .len()
                        .checked_add(collected.relations.len())
                        .ok_or_else(exhausted)?
                        > MAX_DIRECT_ASSERTIONS
                {
                    return Err(direct_counts_exhausted(&collected, false));
                }
                let mut reasons = vec!["source_graph.captured_known_source_spans_only".into()];
                if omitted > 0 {
                    reasons.push("source_graph.unknown_source_spans_omitted".into());
                }
                let coverage = GraphCoverageRecord::new(
                    GraphRelationKind::Contains,
                    GraphCoverageState::Partial,
                    false,
                    reasons,
                    collected.limits,
                )
                .map_err(graph_error)?;
                budget.charge_serialized(&coverage, stop)?;
                budget.charge_serialized(&omitted, stop)?;
                collected.coverage.push(coverage);
                Some(omitted)
            } else {
                None
            };
            let mut xml_source_maps = None;
            if recipe.structural_roles() {
                inventory_project::append_project(
                    project,
                    &collected.provenance,
                    &mut collected.entities,
                    &mut collected.relations,
                    budget,
                    stop,
                )?;
                toc_roles::append(
                    project,
                    &collected.provenance,
                    &mut collected.entities,
                    &mut collected.relations,
                    budget,
                    stop,
                )?;
                xml_roles::append(
                    project,
                    &collected.provenance,
                    &mut collected.entities,
                    &mut collected.relations,
                    budget,
                    stop,
                )?;
                if let Some(prepared) = prepared_classification {
                    collected.provenance.xml_classification = Some(xml_classification::append(
                        project,
                        &collected.provenance,
                        prepared,
                        &mut collected.entities,
                        &mut collected.relations,
                        budget,
                        stop,
                    )?);
                    let coverage = GraphCoverageRecord::new(
                        GraphRelationKind::HasSchemaClass,
                        GraphCoverageState::Partial,
                        false,
                        vec![
                            "source_graph.selected_schema_policy_only".into(),
                            "source_graph.schema_unknown_conflict_and_catalog_issues_retained"
                                .into(),
                            "source_graph.runtime_class_not_evaluated".into(),
                        ],
                        collected.limits,
                    )
                    .map_err(graph_error)?;
                    budget.charge_serialized(&coverage, stop)?;
                    collected.coverage.push(coverage);
                }
                if let Some(prepared) = prepared_maps {
                    xml_source_maps = Some(xml_source_maps::append(
                        project,
                        &collected.provenance,
                        prepared,
                        &mut collected.entities,
                        &mut collected.relations,
                        budget,
                        stop,
                    )?);
                }
                let raw_count = collected
                    .inventory_batch
                    .as_ref()
                    .ok_or_else(invalid)?
                    .entity_proposals()
                    .len();
                let all_entities = collected
                    .entities
                    .len()
                    .checked_add(raw_count)
                    .ok_or_else(exhausted)?;
                if all_entities > MAX_NODES
                    || collected.relations.len() > MAX_EDGES
                    || all_entities
                        .checked_add(collected.relations.len())
                        .ok_or_else(exhausted)?
                        > MAX_DIRECT_ASSERTIONS
                {
                    return Err(direct_counts_exhausted(&collected, true));
                }
                let contains = collected
                    .coverage
                    .iter_mut()
                    .find(|record| record.relation() == GraphRelationKind::Contains)
                    .ok_or_else(invalid)?;
                let mut reasons = vec![
                    "source_graph.native_captured_spans_and_declared_inventory_membership_only"
                        .into(),
                    "source_graph.native_selected_toc_and_lexical_xml_containment_only".into(),
                    "source_graph.original_inventory_omissions_and_span_omissions_retained".into(),
                ];
                if recipe.xml_source_maps() {
                    reasons.push("source_graph.native_xml_map_segment_containment_only".into());
                }
                let coverage = GraphCoverageRecord::new(
                    GraphRelationKind::Contains,
                    GraphCoverageState::Partial,
                    false,
                    reasons,
                    collected.limits,
                )
                .map_err(graph_error)?;
                budget.charge_serialized(&coverage, stop)?;
                *contains = coverage;
                for (relation, blocker) in [
                    (
                        GraphRelationKind::Defines,
                        "source_graph.native_selected_toc_definitions_only",
                    ),
                    (
                        GraphRelationKind::LoadsBefore,
                        "source_graph.native_toc_lexical_order_is_not_runtime_execution_order",
                    ),
                ] {
                    let coverage = GraphCoverageRecord::new(
                        relation,
                        GraphCoverageState::Partial,
                        false,
                        vec![blocker.into()],
                        collected.limits,
                    )
                    .map_err(graph_error)?;
                    budget.charge_serialized(&coverage, stop)?;
                    collected.coverage.push(coverage);
                }
                if recipe.xml_source_maps() {
                    let owns = collected
                        .coverage
                        .iter_mut()
                        .find(|record| record.relation() == GraphRelationKind::Owns)
                        .ok_or_else(invalid)?;
                    let mut reasons = owns.blocker_ids().to_vec();
                    reasons.push("source_graph.native_xml_virtual_mapping_only".into());
                    reasons.push("source_graph.xml_virtual_receiver_dispatch_not_evaluated".into());
                    let coverage = GraphCoverageRecord::new(
                        GraphRelationKind::Owns,
                        GraphCoverageState::Partial,
                        false,
                        reasons,
                        collected.limits,
                    )
                    .map_err(graph_error)?;
                    budget.charge_serialized(&coverage, stop)?;
                    *owns = coverage;
                }
            }
            let scope = GraphAssertionRecordScope {
                universe: collected.universe.clone(),
                generation: collected.generation.clone(),
                source_context_id: collected.provenance.context.context_id(),
            };
            let foundation = GraphSnapshot::build(
                collected.universe.clone(),
                collected.generation.clone(),
                collected.limits,
                Vec::new(),
                Vec::new(),
                collected.coverage.clone(),
            )
            .map_err(graph_error)?;
            budget.charge_serialized(&scope, stop)?;
            budget.charge_serialized(&foundation, stop)?;
            let mut owners = BTreeMap::<Box<str>, PlatformGraphProducer>::new();
            let mut keys = BTreeSet::new();
            if recipe.structural_roles() {
                let raw = collected.inventory_batch.as_ref().ok_or_else(invalid)?;
                for proposal in raw.entity_proposals() {
                    crate::analyzer::checkpoint(stop)?;
                    budget.charge_serialized(
                        &(proposal.proposal_id(), PlatformGraphProducer::Inventory),
                        stop,
                    )?;
                    if owners
                        .insert(
                            proposal.proposal_id().into(),
                            PlatformGraphProducer::Inventory,
                        )
                        .is_some()
                    {
                        return Err(invalid());
                    }
                }
            }
            for draft in &collected.entities {
                crate::analyzer::checkpoint(stop)?;
                if !entity_permission(recipe, draft.producer, draft.proposal.entity_kind_id())
                    || !keys.insert(local(
                        GraphAssertionKind::Entity,
                        draft.proposal.proposal_id(),
                    ))
                    || owners
                        .insert(draft.proposal.proposal_id().into(), draft.producer)
                        .is_some()
                {
                    return Err(invalid());
                }
            }
            for draft in &collected.relations {
                crate::analyzer::checkpoint(stop)?;
                if !relation_permission(recipe, draft.producer, &draft.relation_kind_id)
                    || !keys.insert(local(GraphAssertionKind::Relation, &draft.proposal_id))
                {
                    return Err(invalid());
                }
                for endpoint in [&draft.input.source, &draft.input.target] {
                    let GraphProposalEndpoint::Proposed(id) = endpoint else {
                        return Err(invalid());
                    };
                    let owner = owners.get(id).ok_or_else(invalid)?;
                    if owner > &draft.producer {
                        return Err(invalid());
                    }
                }
                if draft.relation_kind_id.as_ref() == "source_declaration_owns" {
                    let GraphProposalEndpoint::Proposed(target) = &draft.input.target else {
                        return Err(invalid());
                    };
                    if owners.get(target) != Some(&draft.producer) {
                        return Err(invalid());
                    }
                }
            }
            crate::analyzer::checkpoint(stop)?;
            Ok(PlatformGraphProposalPlan {
                recipe,
                inventory_span_omissions,
                xml_source_maps,
                project,
                collected,
                scope,
                foundation,
                stages: Vec::new(),
                budget: *budget,
            })
        },
    )
}

impl<'a> PlatformGraphProposalPlan<'a> {
    #[must_use]
    pub const fn capabilities(&self) -> Option<crate::PlatformGraphCapabilities> {
        self.recipe.capabilities()
    }
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.recipe.profile()
    }
    /// Count of retained Unknown spans omitted by native /2; /1 has no span role.
    #[must_use]
    pub const fn inventory_span_omissions(&self) -> Option<usize> {
        self.inventory_span_omissions
    }
    /// Mapping metadata belongs only to the native /4 recipe.
    #[must_use]
    pub const fn xml_source_maps(&self) -> Option<&XmlSourceMapSummary> {
        self.xml_source_maps.as_ref()
    }
    #[must_use]
    pub fn registry(&self) -> &GraphRegistryBundle {
        &self.collected.registry
    }
    #[must_use]
    pub const fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }
    #[must_use]
    pub const fn limits(&self) -> GraphLimits {
        self.collected.limits
    }
    #[must_use]
    pub const fn foundation(&self) -> &GraphSnapshot {
        &self.foundation
    }
    #[must_use]
    pub fn raw_inventory_batch(&self) -> Option<&GraphProposalBatch> {
        self.collected.inventory_batch.as_ref()
    }
    #[must_use]
    pub const fn raw_inventory_producer_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    #[must_use]
    pub const fn producer_order(&self) -> &'static [PlatformGraphProducer] {
        &ORDER
    }

    pub fn build_stage(
        &mut self,
        producer: PlatformGraphProducer,
        owner: &GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<PlatformGraphProducerProposals> {
        self.build_stage_diagnosed(
            producer,
            owner,
            stop,
            &mut SourceGraphDiagnostics::default(),
        )
    }

    /// Observe one fixed producer slot without changing reservation or retry input.
    /// Repeated calls replace that slot; no per-charge or per-retry trace is retained.
    pub fn build_stage_diagnosed(
        &mut self,
        producer: PlatformGraphProducer,
        owner: &GraphPartitionSnapshot,
        stop: &AtomicBool,
        diagnostics: &mut SourceGraphDiagnostics,
    ) -> ProjectResult<PlatformGraphProducerProposals> {
        let phase = match producer {
            PlatformGraphProducer::Inventory => DiagnosticPhase::InventoryStage,
            PlatformGraphProducer::TocLoad => DiagnosticPhase::TocLoadStage,
            PlatformGraphProducer::AnalyzerStructure => DiagnosticPhase::AnalyzerStructureStage,
            PlatformGraphProducer::XmlStructure => DiagnosticPhase::XmlStructureStage,
        };
        let mut observation = self.budget;
        let mut completed = None;
        let result = diagnostics.measure(
            phase,
            &mut observation,
            None,
            CountingMethod::SerializedBytes,
            |observation| {
                self.build_stage_observed(producer, owner, stop, observation, &mut completed)
            },
        );
        if let Some(count) = completed {
            diagnostics.completed_records(phase, count);
        }
        result
    }

    fn build_stage_observed(
        &mut self,
        producer: PlatformGraphProducer,
        owner: &GraphPartitionSnapshot,
        stop: &AtomicBool,
        observation: &mut ProducerBudget,
        completed: &mut Option<usize>,
    ) -> ProjectResult<PlatformGraphProducerProposals> {
        crate::analyzer::checkpoint(stop)?;
        let index = ORDER
            .iter()
            .position(|candidate| *candidate == producer)
            .ok_or_else(invalid)?;
        if index + 1 == self.stages.len() {
            self.validate_predecessor(owner, index, true, stop)?;
            let mut retry_budget = self.budget;
            capture_scratch(observation, &mut retry_budget, |budget| {
                budget.charge_serialized(&self.stages[index], stop)
            })?;
            let retry = self.stages[index].clone();
            crate::analyzer::checkpoint(stop)?;
            return Ok(retry);
        }
        if index != self.stages.len() {
            return Err(invalid());
        }
        self.validate_predecessor(owner, index, false, stop)?;
        let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
        let mut addresses = BTreeMap::new();
        let mut address_budget = self.budget;
        if self.recipe.structural_roles() {
            let raw = self
                .collected
                .inventory_batch
                .as_ref()
                .ok_or_else(invalid)?;
            for proposal in raw.entity_proposals() {
                crate::analyzer::checkpoint(stop)?;
                let reference = producer_reference(
                    raw,
                    local(GraphAssertionKind::Entity, proposal.proposal_id()),
                );
                let resolved = lookup
                    .entity(&self.scope, &reference, stop)
                    .map_err(graph_error)?;
                if resolved.proposal() != proposal {
                    return Err(invalid());
                }
                capture_scratch(observation, &mut address_budget, |budget| {
                    budget.charge_serialized(&(&reference, proposal.proposal_id()), stop)
                })?;
                if addresses
                    .insert(proposal.proposal_id().into(), resolved.reference())
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        for draft in &self.collected.entities {
            crate::analyzer::checkpoint(stop)?;
            if draft.producer > producer {
                continue;
            }
            let assertion = local(GraphAssertionKind::Entity, draft.proposal.proposal_id());
            let reference = if draft.producer == producer {
                GraphAssertionRef::Local { assertion }
            } else {
                let stage = self
                    .stages
                    .iter()
                    .find(|stage| stage.producer == draft.producer)
                    .ok_or_else(invalid)?;
                let reference = producer_reference(&stage.batch, assertion);
                let resolved = lookup
                    .entity(&self.scope, &reference, stop)
                    .map_err(graph_error)?;
                if resolved.proposal() != &draft.proposal {
                    return Err(invalid());
                }
                resolved.reference()
            };
            addresses.insert(draft.proposal.proposal_id().into(), reference);
        }
        let mut entities = Vec::new();
        let mut relations = Vec::new();
        let mut original_relations = Vec::new();
        let base = address_budget;
        let mut provisional = base;
        // This prefix counts copied entity/relation drafts, not address entries
        // or derivation records. Retry/predecessor-only paths leave it unknown.
        *completed = Some(0);
        for draft in &self.collected.entities {
            crate::analyzer::checkpoint(stop)?;
            if draft.producer == producer {
                capture_scratch(observation, &mut provisional, |budget| {
                    budget.charge_serialized(&draft.proposal, stop)
                })?;
                entities.push(draft.proposal.clone());
                *completed = Some(entities.len());
            }
        }
        for draft in &self.collected.relations {
            crate::analyzer::checkpoint(stop)?;
            if draft.producer != producer {
                continue;
            }
            capture_scratch(observation, &mut provisional, |budget| {
                budget.charge_serialized(&draft.input, stop)
            })?;
            let mut input = draft.input.clone();
            input.source = endpoint(&input.source, &addresses, &lookup, &self.scope, stop)?;
            input.target = endpoint(&input.target, &addresses, &lookup, &self.scope, stop)?;
            let relation = GraphRelationProposal::new(
                draft.proposal_id.clone(),
                draft.relation_kind_id.clone(),
                input,
            )
            .map_err(graph_error)?;
            capture_scratch(observation, &mut provisional, |budget| {
                budget.charge_serialized(&relation, stop)
            })?;
            relations.push(relation);
            original_relations.push(draft);
            // Combined drafts already passed MAX_DIRECT_ASSERTIONS preflight.
            *completed = Some(entities.len() + relations.len());
        }
        let records = capture_scratch(observation, &mut provisional, |budget| {
            producer_derivations::records(
                &self.collected.provenance,
                &self.scope,
                &entities,
                &original_relations,
                &addresses,
                budget,
                stop,
            )
        })?;
        let batch = GraphProposalBatch::build(
            self.collected.registry.bundle_id(),
            self.collected.registry.registry_digest(),
            self.scope.universe.clone(),
            self.scope.generation.clone(),
            self.scope.source_context_id,
            producer.partition_id(),
            entities,
            relations,
        )
        .and_then(|batch| batch.with_assertion_records(records))
        .map_err(graph_error)?;
        let mut kinds = BTreeSet::new();
        for definition in self.collected.registry.relation_kinds() {
            crate::analyzer::checkpoint(stop)?;
            if relation_permission(self.recipe, producer, definition.relation_id()) {
                kinds.insert(definition.relation());
            }
        }
        let mut coverage = self
            .collected
            .coverage
            .iter()
            .filter(|record| kinds.contains(&record.relation()))
            .cloned()
            .collect::<Vec<_>>();
        coverage.sort_by_key(GraphCoverageRecord::relation);
        let stage = PlatformGraphProducerProposals {
            producer_version: self.recipe.producer_version(producer),
            producer,
            batch,
            coverage,
        };
        // Replace provisional body charges with the complete retained stage.
        // The shared base includes every previously retained stage and draft.
        let mut exact = base;
        capture_scratch(observation, &mut exact, |budget| {
            budget.charge_serialized(&stage, stop)
        })?;
        // The plan retains its retry input while returning a separate native
        // batch to the caller. Both owner-created bodies use this allowance.
        capture_scratch(observation, &mut exact, |budget| {
            budget.charge_serialized(&stage, stop)
        })?;
        let cached = stage.clone();
        crate::analyzer::checkpoint(stop)?;
        self.stages.push(cached);
        self.budget = exact;
        Ok(stage)
    }

    fn validate_predecessor(
        &self,
        owner: &GraphPartitionSnapshot,
        next: usize,
        allow_pending: bool,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        owner.validate(stop).map_err(graph_error)?;
        if owner.registry() != &self.collected.registry
            || owner.foundation() != &self.foundation
            || owner.source_context_id() != self.scope.source_context_id
            || owner.partition(SOURCE_GRAPH_PARTITION).is_some()
        {
            return Err(invalid());
        }
        match (
            &self.collected.inventory_batch,
            owner.partition(PLATFORM_RAW_INVENTORY_PARTITION),
        ) {
            (None, None) => {}
            (Some(batch), Some(partition))
                if partition.batch() == batch
                    && partition.producer_version() == self.raw_inventory_producer_version()
                    && partition.coverage().is_empty()
                    && partition.report().accepted_entities().len()
                        == batch.entity_proposals().len()
                    && partition.report().accepted_relations().is_empty() => {}
            _ => return Err(invalid()),
        }
        // This native plan owns a closed direct layout. Other graph producers
        // are composed only by a later explicitly selected application route.
        for partition in owner.partitions() {
            crate::analyzer::checkpoint(stop)?;
            if self.collected.inventory_batch.is_some()
                && partition.partition_id() == PLATFORM_RAW_INVENTORY_PARTITION
            {
                continue;
            }
            let index = ORDER
                .iter()
                .position(|producer| producer.partition_id() == partition.partition_id())
                .ok_or_else(invalid)?;
            if index >= next && !(index == next && allow_pending) {
                return Err(invalid());
            }
        }
        for (index, producer) in ORDER.iter().enumerate() {
            crate::analyzer::checkpoint(stop)?;
            let actual = owner.partition(producer.partition_id());
            if index < next || (index == next && allow_pending && actual.is_some()) {
                let expected = self.stages.get(index).ok_or_else(invalid)?;
                let actual = actual.ok_or_else(invalid)?;
                if actual.batch() != &expected.batch
                    || actual.producer_version() != expected.producer_version()
                    || actual.coverage() != expected.coverage
                    || actual.report().accepted_entities().len()
                        != expected.batch.entity_proposals().len()
                    || actual.report().accepted_relations().len()
                        != self
                            .collected
                            .relations
                            .iter()
                            .filter(|draft| draft.producer == *producer)
                            .count()
                {
                    return Err(invalid());
                }
            } else if actual.is_some() {
                return Err(invalid());
            }
        }
        crate::analyzer::checkpoint(stop)
    }

    pub fn finish(
        self,
        owner: &'a GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ProjectResult<PlatformGraphProvenance<'a>> {
        self.finish_diagnosed(owner, stop, &mut SourceGraphDiagnostics::default())
    }

    /// Observe address/support assembly against the same retained native owner.
    pub fn finish_diagnosed(
        self,
        owner: &'a GraphPartitionSnapshot,
        stop: &AtomicBool,
        diagnostics: &mut SourceGraphDiagnostics,
    ) -> ProjectResult<PlatformGraphProvenance<'a>> {
        let mut observation = self.budget;
        let mut completed = None;
        let result = diagnostics.measure(
            DiagnosticPhase::DirectSupport,
            &mut observation,
            None,
            CountingMethod::SerializedBytes,
            |observation| self.finish_observed(owner, stop, observation, &mut completed),
        );
        if let Some(count) = completed {
            diagnostics.completed_records(DiagnosticPhase::DirectSupport, count);
        }
        result
    }

    fn finish_observed(
        self,
        owner: &'a GraphPartitionSnapshot,
        stop: &AtomicBool,
        observation: &mut ProducerBudget,
        completed: &mut Option<usize>,
    ) -> ProjectResult<PlatformGraphProvenance<'a>> {
        if self.stages.len() != ORDER.len() {
            return Err(invalid());
        }
        self.validate_predecessor(owner, ORDER.len(), false, stop)?;
        let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
        let mut addresses = BTreeMap::new();
        let mut budget = self.budget;
        // Only accepted address-map rows are counted here, not catalog evidence.
        *completed = Some(0);
        for stage in &self.stages {
            for proposal in stage.batch.entity_proposals() {
                crate::analyzer::checkpoint(stop)?;
                let key = local(GraphAssertionKind::Entity, proposal.proposal_id());
                let reference = producer_reference(&stage.batch, key.clone());
                let resolved = lookup
                    .entity(&self.scope, &reference, stop)
                    .map_err(graph_error)?;
                if resolved.proposal() != proposal {
                    return Err(invalid());
                }
                capture_scratch(observation, &mut budget, |budget| {
                    budget.charge_serialized(&(&key, &reference), stop)
                })?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
                *completed = Some(addresses.len());
            }
            for draft in self
                .collected
                .relations
                .iter()
                .filter(|draft| draft.producer == stage.producer)
            {
                crate::analyzer::checkpoint(stop)?;
                let proposal = stage
                    .batch
                    .relation_proposal(&draft.proposal_id)
                    .ok_or_else(invalid)?;
                let key = local(GraphAssertionKind::Relation, proposal.proposal_id());
                let reference = producer_reference(&stage.batch, key.clone());
                let resolved = lookup
                    .relation(&self.scope, &reference, stop)
                    .map_err(graph_error)?;
                if resolved.proposal() != proposal {
                    return Err(invalid());
                }
                capture_scratch(observation, &mut budget, |budget| {
                    budget.charge_serialized(&(&key, &reference), stop)
                })?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
                *completed = Some(addresses.len());
            }
        }
        if let Some(batch) = &self.collected.inventory_batch {
            for proposal in batch.entity_proposals() {
                crate::analyzer::checkpoint(stop)?;
                let key = local(GraphAssertionKind::Entity, proposal.proposal_id());
                let reference = producer_reference(batch, key.clone());
                let resolved = lookup
                    .entity(&self.scope, &reference, stop)
                    .map_err(graph_error)?;
                if resolved.proposal() != proposal {
                    return Err(invalid());
                }
                capture_scratch(observation, &mut budget, |budget| {
                    budget.charge_serialized(&(&key, &reference), stop)
                })?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
                *completed = Some(addresses.len());
            }
        }
        let evidence = capture_scratch(observation, &mut budget, |budget| {
            producer_evidence::catalog(
                self.project,
                owner,
                &self.collected.provenance,
                &self.scope,
                &addresses,
                budget,
                stop,
            )
        })?;
        let xml_classification_membership = match self.collected.provenance.xml_classification() {
            Some(report) => Some(capture_scratch(observation, &mut budget, |budget| {
                xml_classification::membership(report, owner, &self.scope, &addresses, budget, stop)
            })?),
            None => None,
        };
        crate::analyzer::checkpoint(stop)?;
        Ok(PlatformGraphProvenance {
            recipe: self.recipe,
            inventory_span_omissions: self.inventory_span_omissions,
            xml_source_maps: self.xml_source_maps,
            project: self.project,
            graph: owner,
            xml_classification_membership,
            scope: self.scope,
            source: self.collected.provenance,
            addresses,
            evidence,
        })
    }
}

fn endpoint(
    original: &GraphProposalEndpoint,
    addresses: &BTreeMap<Box<str>, GraphAssertionRef>,
    lookup: &GraphProducerLookup<'_>,
    scope: &GraphAssertionRecordScope,
    stop: &AtomicBool,
) -> ProjectResult<GraphProposalEndpoint> {
    let GraphProposalEndpoint::Proposed(id) = original else {
        return Err(invalid());
    };
    match addresses.get(id).ok_or_else(invalid)? {
        GraphAssertionRef::Local { assertion } if assertion.kind == GraphAssertionKind::Entity => {
            Ok(GraphProposalEndpoint::Proposed(
                assertion.proposal_id.clone(),
            ))
        }
        reference @ GraphAssertionRef::Producer { .. } => {
            let resolved = lookup.entity(scope, reference, stop).map_err(graph_error)?;
            Ok(GraphProposalEndpoint::Existing(
                resolved.accepted().node().node_id().clone(),
            ))
        }
        _ => Err(invalid()),
    }
}
fn local(kind: GraphAssertionKind, id: &str) -> GraphLocalAssertion {
    GraphLocalAssertion {
        kind,
        proposal_id: id.into(),
    }
}
fn producer_reference(
    batch: &GraphProposalBatch,
    assertion: GraphLocalAssertion,
) -> GraphAssertionRef {
    GraphAssertionRef::Producer {
        partition_id: batch.producer_partition_id().into(),
        batch_id: batch.batch_id().into(),
        assertion,
    }
}
fn entity_permission(recipe: DirectRecipe, producer: PlatformGraphProducer, kind: &str) -> bool {
    match producer {
        PlatformGraphProducer::Inventory => {
            matches!(kind, "source_file" | "source_package")
                || (recipe.inventory_spans() && kind == "source_span")
                || (recipe.structural_roles() && kind == "source_project")
        }
        PlatformGraphProducer::TocLoad => {
            kind == "state_root"
                || (recipe.structural_roles()
                    && matches!(
                        kind,
                        "source_toc_manifest"
                            | "source_toc_variant"
                            | "source_toc_load_occurrence"
                            | "source_toc_load_policy"
                    ))
        }
        PlatformGraphProducer::AnalyzerStructure => matches!(
            kind,
            "lua_source_declaration" | "lua_source_function" | "state_path"
        ),
        PlatformGraphProducer::XmlStructure => {
            matches!(kind, "xml_source_declaration" | "xml_source_handler")
                || (recipe.xml_schema_classification() && kind == xml_classification::CLASS_KIND)
                || (recipe.structural_roles()
                    && matches!(
                        kind,
                        "xml_source_document"
                            | "xml_source_occurrence"
                            | "xml_source_script_site"
                            | "xml_source_load_site"
                            | "xml_source_parent_reference"
                    ))
                || (recipe.xml_source_maps()
                    && matches!(
                        kind,
                        "xml_source_virtual_lua_unit" | "xml_source_virtual_lua_map_piece"
                    ))
        }
    }
}
fn relation_permission(recipe: DirectRecipe, producer: PlatformGraphProducer, kind: &str) -> bool {
    match producer {
        PlatformGraphProducer::Inventory => {
            kind == "source_package_owns"
                || (recipe.inventory_spans() && kind == "source_file_contains_span")
                || (recipe.structural_roles()
                    && matches!(
                        kind,
                        "source_project_contains_package"
                            | "source_project_contains_file"
                            | "source_project_contains_raw_member"
                            | "source_package_contains_raw_member"
                    ))
        }
        PlatformGraphProducer::TocLoad => {
            matches!(
                kind,
                "source_loads"
                    | "source_package_depends_on"
                    | "source_package_loads"
                    | "source_declaration_owns"
            ) || (recipe.structural_roles()
                && matches!(
                    kind,
                    "source_package_selects_toc"
                        | "source_toc_defines_variant"
                        | "source_toc_contains_occurrence"
                        | "source_toc_occurrence_loads"
                        | "source_toc_occurs_before"
                        | "source_toc_defines_load_policy"
                ))
        }
        PlatformGraphProducer::AnalyzerStructure => kind == "source_declaration_owns",
        PlatformGraphProducer::XmlStructure => {
            matches!(
                kind,
                "source_loads"
                    | "source_declaration_owns"
                    | "source_xml_inherits"
                    | "source_mixes_in"
            ) || (recipe.structural_roles()
                && matches!(
                    kind,
                    "xml_file_contains_document"
                        | "xml_document_contains_occurrence"
                        | "xml_lexical_contains"
                        | "xml_occurrence_source_span"
                        | "xml_occurrence_owns_script_site"
                        | "xml_occurrence_owns_load_site"
                        | "xml_include_target"
                        | "xml_external_script_target"
                        | "xml_occurrence_owns_parent_reference"
                ))
                || (recipe.xml_schema_classification()
                    && kind == xml_classification::CLASSIFICATION_RELATION)
                || (recipe.xml_source_maps()
                    && matches!(
                        kind,
                        "xml_script_site_owns_virtual_lua"
                            | "xml_virtual_lua_contains_map_piece"
                            | "xml_map_piece_source_span"
                    ))
        }
    }
}
fn direct_counts_exhausted(
    collected: &CollectedSourceGraph,
    include_raw_assertions: bool,
) -> ProjectError {
    let raw = collected
        .inventory_batch
        .as_ref()
        .map_or(0, |batch| batch.entity_proposals().len()) as u128;
    let nodes = collected.entities.len() as u128 + raw;
    let edges = collected.relations.len();
    let assertions = collected.entities.len() as u128
        + edges as u128
        + if include_raw_assertions { raw } else { 0 };
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        format!(
            "source graph direct counts exceed their bounded profile: nodes {nodes}/{MAX_NODES}, edges {edges}/{MAX_EDGES}, assertions {assertions}/{MAX_DIRECT_ASSERTIONS}"
        ),
    )
}

pub(super) fn graph_error(error: wow_graph::GraphError) -> ProjectError {
    ProjectError::new(
        match error.code() {
            wow_graph::GraphErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
            wow_graph::GraphErrorCode::BudgetExceeded => ProjectErrorCode::SourceBudgetExceeded,
            _ => ProjectErrorCode::SnapshotInvalid,
        },
        ProjectPhase::View,
        if error.code() == wow_graph::GraphErrorCode::BudgetExceeded {
            format!(
                "native direct producer validation failed: {}",
                error.message()
            )
        } else {
            "native direct producer validation failed".into()
        },
    )
}
