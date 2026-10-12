//! Selected platform metadata/handles over one genuine retained Skeleton input.
//! Primary-only universe admission and its serialized artifacts stay unchanged.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{ProfileIdentity, ReferenceGenerationId};
use wow_graph::{GraphNeighborView, GraphNode, GraphQueryState, GraphResolvedExplanation};
use wow_project::skeleton_input::{
    OpenSkeletonInputRequest, SkeletonInputNotEvaluated, SkeletonInputPage, SkeletonInputView,
};
use wow_project::{ProjectError, ProjectErrorCode, ProjectKind};
use wow_reference::{ReferenceConflict, ReferenceView, ReferenceViewError};

use crate::budget::{Budget, canonical, checkpoint, encoded_len, exhausted, identity};
use crate::universe::ReferenceCoverage;
use crate::{ContextError, ContextErrorCode, ContextResult, PrimaryMapLimits};

/// One selected family; artifact roles distinguish universe, map and inspect.
pub const PLATFORM_NATIVE_ONE_HOP_PROFILE: &str = "wow-context/platform-native-one-hop/1";
pub const PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES: &[u8] = br#"{"native":"genuine_admitted_platform_skeleton_input","neighborhood":"one_exact_native_query","profile":"wow-context/platform-native-one-hop/1","reference":"configured_profile_generation_and_validated_full_content_binding","runtime":"not_evaluated","source":"metadata_and_handles_only","support":"original_root_support_neighbor_evidence_not_evaluated","tokens":"not_evaluated"}"#;

#[derive(Serialize)]
struct PlatformProfile {
    native: &'static str,
    neighborhood: &'static str,
    profile: &'static str,
    reference: &'static str,
    runtime: &'static str,
    source: &'static str,
    support: &'static str,
    tokens: &'static str,
}
const PROFILE: PlatformProfile = PlatformProfile {
    native: "genuine_admitted_platform_skeleton_input",
    neighborhood: "one_exact_native_query",
    profile: PLATFORM_NATIVE_ONE_HOP_PROFILE,
    reference: "configured_profile_generation_and_validated_full_content_binding",
    runtime: "not_evaluated",
    source: "metadata_and_handles_only",
    support: "original_root_support_neighbor_evidence_not_evaluated",
    tokens: "not_evaluated",
};

#[derive(Serialize)]
struct PlatformBinding<'a> {
    family: &'static str,
    artifact_kind: &'static str,
    profile: &'static PlatformProfile,
    native_request: &'a OpenSkeletonInputRequest,
    target_profile: &'a ProfileIdentity,
    reference_generation: ReferenceGenerationId,
    reference_schema: &'a str,
    reference_view_digest: &'a str,
    reference_coverage: Vec<ReferenceCoverage<'a>>,
    reference_conflicts: &'a [ReferenceConflict],
    compatibility_basis: &'static str,
    limits: PrimaryMapLimits,
    input_bytes_reserved: u64,
}

/// Borrowed qualified view and the original continuing context ledger.
/// No decoding, cloning or public field constructor can create this authority.
pub struct PlatformContextUniverse<'a> {
    view: &'a SkeletonInputView<'a>,
    binding: PlatformBinding<'a>,
    universe_id: Box<str>,
    budget: Budget,
}
impl PlatformContextUniverse<'_> {
    #[must_use]
    pub fn universe_id(&self) -> &str {
        &self.universe_id
    }
}

/// Admit the selected platform universe without weakening primary admission.
pub fn bind_platform_context_universe<'a>(
    view: &'a SkeletonInputView<'a>,
    reference: &'a ReferenceView,
    reference_profile: &'a ProfileIdentity,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextUniverse<'a>> {
    bind_platform_context_universe_with_budget(
        view,
        reference,
        reference_profile,
        limits,
        Budget::new(limits),
        0,
        stop,
    )
}

