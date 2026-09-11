use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::identity::canonical_id;
use crate::model::{MISSING_API_RULE, REPORT_SCHEMA, SECRET_OPERATION_RULE};
use crate::{
    AccessGuardObservation, AnalyzerResolution, ApiCallObservation, ApiPresence, BindingObservation,
    CoverageState, DominanceObservation, OperationObservation, ReferenceApiEvidence, RuleDecision,
    RuleDiagnostic, RuleDiagnosticCode, RuleError, RuleErrorCode, RuleEvaluationInput, RuleOutcome,
    RuleReport, RuleReportStatus, RuleResult, SourceLocation, ValueRestriction,
};

const MAX_FACTS: usize = 100_000;
const MAX_OPERANDS: usize = 64;
const MAX_ID_BYTES: usize = 1024;
const MAX_PATH_BYTES: usize = 4096;

struct Index<'a> {
    calls: BTreeMap<&'a str, &'a ApiCallObservation>,
    bindings: BTreeMap<&'a str, &'a BindingObservation>,
    operations: BTreeMap<&'a str, &'a OperationObservation>,
    guards: BTreeMap<&'a str, &'a AccessGuardObservation>,
    dominance: BTreeSet<(&'a str, &'a str)>,
    reference: BTreeMap<(&'a str, &'a str), &'a ReferenceApiEvidence>,
}

/// Evaluates the two closed E0-E rules over one exact immutable input package.
///
/// `Unresolved` analyzer state alone never establishes API absence. Secret-value
/// diagnostics require complete presence and restriction evidence for the exact
/// initializer API plus the absence of a dominating guard for the exact binding.
pub fn evaluate(input: &RuleEvaluationInput) -> RuleResult<RuleReport> {
    let index = validate(input)?;
    let normalized = normalized_input(input);
    let input_id = canonical_id("rules-input:sha256:", &normalized)?;
    let mut outcomes = Vec::with_capacity(input.calls.len() + input.operations.len());
    let mut diagnostics = Vec::new();

    evaluate_missing_api(input, &index, &mut outcomes, &mut diagnostics)?;
    evaluate_secret_operations(input, &index, &mut outcomes, &mut diagnostics)?;

    outcomes.sort();
    diagnostics.sort();
    let status = if outcomes
        .iter()
        .any(|outcome| outcome.decision == RuleDecision::NotEvaluated)
    {
        RuleReportStatus::Partial
    } else {
        RuleReportStatus::Complete
    };

    #[derive(Serialize)]
    struct ReportIdentity<'a> {
        schema: &'static str,
        input_id: &'a str,
        project_snapshot_id: &'a str,
        project_generation: &'a str,
        reference_view_id: &'a str,
        target_profile: &'a str,
        status: RuleReportStatus,
        outcomes: &'a [RuleOutcome],
        diagnostics: &'a [RuleDiagnostic],
    }
    let report_identity = ReportIdentity {
        schema: REPORT_SCHEMA,
        input_id: &input_id,
        project_snapshot_id: &input.project_snapshot_id,
        project_generation: &input.project_generation,
        reference_view_id: &input.reference_view_id,
        target_profile: &input.target_profile,
        status,
        outcomes: &outcomes,
        diagnostics: &diagnostics,
    };
    let report_id = canonical_id("rules-report:sha256:", &report_identity)?;
    Ok(RuleReport {
        schema: REPORT_SCHEMA,
        report_id,
        input_id,
        project_snapshot_id: input.project_snapshot_id.clone(),
        project_generation: input.project_generation.clone(),
        reference_view_id: input.reference_view_id.clone(),
        target_profile: input.target_profile.clone(),
        status,
        outcomes,
        diagnostics,
        limitations: vec![
            "analyzer resolution is not WoW API presence or absence authority",
            "secret classification requires complete independently identified reference evidence",
            "a guard suppresses only the exact binding and operation linked by a dominance fact",
            "no runtime, client build, combat, taint, secure execution or replacement inference is performed",
        ],
    })
}

