//! Snapshot-bound breadth-first neighborhoods. Projection preserves original
//! nodes, directed edges and evidence; it never creates a transitive relation.
mod walk;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    GraphCoverageRecord, GraphDirection, GraphEdge, GraphEdgeId, GraphError, GraphErrorCode,
    GraphGenerationId, GraphNode, GraphNodeId, GraphPathConfidence, GraphQueryState,
    GraphRelationKind, GraphResult, GraphSnapshot, GraphSnapshotId, GraphUniverseId,
};

pub const GRAPH_SUBGRAPH_QUERY_SCHEMA: &str = "wow-graph/project-subgraph/e2-a/1";
/// Per-relation traversal directions require a distinct query/result profile.
/// Uniform-direction requests retain their original v1 bytes and identities.
pub const GRAPH_DIRECTED_SUBGRAPH_QUERY_SCHEMA: &str = "wow-graph/project-subgraph/e2-a/2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphRelationDirection {
    pub relation: GraphRelationKind,
    pub direction: GraphDirection,
}

/// Uses the same explicit confidence ceiling as bounded paths. Candidate edges
/// remain opt-in and never grant negative authority.
pub type GraphSubgraphConfidence = GraphPathConfidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphSubgraphLimits {
    pub max_depth: u32,
    pub max_nodes: u32,
    pub max_edges: u32,
    /// All snapshot edges examined during adjacency indexing, including filtered
    /// edges. An incomplete index is rejected, never used as a complete graph.
    pub max_scanned_edges: u32,
    /// Adjacency entries examined during traversal, including revisits and depth
    /// boundary checks. Snapshot validation is separately snapshot-bounded.
    pub max_expansions: u32,
    pub max_output_bytes: u32,
}

impl GraphSubgraphLimits {
    pub fn validate(self) -> GraphResult<()> {
        if self.max_depth > 64
            || !(1..=16_384).contains(&self.max_nodes)
            || !(1..=100_000).contains(&self.max_edges)
            || !(1..=4_000_000).contains(&self.max_scanned_edges)
            || !(1..=1_000_000).contains(&self.max_expansions)
            || !(16_384..=8_388_608).contains(&self.max_output_bytes)
        {
            return Err(invalid("subgraph limits are outside the bounded profile"));
        }
        Ok(())
    }
}

impl Default for GraphSubgraphLimits {
    fn default() -> Self {
        Self {
            max_depth: 4,
            max_nodes: 256,
            max_edges: 1024,
            max_scanned_edges: 500_000,
            max_expansions: 100_000,
            max_output_bytes: 1_048_576,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphSubgraphQuery {
    snapshot_id: GraphSnapshotId,
    roots: Vec<GraphNodeId>,
    direction: GraphDirection,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    relation_directions: Vec<GraphRelationDirection>,
    relations: Vec<GraphRelationKind>,
    confidence: GraphSubgraphConfidence,
    limits: GraphSubgraphLimits,
}

impl GraphSubgraphQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        mut roots: Vec<GraphNodeId>,
        direction: GraphDirection,
        mut relations: Vec<GraphRelationKind>,
        limits: GraphSubgraphLimits,
    ) -> GraphResult<Self> {
        roots.sort();
        relations.sort();
        let query = Self {
            snapshot_id,
            roots,
            direction,
            relation_directions: Vec::new(),
            relations,
            confidence: GraphSubgraphConfidence::default(),
            limits,
        };
        query.validate()?;
        Ok(query)
    }

    /// Traverse each admitted relation in its own declared direction. Original
    /// edge endpoints remain unchanged; inverse traversal creates no inverse edge.
    pub fn new_directed(
        snapshot_id: GraphSnapshotId,
        mut roots: Vec<GraphNodeId>,
        mut relation_directions: Vec<GraphRelationDirection>,
        limits: GraphSubgraphLimits,
    ) -> GraphResult<Self> {
        roots.sort();
        relation_directions.sort();
        let query = Self {
            snapshot_id,
            roots,
            direction: GraphDirection::Both,
            relations: relation_directions
                .iter()
                .map(|step| step.relation)
                .collect(),
            relation_directions,
            confidence: GraphSubgraphConfidence::default(),
            limits,
        };
        query.validate()?;
        Ok(query)
    }

    #[must_use]
    pub const fn schema(&self) -> &'static str {
        if self.relation_directions.is_empty() {
            GRAPH_SUBGRAPH_QUERY_SCHEMA
        } else {
            GRAPH_DIRECTED_SUBGRAPH_QUERY_SCHEMA
        }
    }

    #[must_use]
    pub fn relation_directions(&self) -> &[GraphRelationDirection] {
        &self.relation_directions
    }

