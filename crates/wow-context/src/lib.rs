#![forbid(unsafe_code)]

//! Bounded primary navigation over exact acquired native owners.
//! No source acquisition, parser, storage, or runtime authority.
mod budget;
mod error;
mod map;
mod pack;
mod profile;
mod render;
mod skeleton;
mod universe;

pub use budget::PrimaryMapLimits;
pub use error::{ContextError, ContextErrorCode, ContextResult};
pub use map::{
    ContextExpansionPlan, ContextExpansionPreparation, ContextExpansionStep,
    PRIMARY_FRONTIER_EXPANSION_PROFILE, PRIMARY_PROJECT_MAP_SCHEMA, PROJECT_MAP_SCHEMA, ProjectMap,
    ProjectMapSelection, ProjectMapState, ProjectMapView, ProjectNavigationRequest,
    build_project_map, expand_context_frontier, open_project_map_view, plan_context_expansion,
    prepare_context_expansion, prepare_detail_map, prepare_primary_project_map,
    prepare_project_map, primary_detail_query, primary_navigation_query,
    primary_project_navigation_queries,
};
pub use pack::{
    ContextOperationBudgetUsage, ContextSemanticPack, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE, PRIMARY_CONTEXT_PACK_PROFILE,
    PRIMARY_CONTEXT_PROFILES_BYTES, PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE, PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE,
    build_context_semantic_pack, build_context_semantic_pack_from_expansion,
    build_context_semantic_pack_with_l1_profile, build_context_semantic_pack_with_profile,
    validate_context_semantic_pack,
};
pub use profile::{
    PRIMARY_MAP_PROFILE, PRIMARY_MAP_PROFILE_BYTES, PRIMARY_PROJECT_MAP_PROFILE,
    PRIMARY_PROJECT_MAP_PROFILE_BYTES,
};
pub use render::{
    ContextRenderTrace, RenderSelection, RenderedContextArtifact, render_context_pack_json,
    render_context_pack_markdown, validate_rendered_context_artifact,
};
pub use skeleton::{
    L0Scope, L0Skeleton, L1Skeleton, PRIMARY_L0_PROFILE, PRIMARY_L1_PROFILE,
    PRIMARY_STATE_DETAIL_L1_PROFILE, PRIMARY_STATE_ROOT_L0_PROFILE, PrimaryL0Profile,
    PrimaryL1Profile, PrimarySkeletons, SkeletonLevel, build_l0_skeleton,
    build_l0_skeleton_with_profile, build_l1_skeleton, build_primary_l1_skeletons_with_profile,
    build_primary_skeletons, build_primary_skeletons_with_profile,
};
pub use universe::{ContextUniverseSet, bind_context_universe_set};
