use serde::Serialize;

pub(crate) const REPORT_SCHEMA: &str = "wow-rules/e0-e/1";
pub(crate) const MISSING_API_RULE: &str = "wow.api.missing.complete-reference/1";
pub(crate) const SECRET_OPERATION_RULE: &str = "wow.secret.concat.require-dominating-access-guard/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    Complete,
    Partial,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerResolution {
    Resolved,
    Unresolved,
    Possible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiPresence {
    Present,
    Absent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueRestriction {
    Secret,
    NonSecret,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Concatenate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleDecision {
    Pass,
    Diagnostic,
    NotEvaluated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleDiagnosticCode {
    MissingApi,
    SecretValueUsedWithoutDominatingAccessGuard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleReportStatus {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ByteSpan {
    start: u64,
    end: u64,
}

impl ByteSpan {
    #[must_use]
    pub fn new(start: u64, end: u64) -> Option<Self> {
        (start <= end).then_some(Self { start, end })
    }

    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u64 {
        self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceLocation {
    path: Box<str>,
    content_sha256: Box<str>,
    span: ByteSpan,
}

impl SourceLocation {
    #[must_use]
    pub fn new(
        path: impl Into<Box<str>>,
        content_sha256: impl Into<Box<str>>,
        span: ByteSpan,
    ) -> Self {
        Self {
            path: path.into(),
            content_sha256: content_sha256.into(),
            span,
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    #[must_use]
    pub const fn span(&self) -> ByteSpan {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiCallObservation {
    pub fact_id: Box<str>,
    pub source: SourceLocation,
    pub receiver: Box<str>,
    pub member: Box<str>,
    pub resolution: AnalyzerResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BindingObservation {
    pub fact_id: Box<str>,
    pub source: SourceLocation,
    pub name: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initializer_call_fact_id: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationObservation {
    pub fact_id: Box<str>,
    pub source: SourceLocation,
    pub kind: OperationKind,
    pub operand_binding_fact_ids: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccessGuardObservation {
    pub fact_id: Box<str>,
    pub source: SourceLocation,
    pub binding_fact_id: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DominanceObservation {
    pub guard_fact_id: Box<str>,
    pub operation_fact_id: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceApiEvidence {
    pub evidence_id: Box<str>,
    pub target_profile: Box<str>,
    pub receiver: Box<str>,
    pub member: Box<str>,
    pub presence_coverage: CoverageState,
    pub presence: ApiPresence,
    pub restriction_coverage: CoverageState,
    pub restriction: ValueRestriction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationInput {
    pub project_snapshot_id: Box<str>,
    pub project_generation: Box<str>,
    pub reference_view_id: Box<str>,
    pub target_profile: Box<str>,
    pub calls: Vec<ApiCallObservation>,
    pub bindings: Vec<BindingObservation>,
    pub operations: Vec<OperationObservation>,
    pub guards: Vec<AccessGuardObservation>,
    pub dominance: Vec<DominanceObservation>,
    pub reference: Vec<ReferenceApiEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleOutcome {
    pub rule_id: &'static str,
    pub subject_fact_id: Box<str>,
    pub decision: RuleDecision,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic_id: Option<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleDiagnostic {
    pub diagnostic_id: Box<str>,
    pub code: RuleDiagnosticCode,
    pub rule_id: &'static str,
    pub source: SourceLocation,
    pub related_fact_ids: Vec<Box<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleReport {
    pub schema: &'static str,
    pub report_id: Box<str>,
    pub input_id: Box<str>,
    pub project_snapshot_id: Box<str>,
    pub project_generation: Box<str>,
    pub reference_view_id: Box<str>,
    pub target_profile: Box<str>,
    pub status: RuleReportStatus,
    pub outcomes: Vec<RuleOutcome>,
    pub diagnostics: Vec<RuleDiagnostic>,
    pub limitations: Vec<&'static str>,
}

impl RuleReport {
    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub const fn status(&self) -> RuleReportStatus {
        self.status
    }

    #[must_use]
    pub fn outcomes(&self) -> &[RuleOutcome] {
        &self.outcomes
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[RuleDiagnostic] {
        &self.diagnostics
    }
}
