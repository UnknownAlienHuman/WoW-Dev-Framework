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
    xml_nodes: Vec<XmlNode>,
    lua_nodes: Vec<LuaNode>,
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
            schema: "wow-service/graph-build-request/3",
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
struct XmlNode {
    occurrence_id: String,
    path: String,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
struct LuaNode {
    declaration_id: String,
    path: String,
    span: wow_core::SourceSpan,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub struct GraphBuildResult {
    schema: &'static str,
    request: GraphBuildRequest,
    request_digest: Box<str>,
    status: GraphReadStatus,
    file_nodes: Vec<FileNode>,
    xml_nodes: Vec<XmlNode>,
    lua_nodes: Vec<LuaNode>,
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
        self.xml_nodes.clear();
        self.lua_nodes.clear();
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
        schema: "wow-service/graph-build-result/3",
        request: request.clone(),
        request_digest,
        status: GraphReadStatus::Partial,
        file_nodes: Vec::new(),
        xml_nodes: Vec::new(),
        lua_nodes: Vec::new(),
        snapshot: None,
        snapshot_input_digest: None,
        provenance: None,
        failure: None,
        result_digest: None,
        boundaries: vec![
            "captured_files_loads_xml_and_main_mixin_source_topology_only",
            "not_coherent_project_store_publication",
            "package_dependencies_not_evaluated",
            "lua_calls_and_recognizers_not_evaluated",
            "xml_runtime_objects_parentage_and_mixin_execution_not_evaluated",
            "library_mixin_targets_not_projected",
            "no_negative_authority",
        ],
    };
    match compose(input, request, stop) {
        Ok(BuiltGraph {
            snapshot,
            provenance,
            digest,
            file_nodes,
            xml_nodes,
            lua_nodes,
        }) => {
            result.file_nodes = file_nodes;
            result.xml_nodes = xml_nodes;
            result.lua_nodes = lua_nodes;
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
    let mut file_nodes = Vec::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        file_nodes.push(FileNode {
            path: file.path.clone(),
            node_id: materialized_node_id(&snapshot, &file.proposal_id, limits)?,
        });
    }
    let mut xml_nodes = Vec::new();
    for declaration in provenance.xml_declarations() {
        checkpoint(stop)?;
        xml_nodes.push(XmlNode {
            occurrence_id: declaration.occurrence_id.clone(),
            path: declaration.path.clone(),
            node_id: materialized_node_id(&snapshot, &declaration.proposal_id, limits)?,
        });
    }
    let mut lua_nodes = Vec::new();
    for declaration in provenance.lua_declarations() {
        checkpoint(stop)?;
        lua_nodes.push(LuaNode {
            declaration_id: declaration.declaration_id.clone(),
            path: declaration.path.clone(),
            span: declaration.span,
            node_id: materialized_node_id(&snapshot, &declaration.proposal_id, limits)?,
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
        xml_nodes,
        lua_nodes,
    })
}

/// Rebind the accepted semantic key, never search materialized nodes by name.
fn materialized_node_id(
    snapshot: &GraphPartitionSnapshot,
    proposal_id: &str,
    limits: wow_graph::GraphLimits,
) -> ServiceResult<wow_graph::GraphNodeId> {
    let partition = snapshot
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_entities();
    let index = accepted
        .binary_search_by(|entry| entry.proposal_id().cmp(proposal_id))
        .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = accepted[index].node();
    let node = wow_graph::GraphNode::new(
        snapshot.snapshot().universe().clone(),
        snapshot.snapshot().generation().clone(),
        accepted.kind(),
        accepted.owner_key(),
        accepted.evidence_ids().to_vec(),
        limits,
    )
    .map_err(graph_error)?;
    if snapshot.snapshot().node(node.node_id()).is_none() {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    Ok(node.node_id().clone())
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
