//! Exact accepted proposal -> definition -> published edge association. This is
//! a query input over native retained records, never a replacement graph.
use super::{GraphAxisProfile, checkpoint, encoded, error};
use crate::{
    GraphDirection, GraphEdge, GraphEdgeId, GraphErrorCode, GraphPartitionSnapshot, GraphResult,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

/// One edge may retain several distinct selected definition identities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct GraphAxisEdgeDefinition {
    edge_id: GraphEdgeId,
    relation_id: Box<str>,
}
impl GraphAxisEdgeDefinition {
    #[must_use]
    pub fn edge_id(&self) -> &GraphEdgeId {
        &self.edge_id
    }
    #[must_use]
    pub fn relation_id(&self) -> &str {
        &self.relation_id
    }
}

pub(crate) struct GraphAxisSelection {
    pub(crate) profile_digest: Box<str>,
    directions: BTreeMap<GraphEdgeId, GraphDirection>,
    definitions: BTreeSet<GraphAxisEdgeDefinition>,
    costs: BTreeMap<GraphEdgeId, usize>,
    pub(super) scanned: u32,
    bytes: usize,
}
impl GraphAxisSelection {
    /// Caller has validated the immutable owner. Every retained relation is
    /// charged, including those outside the selected profile.
    pub(super) fn build(
        owner: &GraphPartitionSnapshot,
        profile: &GraphAxisProfile,
        max_scanned: u32,
        cancelled: &AtomicBool,
    ) -> GraphResult<Self> {
        let mut result = Self {
            profile_digest: profile.digest().into(),
            directions: BTreeMap::new(),
            definitions: BTreeSet::new(),
            costs: BTreeMap::new(),
            scanned: 0,
            bytes: 2,
        };
        let mut remap = BTreeMap::new();
        for node in owner.snapshot().nodes() {
            checkpoint(cancelled)?;
            remap.insert(
                crate::partition::materialize::input_node_id(node, owner.foundation())?,
                node.node_id().clone(),
            );
        }
        for edge in owner.foundation().edges() {
            result.scan(max_scanned, cancelled)?;
            if !profile
                .relations()
                .iter()
                .any(|step| step.relation() == edge.relation())
            {
                continue;
            }
            // Foundation has no retained producer proposal. A subset of a rich
            // family cannot certify which definition produced a legacy edge.
            let mut definitions = owner
                .registry()
                .relation_kinds()
                .iter()
                .filter(|d| d.relation() == edge.relation());
            let definition = definitions
                .next()
                .ok_or_else(|| error(GraphErrorCode::AxisUnsupported))?;
            if definitions.next().is_some() {
                return Err(error(GraphErrorCode::AxisProfileInvalid));
            }
            result.admit(
                owner,
                profile,
                edge,
                definition.relation_id(),
                &remap,
                cancelled,
            )?;
        }
        for partition in owner.partitions() {
            for accepted in partition.report().accepted_relations() {
                result.scan(max_scanned, cancelled)?;
                let proposal = partition
                    .batch()
                    .relation_proposal(accepted.proposal_id())
                    .ok_or_else(|| error(GraphErrorCode::PartitionInvalid))?;
                result.admit(
                    owner,
                    profile,
                    accepted.edge(),
                    proposal.relation_kind_id(),
                    &remap,
                    cancelled,
                )?;
            }
        }
        checkpoint(cancelled)?;
        Ok(result)
    }

    fn scan(&mut self, max: u32, stop: &AtomicBool) -> GraphResult<()> {
        checkpoint(stop)?;
        self.scanned = self
            .scanned
            .checked_add(1)
            .ok_or_else(|| error(GraphErrorCode::BudgetExceeded))?;
        if self.scanned > max {
            return Err(error(GraphErrorCode::BudgetExceeded));
        }
        Ok(())
    }

    fn admit(
        &mut self,
        owner: &GraphPartitionSnapshot,
        profile: &GraphAxisProfile,
        edge: &GraphEdge,
        id: &str,
        remap: &BTreeMap<crate::GraphNodeId, crate::GraphNodeId>,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        checkpoint(stop)?;
        let Some(step) = profile
            .relations()
            .iter()
            .find(|step| step.relation_id() == id)
        else {
            return Ok(());
        };
        let edge =
            crate::partition::materialize::remap_edge(edge, remap, owner.snapshot().limits())?;
        if owner.snapshot().edge(edge.edge_id()) != Some(&edge) {
            return Err(error(GraphErrorCode::PartitionInvalid));
        }
        let definition = owner
            .registry()
            .relation_kind(id)
            .ok_or_else(|| error(GraphErrorCode::RegistryInvalid))?;
        let source = owner
            .snapshot()
            .node(edge.from())
            .ok_or_else(|| error(GraphErrorCode::EndpointMissing))?;
        let target = owner
            .snapshot()
            .node(edge.to())
            .ok_or_else(|| error(GraphErrorCode::EndpointMissing))?;
        if edge.relation() != definition.relation()
            || !definition.allows_source_kind(source.kind())
            || !definition.allows_target_kind(target.kind())
            || !definition.allows_confidence(edge.confidence())
        {
            return Err(error(GraphErrorCode::RegistryInvalid));
        }
        let binding = GraphAxisEdgeDefinition {
            edge_id: edge.edge_id().clone(),
            relation_id: id.into(),
        };
        if self.definitions.contains(&binding) {
            return Ok(());
        }
        let cost = encoded(&binding)?
            .len()
            .checked_add(1)
            .ok_or_else(|| error(GraphErrorCode::BudgetExceeded))?;
        self.bytes = self
            .bytes
            .checked_add(cost)
            .ok_or_else(|| error(GraphErrorCode::BudgetExceeded))?;
        // Reuse the finite query-metadata ceiling; counts alone do not bound
        // arbitrarily many distinct definition bindings on one merged edge.
        if self.bytes > 8_388_608 {
            return Err(error(GraphErrorCode::BudgetExceeded));
        }
        if let Some(previous) = self
            .directions
            .insert(edge.edge_id().clone(), step.forward_direction())
            && previous != step.forward_direction()
        {
            return Err(error(GraphErrorCode::AxisProfileInvalid));
        }
        let debit = self.costs.entry(edge.edge_id().clone()).or_default();
        *debit = debit
            .checked_add(cost)
            .ok_or_else(|| error(GraphErrorCode::BudgetExceeded))?;
        self.definitions.insert(binding);
        Ok(())
    }

    pub(crate) fn direction(&self, id: &GraphEdgeId) -> Option<GraphDirection> {
        self.directions.get(id).copied()
    }
    pub(crate) fn output_cost(&self, id: &GraphEdgeId) -> usize {
        self.costs.get(id).copied().unwrap_or(0)
    }
    pub(super) fn project(
        &self,
        edges: &[GraphEdge],
        stop: &AtomicBool,
    ) -> GraphResult<Vec<GraphAxisEdgeDefinition>> {
        let ids = edges
            .iter()
            .map(GraphEdge::edge_id)
            .collect::<BTreeSet<_>>();
        let mut result = Vec::new();
        for entry in &self.definitions {
            checkpoint(stop)?;
            if ids.contains(&entry.edge_id) {
                result.push(entry.clone());
            }
        }
        Ok(result)
    }
}
