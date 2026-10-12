//! Selected composition of separate genuine Primary/platform map artifacts.
//! One continuing construction ledger; no cross-universe relation inference.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::ProfileIdentity;
use wow_graph::GraphQueryState;
use wow_project::skeleton_input::{SkeletonInputPage, SkeletonInputView};
use wow_reference::ReferenceView;

use crate::budget::{Budget, canonical, checkpoint, exhausted, identity};
use crate::platform::{
    PlatformMapArtifact, bind_platform_context_universe_with_budget, encode_retained,
};
use crate::{
    ContextError, ContextErrorCode, ContextResult, PrimaryMapForComposition, PrimaryMapLimits,
    ProjectMap, ProjectMapState, build_platform_context_map, open_project_map_view,
};

/// Explicit new semantics; original component schemas and IDs stay separate.
pub const COMBINED_NATIVE_ONE_HOP_PROFILE: &str = "wow-context/combined-native-one-hop/1";
pub const COMBINED_NATIVE_ONE_HOP_PROFILE_BYTES: &[u8] = br#"{"budget":"continuing_primary_then_platform_construction","cross_universe_relations":"not_evaluated","maps":"distinct_original_native_artifacts","profile":"wow-context/combined-native-one-hop/1","reference":"exact_shared_profile_generation_schema_and_validated_full_content","runtime":"not_evaluated","support":"unchanged_native_component_authority","tokens":"not_evaluated"}"#;

#[derive(Serialize)]
struct CombinedProfile {
    budget: &'static str,
    cross_universe_relations: &'static str,
    maps: &'static str,
    profile: &'static str,
    reference: &'static str,
    runtime: &'static str,
    support: &'static str,
    tokens: &'static str,
}
const PROFILE: CombinedProfile = CombinedProfile {
    budget: "continuing_primary_then_platform_construction",
    cross_universe_relations: "not_evaluated",
    maps: "distinct_original_native_artifacts",
    profile: COMBINED_NATIVE_ONE_HOP_PROFILE,
    reference: "exact_shared_profile_generation_schema_and_validated_full_content",
    runtime: "not_evaluated",
    support: "unchanged_native_component_authority",
    tokens: "not_evaluated",
};

#[derive(Serialize)]
struct CombinedMapCore<'p, 'f> {
    profile: &'static CombinedProfile,
    primary: ProjectMap<'p>,
    platform: PlatformMapArtifact<'f>,
    limits: PrimaryMapLimits,
    input_bytes_reserved: u64,
    state: ProjectMapState,
    absence_authoritative: bool,
    cross_universe_relations: &'static str,
}
#[derive(Serialize)]
struct CombinedMapArtifact<'p, 'f> {
    family: &'static str,
    artifact_kind: &'static str,
    combined_map_id: Box<str>,
    core: CombinedMapCore<'p, 'f>,
}

/// Serialize-only original artifacts borrowing their independently held owners.
/// Decoded IDs/pages cannot create this sealed construction result or ledger.
#[derive(Serialize)]
pub struct CombinedContextMap<'p, 'f> {
    #[serde(flatten)]
    artifact: CombinedMapArtifact<'p, 'f>,
    #[serde(skip)]
    platform_map_canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl<'p, 'f> CombinedContextMap<'p, 'f> {
    #[must_use]
    pub fn combined_map_id(&self) -> &str {
        &self.artifact.combined_map_id
    }
    #[must_use]
    pub fn primary_map(&self) -> &ProjectMap<'p> {
        &self.artifact.core.primary
    }
    #[must_use]
    pub fn platform_map_id(&self) -> &str {
        self.artifact.core.platform.map_id()
    }
    #[must_use]
    pub fn platform_map_canonical_bytes(&self) -> &[u8] {
        &self.platform_map_canonical_bytes
    }
    #[must_use]
    pub fn platform_page(&self) -> &SkeletonInputPage<'f> {
        self.artifact.core.platform.page()
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub const fn state(&self) -> ProjectMapState {
        self.artifact.core.state
    }
    #[must_use]
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.artifact.core.limits.max_metadata_bytes - self.budget.retained_bytes
    }
    /// Actual conservative construction reservations; not measured CPU work.
    #[must_use]
    pub fn context_work_reserved(&self) -> u64 {
        self.budget.work
    }
}

/// Consume the real Primary ledger before platform binding and its sole read.
/// The selected aggregate limits were already supplied to the Primary universe;
/// they cannot be reset or substituted after Primary construction.
pub fn build_combined_context_map<'p, 'f>(
    primary: PrimaryMapForComposition<'p>,
    platform: &'f SkeletonInputView<'f>,
    reference: &'f ReferenceView,
    reference_profile: &'f ProfileIdentity,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextMap<'p, 'f>> {
    checkpoint(stop)?;
    let (primary, universe, budget, primary_input_bytes) = primary.into_parts();
    if !open_project_map_view(&primary, stop)?.bound_to(universe)
        || !universe.matches_reference(reference, reference_profile)
    {
        return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
    }
    let limits = universe.limits;
    let platform = bind_platform_context_universe_with_budget(
        platform,
        reference,
        reference_profile,
        limits,
        budget,
        primary_input_bytes,
        stop,
    )?;
    // The helper validates full Reference content and preserves all platform
    // admission/scan guards. No second native read or Primary-kind substitution.
    let platform = build_platform_context_map(platform, stop)?;
    let (platform, platform_map_canonical_bytes, mut budget) = platform.into_parts();
    let input_bytes_reserved = primary_input_bytes
        .checked_add(platform.input_bytes_reserved())
        .filter(|bytes| *bytes <= limits.max_input_bytes)
        .ok_or_else(exhausted)?;
    budget.work(1, stop)?;
    budget.retain(&PROFILE, stop)?;
    if canonical(
        &PROFILE,
        u64::try_from(COMBINED_NATIVE_ONE_HOP_PROFILE_BYTES.len()).map_err(|_| exhausted())?,
        stop,
    )?
    .as_slice()
        != COMBINED_NATIVE_ONE_HOP_PROFILE_BYTES
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    let state = if primary.state() == ProjectMapState::Truncated
        || platform.page().state() == GraphQueryState::Truncated
    {
        ProjectMapState::Truncated
    } else {
        ProjectMapState::Partial
    };
    let core = CombinedMapCore {
        profile: &PROFILE,
        primary,
        platform,
        limits,
        input_bytes_reserved,
        state,
        absence_authoritative: false,
        cross_universe_relations: "not_evaluated",
    };
    budget.retain(&core, stop)?;
    let combined_map_id = identity(
        COMBINED_NATIVE_ONE_HOP_PROFILE,
        "combined-context-map:sha256:",
        &core,
        limits
            .max_metadata_bytes
            .checked_sub(budget.retained_bytes)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let artifact = CombinedMapArtifact {
        family: COMBINED_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "map",
        combined_map_id,
        core,
    };
    let canonical_bytes = encode_retained(&artifact, &mut budget, limits, stop)?;
    checkpoint(stop)?;
    Ok(CombinedContextMap {
        artifact,
        platform_map_canonical_bytes,
        canonical_bytes,
        budget,
    })
}
