#![forbid(unsafe_code)]

//! Bounded primary navigation over exact acquired native owners.
//! No source acquisition, parser, storage, renderer, or runtime authority.
mod budget;
mod error;
mod map;
mod profile;
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
pub use profile::{
    PRIMARY_MAP_PROFILE, PRIMARY_MAP_PROFILE_BYTES, PRIMARY_PROJECT_MAP_PROFILE,
    PRIMARY_PROJECT_MAP_PROFILE_BYTES,
};
pub use skeleton::{
    L0Scope, L0Skeleton, L1Skeleton, PRIMARY_L0_PROFILE, PRIMARY_L1_PROFILE, PrimarySkeletons,
    SkeletonLevel, build_l0_skeleton, build_l1_skeleton, build_primary_skeletons,
};
pub use universe::{ContextUniverseSet, bind_context_universe_set};