// The standalone wrapper creates its own ledger; composition transfers the
// actual Primary ledger here before any additional platform validation/read.
pub(crate) fn bind_platform_context_universe_with_budget<'a>(
    view: &'a SkeletonInputView<'a>,
    reference: &'a ReferenceView,
    reference_profile: &'a ProfileIdentity,
    limits: PrimaryMapLimits,
    mut budget: Budget,
    prior_input_bytes: u64,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextUniverse<'a>> {
    checkpoint(stop)?;
    let (source, accepted, request) = view.context_parts();
    let configuration = source.project().configuration();
    if configuration.project_kind() != ProjectKind::BlizzardUiPlatformSource
        || configuration.selected_profile() != reference_profile
        || accepted.source_context_id() != source.scope().source_context_id
        || request.neighborhood().snapshot_id() != accepted.snapshot().snapshot_id()
        || request.neighborhood().limits().max_scanned_edges > limits.neighbors.max_scanned_edges
        || request.neighborhood().limits().max_output_bytes > limits.neighbors.max_output_bytes
        || request.neighborhood().query().max_edges() > limits.max_direct_edges
        || request.limits().root_explanation.max_scanned_assertions
            > limits.explain.max_scanned_assertions
        || request.limits().root_explanation.max_supports > limits.explain.max_supports
        || request.limits().root_explanation.max_output_bytes > limits.explain.max_output_bytes
        || request.limits().root_explanation.max_derivation_depth
            > limits.explain.max_derivation_depth
    {
        return Err(mismatch());
    }
    budget.work(1, stop)?;
    let input_bytes_reserved = encoded_len(
        &(reference_profile, reference, request),
        limits
            .max_input_bytes
            .checked_sub(prior_input_bytes)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let profile_validation = reference_profile.validate();
    checkpoint(stop)?;
    profile_validation.map_err(|_| mismatch())?;
    let generation: ReferenceGenerationId =
        reference.generation_id().parse().map_err(|_| mismatch())?;
    if generation != configuration.reference_generation() {
        return Err(mismatch());
    }
    let validation = reference.validate_bounded(
        usize::try_from(limits.max_input_bytes).map_err(|_| exhausted())?,
        usize::try_from(
            limits
                .max_context_work
                .checked_sub(budget.work)
                .ok_or_else(exhausted)?,
        )
        .map_err(|_| exhausted())?,
        stop,
    );
    checkpoint(stop)?;
    let (_, steps) = validation.map_err(reference_error)?;
    budget.work(steps, stop)?;
    budget.retain(&PROFILE, stop)?;
    if canonical(
        &PROFILE,
        u64::try_from(PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES.len()).map_err(|_| exhausted())?,
        stop,
    )?
    .as_slice()
        != PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    let mut reference_coverage = Vec::new();
    for partition in reference.partitions() {
        budget.work(1, stop)?;
        let row = ReferenceCoverage {
            partition_id: partition.id(),
            state: partition.coverage(),
        };
        budget.retain(&row, stop)?;
        reference_coverage.push(row);
    }
    let binding = PlatformBinding {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "universe",
        profile: &PROFILE,
        native_request: request,
        target_profile: reference_profile,
        reference_generation: generation,
        reference_schema: reference.schema(),
        reference_view_digest: reference.self_digest(),
        reference_coverage,
        reference_conflicts: reference.conflicts(),
        compatibility_basis: "exact_configured_generation_and_explicit_caller_profile_checked_full_view",
        limits,
        input_bytes_reserved,
    };
    budget.retain(&binding, stop)?;
    let universe_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-universe:sha256:",
        &binding,
        remaining(&budget, limits)?,
        stop,
    )?;
    budget.retain(&universe_id, stop)?;
    checkpoint(stop)?;
    Ok(PlatformContextUniverse {
        view,
        binding,
        universe_id,
        budget,
    })
}

#[derive(Serialize)]
struct PlatformMapCore<'a> {
    universe_id: Box<str>,
    binding: PlatformBinding<'a>,
    native: SkeletonInputPage<'a>,
    absence_authoritative: bool,
}
#[derive(Serialize)]
pub(crate) struct PlatformMapArtifact<'a> {
    family: &'static str,
    artifact_kind: &'static str,
    map_id: Box<str>,
    core: PlatformMapCore<'a>,
}

