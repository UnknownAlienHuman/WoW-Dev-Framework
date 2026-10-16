//! Explicit candidate-qualified platform Map/Inspect; Primary transport is unchanged.
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::atomic::AtomicBool};
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity};
use wow_graph::GraphQueryState;
use wow_project::skeleton_input::{
    OpenSkeletonInputRequest, SkeletonInputBinding, open_skeleton_input_view,
};
pub use wow_project::skeleton_input::{
    SkeletonInputCounts as ContextPlatformCounts,
    SkeletonInputNotEvaluated as ContextPlatformNotEvaluated,
};
use wow_reference::ReferenceView;
use wow_store::project::{ReadSelector, StoreGenerationId};

use super::{
    ContextMapResult, ContextMapStatus, checkpoint, context_error, failure,
    host::Limits,
    operation::{Envelope, encode, failed_operation, preflight},
    request,
};
use crate::{
    ServiceErrorCode, ServiceResult,
    live_project::{LiveProjectRead, PlatformProjectSelection, SelectionDocument},
};

pub const CONTEXT_PLATFORM_HOST_SCHEMA: &str = "wow-service/context-platform-native-host/1";
pub const CONTEXT_PLATFORM_REQUEST_SCHEMA: &str = "wow-service/context-platform-request/1";
pub const CONTEXT_PLATFORM_RESULT_SCHEMA: &str = "wow-service/context-platform-result/1";
pub const CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES: usize = super::CONTEXT_MAP_HOST_INPUT_MAX_BYTES;
pub const CONTEXT_PLATFORM_REQUEST_MAX_BYTES: usize = crate::graph::GRAPH_REQUEST_MAX_BYTES;
pub const PLATFORM_NATIVE_ONE_HOP_PROFILE_ID: &str = wow_context::PLATFORM_NATIVE_ONE_HOP_PROFILE;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HostDocument {
    schema: Box<str>,
    project_store_id: Box<str>,
    project_store_root: PathBuf,
    // The existing inert document and common admission retain one catalog owner.
    selection: SelectionDocument,
    reference_profile: ProfileIdentity,
    reference: ReferenceView,
    limits: Limits,
}

