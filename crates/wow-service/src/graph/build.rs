//! One-shot composition of project-owned source proposals and graph materialization.
//! Emits artifacts only; no current pointer, ProjectStore write or source mutation.
use super::{GRAPH_INPUT_MAX_BYTES, GraphReadStage, GraphReadStatus};
use crate::{
    GenerationSelector, LocalProjectBackend, LocalProjectInput, ServiceError, ServiceErrorCode,
    ServiceResult,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_graph::{GraphPartitionReplacement, GraphPartitionSnapshot, GraphSnapshot};
use wow_project::graph::{
    ProjectGraphProvenance, SOURCE_GRAPH_PROFILE, build_source_graph_proposals,
};

const MAX_BUNDLE_BYTES: usize = 32 * 1024 * 1024;

struct BuiltGraph {
    snapshot: GraphPartitionSnapshot,
    provenance: ProjectGraphProvenance,
    digest: Box<str>,
    file_nodes: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphBuildRequest {
    schema: &'static str,
    project_id: String,
    selector: GenerationSelector,
    projection: &'static str,
}
impl GraphBuildRequest {
    pub fn new(project_id: String, generation: String) -> ServiceResult<Self> {
        wow_project::ProjectId::new(project_id.as_str())
            .map_err(|_| error(ServiceErrorCode::InvalidRequest))?;
        let selector = if generation == "current" {
            GenerationSelector::current_published(project_id.clone())?
        } else {
            generation
                .parse::<wow_core::ProjectGenerationId>()
                .map_err(|_| error(ServiceErrorCode::InvalidRequest))?;
            GenerationSelector::exact(generation)?
        };
        Ok(Self {
            schema: "wow-service/graph-build-request/1",
            project_id,
            selector,
            projection: SOURCE_GRAPH_PROFILE,
        })
    }
}

#[derive(Debug, Serialize)]
struct FileNode {
    path: String,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub struct GraphBuildResult {
    schema: &'static str,
    request: GraphBuildRequest,
    request_digest: Box<str>,
    status: GraphReadStatus,
    file_nodes: Vec<FileNode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot: Option<GraphPartitionSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_input_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<ProjectGraphProvenance>,
    boundaries: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<ServiceErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_digest: Option<Box<str>>,
}
impl GraphBuildResult {
    #[must_use]
    pub const fn status(&self) -> GraphReadStatus {
        self.status
    }

    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        bounded(self, MAX_BUNDLE_BYTES)
    }

    /// The exact bare partition snapshot accepted by graph-read commands. Error
    /// and cancellation results never emit an empty or substituted graph artifact.
    pub fn snapshot_bytes(&self) -> ServiceResult<Option<Vec<u8>>> {
        self.snapshot
            .as_ref()
            .map(|s| bounded(s, GRAPH_INPUT_MAX_BYTES))
            .transpose()
    }

    pub fn into_cancelled(mut self) -> ServiceResult<Self> {
        self.fail(ServiceErrorCode::Cancelled);
        self.seal()?;
        Ok(self)
    }

    fn fail(&mut self, code: ServiceErrorCode) {
        self.file_nodes.clear();
        self.snapshot = None;
        self.snapshot_input_digest = None;
        self.provenance = None;
        self.failure = Some(code);
        self.status = if code == ServiceErrorCode::Cancelled {
            GraphReadStatus::Cancelled
        } else {
            GraphReadStatus::Failed
        };
        self.result_digest = None;
    }

    fn seal(&mut self) -> ServiceResult<()> {
        self.result_digest = None;
        self.result_digest = Some(super::hash(&self.canonical_bytes()?));
        self.canonical_bytes()?;
        Ok(())
    }
}

/// Uses the same project materialization path as `wow check`, once. The source
/// owner creates proposals; only the graph owner validates/materializes them.
pub fn execute_graph_build(
    input: LocalProjectInput,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<GraphBuildResult> {
    let request_digest = super::hash(&bounded(request, super::GRAPH_REQUEST_MAX_BYTES)?);
    let mut result = GraphBuildResult {
        schema: "wow-service/graph-build-result/1",
        request: request.clone(),
        request_digest,
        status: GraphReadStatus::Partial,
        file_nodes: Vec::new(),
        snapshot: None,
        snapshot_input_digest: None,
        provenance: None,
        failure: None,
        result_digest: None,
        boundaries: vec![
            "direct_captured_file_loads_only",
            "not_coherent_project_store_publication",
            "package_dependencies_not_evaluated",
            "lua_calls_and_recognizers_not_evaluated",
            "xml_objects_and_runtime_not_evaluated",
            "no_negative_authority",
        ],
    };
    match compose(input, request, stop) {
        Ok(BuiltGraph {
            snapshot,
            provenance,
            digest,
            file_nodes,
        }) => {
            result.file_nodes = file_nodes;
            result.snapshot = Some(snapshot);
            result.provenance = Some(provenance);
            result.snapshot_input_digest = Some(digest);
        }
        Err(failure) => result.fail(failure.code()),
    }
    if stop.load(Ordering::Acquire) {
        result.fail(ServiceErrorCode::Cancelled);
    }
    if let Err(failure) = result.seal() {
        result.fail(failure.code());
        result.seal()?;
    }
    Ok(result)
}

fn compose(
    input: LocalProjectInput,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<BuiltGraph> {
    checkpoint(stop)?;
    let backend = LocalProjectBackend::new(input)?;
    if backend.configuration().project_id() != request.project_id {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let project = backend.acquire_project(&request.selector, stop)?;
    let proposals = build_source_graph_proposals(&project, stop).map_err(|e| {
        error(match e.code() {
            wow_project::ProjectErrorCode::AnalysisCancelled
            | wow_project::ProjectErrorCode::SourceReadCancelled => ServiceErrorCode::Cancelled,
            wow_project::ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
            _ => ServiceErrorCode::InternalContractViolation,
        })
    })?;
    let (registry, batch, coverage, provenance, limits) = proposals.into_parts();
    checkpoint(stop)?;
    // The empty foundation makes no semantic absence claim. Preserve the same
    // narrow partial/unevaluated coverage instead of inventing complete coverage.
    let foundation = GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )
    .map_err(graph_error)?;
    let owner = GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), stop)
        .map_err(graph_error)?;
    let replacement = owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch,
                coverage,
            },
            stop,
        )
        .map_err(graph_error)?;
    let snapshot = replacement.candidate().clone();
    checkpoint(stop)?;
    let partition = snapshot
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let mut file_nodes = Vec::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        let accepted = partition
            .report()
            .accepted_entities()
            .binary_search_by(|entry| entry.proposal_id().cmp(&file.proposal_id))
            .ok()
            .and_then(|index| partition.report().accepted_entities().get(index))
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        // Rebind the exact accepted semantic key through the graph constructor,
        // not a display-name/path search against materialized entities.
        let node = wow_graph::GraphNode::new(
            snapshot.snapshot().universe().clone(),
            snapshot.snapshot().generation().clone(),
            accepted.node().kind(),
            accepted.node().owner_key(),
            accepted.node().evidence_ids().to_vec(),
            limits,
        )
        .map_err(graph_error)?;
        if snapshot.snapshot().node(node.node_id()).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        file_nodes.push(FileNode {
            path: file.path.clone(),
            node_id: node.node_id().clone(),
        });
    }
    let bytes = bounded(&snapshot, GRAPH_INPUT_MAX_BYTES)?;
    // Ensure export and existing import share byte/token/depth/string limits.
    // This is runtime admission, not a test or a second project/analyzer pass.
    let admitted: GraphPartitionSnapshot = super::input::decode(
        &bytes,
        GRAPH_INPUT_MAX_BYTES,
        1_000_000,
        GraphReadStage::Snapshot,
        stop,
    )
    .map_err(|e| error(e.code))?;
    if admitted != snapshot {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    checkpoint(stop)?;
    Ok(BuiltGraph {
        snapshot,
        provenance,
        digest: super::hash(&bytes),
        file_nodes,
    })
}

fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(error(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn error(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "source graph construction could not produce a coherent bounded artifact",
    )
}
fn graph_error(e: wow_graph::GraphError) -> ServiceError {
    error(match e.code() {
        wow_graph::GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    })
}

fn bounded(value: &impl Serialize, limit: usize) -> ServiceResult<Vec<u8>> {
    struct Count {
        used: usize,
        limit: usize,
    }
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.used) {
                return Err(std::io::Error::other("graph build output limit"));
            }
            self.used += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    // Bound serialized allocation before constructing the canonical representation.
    serde_json::to_writer(Count { used: 0, limit }, value)
        .map_err(|_| error(ServiceErrorCode::BudgetExceeded))?;
    let bytes = wow_core::canonical_json_bytes(value)
        .map_err(|_| error(ServiceErrorCode::CanonicalizationFailed))?;
    if bytes.len() > limit {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
