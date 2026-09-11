use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphDirection, GraphEdge, GraphError,
    GraphErrorCode, GraphNode, GraphNodeId, GraphRelationKind, GraphResult, GraphSnapshot,
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
        if relations.is_empty() || max_edges == 0 {
            return Err(GraphError::new(
                GraphErrorCode::QueryInvalid,
                "graph neighbor query requires relations and a positive edge limit",
            ));
        }
        relations.sort();
        if relations.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(GraphError::new(
                GraphErrorCode::QueryInvalid,
                "graph neighbor query contains duplicate relations",
            ));
        }
        Ok(Self {
            node_id,
            direction,
            relations,
            max_edges,
        })
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
        let relation_set = self.relations.iter().copied().collect::<BTreeSet<_>>();
        let mut edges = snapshot
            .edges()
            .iter()
            .filter(|edge| relation_set.contains(&edge.relation()))
            .filter(|edge| match self.direction {
                GraphDirection::Outgoing => edge.from() == &self.node_id,
                GraphDirection::Incoming => edge.to() == &self.node_id,
                GraphDirection::Both => {
                    edge.from() == &self.node_id || edge.to() == &self.node_id
                }
            })
            .cloned()
            .collect::<Vec<_>>();
        edges.sort_by(|left, right| left.edge_id().cmp(right.edge_id()));
        let truncated = edges.len() > self.max_edges as usize;
        if truncated {
            edges.truncate(self.max_edges as usize);
        }
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
        let partial_coverage = coverage.iter().any(|record| {
            matches!(
                record.state(),
                GraphCoverageState::Partial | GraphCoverageState::Failed
            )
        });
        let not_evaluated = missing_coverage
            || coverage
                .iter()
                .any(|record| record.state() == GraphCoverageState::NotEvaluated);
        let state = if truncated {
            GraphQueryState::Truncated
        } else if partial_coverage {
            GraphQueryState::Partial
        } else if not_evaluated {
            GraphQueryState::NotEvaluated
        } else {
            GraphQueryState::Complete
        };
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