impl<'a> PlatformMapArtifact<'a> {
    pub(crate) fn map_id(&self) -> &str {
        &self.map_id
    }
    pub(crate) fn page(&self) -> &SkeletonInputPage<'a> {
        &self.core.native
    }
    pub(crate) fn input_bytes_reserved(&self) -> u64 {
        self.core.binding.input_bytes_reserved
    }
}

/// Serialize-only output retaining the original native page and its view lifetime.
#[derive(Serialize)]
pub struct PlatformContextMap<'a> {
    #[serde(flatten)]
    artifact: PlatformMapArtifact<'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl<'a> PlatformContextMap<'a> {
    pub(crate) fn into_parts(self) -> (PlatformMapArtifact<'a>, Box<[u8]>, Budget) {
        (self.artifact, self.canonical_bytes, self.budget)
    }
    #[must_use]
    pub fn map_id(&self) -> &str {
        &self.artifact.map_id
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn page(&self) -> &SkeletonInputPage<'_> {
        &self.artifact.core.native
    }
    #[must_use]
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.artifact.core.binding.limits.max_metadata_bytes - self.budget.retained_bytes
    }
}

/// Consume the admitted ledger; invoke the genuine native one-hop read once.
pub fn build_platform_context_map<'a>(
    universe: PlatformContextUniverse<'a>,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextMap<'a>> {
    checkpoint(stop)?;
    let PlatformContextUniverse {
        view,
        binding,
        universe_id,
        mut budget,
    } = universe;
    let limits = binding.limits;
    // Bound native materialization before it occurs; commit exact retained bytes
    // below. The original immutable request and native ceilings are unchanged.
    if u64::from(view.context_parts().2.limits().max_output_bytes) > remaining(&budget, limits)? {
        return Err(exhausted());
    }
    // Native neighbors scans the full edge population even when few rows match.
    // Reserve that exact scan plus the caller's root explanation/evidence ceilings
    // before executing; returned-row charges below cover the context projection.
    let (_, accepted, request) = view.context_parts();
    let native_scan_work = accepted
        .snapshot()
        .edges()
        .len()
        .checked_add(request.limits().root_explanation.max_scanned_assertions as usize)
        .and_then(|work| work.checked_add(request.limits().root_evidence.max_work as usize))
        .and_then(|work| work.checked_add(1))
        .ok_or_else(exhausted)?;
    budget.work(native_scan_work, stop)?;
    let page = view.read(stop).map_err(project_error)?;
    checkpoint(stop)?;
    let counts = page.counts();
    budget.work(
        counts
            .returned_entity_records
            .checked_add(counts.returned_relations)
            .and_then(|n| n.checked_add(counts.returned_root_support_records))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let core = PlatformMapCore {
        universe_id,
        binding,
        native: page,
        absence_authoritative: false,
    };
    budget.retain(&core, stop)?;
    let map_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-map:sha256:",
        &("map", &core),
        remaining(&budget, limits)?,
        stop,
    )?;
    let artifact = PlatformMapArtifact {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "map",
        map_id,
        core,
    };
    let canonical_bytes = encode_retained(&artifact, &mut budget, limits, stop)?;
    Ok(PlatformContextMap {
        artifact,
        canonical_bytes,
        budget,
    })
}

#[derive(Serialize)]
struct PlatformInspectArtifact<'a> {
    family: &'static str,
    artifact_kind: &'static str,
    inspect_id: Box<str>,
    map: PlatformMapArtifact<'a>,
}

