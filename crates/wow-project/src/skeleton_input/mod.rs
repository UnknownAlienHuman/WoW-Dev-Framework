//! Owner-neutral native, metadata/handles-only one-hop read. The service requires
//! a sealed composed candidate before acquiring this read primitive.
mod budget;

use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_core::{
    CanonicalResult, ContentDigest, CoverageRecord, GenerationContextId, ProfileId,
    ProjectGenerationId,
};
use wow_graph::{
    GraphAssertionRecordScope, GraphError, GraphErrorCode, GraphEvidenceResolveLimits,
    GraphExplainLimits, GraphExplainQuery, GraphExplainSubject, GraphGenerationId,
    GraphNeighborReadQuery, GraphNeighborView, GraphPartitionSnapshot, GraphQueryState,
    GraphResolvedExplanation, GraphSnapshotId,
};

use crate::{
    ProjectDeferredCapability, ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult,
    graph::{PlatformIndexSourceParts, SOURCE_GRAPH_PARTITION},
    identity::canonical_digest,
    load::ProjectPackageLoadCoverage,
    platform_source::{PlatformAdmissionCoverage, PlatformLicenseRecord},
};

pub const SKELETON_INPUT_PROFILE: &str = "wow-project/skeleton-input/native-one-hop-handles/1";

/// Inert identity data. Only a genuine finished native capability admits a view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonInputBinding {
    project_snapshot_id: Box<str>,
    project_generation: ProjectGenerationId,
    configuration_digest: ContentDigest<CanonicalResult>,
    analyzer_snapshot_id: Box<str>,
    source_snapshot_id: Box<str>,
    source_profile_id: ProfileId,
    source_profile_digest: ContentDigest<CanonicalResult>,
    package_binding_digest: ContentDigest<CanonicalResult>,
    source_context_id: GenerationContextId,
    source_assertion_scope: GraphAssertionRecordScope,
    native_profile: Box<str>,
    native_prefix_snapshot_id: GraphSnapshotId,
    native_prefix_generation: GraphGenerationId,
    registry_digest: Box<str>,
    graph_snapshot_id: GraphSnapshotId,
    graph_generation: GraphGenerationId,
}

impl SkeletonInputBinding {
    /// Inert identity projection only; never creates native/candidate authority.
    pub fn from_native(
        source: &PlatformIndexSourceParts,
        accepted: &GraphPartitionSnapshot,
    ) -> ProjectResult<Self> {
        binding(source, accepted)
    }
}

/// Positive caller limits narrow existing read owners. No project/graph cap changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkeletonInputLimits {
    /// Returned root plus adjacent entity records; a repeated self-loop root counts again.
    pub max_entities: u32,
    /// Combined returned root support, evidence, handle, derivation/conflict rows.
    pub max_support_records: u32,
    pub max_output_bytes: u32,
    pub root_explanation: GraphExplainLimits,
    pub root_evidence: GraphEvidenceResolveLimits,
}

impl SkeletonInputLimits {
    pub fn validate(self) -> ProjectResult<()> {
        self.root_explanation.validate().map_err(graph_error)?;
        self.root_evidence.validate().map_err(graph_error)?;
        let owner_rows = self
            .root_explanation
            .max_supports
            .checked_add(self.root_evidence.max_evidence_records)
            .and_then(|n| n.checked_add(self.root_evidence.max_source_handles))
            .ok_or_else(budget::exhausted)?;
        if self.max_entities == 0
            || self.max_support_records == 0
            || self.max_support_records > owner_rows
            || self.max_output_bytes < 16_384
            || self.max_output_bytes > 8_388_608
            || self.root_explanation.max_output_bytes > self.max_output_bytes
        {
            return Err(invalid());
        }
        Ok(())
    }
}

/// First profile accepts one exact materialized entity root and one direct hop.
/// Package/file/span roots require their actual accepted entity ID. Excerpts,
/// axis paths, fuzzy roots and cursors are not silently interpreted by this DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OpenSkeletonInputRequest {
    binding: SkeletonInputBinding,
    neighborhood: GraphNeighborReadQuery,
    limits: SkeletonInputLimits,
}

impl OpenSkeletonInputRequest {
    pub fn new(
        source: &PlatformIndexSourceParts,
        accepted: &GraphPartitionSnapshot,
        neighborhood: GraphNeighborReadQuery,
        limits: SkeletonInputLimits,
    ) -> ProjectResult<Self> {
        let request = Self {
            binding: binding(source, accepted)?,
            neighborhood,
            limits,
        };
        request.validate()?;
        Ok(request)
    }

