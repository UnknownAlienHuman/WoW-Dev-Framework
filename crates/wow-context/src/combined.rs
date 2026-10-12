//! Selected composition of separate genuine Primary/platform map artifacts.
//! One continuing construction ledger; no cross-universe relation inference.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::ProfileIdentity;
use wow_graph::{GraphPartitionRead, GraphQueryState};
use wow_project::skeleton_input::{SkeletonInputPage, SkeletonInputView};
use wow_reference::ReferenceView;

use crate::budget::{Budget, canonical, checkpoint, exhausted, identity};
use crate::platform::{
    PlatformContextInspectView, PlatformMapArtifact, PlatformRootCallableFacet, PlatformRootFacet,
    PlatformRootXmlHandlerFacet, bind_platform_context_universe_with_budget,
    build_platform_context_inspect_on_budget, build_platform_root_callable_facet_on_budget,
    build_platform_root_xml_handler_facet_on_budget, encode_retained,
};
use crate::skeleton::{
    build_primary_reference_facts_on_budget, build_primary_skeletons_on_budget,
    build_primary_state_detail_on_budget,
};
use crate::{
    ContextError, ContextErrorCode, ContextResult, ContextUniverseSet, L0Scope, PrimaryL0Profile,
    PrimaryMapForComposition, PrimaryMapLimits, PrimarySkeletons, ProjectMap, ProjectMapState,
    SkeletonLevel, build_platform_context_map, open_project_map_view,
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
    /// Borrow the original map and its scoped origin-preparation ledger together.
    pub fn reference_origin_preparation(
        &mut self,
        stop: &AtomicBool,
    ) -> ContextResult<(&ProjectMap<'p>, crate::ReferenceOriginPreparation<'_>)> {
        let map_node_count = open_project_map_view(&self.artifact.core.primary, stop)?
            .nodes()
            .len();
        Ok((
            &self.artifact.core.primary,
            crate::ReferenceOriginPreparation {
                budget: &mut self.budget,
                map_node_count,
            },
        ))
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

#[derive(Serialize)]
struct CombinedInspectCore<'m, 'p, 'f> {
    map: &'m CombinedMapArtifact<'p, 'f>,
    primary_skeletons: PrimarySkeletons<'m, 'p>,
    platform_inspect: PlatformContextInspectView<'m, 'f>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_root_callable: Option<PlatformRootCallableFacet<'m, 'f>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    platform_root_xml_handler: Option<PlatformRootXmlHandlerFacet<'m, 'f>>,
    state: ProjectMapState,
    absence_authoritative: bool,
    cross_universe_relations: &'static str,
}
#[derive(Serialize)]
pub(crate) struct CombinedInspectArtifact<'m, 'p, 'f> {
    family: &'static str,
    artifact_kind: &'static str,
    inspect_id: Box<str>,
    core: CombinedInspectCore<'m, 'p, 'f>,
}

impl<'m, 'p, 'f> CombinedInspectArtifact<'m, 'p, 'f> {
    pub(crate) fn inspect_id(&self) -> &str {
        &self.inspect_id
    }
    pub(crate) fn combined_map_id(&self) -> &'m str {
        &self.core.map.combined_map_id
    }
    pub(crate) fn primary_map(&self) -> &'m ProjectMap<'p> {
        &self.core.map.core.primary
    }
    pub(crate) fn platform_map_id(&self) -> &'m str {
        self.core.map.core.platform.map_id()
    }
    pub(crate) fn platform_page(&self) -> &'m SkeletonInputPage<'f> {
        self.core.map.core.platform.page()
    }
    pub(crate) fn platform_inspect_id(&self) -> &str {
        self.core.platform_inspect.inspect_id()
    }
    pub(crate) fn primary_skeletons(&self) -> &PrimarySkeletons<'m, 'p> {
        &self.core.primary_skeletons
    }
    pub(crate) fn limits(&self) -> PrimaryMapLimits {
        self.core.map.core.limits
    }
    pub(crate) fn state(&self) -> ProjectMapState {
        self.core.state
    }
    pub(crate) fn validate_for_pack(
        &self,
        bytes: &[u8],
        budget: &mut Budget,
        stop: &AtomicBool,
    ) -> ContextResult<()> {
        budget.work(1, stop)?;
        if self.family != COMBINED_NATIVE_ONE_HOP_PROFILE
            || self.artifact_kind != "inspect"
            || self.core.absence_authoritative
            || self.core.cross_universe_relations != "not_evaluated"
            || self.core.platform_inspect.map_id() != self.platform_map_id()
        {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
        let id = identity(
            COMBINED_NATIVE_ONE_HOP_PROFILE,
            "combined-context-inspect:sha256:",
            &("inspect", &self.core),
            self.limits().max_metadata_bytes,
            stop,
        )?;
        budget.retain(self, stop)?;
        if id != self.inspect_id
            || canonical(self, self.limits().max_metadata_bytes, stop)?.as_slice() != bytes
        {
            return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
        }
        Ok(())
    }
}

