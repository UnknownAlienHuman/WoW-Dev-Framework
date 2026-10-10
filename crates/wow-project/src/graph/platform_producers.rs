//! Native stage plans over one retained project projection, never a wire owner.
use super::*;
use producer_budget::ProducerBudget;
use std::collections::BTreeSet;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphEvidenceCatalog,
    GraphLocalAssertion, GraphPartitionSnapshot, GraphProducerLookup, GraphSnapshot,
};

pub const PLATFORM_DIRECT_GRAPH_PROFILE: &str = "wow-project/platform-direct-producers/1";
const PRODUCER_VERSION: &str = "1";
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
    project: &'a ProjectView,
    collected: CollectedSourceGraph,
    scope: GraphAssertionRecordScope,
    foundation: GraphSnapshot,
    stages: Vec<PlatformGraphProducerProposals>,
    budget: ProducerBudget,
}

#[derive(Clone, Serialize)]
pub struct PlatformGraphProducerProposals {
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
    pub const fn producer_version(&self) -> &'static str {
        PRODUCER_VERSION
    }
    pub fn into_parts(self) -> (GraphProposalBatch, Vec<GraphCoverageRecord>) {
        (self.batch, self.coverage)
    }
}

/// Exact addresses and source support bound to the original held native owners.
/// Serialized reports cannot construct this capability.
pub struct PlatformGraphProvenance<'a> {
    project: &'a ProjectView,
    graph: &'a GraphPartitionSnapshot,
    scope: GraphAssertionRecordScope,
    source: ProjectGraphProvenance,
    addresses: BTreeMap<GraphLocalAssertion, GraphAssertionRef>,
    evidence: GraphEvidenceCatalog,
}
impl PlatformGraphProvenance<'_> {
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        PLATFORM_DIRECT_GRAPH_PROFILE
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
    #[must_use]
    pub fn assertion(&self, assertion: &GraphLocalAssertion) -> Option<&GraphAssertionRef> {
        self.addresses.get(assertion)
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
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if config.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || config.platform_graph_profile().is_none()
    {
        return Err(ProjectError::new(
            ProjectErrorCode::DeferredCapability,
            ProjectPhase::View,
            "direct platform stages require a selected genuine platform project",
        ));
    }
    let collected = collect_source_graph_proposals(project, stop)?;
    if collected
        .entities
        .len()
        .saturating_add(collected.relations.len())
        > MAX_DIRECT_ASSERTIONS
    {
        return Err(exhausted());
    }
    // Refine the collector's provisional tally with complete borrowed metadata.
    // Shared raw/provenance bodies are counted once, under the same operation cap.
    let mut exact = 0;
    raw_inventory::charge_serialized(&mut exact, &collected, stop)?;
    let mut budget = ProducerBudget::new(exact.max(collected.text_bytes))?;
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
    for draft in &collected.entities {
        crate::analyzer::checkpoint(stop)?;
        if !entity_permission(draft.producer, draft.proposal.entity_kind_id())
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
        if !relation_permission(draft.producer, &draft.relation_kind_id)
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
        project,
        collected,
        scope,
        foundation,
        stages: Vec::new(),
        budget,
    })
}

