//! Metadata from one genuine held publication, without context-artifact generation.
mod registry;

use serde::{Serialize, Serializer, ser::SerializeSeq};
use std::sync::atomic::AtomicBool;
use wow_core::{CanonicalResult, ContentDigest};
use wow_graph::GraphProducerPartition;
use wow_reference::ReferencePartition;
use wow_store::project::ReadSelector;

use super::{
    ContextMapCurrentGuard, ContextMapHostInput, ContextMapPublication, ContextMapReferenceGuard,
    ContextMapResult, ContextMapStatus, checkpoint, context_error, failure,
    operation::{
        Envelope, current_record_digest, encode, failed_operation, preflight,
        with_checked_context_read,
    },
};
use crate::{ServiceErrorCode, ServiceResult, live_project::LiveProjectRead};

const REQUEST_SCHEMA: &str = "wow-service/context-status-request/1";
const RESULT_SCHEMA: &str = "wow-service/context-status-result/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextStatusDetail {
    Summary,
    Capabilities,
    Profiles,
    Generations,
}

impl<'de> serde::Deserialize<'de> for ContextStatusDetail {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <String as serde::Deserialize>::deserialize(deserializer)?;
        match value.as_str() {
            "summary" => Ok(Self::Summary),
            "capabilities" => Ok(Self::Capabilities),
            "profiles" => Ok(Self::Profiles),
            "generations" => Ok(Self::Generations),
            _ => Err(serde::de::Error::unknown_variant(
                &value,
                &["summary", "capabilities", "profiles", "generations"],
            )),
        }
    }
}

/// Inert selection. Host registrations and filesystem paths are separate.
#[derive(Debug, Clone, Serialize)]
pub struct ContextStatusRequest {
    schema: &'static str,
    project_store_id: Box<str>,
    publication: ContextMapPublication,
    platform: &'static str,
    detail: ContextStatusDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<ContextMapCurrentGuard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<ContextMapReferenceGuard>,
}
impl ContextStatusRequest {
    pub fn primary(
        store: &str,
        publication: ContextMapPublication,
        detail: ContextStatusDetail,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> ServiceResult<Self> {
        super::request::registration_id(store)?;
        if current.is_some() && !matches!(publication, ContextMapPublication::Current) {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        Ok(Self {
            schema: REQUEST_SCHEMA,
            project_store_id: store.into(),
            publication,
            platform: "omitted",
            detail,
            current,
            reference,
        })
    }
}

pub fn execute_context_status(
    host: &ContextMapHostInput,
    request: &ContextStatusRequest,
    stop: &AtomicBool,
) -> ServiceResult<ContextMapResult> {
    let digest = wow_core::domain_separated_digest(REQUEST_SCHEMA, request)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let request_id = format!(
        "context-status-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    );
    let selector = match &request.publication {
        ContextMapPublication::Current => ReadSelector::Current,
        ContextMapPublication::StoreGeneration(id) => ReadSelector::Exact(id.clone()),
    };
    // Admit the immutable request/configuration before any retained owner read.
    let admitted: ServiceResult<()> = (|| {
        host.validate(stop)?;
        let input_limit = usize::try_from(host.limits.max_input_bytes)
            .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
        preflight(
            &(
                request,
                &host.reference_profile,
                &host.reference,
                &host.limits,
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
            "context_status",
        );
    }
    let (outcome, closure) =
        with_checked_context_read(host, &request.project_store_id, &selector, stop, |read| {
            build(read, host, request, &request_id, stop)
        });
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
            "context_status",
        ),
    }
}

#[derive(Serialize)]
struct Payload<'a> {
    resolved_request_id: Box<str>,
    configured_registration_id: &'a str,
    publication_set_id: &'a str,
    store_generation_id: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_at_acquisition: Option<&'a wow_store::project::CurrentPublication>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_record_digest: Option<Box<str>>,
    context_universe_set_id: &'a str,
    project_id: &'a wow_project::ProjectId,
    reference_profile_id: &'a wow_core::ProfileId,
    reference_generation_id: &'a str,
    reference_view_digest: &'a str,
    source_context_id: wow_core::GenerationContextId,
    project_publication_status: wow_project::ProjectPublicationStatus,
    project_coverage: &'a [wow_core::CoverageRecord],
    project_deferred_capabilities: ProjectDeferred<'a>,
    reference_coverage: ReferenceCoverage<'a>,
    reference_conflict_count: usize,
    graph_coverage: GraphCoverage<'a>,
    file_count: usize,
    graph_node_count: usize,
    graph_edge_count: usize,
    context_generation: &'static str,
    evaluation: &'static str,
    continuation_retention: &'static str,
    optional_platform_status: &'static str,
    last_known_good: &'static str,
    failed_targets: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities: Option<&'static [registry::Capability]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profiles: Option<&'static [registry::Profile]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generations: Option<Generations<'a>>,
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

// Counted writers traverse original bounded metadata only. No record/body copies.
pub(super) struct ReferenceCoverage<'a>(pub(super) &'a [ReferencePartition]);
pub(super) struct ProjectDeferred<'a>(pub(super) &'a [wow_project::ProjectDeferredCapability]);
impl Serialize for ProjectDeferred<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            capability_id: &'a wow_core::CapabilityId,
            record: &'a wow_core::NotEvaluatedRecord,
        }
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for capability in self.0 {
            rows.serialize_element(&Row {
                capability_id: capability.capability_id(),
                record: capability.record(),
            })?;
        }
        rows.end()
    }
}
impl Serialize for ReferenceCoverage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            partition_id: &'a str,
            coverage: wow_reference::CoverageStatus,
            record_count: usize,
        }
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for partition in self.0 {
            rows.serialize_element(&Row {
                partition_id: partition.id(),
                coverage: partition.coverage(),
                record_count: partition.records().len(),
            })?;
        }
        rows.end()
    }
}
pub(super) struct GraphCoverage<'a>(pub(super) &'a [GraphProducerPartition]);
impl Serialize for GraphCoverage<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Row<'a> {
            partition_id: &'a str,
            partition_digest: &'a str,
            coverage: &'a [wow_graph::GraphCoverageRecord],
        }
        let mut rows = serializer.serialize_seq(Some(self.0.len()))?;
        for partition in self.0 {
            rows.serialize_element(&Row {
                partition_id: partition.partition_id(),
                partition_digest: partition.partition_digest(),
                coverage: partition.coverage(),
            })?;
        }
        rows.end()
    }
}

