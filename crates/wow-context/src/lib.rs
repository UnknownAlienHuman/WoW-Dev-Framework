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
    PRIMARY_PROJECT_MAP_SCHEMA, PROJECT_MAP_SCHEMA, ProjectMap, ProjectMapSelection,
    ProjectMapState, ProjectMapView, ProjectNavigationRequest, build_project_map,
    open_project_map_view, prepare_detail_map, prepare_primary_project_map, prepare_project_map,
    primary_detail_query, primary_navigation_query, primary_project_navigation_queries,
};
pub use pack::{
    ContextOperationBudgetUsage, ContextSemanticPack, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE,
    PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE, PRIMARY_CONTEXT_PACK_PROFILE,
    PRIMARY_CONTEXT_PROFILES_BYTES, build_context_semantic_pack, validate_context_semantic_pack,
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
    L0Scope, L0Skeleton, L1Skeleton, PRIMARY_L0_PROFILE, PRIMARY_L1_PROFILE, PrimarySkeletons,
    SkeletonLevel, build_l0_skeleton, build_l1_skeleton, build_primary_skeletons,
};
pub use universe::{ContextUniverseSet, bind_context_universe_set};