impl<'a> PlatformGraphProposalPlan<'a> {
    #[must_use]
    pub const fn profile(&self) -> &'static str {
        PLATFORM_DIRECT_GRAPH_PROFILE
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
        crate::analyzer::checkpoint(stop)?;
        let index = ORDER
            .iter()
            .position(|candidate| *candidate == producer)
            .ok_or_else(invalid)?;
        if index + 1 == self.stages.len() {
            self.validate_predecessor(owner, index, true, stop)?;
            let mut retry_budget = self.budget;
            retry_budget.charge_serialized(&self.stages[index], stop)?;
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
        let base = self.budget;
        let mut provisional = base;
        for draft in &self.collected.entities {
            crate::analyzer::checkpoint(stop)?;
            if draft.producer == producer {
                provisional.charge_serialized(&draft.proposal, stop)?;
                entities.push(draft.proposal.clone());
            }
        }
        for draft in &self.collected.relations {
            crate::analyzer::checkpoint(stop)?;
            if draft.producer != producer {
                continue;
            }
            provisional.charge_serialized(&draft.input, stop)?;
            let mut input = draft.input.clone();
            input.source = endpoint(&input.source, &addresses, &lookup, &self.scope, stop)?;
            input.target = endpoint(&input.target, &addresses, &lookup, &self.scope, stop)?;
            let relation = GraphRelationProposal::new(
                draft.proposal_id.clone(),
                draft.relation_kind_id.clone(),
                input,
            )
            .map_err(graph_error)?;
            provisional.charge_serialized(&relation, stop)?;
            relations.push(relation);
            original_relations.push(draft);
        }
        let records = producer_derivations::records(
            &self.collected.provenance,
            &self.scope,
            &entities,
            &original_relations,
            &addresses,
            &mut provisional,
            stop,
        )?;
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
            if relation_permission(producer, definition.relation_id()) {
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
            producer,
            batch,
            coverage,
        };
        // Replace provisional body charges with the complete retained stage.
        // The shared base includes every previously retained stage and draft.
        let mut exact = base;
        exact.charge_serialized(&stage, stop)?;
        // The plan retains its retry input while returning a separate native
        // batch to the caller. Both owner-created bodies use this allowance.
        exact.charge_serialized(&stage, stop)?;
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
                    || actual.producer_version() != PRODUCER_VERSION
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
        if self.stages.len() != ORDER.len() {
            return Err(invalid());
        }
        self.validate_predecessor(owner, ORDER.len(), false, stop)?;
        let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
        let mut addresses = BTreeMap::new();
        let mut budget = self.budget;
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
                budget.charge_serialized(&(&key, &reference), stop)?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
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
                budget.charge_serialized(&(&key, &reference), stop)?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
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
                budget.charge_serialized(&(&key, &reference), stop)?;
                if addresses.insert(key, resolved.reference()).is_some() {
                    return Err(invalid());
                }
            }
        }
        let evidence = producer_evidence::catalog(
            self.project,
            owner,
            &self.collected.provenance,
            &self.scope,
            &addresses,
            &mut budget,
            stop,
        )?;
        crate::analyzer::checkpoint(stop)?;
        Ok(PlatformGraphProvenance {
            project: self.project,
            graph: owner,
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
fn entity_permission(producer: PlatformGraphProducer, kind: &str) -> bool {
    match producer {
        PlatformGraphProducer::Inventory => matches!(kind, "source_file" | "source_package"),
        PlatformGraphProducer::TocLoad => kind == "state_root",
        PlatformGraphProducer::AnalyzerStructure => matches!(
            kind,
            "lua_source_declaration" | "lua_source_function" | "state_path"
        ),
        PlatformGraphProducer::XmlStructure => {
            matches!(kind, "xml_source_declaration" | "xml_source_handler")
        }
    }
}
fn relation_permission(producer: PlatformGraphProducer, kind: &str) -> bool {
    match producer {
        PlatformGraphProducer::Inventory => kind == "source_package_owns",
        PlatformGraphProducer::TocLoad => matches!(
            kind,
            "source_loads"
                | "source_package_depends_on"
                | "source_package_loads"
                | "source_declaration_owns"
        ),
        PlatformGraphProducer::AnalyzerStructure => kind == "source_declaration_owns",
        PlatformGraphProducer::XmlStructure => matches!(
            kind,
            "source_loads" | "source_declaration_owns" | "source_xml_inherits" | "source_mixes_in"
        ),
    }
}
pub(super) fn graph_error(error: wow_graph::GraphError) -> ProjectError {
    ProjectError::new(
        match error.code() {
            wow_graph::GraphErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
            wow_graph::GraphErrorCode::BudgetExceeded => ProjectErrorCode::SourceBudgetExceeded,
            _ => ProjectErrorCode::SnapshotInvalid,
        },
        ProjectPhase::View,
        "native direct producer validation failed",
    )
}