/// Inert host registration. Host paths never enter semantic identity or output.
pub struct ContextPlatformHostInput {
    schema: Box<str>,
    pub(super) project_store_id: Box<str>,
    project_store_root: PathBuf,
    pub(super) reference_profile: ProfileIdentity,
    pub(super) reference: ReferenceView,
    pub(super) limits: Limits,
    pub(super) selection: PlatformProjectSelection,
}
impl ContextPlatformHostInput {
    /// Semantic configuration for a durable consumer; filesystem routing is
    /// excluded and remains an explicit host transport location.
    pub(crate) fn semantic_binding_digest(&self) -> ServiceResult<Box<str>> {
        crate::identity::canonical_digest(
            "wow-service/platform-context-host-binding/1:",
            &(
                &self.schema,
                &self.project_store_id,
                self.selection.intent(),
                &self.reference_profile,
                &self.reference,
                &self.limits,
            ),
        )
    }
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: HostDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES,
            2_000_000,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        Self::from_document(document, stop)
    }
    pub(super) fn from_document(document: HostDocument, stop: &AtomicBool) -> ServiceResult<Self> {
        let selection = PlatformProjectSelection::from_document(document.selection, stop)?;
        let host = Self {
            schema: document.schema,
            project_store_id: document.project_store_id,
            project_store_root: document.project_store_root,
            reference_profile: document.reference_profile,
            reference: document.reference,
            limits: document.limits,
            selection,
        };
        host.validate(stop)?;
        Ok(host)
    }
    pub(super) fn validate(&self, stop: &AtomicBool) -> ServiceResult<()> {
        checkpoint(stop)?;
        let host = self;
        if host.schema.as_ref() != CONTEXT_PLATFORM_HOST_SCHEMA
            || !host.project_store_root.is_absolute()
            || host.project_store_root.as_os_str().len() > 32768
        {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        request::registration_id(&host.project_store_id)?;
        self.selection.require_candidate()?;
        self.selection.owner().validate()?;
        host.reference_profile
            .validate()
            .map_err(|_| failure(ServiceErrorCode::InvalidConfiguration))?;
        host.limits.owner_limits()?;
        checkpoint(stop)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextPlatformOperation {
    Map,
    Inspect,
}
impl ContextPlatformOperation {
    fn name(self) -> &'static str {
        match self {
            Self::Map => "context_platform_map",
            Self::Inspect => "context_platform_inspect",
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RequestDocument {
    schema: Box<str>,
    operation: ContextPlatformOperation,
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
    candidate_id: Box<str>,
    candidate_digest: Box<str>,
    profile_id: Box<str>,
    input: OpenSkeletonInputRequest,
}
/// Serialize-only inert request. Exact native authority is acquired, never decoded.
#[derive(Serialize)]
pub struct ContextPlatformRequest {
    #[serde(flatten)]
    pub(super) document: RequestDocument,
}
impl ContextPlatformRequest {
    /// Strict nested transport for an already byte-bounded Service request.
    /// Deserialize the document directly so duplicate fields cannot disappear
    /// through an intermediate JSON Value. No native owner is acquired here.
    pub(crate) fn deserialize_nested<'de, D: serde::Deserializer<'de>>(
        decoder: D,
    ) -> Result<Self, D::Error> {
        let document = RequestDocument::deserialize(decoder)?;
        Self::from_document(document, &AtomicBool::new(false)).map_err(serde::de::Error::custom)
    }
    /// Compare inert request/host metadata with an immutable Search selection.
    /// E3 still acquires and validates the genuine candidate/Skeleton owners.
    pub(crate) fn guard_search_selection(
        &self,
        host: &ContextPlatformHostInput,
        source: &serde_json::Value,
        root: &str,
        expected_view: Option<&str>,
    ) -> ServiceResult<()> {
        let field = |name: &str| {
            source
                .get(name)
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))
        };
        let document = &self.document;
        if document.project_store_id != host.project_store_id
            || document.store_generation_id.as_str() != field("store_generation_id")?
            || document.publication_set_id.as_ref() != field("publication_set_id")?
            || document.input.neighborhood().query().node_id().as_str() != root
            || document.input.neighborhood().snapshot_id().as_str() != field("graph_snapshot_id")?
            || host.selection.owner().namespace().owner_project_id() != field("project_id")?
            || host.reference.generation_id() != field("reference_generation_id")?
            || source.get("project_profile")
                != Some(
                    &serde_json::to_value(&host.reference_profile)
                        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?,
                )
            || expected_view.is_some_and(|expected| host.reference.self_digest() != expected)
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        // This is mechanical validation of a strict typed request, not native
        // owner reconstruction. The actual binding is derived again inside E3.
        let binding = serde_json::to_value(document.input.binding())
            .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
        for (input, selected) in [
            ("project_snapshot_id", "project_snapshot_id"),
            ("project_generation", "project_generation_id"),
            ("analyzer_snapshot_id", "analyzer_snapshot_id"),
            ("graph_snapshot_id", "graph_snapshot_id"),
            ("graph_generation", "graph_generation_id"),
        ] {
            if binding.get(input).and_then(serde_json::Value::as_str) != Some(field(selected)?) {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
        }
        Ok(())
    }
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: RequestDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_PLATFORM_REQUEST_MAX_BYTES,
            65_536,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        Self::from_document(document, stop)
    }
    pub(super) fn from_document(
        document: RequestDocument,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        if document.schema.as_ref() != CONTEXT_PLATFORM_REQUEST_SCHEMA
            || document.profile_id.as_ref() != PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        request::registration_id(&document.project_store_id)?;
        request::identifier(&document.publication_set_id)?;
        request::identifier(&document.candidate_id)?;
        crate::identity::validate_digest(&document.candidate_digest, "candidate digest")?;
        document.input.validate().map_err(project_error)?;
        checkpoint(stop)?;
        Ok(Self { document })
    }
    pub fn operation(&self) -> ContextPlatformOperation {
        self.document.operation
    }
}

/// Final inert selected envelope; no live owner or lease escapes.
pub struct ContextPlatformResult {
    result: ContextMapResult,
    summary: Option<ContextPlatformSummary>,
    resource_closure: &'static str,
}
impl ContextPlatformResult {
    pub fn canonical_bytes(&self) -> &[u8] {
        self.result.canonical_bytes()
    }
    pub const fn status(&self) -> ContextMapStatus {
        self.result.status()
    }
    pub const fn failure_code(&self) -> Option<ServiceErrorCode> {
        self.result.failure_code()
    }
    pub fn summary(&self) -> Option<&ContextPlatformSummary> {
        self.summary.as_ref()
    }
    pub const fn resource_closure(&self) -> &'static str {
        self.resource_closure
    }
}

/// Bounded observation for application text framing; no owner authority or paths.
#[derive(Serialize)]
pub struct ContextPlatformSummary {
    pub resolved_request_id: Box<str>,
    pub publication_set_id: Box<str>,
    pub store_generation_id: Box<str>,
    pub candidate_id: Box<str>,
    pub map_id: Box<str>,
    pub artifact_id: Box<str>,
    pub root_id: Box<str>,
    pub counts: ContextPlatformCounts,
    pub not_evaluated: Box<[ContextPlatformNotEvaluated]>,
    pub root_conflicts: usize,
    pub root_unresolved_evidence: usize,
    pub root_support_truncations: usize,
    pub root_evidence_truncations: usize,
    pub absence_authoritative: bool,
}
#[derive(Serialize)]
struct BorrowedSummary<'a> {
    resolved_request_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    candidate_id: &'a str,
    map_id: &'a str,
    artifact_id: &'a str,
    root_id: &'a str,
    counts: ContextPlatformCounts,
    not_evaluated: &'a [ContextPlatformNotEvaluated],
    root_conflicts: usize,
    root_unresolved_evidence: usize,
    root_support_truncations: usize,
    root_evidence_truncations: usize,
    absence_authoritative: bool,
}
impl BorrowedSummary<'_> {
    fn owned(&self) -> ContextPlatformSummary {
        ContextPlatformSummary {
            resolved_request_id: self.resolved_request_id.into(),
            publication_set_id: self.publication_set_id.into(),
            store_generation_id: self.store_generation_id.into(),
            candidate_id: self.candidate_id.into(),
            map_id: self.map_id.into(),
            artifact_id: self.artifact_id.into(),
            root_id: self.root_id.into(),
            counts: self.counts,
            not_evaluated: self.not_evaluated.into(),
            root_conflicts: self.root_conflicts,
            root_unresolved_evidence: self.root_unresolved_evidence,
            root_support_truncations: self.root_support_truncations,
            root_evidence_truncations: self.root_evidence_truncations,
            absence_authoritative: self.absence_authoritative,
        }
    }
}