fn evaluate_missing_api(
    input: &RuleEvaluationInput,
    index: &Index<'_>,
    outcomes: &mut Vec<RuleOutcome>,
    diagnostics: &mut Vec<RuleDiagnostic>,
) -> RuleResult<()> {
    let mut calls = input.calls.iter().collect::<Vec<_>>();
    calls.sort_by_key(|call| call.fact_id.as_ref());
    for call in calls {
        let evidence = index
            .reference
            .get(&(call.receiver.as_ref(), call.member.as_ref()))
            .copied();
        let (decision, reason, diagnostic) = match evidence {
            None => (
                RuleDecision::NotEvaluated,
                "reference_evidence_missing",
                None,
            ),
            Some(reference) if reference.presence_coverage != CoverageState::Complete => (
                RuleDecision::NotEvaluated,
                "reference_presence_coverage_incomplete",
                None,
            ),
            Some(reference) if reference.presence == ApiPresence::Present => (
                RuleDecision::Pass,
                "complete_reference_reports_present",
                None,
            ),
            Some(reference) if reference.presence == ApiPresence::Absent => {
                match call.resolution {
                    AnalyzerResolution::Resolved => {
                        return Err(RuleError::new(
                            RuleErrorCode::ContradictoryEvidence,
                            "resolved analyzer call conflicts with complete reference absence",
                            Some(&call.fact_id),
                        ));
                    }
                    AnalyzerResolution::Possible => (
                        RuleDecision::NotEvaluated,
                        "analyzer_resolution_possible",
                        None,
                    ),
                    AnalyzerResolution::Unresolved => {
                        let diagnostic = diagnostic(
                            RuleDiagnosticCode::MissingApi,
                            MISSING_API_RULE,
                            &call.fact_id,
                            call.source.clone(),
                            vec![call.fact_id.clone(), reference.evidence_id.clone()],
                        )?;
                        (
                            RuleDecision::Diagnostic,
                            "complete_reference_reports_absent",
                            Some(diagnostic),
                        )
                    }
                }
            }
            Some(_) => (
                RuleDecision::NotEvaluated,
                "reference_presence_unknown",
                None,
            ),
        };
        let diagnostic_id = diagnostic
            .as_ref()
            .map(|item| item.diagnostic_id.clone());
        if let Some(item) = diagnostic {
            diagnostics.push(item);
        }
        outcomes.push(RuleOutcome {
            rule_id: MISSING_API_RULE,
            subject_fact_id: call.fact_id.clone(),
            decision,
            reason,
            diagnostic_id,
        });
    }
    Ok(())
}

fn evaluate_secret_operations(
    input: &RuleEvaluationInput,
    index: &Index<'_>,
    outcomes: &mut Vec<RuleOutcome>,
    diagnostics: &mut Vec<RuleDiagnostic>,
) -> RuleResult<()> {
    let mut operations = input.operations.iter().collect::<Vec<_>>();
    operations.sort_by_key(|operation| operation.fact_id.as_ref());
    for operation in operations {
        let mut secret = BTreeSet::new();
        let mut unknown = false;
        for binding_id in &operation.operand_binding_fact_ids {
            let binding = index
                .bindings
                .get(binding_id.as_ref())
                .copied()
                .ok_or_else(|| dangling("operation operand binding", &operation.fact_id))?;
            let Some(call_id) = binding.initializer_call_fact_id.as_deref() else {
                unknown = true;
                continue;
            };
            let call = index
                .calls
                .get(call_id)
                .copied()
                .ok_or_else(|| dangling("binding initializer call", &binding.fact_id))?;
            if call.resolution != AnalyzerResolution::Resolved {
                unknown = true;
                continue;
            }
            let Some(reference) = index
                .reference
                .get(&(call.receiver.as_ref(), call.member.as_ref()))
                .copied()
            else {
                unknown = true;
                continue;
            };
            if reference.presence_coverage != CoverageState::Complete
                || reference.presence != ApiPresence::Present
                || reference.restriction_coverage != CoverageState::Complete
            {
                unknown = true;
                continue;
            }
            match reference.restriction {
                ValueRestriction::Secret => {
                    secret.insert(binding.fact_id.as_ref());
                }
                ValueRestriction::NonSecret => {}
                ValueRestriction::Unknown => unknown = true,
            }
        }

        let unguarded = secret
            .iter()
            .copied()
            .filter(|binding_id| !guard_dominates(binding_id, &operation.fact_id, index))
            .collect::<Vec<_>>();
        let (decision, reason, diagnostic) = if !unguarded.is_empty() {
            let mut related = vec![operation.fact_id.clone()];
            related.extend(unguarded.iter().map(|id| Box::<str>::from(*id)));
            let item = diagnostic(
                RuleDiagnosticCode::SecretValueUsedWithoutDominatingAccessGuard,
                SECRET_OPERATION_RULE,
                &operation.fact_id,
                operation.source.clone(),
                related,
            )?;
            (
                RuleDecision::Diagnostic,
                "secret_operand_not_dominated_by_exact_access_guard",
                Some(item),
            )
        } else if unknown {
            (
                RuleDecision::NotEvaluated,
                "secret_or_initializer_evidence_incomplete",
                None,
            )
        } else {
            (
                RuleDecision::Pass,
                if secret.is_empty() {
                    "all_operands_proven_non_secret"
                } else {
                    "all_secret_operands_have_exact_dominating_guards"
                },
                None,
            )
        };
        let diagnostic_id = diagnostic
            .as_ref()
            .map(|item| item.diagnostic_id.clone());
        if let Some(item) = diagnostic {
            diagnostics.push(item);
        }
        outcomes.push(RuleOutcome {
            rule_id: SECRET_OPERATION_RULE,
            subject_fact_id: operation.fact_id.clone(),
            decision,
            reason,
            diagnostic_id,
        });
    }
    Ok(())
}

