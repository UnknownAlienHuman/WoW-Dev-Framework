use serde::Serialize;
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::{io::Write, sync::atomic::AtomicBool};
use wow_core::{CanonicalResult, ContentDigest};
use wow_graph::GraphEvidenceCatalog;
use wow_store::project::{CurrentPublication, ReadSelector};

use super::{
    CONTEXT_MAP_RESULT_MAX_BYTES, CONTEXT_MAP_RESULT_SCHEMA, ContextMapHostInput,
    ContextMapPublication, ContextMapRequest, ContextMapRoot,
    PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID, checkpoint, context_error, failure, graph_error,
};
use crate::{
    ServiceErrorCode, ServiceResult,
    live_project::{LiveProjectRead, LiveProjectStore, ReadFailureClosure},
};

#[derive(Debug, Clone, Serialize)]
pub(super) struct FrontierExpansion {
    profile_id: &'static str,
    frontier: Vec<wow_graph::GraphNodeId>,
}

impl FrontierExpansion {
    pub(super) fn new(
        level: super::ContextInspectLevel,
        root: &ContextMapRoot,
        frontier: Vec<ContextMapRoot>,
    ) -> ServiceResult<Self> {
        if level != super::ContextInspectLevel::L1
            || !matches!(root, ContextMapRoot::GraphEntity(_))
            || frontier.is_empty()
            || frontier.len() > 32
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        let mut ids = frontier
            .into_iter()
            .map(|root| match root {
                ContextMapRoot::GraphEntity(id) => Ok(id),
                ContextMapRoot::Project(_) => Err(failure(ServiceErrorCode::InvalidRequest)),
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        ids.sort();
        if ids.windows(2).any(|pair| pair[0] == pair[1])
            || matches!(root, ContextMapRoot::GraphEntity(root) if ids.contains(root))
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            profile_id: super::PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            frontier: ids,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextMapStatus {
    Partial,
    Truncated,
    Failed,
    Cancelled,
}

/// Final inert envelope bytes; executable owner/read/lease handles never escape.
pub struct ContextMapResult {
    pub(super) bytes: Box<[u8]>,
    pub(super) status: ContextMapStatus,
    pub(super) code: Option<ServiceErrorCode>,
}
impl ContextMapResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub const fn status(&self) -> ContextMapStatus {
        self.status
    }
    pub const fn failure_code(&self) -> Option<ServiceErrorCode> {
        self.code
    }
}

pub fn execute_context_map(
    host: &ContextMapHostInput,
    request: &ContextMapRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let request_digest =
        wow_core::domain_separated_digest("wow-service/context-map-request/1", request)
            .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-map-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(request_digest)
    );
    let selector = match &request.publication {
        ContextMapPublication::Current => ReadSelector::Current,
        ContextMapPublication::StoreGeneration(id) => ReadSelector::Exact(id.clone()),
    };
    let (outcome, closure) =
        with_checked_context_read(host, &request.project_store_id, &selector, stop, |read| {
            build(read, host, request, &request_id, stop)
        });
    match outcome {
        Ok((bytes, status)) => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status,
            code: None,
        }),
        Err(error) => failed(request, &request_id, error.code(), closure),
    }
}

pub(super) fn with_checked_context_read<T>(
    host: &ContextMapHostInput,
    registration: &str,
    selector: &ReadSelector,
    stop: &AtomicBool,
    build: impl FnOnce(&LiveProjectRead) -> ServiceResult<T>,
) -> (ServiceResult<T>, &'static str) {
    let closure = Cell::new("not_acquired");
    let outcome = (|| {
        host.validate(stop)?;
        if host.project_store_id.as_ref() != registration {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        let store = LiveProjectStore::open(&host.project_store_root)?;
        closure.set("pending");
        let acquired = store.read_checked(selector, stop);
        let draft = match acquired {
            Ok(read) => {
                let draft = build(&read);
                // Close failures supersede a draft or an earlier semantic refusal.
                let closed = read.close();
                match closed {
                    Ok(()) => draft,
                    Err(_) => {
                        closure.set("failed");
                        Err(failure(ServiceErrorCode::InternalContractViolation))
                    }
                }
            }
            Err(failure) => {
                closure.set(match failure.closure {
                    ReadFailureClosure::Closed => "closed",
                    ReadFailureClosure::Failed => "failed",
                    ReadFailureClosure::NotEvaluated => "not_evaluated",
                });
                Err(failure.error)
            }
        };
        if store.close().is_err() {
            closure.set("failed");
            return Err(failure(ServiceErrorCode::InternalContractViolation));
        }
        if closure.get() == "pending" {
            closure.set("closed");
        }
        checkpoint(stop)?;
        draft
    })();
    (outcome, closure.get())
}

#[derive(Serialize)]
pub(super) struct Envelope<'a, R: Serialize, T: Serialize> {
    pub(super) schema: &'static str,
    pub(super) operation: &'static str,
    pub(super) request_id: &'a str,
    pub(super) request: &'a R,
    pub(super) status: ContextMapStatus,
    pub(super) resource_closure: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) failure_code: Option<ServiceErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) payload: Option<T>,
}
#[derive(Serialize)]
struct Payload<'a, 'map> {
    resolved_request_id: Box<str>,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_at_acquisition: Option<&'a CurrentPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_record_digest: Option<Box<str>>,
    configured_registration_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    native_project_store_id: Option<&'a str>,
    project_map: &'a wow_context::ProjectMap<'map>,
    project_map_byte_length: usize,
}

