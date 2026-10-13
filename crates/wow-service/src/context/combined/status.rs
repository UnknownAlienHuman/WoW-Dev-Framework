//! Exact Primary + candidate-qualified Platform metadata status.
//! One held metadata observation; no Map, skeleton, pack, or renderer invocation.
use std::{cell::Cell, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, ReferenceGenerationId};
use wow_store::project::{ReadSelector, StoreGenerationId};

use super::super::{
    ContextMapResult, ContextMapStatus, ContextStatusDetail, checkpoint, failure,
    operation::{Envelope, encode, failed_operation, preflight, with_checked_context_read},
    platform::{PLATFORM_NATIVE_ONE_HOP_PROFILE_ID, with_checked_platform_generation},
    status::{GraphCoverage, ProjectDeferred, ReferenceCoverage},
};
use super::{
    COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID, COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    COMBINED_CONTEXT_PACK_PROFILE_ID, COMBINED_NATIVE_MAP_PROFILE_ID, ContextCombinedHostInput,
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};

pub const CONTEXT_COMBINED_STATUS_REQUEST_SCHEMA: &str =
    "wow-service/context-combined-status-request/1";
pub const CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES: usize = crate::graph::GRAPH_REQUEST_MAX_BYTES;
const RESULT_SCHEMA: &str = "wow-service/context-combined-status-result/1";
const OPERATION: &str = "context_combined_status";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrimarySelection {
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformSelection {
    project_store_id: Box<str>,
    store_generation_id: StoreGenerationId,
    publication_set_id: Box<str>,
    candidate_id: Box<str>,
    candidate_digest: Box<str>,
    profile_id: Box<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestDocument {
    schema: Box<str>,
    primary: PrimarySelection,
    platform: PlatformSelection,
    detail: ContextStatusDetail,
}

/// Exact, root-free selectors for one composed metadata observation.
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct ContextCombinedStatusRequest(RequestDocument);

impl ContextCombinedStatusRequest {
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        let document: RequestDocument = crate::graph::input::decode(
            bytes,
            CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES,
            65_536,
            crate::graph::GraphReadStage::Request,
            stop,
        )
        .map_err(|error| failure(error.code))?;
        if document.schema.as_ref() != CONTEXT_COMBINED_STATUS_REQUEST_SCHEMA
            || document.platform.profile_id.as_ref() != PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        super::super::request::registration_id(&document.primary.project_store_id)?;
        super::super::request::registration_id(&document.platform.project_store_id)?;
        super::super::request::identifier(&document.primary.publication_set_id)?;
        super::super::request::identifier(&document.platform.publication_set_id)?;
        super::super::request::identifier(&document.platform.candidate_id)?;
        crate::identity::validate_digest(&document.platform.candidate_digest, "candidate digest")?;
        checkpoint(stop)?;
        Ok(Self(document))
    }
}

#[derive(Serialize)]
struct Capability {
    operation: &'static str,
    availability: &'static str,
    scope: &'static str,
}
const CAPABILITIES: &[Capability] = &[
    Capability {
        operation: "context_combined_status",
        availability: "partial_functional",
        scope: "exact_primary_and_candidate_metadata",
    },
    Capability {
        operation: "context_combined_map",
        availability: "partial_functional",
        scope: "exact_primary_and_candidate_profiles",
    },
    Capability {
        operation: "context_combined_inspect",
        availability: "partial_functional",
        scope: "selected_exact_roots_and_profiles",
    },
    Capability {
        operation: "context_combined_build",
        availability: "partial_functional",
        scope: "selected_exact_roots_and_profiles",
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

#[derive(Serialize)]
struct Profile {
    profile_id: &'static str,
    role: &'static str,
    availability: &'static str,
}
const PROFILES: &[Profile] = &[
    Profile {
        profile_id: COMBINED_NATIVE_MAP_PROFILE_ID,
        role: "combined_map",
        availability: "partial_functional",
    },
    Profile {
        profile_id: COMBINED_CONTEXT_PACK_PROFILE_ID,
        role: "combined_pack",
        availability: "partial_functional",
    },
    Profile {
        profile_id: COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID,
        role: "combined_json",
        availability: "partial_functional",
    },
    Profile {
        profile_id: COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        role: "combined_markdown",
        availability: "partial_functional",
    },
    Profile {
        profile_id: PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
        role: "platform_map_inspect",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        role: "primary_map",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        role: "primary_project_map",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_L0_PROFILE_ID,
        role: "primary_default_l0",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_L1_PROFILE_ID,
        role: "primary_default_l1",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_STATE_ROOT_L0_PROFILE_ID,
        role: "primary_state_root_l0",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        role: "primary_state_detail_l1",
        availability: "partial_functional",
    },
    Profile {
        profile_id: super::super::PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
        role: "primary_reference_facts_l1",
        availability: "partial_functional",
    },
];

pub fn execute_context_combined_status(
    host: &ContextCombinedHostInput,
    request: &ContextCombinedStatusRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let digest = wow_core::domain_separated_digest(CONTEXT_COMBINED_STATUS_REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-combined-status-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let doc = &request.0;
    let admitted: ServiceResult<()> = (|| {
        host.validate(stop)?;
        if doc.primary.project_store_id != host.primary.project_store_id
            || doc.platform.project_store_id != host.platform.project_store_id
            || doc.platform.profile_id.as_ref() != PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        {
            return Err(failure(ServiceErrorCode::InvalidConfiguration));
        }
        let input_limit = usize::try_from(host.primary.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        preflight(
            &(
                request,
                &host.primary.reference_profile,
                &host.primary.reference,
                &host.platform.reference,
                host.platform.selection.intent(),
                &host.primary.limits,
            ),
            input_limit,
            stop,
        )?;
        Ok(())
    })();
    if let Err(error) = admitted {
        return failed_operation(
            request,
            &request_id,
            error.code(),
            "not_acquired",
            RESULT_SCHEMA,
            OPERATION,
        );
    }

    let platform_closure = Cell::new("not_acquired");
    let (outcome, primary_closure) = with_checked_context_read(
        &host.primary,
        &doc.primary.project_store_id,
        &ReadSelector::Exact(doc.primary.store_generation_id.clone()),
        stop,
        |primary_read| {
            if primary_read.store_generation_id() != &doc.primary.store_generation_id
                || primary_read.publication_set_id() != doc.primary.publication_set_id.as_ref()
                || primary_read.project().configuration().project_id() != &host.primary.project_id
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            let (platform_outcome, closure) = with_checked_platform_generation(
                &host.platform,
                &doc.platform.store_generation_id,
                stop,
                |platform_read| {
                    build(
                        primary_read,
                        platform_read,
                        host,
                        request,
                        &request_id,
                        stop,
                    )
                },
            );
            platform_closure.set(closure);
            platform_outcome
        },
    );
    let closure = super::combined_closure(primary_closure, platform_closure.get());
    match outcome {
        Ok(bytes) => Ok(ContextMapResult {
            bytes: bytes.into_boxed_slice(),
            status: ContextMapStatus::Partial,
            code: None,
        }),
        Err(error) => failed_operation(
            request,
            &request_id,
            error.code(),
            closure,
            RESULT_SCHEMA,
            OPERATION,
        ),
    }
}

#[derive(Serialize)]
struct Generations<'a> {
    project_snapshot_id: &'a str,
    project_generation: wow_core::ProjectGenerationId,
    graph_snapshot_id: &'a wow_graph::GraphSnapshotId,
    graph_generation: &'a wow_graph::GraphGenerationId,
    graph_universe: &'a wow_graph::GraphUniverseId,
    graph_registry_digest: &'a str,
    source_registry_id: &'a str,
    analyzer_snapshot_id: &'a str,
}

#[derive(Serialize)]
struct PrimaryMetadata<'a> {
    configured_registration_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    project_snapshot_id: &'a str,
    project_generation: wow_core::ProjectGenerationId,
    graph_snapshot_id: &'a wow_graph::GraphSnapshotId,
    graph_generation: &'a wow_graph::GraphGenerationId,
    graph_universe: &'a wow_graph::GraphUniverseId,
    graph_registry_digest: &'a str,
    project_id: &'a wow_project::ProjectId,
    project_publication_status: wow_project::ProjectPublicationStatus,
    project_coverage: &'a [wow_core::CoverageRecord],
    project_deferred_capabilities: ProjectDeferred<'a>,
    graph_coverage: GraphCoverage<'a>,
    file_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
    source_context_id: wow_core::GenerationContextId,
    #[serde(skip_serializing_if = "Option::is_none")]
    generations: Option<Generations<'a>>,
}

#[derive(Serialize)]
struct PlatformMetadata<'a> {
    configured_registration_id: &'a str,
    native_project_store_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    candidate_id: &'a str,
    candidate_digest: &'a str,
    profile_id: &'static str,
    project_id: &'a wow_project::ProjectId,
    project_snapshot_id: &'a str,
    project_generation: wow_core::ProjectGenerationId,
    graph_snapshot_id: &'a wow_graph::GraphSnapshotId,
    graph_generation: &'a wow_graph::GraphGenerationId,
    graph_universe: &'a wow_graph::GraphUniverseId,
    graph_registry_digest: &'a str,
    project_publication_status: wow_project::ProjectPublicationStatus,
    project_coverage: &'a [wow_core::CoverageRecord],
    project_deferred_capabilities: ProjectDeferred<'a>,
    graph_coverage: GraphCoverage<'a>,
    file_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
    source_context_id: wow_core::GenerationContextId,
    #[serde(skip_serializing_if = "Option::is_none")]
    generations: Option<Generations<'a>>,
}

#[derive(Serialize)]
struct SharedReference<'a> {
    profile_id: &'a wow_core::ProfileId,
    generation_id: &'a str,
    view_digest: &'a str,
    coverage: ReferenceCoverage<'a>,
    conflict_count: usize,
}

#[derive(Serialize)]
struct Payload<'a> {
    resolved_request_id: Box<str>,
    primary: PrimaryMetadata<'a>,
    platform: PlatformMetadata<'a>,
    shared_reference: SharedReference<'a>,
    context_generation: &'static str,
    evaluation: &'static str,
    root_coverage: &'static str,
    continuation_retention: &'static str,
    last_known_good: &'static str,
    failed_targets: &'static str,
    optional_platform_status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities: Option<&'static [Capability]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profiles: Option<&'static [Profile]>,
}