fn guard_dominates(binding_id: &str, operation_id: &str, index: &Index<'_>) -> bool {
    index.guards.values().any(|guard| {
        guard.binding_fact_id.as_ref() == binding_id
            && index
                .dominance
                .contains(&(guard.fact_id.as_ref(), operation_id))
    })
}

fn diagnostic(
    code: RuleDiagnosticCode,
    rule_id: &'static str,
    subject_id: &str,
    source: SourceLocation,
    mut related_fact_ids: Vec<Box<str>>,
) -> RuleResult<RuleDiagnostic> {
    related_fact_ids.sort();
    related_fact_ids.dedup();
    #[derive(Serialize)]
    struct Identity<'a> {
        code: RuleDiagnosticCode,
        rule_id: &'static str,
        subject_id: &'a str,
        source: &'a SourceLocation,
        related_fact_ids: &'a [Box<str>],
    }
    let identity = Identity {
        code,
        rule_id,
        subject_id,
        source: &source,
        related_fact_ids: &related_fact_ids,
    };
    Ok(RuleDiagnostic {
        diagnostic_id: canonical_id("rule-diagnostic:sha256:", &identity)?,
        code,
        rule_id,
        source,
        related_fact_ids,
    })
}

fn validate(input: &RuleEvaluationInput) -> RuleResult<Index<'_>> {
    for identity in [
        input.project_snapshot_id.as_ref(),
        input.project_generation.as_ref(),
        input.reference_view_id.as_ref(),
        input.target_profile.as_ref(),
    ] {
        validate_identity(identity, None)?;
    }
    let total = input
        .calls
        .len()
        .checked_add(input.bindings.len())
        .and_then(|value| value.checked_add(input.operations.len()))
        .and_then(|value| value.checked_add(input.guards.len()))
        .and_then(|value| value.checked_add(input.dominance.len()))
        .and_then(|value| value.checked_add(input.reference.len()))
        .ok_or_else(limit)?;
    if total > MAX_FACTS {
        return Err(limit());
    }

    let mut all_ids = BTreeSet::new();
    let mut calls = BTreeMap::new();
    for call in &input.calls {
        fact_identity(&call.fact_id, &call.source, &mut all_ids)?;
        validate_qualified(&call.receiver, &call.fact_id)?;
        validate_identifier(&call.member, &call.fact_id)?;
        calls.insert(call.fact_id.as_ref(), call);
    }
    let mut bindings = BTreeMap::new();
    for binding in &input.bindings {
        fact_identity(&binding.fact_id, &binding.source, &mut all_ids)?;
        validate_identifier(&binding.name, &binding.fact_id)?;
        bindings.insert(binding.fact_id.as_ref(), binding);
    }
    let mut operations = BTreeMap::new();
    for operation in &input.operations {
        fact_identity(&operation.fact_id, &operation.source, &mut all_ids)?;
        if operation.operand_binding_fact_ids.is_empty()
            || operation.operand_binding_fact_ids.len() > MAX_OPERANDS
        {
            return Err(RuleError::new(
                RuleErrorCode::InputLimitExceeded,
                "operation operand count is outside the admitted range",
                Some(&operation.fact_id),
            ));
        }
        operations.insert(operation.fact_id.as_ref(), operation);
    }
    let mut guards = BTreeMap::new();
    for guard in &input.guards {
        fact_identity(&guard.fact_id, &guard.source, &mut all_ids)?;
        guards.insert(guard.fact_id.as_ref(), guard);
    }

    for binding in &input.bindings {
        if let Some(call_id) = binding.initializer_call_fact_id.as_deref() {
            let call = calls
                .get(call_id)
                .copied()
                .ok_or_else(|| dangling("binding initializer call", &binding.fact_id))?;
            same_source(&binding.source, &call.source, &binding.fact_id)?;
        }
    }
    for operation in &input.operations {
        for binding_id in &operation.operand_binding_fact_ids {
            let binding = bindings
                .get(binding_id.as_ref())
                .copied()
                .ok_or_else(|| dangling("operation operand binding", &operation.fact_id))?;
            same_source(&operation.source, &binding.source, &operation.fact_id)?;
        }
    }
    for guard in &input.guards {
        let binding = bindings
            .get(guard.binding_fact_id.as_ref())
            .copied()
            .ok_or_else(|| dangling("guard binding", &guard.fact_id))?;
        same_source(&guard.source, &binding.source, &guard.fact_id)?;
    }

    let mut dominance = BTreeSet::new();
    for relation in &input.dominance {
        let guard = guards
            .get(relation.guard_fact_id.as_ref())
            .copied()
            .ok_or_else(|| dangling("dominance guard", &relation.guard_fact_id))?;
        let operation = operations
            .get(relation.operation_fact_id.as_ref())
            .copied()
            .ok_or_else(|| dangling("dominance operation", &relation.operation_fact_id))?;
        same_source(&guard.source, &operation.source, &relation.guard_fact_id)?;
        if !dominance.insert((relation.guard_fact_id.as_ref(), relation.operation_fact_id.as_ref())) {
            return Err(RuleError::new(
                RuleErrorCode::DuplicateFact,
                "duplicate dominance relation",
                Some(&relation.guard_fact_id),
            ));
        }
    }

    let mut reference = BTreeMap::new();
    let mut evidence_ids = BTreeSet::new();
    for evidence in &input.reference {
        validate_identity(&evidence.evidence_id, Some(&evidence.evidence_id))?;
        if !evidence_ids.insert(evidence.evidence_id.as_ref()) {
            return Err(duplicate(&evidence.evidence_id));
        }
        validate_identity(&evidence.target_profile, Some(&evidence.evidence_id))?;
        validate_qualified(&evidence.receiver, &evidence.evidence_id)?;
        validate_identifier(&evidence.member, &evidence.evidence_id)?;
        if evidence.target_profile != input.target_profile {
            return Err(RuleError::new(
                RuleErrorCode::ContradictoryEvidence,
                "reference evidence target profile differs from the evaluation profile",
                Some(&evidence.evidence_id),
            ));
        }
        validate_reference_state(evidence)?;
        if reference
            .insert((evidence.receiver.as_ref(), evidence.member.as_ref()), evidence)
            .is_some()
        {
            return Err(RuleError::new(
                RuleErrorCode::DuplicateFact,
                "duplicate reference evidence for one API member and target profile",
                Some(&evidence.evidence_id),
            ));
        }
    }

    Ok(Index {
        calls,
        bindings,
        operations,
        guards,
        dominance,
        reference,
    })
}