fn build(
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    request: &ContextMapRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus)> {
    checkpoint(stop)?;
    let project = read.project();
    if project.configuration().project_id() != &host.project_id {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let current = read.current_at_acquisition();
    let current_digest = current.map(current_record_digest).transpose()?;
    if let Some(guard) = &request.current {
        let actual =
            current.ok_or_else(|| failure(ServiceErrorCode::CurrentGenerationUnavailable))?;
        if guard
            .record
            .as_ref()
            .is_some_and(|id| id != &actual.record_id)
            || guard
                .digest
                .as_ref()
                .is_some_and(|digest| Some(digest) != current_digest.as_ref())
        {
            return Err(failure(ServiceErrorCode::StoreCurrentConflict));
        }
    }
    if let Some(guard) = &request.reference
        && (&guard.profile != host.reference_profile.profile_id()
            || guard.generation.to_string() != host.reference.generation_id()
            || guard.view.as_ref() != host.reference.self_digest())
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let limits = host.limits.owner_limits()?;
    let universe = wow_context::bind_context_universe_set(
        read.pair(),
        &host.reference,
        &host.reference_profile,
        limits,
        stop,
    )
    .map_err(context_error)?;
    let project_profile = request.map_profile_id == PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID;
    let root = match &request.root {
        Some(ContextMapRoot::GraphEntity(id)) => ContextMapRoot::GraphEntity(id.clone()),
        Some(ContextMapRoot::Project(id)) if id != project.configuration().project_id() => {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Some(ContextMapRoot::Project(id)) if project_profile => ContextMapRoot::Project(id.clone()),
        None if project_profile => {
            ContextMapRoot::Project(project.configuration().project_id().clone())
        }
        // The selected ordinary source owner has no project graph entity. The
        // original project container is retained by the map independently; no
        // first-file/name lookup or synthetic graph relation supplies a root.
        Some(ContextMapRoot::Project(_)) | None => {
            return Err(failure(ServiceErrorCode::ComponentUnavailable));
        }
    };
    let resolved_digest = wow_core::domain_separated_digest(
        "wow-service/context-map-resolved-request/1",
        &(
            request_id,
            read.publication_set_id(),
            read.store_generation_id(),
            host.reference_profile.profile_id(),
            host.reference.generation_id(),
            host.reference.self_digest(),
            // Preserve the frozen direct profile's exact identity material;
            // a project container has a distinct tagged material/algorithm.
            resolved_root(&root),
            &host.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-map-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(resolved_digest)
    )
    .into_boxed_str();
    with_native_origins(&universe, read, host, root, false, stop, |map, _graph| {
        let status = match map.state() {
            wow_context::ProjectMapState::Partial => ContextMapStatus::Partial,
            wow_context::ProjectMapState::Truncated => ContextMapStatus::Truncated,
        };
        let payload = Payload {
            resolved_request_id,
            publication_set_id: read.publication_set_id(),
            store_generation_id: read.store_generation_id().as_str(),
            current_at_acquisition: current,
            current_record_digest: current_digest,
            configured_registration_id: &host.project_store_id,
            native_project_store_id: read.namespace().map(|namespace| namespace.id().as_str()),
            project_map_byte_length: map.canonical_byte_length(),
            project_map: map,
        };
        let envelope = Envelope {
            schema: CONTEXT_MAP_RESULT_SCHEMA,
            operation: "context_map",
            request_id,
            request,
            status,
            resource_closure: "closed",
            failure_code: None,
            payload: Some(payload),
        };
        let limit = usize::try_from(host.limits.max_metadata_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        Ok((encode(&envelope, limit, stop)?, status))
    })
}

pub(super) fn with_native_origins<T>(
    universe: &wow_context::ContextUniverseSet<'_>,
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    root: ContextMapRoot,
    detail: bool,
    stop: &AtomicBool,
    build: impl FnOnce(
        &wow_context::ProjectMap<'_>,
        &wow_graph::GraphPartitionRead<'_>,
    ) -> ServiceResult<T>,
) -> ServiceResult<T> {
    with_native_origins_with_catalog(
        universe,
        read,
        host,
        root,
        detail,
        stop,
        |map, graph, _catalog| build(map, graph),
    )
}

/// Selected orchestration borrows the same admitted catalog and native read.
pub(super) fn with_native_origins_with_catalog<T>(
    universe: &wow_context::ContextUniverseSet<'_>,
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    root: ContextMapRoot,
    detail: bool,
    stop: &AtomicBool,
    build: impl FnOnce(
        &wow_context::ProjectMap<'_>,
        &wow_graph::GraphPartitionRead<'_>,
        &GraphEvidenceCatalog,
    ) -> ServiceResult<T>,
) -> ServiceResult<T> {
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    // Existing native catalog admission owns DAG/support validation and limits.
    // One aggregate borrowed reservation includes every encoded input view
    // before the necessary catalog copy; separate owner ceilings cannot prove
    // the combined input fits the selected service budget.
    preflight(
        &(
            &host.reference,
            read.graph().snapshot(),
            provenance.context(),
            provenance.evidence(),
            provenance.source_handles(),
        ),
        usize::try_from(host.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?,
        stop,
    )?;
    let catalog = GraphEvidenceCatalog::new(
        provenance.context().clone(),
        provenance.evidence().clone(),
        provenance.source_handles().clone(),
        stop,
    )
    .map_err(graph_error)?;
    let graph = read.pair().graph_read(stop).map_err(graph_error)?;
    let mut selection = match root {
        ContextMapRoot::GraphEntity(id) => {
            let query = if detail {
                wow_context::primary_detail_query(universe, id.clone(), stop)
            } else {
                wow_context::primary_navigation_query(universe, id.clone(), stop)
            }
            .map_err(context_error)?;
            let neighbors = graph.neighbors(&query, stop).map_err(graph_error)?;
            (if detail {
                wow_context::prepare_detail_map(universe, id, neighbors, stop)
            } else {
                wow_context::prepare_project_map(universe, id, neighbors, stop)
            })
            .map_err(context_error)?
        }
        ContextMapRoot::Project(id) => {
            let mut request =
                wow_context::primary_project_navigation_queries(universe, &id, &graph, stop)
                    .map_err(context_error)?;
            for index in 0..request.navigation_queries().len() {
                checkpoint(stop)?;
                let view = graph
                    .neighbors(&request.navigation_queries()[index], stop)
                    .map_err(graph_error)?;
                request
                    .admit_neighborhood(view, stop)
                    .map_err(context_error)?;
            }
            wow_context::prepare_primary_project_map(request, stop).map_err(context_error)?
        }
    };
    let mut receipts = Vec::with_capacity(selection.explanation_queries().len());
    for index in 0..selection.explanation_queries().len() {
        checkpoint(stop)?;
        let receipt = graph
            .explain_with_evidence(
                &selection.explanation_queries()[index],
                &catalog,
                host.limits.evidence,
                stop,
            )
            .map_err(graph_error)?;
        selection
            .admit_explanation(&receipt, stop)
            .map_err(context_error)?;
        receipts.push(receipt);
    }
    let map = wow_context::build_project_map(selection, &receipts, stop).map_err(context_error)?;
    build(&map, &graph, &catalog)
}

/// Keep additional native receipts stable until the consuming callback returns.
pub(super) fn with_native_frontier<T>(
    universe: &wow_context::ContextUniverseSet<'_>,
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    root: ContextMapRoot,
    expansion: &FrontierExpansion,
    stop: &AtomicBool,
    build: impl FnOnce(wow_context::ContextExpansionStep<'_>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    with_native_origins_with_catalog(
        universe,
        read,
        host,
        root,
        true,
        stop,
        |map, graph, catalog| {
            let mut plan = wow_context::plan_context_expansion(
                universe,
                map,
                graph,
                &expansion.frontier,
                stop,
            )
            .map_err(context_error)?;
            for index in 0..plan.navigation_queries().len() {
                checkpoint(stop)?;
                let view = graph
                    .neighbors(&plan.navigation_queries()[index], stop)
                    .map_err(graph_error)?;
                plan.admit_neighborhood(view, stop).map_err(context_error)?;
            }
            let mut prepared =
                wow_context::prepare_context_expansion(plan, stop).map_err(context_error)?;
            let mut receipts = Vec::with_capacity(prepared.explanation_queries().len());
            for index in 0..prepared.explanation_queries().len() {
                checkpoint(stop)?;
                let receipt = graph
                    .explain_with_evidence(
                        &prepared.explanation_queries()[index],
                        catalog,
                        host.limits.evidence,
                        stop,
                    )
                    .map_err(graph_error)?;
                prepared
                    .admit_explanation(&receipt, stop)
                    .map_err(context_error)?;
                receipts.push(receipt);
            }
            let step = wow_context::expand_context_frontier(prepared, &receipts, stop)
                .map_err(context_error)?;
            build(step)
        },
    )
}

/// Digest of the exact native CurrentPublication canonical observation, not
/// its record-ID derivation material. A guard compares this same frozen encoding.
pub(super) fn current_record_digest(current: &CurrentPublication) -> ServiceResult<Box<str>> {
    let bytes = wow_core::canonical_json_bytes(current)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    Ok(
        ContentDigest::<CanonicalResult>::from_bytes(Sha256::digest(bytes).into())
            .to_string()
            .into(),
    )
}

#[derive(Serialize)]
#[serde(untagged)]
enum ResolvedRoot<'a> {
    Graph(&'a wow_graph::GraphNodeId),
    Project(&'a ContextMapRoot),
}
fn resolved_root(root: &ContextMapRoot) -> ResolvedRoot<'_> {
    match root {
        ContextMapRoot::GraphEntity(id) => ResolvedRoot::Graph(id),
        ContextMapRoot::Project(_) => ResolvedRoot::Project(root),
    }
}

fn failed(
    request: &ContextMapRequest,
    request_id: &str,
    code: ServiceErrorCode,
    closure: &'static str,
) -> ServiceResult<ContextMapResult> {
    failed_operation(
        request,
        request_id,
        code,
        closure,
        CONTEXT_MAP_RESULT_SCHEMA,
        "context_map",
    )
}
pub(super) fn failed_operation<R: Serialize>(
    request: &R,
    request_id: &str,
    code: ServiceErrorCode,
    closure: &'static str,
    schema: &'static str,
    operation: &'static str,
) -> ServiceResult<ContextMapResult> {
    let status = if code == ServiceErrorCode::Cancelled {
        ContextMapStatus::Cancelled
    } else {
        ContextMapStatus::Failed
    };
    let envelope = Envelope::<_, ()> {
        schema,
        operation,
        request_id,
        request,
        status,
        resource_closure: closure,
        failure_code: Some(code),
        payload: None,
    };
    // Bounded finalization of a Cancelled result is required after semantic work
    // stops; it does not acquire owners or restart the cancelled operation.
    let bytes = encode(
        &envelope,
        CONTEXT_MAP_RESULT_MAX_BYTES,
        &AtomicBool::new(false),
    )?;
    Ok(ContextMapResult {
        bytes: bytes.into_boxed_slice(),
        status,
        code: Some(code),
    })
}

fn preflight(value: &impl Serialize, limit: usize, stop: &AtomicBool) -> ServiceResult<usize> {
    struct Counter<'a> {
        bytes: usize,
        limit: usize,
        exhausted: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            checkpoint(self.stop).map_err(|_| std::io::Error::other("context output stopped"))?;
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| {
                    self.exhausted = true;
                    std::io::Error::other("context output limit")
                })?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    checkpoint(stop)?;
    let mut counter = Counter {
        bytes: 0,
        limit,
        exhausted: false,
        stop,
    };
    let encoded = serde_json::to_writer(&mut counter, value);
    checkpoint(stop)?;
    if counter.exhausted {
        return Err(failure(ServiceErrorCode::BudgetExceeded));
    }
    encoded.map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    Ok(counter.bytes)
}
pub(super) fn encode(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    let size = preflight(value, limit, stop)?;
    let bytes = wow_core::canonical_json_bytes(value)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    checkpoint(stop)?;
    if size != bytes.len() {
        return Err(failure(ServiceErrorCode::CanonicalizationFailed));
    }
    Ok(bytes)
}