    /// Constructs request data only, never source/candidate/graph authority.
    pub fn from_json_slice(bytes: &[u8], stop: &AtomicBool) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        // Inherit the maximum native read envelope, not a new execution capacity.
        if bytes.len() > 8_388_608 {
            return Err(budget::exhausted());
        }
        let request: Self = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        crate::analyzer::checkpoint(stop)?;
        request.validate()?;
        Ok(request)
    }

    pub fn validate(&self) -> ProjectResult<()> {
        self.limits.validate()?;
        self.neighborhood.validate().map_err(graph_error)?;
        if self.neighborhood.snapshot_id() != &self.binding.graph_snapshot_id
            || self.neighborhood.limits().max_output_bytes > self.limits.max_output_bytes
        {
            return Err(invalid());
        }
        Ok(())
    }

    #[must_use]
    pub const fn limits(&self) -> SkeletonInputLimits {
        self.limits
    }

    #[must_use]
    pub const fn binding(&self) -> &SkeletonInputBinding {
        &self.binding
    }

    #[must_use]
    pub const fn neighborhood(&self) -> &GraphNeighborReadQuery {
        &self.neighborhood
    }
}

/// Borrowed sealed native authority and exact accepted input; no serde/Clone owner.
/// Lifetime keeps the native source predecessor and final graph immutable and held.
pub struct SkeletonInputView<'view> {
    source: &'view PlatformIndexSourceParts,
    accepted: &'view GraphPartitionSnapshot,
    request: OpenSkeletonInputRequest,
    query_digest: ContentDigest<CanonicalResult>,
}

pub fn open_skeleton_input_view<'view>(
    source: &'view PlatformIndexSourceParts,
    accepted: &'view GraphPartitionSnapshot,
    request: OpenSkeletonInputRequest,
    stop: &AtomicBool,
) -> ProjectResult<SkeletonInputView<'view>> {
    crate::analyzer::checkpoint(stop)?;
    request.validate()?;
    budget::encoded_size(&request, request.limits.max_output_bytes, stop)?;
    if request.binding != binding(source, accepted)?
        || request.limits.max_entities > accepted.snapshot().limits().max_nodes
    {
        return Err(invalid());
    }
    // Reuse the native pair's exact-prefix rule. No replay, parsing or graph build.
    accepted.validate(stop).map_err(graph_error)?;
    if accepted.registry().registry_digest() != source.registry_digest()
        || accepted.foundation().snapshot_id() != source.foundation_snapshot_id()
        || accepted.source_context_id() != source.scope().source_context_id
        || accepted.partition(SOURCE_GRAPH_PARTITION).is_some()
    {
        return Err(invalid());
    }
    for (partition_id, partition_digest) in source.native_prefix_partitions() {
        crate::analyzer::checkpoint(stop)?;
        if accepted
            .partition(partition_id)
            .is_none_or(|partition| partition.partition_digest() != partition_digest)
        {
            return Err(invalid());
        }
    }
    let query_digest = canonical_digest(SKELETON_INPUT_PROFILE, &request, ProjectPhase::View)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(SkeletonInputView {
        source,
        accepted,
        request,
        query_digest,
    })
}

/// Unsupported facets are explicit NotEvaluated, never an empty complete result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonInputNotEvaluated {
    TypedDeclarationsSignaturesAndTypes,
    PackageLoadRoleProjection,
    NeighborhoodEvidenceClosure,
    RejectionToRootAssociation,
    CoverageReferenceResolution,
    SourceBytesAndExcerpts,
    AxesPathsAndContinuation,
}
const NOT_EVALUATED: &[SkeletonInputNotEvaluated] = &[
    SkeletonInputNotEvaluated::TypedDeclarationsSignaturesAndTypes,
    SkeletonInputNotEvaluated::PackageLoadRoleProjection,
    SkeletonInputNotEvaluated::NeighborhoodEvidenceClosure,
    SkeletonInputNotEvaluated::RejectionToRootAssociation,
    SkeletonInputNotEvaluated::CoverageReferenceResolution,
    SkeletonInputNotEvaluated::SourceBytesAndExcerpts,
    SkeletonInputNotEvaluated::AxesPathsAndContinuation,
];

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SkeletonInputCounts {
    pub scanned_edges: u32,
    pub matching_edges: u32,
    pub omitted_edges: u32,
    pub root_scanned_assertions: u32,
    pub returned_entity_records: usize,
    pub returned_relations: usize,
    pub returned_root_support_records: usize,
    pub root_source_handles: usize,
    pub root_evidence_records: usize,
}

