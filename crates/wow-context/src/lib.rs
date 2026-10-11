#![forbid(unsafe_code)]

//! Bounded primary navigation over exact acquired native owners.
//! No source acquisition, parser, storage, renderer, or runtime authority.
mod budget;
mod error;
mod map;
mod profile;
mod universe;

pub use budget::PrimaryMapLimits;
pub use error::{ContextError, ContextErrorCode, ContextResult};
pub use map::{
    PROJECT_MAP_SCHEMA, ProjectMap, ProjectMapSelection, ProjectMapState, ProjectMapView,
    build_project_map, open_project_map_view, prepare_project_map, primary_navigation_query,
};
pub use profile::{PRIMARY_MAP_PROFILE, PRIMARY_MAP_PROFILE_BYTES};
pub use universe::{ContextUniverseSet, bind_context_universe_set};
