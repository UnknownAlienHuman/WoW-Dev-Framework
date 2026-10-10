//! Bind original source receipts to their actual native producer, then rebind
//! accepted input identities into the final materialized graph.
use super::assembly_budget::AssemblyBudget;
use super::*;
use std::collections::BTreeMap;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphLocalAssertion,
    GraphNodeId, GraphProducerLookup,
};
use wow_project::graph::{PlatformGraphProducer, PlatformGraphProvenance, SOURCE_GRAPH_PARTITION};

enum Authority<'view, 'native> {
    Legacy {
        source: &'view ProjectGraphProvenance,
        owner: &'native GraphPartitionSnapshot,
        scope: GraphAssertionRecordScope,
    },
    Direct(&'view PlatformGraphProvenance<'native>),
}

pub(super) struct SourceGraphAddressCrosswalk<'view, 'native> {
    authority: Authority<'view, 'native>,
    budget: Option<AssemblyBudget>,
}

impl<'view, 'native> SourceGraphAddressCrosswalk<'view, 'native> {
    fn original_owner(&self) -> &GraphPartitionSnapshot {
        match &self.authority {
            Authority::Legacy { owner, .. } => owner,
            Authority::Direct(value) => value.graph(),
        }
    }

    fn validate_partition(
        &self,
        current: &GraphPartitionSnapshot,
        partition: &wow_graph::GraphProducerPartition,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        checkpoint(stop)?;
        let original = self.original_owner();
        if original.partition(partition.partition_id()) != Some(partition)
            || current.partition(partition.partition_id()) != Some(partition)
            || current.source_context_id() != self.scope().source_context_id
            || current.registry() != original.registry()
            || current.foundation() != original.foundation()
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        checkpoint(stop)
    }

    pub(super) fn legacy(
        source: &'view ProjectGraphProvenance,
        owner: &'native GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
        if source.context().context_id() != lookup.scope().source_context_id
            || owner.partition(SOURCE_GRAPH_PARTITION).is_none()
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        let scope = lookup.scope().clone();
        checkpoint(stop)?;
        Ok(Self {
            authority: Authority::Legacy {
                source,
                owner,
                scope,
            },
            budget: None,
        })
    }

    pub(super) fn direct(
        finished: &'view PlatformGraphProvenance<'native>,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        let value = Self {
            authority: Authority::Direct(finished),
            budget: Some(AssemblyBudget::new()),
        };
        value.reserve("held_source_report", finished.source(), stop)?;
        value.reserve("service_source_report", finished.source(), stop)?;
        Ok(value)
    }

    pub(super) fn reserve(
        &self,
        field: &'static str,
        value: &impl Serialize,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        if let Some(budget) = &self.budget {
            budget.reserve(field, value, stop)?;
        }
        checkpoint(stop)
    }

    pub(super) fn append<T: Serialize>(
        &self,
        field: &'static str,
        output: &mut Vec<T>,
        value: T,
        stop: &AtomicBool,
    ) -> ServiceResult<()> {
        self.reserve(field, &value, stop)?;
        output.push(value);
        checkpoint(stop)
    }

    pub(super) fn copied_assertion(
        &self,
        kind: GraphAssertionKind,
        proposal_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<GraphAssertionRef> {
        checkpoint(stop)?;
        let Authority::Direct(value) = &self.authority else {
            return self.assertion(kind, proposal_id, stop);
        };
        let local = GraphLocalAssertion {
            kind,
            proposal_id: proposal_id.into(),
        };
        let reference = value
            .assertion(&local)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        self.reserve("source_endpoint", reference, stop)?;
        let reference = reference.clone();
        checkpoint(stop)?;
        Ok(reference)
    }

    pub(super) fn source(&self) -> &ProjectGraphProvenance {
        match &self.authority {
            Authority::Legacy { source, .. } => source,
            Authority::Direct(value) => value.source(),
        }
    }

    pub(super) fn scope(&self) -> &GraphAssertionRecordScope {
        match &self.authority {
            Authority::Legacy { scope, .. } => scope,
            Authority::Direct(value) => value.scope(),
        }
    }

    pub(super) fn direct_provenance(&self) -> Option<&PlatformGraphProvenance<'native>> {
        match &self.authority {
            Authority::Direct(value) => Some(value),
            Authority::Legacy { .. } => None,
        }
    }

    pub(super) fn assertion(
        &self,
        kind: GraphAssertionKind,
        proposal_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<GraphAssertionRef> {
        checkpoint(stop)?;
        let assertion = GraphLocalAssertion {
            kind,
            proposal_id: proposal_id.into(),
        };
        let reference = match &self.authority {
            Authority::Legacy { owner, .. } => {
                let partition = owner
                    .partition(SOURCE_GRAPH_PARTITION)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                GraphAssertionRef::Producer {
                    partition_id: partition.partition_id().into(),
                    batch_id: partition.batch().batch_id().into(),
                    assertion,
                }
            }
            Authority::Direct(value) => value
                .assertion(&assertion)
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
                .clone(),
        };
        checkpoint(stop)?;
        Ok(reference)
    }

    pub(super) fn analyzer_partition<'graph>(
        &self,
        current: &'graph GraphPartitionSnapshot,
        stop: &AtomicBool,
    ) -> ServiceResult<&'graph str> {
        checkpoint(stop)?;
        let id = match &self.authority {
            Authority::Legacy { .. } => SOURCE_GRAPH_PARTITION,
            Authority::Direct(_) => PlatformGraphProducer::AnalyzerStructure.partition_id(),
        };
        let partition = current
            .partition(id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        self.validate_partition(current, partition, stop)?;
        Ok(partition.partition_id())
    }

    pub(super) fn node_id(
        &self,
        snapshot: &GraphPartitionSnapshot,
        lookup: &GraphProducerLookup<'_>,
        proposal_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<GraphNodeId> {
        let reference = self.assertion(GraphAssertionKind::Entity, proposal_id, stop)?;
        let resolved = lookup
            .entity(self.scope(), &reference, stop)
            .map_err(graph_error)?;
        self.validate_partition(snapshot, resolved.partition(), stop)?;
        let id = materialized_partition_node_id(
            snapshot,
            resolved.partition().partition_id(),
            resolved.proposal().proposal_id(),
            snapshot.snapshot().limits(),
        )?;
        checkpoint(stop)?;
        Ok(id)
    }

    pub(super) fn edge_id(
        &self,
        snapshot: &GraphPartitionSnapshot,
        lookup: &GraphProducerLookup<'_>,
        proposal_id: &str,
        nodes: &BTreeMap<GraphNodeId, GraphNodeId>,
        stop: &AtomicBool,
    ) -> ServiceResult<wow_graph::GraphEdgeId> {
        let reference = self.assertion(GraphAssertionKind::Relation, proposal_id, stop)?;
        let resolved = lookup
            .relation(self.scope(), &reference, stop)
            .map_err(graph_error)?;
        self.validate_partition(snapshot, resolved.partition(), stop)?;
        let id = materialized::edge_id(
            snapshot,
            resolved.partition().partition_id(),
            resolved.proposal().proposal_id(),
            nodes,
        )?;
        checkpoint(stop)?;
        Ok(id)
    }
}