#[derive(Serialize)]
struct SkeletonInputBody<'a> {
    profile: &'static str,
    query_digest: ContentDigest<CanonicalResult>,
    request: &'a OpenSkeletonInputRequest,
    neighborhood: GraphNeighborView<'a>,
    root_support: GraphResolvedExplanation<'a>,
    source_admission: &'a PlatformAdmissionCoverage,
    source_license: &'a PlatformLicenseRecord,
    package_load_coverage: ProjectPackageLoadCoverage,
    package_issue_count: usize,
    project_coverage: &'a [CoverageRecord],
    #[serde(serialize_with = "serialize_deferred")]
    deferred_capabilities: &'a [ProjectDeferredCapability],
    #[serde(skip_serializing_if = "Option::is_none")]
    inventory_span_omissions: Option<usize>,
    not_evaluated: &'static [SkeletonInputNotEvaluated],
    state: GraphQueryState,
    no_new_evidence: bool,
    counts: SkeletonInputCounts,
}

/// Serialize-only observation: its bytes do not reconstruct the held read owner.
#[derive(Serialize)]
pub struct SkeletonInputPage<'a> {
    schema: &'static str,
    body: SkeletonInputBody<'a>,
    canonical_digest: ContentDigest<CanonicalResult>,
    encoded_bytes: usize,
}

impl SkeletonInputPage<'_> {
    #[must_use]
    pub const fn canonical_digest(&self) -> ContentDigest<CanonicalResult> {
        self.canonical_digest
    }
    #[must_use]
    pub const fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    #[must_use]
    pub const fn state(&self) -> GraphQueryState {
        self.body.state
    }
    #[must_use]
    pub const fn counts(&self) -> SkeletonInputCounts {
        self.body.counts
    }
    #[must_use]
    pub fn neighborhood(&self) -> &GraphNeighborView<'_> {
        &self.body.neighborhood
    }
    #[must_use]
    pub fn root_support(&self) -> &GraphResolvedExplanation<'_> {
        &self.body.root_support
    }
    #[must_use]
    pub fn not_evaluated(&self) -> &[SkeletonInputNotEvaluated] {
        self.body.not_evaluated
    }
}

impl SkeletonInputView<'_> {
    pub fn read<'page>(&'page self, stop: &AtomicBool) -> ProjectResult<SkeletonInputPage<'page>> {
        crate::analyzer::checkpoint(stop)?;
        let limits = self.request.limits;
        let neighborhood = self
            .request
            .neighborhood
            .execute(self.accepted.snapshot(), stop)
            .map_err(graph_error)?;
        let entities = neighborhood
            .adjacent_nodes()
            .len()
            .checked_add(1)
            .ok_or_else(budget::exhausted)?;
        if entities > limits.max_entities as usize {
            return Err(budget::exhausted());
        }
        // Explain only the exact root. Neighbor support IDs/coverage remain on
        // their original graph records; their evidence closure is NotEvaluated.
        let explanation = GraphExplainQuery::new(
            self.accepted.snapshot().snapshot_id().clone(),
            GraphExplainSubject::Entity(neighborhood.root().node_id().clone()),
            limits.root_explanation,
        )
        .map_err(graph_error)?;
        let root_support = explanation
            .execute_with_evidence(
                self.accepted,
                self.source.evidence_catalog(),
                limits.root_evidence,
                stop,
            )
            .map_err(graph_error)?;
        crate::analyzer::checkpoint(stop)?;
        let support = root_support.explanation();
        let evidence = root_support.evidence_resolution();
        let records = [
            support.supports().len(),
            support.assertion_supports().len(),
            support.derivations().len(),
            support.conflicts().len(),
            evidence.evidence_records().len(),
            evidence.source_handles().len(),
            evidence.unresolved().len(),
        ]
        .into_iter()
        .try_fold(0usize, |sum, count| {
            sum.checked_add(count).ok_or_else(budget::exhausted)
        })?;
        if records > limits.max_support_records as usize {
            return Err(budget::exhausted());
        }
        let counts = SkeletonInputCounts {
            scanned_edges: neighborhood.scanned_edges(),
            matching_edges: neighborhood.matching_edges(),
            omitted_edges: neighborhood.omitted_edges(),
            root_scanned_assertions: support.scanned_assertions(),
            returned_entity_records: entities,
            returned_relations: neighborhood.edges().len(),
            returned_root_support_records: records,
            root_source_handles: evidence.source_handles().len(),
            root_evidence_records: evidence.evidence_records().len(),
        };
        let project = self.source.project();
        let packages = project
            .configuration()
            .platform_packages()
            .ok_or_else(invalid)?;
        let receipt = packages.source().receipt();
        let body = SkeletonInputBody {
            profile: SKELETON_INPUT_PROFILE,
            query_digest: self.query_digest,
            request: &self.request,
            // Fixed unsupported facets prevent a complete Skeleton frontier claim.
            state: if neighborhood.state() == GraphQueryState::Truncated
                || !root_support.explanation().truncations().is_empty()
                || !root_support.evidence_resolution().truncations().is_empty()
            {
                GraphQueryState::Truncated
            } else {
                GraphQueryState::Partial
            },
            no_new_evidence: false,
            counts,
            neighborhood,
            root_support,
            source_admission: receipt.coverage(),
            source_license: &receipt.inventory().license,
            package_load_coverage: packages.load_plan().coverage(),
            package_issue_count: packages.load_plan().issues().len(),
            project_coverage: project.project_coverage_records(),
            deferred_capabilities: project.deferred_capabilities(),
            inventory_span_omissions: self.source.inventory_span_omissions(),
            not_evaluated: NOT_EVALUATED,
        };
        let mut page = SkeletonInputPage {
            schema: SKELETON_INPUT_PROFILE,
            body,
            canonical_digest: ContentDigest::from_bytes([0; 32]),
            encoded_bytes: 0,
        };
        // Exact whole envelope includes framing, policy, coverage and digest.
        // The fixed-width placeholder has the same encoded length as the digest.
        // Only the decimal counter width can change: monotone, at most the
        // number of decimal digits in the inherited u32 output bound.
        loop {
            let measured = budget::encoded_size(&page, limits.max_output_bytes, stop)?;
            if page.encoded_bytes == measured {
                break;
            }
            page.encoded_bytes = measured;
        }
        page.canonical_digest = canonical_digest(
            SKELETON_INPUT_PROFILE,
            &(&page.body, page.encoded_bytes),
            ProjectPhase::View,
        )?;
        crate::analyzer::checkpoint(stop)?;
        if budget::encoded_size(&page, limits.max_output_bytes, stop)? != page.encoded_bytes {
            return Err(invalid());
        }
        Ok(page)
    }
}

