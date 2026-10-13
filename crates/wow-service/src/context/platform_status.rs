//! Candidate-qualified retained platform metadata without root acquisition.
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_core::ReferenceGenerationId;
use wow_store::project::StoreGenerationId;

use super::{
    ContextMapResult, ContextStatusDetail, checkpoint, failure,
    operation::{Envelope, encode, failed_operation, preflight},
    platform::{
        ContextPlatformHostInput, PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
        with_checked_platform_generation,
    },
    status::{GraphCoverage, ProjectDeferred, ReferenceCoverage},
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};

pub const CONTEXT_PLATFORM_STATUS_REQUEST_SCHEMA: &str =
    "wow-service/context-platform-status-request/1";
pub const CONTEXT_PLATFORM_STATUS_REQUEST_MAX_BYTES: usize = crate::graph::GRAPH_REQUEST_MAX_BYTES;
const RESULT_SCHEMA: &str = "wow-service/context-platform-status-result/1";

#[derive(Serialize)]
struct Capability {
    operation: &'static str,
    availability: &'static str,
    scope: &'static str,
}
#[derive(Serialize)]
struct Profile {
    profile_id: &'static str,
    role: &'static str,
    availability: &'static str,
}
const PLATFORM_CAPABILITIES: &[Capability] = &[
    Capability {
        operation: "context_platform_status",
        availability: "partial_functional",
        scope: "exact_candidate_metadata",
    },
    Capability {
        operation: "context_platform_map",
        availability: "partial_functional",
        scope: "exact_native_root_required",
    },
    Capability {
        operation: "context_platform_inspect",
        availability: "partial_functional",
        scope: "exact_native_root_required",
    },
    Capability {
        operation: "context_map",
        availability: "requires_primary_selection",
        scope: "primary_owner_not_selected",
    },
    Capability {
        operation: "context_inspect",
        availability: "requires_primary_selection",
        scope: "primary_owner_not_selected",
    },
    Capability {
        operation: "context_build",
        availability: "requires_primary_selection",
        scope: "primary_owner_not_selected",
    },
    Capability {
        operation: "context_combined",
        availability: "requires_primary_selection",
        scope: "primary_owner_not_selected",
    },
    Capability {
        operation: "context_continue",
        availability: "not_implemented",
        scope: "retained_continuation",
    },
    Capability {
        operation: "context_validate",
        availability: "not_implemented",
        scope: "external_artifact_admission",
    },
    Capability {
        operation: "context_render",
        availability: "not_implemented",
        scope: "external_artifact_rendering",
    },
];
const PLATFORM_PROFILES: &[Profile] = &[
    Profile {
        profile_id: PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
        role: "platform_map_inspect",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        role: "primary_map",
        availability: "requires_primary_selection",
    },
    Profile {
        profile_id: super::PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        role: "primary_project_map",
        availability: "requires_primary_selection",
    },
    Profile {
        profile_id: super::COMBINED_NATIVE_MAP_PROFILE_ID,
        role: "combined_map",
        availability: "requires_primary_selection",
    },
    Profile {
        profile_id: super::COMBINED_CONTEXT_PACK_PROFILE_ID,
        role: "combined_pack",
        availability: "requires_primary_selection",
    },
];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReferenceGuard {
    profile: wow_core::ProfileId,
    generation: ReferenceGenerationId,
    view: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDocument {
    schema: Box<str>,
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
    candidate_id: Box<str>,
    candidate_digest: Box<str>,
    profile_id: Box<str>,
    detail: ContextStatusDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<ReferenceGuard>,
}

/// Inert exact selector; it contains no root, source, path, or skeleton input.
#[derive(Debug, Clone, Serialize)]
pub struct ContextPlatformStatusRequest {
    #[serde(flatten)]
    document: RequestDocument,
}
impl ContextPlatformStatusRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: RequestDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_PLATFORM_STATUS_REQUEST_MAX_BYTES,
            65_536,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_PLATFORM_STATUS_REQUEST_SCHEMA
            || document.profile_id.as_ref() != PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        super::request::registration_id(&document.project_store_id)?;
        super::request::identifier(&document.publication_set_id)?;
        super::request::identifier(&document.candidate_id)?;
        crate::identity::validate_digest(&document.candidate_digest, "candidate digest")?;
        if let Some(reference) = &document.reference {
            crate::identity::validate_digest(&reference.view, "reference view digest")?;
        }
        checkpoint(stop)?;
        Ok(Self { document })
    }
}

