use std::fmt;

use serde::Serialize;

/// Stable E0-E provider/context/output failure vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleErrorCode {
    RuleDescriptorInvalid,
    RuleDescriptorDuplicate,
    RuleRegistryInvalid,
    RuleFixturePolicyInvalid,
    RuleExecutionContextInvalid,
    RuleProfileMismatch,
    RuleReferenceGenerationMismatch,
    RuleProjectGenerationMismatch,
    RuleAnalyzerSnapshotMismatch,
    RuleSourceHandleInvalid,
    RuleRequiredCapabilityUnavailable,
    RuleStaleInputForbidden,
    RuleFactReferenceGraphInvalid,
    RuleLookupOutcomeInvalid,
    ApiExistsScopeInvalid,
    SecretLocalScopeInvalid,
    RuleFindingInputInvalid,
    RuleOutcomeInvalid,
    RuleExecutionBudgetExceeded,
    RuleOutputBudgetExceeded,
    RuleCancelled,
    CanonicalizationFailed,
    CoreConstructionFailed,
}

/// Bounded typed error. Untrusted source text is never retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError {
    code: RuleErrorCode,
    message: Box<str>,
    rule_id: Option<Box<str>>,
    scope_id: Option<Box<str>>,
}

impl RuleError {
    pub(crate) fn new(code: RuleErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
            rule_id: None,
            scope_id: None,
        }
    }

    #[must_use]
    pub(crate) fn with_rule(mut self, rule_id: &str) -> Self {
        self.rule_id = Some(rule_id.into());
        self
    }

    #[must_use]
    pub(crate) fn with_scope(mut self, scope_id: &str) -> Self {
        self.scope_id = Some(scope_id.into());
        self
    }

    #[must_use]
    pub const fn code(&self) -> RuleErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn rule_id(&self) -> Option<&str> {
        self.rule_id.as_deref()
    }

    #[must_use]
    pub fn scope_id(&self) -> Option<&str> {
        self.scope_id.as_deref()
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RuleError {}

pub type RuleResult<T> = Result<T, RuleError>;
