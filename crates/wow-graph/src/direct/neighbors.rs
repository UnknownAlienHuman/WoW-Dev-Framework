use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

use super::{
    admit_snapshot, budget, checkpoint, encoded_len, ensure_fits, invalid, query_digest,
    validate_output_limit,
};
use crate::{
    GraphCoverageRecord, GraphEdge, GraphGenerationId, GraphNeighborQuery, GraphNode, GraphNodeId,
    GraphPathConfidence, GraphQueryState, GraphRelationKind, GraphResult, GraphSnapshot,
    GraphSnapshotId, GraphUniverseId,
};

pub const GRAPH_NEIGHBOR_READ_SCHEMA: &str = "wow-graph/neighbors/e2-a/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborReadLimits {
    pub max_scanned_edges: u32,
    pub max_output_bytes: u32,
}
impl Default for GraphNeighborReadLimits {
    fn default() -> Self {
        Self {
            max_scanned_edges: 500_000,
            max_output_bytes: 1_048_576,
        }
    }
}
impl GraphNeighborReadLimits {
    pub fn validate(self) -> GraphResult<()> {
        validate_output_limit(self.max_output_bytes)?;
        if !(1..=4_000_000).contains(&self.max_scanned_edges) {
            return Err(invalid(
                "neighbor scan limit is outside the bounded profile",
            ));
        }
        Ok(())
    }
}

