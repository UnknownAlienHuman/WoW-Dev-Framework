//! Deterministic reference-data boundary for the WoW development framework.

mod deterministic_view;
mod generated_api_limits;
mod ui_topology_limits;

pub use deterministic_view::{
    CoverageStatus, LookupResult, LookupUnknownReason, REFERENCE_VIEW_SCHEMA, ReferenceConflict,
    ReferencePartition, ReferenceRecord, ReferenceRecordKind, ReferenceView, ReferenceViewError,
    ReferenceViewResult, RestrictionFacet, RestrictionState,
};

pub mod generated_api;
pub mod ui_topology;
pub mod wire_json;

pub mod compatibility_aliases;
pub mod native;
pub mod native_constants;
pub mod native_model;

pub mod native_corrections;

pub mod native_aliases;
pub mod persistent;
pub mod publication;

pub mod native_view;
