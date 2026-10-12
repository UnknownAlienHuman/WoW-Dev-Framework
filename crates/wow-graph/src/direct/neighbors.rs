use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;
#[cfg(test)]
thread_local! {
    static PHYSICAL_EDGE_VISITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

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

/// Runtime admission for the complete batch, including repeated roots. Bytes
/// are the sum of compact encoded views, not per-query maxima or heap usage.
/// Logical scan work remains query count times the complete edge inventory.
/// These ceilings do not participate in query/output identities or persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphNeighborBatchLimits {
    pub max_logical_scanned_edges: u64,
    pub max_output_bytes: u64,
}

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
        limits: GraphNeighborBatchLimits,
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
        u64::try_from(queries.len())
            .ok()
            .and_then(|count| count.checked_mul(u64::try_from(snapshot.edges().len()).ok()?))
            .filter(|work| *work <= limits.max_logical_scanned_edges)
            .ok_or_else(budget)?;
        // The existing per-query preflight also bounds work against the sealed
        // snapshot and each selected scan policy. No dispatch can exceed Q*E.
        for query in queries {
            query.preflight(snapshot, stop)?;
        }
        collect_batch(queries, snapshot, limits, stop)
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
        collect_batch(
            std::slice::from_ref(self),
            snapshot,
            GraphNeighborBatchLimits {
                max_logical_scanned_edges: u64::from(self.limits.max_scanned_edges),
                max_output_bytes: u64::from(self.limits.max_output_bytes),
            },
            stop,
        )?
        .pop()
        .ok_or_else(|| invalid("neighbor read batch lost its result"))
    }
}

fn collect_batch<'a>(
    queries: &[GraphNeighborReadQuery],
    snapshot: &'a GraphSnapshot,
    limits: GraphNeighborBatchLimits,
    stop: &AtomicBool,
) -> GraphResult<Vec<GraphNeighborView<'a>>> {
    let mut output = BatchOutputBudget {
        used: 0,
        limit: usize::try_from(limits.max_output_bytes).map_err(|_| budget())?,
    };
    let mut collectors = Vec::new();
    let mut roots = BTreeMap::<&str, Vec<usize>>::new();
    for query in queries {
        checkpoint(stop)?;
        // At most one individually bounded candidate exists before cumulative
        // admission. Charge it before retaining a collector or dispatch entry.
        let collector = GraphNeighborCollector::new(query, snapshot, &mut output, stop)?;
        let index = collectors.len();
        roots
            .entry(query.query().node_id().as_str())
            .or_default()
            .push(index);
        collectors.push(collector);
    }
    for edge in snapshot.edges() {
        checkpoint(stop)?;
        #[cfg(test)]
        PHYSICAL_EDGE_VISITS.with(|visits| visits.set(visits.get() + 1));
        if let Some(indices) = roots.get(edge.from().as_str()) {
            for &index in indices {
                checkpoint(stop)?;
                collectors[index].observe(snapshot, edge, &mut output, stop)?;
            }
        }
        if edge.to() != edge.from()
            && let Some(indices) = roots.get(edge.to().as_str())
        {
            for &index in indices {
                checkpoint(stop)?;
                collectors[index].observe(snapshot, edge, &mut output, stop)?;
            }
        }
    }
    collectors
        .into_iter()
        .map(|collector| collector.finish(stop))
        .collect()
}

struct BatchOutputBudget {
    used: usize,
    limit: usize,
}
impl BatchOutputBudget {
    fn replace(&mut self, before: usize, after: usize) -> GraphResult<()> {
        self.used = self
            .used
            .checked_sub(before)
            .and_then(|used| used.checked_add(after))
            .filter(|used| *used <= self.limit)
            .ok_or_else(budget)?;
        Ok(())
    }
}

/// Only these scalar/finite fields change as a prefix grows. Count them through
/// serde, using the same values as the final view. Payloads and array separators
/// are charged separately, so earlier prefixes are never reencoded per edge.
#[derive(Serialize)]
struct NeighborProgress<'a> {
    state: GraphQueryState,
    scanned_edges: u32,
    matching_edges: u32,
    omitted_edges: u32,
    truncations: &'a [GraphNeighborTruncation],
    no_new_evidence: bool,
    absence_authoritative: bool,
}
impl NeighborProgress<'_> {
    fn apply(&self, view: &mut GraphNeighborView<'_>) {
        view.state = self.state;
        view.scanned_edges = self.scanned_edges;
        view.matching_edges = self.matching_edges;
        view.omitted_edges = self.omitted_edges;
        view.no_new_evidence = self.no_new_evidence;
        view.absence_authoritative = self.absence_authoritative;
    }
}

struct GraphNeighborCollector<'a> {
    result: GraphNeighborView<'a>,
    nodes: BTreeMap<&'a GraphNodeId, &'a GraphNode>,
    used: usize,
    limit: usize,
    output_bytes: usize,
    progress_bytes: usize,
}

