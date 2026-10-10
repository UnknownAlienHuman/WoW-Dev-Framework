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
    ProjectGraphPackageDependencyOutcome, ProjectGraphPackageLoadOutcome, ProjectGraphProvenance,
    SOURCE_GRAPH_PROFILE, build_source_graph_proposals,
};

mod calls;
mod construction;
mod lua_mixins;
mod materialized;
mod scripts;
mod signals;
mod state;
mod state_core;
#[cfg(test)]
mod state_core_tests;
mod toc;
#[cfg(test)]
mod toc_tests;
mod xml;
#[cfg(test)]
mod xml_tests;
use calls::{CallEdge, FunctionNode};
use construction::{CreationEdge, FrameNode};
use lua_mixins::{
    AssignmentMixinEdge, ConstructionMixinEdge, InstantiationEdge, MixinInstanceNode,
};
use scripts::{HandlerNode, ScriptEdge};
use state::{StateEdge, StateNodes};
use wow_recognizers::source_calls::SourceCallRecognition;
use wow_recognizers::source_construction::SourceConstructionRecognition;
use wow_recognizers::source_mixins::{SourceMixinAssignmentRecognition, SourceMixinRecognition};
use wow_recognizers::source_scripts::SourceScriptRecognition;
use wow_recognizers::source_state::SourceStateRecognition;

const MAX_BUNDLE_BYTES: usize = 32 * 1024 * 1024;
pub(super) const GRAPH_BUILD_RESULT_SCHEMA: &str = "wow-service/graph-build-result/16";

