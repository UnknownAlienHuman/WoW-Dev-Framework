use serde::Serialize;
use wow_core::{
    CapabilityId, CoverageId, EvidenceRecord, Finding, GenerationContextId, NotEvaluatedRecord,
    RuleId, SourceHandle, ToolVersion,
};

use crate::identity::canonical_id;
use crate::{RuleError, RuleErrorCode, RuleResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleEvaluationStatus {
    Findings,
    EvaluatedClean,
    NotEvaluated,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleCleanClaimKind {
    ApiExistsForExactUse,
    SecretFixtureOperationGuardedForExactValueAndScope,
    SecretProducerHasNoMatchingFacet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleBlockerKind {
    MissingCapability,
    FailedCapability,
    PartialCoverage,
    ReferencePartitionMissing,
    ReferenceConflict,
    AmbiguousReference,
    UnsupportedFactShape,
    MissingFactRelation,
    MissingRestrictionFacet,
    StaleInput,
    BudgetIncomplete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleGuardClassification {
    Absent,
    DominatingExactValue,
    AfterUse,
    DifferentValue,
    NonDominating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleReferenceOutcome {
    Found,
    AuthoritativeAbsent,
    Conflict,
    PartialCoverage,
    NotEvaluated,
    PartitionMissing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleReferenceLookupRecord {
    lookup_id: Box<str>,
    partition_id: Box<str>,
    entity_key: Box<str>,
    reference_generation: Box<str>,
    reference_view_digest: Box<str>,
    outcome: RuleReferenceOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    record_digest: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflict_digest: Option<Box<str>>,
}

impl RuleReferenceLookupRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        partition_id: &str,
        entity_key: &str,
        reference_generation: &str,
        reference_view_digest: &str,
        outcome: RuleReferenceOutcome,
        record_digest: Option<String>,
        conflict_digest: Option<String>,
    ) -> RuleResult<Self> {
        #[derive(Serialize)]
        struct Identity<'a> {
            partition_id: &'a str,
            entity_key: &'a str,
            reference_generation: &'a str,
            reference_view_digest: &'a str,
            outcome: RuleReferenceOutcome,
            #[serde(skip_serializing_if = "Option::is_none")]
            record_digest: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            conflict_digest: Option<&'a str>,
        }
        let identity = Identity {
            partition_id,
            entity_key,
            reference_generation,
            reference_view_digest,
            outcome,
            record_digest: record_digest.as_deref(),
            conflict_digest: conflict_digest.as_deref(),
        };
        Ok(Self {
            lookup_id: canonical_id(
                "rule-lookup:sha256:",
                "wow-rules/reference-lookup/e0-e/1",
                &identity,
            )?,
            partition_id: partition_id.into(),
            entity_key: entity_key.into(),
            reference_generation: reference_generation.into(),
            reference_view_digest: reference_view_digest.into(),
            outcome,
            record_digest: record_digest.map(String::into_boxed_str),
            conflict_digest: conflict_digest.map(String::into_boxed_str),
        })
    }

    #[must_use]
    pub fn lookup_id(&self) -> &str {
        &self.lookup_id
    }

    #[must_use]
    pub const fn outcome(&self) -> RuleReferenceOutcome {
        self.outcome
    }

    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    #[must_use]
    pub fn entity_key(&self) -> &str {
        &self.entity_key
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleBudgetUsage {
    pub evaluations: u64,
    pub findings: u64,
    pub source_handles: u64,
    pub evidence_records: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CleanEvaluationRecord {
    evaluation_id: Box<str>,
    rule_id: RuleId,
    rule_version: ToolVersion,
    context_id: GenerationContextId,
    fixture_policy_id: Box<str>,
    scope_id: Box<str>,
    clean_claim_kind: RuleCleanClaimKind,
    input_fact_ids: Vec<Box<str>>,
    reference_lookup_id: Box<str>,
    capability_ids: Vec<CapabilityId>,
    coverage_ids: Vec<CoverageId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    guard_classification: Option<RuleGuardClassification>,
    budget_usage: RuleBudgetUsage,
}

impl CleanEvaluationRecord {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        rule_id: RuleId,
        rule_version: ToolVersion,
        context_id: GenerationContextId,
        fixture_policy_id: &str,
        scope_id: &str,
        clean_claim_kind: RuleCleanClaimKind,
        mut input_fact_ids: Vec<Box<str>>,
        reference_lookup_id: &str,
        mut capability_ids: Vec<CapabilityId>,
        mut coverage_ids: Vec<CoverageId>,
        guard_classification: Option<RuleGuardClassification>,
        budget_usage: RuleBudgetUsage,
    ) -> RuleResult<Self> {
        input_fact_ids.sort();
        input_fact_ids.dedup();
        capability_ids.sort();
        capability_ids.dedup();
        coverage_ids.sort_unstable();
        coverage_ids.dedup();
        #[derive(Serialize)]
        struct Identity<'a> {
            rule_id: &'a RuleId,
            rule_version: &'a ToolVersion,
            context_id: GenerationContextId,
            fixture_policy_id: &'a str,
            scope_id: &'a str,
            clean_claim_kind: RuleCleanClaimKind,
            input_fact_ids: &'a [Box<str>],
            reference_lookup_id: &'a str,
            capability_ids: &'a [CapabilityId],
            coverage_ids: &'a [CoverageId],
            #[serde(skip_serializing_if = "Option::is_none")]
            guard_classification: Option<RuleGuardClassification>,
            budget_usage: RuleBudgetUsage,
        }
        let identity = Identity {
            rule_id: &rule_id,
            rule_version: &rule_version,
            context_id,
            fixture_policy_id,
            scope_id,
            clean_claim_kind,
            input_fact_ids: &input_fact_ids,
            reference_lookup_id,
            capability_ids: &capability_ids,
            coverage_ids: &coverage_ids,
            guard_classification,
            budget_usage,
        };
        Ok(Self {
            evaluation_id: canonical_id(
                "rule-evaluation:sha256:",
                "wow-rules/clean-evaluation/e0-e/1",
                &identity,
            )?,
            rule_id,
            rule_version,
            context_id,
            fixture_policy_id: fixture_policy_id.into(),
            scope_id: scope_id.into(),
            clean_claim_kind,
            input_fact_ids,
            reference_lookup_id: reference_lookup_id.into(),
            capability_ids,
            coverage_ids,
            guard_classification,
            budget_usage,
        })
    }

    #[must_use]
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }

    #[must_use]
    pub const fn clean_claim_kind(&self) -> RuleCleanClaimKind {
        self.clean_claim_kind
    }

    #[must_use]
    pub const fn guard_classification(&self) -> Option<RuleGuardClassification> {
        self.guard_classification
    }

    #[must_use]
    pub fn coverage_ids(&self) -> &[CoverageId] {
        &self.coverage_ids
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleNotEvaluatedDetail {
    evaluation_id: Box<str>,
    rule_id: RuleId,
    rule_version: ToolVersion,
    context_id: GenerationContextId,
    scope_id: Box<str>,
    blockers: Vec<RuleBlockerKind>,
    core_record: NotEvaluatedRecord,
    input_fact_ids: Vec<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_lookup_id: Option<Box<str>>,
}

impl RuleNotEvaluatedDetail {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        rule_id: RuleId,
        rule_version: ToolVersion,
        context_id: GenerationContextId,
        scope_id: &str,
        mut blockers: Vec<RuleBlockerKind>,
        core_record: NotEvaluatedRecord,
        mut input_fact_ids: Vec<Box<str>>,
        reference_lookup_id: Option<&str>,
    ) -> RuleResult<Self> {
        blockers.sort();
        blockers.dedup();
        input_fact_ids.sort();
        input_fact_ids.dedup();
        #[derive(Serialize)]
        struct Identity<'a> {
            rule_id: &'a RuleId,
            rule_version: &'a ToolVersion,
            context_id: GenerationContextId,
            scope_id: &'a str,
            blockers: &'a [RuleBlockerKind],
            core_record_id: wow_core::NotEvaluatedId,
            input_fact_ids: &'a [Box<str>],
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_lookup_id: Option<&'a str>,
        }
        let identity = Identity {
            rule_id: &rule_id,
            rule_version: &rule_version,
            context_id,
            scope_id,
            blockers: &blockers,
            core_record_id: core_record.not_evaluated_id(),
            input_fact_ids: &input_fact_ids,
            reference_lookup_id,
        };
        Ok(Self {
            evaluation_id: canonical_id(
                "rule-evaluation:sha256:",
                "wow-rules/not-evaluated/e0-e/1",
                &identity,
            )?,
            rule_id,
            rule_version,
            context_id,
            scope_id: scope_id.into(),
            blockers,
            core_record,
            input_fact_ids,
            reference_lookup_id: reference_lookup_id.map(Into::into),
        })
    }

    #[must_use]
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }

    #[must_use]
    pub fn blockers(&self) -> &[RuleBlockerKind] {
        &self.blockers
    }

    #[must_use]
    pub const fn core_record(&self) -> &NotEvaluatedRecord {
        &self.core_record
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleFailure {
    evaluation_id: Box<str>,
    rule_id: RuleId,
    rule_version: ToolVersion,
    context_id: GenerationContextId,
    scope_id: Box<str>,
    error_code: RuleErrorCode,
}

impl RuleFailure {
    pub fn new(
        rule_id: RuleId,
        rule_version: ToolVersion,
        context_id: GenerationContextId,
        scope_id: &str,
        error_code: RuleErrorCode,
    ) -> RuleResult<Self> {
        #[derive(Serialize)]
        struct Identity<'a> {
            rule_id: &'a RuleId,
            rule_version: &'a ToolVersion,
            context_id: GenerationContextId,
            scope_id: &'a str,
            error_code: RuleErrorCode,
        }
        let identity = Identity {
            rule_id: &rule_id,
            rule_version: &rule_version,
            context_id,
            scope_id,
            error_code,
        };
        Ok(Self {
            evaluation_id: canonical_id(
                "rule-evaluation:sha256:",
                "wow-rules/failure/e0-e/1",
                &identity,
            )?,
            rule_id,
            rule_version,
            context_id,
            scope_id: scope_id.into(),
            error_code,
        })
    }
    #[must_use]
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }

    #[must_use]
    pub const fn error_code(&self) -> RuleErrorCode {
        self.error_code
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleFindingSet {
    evaluation_id: Box<str>,
    fixture_policy_id: Box<str>,
    input_fact_ids: Vec<Box<str>>,
    findings: Vec<Finding>,
    source_handles: Vec<SourceHandle>,
    evidence_records: Vec<EvidenceRecord>,
    coverage_ids: Vec<CoverageId>,
    reference_lookup: RuleReferenceLookupRecord,
    #[serde(skip_serializing_if = "Option::is_none")]
    guard_classification: Option<RuleGuardClassification>,
    budget_usage: RuleBudgetUsage,
}

pub(crate) struct RuleFindingSetInput {
    pub input_fact_ids: Vec<Box<str>>,
    pub findings: Vec<Finding>,
    pub source_handles: Vec<SourceHandle>,
    pub evidence_records: Vec<EvidenceRecord>,
    pub coverage_ids: Vec<CoverageId>,
    pub reference_lookup: RuleReferenceLookupRecord,
    pub guard_classification: Option<RuleGuardClassification>,
    pub budget_usage: RuleBudgetUsage,
}

impl RuleFindingSet {
    pub(crate) fn build(fixture_policy_id: &str, input: RuleFindingSetInput) -> RuleResult<Self> {
        let RuleFindingSetInput {
            mut input_fact_ids,
            findings,
            source_handles,
            evidence_records,
            mut coverage_ids,
            reference_lookup,
            guard_classification,
            budget_usage,
        } = input;
        if findings.is_empty() {
            return Err(RuleError::new(
                RuleErrorCode::RuleOutcomeInvalid,
                "Findings outcome cannot contain an empty finding list",
            ));
        }
        #[derive(Serialize)]
        struct Identity<'a> {
            fixture_policy_id: &'a str,
            input_fact_ids: &'a [Box<str>],
            finding_ids: Vec<wow_core::FindingId>,
            source_handle_ids: Vec<wow_core::StableHandleId>,
            evidence_ids: Vec<wow_core::EvidenceId>,
            coverage_ids: &'a [CoverageId],
            reference_lookup_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            guard_classification: Option<RuleGuardClassification>,
            budget_usage: RuleBudgetUsage,
        }
        input_fact_ids.sort();
        input_fact_ids.dedup();
        coverage_ids.sort_unstable();
        coverage_ids.dedup();
        let mut finding_ids = findings.iter().map(Finding::finding_id).collect::<Vec<_>>();
        let mut source_handle_ids = source_handles
            .iter()
            .map(SourceHandle::handle_id)
            .collect::<Vec<_>>();
        let mut evidence_ids = evidence_records
            .iter()
            .map(EvidenceRecord::evidence_id)
            .collect::<Vec<_>>();
        finding_ids.sort_unstable();
        source_handle_ids.sort_unstable();
        evidence_ids.sort_unstable();
        let identity = Identity {
            fixture_policy_id,
            input_fact_ids: &input_fact_ids,
            finding_ids,
            source_handle_ids,
            evidence_ids,
            coverage_ids: &coverage_ids,
            reference_lookup_id: reference_lookup.lookup_id(),
            guard_classification,
            budget_usage,
        };
        Ok(Self {
            evaluation_id: canonical_id(
                "rule-evaluation:sha256:",
                "wow-rules/findings/e0-e/1",
                &identity,
            )?,
            fixture_policy_id: fixture_policy_id.into(),
            input_fact_ids,
            findings,
            source_handles,
            evidence_records,
            coverage_ids,
            reference_lookup,
            guard_classification,
            budget_usage,
        })
    }

    #[must_use]
    pub fn evaluation_id(&self) -> &str {
        &self.evaluation_id
    }

    #[must_use]
    pub fn fixture_policy_id(&self) -> &str {
        &self.fixture_policy_id
    }

    #[must_use]
    pub fn input_fact_ids(&self) -> &[Box<str>] {
        &self.input_fact_ids
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub fn source_handles(&self) -> &[SourceHandle] {
        &self.source_handles
    }

    #[must_use]
    pub fn evidence_records(&self) -> &[EvidenceRecord] {
        &self.evidence_records
    }

    #[must_use]
    pub const fn reference_lookup(&self) -> &RuleReferenceLookupRecord {
        &self.reference_lookup
    }

    #[must_use]
    pub fn coverage_ids(&self) -> &[CoverageId] {
        &self.coverage_ids
    }

    #[must_use]
    pub const fn guard_classification(&self) -> Option<RuleGuardClassification> {
        self.guard_classification
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RuleEvaluationOutcome {
    Findings { result: Box<RuleFindingSet> },
    EvaluatedClean { record: CleanEvaluationRecord },
    NotEvaluated { detail: Box<RuleNotEvaluatedDetail> },
    Failed { failure: RuleFailure },
    Cancelled,
}

impl RuleEvaluationOutcome {
    #[must_use]
    pub const fn status(&self) -> RuleEvaluationStatus {
        match self {
            Self::Findings { .. } => RuleEvaluationStatus::Findings,
            Self::EvaluatedClean { .. } => RuleEvaluationStatus::EvaluatedClean,
            Self::NotEvaluated { .. } => RuleEvaluationStatus::NotEvaluated,
            Self::Failed { .. } => RuleEvaluationStatus::Failed,
            Self::Cancelled => RuleEvaluationStatus::Cancelled,
        }
    }

    #[must_use]
    pub fn evaluation_id(&self) -> Option<&str> {
        match self {
            Self::Findings { result } => Some(result.evaluation_id()),
            Self::EvaluatedClean { record } => Some(record.evaluation_id()),
            Self::NotEvaluated { detail } => Some(detail.evaluation_id()),
            Self::Failed { failure } => Some(&failure.evaluation_id),
            Self::Cancelled => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleEvaluationRecord {
    rule_id: RuleId,
    rule_version: ToolVersion,
    scope_id: Box<str>,
    outcome: RuleEvaluationOutcome,
}

impl RuleEvaluationRecord {
    pub(crate) fn new(
        rule_id: RuleId,
        rule_version: ToolVersion,
        scope_id: &str,
        outcome: RuleEvaluationOutcome,
    ) -> Self {
        Self {
            rule_id,
            rule_version,
            scope_id: scope_id.into(),
            outcome,
        }
    }

    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    #[must_use]
    pub fn scope_id(&self) -> &str {
        &self.scope_id
    }

    #[must_use]
    pub const fn outcome(&self) -> &RuleEvaluationOutcome {
        &self.outcome
    }

    #[must_use]
    pub const fn rule_version(&self) -> &ToolVersion {
        &self.rule_version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleExecutionReport {
    schema: &'static str,
    report_id: Box<str>,
    registry_id: Box<str>,
    context_id: GenerationContextId,
    project_snapshot_id: Box<str>,
    analyzer_snapshot_id: Box<str>,
    reference_generation: Box<str>,
    reference_view_digest: Box<str>,
    fixture_policy_id: Box<str>,
    evaluations: Vec<RuleEvaluationRecord>,
    budget_usage: RuleBudgetUsage,
}

impl RuleExecutionReport {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        registry_id: &str,
        context_id: GenerationContextId,
        project_snapshot_id: &str,
        analyzer_snapshot_id: &str,
        reference_generation: &str,
        reference_view_digest: &str,
        fixture_policy_id: &str,
        mut evaluations: Vec<RuleEvaluationRecord>,
        budget_usage: RuleBudgetUsage,
    ) -> RuleResult<Self> {
        evaluations.sort_by(|left, right| {
            left.rule_id
                .cmp(&right.rule_id)
                .then(left.scope_id.cmp(&right.scope_id))
                .then(
                    left.outcome
                        .evaluation_id()
                        .cmp(&right.outcome.evaluation_id()),
                )
        });
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            registry_id: &'a str,
            context_id: GenerationContextId,
            project_snapshot_id: &'a str,
            analyzer_snapshot_id: &'a str,
            reference_generation: &'a str,
            reference_view_digest: &'a str,
            fixture_policy_id: &'a str,
            evaluations: &'a [RuleEvaluationRecord],
            budget_usage: RuleBudgetUsage,
        }
        let identity = Identity {
            schema: "wow-rules/execution-report/1",
            registry_id,
            context_id,
            project_snapshot_id,
            analyzer_snapshot_id,
            reference_generation,
            reference_view_digest,
            fixture_policy_id,
            evaluations: &evaluations,
            budget_usage,
        };
        Ok(Self {
            schema: identity.schema,
            report_id: canonical_id(
                "rule-execution:sha256:",
                "wow-rules/execution-report/e0-e/1",
                &identity,
            )?,
            registry_id: registry_id.into(),
            context_id,
            project_snapshot_id: project_snapshot_id.into(),
            analyzer_snapshot_id: analyzer_snapshot_id.into(),
            reference_generation: reference_generation.into(),
            reference_view_digest: reference_view_digest.into(),
            fixture_policy_id: fixture_policy_id.into(),
            evaluations,
            budget_usage,
        })
    }

    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub fn evaluations(&self) -> &[RuleEvaluationRecord] {
        &self.evaluations
    }

    #[must_use]
    pub const fn budget_usage(&self) -> RuleBudgetUsage {
        self.budget_usage
    }

    #[must_use]
    pub fn finding_count(&self) -> usize {
        self.evaluations
            .iter()
            .map(|evaluation| match evaluation.outcome() {
                RuleEvaluationOutcome::Findings { result } => result.findings().len(),
                _ => 0,
            })
            .sum()
    }
}
