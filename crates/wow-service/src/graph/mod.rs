//! Read-only application seam for one explicitly supplied retained graph artifact.
//! No ProjectStore, implicit current selector or analyzer. Source read-back needs
//! the distinct explicit source-root route; metadata-only reads never open sources.
mod build;
mod bundle;
pub use bundle::GRAPH_BUNDLE_MAX_BYTES;
mod input;
mod sources;
pub use build::{GraphBuildRequest, GraphBuildResult, execute_graph_build};
pub use wow_project::graph::ProjectSourceReadLimits;

use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{GenerationContextId, canonical_json_bytes};
use wow_graph::GraphPartitionSnapshot;

// Applications depend on this seam, never directly on the graph implementation.
pub use wow_graph::{
    GraphAxis, GraphAxisProfile, GraphAxisQuery, GraphAxisTraversal, GraphDirection, GraphEdgeId,
    GraphEntityQuery, GraphErrorCode, GraphEvidenceResolveLimits, GraphExplainLimits,
    GraphExplainQuery, GraphExplainSubject, GraphGenerationId, GraphNeighborQuery,
    GraphNeighborReadLimits, GraphNeighborReadQuery, GraphNodeId, GraphPathConfidence,
    GraphPathCursor, GraphPathLimits, GraphPathQuery, GraphQueryState, GraphRelationDirection,
    GraphRelationKind, GraphSnapshotId, GraphSubgraphLimits, GraphSubgraphQuery, GraphUniverseId,
};

pub const GRAPH_READ_REQUEST_SCHEMA: &str = "wow-service/graph-read-request/1";
pub const GRAPH_READ_RESULT_SCHEMA: &str = "wow-service/graph-read-result/1";
pub const GRAPH_INPUT_MAX_BYTES: usize = 16 * 1024 * 1024;
pub const GRAPH_REQUEST_MAX_BYTES: usize = 64 * 1024;
/// Owner payloads allow 8 MiB. The full envelope has a separate hard ceiling.
pub const GRAPH_RESULT_MAX_BYTES: usize = 8 * 1024 * 1024 + 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReadOperation {
    Subgraph,
    Axis,
    Explain,
    Path,
    Entity,
    Neighbors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum GraphReadQuery {
    Subgraph(GraphSubgraphQuery),
    Axis(GraphAxisQuery),
    Explain(GraphExplainQuery),
    Path(GraphPathReadQuery),
    Entity(GraphEntityQuery),
    Neighbors(GraphNeighborReadQuery),
}
impl GraphReadQuery {
    #[must_use]
    pub const fn operation(&self) -> GraphReadOperation {
        match self {
            Self::Subgraph(_) => GraphReadOperation::Subgraph,
            Self::Axis(_) => GraphReadOperation::Axis,
            Self::Explain(_) => GraphReadOperation::Explain,
            Self::Path(_) => GraphReadOperation::Path,
            Self::Entity(_) => GraphReadOperation::Entity,
            Self::Neighbors(_) => GraphReadOperation::Neighbors,
        }
    }
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        match self {
            Self::Subgraph(query) => query.snapshot_id(),
            Self::Axis(query) => query.snapshot_id(),
            Self::Explain(query) => query.snapshot_id(),
            Self::Path(request) => request.query().snapshot_id(),
            Self::Entity(query) => query.snapshot_id(),
            Self::Neighbors(query) => query.snapshot_id(),
        }
    }
}

/// One page of a graph-owned path query. The cursor is supplied explicitly and
/// validated by the path owner against the entire unchanged query and snapshot.
/// It is not a durable lease, an authorization token, or an instruction to retry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPathReadQuery {
    query: GraphPathQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    continuation: Option<GraphPathCursor>,
}
impl GraphPathReadQuery {
    #[must_use]
    pub fn new(query: GraphPathQuery) -> Self {
        Self {
            query,
            continuation: None,
        }
    }