struct BuiltGraph {
    snapshot: GraphPartitionSnapshot,
    provenance: ProjectGraphProvenance,
    call_recognition: SourceCallRecognition,
    construction_recognition: SourceConstructionRecognition,
    mixin_recognition: SourceMixinRecognition,
    mixin_assignment_recognition: SourceMixinAssignmentRecognition,
    script_recognition: SourceScriptRecognition,
    state_recognition: SourceStateRecognition,
    state_nodes: StateNodes,
    state_edges: Vec<StateEdge>,
    handler_nodes: Vec<HandlerNode>,
    script_edges: Vec<ScriptEdge>,
    digest: Box<str>,
    file_nodes: Vec<FileNode>,
    package_nodes: Vec<PackageNode>,
    package_file_edges: Vec<PackageFileEdge>,
    package_dependency_edges: Vec<PackageDependencyEdge>,
    package_load_edges: Vec<PackageLoadEdge>,
    xml_nodes: Vec<XmlNode>,
    lua_nodes: Vec<LuaNode>,
    function_nodes: Vec<FunctionNode>,
    call_edges: Vec<CallEdge>,
    frame_nodes: Vec<FrameNode>,
    creation_edges: Vec<CreationEdge>,
    mixin_instance_nodes: Vec<MixinInstanceNode>,
    instantiation_edges: Vec<InstantiationEdge>,
    construction_mixin_edges: Vec<ConstructionMixinEdge>,
    assignment_mixin_edges: Vec<AssignmentMixinEdge>,
    signal_recognition: signals::W1Recognition,
    bridge_recognition: signals::W2BridgeRecognition,
    custom_recognition: signals::W3Recognition,
    cvar_recognition: signals::W4Recognition,
    hook_recognition: signals::W5HookRecognition,
    library_recognition: signals::SourceLibraryRecognition,
    signal_nodes: Vec<signals::SignalNode>,
    signal_edges: Vec<signals::SignalEdge>,
    toc_recognition: Vec<wow_recognizers::source_toc::SourceTocRecognition>,
    toc_topology: toc::TocTopology,
    xml_recognition: Vec<wow_recognizers::source_xml::SourceXmlRecognition>,
    xml_topology: xml::XmlTopology,
    state_root_recognition: wow_recognizers::source_toc::SourceTocRecognition,
    state_root_topology: toc::TocTopology,
    state_core_recognition: Vec<wow_recognizers::source_state_core::SourceStateCoreRecognition>,
    state_core_topology: state_core::StateCoreTopology,
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
            schema: "wow-service/graph-build-request/9",
            project_id,
            selector,
            projection: SOURCE_GRAPH_PROFILE,
        })
    }

    pub(crate) fn acquire_project(
        &self,
        backend: &LocalProjectBackend,
        stop: &AtomicBool,
    ) -> ServiceResult<wow_project::ProjectView> {
        checkpoint(stop)?;
        if backend.configuration().project_id() != self.project_id {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        backend.acquire_project(&self.selector, stop)
    }

    pub(crate) fn live_publication_in_namespace(
        &self,
        input: LocalProjectInput,
        namespace: &wow_store::project::ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ServiceResult<(
        wow_project::replay::publication::ProjectPublicationBundle,
        String,
    )> {
        live_publication_in_namespace(input, self, namespace, stop)
    }

    pub(crate) fn live_publication_from_backend_in_namespace(
        &self,
        backend: &LocalProjectBackend,
        namespace: &wow_store::project::ProjectStoreNamespace,
        stop: &AtomicBool,
    ) -> ServiceResult<(
        wow_project::replay::publication::ProjectPublicationBundle,
        String,
    )> {
        live_publication_from_backend_in_namespace(backend, self, namespace, stop)
    }
}

#[derive(Debug, Serialize)]
struct FileNode {
    path: String,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
struct PackageNode {
    package: String,
    order_group: u64,
    reachability: wow_project::load::ProjectPackageReachability,
    phase: wow_project::load::ProjectPackageLoadPhase,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
struct PackageFileEdge {
    package: String,
    path: String,
    edge_id: wow_graph::GraphEdgeId,
}

#[derive(Debug, Serialize)]
struct PackageDependencyEdge {
    ordinal: u64,
    package: String,
    dependency: String,
    kind: wow_project::load::TocDependencyKind,
    confidence: wow_graph::GraphConfidence,
    edge_id: wow_graph::GraphEdgeId,
}

#[derive(Debug, Serialize)]
struct PackageLoadEdge {
    unit_digest: wow_core::ContentDigest<wow_core::CanonicalResult>,
    package: String,
    target: String,
    confidence: wow_graph::GraphConfidence,
    edge_id: wow_graph::GraphEdgeId,
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
    package_nodes: Vec<PackageNode>,
    package_file_edges: Vec<PackageFileEdge>,
    package_dependency_edges: Vec<PackageDependencyEdge>,
    package_load_edges: Vec<PackageLoadEdge>,
    xml_nodes: Vec<XmlNode>,
    lua_nodes: Vec<LuaNode>,
    function_nodes: Vec<FunctionNode>,
    call_edges: Vec<CallEdge>,
    frame_nodes: Vec<FrameNode>,
    creation_edges: Vec<CreationEdge>,
    mixin_instance_nodes: Vec<MixinInstanceNode>,
    instantiation_edges: Vec<InstantiationEdge>,
    construction_mixin_edges: Vec<ConstructionMixinEdge>,
    assignment_mixin_edges: Vec<AssignmentMixinEdge>,
    handler_nodes: Vec<HandlerNode>,
    script_edges: Vec<ScriptEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    script_recognition: Option<SourceScriptRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_recognition: Option<SourceStateRecognition>,
    state_nodes: StateNodes,
    state_edges: Vec<StateEdge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    signal_recognition: Option<signals::W1Recognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bridge_recognition: Option<signals::W2BridgeRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    custom_recognition: Option<signals::W3Recognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cvar_recognition: Option<signals::W4Recognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hook_recognition: Option<signals::W5HookRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    library_recognition: Option<signals::SourceLibraryRecognition>,
    signal_nodes: Vec<signals::SignalNode>,
    signal_edges: Vec<signals::SignalEdge>,
    toc_recognition: Vec<wow_recognizers::source_toc::SourceTocRecognition>,
    toc_topology: toc::TocTopology,
    xml_recognition: Vec<wow_recognizers::source_xml::SourceXmlRecognition>,
    xml_topology: xml::XmlTopology,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_root_recognition: Option<wow_recognizers::source_toc::SourceTocRecognition>,
    state_root_topology: toc::TocTopology,
    state_core_recognition: Vec<wow_recognizers::source_state_core::SourceStateCoreRecognition>,
    state_core_topology: state_core::StateCoreTopology,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot: Option<GraphPartitionSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_input_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<ProjectGraphProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    call_recognition: Option<SourceCallRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    construction_recognition: Option<SourceConstructionRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mixin_recognition: Option<SourceMixinRecognition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mixin_assignment_recognition: Option<SourceMixinAssignmentRecognition>,
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
        self.package_nodes.clear();
        self.package_file_edges.clear();
        self.package_dependency_edges.clear();
        self.package_load_edges.clear();
        self.xml_nodes.clear();
        self.lua_nodes.clear();
        self.function_nodes.clear();
        self.call_edges.clear();
        self.frame_nodes.clear();
        self.creation_edges.clear();
        self.mixin_instance_nodes.clear();
        self.instantiation_edges.clear();
        self.construction_mixin_edges.clear();
        self.assignment_mixin_edges.clear();
        self.call_recognition = None;
        self.construction_recognition = None;
        self.mixin_recognition = None;
        self.mixin_assignment_recognition = None;
        self.script_recognition = None;
        self.state_recognition = None;
        self.state_nodes = StateNodes::empty();
        self.state_edges.clear();
        self.toc_recognition.clear();
        self.toc_topology = toc::TocTopology::default();
        self.xml_recognition.clear();
        self.xml_topology = xml::XmlTopology::default();
        self.state_root_recognition = None;
        self.state_root_topology = toc::TocTopology::default();
        self.state_core_recognition.clear();
        self.state_core_topology = state_core::StateCoreTopology::default();
        self.signal_recognition = None;
        self.bridge_recognition = None;
        self.custom_recognition = None;
        self.cvar_recognition = None;
        self.hook_recognition = None;
        self.library_recognition = None;
        self.signal_nodes.clear();
        self.signal_edges.clear();
        self.handler_nodes.clear();
        self.script_edges.clear();
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
        schema: GRAPH_BUILD_RESULT_SCHEMA,
        request: request.clone(),
        request_digest,
        status: GraphReadStatus::Partial,
        file_nodes: Vec::new(),
        package_nodes: Vec::new(),
        package_file_edges: Vec::new(),
        package_dependency_edges: Vec::new(),
        package_load_edges: Vec::new(),
        xml_nodes: Vec::new(),
        lua_nodes: Vec::new(),
        function_nodes: Vec::new(),
        call_edges: Vec::new(),
        frame_nodes: Vec::new(),
        creation_edges: Vec::new(),
        mixin_instance_nodes: Vec::new(),
        instantiation_edges: Vec::new(),
        construction_mixin_edges: Vec::new(),
        assignment_mixin_edges: Vec::new(),
        call_recognition: None,
        construction_recognition: None,
        mixin_recognition: None,
        mixin_assignment_recognition: None,
        script_recognition: None,
        state_recognition: None,
        signal_recognition: None,
        bridge_recognition: None,
        custom_recognition: None,
        cvar_recognition: None,
        hook_recognition: None,
        library_recognition: None,
        signal_nodes: Vec::new(),
        signal_edges: Vec::new(),
        toc_recognition: Vec::new(),
        toc_topology: toc::TocTopology::default(),
        xml_recognition: Vec::new(),
        xml_topology: xml::XmlTopology::default(),
        state_root_recognition: None,
        state_root_topology: toc::TocTopology::default(),
        state_core_recognition: Vec::new(),
        state_core_topology: state_core::StateCoreTopology::default(),
        state_nodes: StateNodes::empty(),
        state_edges: Vec::new(),
        handler_nodes: Vec::new(),
        script_edges: Vec::new(),
        snapshot: None,
        snapshot_input_digest: None,
        provenance: None,
        failure: None,
        result_digest: None,
        boundaries: vec![
            "captured_source_topology_calls_xml_handlers_and_saved_variable_slots",
            "saved_variable_paths_are_source_accesses_not_runtime_values_or_persistence",
            "state_alias_links_are_possible_and_reassigned_local_bindings_are_not_followed",
            "state_fractional_dynamic_keys_environment_changes_and_inline_xml_not_evaluated",
            "not_coherent_project_store_publication",
            "package_dependencies_and_order_are_static_selected_toc_evidence_not_runtime_load_success",
            "package_order_groups_remain_exact_provenance_not_synthetic_transitive_edges",
            "toc_declarations_are_static_structure_with_explicit_omissions_not_runtime_load_success",
            "toc_repeated_or_unselected_occurrences_do_not_form_a_synthetic_file_order_dag",
            "dynamic_library_inline_xml_calls_and_remaining_recognizers_not_evaluated",
            "create_frame_is_static_construction_evidence_not_runtime_frame_existence",
            "create_from_mixins_is_static_main_declaration_evidence_not_runtime_instantiation",
            "dynamic_and_library_mixin_arguments_remain_possible_or_unlinked",
            "mixin_assignment_is_exact_static_structure_not_inheritance_or_runtime_behavior",
            "xml_runtime_objects_parentage_and_mixin_execution_not_evaluated",
            "library_mixin_and_handler_targets_not_projected",
            "xml_method_and_inherited_handler_associations_are_possible_not_dispatch",
            "xml_inline_script_sites_are_exact_but_receiver_and_runtime_dispatch_are_not_evaluated",
            "xml_handler_override_append_prepend_and_intrinsic_order_not_evaluated",
            "no_negative_authority",
        ],
    };
    match compose(input, request, stop) {
        Ok(BuiltGraph {
            snapshot,
            provenance,
            digest,
            file_nodes,
            package_nodes,
            package_file_edges,
            package_dependency_edges,
            package_load_edges,
            xml_nodes,
            lua_nodes,
            function_nodes,
            call_edges,
            frame_nodes,
            creation_edges,
            mixin_instance_nodes,
            instantiation_edges,
            construction_mixin_edges,
            assignment_mixin_edges,
            call_recognition,
            construction_recognition,
            mixin_recognition,
            mixin_assignment_recognition,
            script_recognition,
            state_recognition,
            signal_recognition,
            bridge_recognition,
            custom_recognition,
            cvar_recognition,
            hook_recognition,
            library_recognition,
            state_nodes,
            state_edges,
            signal_nodes,
            signal_edges,
            toc_recognition,
            toc_topology,
            xml_recognition,
            xml_topology,
            state_root_recognition,
            state_root_topology,
            state_core_recognition,
            state_core_topology,
            handler_nodes,
            script_edges,
        }) => {
            result.file_nodes = file_nodes;
            result.package_nodes = package_nodes;
            result.package_file_edges = package_file_edges;
            result.package_dependency_edges = package_dependency_edges;
            result.package_load_edges = package_load_edges;
            result.xml_nodes = xml_nodes;
            result.lua_nodes = lua_nodes;
            result.function_nodes = function_nodes;
            result.call_edges = call_edges;
            result.frame_nodes = frame_nodes;
            result.creation_edges = creation_edges;
            result.mixin_instance_nodes = mixin_instance_nodes;
            result.instantiation_edges = instantiation_edges;
            result.construction_mixin_edges = construction_mixin_edges;
            result.assignment_mixin_edges = assignment_mixin_edges;
            result.call_recognition = Some(call_recognition);
            result.construction_recognition = Some(construction_recognition);
            result.mixin_recognition = Some(mixin_recognition);
            result.mixin_assignment_recognition = Some(mixin_assignment_recognition);
            result.script_recognition = Some(script_recognition);
            result.state_recognition = Some(state_recognition);
            result.state_nodes = state_nodes;
            result.state_edges = state_edges;
            result.signal_recognition = Some(signal_recognition);
            result.bridge_recognition = Some(bridge_recognition);
            result.custom_recognition = Some(custom_recognition);
            result.cvar_recognition = Some(cvar_recognition);
            result.hook_recognition = Some(hook_recognition);
            result.library_recognition = Some(library_recognition);
            result.signal_nodes = signal_nodes;
            result.signal_edges = signal_edges;
            result.toc_recognition = toc_recognition;
            result.toc_topology = toc_topology;
            result.xml_recognition = xml_recognition;
            result.xml_topology = xml_topology;
            result.state_root_recognition = Some(state_root_recognition);
            result.state_root_topology = state_root_topology;
            result.state_core_recognition = state_core_recognition;
            result.state_core_topology = state_core_topology;
            result.handler_nodes = handler_nodes;
            result.script_edges = script_edges;
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
    let backend = LocalProjectBackend::for_graph(input)?;
    compose_backend(&backend, request, stop)
}

/// Uses the same materialization and producer chain as the artifact command,
/// retaining the original native publisher for the project owner handoff.
pub(crate) fn live_publication(
    input: LocalProjectInput,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<(
    wow_project::replay::publication::ProjectPublicationBundle,
    String,
)> {
    checkpoint(stop)?;
    let backend = LocalProjectBackend::for_graph(input)?;
    live_publication_from_backend(&backend, request, stop)
}

/// Reuse the same complete producer chain over an already validated native owner.
pub(crate) fn live_publication_from_backend(
    backend: &LocalProjectBackend,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<(
    wow_project::replay::publication::ProjectPublicationBundle,
    String,
)> {
    checkpoint(stop)?;
    let built = compose_backend(backend, request, stop)?;
    let bundle = backend.capture_project_bundle(&built.snapshot, stop)?;
    Ok((bundle, built.snapshot.snapshot().universe().as_str().into()))
}

pub(crate) fn live_publication_in_namespace(
    input: LocalProjectInput,
    request: &GraphBuildRequest,
    namespace: &wow_store::project::ProjectStoreNamespace,
    stop: &AtomicBool,
) -> ServiceResult<(
    wow_project::replay::publication::ProjectPublicationBundle,
    String,
)> {
    checkpoint(stop)?;
    let backend = LocalProjectBackend::for_graph(input)?;
    live_publication_from_backend_in_namespace(&backend, request, namespace, stop)
}

/// Capture the selected namespace from the same retained native producer chain.
pub(crate) fn live_publication_from_backend_in_namespace(
    backend: &LocalProjectBackend,
    request: &GraphBuildRequest,
    namespace: &wow_store::project::ProjectStoreNamespace,
    stop: &AtomicBool,
) -> ServiceResult<(
    wow_project::replay::publication::ProjectPublicationBundle,
    String,
)> {
    checkpoint(stop)?;
    let built = compose_backend(backend, request, stop)?;
    let bundle = backend.capture_project_bundle_in_namespace(&built.snapshot, namespace, stop)?;
    Ok((bundle, namespace.id().as_str().into()))
}

fn compose_backend(
    backend: &LocalProjectBackend,
    request: &GraphBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<BuiltGraph> {
    checkpoint(stop)?;
    let project = request.acquire_project(backend, stop)?;
    let proposals = build_source_graph_proposals(&project, stop).map_err(|e| {
        ServiceError::new(
            match e.code() {
                wow_project::ProjectErrorCode::AnalysisCancelled
                | wow_project::ProjectErrorCode::SourceReadCancelled => ServiceErrorCode::Cancelled,
                wow_project::ProjectErrorCode::SourceBudgetExceeded => {
                    ServiceErrorCode::BudgetExceeded
                }
                _ => ServiceErrorCode::InternalContractViolation,
            },
            format!("native source graph projection rejected ({:?})", e.code()),
        )
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
    let (calls_snapshot, call_recognition) =
        calls::publish(replacement.candidate(), &provenance, stop)
            .map_err(|e| stage_error("calls", e))?;
    let (construction_snapshot, construction_recognition) =
        construction::publish(&calls_snapshot, &provenance, stop)
            .map_err(|e| stage_error("construction", e))?;
    let (mixin_snapshot, mixin_recognition, mixin_assignment_recognition) =
        lua_mixins::publish(&construction_snapshot, &provenance, stop)
            .map_err(|e| stage_error("mixins", e))?;
    let (scripts_snapshot, script_recognition) =
        scripts::publish(&mixin_snapshot, &provenance, stop)
            .map_err(|e| stage_error("scripts", e))?;
    let (snapshot, state_recognition) = state::publish(&scripts_snapshot, &provenance, stop)
        .map_err(|e| stage_error("state", e))?;
    // W11 signal and hook families publish after every earlier owner, so each
    // adapter crosswalks against the accepted source graph that precedes it.
    let (
        snapshot,
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        hook_recognition,
        library_recognition,
    ) = signals::publish_signals(&snapshot, &provenance, stop)
        .map_err(|e| stage_error("signals", e))?;
    let (snapshot, toc_recognition) =
        toc::publish(&snapshot, &provenance, stop).map_err(|e| stage_error("toc", e))?;
    let (snapshot, xml_recognition) =
        xml::publish(&snapshot, &provenance, stop).map_err(|e| stage_error("xml", e))?;
    let (snapshot, state_root_recognition) = toc::publish_state_root(&snapshot, &provenance, stop)
        .map_err(|e| stage_error("state-roots", e))?;
    let (snapshot, state_core_recognition) =
        state_core::publish(&snapshot, &provenance, &state_recognition, stop)
            .map_err(|e| stage_error("state-core", e))?;
    let toc_topology = toc::maps(&snapshot, &toc_recognition, stop)
        .map_err(|e| stage_error("toc-crosswalk", e))?;
    let xml_topology = xml::maps(&snapshot, &xml_recognition, stop)
        .map_err(|e| stage_error("xml-crosswalk", e))?;
    let state_root_topology = toc::maps(
        &snapshot,
        std::slice::from_ref(&state_root_recognition),
        stop,
    )?;
    let state_core_topology = state_core::maps(&snapshot, &state_core_recognition, stop)?;
    let (state_nodes, state_edges) = state::maps(&snapshot, &provenance, &state_recognition, stop)?;
    let signal_topology = signals::maps(
        &snapshot,
        &provenance,
        &signal_recognition,
        &bridge_recognition,
        &custom_recognition,
        &cvar_recognition,
        stop,
    )?;
    let signal_nodes = signal_topology.nodes;
    let signal_edges = signal_topology.edges;
    let (handler_nodes, script_edges) =
        scripts::maps(&snapshot, &provenance, &script_recognition, stop)?;
    let (function_nodes, call_edges) =
        calls::maps(&snapshot, &provenance, &call_recognition, stop)?;
    let (frame_nodes, creation_edges) =
        construction::maps(&snapshot, &provenance, &construction_recognition, stop)?;
    let (
        mixin_instance_nodes,
        instantiation_edges,
        construction_mixin_edges,
        assignment_mixin_edges,
    ) = lua_mixins::maps(
        &snapshot,
        &provenance,
        &mixin_recognition,
        &mixin_assignment_recognition,
        stop,
    )?;
    checkpoint(stop)?;
    let source_nodes = materialized::nodes(&snapshot, stop)?;
    let mut file_nodes = Vec::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        file_nodes.push(FileNode {
            path: file.path.clone(),
            node_id: materialized_node_id(&snapshot, &file.proposal_id, limits)?,
        });
    }
    let mut package_nodes = Vec::new();
    for package in provenance.packages() {
        checkpoint(stop)?;
        package_nodes.push(PackageNode {
            package: package.package.clone(),
            order_group: package.order_group,
            reachability: package.reachability,
            phase: package.phase,
            node_id: materialized_node_id(&snapshot, &package.proposal_id, limits)?,
        });
    }
    let mut package_file_edges = Vec::new();
    for receipt in provenance.package_files() {
        checkpoint(stop)?;
        package_file_edges.push(PackageFileEdge {
            package: receipt.package.clone(),
            path: receipt.path.clone(),
            edge_id: materialized_edge_id(&snapshot, &receipt.proposal_id, &source_nodes)?,
        });
    }
    let mut package_dependency_edges = Vec::new();
    for dependency in provenance.package_dependencies() {
        checkpoint(stop)?;
        let ProjectGraphPackageDependencyOutcome::Projected {
            proposal_id,
            confidence,
        } = &dependency.outcome
        else {
            continue;
        };
        package_dependency_edges.push(PackageDependencyEdge {
            ordinal: dependency.ordinal,
            package: dependency.package.clone(),
            dependency: dependency.dependency.clone(),
            kind: dependency.kind,
            confidence: *confidence,
            edge_id: materialized_edge_id(&snapshot, proposal_id, &source_nodes)?,
        });
    }
    let mut package_load_edges = Vec::new();
    for load in provenance.package_loads() {
        checkpoint(stop)?;
        let ProjectGraphPackageLoadOutcome::Projected {
            proposal_id,
            confidence,
        } = &load.outcome
        else {
            continue;
        };
        package_load_edges.push(PackageLoadEdge {
            unit_digest: load.unit_digest,
            package: load.package.clone(),
            target: load.target.clone(),
            confidence: *confidence,
            edge_id: materialized_edge_id(&snapshot, proposal_id, &source_nodes)?,
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
    .map_err(|e| stage_error("final-graph-decode", error(e.code)))?;
    if admitted != snapshot {
        return Err(ServiceError::new(
            ServiceErrorCode::InternalContractViolation,
            "final graph export differs from its admitted owner snapshot",
        ));
    }
    checkpoint(stop)?;
    Ok(BuiltGraph {
        snapshot,
        provenance,
        digest: super::hash(&bytes),
        file_nodes,
        package_nodes,
        package_file_edges,
        package_dependency_edges,
        package_load_edges,
        xml_nodes,
        lua_nodes,
        function_nodes,
        call_edges,
        frame_nodes,
        creation_edges,
        mixin_instance_nodes,
        instantiation_edges,
        construction_mixin_edges,
        assignment_mixin_edges,
        call_recognition,
        construction_recognition,
        mixin_recognition,
        mixin_assignment_recognition,
        script_recognition,
        state_recognition,
        state_nodes,
        state_edges,
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        signal_nodes,
        signal_edges,
        toc_recognition,
        toc_topology,
        xml_recognition,
        xml_topology,
        state_root_recognition,
        state_root_topology,
        state_core_recognition,
        state_core_topology,
        hook_recognition,
        library_recognition,
        handler_nodes,
        script_edges,
    })
}

/// Rebind the accepted semantic key, never search materialized nodes by name.
fn materialized_node_id(
    snapshot: &GraphPartitionSnapshot,
    proposal_id: &str,
    limits: wow_graph::GraphLimits,
) -> ServiceResult<wow_graph::GraphNodeId> {
    materialized_partition_node_id(
        snapshot,
        wow_project::graph::SOURCE_GRAPH_PARTITION,
        proposal_id,
        limits,
    )
}

pub(super) fn materialized_partition_node_id(
    snapshot: &GraphPartitionSnapshot,
    partition_id: &str,
    proposal_id: &str,
    limits: wow_graph::GraphLimits,
) -> ServiceResult<wow_graph::GraphNodeId> {
    let partition = snapshot
        .partition(partition_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_entities();
    let index = accepted
        .binary_search_by(|entry| entry.proposal_id().cmp(proposal_id))
        .map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "source entity receipt is absent from accepted entities",
            )
        })?;
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
        return Err(ServiceError::new(
            ServiceErrorCode::InternalContractViolation,
            "accepted source entity is absent from materialized graph",
        ));
    }
    Ok(node.node_id().clone())
}

/// Rebind an accepted source-partition proposal to its exact materialized edge.
fn materialized_edge_id(
    snapshot: &GraphPartitionSnapshot,
    proposal_id: &str,
    nodes: &std::collections::BTreeMap<wow_graph::GraphNodeId, wow_graph::GraphNodeId>,
) -> ServiceResult<wow_graph::GraphEdgeId> {
    materialized::edge_id(
        snapshot,
        wow_project::graph::SOURCE_GRAPH_PARTITION,
        proposal_id,
        nodes,
    )
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
fn stage_error(stage: &'static str, error: ServiceError) -> ServiceError {
    ServiceError::new(
        error.code(),
        format!(
            "source graph {stage} publication failed: {}",
            error.message()
        ),
    )
}
fn graph_error(e: wow_graph::GraphError) -> ServiceError {
    ServiceError::new(
        match e.code() {
            wow_graph::GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_graph::GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
            _ => ServiceErrorCode::InternalContractViolation,
        },
        format!("source graph owner rejected ({:?})", e.code()),
    )
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
