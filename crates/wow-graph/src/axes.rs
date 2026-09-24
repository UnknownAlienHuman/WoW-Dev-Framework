//! Named read-only projections over a registry-pinned materialized graph.
//! Axis direction is presentation policy; original assertions never change.
mod profile;

pub use profile::{
    GRAPH_AXIS_PROFILE_SCHEMA, GraphAxis, GraphAxisProfile, GraphAxisRelation, GraphAxisShape,
};

use crate::{
    GraphDirection, GraphError, GraphErrorCode, GraphNodeId, GraphPartitionSnapshot,
    GraphPathConfidence, GraphQueryState, GraphRelationDirection, GraphResult, GraphSnapshotId,
    GraphSubgraphLimits, GraphSubgraphQuery, GraphSubgraphResult,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::canonical_json_bytes;

pub const GRAPH_AXIS_QUERY_SCHEMA: &str = "wow-graph/axis-query/e2-a/1";

/// Forward means the direction declared independently for each relation by the
/// axis profile, not necessarily the physical direction of every stored edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphAxisTraversal {
    Forward,
    Reverse,
    Both,
    /// One level toward parents; supported only on an explicit multi-parent axis.
    Parents,
    /// One level toward children; supported only on an explicit multi-parent axis.
    Children,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphAxisQuery {
    snapshot_id: GraphSnapshotId,
    axis: GraphAxis,
    profile_digest: Box<str>,
    roots: Vec<GraphNodeId>,
    traversal: GraphAxisTraversal,
    confidence: GraphPathConfidence,
    limits: GraphSubgraphLimits,
}

impl GraphAxisQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        profile: &GraphAxisProfile,
        mut roots: Vec<GraphNodeId>,
        traversal: GraphAxisTraversal,
        limits: GraphSubgraphLimits,
    ) -> GraphResult<Self> {
        roots.sort();
        let query = Self {
            snapshot_id,
            axis: profile.axis(),
            profile_digest: profile.digest().into(),
            roots,
            traversal,
            confidence: GraphPathConfidence::default(),
            limits,
        };
        query.validate(profile)?;
        Ok(query)
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
    pub const fn axis(&self) -> GraphAxis {
        self.axis
    }
    #[must_use]
    pub fn profile_digest(&self) -> &str {
        &self.profile_digest
    }
    #[must_use]
    pub fn roots(&self) -> &[GraphNodeId] {
        &self.roots
    }
    #[must_use]
    pub const fn traversal(&self) -> GraphAxisTraversal {
        self.traversal
    }
    #[must_use]
    pub const fn confidence(&self) -> GraphPathConfidence {
        self.confidence
    }
    #[must_use]
    pub const fn limits(&self) -> GraphSubgraphLimits {
        self.limits
    }

    fn validate(&self, profile: &GraphAxisProfile) -> GraphResult<()> {
        if self.axis != profile.axis() || self.profile_digest.as_ref() != profile.digest() {
            return Err(error(GraphErrorCode::AxisProfileIdentityMismatch));
        }
        self.limits.validate()?;
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        if self.roots.is_empty()
            || self.roots.len() > 64
            || self.roots.len() > self.limits.max_nodes as usize
            || self.roots.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(error(GraphErrorCode::QueryInvalid));
        }
        for root in &self.roots {
            GraphNodeId::new(root.as_str())?;
        }
        if matches!(
            self.traversal,
            GraphAxisTraversal::Parents | GraphAxisTraversal::Children
        ) && (profile.shape() != GraphAxisShape::MultiParent || self.limits.max_depth != 1)
        {
            return Err(error(GraphErrorCode::QueryInvalid));
        }
        Ok(())
    }

    /// Resolve against one exact immutable partition owner. Profiles are bound
    /// to that owner's registry, not supplied by source text or an implicit current.
    pub fn execute(
        &self,
        owner: &GraphPartitionSnapshot,
        profile: &GraphAxisProfile,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphAxisResult> {
        checkpoint(cancelled)?;
        self.validate(profile)?;
        if self.snapshot_id != *owner.snapshot().snapshot_id() {
            return Err(error(GraphErrorCode::SnapshotIdentityMismatch));
        }
        // Bound adjacency work before the existing owner validation rebuild.
        if owner.snapshot().edges().len() > self.limits.max_scanned_edges as usize / 2 {
            return Err(error(GraphErrorCode::BudgetExceeded));
        }
        profile.validate(owner.registry())?;
        checkpoint(cancelled)?;
        owner.validate(cancelled)?;
        let snapshot = owner.snapshot();
        for root in &self.roots {
            checkpoint(cancelled)?;
            let node = snapshot
                .node(root)
                .ok_or_else(|| error(GraphErrorCode::QueryInvalid))?;
            if owner.registry().entity_kind(node.kind()).is_none() {
                return Err(error(GraphErrorCode::RegistryInvalid));
            }
        }
        // Foundation assertions predate the registry-based producer validator.
        // Check selected stored relations against the exact admitted definitions,
        // rather than trusting an enum match to certify their endpoint semantics.
        for edge in snapshot.edges() {
            checkpoint(cancelled)?;
            let Some(step) = profile
                .relations()
                .iter()
                .find(|s| s.relation() == edge.relation())
            else {
                continue;
            };
            let definition = owner
                .registry()
                .relation_kind(step.relation_id())
                .ok_or_else(|| error(GraphErrorCode::AxisProfileInvalid))?;
            let source = snapshot
                .node(edge.from())
                .ok_or_else(|| error(GraphErrorCode::EndpointMissing))?;
            let target = snapshot
                .node(edge.to())
                .ok_or_else(|| error(GraphErrorCode::EndpointMissing))?;
            if !definition.allows_source_kind(source.kind())
                || !definition.allows_target_kind(target.kind())
                || !definition.allows_confidence(edge.confidence())
            {
                return Err(error(GraphErrorCode::RegistryInvalid));
            }
        }
        let directions = profile
            .relations()
            .iter()
            .map(|step| GraphRelationDirection {
                relation: step.relation(),
                direction: match self.traversal {
                    GraphAxisTraversal::Forward | GraphAxisTraversal::Children => {
                        step.forward_direction()
                    }
                    GraphAxisTraversal::Reverse | GraphAxisTraversal::Parents => {
                        reverse(step.forward_direction())
                    }
                    GraphAxisTraversal::Both => GraphDirection::Both,
                },
            })
            .collect();
        let header = AxisHeader {
            schema: GRAPH_AXIS_QUERY_SCHEMA,
            query: self.clone(),
            query_digest: digest("graph-axis-query:sha256:", &(GRAPH_AXIS_QUERY_SCHEMA, self))?,
            profile: profile.clone(),
            registry_scanned_edges: snapshot.edges().len() as u32,
            boundaries: vec![
                GraphAxisBoundary::StoredRelationFamiliesOnly,
                GraphAxisBoundary::ConflictAssessmentNotEvaluated,
                GraphAxisBoundary::EvidenceDereferencingNotEvaluated,
            ],
            absence_authoritative: false,
        };
        // The nested subgraph budget includes its own request and complete records.
        // Reserve the *entire* axis header as well; axes cannot escape byte limits
        // merely by wrapping a result that fit its original budget.
        let overhead = encoded(&header)?.len() + b",\"projection\":".len();
        let remaining = (self.limits.max_output_bytes as usize)
            .checked_sub(overhead)
            .ok_or_else(|| error(GraphErrorCode::BudgetExceeded))?;
        if remaining < 16_384 {
            return Err(error(GraphErrorCode::BudgetExceeded));
        }
        let limits = GraphSubgraphLimits {
            max_output_bytes: remaining as u32,
            max_scanned_edges: self.limits.max_scanned_edges - header.registry_scanned_edges,
            ..self.limits
        };
        checkpoint(cancelled)?;
        let projection = GraphSubgraphQuery::new_directed(
            self.snapshot_id.clone(),
            self.roots.clone(),
            directions,
            limits,
        )?
        .with_confidence(self.confidence)
        .execute(snapshot, cancelled)?;
        let result = GraphAxisResult { header, projection };
        if encoded(&result)?.len() > self.limits.max_output_bytes as usize {
            return Err(error(GraphErrorCode::BudgetExceeded));
        }
        checkpoint(cancelled)?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphAxisBoundary {
    StoredRelationFamiliesOnly,
    ConflictAssessmentNotEvaluated,
    EvidenceDereferencingNotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct AxisHeader {
    schema: &'static str,
    query: GraphAxisQuery,
    query_digest: Box<str>,
    profile: GraphAxisProfile,
    registry_scanned_edges: u32,
    boundaries: Vec<GraphAxisBoundary>,
    absence_authoritative: bool,
}

/// A query view, never a new graph generation or a publishable truth table.
/// Projection completeness is distinct from full axis/conflict/evidence coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphAxisResult {
    #[serde(flatten)]
    header: AxisHeader,
    projection: GraphSubgraphResult,
}
impl GraphAxisResult {
    #[must_use]
    pub fn query(&self) -> &GraphAxisQuery {
        &self.header.query
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.header.query_digest
    }
    #[must_use]
    pub fn profile(&self) -> &GraphAxisProfile {
        &self.header.profile
    }
    #[must_use]
    pub const fn projection(&self) -> &GraphSubgraphResult {
        &self.projection
    }
    /// Counts the registry-admission and adjacency-indexing passes together.
    #[must_use]
    pub fn scanned_edges(&self) -> u32 {
        self.header.registry_scanned_edges + self.projection.scanned_edges()
    }
    /// State of the exact stored-relation traversal, not complete WoW semantics.
    #[must_use]
    pub fn state(&self) -> GraphQueryState {
        self.projection.state()
    }
    #[must_use]
    pub fn boundaries(&self) -> &[GraphAxisBoundary] {
        &self.header.boundaries
    }
    #[must_use]
    pub fn no_new_evidence(&self) -> bool {
        self.projection.no_new_evidence()
    }
    /// The underlying exact-relation query may have scoped absence authority;
    /// the broader named axis cannot waive the explicit semantic boundaries.
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        false
    }
}

fn reverse(direction: GraphDirection) -> GraphDirection {
    match direction {
        GraphDirection::Outgoing => GraphDirection::Incoming,
        GraphDirection::Incoming => GraphDirection::Outgoing,
        GraphDirection::Both => GraphDirection::Both,
    }
}
fn checkpoint(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Acquire) {
        return Err(error(GraphErrorCode::Cancelled));
    }
    Ok(())
}
fn error(code: GraphErrorCode) -> GraphError {
    GraphError::new(
        code,
        "graph axis query or profile does not match its bounded owner contract",
    )
}
fn encoded<T: Serialize>(value: &T) -> GraphResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| error(GraphErrorCode::QueryInvalid))
}
fn digest<T: Serialize>(prefix: &str, value: &T) -> GraphResult<Box<str>> {
    let hash = Sha256::digest(encoded(value)?);
    let hex = hash
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(format!("{prefix}{hex}").into_boxed_str())
}