    /// Attach the exact cursor returned by the previous page. Does not change
    /// any query limit or policy; execution rejects mismatched/invalid cursors.
    #[must_use]
    pub fn with_continuation(mut self, continuation: GraphPathCursor) -> Self {
        self.continuation = Some(continuation);
        self
    }

    #[must_use]
    pub fn query(&self) -> &GraphPathQuery {
        &self.query
    }

    #[must_use]
    pub fn continuation(&self) -> Option<&GraphPathCursor> {
        self.continuation.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphReadRequest {
    schema: Box<str>,
    query: GraphReadQuery,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence_limits: Option<GraphEvidenceResolveLimits>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_limits: Option<ProjectSourceReadLimits>,
}
impl GraphReadRequest {
    #[must_use]
    pub fn new(query: GraphReadQuery) -> Self {
        Self {
            schema: GRAPH_READ_REQUEST_SCHEMA.into(),
            query,
            evidence_limits: None,
            source_limits: None,
        }
    }
    pub fn with_evidence_limits(
        mut self,
        limits: GraphEvidenceResolveLimits,
    ) -> ServiceResult<Self> {
        limits.validate().map_err(|_| {
            ServiceError::new(ServiceErrorCode::InvalidRequest, "invalid evidence limits")
        })?;
        self.evidence_limits = Some(limits);
        Ok(self)
    }
    pub fn with_source_limits(mut self, limits: ProjectSourceReadLimits) -> ServiceResult<Self> {
        limits.validate().map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::InvalidRequest,
                "invalid source read limits",
            )
        })?;
        self.source_limits = Some(limits);
        Ok(self)
    }
    #[must_use]
    pub fn query(&self) -> &GraphReadQuery {
        &self.query
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReadStatus {
    Complete,
    Partial,
    NotEvaluated,
    Truncated,
    Failed,
    Cancelled,
}
impl From<GraphQueryState> for GraphReadStatus {
    fn from(state: GraphQueryState) -> Self {
        match state {
            GraphQueryState::Complete => Self::Complete,
            GraphQueryState::Partial => Self::Partial,
            GraphQueryState::NotEvaluated => Self::NotEvaluated,
            GraphQueryState::Truncated => Self::Truncated,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphReadStage {
    Request,
    Snapshot,
    Bundle,
    Evidence,
    Source,
    Query,
    Encoding,
    Cancellation,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphReadFailure {
    pub stage: GraphReadStage,
    pub code: ServiceErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_code: Option<GraphErrorCode>,
}
impl GraphReadFailure {
    fn service(stage: GraphReadStage, code: ServiceErrorCode) -> Self {
        Self {
            stage,
            code,
            owner_code: None,
        }
    }
    fn owner(stage: GraphReadStage, error: wow_graph::GraphError) -> Self {
        let code = match error.code() {
            GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
            GraphErrorCode::AxisUnsupported => {
                ServiceErrorCode::OperationNotImplementedForMilestone
            }
            GraphErrorCode::SnapshotIdentityMismatch
            | GraphErrorCode::AxisProfileIdentityMismatch
            | GraphErrorCode::RegistryIdentityMismatch => ServiceErrorCode::IdentityMismatch,
            _ => ServiceErrorCode::InvalidRequest,
        };
        Self {
            stage,
            code,
            owner_code: Some(error.code()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphReadContext {
    pub snapshot_id: GraphSnapshotId,
    pub universe: GraphUniverseId,
    pub generation: GraphGenerationId,
    pub source_context_id: GenerationContextId,
    pub registry_digest: Box<str>,
}

#[derive(Debug, Clone, Serialize)]
struct Envelope {
    schema: &'static str,
    operation: GraphReadOperation,
    status: GraphReadStatus,
    /// Exact accepted transport bytes; paths and host metadata are never included.
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_input_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_input_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bundle_result_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    evidence_catalog_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_boundaries: Option<Vec<Box<str>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_input_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context: Option<GraphReadContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<GraphReadFailure>,
    // Full project/evidence authority is not provided by an imported snapshot.
    absence_authoritative: bool,
    boundaries: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphReadResult {
    #[serde(flatten)]
    envelope: Envelope,
    result_digest: Box<str>,
}
impl GraphReadResult {
    #[must_use]
    pub const fn status(&self) -> GraphReadStatus {
        self.envelope.status
    }
    #[must_use]
    pub const fn operation(&self) -> GraphReadOperation {
        self.envelope.operation
    }
    #[must_use]
    pub fn context(&self) -> Option<&GraphReadContext> {
        self.envelope.context.as_ref()
    }
    #[must_use]
    pub fn payload(&self) -> Option<&serde_json::Value> {
        self.envelope.payload.as_ref()
    }
    #[must_use]
    pub fn failure(&self) -> Option<&GraphReadFailure> {
        self.envelope.failure.as_ref()
    }
    #[must_use]
    pub fn boundaries(&self) -> &[&'static str] {
        &self.envelope.boundaries
    }
    /// Replace an as-yet-unwritten result after a late signal, without another
    /// operation, owner acquisition or analysis call.
    pub fn into_cancelled(mut self) -> ServiceResult<Self> {
        self.envelope.payload = None;
        self.envelope.status = GraphReadStatus::Cancelled;
        self.envelope.failure = Some(GraphReadFailure::service(
            GraphReadStage::Cancellation,
            ServiceErrorCode::Cancelled,
        ));
        self.result_digest = hash(&encode(&self.envelope).map_err(encoding_error)?);
        Ok(self)
    }
    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        encode(self).map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::CanonicalizationFailed,
                "graph result encoding failed or exceeded the envelope byte limit",
            )
        })
    }
}

/// One synchronous service invocation. The application supplies bounded file
/// bytes, not filesystem handles. Snapshot reconstruction/queries stay in graph.
/// Result encoding finishes while the immutable owner is retained; only values
/// escape. No durable effect, publication, retry, or cross-process lease occurs.
pub fn execute_graph_read(
    operation: GraphReadOperation,
    snapshot_bytes: &[u8],
    request_bytes: &[u8],
    stop: &AtomicBool,
) -> ServiceResult<GraphReadResult> {
    execute_read(operation, snapshot_bytes, request_bytes, false, None, stop)
}

/// Read an explicit graph-build bundle with independent graph, manifest and
/// evidence admission. `explain` resolves source records; other operations keep
/// their original payload shape. No source analysis, store or current selection.
pub fn execute_graph_bundle_read(
    operation: GraphReadOperation,
    bundle_bytes: &[u8],
    request_bytes: &[u8],
    stop: &AtomicBool,
) -> ServiceResult<GraphReadResult> {
    execute_read(operation, bundle_bytes, request_bytes, true, None, stop)
}

/// Explain one admitted bundle and read back only the resolved source handles
/// under an explicitly selected Main source root. No source execution or reindex.
/// The root is private transport configuration, never part of semantic output.
pub fn execute_graph_bundle_source_read(
    operation: GraphReadOperation,
    bundle_bytes: &[u8],
    request_bytes: &[u8],
    source_root: &std::path::Path,
    stop: &AtomicBool,
) -> ServiceResult<GraphReadResult> {
    execute_read(
        operation,
        bundle_bytes,
        request_bytes,
        true,
        Some(source_root),
        stop,
    )
}

fn execute_read(
    operation: GraphReadOperation,
    artifact_bytes: &[u8],
    request_bytes: &[u8],
    is_bundle: bool,
    source_root: Option<&std::path::Path>,
    stop: &AtomicBool,
) -> ServiceResult<GraphReadResult> {
    let mut envelope = Envelope {
        schema: if source_root.is_some() {
            "wow-service/graph-source-read-result/1"
        } else if is_bundle {
            "wow-service/graph-bundle-read-result/1"
        } else {
            GRAPH_READ_RESULT_SCHEMA
        },
        operation,
        status: GraphReadStatus::Failed,
        snapshot_input_digest: None,
        bundle_input_digest: None,
        bundle_result_digest: None,
        evidence_catalog_digest: None,
        input_boundaries: None,
        request_input_digest: None,
        request_digest: None,
        context: None,
        payload: None,
        failure: None,
        absence_authoritative: false,
        boundaries: if source_root.is_some() {
            vec![
                "imported_graph_build_bundle",
                "project_publication_not_acquired",
                "explicit_local_source_root",
                "only_returned_source_handles_read_back",
                "unselected_source_bytes_and_runtime_not_verified",
                "filesystem_snapshot_not_acquired",
                "source_excerpts_are_untrusted_data",
                "build_sidecars_integrity_checked_not_semantically_revalidated",
                "content_integrity_is_not_provenance_authentication",
            ]
        } else if is_bundle {
            vec![
                "imported_graph_build_bundle",
                "project_publication_not_acquired",
                "source_bytes_and_runtime_not_verified",
                "build_sidecars_integrity_checked_not_semantically_revalidated",
                "content_integrity_is_not_provenance_authentication",
            ]
        } else {
            vec![
                "standalone_retained_graph",
                "project_publication_not_acquired",
                "external_evidence_and_runtime_not_evaluated",
            ]
        },
    };
    if request_bytes.len() <= GRAPH_REQUEST_MAX_BYTES {
        envelope.request_input_digest = Some(hash(request_bytes));
    }
    let outcome = run(
        &mut envelope,
        artifact_bytes,
        request_bytes,
        is_bundle,
        source_root,
        stop,
    );
    // Cancellation wins over a decoder/owner error caused by the same signal.
    let outcome = checkpoint(stop).and(outcome);
    if let Err(error) = outcome {
        envelope.status = if error.code == ServiceErrorCode::Cancelled {
            GraphReadStatus::Cancelled
        } else {
            GraphReadStatus::Failed
        };
        envelope.payload = None;
        envelope.failure = Some(error);
    }
    let result_digest = hash(&encode(&envelope).map_err(encoding_error)?);
    let result = GraphReadResult {
        envelope,
        result_digest,
    };
    // Include the digest field and all metadata in the hard transport ceiling.
    result.canonical_bytes()?;
    Ok(result)
}

fn run(
    envelope: &mut Envelope,
    snapshot_bytes: &[u8],
    request_bytes: &[u8],
    is_bundle: bool,
    source_root: Option<&std::path::Path>,
    stop: &AtomicBool,
) -> Result<(), GraphReadFailure> {
    checkpoint(stop)?;
    let request: GraphReadRequest = input::decode(
        request_bytes,
        GRAPH_REQUEST_MAX_BYTES,
        16_384,
        GraphReadStage::Request,
        stop,
    )?;
    if request.schema.as_ref() != GRAPH_READ_REQUEST_SCHEMA
        || request.query.operation() != envelope.operation
    {
        return Err(GraphReadFailure::service(
            GraphReadStage::Request,
            ServiceErrorCode::InvalidRequest,
        ));
    }
    if (source_root.is_some() && (!is_bundle || envelope.operation != GraphReadOperation::Explain))
        || (request.source_limits.is_some() && source_root.is_none())
    {
        return Err(GraphReadFailure::service(
            GraphReadStage::Request,
            ServiceErrorCode::InvalidRequest,
        ));
    }
    if let Some(limits) = request.source_limits {
        limits.validate().map_err(|_| {
            GraphReadFailure::service(GraphReadStage::Request, ServiceErrorCode::InvalidRequest)
        })?;
    }
    if request.evidence_limits.is_some()
        && (!is_bundle || envelope.operation != GraphReadOperation::Explain)
    {
        return Err(GraphReadFailure::service(
            GraphReadStage::Request,
            ServiceErrorCode::InvalidRequest,
        ));
    }
    if let Some(limits) = request.evidence_limits {
        limits
            .validate()
            .map_err(|error| GraphReadFailure::owner(GraphReadStage::Request, error))?;
    }
    envelope.request_digest = Some(hash(&encode(&request)?));
    if is_bundle {
        if snapshot_bytes.len() <= GRAPH_BUNDLE_MAX_BYTES {
            envelope.bundle_input_digest = Some(hash(snapshot_bytes));
        }
        let admitted = bundle::admit(snapshot_bytes, stop)?;
        envelope.snapshot_input_digest = Some(admitted.snapshot_digest);
        envelope.bundle_result_digest = Some(admitted.result_digest);
        envelope.evidence_catalog_digest = Some(admitted.evidence.digest().into());
        envelope.input_boundaries = Some(admitted.boundaries);
        return run_query(
            envelope,
            &admitted.owner,
            &request,
            Some(&admitted.evidence),
            source_root.map(|root| sources::SourceRead {
                root,
                manifest: &admitted.sources,
                limits: request.source_limits.unwrap_or_default(),
            }),
            stop,
        );
    }
    let owner: GraphPartitionSnapshot = input::decode(
        snapshot_bytes,
        GRAPH_INPUT_MAX_BYTES,
        1_000_000,
        GraphReadStage::Snapshot,
        stop,
    )?;
    envelope.snapshot_input_digest = Some(hash(snapshot_bytes));
    run_query(envelope, &owner, &request, None, None, stop)
}

fn run_query(
    envelope: &mut Envelope,
    owner: &GraphPartitionSnapshot,
    request: &GraphReadRequest,
    evidence: Option<&wow_graph::GraphEvidenceCatalog>,
    source_read: Option<sources::SourceRead<'_>>,
    stop: &AtomicBool,
) -> Result<(), GraphReadFailure> {
    if owner.snapshot().snapshot_id() != request.query.snapshot_id() {
        return Err(GraphReadFailure::service(
            GraphReadStage::Snapshot,
            ServiceErrorCode::IdentityMismatch,
        ));
    }
    // Subgraph/path/direct APIs validate only the materialized graph. Also validate
    // its complete imported partition owner, including every producer report.
    // Axis/explain already perform that full validation in their execute.
    if matches!(
        &request.query,
        GraphReadQuery::Subgraph(_)
            | GraphReadQuery::Path(_)
            | GraphReadQuery::Entity(_)
            | GraphReadQuery::Neighbors(_)
    ) {
        owner
            .validate(stop)
            .map_err(|e| GraphReadFailure::owner(GraphReadStage::Snapshot, e))?;
    }
    let (status, payload) = match &request.query {
        GraphReadQuery::Entity(query) => {
            let result = query
                .execute(owner.snapshot(), stop)
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            (result.state().into(), value(&result)?)
        }
        GraphReadQuery::Neighbors(query) => {
            let result = query
                .execute(owner.snapshot(), stop)
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            (result.state().into(), value(&result)?)
        }
        GraphReadQuery::Subgraph(query) => {
            let result = query
                .execute(owner.snapshot(), stop)
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            (result.state().into(), value(&result)?)
        }
        GraphReadQuery::Path(request) => {
            // Exactly one page. The owner checks the cursor and charges replay
            // work against the original expansion ceiling; never chase cursors.
            let result = request
                .query()
                .execute(owner.snapshot(), request.continuation(), stop)
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            (result.state().into(), value(&result)?)
        }
        GraphReadQuery::Axis(query) => {
            // No "latest" profile or first registry match. The owner binds its
            // reviewed recipe; the query's exact digest must match that binding.
            let profile = GraphAxisProfile::bind(owner.registry(), query.axis())
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            let result = query
                .execute(owner, &profile, stop)
                .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
            (result.state().into(), value(&result)?)
        }
        GraphReadQuery::Explain(query) => {
            if let Some(catalog) = evidence {
                let result = query
                    .execute_with_evidence(
                        owner,
                        catalog,
                        request.evidence_limits.unwrap_or_default(),
                        stop,
                    )
                    .map_err(|e| GraphReadFailure::owner(GraphReadStage::Evidence, e))?;
                let state = if !result.explanation().truncations().is_empty()
                    || !result.evidence_resolution().truncations().is_empty()
                {
                    GraphReadStatus::Truncated
                } else {
                    // Core source evidence closure alone does not close conflicts,
                    // graph derivation records, runtime or project publication.
                    GraphReadStatus::Partial
                };
                if let Some(source_read) = source_read {
                    let (payload, truncated) = source_read.execute(
                        &result,
                        catalog,
                        query.limits().max_output_bytes as usize,
                        stop,
                    )?;
                    (
                        if truncated {
                            GraphReadStatus::Truncated
                        } else {
                            state
                        },
                        payload,
                    )
                } else {
                    (state, value(&result)?)
                }
            } else {
                let result = query
                    .execute(owner, stop)
                    .map_err(|e| GraphReadFailure::owner(GraphReadStage::Query, e))?;
                let state = if !result.truncations().is_empty() {
                    GraphReadStatus::Truncated
                } else if !result.support_complete() || !result.boundaries().is_empty() {
                    GraphReadStatus::Partial
                } else {
                    GraphReadStatus::Complete
                };
                (state, value(&result)?)
            }
        }
    };
    checkpoint(stop)?;
    envelope.context = Some(GraphReadContext {
        snapshot_id: owner.snapshot().snapshot_id().clone(),
        universe: owner.snapshot().universe().clone(),
        generation: owner.snapshot().generation().clone(),
        source_context_id: owner.source_context_id(),
        registry_digest: owner.registry().registry_digest().into(),
    });
    envelope.status = if evidence.is_some() && status == GraphReadStatus::Complete {
        GraphReadStatus::Partial
    } else {
        status
    };
    envelope.payload = Some(payload);
    Ok(())
}

fn checkpoint(stop: &AtomicBool) -> Result<(), GraphReadFailure> {
    if stop.load(Ordering::Acquire) {
        Err(GraphReadFailure::service(
            GraphReadStage::Cancellation,
            ServiceErrorCode::Cancelled,
        ))
    } else {
        Ok(())
    }
}
fn hash(bytes: &[u8]) -> Box<str> {
    format!("sha256:{:x}", Sha256::digest(bytes)).into()
}
fn encode(value: &impl Serialize) -> Result<Vec<u8>, GraphReadFailure> {
    // Adding bundle metadata must not allocate an oversized canonical result
    // before noticing the envelope cap. Count compact typed JSON first.
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .filter(|n| *n <= GRAPH_RESULT_MAX_BYTES)
                .ok_or_else(|| std::io::Error::other("graph envelope byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(&mut Counter(0), value).map_err(|_| {
        GraphReadFailure::service(GraphReadStage::Encoding, ServiceErrorCode::BudgetExceeded)
    })?;
    let bytes = canonical_json_bytes(value).map_err(|_| {
        GraphReadFailure::service(
            GraphReadStage::Encoding,
            ServiceErrorCode::CanonicalizationFailed,
        )
    })?;
    if bytes.len() > GRAPH_RESULT_MAX_BYTES {
        return Err(GraphReadFailure::service(
            GraphReadStage::Encoding,
            ServiceErrorCode::BudgetExceeded,
        ));
    }
    Ok(bytes)
}
fn value(payload: &impl Serialize) -> Result<serde_json::Value, GraphReadFailure> {
    // The owner's exact canonical payload is copied, not reinterpreted or folded.
    serde_json::from_slice(&encode(payload)?).map_err(|_| {
        GraphReadFailure::service(
            GraphReadStage::Encoding,
            ServiceErrorCode::CanonicalizationFailed,
        )
    })
}
fn encoding_error(_: GraphReadFailure) -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::CanonicalizationFailed,
        "graph envelope encoding failed",
    )
}