pub fn execute_context_platform(
    host: &ContextPlatformHostInput,
    request: &ContextPlatformRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextPlatformResult> {
    let digest = wow_core::domain_separated_digest(CONTEXT_PLATFORM_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-platform-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let (outcome, closure) = with_checked_platform_read(host, request, stop, |read| {
        build(read, host, request, &request_id, stop)
    });
    let (result, summary) = match outcome {
        Ok((bytes, status, summary)) => (
            ContextMapResult {
                bytes: bytes.into_boxed_slice(),
                status,
                code: None,
            },
            Some(summary),
        ),
        Err(error) => (
            failed_operation(
                request,
                &request_id,
                error.code(),
                closure,
                CONTEXT_PLATFORM_RESULT_SCHEMA,
                request.operation().name(),
            )?,
            None,
        ),
    };
    Ok(ContextPlatformResult {
        result,
        summary,
        resource_closure: closure,
    })
}

pub(super) fn with_checked_platform_read<T>(
    host: &ContextPlatformHostInput,
    request: &ContextPlatformRequest,
    stop: &AtomicBool,
    build: impl FnOnce(&LiveProjectRead) -> ServiceResult<T>,
) -> (ServiceResult<T>, &'static str) {
    let admitted: ServiceResult<()> = (|| {
        host.validate(stop)?;
        let document = host;
        if document.project_store_id != request.document.project_store_id {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        narrow_evidence_limits(host, request)?;
        // Count aggregate semantic inputs before acquisition/allocation. Paths and
        // textual selection formatting are excluded; exact validated intent is used.
        let input_limit = usize::try_from(document.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        preflight(
            &(
                request,
                host.selection.intent(),
                &document.reference_profile,
                &document.reference,
                &document.limits,
            ),
            input_limit,
            stop,
        )?;
        Ok(())
    })();
    if let Err(error) = admitted {
        return (Err(error), "not_acquired");
    }
    with_checked_platform_generation(host, &request.document.store_generation_id, stop, build)
}

/// Shared exact-generation custody after the caller's request/config admission.
/// This seam acquires no root, query, skeleton, or context artifact.
pub(super) fn with_checked_platform_generation<T>(
    host: &ContextPlatformHostInput,
    generation: &StoreGenerationId,
    stop: &AtomicBool,
    build: impl FnOnce(&LiveProjectRead) -> ServiceResult<T>,
) -> (ServiceResult<T>, &'static str) {
    crate::live_project::selected_read_scope::with_checked_selected_read(
        &host.project_store_root,
        host.selection.owner(),
        &ReadSelector::Exact(generation.clone()),
        stop,
        build,
    )
}
fn narrow_evidence_limits(
    host: &ContextPlatformHostInput,
    request: &ContextPlatformRequest,
) -> ServiceResult<()> {
    // The context owner already fences neighbors/explain/direct edges. Resolve
    // limits are independent and must narrow every actual selected field too.
    let actual = request.document.input.limits().root_evidence;
    let selected = host.limits.evidence;
    if actual.max_evidence_records > selected.max_evidence_records
        || actual.max_source_handles > selected.max_source_handles
        || actual.max_derivation_depth > selected.max_derivation_depth
        || actual.max_work > selected.max_work
    {
        return Err(failure(ServiceErrorCode::InvalidConfiguration));
    }
    Ok(())
}

fn build(
    read: &LiveProjectRead,
    host: &ContextPlatformHostInput,
    request: &ContextPlatformRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus, ContextPlatformSummary)> {
    with_qualified_platform_input(read, request, stop, |view| {
        build_qualified(read, host, request, request_id, view, stop)
    })
}

pub(super) fn with_qualified_platform_input<T>(
    read: &LiveProjectRead,
    request: &ContextPlatformRequest,
    stop: &AtomicBool,
    build: impl FnOnce(&wow_project::skeleton_input::SkeletonInputView<'_>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    checkpoint(stop)?;
    let selected = &request.document;
    if read.store_generation_id() != &selected.store_generation_id
        || read.publication_set_id() != selected.publication_set_id.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let candidate = read
        .candidate()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    if candidate.manifest().candidate_id() != selected.candidate_id.as_ref()
        || candidate.manifest().candidate_digest() != selected.candidate_digest.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let binding = SkeletonInputBinding::from_native(candidate.native(), candidate.graph())
        .map_err(project_error)?;
    checkpoint(stop)?;
    if selected.input.binding() != &binding {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let view = open_skeleton_input_view(
        candidate.native(),
        candidate.graph(),
        selected.input.clone(),
        stop,
    )
    .map_err(project_error)?;
    build(&view)
}

fn build_qualified(
    read: &LiveProjectRead,
    host: &ContextPlatformHostInput,
    request: &ContextPlatformRequest,
    request_id: &str,
    view: &wow_project::skeleton_input::SkeletonInputView<'_>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus, ContextPlatformSummary)> {
    let limits = host.limits.owner_limits()?;
    let namespace = read
        .namespace()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    let resolved = (
        request_id,
        host.selection.intent(),
        read.publication_set_id(),
        read.store_generation_id(),
        &host.reference_profile,
        host.reference.generation_id(),
        host.reference.self_digest(),
        &host.limits,
    );
    // Borrow the configured profile, not a cloned native source or Reference.
    let digest = wow_core::domain_separated_digest(
        "wow-service/context-platform-resolved-request/1",
        &resolved,
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-platform-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    checkpoint(stop)?;
    let universe = wow_context::bind_platform_context_universe(
        view,
        &host.reference,
        &host.reference_profile,
        limits,
        stop,
    )
    .map_err(context_error)?;
    let map = wow_context::build_platform_context_map(universe, stop).map_err(context_error)?;
    let output = OutputBinding {
        host,
        read,
        request,
        request_id,
        resolved_request_id: &resolved_request_id,
        native_project_store_id: namespace.id().as_str(),
    };
    match request.operation() {
        ContextPlatformOperation::Map => encode_result(
            &output,
            &map,
            map.canonical_bytes().len(),
            map.remaining_metadata_bytes(),
            (map.map_id(), map.map_id()),
            map.page(),
            stop,
        ),
        ContextPlatformOperation::Inspect => {
            let inspect =
                wow_context::build_platform_context_inspect(map, stop).map_err(context_error)?;
            encode_result(
                &output,
                &inspect,
                inspect.canonical_bytes().len(),
                inspect.remaining_metadata_bytes(),
                (inspect.map_id(), inspect.inspect_id()),
                inspect.page(),
                stop,
            )
        }
    }
}
struct OutputBinding<'a> {
    host: &'a ContextPlatformHostInput,
    read: &'a LiveProjectRead,
    request: &'a ContextPlatformRequest,
    request_id: &'a str,
    resolved_request_id: &'a str,
    native_project_store_id: &'a str,
}
#[derive(Serialize)]
struct Payload<'a, S: Serialize, T: Serialize> {
    resolved_request_id: &'a str,
    selection: &'a S,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    configured_registration_id: &'a str,
    native_project_store_id: &'a str,
    candidate_id: &'a str,
    candidate_digest: &'a str,
    artifact_byte_length: usize,
    summary: &'a BorrowedSummary<'a>,
    artifact: &'a T,
}
fn encode_result<T: Serialize>(
    output: &OutputBinding<'_>,
    artifact: &T,
    artifact_byte_length: usize,
    remaining_metadata_bytes: u64,
    artifact_ids: (&str, &str),
    page: &wow_project::skeleton_input::SkeletonInputPage<'_>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, ContextMapStatus, ContextPlatformSummary)> {
    let status = match page.state() {
        GraphQueryState::Partial => ContextMapStatus::Partial,
        GraphQueryState::Truncated => ContextMapStatus::Truncated,
        _ => return Err(failure(ServiceErrorCode::InternalContractViolation)),
    };
    let summary = BorrowedSummary {
        resolved_request_id: output.resolved_request_id,
        publication_set_id: output.read.publication_set_id(),
        store_generation_id: output.read.store_generation_id().as_str(),
        candidate_id: &output.request.document.candidate_id,
        map_id: artifact_ids.0,
        artifact_id: artifact_ids.1,
        root_id: page.neighborhood().root().node_id().as_str(),
        counts: page.counts(),
        not_evaluated: page.not_evaluated(),
        root_conflicts: page.root_support().explanation().conflicts().len(),
        root_unresolved_evidence: page.root_support().evidence_resolution().unresolved().len(),
        root_support_truncations: page.root_support().explanation().truncations().len(),
        root_evidence_truncations: page
            .root_support()
            .evidence_resolution()
            .truncations()
            .len(),
        absence_authoritative: false,
    };
    let payload = Payload {
        resolved_request_id: output.resolved_request_id,
        selection: output.host.selection.intent(),
        publication_set_id: output.read.publication_set_id(),
        store_generation_id: output.read.store_generation_id().as_str(),
        configured_registration_id: &output.host.project_store_id,
        native_project_store_id: output.native_project_store_id,
        candidate_id: &output.request.document.candidate_id,
        candidate_digest: &output.request.document.candidate_digest,
        artifact_byte_length,
        summary: &summary,
        artifact,
    };
    let envelope = Envelope {
        schema: CONTEXT_PLATFORM_RESULT_SCHEMA,
        operation: output.request.operation().name(),
        request_id: output.request_id,
        request: output.request,
        status,
        resource_closure: "closed",
        failure_code: None,
        payload: Some(payload),
    };
    // Entire final service copy fits the ORIGINAL consumed owner's remainder.
    // No stage resets the ledger or discounts the retained map/inspect bytes.
    let limit = usize::try_from(remaining_metadata_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let envelope_bytes = preflight(&envelope, limit, stop)?;
    let summary_limit = limit
        .checked_sub(envelope_bytes)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    // Charge the extra retained text observation as well as canonical delivery,
    // before either owned copy; this is the same terminal owner remainder.
    preflight(&summary, summary_limit, stop)?;
    let owned_summary = summary.owned();
    Ok((
        encode(&envelope, envelope_bytes, stop)?,
        status,
        owned_summary,
    ))
}
fn project_error(error: wow_project::ProjectError) -> crate::ServiceError {
    failure(match error.code() {
        wow_project::ProjectErrorCode::AnalysisCancelled => ServiceErrorCode::Cancelled,
        wow_project::ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::IdentityMismatch,
    })
}
