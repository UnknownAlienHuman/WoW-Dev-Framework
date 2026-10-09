//! Public one-shot operations keep owner handles and selector resolution in the
//! service. Frontends transport only explicit requests and bounded result DTOs.
use super::{LiveProjectRead, LiveProjectStore, fail};
use crate::{LocalProjectInput, ServiceErrorCode, ServiceResult, graph::GraphBuildRequest};
use serde::Serialize;
use std::{path::Path, sync::atomic::AtomicBool};
use wow_store::{
    OperationId,
    project::{
        CurrentPublication, CurrentRecordId, PublicationOperation, ReadSelector, StoreGenerationId,
    },
};

pub struct LiveProjectPublishRequest {
    operation_id: OperationId,
    expected: Option<CurrentRecordId>,
    initialize: bool,
}
impl LiveProjectPublishRequest {
    pub fn new(
        operation_id: &str,
        expected_current: &str,
        initialize: bool,
        allow_partial: bool,
    ) -> ServiceResult<Self> {
        if !allow_partial {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        let expected = if expected_current == "absent" {
            None
        } else {
            Some(
                CurrentRecordId::parse(expected_current)
                    .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            )
        };
        if initialize && expected.is_some() {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            operation_id: OperationId::new(operation_id)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            expected,
            initialize,
        })
    }
}

/// Explicit Library intent for a supplied final physical project input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveProjectLibraryMode {
    Keep,
    Replace,
    Clear,
}

