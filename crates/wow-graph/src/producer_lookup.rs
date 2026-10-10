//! Exact producer addresses resolve against one validated immutable input view.
//! Accepted proposal IDs belong to the input generation; publication rebinding
//! remains a separate graph-owned operation.
use std::sync::atomic::AtomicBool;

use crate::{
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphAssertionKind,
    GraphAssertionRecordScope, GraphAssertionRef, GraphEntityProposal, GraphError, GraphErrorCode,
    GraphLocalAssertion, GraphPartitionSnapshot, GraphProducerPartition, GraphRelationProposal,
    GraphResult, GraphSnapshot,
};

/// A borrowed native owner and its once-validated original input graph.
#[derive(Debug)]
pub struct GraphProducerLookup<'a> {
    owner: &'a GraphPartitionSnapshot,
    input: GraphSnapshot,
    scope: GraphAssertionRecordScope,
}

impl GraphPartitionSnapshot {
    pub fn producer_lookup(&self, stop: &AtomicBool) -> GraphResult<GraphProducerLookup<'_>> {
        let input = self.input_view(stop)?;
        let scope = GraphAssertionRecordScope {
            universe: input.universe().clone(),
            generation: input.generation().clone(),
            source_context_id: self.source_context_id(),
        };
        crate::partition::check_cancelled(stop)?;
        Ok(GraphProducerLookup {
            owner: self,
            input,
            scope,
        })
    }
}

impl<'a> GraphProducerLookup<'a> {
    #[must_use]
    pub fn scope(&self) -> &GraphAssertionRecordScope {
        &self.scope
    }

    #[must_use]
    pub fn input_view(&self) -> &GraphSnapshot {
        &self.input
    }

    /// Refuses an absent, unaccepted, stale, local or differently scoped address.
    pub fn entity(
        &self,
        scope: &GraphAssertionRecordScope,
        reference: &GraphAssertionRef,
        stop: &AtomicBool,
    ) -> GraphResult<GraphResolvedEntity<'a>> {
        let (partition, assertion) =
            self.resolve(scope, reference, GraphAssertionKind::Entity, stop)?;
        let proposal = partition
            .batch()
            .entity_proposal(&assertion.proposal_id)
            .ok_or_else(invalid)?;
        let entries = partition.report().accepted_entities();
        let index = entries
            .binary_search_by(|entry| entry.proposal_id().cmp(&assertion.proposal_id))
            .map_err(|_| invalid())?;
        let accepted = &entries[index];
        if self.input.node(accepted.node().node_id()).is_none() {
            return Err(invalid());
        }
        crate::partition::check_cancelled(stop)?;
        Ok(GraphResolvedEntity {
            partition,
            proposal,
            accepted,
        })
    }

    pub fn relation(
        &self,
        scope: &GraphAssertionRecordScope,
        reference: &GraphAssertionRef,
        stop: &AtomicBool,
    ) -> GraphResult<GraphResolvedRelation<'a>> {
        let (partition, assertion) =
            self.resolve(scope, reference, GraphAssertionKind::Relation, stop)?;
        let proposal = partition
            .batch()
            .relation_proposal(&assertion.proposal_id)
            .ok_or_else(invalid)?;
        let entries = partition.report().accepted_relations();
        let index = entries
            .binary_search_by(|entry| entry.proposal_id().cmp(&assertion.proposal_id))
            .map_err(|_| invalid())?;
        let accepted = &entries[index];
        if self.input.edge(accepted.edge().edge_id()).is_none() {
            return Err(invalid());
        }
        crate::partition::check_cancelled(stop)?;
        Ok(GraphResolvedRelation {
            partition,
            proposal,
            accepted,
        })
    }

    fn resolve(
        &self,
        scope: &GraphAssertionRecordScope,
        reference: &GraphAssertionRef,
        kind: GraphAssertionKind,
        stop: &AtomicBool,
    ) -> GraphResult<(&'a GraphProducerPartition, GraphLocalAssertion)> {
        crate::partition::check_cancelled(stop)?;
        if scope != &self.scope {
            return Err(invalid());
        }
        let GraphAssertionRef::Producer { assertion, .. } = reference else {
            return Err(invalid());
        };
        if assertion.kind != kind {
            return Err(invalid());
        }
        // Reuse the native exact-batch resolver; this entry never admits Local.
        crate::assertion_validation::resolve(self.owner.partitions(), "", reference)
    }
}

/// Original native proposal and its accepted input-generation receipt.
#[derive(Debug)]
pub struct GraphResolvedEntity<'a> {
    partition: &'a GraphProducerPartition,
    proposal: &'a GraphEntityProposal,
    accepted: &'a GraphAcceptedEntityProposal,
}

impl<'a> GraphResolvedEntity<'a> {
    #[must_use]
    pub const fn partition(&self) -> &'a GraphProducerPartition {
        self.partition
    }
    #[must_use]
    pub const fn proposal(&self) -> &'a GraphEntityProposal {
        self.proposal
    }
    #[must_use]
    pub const fn accepted(&self) -> &'a GraphAcceptedEntityProposal {
        self.accepted
    }
    #[must_use]
    pub fn reference(&self) -> GraphAssertionRef {
        producer_ref(
            self.partition,
            GraphAssertionKind::Entity,
            self.proposal.proposal_id(),
        )
    }
}

#[derive(Debug)]
pub struct GraphResolvedRelation<'a> {
    partition: &'a GraphProducerPartition,
    proposal: &'a GraphRelationProposal,
    accepted: &'a GraphAcceptedRelationProposal,
}

impl<'a> GraphResolvedRelation<'a> {
    #[must_use]
    pub const fn partition(&self) -> &'a GraphProducerPartition {
        self.partition
    }
    #[must_use]
    pub const fn proposal(&self) -> &'a GraphRelationProposal {
        self.proposal
    }
    #[must_use]
    pub const fn accepted(&self) -> &'a GraphAcceptedRelationProposal {
        self.accepted
    }
    #[must_use]
    pub fn reference(&self) -> GraphAssertionRef {
        producer_ref(
            self.partition,
            GraphAssertionKind::Relation,
            self.proposal.proposal_id(),
        )
    }
}

fn producer_ref(
    partition: &GraphProducerPartition,
    kind: GraphAssertionKind,
    proposal_id: &str,
) -> GraphAssertionRef {
    GraphAssertionRef::Producer {
        partition_id: partition.partition_id().into(),
        batch_id: partition.batch().batch_id().into(),
        assertion: GraphLocalAssertion {
            kind,
            proposal_id: proposal_id.into(),
        },
    }
}

fn invalid() -> GraphError {
    GraphError::new(
        GraphErrorCode::PartitionInvalid,
        "producer assertion lookup failed",
    )
}