fn binding(
    source: &PlatformIndexSourceParts,
    accepted: &GraphPartitionSnapshot,
) -> ProjectResult<SkeletonInputBinding> {
    let project = source.project();
    let config = project.configuration();
    match config.platform_graph_profile() {
        Some(crate::PlatformGraphProfile::DirectCapabilitiesV1(selected))
            if source.capabilities() == Some(selected) => {}
        Some(crate::PlatformGraphProfile::DirectPlatformProducersWithRawInventoryV1)
            if source.capabilities().is_none() => {}
        _ => return Err(invalid()),
    }
    let packages = config.platform_packages().ok_or_else(invalid)?;
    Ok(SkeletonInputBinding {
        project_snapshot_id: project.snapshot_id().into(),
        project_generation: project.project_generation(),
        configuration_digest: config.configuration_digest(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        source_snapshot_id: packages.source().receipt().source_snapshot_id().into(),
        source_profile_id: packages.source().profile().profile_id().clone(),
        source_profile_digest: packages.source().profile().digest(),
        package_binding_digest: packages.binding().binding_digest(),
        source_context_id: source.scope().source_context_id,
        source_assertion_scope: source.scope().clone(),
        native_profile: source.profile().into(),
        native_prefix_snapshot_id: source.native_prefix_snapshot_id().clone(),
        native_prefix_generation: source.native_prefix_generation().clone(),
        registry_digest: accepted.registry().registry_digest().into(),
        graph_snapshot_id: accepted.snapshot().snapshot_id().clone(),
        graph_generation: accepted.snapshot().generation().clone(),
    })
}

fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::View,
        "native Skeleton input request or exact owner binding is inconsistent",
    )
}

fn graph_error(source: GraphError) -> ProjectError {
    let code = match source.code() {
        GraphErrorCode::Cancelled => ProjectErrorCode::AnalysisCancelled,
        GraphErrorCode::BudgetExceeded => ProjectErrorCode::SourceBudgetExceeded,
        _ => ProjectErrorCode::SnapshotInvalid,
    };
    ProjectError::new(
        code,
        ProjectPhase::View,
        "native Skeleton graph read refused",
    )
}

fn serialize_deferred<S: serde::Serializer>(
    records: &[ProjectDeferredCapability],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    // Borrow original capability IDs and NotEvaluated records; no receipt clone.
    serializer.collect_seq(
        records
            .iter()
            .map(|record| (record.capability_id(), record.record())),
    )
}
