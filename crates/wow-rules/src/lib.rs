#![forbid(unsafe_code)]

//! Deterministic diagnostics over one immutable project and reference view.
//! `wow.api.exists@1` and `wow.secret.local_operation@1` remain the only active providers;
//! fixture and explicitly admitted native production policies are separate.

mod context;
mod descriptor;
mod engine;
mod error;
mod identity;
mod output;

pub use context::{
    RuleExecutionBudget, RuleExecutionContext, RuleFixturePolicy, RuleProductionPolicy, RuleScope,
};
pub use descriptor::{
    API_EXISTS_RULE, FIXTURE_POLICY_ID, FIXTURE_PROFILE_ID, RULE_VERSION, RuleDescriptor,
    RuleRegistry, SECRET_LOCAL_RULE,
};
pub use engine::{execute, execute_e0};
pub use error::{RuleError, RuleErrorCode, RuleResult};
pub use output::{
    CleanEvaluationRecord, RuleBlockerKind, RuleBudgetUsage, RuleCleanClaimKind,
    RuleEvaluationOutcome, RuleEvaluationRecord, RuleEvaluationStatus, RuleExecutionReport,
    RuleFailure, RuleFindingSet, RuleGuardClassification, RuleNotEvaluatedDetail,
    RuleReferenceLookupRecord, RuleReferenceOutcome,
};