fn validate_reference_state(evidence: &ReferenceApiEvidence) -> RuleResult<()> {
    let presence_valid = match evidence.presence_coverage {
        CoverageState::Complete => evidence.presence != ApiPresence::Unknown,
        CoverageState::Partial | CoverageState::Unavailable => {
            evidence.presence == ApiPresence::Unknown
        }
    };
    let restriction_valid = match (
        evidence.presence,
        evidence.restriction_coverage,
        evidence.restriction,
    ) {
        (
            ApiPresence::Present,
            CoverageState::Complete,
            ValueRestriction::Secret | ValueRestriction::NonSecret,
        ) => true,
        (
            ApiPresence::Present,
            CoverageState::Partial | CoverageState::Unavailable,
            ValueRestriction::Unknown,
        )
        | (
            ApiPresence::Absent | ApiPresence::Unknown,
            CoverageState::Partial | CoverageState::Unavailable,
            ValueRestriction::Unknown,
        ) => true,
        _ => false,
    };
    if !presence_valid || !restriction_valid {
        return Err(RuleError::new(
            RuleErrorCode::ContradictoryEvidence,
            "reference coverage, presence, and restriction states are inconsistent",
            Some(&evidence.evidence_id),
        ));
    }
    Ok(())
}

fn fact_identity<'a>(
    id: &'a str,
    source: &SourceLocation,
    ids: &mut BTreeSet<&'a str>,
) -> RuleResult<()> {
    validate_identity(id, Some(id))?;
    validate_source(source, id)?;
    if !ids.insert(id) {
        return Err(duplicate(id));
    }
    Ok(())
}