/// Original component inspections borrowing disjoint map fields and its ledger.
/// No self-reference or decoded receipt can construct this sealed result.
#[derive(Serialize)]
pub struct CombinedContextInspect<'m, 'p, 'f> {
    #[serde(flatten)]
    artifact: CombinedInspectArtifact<'m, 'p, 'f>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: &'m mut Budget,
}
impl<'m, 'p, 'f> CombinedContextInspect<'m, 'p, 'f> {
    pub(crate) fn into_pack_parts(
        self,
    ) -> (
        CombinedInspectArtifact<'m, 'p, 'f>,
        Box<[u8]>,
        &'m mut Budget,
    ) {
        (self.artifact, self.canonical_bytes, self.budget)
    }

    #[must_use]
    pub fn inspect_id(&self) -> &str {
        &self.artifact.inspect_id
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
        self.artifact.core.map.core.limits.max_metadata_bytes - self.budget.retained_bytes
    }
    /// Continuing conservative reservations, not measured CPU work.
    #[must_use]
    pub fn context_work_reserved(&self) -> u64 {
        self.budget.work
    }
    #[must_use]
    pub fn primary_skeletons(&self) -> &PrimarySkeletons<'m, 'p> {
        &self.artifact.core.primary_skeletons
    }
    #[must_use]
    pub fn platform_inspect(&self) -> &PlatformContextInspectView<'m, 'f> {
        &self.artifact.core.platform_inspect
    }
    #[must_use]
    pub fn combined_map_id(&self) -> &str {
        &self.artifact.core.map.combined_map_id
    }
}

/// Build default Primary L0/L1/Both and the original platform inspection using
/// the actual combined construction ledger. L1/Both require a prepared detail
/// origin; they never widen or reconstruct the original map here.
pub fn build_combined_context_inspect<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    root: &L0Scope,
    level: SkeletonLevel,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    build_combined_context_inspect_with_platform_root_facet(
        map,
        universe,
        graph,
        (root, level),
        None,
        stop,
    )
}

/// Default Primary inspection with an optional native selected platform-root
/// facet. Both use the held map and continuing construction ledger.
pub fn build_combined_context_inspect_with_platform_root_facet<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    selection: (&L0Scope, SkeletonLevel),
    platform_root_facet: Option<PlatformRootFacet>,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    let (root, level) = selection;
    build_combined_inspect(
        map,
        universe,
        graph,
        CombinedInspection::Default {
            root,
            level,
            platform_root_facet,
        },
        stop,
    )
}

/// Explicit original StateRoot L0 on the continuing combined construction ledger.
/// The existing StateRootV1 owner validates the exact root and its native custody.
pub fn build_combined_state_root_l0_inspect<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    root: &L0Scope,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    build_combined_inspect(
        map,
        universe,
        graph,
        CombinedInspection::StateRootL0 { root },
        stop,
    )
}

/// Explicit Primary StateDetail L1 with the unchanged original platform inspect.
/// Both components continue the same held combined construction ledger.
pub fn build_combined_state_detail_inspect<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    root: &L0Scope,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    build_combined_state_detail_inspect_with_level(
        map,
        universe,
        graph,
        root,
        SkeletonLevel::L1,
        stop,
    )
}

/// Explicit Primary StateDetail L1 or Both, retaining the original components.
pub fn build_combined_state_detail_inspect_with_level<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    root: &L0Scope,
    level: SkeletonLevel,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    build_combined_inspect(
        map,
        universe,
        graph,
        CombinedInspection::StateDetail { root, level },
        stop,
    )
}

/// Explicit Primary ReferenceFacts L1 with the original platform inspection.
/// The reference recipe consumes the same held combined construction ledger.
pub fn build_combined_reference_facts_inspect<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    root: &L0Scope,
    origins: &[crate::reference::ReferenceOriginInput<'p>],
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    build_combined_inspect(
        map,
        universe,
        graph,
        CombinedInspection::ReferenceFacts { root, origins },
        stop,
    )
}

