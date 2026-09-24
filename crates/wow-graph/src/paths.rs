//! Snapshot-bound, cycle-safe simple paths. Paths never create graph edges.
mod walk;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    GraphConfidence, GraphCoverageRecord, GraphDirection, GraphEdge, GraphEdgeId, GraphError,
    GraphErrorCode, GraphNode, GraphNodeId, GraphQueryState, GraphRelationKind, GraphResult,
    GraphSnapshot, GraphSnapshotId,
};

pub const GRAPH_PATH_QUERY_SCHEMA: &str = "wow-graph/bounded-paths/e2-a/1";

/// Explicit confidence ceiling. Candidate paths are opt-in and never authorize absence.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphPathConfidence {
    Proven,
    #[default]
    ProvenAndDerived,
    IncludePossible,
    IncludeCandidate,
}

impl GraphPathConfidence {
    pub(crate) fn admits(self, confidence: GraphConfidence) -> bool {
        confidence
            <= match self {
                Self::Proven => GraphConfidence::Proven,
                Self::ProvenAndDerived => GraphConfidence::Derived,
                Self::IncludePossible => GraphConfidence::Possible,
                Self::IncludeCandidate => GraphConfidence::Candidate,
            }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPathLimits {
    pub max_depth: u32,
    pub max_paths: u32,
    pub max_expansions: u32,
    pub max_output_bytes: u32,
}

impl GraphPathLimits {
    fn validate(self) -> GraphResult<()> {
        if !(1..=64).contains(&self.max_depth)
            || !(1..=256).contains(&self.max_paths)
            || !(1..=1_000_000).contains(&self.max_expansions)
            || !(16_384..=8_388_608).contains(&self.max_output_bytes)
        {
            return Err(invalid("path query limits are outside the bounded profile"));
        }
        Ok(())
    }
}

impl Default for GraphPathLimits {
    fn default() -> Self {
        Self {
            max_depth: 16,
            max_paths: 64,
            max_expansions: 100_000,
            max_output_bytes: 1_048_576,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPathQuery {
    snapshot_id: GraphSnapshotId,
    root: GraphNodeId,
    target: GraphNodeId,
    direction: GraphDirection,
    relations: Vec<GraphRelationKind>,
    confidence: GraphPathConfidence,
    limits: GraphPathLimits,
}

impl GraphPathQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        root: GraphNodeId,
        target: GraphNodeId,
        direction: GraphDirection,
        mut relations: Vec<GraphRelationKind>,
        limits: GraphPathLimits,
    ) -> GraphResult<Self> {
        relations.sort();
        let query = Self {
            snapshot_id,
            root,
            target,
            direction,
            relations,
            confidence: GraphPathConfidence::default(),
            limits,
        };
        query.validate()?;
        Ok(query)
    }

    #[must_use]
    pub fn with_confidence(mut self, confidence: GraphPathConfidence) -> Self {
        self.confidence = confidence;
        self
    }

    fn validate(&self) -> GraphResult<()> {
        self.limits.validate()?;
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        GraphNodeId::new(self.root.as_str())?;
        GraphNodeId::new(self.target.as_str())?;
        if self.root == self.target
            || self.relations.is_empty()
            || self.relations.len() > 19
            || self.relations.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(invalid(
                "paths require distinct endpoints and unique ordered relations",
            ));
        }
        Ok(())
    }

    /// Exact snapshot selected by this request; no implicit current resolution.
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }

    /// Enumerates nonempty simple paths in lexicographic edge-ID order, not shortest-path order.
    /// A continuation replays the same bounded search; expansions include replay work.
    pub fn execute(
        &self,
        snapshot: &GraphSnapshot,
        cursor: Option<&GraphPathCursor>,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphPathResult> {
        check_cancelled(cancelled)?;
        self.validate()?;
        if &self.snapshot_id != snapshot.snapshot_id() {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotIdentityMismatch,
                "path query names another graph snapshot",
            ));
        }
        snapshot.validate()?;
        check_cancelled(cancelled)?;
        if snapshot.node(&self.root).is_none() || snapshot.node(&self.target).is_none() {
            return Err(invalid(
                "path query endpoint is absent from the selected snapshot",
            ));
        }
        if self.limits.max_depth * self.limits.max_paths > snapshot.limits().max_query_edges {
            return Err(budget(
                "path query returned-edge bound exceeds snapshot policy",
            ));
        }
        let query_digest = digest(GRAPH_PATH_QUERY_SCHEMA, self)?;
        if let Some(cursor) = cursor {
            cursor.validate(self, snapshot, &query_digest, cancelled)?;
        }
        walk::execute(self, snapshot, cursor, query_digest, cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPathCursor {
    query_digest: Box<str>,
    after: Vec<GraphEdgeId>,
    integrity_digest: Box<str>,
}

impl GraphPathCursor {
    fn new(query_digest: &str, after: Vec<GraphEdgeId>) -> GraphResult<Self> {
        Ok(Self {
            query_digest: query_digest.into(),
            integrity_digest: digest("graph-path-cursor/1", &(query_digest, &after))?,
            after,
        })
    }

    fn validate(
        &self,
        query: &GraphPathQuery,
        snapshot: &GraphSnapshot,
        query_digest: &str,
        cancelled: &AtomicBool,
    ) -> GraphResult<()> {
        if self.query_digest.as_ref() != query_digest
            || self.after.is_empty()
            || self.after.len() > query.limits.max_depth as usize
            || self
                .after
                .iter()
                .any(|id| GraphEdgeId::new(id.as_str()).is_err())
            || self.integrity_digest != digest("graph-path-cursor/1", &(query_digest, &self.after))?
        {
            return Err(invalid(
                "path cursor is stale, oversized, or has changed identity",
            ));
        }
        let mut node = &query.root;
        let mut visited = std::collections::BTreeSet::from([node]);
        for id in &self.after {
            check_cancelled(cancelled)?;
            let edge = snapshot
                .edges()
                .binary_search_by(|edge| edge.edge_id().cmp(id))
                .ok()
                .map(|index| &snapshot.edges()[index])
                .ok_or_else(|| invalid("path cursor references an absent edge"))?;
            if node == &query.target
                || !query.confidence.admits(edge.confidence())
                || query.relations.binary_search(&edge.relation()).is_err()
            {
                return Err(invalid("path cursor violates the selected query policy"));
            }
            node = walk::next_node(edge, node, query.direction)
                .ok_or_else(|| invalid("path cursor is not a connected directed path"))?;
            if !visited.insert(node) {
                return Err(invalid("path cursor repeats a node"));
            }
        }
        if node != &query.target {
            return Err(invalid("path cursor does not reach the target"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPath {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    confidence: GraphConfidence,
}

impl GraphPath {
    #[must_use]
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }
    #[must_use]
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    #[must_use]
    pub const fn confidence(&self) -> GraphConfidence {
        self.confidence
    }
    fn key(&self) -> Vec<GraphEdgeId> {
        self.edges
            .iter()
            .map(|edge| edge.edge_id().clone())
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphPathTruncation {
    Depth,
    Expansions,
    Paths,
    OutputBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPathResult {
    schema: Box<str>,
    snapshot_id: GraphSnapshotId,
    query_digest: Box<str>,
    state: GraphQueryState,
    paths: Vec<GraphPath>,
    coverage: Vec<GraphCoverageRecord>,
    expansions: u32,
    prior_truncation: bool,
    absence_authoritative: bool,
    truncations: Vec<GraphPathTruncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuation: Option<GraphPathCursor>,
}

impl GraphPathResult {
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }
    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }
    #[must_use]
    pub fn paths(&self) -> &[GraphPath] {
        &self.paths
    }
    #[must_use]
    pub fn coverage(&self) -> &[GraphCoverageRecord] {
        &self.coverage
    }
    #[must_use]
    pub const fn expansions(&self) -> u32 {
        self.expansions
    }
    #[must_use]
    pub const fn prior_truncation(&self) -> bool {
        self.prior_truncation
    }
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }
    #[must_use]
    pub fn truncations(&self) -> &[GraphPathTruncation] {
        &self.truncations
    }
    #[must_use]
    pub fn continuation(&self) -> Option<&GraphPathCursor> {
        self.continuation.as_ref()
    }
}

fn check_cancelled(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "graph path query cancelled",
        ))
    } else {
        Ok(())
    }
}
fn invalid(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::QueryInvalid, message)
}
fn budget(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::BudgetExceeded, message)
}
fn encoded<T: Serialize>(value: &T) -> GraphResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| invalid("path query serialization failed"))
}
fn digest<T: Serialize>(domain: &str, value: &T) -> GraphResult<Box<str>> {
    let hash = Sha256::digest(encoded(&(domain, value))?);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = format!("{domain}:sha256:");
    for byte in hash {
        result.push(char::from(HEX[usize::from(byte >> 4)]));
        result.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(result.into_boxed_str())
}