pub fn execute_context_platform_status(
    host: &ContextPlatformHostInput,
    request: &ContextPlatformStatusRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let digest = wow_core::domain_separated_digest(CONTEXT_PLATFORM_STATUS_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let admitted: ServiceResult<()> = (|| {
        host.validate(stop)?;
        if host.project_store_id != request.document.project_store_id
            || request.document.profile_id.as_ref() != PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        let input_limit = usize::try_from(host.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        preflight(
            &(
                request,
                host.selection.intent(),
                &host.reference_profile,
                &host.reference,
                &host.limits,
            ),
            input_limit,
            stop,
        )?;
        // Reserve both fixed-format identities before allocating them and leave
        // the remainder for the final canonical response.
        const DIGEST_LEN: usize = "sha256:".len() + 64;
        const REQUEST_ID_LEN: usize = "context-platform-status-request:".len() + DIGEST_LEN;
        const RESOLVED_ID_LEN: usize =
            "context-platform-status-resolved-request:".len() + DIGEST_LEN;
        let metadata_limit = usize::try_from(host.limits.max_metadata_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        let _reserved_ids = REQUEST_ID_LEN
            .checked_add(RESOLVED_ID_LEN)
            .filter(|n| *n < metadata_limit)
            .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
        Ok(())
    })();
    // Construct the fixed-length retained ID only after input and ID capacity
    // have been admitted. Failure envelopes still receive their deterministic ID.
    let request_id = format!(
        "context-platform-status-request:{}",
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(digest)
    );
    if let Err(error) = admitted {
        return failed_operation(
            request,
            &request_id,
            error.code(),
            "not_acquired",
            RESULT_SCHEMA,
            "context_platform_status",
        );
    }
    let (outcome, closure) = with_checked_platform_generation(
        host,
        &request.document.store_generation_id,
        stop,
        |read| build(read, host, request, &request_id, stop),
    );
    match outcome {
        Ok(bytes) => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status: super::ContextMapStatus::Partial,
            code: None,
        }),
        Err(error) => failed_operation(
            request,
            &request_id,
            error.code(),
            closure,
            RESULT_SCHEMA,
            "context_platform_status",
        ),
    }
}

#[derive(Serialize)]
struct Payload<'a> {
    resolved_request_id: Box<str>,
    configured_registration_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    candidate_id: &'a str,
    candidate_digest: &'a str,
    project_id: &'a wow_project::ProjectId,
    project_publication_status: wow_project::ProjectPublicationStatus,
    project_snapshot_id: &'a str,
    project_generation: wow_core::ProjectGenerationId,
    graph_snapshot_id: &'a wow_graph::GraphSnapshotId,
    graph_generation: &'a wow_graph::GraphGenerationId,
    graph_universe: &'a wow_graph::GraphUniverseId,
    graph_registry_digest: &'a str,
    source_context_id: wow_core::GenerationContextId,
    reference_profile_id: &'a wow_core::ProfileId,
    reference_generation_id: &'a str,
    reference_view_digest: &'a str,
    file_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
    project_coverage: &'a [wow_core::CoverageRecord],
    project_deferred_capabilities: ProjectDeferred<'a>,
    graph_coverage: GraphCoverage<'a>,
    reference_coverage: ReferenceCoverage<'a>,
    reference_conflict_count: usize,
    context_generation: &'static str,
    evaluation: &'static str,
    last_known_good: &'static str,
    failed_targets: &'static str,
    continuation_retention: &'static str,
    root_coverage: &'static str,
    optional_platform_status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities: Option<&'static [Capability]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profiles: Option<&'static [Profile]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generations: Option<GenerationDetail<'a>>,
}
#[derive(Serialize)]
struct GenerationDetail<'a> {
    project_snapshot_id: &'a str,
    project_generation: wow_core::ProjectGenerationId,
    graph_snapshot_id: &'a wow_graph::GraphSnapshotId,
    graph_generation: &'a wow_graph::GraphGenerationId,
    graph_universe: &'a wow_graph::GraphUniverseId,
    graph_registry_digest: &'a str,
    source_registry_id: &'a str,
    analyzer_snapshot_id: &'a str,
}

