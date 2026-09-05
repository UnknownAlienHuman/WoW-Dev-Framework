//! Deterministic reference-data boundary for the WoW development framework.

mod deterministic_view;

pub use deterministic_view::{
    CoverageStatus, LookupResult, LookupUnknownReason, REFERENCE_VIEW_SCHEMA, ReferenceConflict,
    ReferencePartition, ReferenceRecord, ReferenceRecordKind, ReferenceView, ReferenceViewError,
    ReferenceViewResult, RestrictionFacet, RestrictionState,
};

pub mod generated_api;
pub mod ui_topology;
pub mod wire_json;