enum CombinedInspection<'r, 'p> {
    Default {
        root: &'r L0Scope,
        level: SkeletonLevel,
        platform_root_facet: Option<PlatformRootFacet>,
    },
    StateDetail {
        root: &'r L0Scope,
        level: SkeletonLevel,
    },
    StateRootL0 {
        root: &'r L0Scope,
    },
    ReferenceFacts {
        root: &'r L0Scope,
        origins: &'r [crate::reference::ReferenceOriginInput<'p>],
    },
}

fn build_combined_inspect<'m, 'p, 'f>(
    map: &'m mut CombinedContextMap<'p, 'f>,
    universe: &'m ContextUniverseSet<'p>,
    graph: &GraphPartitionRead<'p>,
    selection: CombinedInspection<'_, 'p>,
    stop: &AtomicBool,
) -> ContextResult<CombinedContextInspect<'m, 'p, 'f>> {
    checkpoint(stop)?;
    let CombinedContextMap {
        artifact,
        platform_map_canonical_bytes,
        budget,
        ..
    } = map;
    let map: &'m CombinedMapArtifact<'p, 'f> = artifact;
    let platform_map_canonical_bytes: &'m [u8] = platform_map_canonical_bytes;
    if map.core.limits != universe.limits {
        return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
    }
    let platform_root_facet = match &selection {
        CombinedInspection::Default {
            platform_root_facet,
            ..
        } => *platform_root_facet,
        _ => None,
    };
    let primary_skeletons = match selection {
        CombinedInspection::Default { root, level, .. } => build_primary_skeletons_on_budget(
            universe,
            &map.core.primary,
            graph,
            (root, level),
            PrimaryL0Profile::ContainersV1,
            budget,
            stop,
        )?,
        CombinedInspection::StateDetail { root, level } => build_primary_state_detail_on_budget(
            universe,
            &map.core.primary,
            graph,
            (root, level),
            budget,
            stop,
        )?,
        CombinedInspection::StateRootL0 { root } => build_primary_skeletons_on_budget(
            universe,
            &map.core.primary,
            graph,
            (root, SkeletonLevel::L0),
            PrimaryL0Profile::StateRootV1,
            budget,
            stop,
        )?,
        CombinedInspection::ReferenceFacts { root, origins } => {
            build_primary_reference_facts_on_budget(
                universe,
                &map.core.primary,
                graph,
                root,
                origins,
                budget,
                stop,
            )?
        }
    };
    let platform_inspect = build_platform_context_inspect_on_budget(
        &map.core.platform,
        platform_map_canonical_bytes,
        budget,
        stop,
    )?;
    let platform_root_callable = match platform_root_facet {
        Some(PlatformRootFacet::CallableMetadata) => Some(
            build_platform_root_callable_facet_on_budget(&map.core.platform, budget, stop)?,
        ),
        None => None,
        Some(PlatformRootFacet::XmlHandlerMetadata) => None,
    };
    let platform_root_xml_handler = match platform_root_facet {
        Some(PlatformRootFacet::XmlHandlerMetadata) => Some(
            build_platform_root_xml_handler_facet_on_budget(&map.core.platform, budget, stop)?,
        ),
        _ => None,
    };
    let state = if map.core.state == ProjectMapState::Truncated
        || primary_skeletons.state() == ProjectMapState::Truncated
        || platform_inspect.state() == GraphQueryState::Truncated
    {
        ProjectMapState::Truncated
    } else {
        ProjectMapState::Partial
    };
    budget.work(1, stop)?;
    let core = CombinedInspectCore {
        map,
        primary_skeletons,
        platform_inspect,
        platform_root_callable,
        platform_root_xml_handler,
        state,
        absence_authoritative: false,
        cross_universe_relations: "not_evaluated",
    };
    budget.retain(&core, stop)?;
    let inspect_id = identity(
        COMBINED_NATIVE_ONE_HOP_PROFILE,
        "combined-context-inspect:sha256:",
        &("inspect", &core),
        map.core
            .limits
            .max_metadata_bytes
            .checked_sub(budget.retained_bytes)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let artifact = CombinedInspectArtifact {
        family: COMBINED_NATIVE_ONE_HOP_PROFILE,
        artifact_kind: "inspect",
        inspect_id,
        core,
    };
    let canonical_bytes = encode_retained(&artifact, budget, map.core.limits, stop)?;
    checkpoint(stop)?;
    Ok(CombinedContextInspect {
        artifact,
        canonical_bytes,
        budget,
    })
}
