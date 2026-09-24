use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphEdge, GraphError, GraphErrorCode, GraphNode,
    GraphNodeId, GraphRelationKind, GraphResult, GraphSnapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphDirection {
    Outgoing,
    Incoming,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphQueryState {
    Complete,
    Partial,
    NotEvaluated,
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborQuery {
    node_id: GraphNodeId,
    direction: GraphDirection,
    relations: Vec<GraphRelationKind>,
    max_edges: u32,
}

impl GraphNeighborQuery {
    pub fn new(
        node_id: GraphNodeId,
        direction: GraphDirection,
        mut relations: Vec<GraphRelationKind>,
        max_edges: u32,
    ) -> GraphResult<Self> {
        relations.sort();
        let query = Self {
            node_id,
            direction,
            relations,
            max_edges,
        };
        query.validate()?;
        Ok(query)
    }

    /// Validate deserialized requests as well as values made by the constructor.
    pub fn validate(&self) -> GraphResult<()> {
        GraphNodeId::new(self.node_id.as_str())?;
        if self.relations.is_empty()
            || self.max_edges == 0
            || self.relations.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(GraphError::new(
                GraphErrorCode::QueryInvalid,
                "graph neighbors require canonical unique relations and a positive edge limit",
            ));
        }
        Ok(())
    }

    pub(crate) fn matches_edge(&self, edge: &GraphEdge) -> bool {
        self.relations.binary_search(&edge.relation()).is_ok()
            && match self.direction {
                GraphDirection::Outgoing => edge.from() == &self.node_id,
                GraphDirection::Incoming => edge.to() == &self.node_id,
                GraphDirection::Both => edge.from() == &self.node_id || edge.to() == &self.node_id,
            }
    }

    #[must_use]
    pub fn node_id(&self) -> &GraphNodeId {
        &self.node_id
    }

    #[must_use]
    pub const fn direction(&self) -> GraphDirection {
        self.direction
    }

    #[must_use]
    pub fn relations(&self) -> &[GraphRelationKind] {
        &self.relations
    }

    #[must_use]
    pub const fn max_edges(&self) -> u32 {
        self.max_edges
    }

    pub fn execute(&self, snapshot: &GraphSnapshot) -> GraphResult<GraphNeighborResult> {
        self.validate()?;
        snapshot.validate()?;
        let root = snapshot.node(&self.node_id).ok_or_else(|| {
            GraphError::new(
                GraphErrorCode::QueryInvalid,
                "graph query root does not exist in the selected snapshot",
            )
        })?;
        if self.max_edges > snapshot.limits().max_query_edges {
            return Err(GraphError::new(
                GraphErrorCode::BudgetExceeded,
                "graph query exceeds the snapshot query budget",
            ));
        }
        // Snapshot validation guarantees edge-ID order. Retain at most the
        // requested prefix plus one lookahead, not a clone of every matching edge.
        let mut selected = snapshot
            .edges()
            .iter()
            .filter(|edge| self.matches_edge(edge));
        let edges = selected
            .by_ref()
            .take(self.max_edges as usize)
            .cloned()
            .collect::<Vec<_>>();
        let truncated = selected.next().is_some();
        let adjacent_ids = edges
            .iter()
            .map(|edge| {
                if edge.from() == &self.node_id {
                    edge.to().clone()
                } else {
                    edge.from().clone()
                }
            })
            .collect::<BTreeSet<_>>();
        let adjacent_nodes = adjacent_ids
            .iter()
            .map(|node_id| {
                snapshot.node(node_id).cloned().ok_or_else(|| {
                    GraphError::new(
                        GraphErrorCode::SnapshotInvalid,
                        "selected graph edge lost its adjacent node",
                    )
                })
            })
            .collect::<GraphResult<Vec<_>>>()?;
        let coverage = self
            .relations
            .iter()
            .filter_map(|relation| snapshot.coverage_for(*relation).cloned())
            .collect::<Vec<_>>();
        let missing_coverage = coverage.len() != self.relations.len();
        let state = query_state(truncated, missing_coverage, coverage.iter());
        let absence_authoritative = edges.is_empty()
            && state == GraphQueryState::Complete
            && coverage.iter().all(GraphCoverageRecord::negative_authority);
        Ok(GraphNeighborResult {
            snapshot_id: snapshot.snapshot_id().clone(),
            root: root.clone(),
            direction: self.direction,
            relations: self.relations.clone(),
            state,
            absence_authoritative,
            edges,
            adjacent_nodes,
            coverage,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborResult {
    snapshot_id: crate::GraphSnapshotId,
    root: GraphNode,
    direction: GraphDirection,
    relations: Vec<GraphRelationKind>,
    state: GraphQueryState,
    absence_authoritative: bool,
    edges: Vec<GraphEdge>,
    adjacent_nodes: Vec<GraphNode>,
    coverage: Vec<GraphCoverageRecord>,
}

impl GraphNeighborResult {
    #[must_use]
    pub fn snapshot_id(&self) -> &crate::GraphSnapshotId {
        &self.snapshot_id
    }

    #[must_use]
    pub fn root(&self) -> &GraphNode {
        &self.root
    }

    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }

    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }

    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    #[must_use]
    pub fn adjacent_nodes(&self) -> &[GraphNode] {
        &self.adjacent_nodes
    }

    #[must_use]
    pub fn coverage(&self) -> &[GraphCoverageRecord] {
        &self.coverage
    }
}

/// Common state folding for legacy and exact-snapshot one-hop reads.
pub(crate) fn query_state<'a>(
    truncated: bool,
    missing_coverage: bool,
    coverage: impl Iterator<Item = &'a GraphCoverageRecord>,
) -> GraphQueryState {
    let mut partial = false;
    let mut not_evaluated = missing_coverage;
    for record in coverage {
        partial |= matches!(
            record.state(),
            GraphCoverageState::Partial | GraphCoverageState::Failed
        );
        not_evaluated |= record.state() == GraphCoverageState::NotEvaluated;
    }
    if truncated {
        GraphQueryState::Truncated
    } else if partial {
        GraphQueryState::Partial
    } else if not_evaluated {
        GraphQueryState::NotEvaluated
    } else {
        GraphQueryState::Complete
    }
}
