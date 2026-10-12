#![forbid(unsafe_code)]

//! Bounded primary navigation over exact acquired native owners.
//! No source acquisition, parser, storage, or runtime authority.
mod budget;
mod combined;
mod error;
mod map;
mod pack;
mod platform;
mod profile;
mod reference;
mod render;
mod skeleton;
mod universe;

pub use budget::PrimaryMapLimits;
pub use combined::{
    COMBINED_NATIVE_ONE_HOP_PROFILE, COMBINED_NATIVE_ONE_HOP_PROFILE_BYTES, CombinedContextInspect,
    CombinedContextMap, build_combined_context_inspect, build_combined_context_map,
    build_combined_reference_facts_inspect, build_combined_state_detail_inspect,
    build_combined_state_detail_inspect_with_level,
};
pub use error::{ContextError, ContextErrorCode, ContextResult};
pub use map::{
    ContextExpansionPlan, ContextExpansionPreparation, ContextExpansionStep,
    PRIMARY_FRONTIER_EXPANSION_PROFILE, PRIMARY_PROGRESSIVE_EXPANSION_PROFILE,
    PRIMARY_PROJECT_MAP_SCHEMA, PROJECT_MAP_SCHEMA, PrimaryMapForComposition, ProjectMap,
    ProjectMapSelection, ProjectMapState, ProjectMapView, ProjectNavigationRequest,
    build_primary_map_for_composition, build_project_map, expand_context_frontier,
    open_project_map_view, plan_context_expansion, plan_context_progression,
    plan_next_context_expansion, prepare_context_expansion, prepare_detail_map,
    prepare_primary_project_map, prepare_project_map, primary_detail_query,
    primary_navigation_query, primary_project_navigation_queries,
};
pub use pack::{
    COMBINED_CONTEXT_JSON_RENDERER_PROFILE, COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE,
    COMBINED_CONTEXT_PACK_PROFILE, COMBINED_CONTEXT_PROFILES_BYTES, CombinedContextSemanticPack,
    ContextOperationBudgetUsage, ContextSemanticPack, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE, PRIMARY_CONTEXT_PACK_PROFILE,
    PRIMARY_CONTEXT_PROFILES_BYTES, PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE, PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE,
    PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE,
    PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE, build_combined_context_semantic_pack,
    build_context_semantic_pack, build_context_semantic_pack_from_expansion,
    build_context_semantic_pack_with_l1_profile, build_context_semantic_pack_with_profile,
    build_context_semantic_pack_with_reference_facts, validate_combined_context_semantic_pack,
    validate_context_semantic_pack,
};
pub use platform::{
    PLATFORM_NATIVE_ONE_HOP_PROFILE, PLATFORM_NATIVE_ONE_HOP_PROFILE_BYTES, PlatformContextInspect,
    PlatformContextInspectView, PlatformContextMap, PlatformContextUniverse,
    bind_platform_context_universe, build_platform_context_inspect, build_platform_context_map,
};
pub use profile::{
    PRIMARY_MAP_PROFILE, PRIMARY_MAP_PROFILE_BYTES, PRIMARY_PROJECT_MAP_PROFILE,
    PRIMARY_PROJECT_MAP_PROFILE_BYTES,
};
pub use reference::{ReferenceOriginInput, ReferenceOriginPreparation};
pub use render::{
    ContextRenderTrace, RenderSelection, RenderedContextArtifact,
    render_combined_context_pack_json, render_combined_context_pack_markdown,
    render_context_pack_json, render_context_pack_markdown,
    validate_rendered_combined_context_artifact, validate_rendered_context_artifact,
};
pub use skeleton::{
    L0Scope, L0Skeleton, L1Skeleton, PRIMARY_L0_PROFILE, PRIMARY_L1_PROFILE,
    PRIMARY_REFERENCE_FACTS_L1_PROFILE, PRIMARY_STATE_DETAIL_L1_PROFILE,
    PRIMARY_STATE_ROOT_L0_PROFILE, PrimaryL0Profile, PrimaryL1Profile, PrimarySkeletons,
    SkeletonLevel, build_l0_skeleton, build_l0_skeleton_with_profile, build_l1_skeleton,
    build_primary_l1_skeletons_with_profile, build_primary_reference_skeletons,
    build_primary_skeletons, build_primary_skeletons_with_profile,
};
pub use universe::{ContextUniverseSet, bind_context_universe_set};