/// Exact-snapshot envelope around the existing one-hop selection. Unlike the
/// legacy operation, this API requires explicit confidence and bounded output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphNeighborReadQuery {
    snapshot_id: GraphSnapshotId,
    query: GraphNeighborQuery,
    confidence: GraphPathConfidence,
    limits: GraphNeighborReadLimits,
}
impl GraphNeighborReadQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        query: GraphNeighborQuery,
        limits: GraphNeighborReadLimits,
    ) -> GraphResult<Self> {
        let request = Self {
            snapshot_id,
            query,
            confidence: GraphPathConfidence::default(),
            limits,
        };
        request.validate()?;
        Ok(request)
    }
    pub fn validate(&self) -> GraphResult<()> {
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        self.query.validate()?;
        self.limits.validate()?;
        if self.query.max_edges() > 100_000 {
            return Err(invalid(
                "neighbor edge limit is outside the bounded profile",
            ));
        }
        Ok(())
    }
    #[must_use]
    pub fn with_confidence(mut self, confidence: GraphPathConfidence) -> Self {
        self.confidence = confidence;
        self
    }
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn query(&self) -> &GraphNeighborQuery {
        &self.query
    }
    #[must_use]
    pub const fn confidence(&self) -> GraphPathConfidence {
        self.confidence
    }
    #[must_use]
    pub const fn limits(&self) -> GraphNeighborReadLimits {
        self.limits
    }

    /// Return an edge-ID-ordered prefix of direct incident edges. No traversal
    /// beyond the root occurs, so a complete one-hop result has no depth cutoff.
    pub fn execute<'a>(
        &self,
        snapshot: &'a GraphSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<GraphNeighborView<'a>> {
        checkpoint(stop)?;
        self.validate()?;
        if self.snapshot_id != *snapshot.snapshot_id() {
            return Err(crate::GraphError::new(
                crate::GraphErrorCode::SnapshotIdentityMismatch,
                "neighbor read names another graph snapshot",
            ));
        }
        // Preflight complete scan work; never interpret a partial adjacency scan.
        if snapshot.edges().len() > self.limits.max_scanned_edges as usize
            || self.query.max_edges() > snapshot.limits().max_query_edges
        {
            return Err(budget());
        }
        admit_snapshot(&self.snapshot_id, snapshot, stop)?;
        let root = snapshot
            .node(self.query.node_id())
            .ok_or_else(|| invalid("neighbor root is absent from the selected snapshot"))?;
        let mut result = GraphNeighborView {
            schema: GRAPH_NEIGHBOR_READ_SCHEMA,
            query: self.clone(),
            query_digest: query_digest(GRAPH_NEIGHBOR_READ_SCHEMA, self)?,
            universe: snapshot.universe(),
            generation: snapshot.generation(),
            root,
            state: GraphQueryState::NotEvaluated,
            edges: Vec::new(),
            adjacent_nodes: Vec::new(),
            coverage: Vec::new(),
            missing_coverage: Vec::new(),
            scanned_edges: 0,
            matching_edges: 0,
            omitted_edges: 0,
            truncations: Vec::new(),
            no_new_evidence: true,
            absence_authoritative: false,
        };
        for relation in self.query.relations() {
            match snapshot.coverage_for(*relation) {
                Some(record) => result.coverage.push(record),
                None => result.missing_coverage.push(*relation),
            }
        }
        let limit = self.limits.max_output_bytes as usize;
        // Full exact metadata plus fixed reserve for counters and stop reasons.
        let mut used = encoded_len(&result, limit, stop)?
            .ok_or_else(budget)?
            .checked_add(2048)
            .ok_or_else(budget)?;
        if used > limit {
            return Err(budget());
        }
        let mut nodes = BTreeMap::<&GraphNodeId, &GraphNode>::new();
        for edge in snapshot.edges() {
            checkpoint(stop)?;
            result.scanned_edges += 1;
            if !self.query.matches_edge(edge) || !self.confidence.admits(edge.confidence()) {
                continue;
            }
            result.matching_edges += 1;
            // Keep counting, but never serialize or allocate omitted supports.
            if !result.truncations.is_empty() {
                continue;
            }
            if result.edges.len() >= self.query.max_edges() as usize {
                result.truncations.push(GraphNeighborTruncation::Edges);
                continue;
            }
            let adjacent_id = if edge.from() == self.query.node_id() {
                edge.to()
            } else {
                edge.from()
            };
            let new_node = if nodes.contains_key(adjacent_id) {
                None
            } else {
                Some(
                    snapshot
                        .node(adjacent_id)
                        .ok_or_else(|| invalid("neighbor endpoint is missing"))?,
                )
            };
            let edge_size = encoded_len(edge, limit - used, stop)?;
            let node_size = match new_node {
                Some(node) => encoded_len(node, limit - used, stop)?,
                None => Some(0),
            };
            let size = edge_size
                .zip(node_size)
                .and_then(|(e, n)| e.checked_add(n)?.checked_add(2));
            let Some(size) = size.filter(|size| *size <= limit - used) else {
                result
                    .truncations
                    .push(GraphNeighborTruncation::OutputBytes);
                continue;
            };
            // Admit the edge and its newly required endpoint atomically.
            used += size;
            result.edges.push(edge);
            if let Some(node) = new_node {
                nodes.insert(adjacent_id, node);
            }
        }
        result.adjacent_nodes = nodes.into_values().collect();
        result.omitted_edges = result.matching_edges - result.edges.len() as u32;
        result.no_new_evidence = result.edges.is_empty();
        result.state = crate::query::query_state(
            !result.truncations.is_empty(),
            !result.missing_coverage.is_empty(),
            result.coverage.iter().copied(),
        );
        result.absence_authoritative = result.no_new_evidence
            && result.state == GraphQueryState::Complete
            && self.confidence != GraphPathConfidence::IncludeCandidate
            && result
                .coverage
                .iter()
                .all(|record| record.negative_authority());
        ensure_fits(&result, limit, stop)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNeighborTruncation {
    Edges,
    OutputBytes,
}

/// Original immutable records are borrowed. One endpoint per distinct neighbor
/// is retained; parallel edges and their individual evidence remain separate.
#[derive(Debug, Serialize)]
pub struct GraphNeighborView<'a> {
    schema: &'static str,
    query: GraphNeighborReadQuery,
    query_digest: Box<str>,
    universe: &'a GraphUniverseId,
    generation: &'a GraphGenerationId,
    root: &'a GraphNode,
    state: GraphQueryState,
    edges: Vec<&'a GraphEdge>,
    adjacent_nodes: Vec<&'a GraphNode>,
    coverage: Vec<&'a GraphCoverageRecord>,
    missing_coverage: Vec<GraphRelationKind>,
    scanned_edges: u32,
    matching_edges: u32,
    omitted_edges: u32,
    truncations: Vec<GraphNeighborTruncation>,
    no_new_evidence: bool,
    absence_authoritative: bool,
}
impl<'a> GraphNeighborView<'a> {
    #[must_use]
    pub fn query(&self) -> &GraphNeighborReadQuery {
        &self.query
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }
    #[must_use]
    pub const fn root(&self) -> &'a GraphNode {
        self.root
    }
    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }
    #[must_use]
    pub fn edges(&self) -> &[&'a GraphEdge] {
        &self.edges
    }
    #[must_use]
    pub fn adjacent_nodes(&self) -> &[&'a GraphNode] {
        &self.adjacent_nodes
    }
    #[must_use]
    pub fn coverage(&self) -> &[&'a GraphCoverageRecord] {
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
    pub const fn matching_edges(&self) -> u32 {
        self.matching_edges
    }
    #[must_use]
    pub const fn omitted_edges(&self) -> u32 {
        self.omitted_edges
    }
    #[must_use]
    pub fn truncations(&self) -> &[GraphNeighborTruncation] {
        &self.truncations
    }
    #[must_use]
    pub const fn no_new_evidence(&self) -> bool {
        self.no_new_evidence
    }
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }
}