/// Selected root/native-neighborhood metadata and original root support only.
#[derive(Serialize)]
pub struct PlatformContextInspect<'a> {
    #[serde(flatten)]
    artifact: PlatformInspectArtifact<'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    map_canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl PlatformContextInspect<'_> {
    #[must_use]
    pub fn inspect_id(&self) -> &str {
        &self.artifact.inspect_id
    }
    #[must_use]
    pub fn map_id(&self) -> &str {
        &self.artifact.map.map_id
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn map_canonical_bytes(&self) -> &[u8] {
        &self.map_canonical_bytes
    }
    #[must_use]
    pub fn page(&self) -> &SkeletonInputPage<'_> {
        &self.artifact.map.core.native
    }
    #[must_use]
    pub fn root(&self) -> &GraphNode {
        self.page().neighborhood().root()
    }
    #[must_use]
    pub fn neighborhood(&self) -> &GraphNeighborView<'_> {
        self.page().neighborhood()
    }
    #[must_use]
    pub fn root_support(&self) -> &GraphResolvedExplanation<'_> {
        self.page().root_support()
    }
    #[must_use]
    pub fn not_evaluated(&self) -> &[SkeletonInputNotEvaluated] {
        self.page().not_evaluated()
    }
    #[must_use]
    pub fn state(&self) -> GraphQueryState {
        self.page().state()
    }
    #[must_use]
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.artifact.map.core.binding.limits.max_metadata_bytes - self.budget.retained_bytes
    }
}

/// Consume the actual map/ledger without another native read or receipt copy.
pub fn build_platform_context_inspect<'a>(
    map: PlatformContextMap<'a>,
    stop: &AtomicBool,
) -> ContextResult<PlatformContextInspect<'a>> {
    checkpoint(stop)?;
    let PlatformContextMap {
        artifact: map,
        canonical_bytes: map_canonical_bytes,
        mut budget,
    } = map;
    let limits = map.core.binding.limits;
    budget.work(1, stop)?;
    let inspect_id = identity(
        PLATFORM_NATIVE_ONE_HOP_PROFILE,
        "platform-context-inspect:sha256:",
        &("inspect", &map),
        remaining(&budget, limits)?,
        stop,
    )?;
    let artifact = PlatformInspectArtifact {
        family: PLATFORM_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "inspect",
        inspect_id,
        map,
    };
    let canonical_bytes = encode_retained(&artifact, &mut budget, limits, stop)?;
    Ok(PlatformContextInspect {
        artifact,
        canonical_bytes,
        map_canonical_bytes,
        budget,
    })
}

fn remaining(budget: &Budget, limits: PrimaryMapLimits) -> ContextResult<u64> {
    limits
        .max_metadata_bytes
        .checked_sub(budget.retained_bytes)
        .ok_or_else(exhausted)
}
pub(crate) fn encode_retained<T: Serialize + ?Sized>(
    artifact: &T,
    budget: &mut Budget,
    limits: PrimaryMapLimits,
    stop: &AtomicBool,
) -> ContextResult<Box<[u8]>> {
    let before = budget.retained_bytes;
    budget.retain(artifact, stop)?;
    let reserved = budget
        .retained_bytes
        .checked_sub(before)
        .ok_or_else(exhausted)?;
    // This allowance is the exact reservation just committed on the same ledger.
    let bytes = canonical(artifact, reserved, stop)?;
    checkpoint(stop)?;
    // The caller's whole-envelope allowance is still the continuing remainder.
    remaining(budget, limits)?;
    Ok(bytes.into_boxed_slice())
}
fn mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::UniverseMismatch)
}
fn reference_error(error: ReferenceViewError) -> ContextError {
    match error {
        ReferenceViewError::Cancelled => ContextError::new(ContextErrorCode::Cancelled),
        ReferenceViewError::LimitExceeded => exhausted(),
        _ => mismatch(),
    }
}
fn project_error(error: ProjectError) -> ContextError {
    ContextError::new(match error.code() {
        ProjectErrorCode::AnalysisCancelled => ContextErrorCode::Cancelled,
        ProjectErrorCode::SourceBudgetExceeded => {
            ContextErrorCode::MinimumRequiredContentExceedsBudget
        }
        _ => ContextErrorCode::NativeOwnerRefused,
    })
}