fn build(
    read: &LiveProjectRead,
    host: &ContextPlatformHostInput,
    request: &ContextPlatformStatusRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    checkpoint(stop)?;
    if read.store_generation_id() != &request.document.store_generation_id
        || read.publication_set_id() != request.document.publication_set_id.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let candidate = read
        .candidate()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    if candidate.manifest().candidate_id() != request.document.candidate_id.as_ref()
        || candidate.manifest().candidate_digest() != request.document.candidate_digest.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let project = read.project();
    let configuration = project.configuration();
    let reference_generation: ReferenceGenerationId = host
        .reference
        .generation_id()
        .parse()
        .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?;
    if configuration.project_kind() != wow_project::ProjectKind::BlizzardUiPlatformSource
        || configuration.selected_profile() != &host.reference_profile
        || configuration.reference_generation() != reference_generation
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    host.reference_profile
        .validate()
        .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?;
    let input_limit = usize::try_from(host.limits.max_input_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let available_work = usize::try_from(host.limits.max_context_work)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    if let Some(expected) = &request.document.reference
        && (expected.profile != *host.reference_profile.profile_id()
            || expected.generation != reference_generation
            || expected.view.as_ref() != host.reference.self_digest())
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    let graph = read.graph();
    // Reserve base work and all partition visits before scanning any nested row.
    let work_limit = available_work;
    let mut work = 1usize
        .checked_add(project.snapshot().project_coverage_records().len())
        .and_then(|n| n.checked_add(project.snapshot().deferred_capabilities().len()))
        .and_then(|n| n.checked_add(graph.partitions().len()))
        .and_then(|n| n.checked_add(host.reference.partitions().len()))
        .and_then(|n| {
            n.checked_add(match request.document.detail {
                ContextStatusDetail::Capabilities => PLATFORM_CAPABILITIES.len(),
                ContextStatusDetail::Profiles => PLATFORM_PROFILES.len(),
                ContextStatusDetail::Summary | ContextStatusDetail::Generations => 0,
            })
        })
        .filter(|n| *n <= work_limit)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    for partition in graph.partitions() {
        checkpoint(stop)?;
        for _row in partition.coverage() {
            checkpoint(stop)?;
            work = work
                .checked_add(1)
                .filter(|n| *n <= work_limit)
                .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
        }
    }
    let remaining_work = available_work
        .checked_sub(work)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    host.reference
        .validate_bounded(input_limit, remaining_work, stop)
        .map_err(|error| match error {
            wow_reference::ReferenceViewError::Cancelled => failure(ServiceErrorCode::Cancelled),
            wow_reference::ReferenceViewError::LimitExceeded => {
                failure(ServiceErrorCode::BudgetExceeded)
            }
            _ => failure(ServiceErrorCode::IdentityMismatch),
        })?;
    checkpoint(stop)?;
    let metadata_limit = usize::try_from(host.limits.max_metadata_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    const RESOLVED_ID_BYTES: usize =
        "context-platform-status-resolved-request:".len() + "sha256:".len() + 64;
    let delivery_limit = metadata_limit
        .checked_sub(request_id.len())
        .and_then(|n| n.checked_sub(RESOLVED_ID_BYTES))
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    let resolved = wow_core::domain_separated_digest(
        "wow-service/context-platform-status-resolved-request/1",
        &(
            request_id,
            read.publication_set_id(),
            read.store_generation_id(),
            candidate.manifest().candidate_id(),
            candidate.manifest().candidate_digest(),
            host.selection.intent(),
            &host.reference_profile,
            host.reference.generation_id(),
            host.reference.self_digest(),
            &host.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-platform-status-resolved-request:{}",
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(resolved)
    )
    .into_boxed_str();
    let payload = Payload {
        resolved_request_id,
        configured_registration_id: &host.project_store_id,
        publication_set_id: read.publication_set_id(),
        store_generation_id: read.store_generation_id().as_str(),
        candidate_id: candidate.manifest().candidate_id(),
        candidate_digest: candidate.manifest().candidate_digest(),
        project_id: project.configuration().project_id(),
        project_publication_status: project.snapshot().publication_status(),
        project_snapshot_id: project.snapshot_id(),
        project_generation: project.project_generation(),
        graph_snapshot_id: graph.snapshot().snapshot_id(),
        graph_generation: graph.snapshot().generation(),
        graph_universe: graph.snapshot().universe(),
        graph_registry_digest: graph.registry().registry_digest(),
        source_context_id: graph.source_context_id(),
        reference_profile_id: host.reference_profile.profile_id(),
        reference_generation_id: host.reference.generation_id(),
        reference_view_digest: host.reference.self_digest(),
        file_count: project.file_manifest().len(),
        graph_node_count: graph.snapshot().nodes().len(),
        graph_edge_count: graph.snapshot().edges().len(),
        project_coverage: project.snapshot().project_coverage_records(),
        project_deferred_capabilities: ProjectDeferred(project.snapshot().deferred_capabilities()),
        graph_coverage: GraphCoverage(graph.partitions()),
        reference_coverage: ReferenceCoverage(host.reference.partitions()),
        reference_conflict_count: host.reference.conflicts().len(),
        context_generation: "NotEvaluated",
        evaluation: "NotEvaluated",
        last_known_good: "NotEvaluated",
        failed_targets: "NotEvaluated",
        continuation_retention: "not_implemented",
        root_coverage: "NotEvaluated",
        optional_platform_status: "selected_metadata_only",
        capabilities: (request.document.detail == ContextStatusDetail::Capabilities)
            .then_some(PLATFORM_CAPABILITIES),
        profiles: (request.document.detail == ContextStatusDetail::Profiles)
            .then_some(PLATFORM_PROFILES),
        generations: (request.document.detail == ContextStatusDetail::Generations).then_some(
            GenerationDetail {
                project_snapshot_id: project.snapshot_id(),
                project_generation: project.project_generation(),
                graph_snapshot_id: graph.snapshot().snapshot_id(),
                graph_generation: graph.snapshot().generation(),
                graph_universe: graph.snapshot().universe(),
                graph_registry_digest: graph.registry().registry_digest(),
                source_registry_id: project.snapshot().source_registry().registry_id(),
                analyzer_snapshot_id: project.analyzer_snapshot_id(),
            },
        ),
    };
    encode(
        &Envelope {
            schema: RESULT_SCHEMA,
            operation: "context_platform_status",
            request_id,
            request,
            status: super::ContextMapStatus::Partial,
            resource_closure: "closed",
            failure_code: None,
            payload: Some(payload),
        },
        delivery_limit,
        stop,
    )
}
