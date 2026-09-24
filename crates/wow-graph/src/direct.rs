//! Exact entity inspection and bounded one-hop reads over retained graph data.
//! These projections do not reconstruct source, assertions, or runtime objects.
mod neighbors;

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    GraphError, GraphErrorCode, GraphGenerationId, GraphNode, GraphNodeId, GraphQueryState,
    GraphResult, GraphSnapshot, GraphSnapshotId, GraphUniverseId,
};

pub use neighbors::{
    GRAPH_NEIGHBOR_READ_SCHEMA, GraphNeighborReadLimits, GraphNeighborReadQuery,
    GraphNeighborTruncation, GraphNeighborView,
};

pub const GRAPH_ENTITY_QUERY_SCHEMA: &str = "wow-graph/entity-exact/e2-a/1";
const MIN_OUTPUT_BYTES: u32 = 16_384;
const MAX_OUTPUT_BYTES: u32 = 8_388_608;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphEntityQuery {
    snapshot_id: GraphSnapshotId,
    node_id: GraphNodeId,
    max_output_bytes: u32,
}
impl GraphEntityQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        node_id: GraphNodeId,
        max_output_bytes: u32,
    ) -> GraphResult<Self> {
        let query = Self {
            snapshot_id,
            node_id,
            max_output_bytes,
        };
        query.validate()?;
        Ok(query)
    }
    pub fn validate(&self) -> GraphResult<()> {
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        GraphNodeId::new(self.node_id.as_str())?;
        validate_output_limit(self.max_output_bytes)
    }
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn node_id(&self) -> &GraphNodeId {
        &self.node_id
    }
    #[must_use]
    pub const fn max_output_bytes(&self) -> u32 {
        self.max_output_bytes
    }

    /// A missing materialized ID is an explicit, non-authoritative observation.
    /// The current graph schema has relation coverage, not entity-kind coverage.
    pub fn execute<'a>(
        &self,
        snapshot: &'a GraphSnapshot,
        stop: &AtomicBool,
    ) -> GraphResult<GraphEntityResult<'a>> {
        checkpoint(stop)?;
        self.validate()?;
        admit_snapshot(&self.snapshot_id, snapshot, stop)?;
        let node = snapshot.node(&self.node_id);
        let result = GraphEntityResult {
            schema: GRAPH_ENTITY_QUERY_SCHEMA,
            query: self.clone(),
            query_digest: query_digest(GRAPH_ENTITY_QUERY_SCHEMA, self)?,
            universe: snapshot.universe(),
            generation: snapshot.generation(),
            node,
            lookup: if node.is_some() {
                GraphEntityLookup::Found
            } else {
                GraphEntityLookup::NotFoundWithPartialCoverage
            },
            state: if node.is_some() {
                GraphQueryState::Complete
            } else {
                GraphQueryState::NotEvaluated
            },
            no_new_evidence: node.is_none(),
            absence_authoritative: false,
            boundaries: [
                "entity_kind_coverage_not_retained",
                "external_evidence_and_conflicts_not_resolved",
            ],
        };
        ensure_fits(&result, self.max_output_bytes as usize, stop)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphEntityLookup {
    Found,
    NotFoundWithPartialCoverage,
}

/// Borrowed original record, not a publishable graph or a confidence promotion.
#[derive(Debug, Serialize)]
pub struct GraphEntityResult<'a> {
    schema: &'static str,
    query: GraphEntityQuery,
    query_digest: Box<str>,
    universe: &'a GraphUniverseId,
    generation: &'a GraphGenerationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    node: Option<&'a GraphNode>,
    lookup: GraphEntityLookup,
    state: GraphQueryState,
    no_new_evidence: bool,
    absence_authoritative: bool,
    boundaries: [&'static str; 2],
}
impl<'a> GraphEntityResult<'a> {
    #[must_use]
    pub fn query(&self) -> &GraphEntityQuery {
        &self.query
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }
    #[must_use]
    pub const fn node(&self) -> Option<&'a GraphNode> {
        self.node
    }
    #[must_use]
    pub const fn lookup(&self) -> GraphEntityLookup {
        self.lookup
    }
    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.state
    }
    #[must_use]
    pub const fn no_new_evidence(&self) -> bool {
        self.no_new_evidence
    }
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }
    #[must_use]
    pub fn boundaries(&self) -> &[&'static str] {
        &self.boundaries
    }
}

fn validate_output_limit(limit: u32) -> GraphResult<()> {
    if !(MIN_OUTPUT_BYTES..=MAX_OUTPUT_BYTES).contains(&limit) {
        return Err(invalid(
            "direct read output limit is outside the bounded profile",
        ));
    }
    Ok(())
}
fn admit_snapshot(
    id: &GraphSnapshotId,
    snapshot: &GraphSnapshot,
    stop: &AtomicBool,
) -> GraphResult<()> {
    if id != snapshot.snapshot_id() {
        return Err(GraphError::new(
            GraphErrorCode::SnapshotIdentityMismatch,
            "direct read names another graph snapshot",
        ));
    }
    snapshot.validate()?;
    checkpoint(stop)
}
fn checkpoint(stop: &AtomicBool) -> GraphResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "graph direct read cancelled",
        ))
    } else {
        Ok(())
    }
}
fn invalid(message: &'static str) -> GraphError {
    GraphError::new(GraphErrorCode::QueryInvalid, message)
}
fn budget() -> GraphError {
    GraphError::new(
        GraphErrorCode::BudgetExceeded,
        "graph direct read exceeds its bounded profile",
    )
}
fn query_digest(schema: &str, query: &impl Serialize) -> GraphResult<Box<str>> {
    let bytes = canonical_json_bytes(&(schema, query))
        .map_err(|_| invalid("direct query encoding failed"))?;
    let hash = Sha256::digest(bytes);
    let hex = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("graph-direct-query:sha256:{hex}").into_boxed_str())
}

/// Count ordinary compact JSON before allocating canonical bytes. Key ordering
/// does not change length for these typed records. Oversize items return None.
fn encoded_len(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> GraphResult<Option<usize>> {
    struct Counter<'a> {
        length: usize,
        limit: usize,
        exceeded: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(Ordering::Acquire) {
                return Err(std::io::Error::other("graph read cancelled"));
            }
            if bytes.len() > self.limit.saturating_sub(self.length) {
                self.exceeded = true;
                return Err(std::io::Error::other("graph read output bound"));
            }
            self.length += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        length: 0,
        limit,
        exceeded: false,
        stop,
    };
    let result = serde_json::to_writer(&mut counter, value);
    checkpoint(stop)?;
    if counter.exceeded {
        return Ok(None);
    }
    result.map_err(|_| invalid("direct read encoding failed"))?;
    Ok(Some(counter.length))
}
fn ensure_fits(value: &impl Serialize, limit: usize, stop: &AtomicBool) -> GraphResult<()> {
    encoded_len(value, limit, stop)?.ok_or_else(budget)?;
    checkpoint(stop)?;
    let bytes =
        canonical_json_bytes(value).map_err(|_| invalid("direct read canonicalization failed"))?;
    if bytes.len() > limit {
        return Err(budget());
    }
    checkpoint(stop)
}