fn build(
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    request: &ContextStatusRequest,
    request_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<Vec<u8>> {
    checkpoint(stop)?;
    let project = read.project();
    if project.configuration().project_id() != &host.project_id {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let current = read.current_at_acquisition();
    // These identity formats are fixed by the canonical digest and universe
    // contracts. Reserve their retained bytes before constructing any strings.
    const DIGEST_BYTES: usize = "sha256:".len() + 64;
    const RESOLVED_ID_BYTES: usize = "context-status-resolved-request:".len() + DIGEST_BYTES;
    const UNIVERSE_ID_BYTES: usize = "context-universe:sha256:".len() + 64;
    let metadata_limit = usize::try_from(host.limits.max_metadata_bytes)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    let retained =
        RESOLVED_ID_BYTES + UNIVERSE_ID_BYTES + if current.is_some() { DIGEST_BYTES } else { 0 };
    let delivery_limit = metadata_limit
        .checked_sub(retained)
        .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    // Hashing Current also consumes canonical metadata; count it first.
    preflight(&current, delivery_limit, stop)?;
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
    let limits = wow_context::PrimaryMapLimits::new(
        host.limits.neighbors,
        host.limits.explain,
        host.limits.max_direct_edges,
        host.limits.max_input_bytes,
        u64::try_from(delivery_limit).map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?,
        host.limits.max_context_work,
    )
    .map_err(context_error)?;
    let universe = wow_context::bind_context_universe_set(
        read.pair(),
        &host.reference,
        &host.reference_profile,
        limits,
        stop,
    )
    .map_err(context_error)?;
    let snapshot = project.snapshot();
    let graph = read.graph();
    let mut work = 1usize;
    for count in [
        snapshot.project_coverage_records().len(),
        snapshot.deferred_capabilities().len(),
        host.reference.partitions().len(),
        graph.partitions().len(),
        match request.detail {
            ContextStatusDetail::Capabilities => registry::CAPABILITIES.len(),
            ContextStatusDetail::Profiles => registry::PROFILES.len(),
            ContextStatusDetail::Summary | ContextStatusDetail::Generations => 0,
        },
    ] {
        work = work
            .checked_add(count)
            .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    }
    let work_limit = usize::try_from(host.limits.max_context_work)
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded))?;
    if work > work_limit {
        return Err(failure(ServiceErrorCode::BudgetExceeded));
    }
    // Partition traversal is reserved above; each coverage row is reserved
    // before the later serializers visit it.
    for partition in graph.partitions() {
        checkpoint(stop)?;
        work = work
            .checked_add(partition.coverage().len())
            .filter(|work| *work <= work_limit)
            .ok_or_else(|| failure(ServiceErrorCode::BudgetExceeded))?;
    }
    let resolved_digest = wow_core::domain_separated_digest(
        "wow-service/context-status-resolved-request/1",
        &(
            request_id,
            read.publication_set_id(),
            read.store_generation_id(),
            universe.universe_set_id(),
            &host.limits,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let resolved_request_id = format!(
        "context-status-resolved-request:{}",
        ContentDigest::<CanonicalResult>::from_bytes(resolved_digest)
    )
    .into_boxed_str();
    if resolved_request_id.len() != RESOLVED_ID_BYTES
        || universe.universe_set_id().len() != UNIVERSE_ID_BYTES
        || current_digest
            .as_ref()
            .is_some_and(|id| id.len() != DIGEST_BYTES)
    {
        return Err(failure(ServiceErrorCode::InternalContractViolation));
    }
    let payload = Payload {
        resolved_request_id,
        configured_registration_id: &host.project_store_id,
        publication_set_id: read.publication_set_id(),
        store_generation_id: read.store_generation_id().as_str(),
        current_at_acquisition: current,
        current_record_digest: current_digest,
        context_universe_set_id: universe.universe_set_id(),
        project_id: project.configuration().project_id(),
        reference_profile_id: host.reference_profile.profile_id(),
        reference_generation_id: host.reference.generation_id(),
        reference_view_digest: host.reference.self_digest(),
        source_context_id: graph.source_context_id(),
        project_publication_status: snapshot.publication_status(),
        project_coverage: snapshot.project_coverage_records(),
        project_deferred_capabilities: ProjectDeferred(snapshot.deferred_capabilities()),
        reference_coverage: ReferenceCoverage(host.reference.partitions()),
        reference_conflict_count: host.reference.conflicts().len(),
        graph_coverage: GraphCoverage(graph.partitions()),
        file_count: project.file_manifest().len(),
        graph_node_count: graph.snapshot().nodes().len(),
        graph_edge_count: graph.snapshot().edges().len(),
        context_generation: "NotEvaluated",
        evaluation: "NotEvaluated",
        continuation_retention: if host.continuation_store_file.is_some() {
            "registered"
        } else {
            "not_configured"
        },
        optional_platform_status: "not_selected",
        last_known_good: "NotEvaluated",
        failed_targets: "NotEvaluated",
        capabilities: matches!(request.detail, ContextStatusDetail::Capabilities)
            .then_some(registry::CAPABILITIES),
        profiles: matches!(request.detail, ContextStatusDetail::Profiles)
            .then_some(registry::PROFILES),
        generations: matches!(request.detail, ContextStatusDetail::Generations).then_some(
            Generations {
                project_snapshot_id: project.snapshot_id(),
                project_generation: project.project_generation(),
                graph_snapshot_id: graph.snapshot().snapshot_id(),
                graph_generation: graph.snapshot().generation(),
                graph_universe: graph.snapshot().universe(),
                graph_registry_digest: graph.registry().registry_digest(),
                source_registry_id: snapshot.source_registry().registry_id(),
                analyzer_snapshot_id: project.analyzer_snapshot_id(),
            },
        ),
    };
    encode(
        &Envelope {
            schema: RESULT_SCHEMA,
            operation: "context_status",
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
