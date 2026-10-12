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
        self.preflight(snapshot, stop)?;
        admit_snapshot(&self.snapshot_id, snapshot, stop)?;
        self.collect(snapshot, stop)
    }

    // Only the graph-owned sealed read proof can reach this alternative entry.
    pub(crate) fn execute_on_read<'a>(
        &self,
        read: &crate::GraphPartitionRead<'a>,
        stop: &AtomicBool,
    ) -> GraphResult<GraphNeighborView<'a>> {
        let snapshot = read.owner().snapshot();
        self.preflight(snapshot, stop)?;
        self.collect(snapshot, stop)
    }

    // A sealed owner can preflight several exact queries and physically scan its
    // immutable edge vector once. Each returned view retains the same logical
    // full-scan counters and output semantics as an independent query.
    pub(crate) fn execute_batch_on_read<'a>(
        queries: &[Self],
        read: &crate::GraphPartitionRead<'a>,
        stop: &AtomicBool,
    ) -> GraphResult<Vec<GraphNeighborView<'a>>> {
        let snapshot = read.owner().snapshot();
        checkpoint(stop)?;
        if queries.is_empty() {
            return Ok(Vec::new());
        }
        if queries.len() > snapshot.limits().max_nodes as usize {
            return Err(budget());
        }
        for query in queries {
            query.preflight(snapshot, stop)?;
        }
        collect_batch(queries, snapshot, stop)
    }

    fn preflight(&self, snapshot: &GraphSnapshot, stop: &AtomicBool) -> GraphResult<()> {
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
        Ok(())
    }

    fn collect<'a>(
        &self,
        snapshot: &'a GraphSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<GraphNeighborView<'a>> {
        collect_batch(std::slice::from_ref(self), snapshot, stop)?
            .pop()
            .ok_or_else(|| invalid("neighbor read batch lost its result"))
    }
}

fn collect_batch<'a>(
    queries: &[GraphNeighborReadQuery],
    snapshot: &'a GraphSnapshot,
    stop: &AtomicBool,
) -> GraphResult<Vec<GraphNeighborView<'a>>> {
    let mut collectors = Vec::with_capacity(queries.len());
    let mut roots = BTreeMap::<&str, Vec<usize>>::new();
    for query in queries {
        checkpoint(stop)?;
        let index = collectors.len();
        roots
            .entry(query.query().node_id().as_str())
            .or_default()
            .push(index);
        collectors.push(GraphNeighborCollector::new(query, snapshot, stop)?);
    }
    for edge in snapshot.edges() {
        checkpoint(stop)?;
        if let Some(indices) = roots.get(edge.from().as_str()) {
            for &index in indices {
                checkpoint(stop)?;
                collectors[index].observe(snapshot, edge, stop)?;
            }
        }
        if edge.to() != edge.from()
            && let Some(indices) = roots.get(edge.to().as_str())
        {
            for &index in indices {
                checkpoint(stop)?;
                collectors[index].observe(snapshot, edge, stop)?;
            }
        }
    }
    let scanned_edges = u32::try_from(snapshot.edges().len()).map_err(|_| budget())?;
    collectors
        .into_iter()
        .map(|collector| collector.finish(scanned_edges, stop))
        .collect()
}

struct GraphNeighborCollector<'a> {
    result: GraphNeighborView<'a>,
    nodes: BTreeMap<&'a GraphNodeId, &'a GraphNode>,
    used: usize,
    limit: usize,
}

impl<'a> GraphNeighborCollector<'a> {
    fn new(
        query: &GraphNeighborReadQuery,
        snapshot: &'a GraphSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<Self> {
        let root = snapshot
            .node(query.query().node_id())
            .ok_or_else(|| invalid("neighbor root is absent from the selected snapshot"))?;
        let mut result = GraphNeighborView {
            schema: GRAPH_NEIGHBOR_READ_SCHEMA,
            query: query.clone(),
            query_digest: query_digest(GRAPH_NEIGHBOR_READ_SCHEMA, query)?,
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
        for relation in query.query().relations() {
            match snapshot.coverage_for(*relation) {
                Some(record) => result.coverage.push(record),
                None => result.missing_coverage.push(*relation),
            }
        }
        let limit = query.limits().max_output_bytes as usize;
        let used = encoded_len(&result, limit, stop)?
            .ok_or_else(budget)?
            .checked_add(2048)
            .ok_or_else(budget)?;
        if used > limit {
            return Err(budget());
        }
        Ok(Self {
            result,
            nodes: BTreeMap::new(),
            used,
            limit,
        })
    }

    fn observe(
        &mut self,
        snapshot: &'a GraphSnapshot,
        edge: &'a GraphEdge,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        if !self.result.query.query().matches_edge(edge)
            || !self.result.query.confidence().admits(edge.confidence())
        {
            return Ok(());
        }
        self.result.matching_edges += 1;
        if !self.result.truncations.is_empty() {
            return Ok(());
        }
        if self.result.edges.len() >= self.result.query.query().max_edges() as usize {
            self.result.truncations.push(GraphNeighborTruncation::Edges);
            return Ok(());
        }
        let adjacent_id = if edge.from() == self.result.query.query().node_id() {
            edge.to()
        } else {
            edge.from()
        };
        let new_node = if self.nodes.contains_key(adjacent_id) {
            None
        } else {
            Some(
                snapshot
                    .node(adjacent_id)
                    .ok_or_else(|| invalid("neighbor endpoint is missing"))?,
            )
        };
        let edge_size = encoded_len(edge, self.limit - self.used, stop)?;
        let node_size = match new_node {
            Some(node) => encoded_len(node, self.limit - self.used, stop)?,
            None => Some(0),
        };
        let size = edge_size
            .zip(node_size)
            .and_then(|(edge, node)| edge.checked_add(node)?.checked_add(2));
        let Some(size) = size.filter(|size| *size <= self.limit - self.used) else {
            self.result
                .truncations
                .push(GraphNeighborTruncation::OutputBytes);
            return Ok(());
        };
        self.used += size;
        self.result.edges.push(edge);
        if let Some(node) = new_node {
            self.nodes.insert(adjacent_id, node);
        }
        Ok(())
    }

    fn finish(
        mut self,
        scanned_edges: u32,
        stop: &AtomicBool,
    ) -> GraphResult<GraphNeighborView<'a>> {
        self.result.scanned_edges = scanned_edges;
        self.result.adjacent_nodes = self.nodes.into_values().collect();
        self.result.omitted_edges = self.result.matching_edges - self.result.edges.len() as u32;
        self.result.no_new_evidence = self.result.edges.is_empty();
        self.result.state = crate::query::query_state(
            !self.result.truncations.is_empty(),
            !self.result.missing_coverage.is_empty(),
            self.result.coverage.iter().copied(),
        );
        self.result.absence_authoritative = self.result.no_new_evidence
            && self.result.state == GraphQueryState::Complete
            && self.result.query.confidence() != GraphPathConfidence::IncludeCandidate
            && self
                .result
                .coverage
                .iter()
                .all(|record| record.negative_authority());
        ensure_fits(&self.result, self.limit, stop)?;
        Ok(self.result)
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