fn generations(read: &LiveProjectRead) -> Generations<'_> {
    let project = read.project();
    let graph = read.graph();
    Generations {
        project_snapshot_id: project.snapshot_id(),
        project_generation: project.project_generation(),
        graph_snapshot_id: graph.snapshot().snapshot_id(),
        graph_generation: graph.snapshot().generation(),
        graph_universe: graph.snapshot().universe(),
        graph_registry_digest: graph.registry().registry_digest(),
        source_registry_id: project.snapshot().source_registry().registry_id(),
        analyzer_snapshot_id: project.analyzer_snapshot_id(),
    }
}

fn build(
    primary_read: &LiveProjectRead,
    platform_read: &LiveProjectRead,
    host: &ContextCombinedHostInput,
    request: &ContextCombinedStatusRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    checkpoint(stop)?;
    let doc = &request.0;
    let primary_project = primary_read.project();
    let platform_project = platform_read.project();
    let platform_namespace = platform_read
        .namespace()
        .ok_or_else(|| failure(ServiceErrorCode::IdentityMismatch))?;
    let platform_candidate = platform_read
        .candidate()
        .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
    if primary_read.publication_set_id() != doc.primary.publication_set_id.as_ref()
        || platform_read.store_generation_id() != &doc.platform.store_generation_id
        || platform_read.publication_set_id() != doc.platform.publication_set_id.as_ref()
        || platform_candidate.manifest().candidate_id() != doc.platform.candidate_id.as_ref()
        || platform_candidate.manifest().candidate_digest()
            != doc.platform.candidate_digest.as_ref()
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    let reference_generation: ReferenceGenerationId = host
        .primary
        .reference
        .generation_id()
        .parse()
        .map_err(|_| failure(ServiceErrorCode::IdentityMismatch))?;
    let primary_configuration = primary_project.configuration();
    let platform_configuration = platform_project.configuration();
    if primary_configuration.project_kind() == wow_project::ProjectKind::BlizzardUiPlatformSource
        || primary_read.graph().source_context_id()
            != primary_project.snapshot().generation_context().context_id()
        || primary_configuration.selected_profile() != &host.primary.reference_profile
        || primary_configuration.reference_generation() != reference_generation
        || platform_configuration.project_kind()
            != wow_project::ProjectKind::BlizzardUiPlatformSource
        || platform_configuration.selected_profile() != &host.platform.reference_profile
        || platform_configuration.reference_generation() != reference_generation
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }

    // Reserve both stores' partition and row work plus the shared Reference scan
    // before validating Reference or constructing a universe/result.
    let detail_work = match doc.detail {
        ContextStatusDetail::Capabilities => CAPABILITIES.len(),
        ContextStatusDetail::Profiles => PROFILES.len(),
        ContextStatusDetail::Summary | ContextStatusDetail::Generations => 0,
    };
    let work_limit = usize::try_from(host.primary.limits.max_context_work)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let mut work = 1usize
        .checked_add(detail_work)
        .and_then(|n| n.checked_add(host.primary.reference.partitions().len()))
        .filter(|n| *n <= work_limit)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    for read in [primary_read, platform_read] {
        let project = read.project();
        let graph = read.graph();
        for count in [
            project.snapshot().project_coverage_records().len(),
            project.snapshot().deferred_capabilities().len(),
            graph.partitions().len(),
        ] {
            work = work
                .checked_add(count)
                .filter(|n| *n <= work_limit)
                .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
        }
        for partition in graph.partitions() {
            checkpoint(stop)?;
            work = work
                .checked_add(partition.coverage().len())
                .filter(|n| *n <= work_limit)
                .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
        }
    }
    let remaining_work = work_limit
        .checked_sub(work)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    let input_limit = usize::try_from(host.primary.limits.max_input_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let (_, primary_reference_work) = host
        .primary
        .reference
        .validate_bounded(input_limit, remaining_work, stop)
        .map_err(|error| match error {
            wow_reference::ReferenceViewError::Cancelled => failure(ServiceErrorCode::Cancelled),
            wow_reference::ReferenceViewError::LimitExceeded => {
                failure(ServiceErrorCode::BudgetExceeded)
            }
            _ => failure(ServiceErrorCode::IdentityMismatch),
        })?;
    let platform_reference_work = remaining_work
        .checked_sub(primary_reference_work)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    host.platform
        .reference
        .validate_bounded(input_limit, platform_reference_work, stop)
        .map_err(|error| match error {
            wow_reference::ReferenceViewError::Cancelled => failure(ServiceErrorCode::Cancelled),
            wow_reference::ReferenceViewError::LimitExceeded => {
                failure(ServiceErrorCode::BudgetExceeded)
            }
            _ => failure(ServiceErrorCode::IdentityMismatch),
        })?;

    const DIGEST_BYTES: usize = "sha256:".len() + 64;
    const RESOLVED_ID_BYTES: usize =
        "context-combined-status-resolved-request:".len() + DIGEST_BYTES;
    let metadata_limit = usize::try_from(host.primary.limits.max_metadata_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let delivery_limit = metadata_limit
        .checked_sub(RESOLVED_ID_BYTES + request_id.len())
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    let resolved = wow_core::domain_separated_digest(
        "wow-service/context-combined-status-resolved-request/1",
        &(
            request_id,
            (
                primary_read.publication_set_id(),
                primary_read.store_generation_id(),
                primary_project.configuration().project_id(),
                generations(primary_read),
                primary_read.graph().source_context_id(),
            ),
            (
                platform_read.publication_set_id(),
                platform_read.store_generation_id(),
                platform_candidate.manifest().candidate_id(),
                platform_candidate.manifest().candidate_digest(),
                host.platform.selection.intent(),
                platform_namespace.id(),
                platform_project.configuration().project_id(),
                generations(platform_read),
                platform_read.graph().source_context_id(),
            ),
            (
                &host.primary.reference_profile,
                host.primary.reference.schema(),
                host.primary.reference.generation_id(),
                host.primary.reference.self_digest(),
            ),
            &host.primary.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-combined-status-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(resolved),
    )
    .into_boxed_str();
    if resolved_request_id.len() != RESOLVED_ID_BYTES {
        return Err(failure(ServiceErrorCode::InternalContractViolation));
    }

    let primary_graph = primary_read.graph();
    let platform_graph = platform_read.graph();
    let primary_snapshot = primary_project.snapshot();
    let platform_snapshot = platform_project.snapshot();
    let payload = Payload {
        resolved_request_id,
        primary: PrimaryMetadata {
            configured_registration_id: &host.primary.project_store_id,
            publication_set_id: primary_read.publication_set_id(),
            store_generation_id: primary_read.store_generation_id().as_str(),
            project_snapshot_id: primary_project.snapshot_id(),
            project_generation: primary_project.project_generation(),
            graph_snapshot_id: primary_graph.snapshot().snapshot_id(),
            graph_generation: primary_graph.snapshot().generation(),
            graph_universe: primary_graph.snapshot().universe(),
            graph_registry_digest: primary_graph.registry().registry_digest(),
            project_id: primary_project.configuration().project_id(),
            project_publication_status: primary_snapshot.publication_status(),
            project_coverage: primary_snapshot.project_coverage_records(),
            project_deferred_capabilities: ProjectDeferred(
                primary_snapshot.deferred_capabilities(),
            ),
            graph_coverage: GraphCoverage(primary_graph.partitions()),
            file_count: primary_project.file_manifest().len(),
            graph_node_count: primary_graph.snapshot().nodes().len(),
            graph_edge_count: primary_graph.snapshot().edges().len(),
            source_context_id: primary_graph.source_context_id(),
            generations: matches!(doc.detail, ContextStatusDetail::Generations)
                .then(|| generations(primary_read)),
        },
        platform: PlatformMetadata {
            configured_registration_id: &host.platform.project_store_id,
            native_project_store_id: platform_namespace.id().as_str(),
            publication_set_id: platform_read.publication_set_id(),
            store_generation_id: platform_read.store_generation_id().as_str(),
            candidate_id: platform_candidate.manifest().candidate_id(),
            candidate_digest: platform_candidate.manifest().candidate_digest(),
            profile_id: PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
            project_id: platform_project.configuration().project_id(),
            project_snapshot_id: platform_project.snapshot_id(),
            project_generation: platform_project.project_generation(),
            graph_snapshot_id: platform_graph.snapshot().snapshot_id(),
            graph_generation: platform_graph.snapshot().generation(),
            graph_universe: platform_graph.snapshot().universe(),
            graph_registry_digest: platform_graph.registry().registry_digest(),
            project_publication_status: platform_snapshot.publication_status(),
            project_coverage: platform_snapshot.project_coverage_records(),
            project_deferred_capabilities: ProjectDeferred(
                platform_snapshot.deferred_capabilities(),
            ),
            graph_coverage: GraphCoverage(platform_graph.partitions()),
            file_count: platform_project.file_manifest().len(),
            graph_node_count: platform_graph.snapshot().nodes().len(),
            graph_edge_count: platform_graph.snapshot().edges().len(),
            source_context_id: platform_graph.source_context_id(),
            generations: matches!(doc.detail, ContextStatusDetail::Generations)
                .then(|| generations(platform_read)),
        },
        shared_reference: SharedReference {
            profile_id: host.primary.reference_profile.profile_id(),
            generation_id: host.primary.reference.generation_id(),
            view_digest: host.primary.reference.self_digest(),
            coverage: ReferenceCoverage(host.primary.reference.partitions()),
            conflict_count: host.primary.reference.conflicts().len(),
        },
        context_generation: "NotEvaluated",
        evaluation: "NotEvaluated",
        root_coverage: "NotEvaluated",
        continuation_retention: "not_implemented",
        last_known_good: "NotEvaluated",
        failed_targets: "NotEvaluated",
        optional_platform_status: "selected_metadata_only",
        capabilities: matches!(doc.detail, ContextStatusDetail::Capabilities)
            .then_some(CAPABILITIES),
        profiles: matches!(doc.detail, ContextStatusDetail::Profiles).then_some(PROFILES),
    };
    encode(
        &Envelope {
            schema: RESULT_SCHEMA,
            operation: OPERATION,
            request_id,
            request,
            status: ContextMapStatus::Partial,
            resource_closure: "closed",
            failure_code: None,
            payload: Some(payload),
        },
        delivery_limit,
        stop,
    )
}