pub struct LiveProjectUpdateRequest {
    operation_id: OperationId,
    expected: CurrentRecordId,
    libraries: LiveProjectLibraryMode,
}
impl LiveProjectUpdateRequest {
    pub fn new(
        operation_id: &str,
        expected_current: &str,
        libraries: LiveProjectLibraryMode,
        allow_partial: bool,
    ) -> ServiceResult<Self> {
        if !allow_partial {
            return Err(fail(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            operation_id: OperationId::new(operation_id)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            expected: CurrentRecordId::parse(expected_current)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
            libraries,
        })
    }
}

#[derive(Debug, Serialize)]
struct PairSummary {
    publication_set_id: String,
    store_generation_id: String,
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    graph_snapshot_id: String,
    project_generation_id: String,
    profile_id: String,
    reference_generation_id: String,
    main_file_count: usize,
    library_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
}
impl PairSummary {
    fn from_read(read: &LiveProjectRead) -> Self {
        let project = read.project();
        let graph = read.graph().snapshot();
        Self {
            publication_set_id: read.publication_set_id().into(),
            store_generation_id: read.store_generation_id().as_str().into(),
            project_snapshot_id: project.snapshot_id().into(),
            analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
            graph_snapshot_id: graph.snapshot_id().as_str().into(),
            project_generation_id: project.project_generation().to_string(),
            profile_id: project
                .configuration()
                .selected_profile()
                .profile_id()
                .to_string(),
            reference_generation_id: project.configuration().reference_generation().to_string(),
            main_file_count: project.snapshot().file_manifest().len(),
            library_count: project
                .snapshot()
                .analyzer_binding()
                .library_snapshot_ids()
                .count(),
            graph_node_count: graph.nodes().len(),
            graph_edge_count: graph.edges().len(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LiveProjectResult {
    schema: &'static str,
    scope: &'static str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    operation: Option<PublicationOperation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<CurrentPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pair: Option<PairSummary>,
    boundaries: [&'static str; 3],
}
impl LiveProjectResult {
    fn new(status: &'static str) -> Self {
        Self {
            schema: "wow-service/live-project-result/1",
            scope: "native-live-project-pair-v1",
            status,
            operation: None,
            current: None,
            pair: None,
            boundaries: [
                "physical_lua_inputs_only",
                "source_graph_coverage_partial_no_negative_authority",
                "not_full_e2_or_source_runtime_acceptance",
            ],
        }
    }
    pub fn exit_code(&self) -> u8 {
        match self.status {
            "activated" | "acquired" | "no_change" => 2,
            "observed" | "released" => 0,
            _ => 3,
        }
    }
    pub fn canonical_bytes(&self) -> ServiceResult<Vec<u8>> {
        let bytes = wow_core::canonical_json_bytes(self)
            .map_err(|_| fail(ServiceErrorCode::CanonicalizationFailed))?;
        if bytes.len() > 128 * 1024 {
            return Err(fail(ServiceErrorCode::BudgetExceeded));
        }
        Ok(bytes)
    }
}

/// Builds the actual graph once from the original materialized publisher, then
/// validates its exact native pair before current CAS. No source mutation.
pub fn publish_local_project(
    config: &Path,
    graph_request: &GraphBuildRequest,
    root: &Path,
    request: &LiveProjectPublishRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    let input = LocalProjectInput::from_config_path(config, stop)?;
    publish_input(input, graph_request, root, request, stop)
}

pub(super) fn publish_input(
    input: LocalProjectInput,
    graph_request: &GraphBuildRequest,
    root: &Path,
    request: &LiveProjectPublishRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    let (bundle, owner) = crate::graph::live_publication(input, graph_request, stop)?;
    super::publication_checkpoint(stop)?;
    let mut store = if request.initialize {
        LiveProjectStore::create(root, &owner)?
    } else {
        LiveProjectStore::open(root)?
    };
    if store.store.epoch().owner() != owner {
        return Err(fail(ServiceErrorCode::IdentityMismatch));
    }
    let operation = store.publish_bundle(
        bundle,
        request.operation_id.as_str(),
        request.expected.clone(),
        stop,
    )?;
    let mut result = LiveProjectResult::new("activated");
    // Report the committed operation, rather than a later current observation.
    result.operation = Some(operation);
    Ok(result)
}

/// Apply explicit final inputs against an exact retained base, rebuild every
/// graph producer and activate only through the existing validated store CAS.
pub fn update_local_project(
    config: &Path,
    graph_request: &GraphBuildRequest,
    root: &Path,
    request: &LiveProjectUpdateRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    super::publication_checkpoint(stop)?;
    let input = LocalProjectInput::from_config_path(config, stop)?;
    update_input(input, graph_request, root, request, stop)
}

pub(super) fn update_input(
    input: LocalProjectInput,
    graph_request: &GraphBuildRequest,
    root: &Path,
    request: &LiveProjectUpdateRequest,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    super::publication_checkpoint(stop)?;
    let mut store = LiveProjectStore::open(root)?;
    store.update(input, graph_request, request, stop)
}

impl LiveProjectStore {
    /// Advance this owned store while previously acquired readers retain their
    /// original lease. Separate owners still obey the OS writer lock.
    pub fn update(
        &mut self,
        input: LocalProjectInput,
        graph_request: &GraphBuildRequest,
        request: &LiveProjectUpdateRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectResult> {
        super::publication_checkpoint(stop)?;
        let target = input.project_bundle();
        if target.configuration().load_plan().is_some()
            || target.configuration().package_load_plan().is_some()
        {
            return Err(fail(ServiceErrorCode::OperationNotImplementedForMilestone));
        }
        let store = self;
        let retained_operation = store.reconcile(request.operation_id.as_str())?;
        if retained_operation
            .as_ref()
            .is_some_and(|operation| operation.release.is_some())
        {
            return Err(fail(ServiceErrorCode::OperationReleased));
        }
        // A fresh operation cannot acquire an old base and silently rebase it.
        if retained_operation.is_none()
            && store.current()?.as_ref().map(|current| &current.record_id)
                != Some(&request.expected)
        {
            return Err(fail(ServiceErrorCode::StoreCurrentConflict));
        }
        let read = store.read(&ReadSelector::Publication(request.expected.clone()), stop)?;
        let base_summary = PairSummary::from_read(&read);
        let (mut publisher, _base_lease) = read.into_update_publisher()?;
        let libraries = match request.libraries {
            LiveProjectLibraryMode::Keep => wow_project::ProjectLibraryOperation::Keep,
            LiveProjectLibraryMode::Replace => {
                wow_project::ProjectLibraryOperation::Replace(target.libraries().to_vec())
            }
            LiveProjectLibraryMode::Clear => wow_project::ProjectLibraryOperation::Clear,
        };
        let update = publisher
            .derive_update_request(target, libraries)
            .map_err(super::project_error)?;
        let outcome = publisher
            .apply_update_cancellable(update, stop)
            .map_err(super::project_error)?;
        let backend = crate::LocalProjectBackend::for_graph_with_publisher(input, publisher, stop)?;
        super::publication_checkpoint(stop)?;
        if !outcome.changed() {
            graph_request.acquire_project(&backend, stop)?;
            if retained_operation.is_some() {
                return Err(fail(ServiceErrorCode::OperationConflict));
            }
            let current = store.current()?;
            if current.as_ref().map(|value| &value.record_id) != Some(&request.expected) {
                return Err(fail(ServiceErrorCode::StoreCurrentConflict));
            }
            let mut result = LiveProjectResult::new("no_change");
            result.scope = "native-live-project-update-v1";
            result.current = current;
            result.pair = Some(base_summary);
            return Ok(result);
        }
        let (bundle, owner) =
            crate::graph::live_publication_from_backend(&backend, graph_request, stop)?;
        if store.store.epoch().owner() != owner {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        super::publication_checkpoint(stop)?;
        let operation = store.publish_bundle(
            bundle,
            request.operation_id.as_str(),
            Some(request.expected.clone()),
            stop,
        )?;
        let mut result = LiveProjectResult::new("activated");
        result.scope = "native-live-project-update-v1";
        result.operation = Some(operation);
        Ok(result)
    }
}

pub fn read_live_project(
    root: &Path,
    generation: &str,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    let selector = if generation == "current" {
        ReadSelector::Current
    } else {
        ReadSelector::Exact(
            StoreGenerationId::parse(generation)
                .map_err(|_| fail(ServiceErrorCode::InvalidRequest))?,
        )
    };
    let store = LiveProjectStore::open(root)?;
    let read = store.read(&selector, stop)?;
    let mut result = LiveProjectResult::new("acquired");
    result.pair = Some(PairSummary::from_read(&read));
    result.current = read.current_at_acquisition().cloned();
    // All result projection completes under the original transaction/lease.
    super::publication_checkpoint(stop)?;
    Ok(result)
}

pub fn reconcile_live_project(
    root: &Path,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectResult> {
    super::publication_checkpoint(stop)?;
    let store = LiveProjectStore::open(root)?;
    let operation = store.reconcile(operation_id)?;
    let mut result = LiveProjectResult::new(
        if operation
            .as_ref()
            .is_some_and(|operation| operation.release.is_some())
        {
            "released"
        } else if operation.is_some() {
            "observed"
        } else {
            "operation_not_retained"
        },
    );
    result.operation = operation;
    result.current = store.current()?;
    Ok(result)
}
