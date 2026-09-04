//! Deterministic reference-data boundary for the WoW development framework.

mod deterministic_view;

pub use deterministic_view::{
    CoverageStatus, LookupResult, LookupUnknownReason, REFERENCE_VIEW_SCHEMA, ReferenceConflict,
    ReferencePartition, ReferenceRecord, ReferenceRecordKind, ReferenceView, ReferenceViewError,
    ReferenceViewResult, RestrictionFacet, RestrictionState,
};