impl<'a> GraphNeighborCollector<'a> {
    fn new(
        query: &GraphNeighborReadQuery,
        snapshot: &'a GraphSnapshot,
        output: &mut BatchOutputBudget,
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
        // Preserve the original per-query pad/selection algorithm above. The
        // aggregate counter uses actual final-prefix bytes without that pad.
        result.scanned_edges = u32::try_from(snapshot.edges().len()).map_err(|_| budget())?;
        let progress = result.progress(0, &[]);
        let progress_bytes = encoded_len(&progress, limit, stop)?.ok_or_else(budget)?;
        progress.apply(&mut result);
        let output_bytes = encoded_len(&result, limit, stop)?.ok_or_else(budget)?;
        output.replace(0, output_bytes)?;
        Ok(Self {
            result,
            nodes: BTreeMap::new(),
            used,
            limit,
            output_bytes,
            progress_bytes,
        })
    }

    fn observe(
        &mut self,
        snapshot: &'a GraphSnapshot,
        edge: &'a GraphEdge,
        output: &mut BatchOutputBudget,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        if !self.result.query.query().matches_edge(edge)
            || !self.result.query.confidence().admits(edge.confidence())
        {
            return Ok(());
        }
        self.result.matching_edges = self
            .result
            .matching_edges
            .checked_add(1)
            .ok_or_else(budget)?;
        if let Some(truncation) = self.result.truncations.first().copied() {
            let truncations = [truncation];
            let progress = self.result.progress(self.result.edges.len(), &truncations);
            self.charge_progress(&progress, 0, output, stop)?;
            progress.apply(&mut self.result);
            return Ok(());
        }
        if self.result.edges.len() >= self.result.query.query().max_edges() as usize {
            return self.truncate(GraphNeighborTruncation::Edges, output, stop);
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
            return self.truncate(GraphNeighborTruncation::OutputBytes, output, stop);
        };
        let payload_bytes = edge_size
            .zip(node_size)
            .and_then(|(edge, node)| {
                edge.checked_add(usize::from(!self.result.edges.is_empty()))?
                    .checked_add(node)?
                    .checked_add(usize::from(new_node.is_some() && !self.nodes.is_empty()))
            })
            .ok_or_else(budget)?;
        let progress = self.result.progress(self.result.edges.len() + 1, &[]);
        self.charge_progress(&progress, payload_bytes, output, stop)?;
        progress.apply(&mut self.result);
        self.used += size;
        self.result.edges.push(edge);
        if let Some(node) = new_node {
            self.nodes.insert(adjacent_id, node);
        }
        Ok(())
    }

    fn truncate(
        &mut self,
        truncation: GraphNeighborTruncation,
        output: &mut BatchOutputBudget,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        let truncations = [truncation];
        let progress = self.result.progress(self.result.edges.len(), &truncations);
        self.charge_progress(&progress, 0, output, stop)?;
        progress.apply(&mut self.result);
        self.result.truncations.push(truncation);
        Ok(())
    }

    fn charge_progress(
        &mut self,
        progress: &NeighborProgress<'_>,
        payload_bytes: usize,
        output: &mut BatchOutputBudget,
        stop: &AtomicBool,
    ) -> GraphResult<()> {
        let progress_bytes = encoded_len(progress, self.limit, stop)?.ok_or_else(budget)?;
        let bytes = self
            .output_bytes
            .checked_sub(self.progress_bytes)
            .and_then(|bytes| {
                bytes
                    .checked_add(progress_bytes)?
                    .checked_add(payload_bytes)
            })
            .ok_or_else(budget)?;
        output.replace(self.output_bytes, bytes)?;
        self.output_bytes = bytes;
        self.progress_bytes = progress_bytes;
        Ok(())
    }

    fn finish(mut self, stop: &AtomicBool) -> GraphResult<GraphNeighborView<'a>> {
        self.result.adjacent_nodes = self.nodes.into_values().collect();
        if encoded_len(&self.result, self.limit, stop)? != Some(self.output_bytes) {
            return Err(invalid(
                "neighbor batch byte accounting disagrees with its view",
            ));
        }
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
    fn progress<'b>(
        &self,
        selected_edges: usize,
        truncations: &'b [GraphNeighborTruncation],
    ) -> NeighborProgress<'b> {
        let state = crate::query::query_state(
            !truncations.is_empty(),
            !self.missing_coverage.is_empty(),
            self.coverage.iter().copied(),
        );
        let no_new_evidence = selected_edges == 0;
        NeighborProgress {
            state,
            scanned_edges: self.scanned_edges,
            matching_edges: self.matching_edges,
            omitted_edges: self.matching_edges - selected_edges as u32,
            truncations,
            no_new_evidence,
            absence_authoritative: no_new_evidence
                && state == GraphQueryState::Complete
                && self.query.confidence() != GraphPathConfidence::IncludeCandidate
                && self
                    .coverage
                    .iter()
                    .all(|record| record.negative_authority()),
        }
    }

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
