#![forbid(unsafe_code)]

//! Deterministic recognition over explicit structured owner observations.
//!
//! This crate does not parse source and does not infer semantics from names or prose.
//! It applies a closed recognizer registry, preserves input provenance, and emits
//! bounded assertions whose confidence never exceeds either the observation or rule ceiling.

mod emmy;
mod engine;
mod error;
mod facts;
mod identity;
mod matcher;
mod model;
mod pack;
mod plan;

pub use emmy::{
    EMMY_DIRECT_CALL_ADAPTER_SCHEMA, EmmyDirectCallAdaptation, EmmyDirectCallBinding,
    adapt_emmy_direct_calls,
};
pub use engine::{project_graph_coverage, project_graph_edges, run_recognizers};
pub use error::{RecognizerError, RecognizerErrorCode, RecognizerResult};
pub use facts::{
    RECOGNIZER_FACT_BUNDLE_SCHEMA, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind, RecognizerFactValue,
};
pub use identity::{
    EmmyDirectCallAdapterId, RecognitionAssertionId, RecognitionReportId, RecognizerFactBundleId,
    RecognizerFactId, RecognizerId, RecognizerMatchId, RecognizerOutputPartitionId,
    RecognizerPlanId, RecognizerProposalId, RecognizerRegistryId, RecognizerVersion,
    StructuredObservationId,
};
pub use matcher::{
    RECOGNIZER_OUTPUT_PARTITION_SCHEMA, RecognizerCapturedValue, RecognizerMatch,
    RecognizerOutputPartition, RecognizerProposedAssertion, RecognizerRuleOutcome,
    RecognizerRuleOutcomeState, execute_recognizer_plan,
};
pub use model::{
    ObservationFamily, ObservationOrigin, RecognitionAssertion, RecognitionCoverage,
    RecognitionCoverageState, RecognitionReport, RecognizerDescriptor, RecognizerLimits,
    RecognizerRegistry, StructuredObservation, StructuredObservationInput,
};
pub use pack::{
    CompiledRecognizerPack, MAX_RECOGNIZER_PACK_BYTES, RECOGNIZER_PACK_SCHEMA_VERSION,
    RecognizerCapture, RecognizerCaptureCardinality, RecognizerClause, RecognizerOutput,
    RecognizerOutputConfidence, RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument,
    RecognizerPackLiteral, RecognizerPackRollout, RecognizerPackTrustClass, RecognizerRule,
    parse_recognizer_pack,
};
pub use plan::{
    CompiledRecognizerPlan, CompiledRecognizerRulePlan, RECOGNIZER_PLAN_SCHEMA,
    RecognizerPlanBounds, RecognizerPlanCostClass, RecognizerPlanStep, RecognizerPlanStepKind,
    RecognizerRulePlanBounds, compile_recognizer_plan,
};