    /// None means that the relation is not part of this query at all.
    #[must_use]
    pub fn direction_for(&self, relation: GraphRelationKind) -> Option<GraphDirection> {
        let index = self.relations.binary_search(&relation).ok()?;
        Some(
            self.relation_directions
                .get(index)
                .map_or(self.direction, |step| step.direction),
        )
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: GraphSubgraphConfidence) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn validate(&self) -> GraphResult<()> {
        self.limits.validate()?;
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        if self.roots.is_empty()
            || self.roots.len() > 64
            || self.roots.len() > self.limits.max_nodes as usize
            || self.roots.windows(2).any(|pair| pair[0] >= pair[1])
            || self.relations.is_empty()
            || self.relations.len() > 19
            || self.relations.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid(
                "subgraph requires bounded unique ordered roots and relations",
            ));
        }
        if !self.relation_directions.is_empty()
            && (self.direction != GraphDirection::Both
                || self.relation_directions.len() != self.relations.len()
                || self
                    .relation_directions
                    .iter()
                    .zip(&self.relations)
                    .any(|(step, relation)| step.relation != *relation))
        {
            return Err(invalid(
                "per-relation directions must exactly cover the canonical whitelist",
            ));
        }
        for root in &self.roots {
            GraphNodeId::new(root.as_str())?;
        }
        Ok(())
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn roots(&self) -> &[GraphNodeId] {
        &self.roots
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
    pub const fn confidence(&self) -> GraphSubgraphConfidence {
        self.confidence
    }
    #[must_use]
    pub const fn limits(&self) -> GraphSubgraphLimits {
        self.limits
    }

    /// Multi-source BFS; roots and incident edges use canonical identity order.
    /// Node/edge/expansion/byte/depth stops are explicit, not successful exhaustion.
    /// No continuation is claimed: another request is a distinct bounded read.
    pub fn execute(
        &self,
        snapshot: &GraphSnapshot,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphSubgraphResult> {
        checkpoint(cancelled)?;
        self.validate()?;
        if self.snapshot_id != *snapshot.snapshot_id() {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotIdentityMismatch,
                "subgraph query names another snapshot",
            ));
        }
        // Reject impossible index work before cloning the snapshot for validation.
        if snapshot.edges().len() > self.limits.max_scanned_edges as usize {
            return Err(budget(
                "subgraph adjacency indexing exceeds its scan budget",
            ));
        }
        if self.limits.max_edges > snapshot.limits().max_query_edges {
            return Err(budget("subgraph edge limit exceeds snapshot query policy"));
        }
        snapshot.validate()?;
        checkpoint(cancelled)?;
        for root in &self.roots {
            checkpoint(cancelled)?;
            if snapshot.node(root).is_none() {
                return Err(invalid(
                    "subgraph root is absent from the selected snapshot",
                ));
            }
        }
        let bytes = encoded(&(self.schema(), self))?;
        let hash = Sha256::digest(bytes);
        let hex = hash
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let query_digest = format!("graph-subgraph-query:sha256:{hex}").into_boxed_str();
        checkpoint(cancelled)?;
        walk::execute(self, snapshot, query_digest, cancelled)
    }
}

/// Depth is the shortest admitted distance from any root. The first canonical
/// discovery edge is a traversal witness, not a new ownership or parent fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphSubgraphNode {
    node: GraphNode,
    depth: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    discovery_edge: Option<GraphEdgeId>,
}
impl GraphSubgraphNode {
    #[must_use]
    pub const fn node(&self) -> &GraphNode {
        &self.node
    }
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }
    #[must_use]
    pub fn discovery_edge(&self) -> Option<&GraphEdgeId> {
        self.discovery_edge.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphSubgraphTruncation {
    Depth,
    Nodes,
    Edges,
    Expansions,
    OutputBytes,
}

/// Read-only owner result, not a replacement/publishable GraphSnapshot. The exact
/// request binds the snapshot (and its universe/generation), policy and budgets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphSubgraphResult {
    schema: &'static str,
    query: GraphSubgraphQuery,
    query_digest: Box<str>,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    state: GraphQueryState,
    nodes: Vec<GraphSubgraphNode>,
    edges: Vec<GraphEdge>,
    coverage: Vec<GraphCoverageRecord>,
    missing_coverage: Vec<GraphRelationKind>,
    scanned_edges: u32,
    expansions: u32,
    truncations: Vec<GraphSubgraphTruncation>,
    no_new_evidence: bool,
    absence_authoritative: bool,
}
impl GraphSubgraphResult {
    #[must_use]
    pub const fn query(&self) -> &GraphSubgraphQuery {
        &self.query
    }
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.query.snapshot_id
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }
    #[must_use]
    pub const fn universe(&self) -> &GraphUniverseId {
        &self.universe
    }
    #[must_use]
    pub const fn generation(&self) -> &GraphGenerationId {
        &self.generation
    }
    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }
    #[must_use]
    pub fn nodes(&self) -> &[GraphSubgraphNode] {
        &self.nodes
    }
    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    #[must_use]
    pub fn coverage(&self) -> &[GraphCoverageRecord] {
        &self.coverage
    }
    #[must_use]
    pub fn missing_coverage(&self) -> &[GraphRelationKind] {
        &self.missing_coverage
    }
    #[must_use]
    pub const fn scanned_edges(&self) -> u32 {
        self.scanned_edges
    }
    #[must_use]
    pub const fn expansions(&self) -> u32 {
        self.expansions
    }
    #[must_use]
    pub fn truncations(&self) -> &[GraphSubgraphTruncation] {
        &self.truncations
    }
    /// No returned adjacency beyond the caller's roots. This is not absence.
    #[must_use]
    pub const fn no_new_evidence(&self) -> bool {
        self.no_new_evidence
    }
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }
}

fn checkpoint(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Acquire) {
        return Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "graph subgraph query cancelled",
        ));
    }
    Ok(())
}
fn invalid(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::QueryInvalid, message)
}
fn budget(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::BudgetExceeded, message)
}
fn encoded<T: Serialize>(value: &T) -> GraphResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| invalid("subgraph serialization failed"))
}
