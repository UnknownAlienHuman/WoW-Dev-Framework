#![forbid(unsafe_code)]

//! Deterministic recognition over explicit structured owner observations.
//!
//! This crate does not parse source and does not infer semantics from names or prose.
//! It applies a closed recognizer registry, preserves input provenance, and emits
//! bounded assertions whose confidence never exceeds either the observation or rule ceiling.

mod engine;
mod error;
mod identity;
mod model;

pub use engine::{project_graph_coverage, project_graph_edges, run_recognizers};
pub use error::{RecognizerError, RecognizerErrorCode, RecognizerResult};
pub use identity::{
    RecognitionAssertionId, RecognitionReportId, RecognizerId, RecognizerRegistryId,
    RecognizerVersion, StructuredObservationId,
};
pub use model::{
    ObservationFamily, ObservationOrigin, RecognitionAssertion, RecognitionCoverage,
    RecognitionCoverageState, RecognitionReport, RecognizerDescriptor, RecognizerLimits,
    RecognizerRegistry, StructuredObservation,
};
