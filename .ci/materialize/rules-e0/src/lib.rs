#![forbid(unsafe_code)]

//! Closed E0-E rule evaluation over exact project and reference observations.
//!
//! This crate never discovers files, executes Lua, queries a live client, or treats
//! analyzer resolution as WoW authority. Callers must provide one immutable project
//! snapshot identity and one independently identified reference view.

mod engine;
mod error;
mod identity;
mod model;

pub use engine::evaluate;
pub use error::{RuleError, RuleErrorCode, RuleResult};
pub use model::{
    AccessGuardObservation, AnalyzerResolution, ApiCallObservation, ApiPresence, BindingObservation,
    ByteSpan, CoverageState, DominanceObservation, OperationKind, OperationObservation,
    ReferenceApiEvidence, RuleDecision, RuleDiagnostic, RuleDiagnosticCode, RuleEvaluationInput,
    RuleOutcome, RuleReport, RuleReportStatus, SourceLocation, ValueRestriction,
};