fn validate_source(source: &SourceLocation, subject: &str) -> RuleResult<()> {
    let path = source.path();
    if path.is_empty()
        || path.len() > MAX_PATH_BYTES
        || path.starts_with('/')
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
        || !valid_sha256(source.content_sha256())
        || source.span().start() > source.span().end()
    {
        return Err(RuleError::new(
            RuleErrorCode::InvalidSourceLocation,
            "source path, digest, or byte span is invalid",
            Some(subject),
        ));
    }
    Ok(())
}

fn same_source(left: &SourceLocation, right: &SourceLocation, subject: &str) -> RuleResult<()> {
    if left.path() != right.path() || left.content_sha256() != right.content_sha256() {
        return Err(RuleError::new(
            RuleErrorCode::SourceMismatch,
            "linked project facts do not belong to the same exact source file",
            Some(subject),
        ));
    }
    Ok(())
}

fn validate_identity(value: &str, subject: Option<&str>) -> RuleResult<()> {
    if value.is_empty()
        || value.len() > MAX_ID_BYTES
        || value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(RuleError::new(
            RuleErrorCode::InvalidIdentity,
            "identity is empty, oversized, or contains whitespace/control characters",
            subject,
        ));
    }
    Ok(())
}

fn validate_identifier(value: &str, subject: &str) -> RuleResult<()> {
    let mut bytes = value.bytes();
    if value.len() > MAX_ID_BYTES
        || !bytes
            .next()
            .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        || !bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
    {
        return Err(RuleError::new(
            RuleErrorCode::InvalidIdentity,
            "member or local name is not a static Lua identifier",
            Some(subject),
        ));
    }
    Ok(())
}

fn validate_qualified(value: &str, subject: &str) -> RuleResult<()> {
    if value.is_empty()
        || value
            .split('.')
            .any(|segment| validate_identifier(segment, subject).is_err())
    {
        return Err(RuleError::new(
            RuleErrorCode::InvalidIdentity,
            "receiver is not a static qualified Lua identifier",
            Some(subject),
        ));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn duplicate(subject: &str) -> RuleError {
    RuleError::new(
        RuleErrorCode::DuplicateFact,
        "duplicate fact or evidence identity",
        Some(subject),
    )
}

fn dangling(kind: &str, subject: &str) -> RuleError {
    RuleError::new(
        RuleErrorCode::DanglingRelation,
        format!("{kind} refers to an unknown fact"),
        Some(subject),
    )
}

fn limit() -> RuleError {
    RuleError::new(
        RuleErrorCode::InputLimitExceeded,
        "rule input exceeds the admitted fact budget",
        None,
    )
}

fn normalized_input(input: &RuleEvaluationInput) -> RuleEvaluationInput {
    let mut value = input.clone();
    value
        .calls
        .sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
    value
        .bindings
        .sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
    value
        .operations
        .sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
    value
        .guards
        .sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
    value.dominance.sort_by(|left, right| {
        (&left.guard_fact_id, &left.operation_fact_id)
            .cmp(&(&right.guard_fact_id, &right.operation_fact_id))
    });
    value.reference.sort_by(|left, right| {
        (
            &left.target_profile,
            &left.receiver,
            &left.member,
            &left.evidence_id,
        )
            .cmp(&(
                &right.target_profile,
                &right.receiver,
                &right.member,
                &right.evidence_id,
            ))
    });
    value
}
