use std::error::Error;

use wow_rules::{
    AccessGuardObservation, AnalyzerResolution, ApiCallObservation, ApiPresence, BindingObservation,
    ByteSpan, CoverageState, DominanceObservation, OperationKind, OperationObservation,
    ReferenceApiEvidence, RuleDecision, RuleDiagnosticCode, RuleErrorCode, RuleEvaluationInput,
    RuleReportStatus, SourceLocation, ValueRestriction, evaluate,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn boxed(value: &str) -> Box<str> {
    value.into()
}

fn source(start: u64, end: u64) -> TestResult<SourceLocation> {
    Ok(SourceLocation::new(
        "main/example.lua",
        format!("sha256:{}", "a".repeat(64)),
        ByteSpan::new(start, end).ok_or("invalid test span")?,
    ))
}

fn call(id: &str, member: &str, resolution: AnalyzerResolution) -> TestResult<ApiCallObservation> {
    Ok(ApiCallObservation {
        fact_id: boxed(id),
        source: source(0, 24)?,
        receiver: boxed("C_Test"),
        member: boxed(member),
        resolution,
    })
}

fn reference(
    id: &str,
    member: &str,
    presence_coverage: CoverageState,
    presence: ApiPresence,
    restriction_coverage: CoverageState,
    restriction: ValueRestriction,
) -> ReferenceApiEvidence {
    ReferenceApiEvidence {
        evidence_id: boxed(id),
        target_profile: boxed("mainline-12.1.5"),
        receiver: boxed("C_Test"),
        member: boxed(member),
        presence_coverage,
        presence,
        restriction_coverage,
        restriction,
    }
}

fn empty_input() -> RuleEvaluationInput {
    RuleEvaluationInput {
        project_snapshot_id: boxed("project-snapshot:sha256:1111"),
        project_generation: boxed("project-generation:sha256:2222"),
        reference_view_id: boxed("reference-view:sha256:3333"),
        target_profile: boxed("mainline-12.1.5"),
        calls: Vec::new(),
        bindings: Vec::new(),
        operations: Vec::new(),
        guards: Vec::new(),
        dominance: Vec::new(),
        reference: Vec::new(),
    }
}

fn secret_operation_input() -> TestResult<RuleEvaluationInput> {
    let mut input = empty_input();
    input.calls.push(call(
        "call:secret",
        "SecretText",
        AnalyzerResolution::Resolved,
    )?);
    input.bindings.push(BindingObservation {
        fact_id: boxed("binding:text"),
        source: source(0, 32)?,
        name: boxed("text"),
        initializer_call_fact_id: Some(boxed("call:secret")),
    });
    input.operations.push(OperationObservation {
        fact_id: boxed("operation:concat"),
        source: source(40, 56)?,
        kind: OperationKind::Concatenate,
        operand_binding_fact_ids: vec![boxed("binding:text")],
    });
    input.reference.push(reference(
        "reference:secret-text",
        "SecretText",
        CoverageState::Complete,
        ApiPresence::Present,
        CoverageState::Complete,
        ValueRestriction::Secret,
    ));
    Ok(input)
}

#[test]
fn missing_api_requires_both_unresolved_call_and_complete_reference_absence() -> TestResult {
    let mut input = empty_input();
    input.calls.push(call(
        "call:removed",
        "RemovedApi",
        AnalyzerResolution::Unresolved,
    )?);
    input.reference.push(reference(
        "reference:removed",
        "RemovedApi",
        CoverageState::Complete,
        ApiPresence::Absent,
        CoverageState::Unavailable,
        ValueRestriction::Unknown,
    ));

    let report = evaluate(&input)?;
    assert_eq!(report.status(), RuleReportStatus::Complete);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(report.diagnostics()[0].code, RuleDiagnosticCode::MissingApi);
    assert_eq!(report.outcomes()[0].decision, RuleDecision::Diagnostic);
    Ok(())
}

#[test]
fn analyzer_unresolved_without_complete_reference_coverage_is_not_a_diagnostic() -> TestResult {
    let mut input = empty_input();
    input.calls.push(call(
        "call:unknown",
        "MaybeApi",
        AnalyzerResolution::Unresolved,
    )?);
    input.reference.push(reference(
        "reference:partial",
        "MaybeApi",
        CoverageState::Partial,
        ApiPresence::Unknown,
        CoverageState::Unavailable,
        ValueRestriction::Unknown,
    ));

    let report = evaluate(&input)?;
    assert_eq!(report.status(), RuleReportStatus::Partial);
    assert!(report.diagnostics().is_empty());
    assert_eq!(report.outcomes()[0].decision, RuleDecision::NotEvaluated);
    Ok(())
}

#[test]
fn known_secret_concatenation_without_dominating_guard_is_reported() -> TestResult {
    let report = evaluate(&secret_operation_input()?)?;
    assert_eq!(report.status(), RuleReportStatus::Complete);
    assert_eq!(report.diagnostics().len(), 1);
    assert_eq!(
        report.diagnostics()[0].code,
        RuleDiagnosticCode::SecretValueUsedWithoutDominatingAccessGuard
    );
    Ok(())
}

#[test]
fn exact_binding_guard_must_dominate_the_operation() -> TestResult {
    let mut input = secret_operation_input()?;
    input.guards.push(AccessGuardObservation {
        fact_id: boxed("guard:text"),
        source: source(33, 39)?,
        binding_fact_id: boxed("binding:text"),
    });

    let without_relation = evaluate(&input)?;
    assert_eq!(without_relation.diagnostics().len(), 1);

    input.dominance.push(DominanceObservation {
        guard_fact_id: boxed("guard:text"),
        operation_fact_id: boxed("operation:concat"),
    });
    let guarded = evaluate(&input)?;
    assert!(guarded.diagnostics().is_empty());
    assert!(guarded.outcomes().iter().any(|outcome| {
        outcome.subject_fact_id.as_ref() == "operation:concat"
            && outcome.decision == RuleDecision::Pass
    }));
    Ok(())
}

#[test]
fn guard_for_shadowed_binding_does_not_authorize_secret_operand() -> TestResult {
    let mut input = secret_operation_input()?;
    input.bindings.push(BindingObservation {
        fact_id: boxed("binding:shadow"),
        source: source(25, 31)?,
        name: boxed("text"),
        initializer_call_fact_id: None,
    });
    input.guards.push(AccessGuardObservation {
        fact_id: boxed("guard:shadow"),
        source: source(33, 39)?,
        binding_fact_id: boxed("binding:shadow"),
    });
    input.dominance.push(DominanceObservation {
        guard_fact_id: boxed("guard:shadow"),
        operation_fact_id: boxed("operation:concat"),
    });

    let report = evaluate(&input)?;
    assert_eq!(report.diagnostics().len(), 1);
    Ok(())
}

#[test]
fn proven_non_secret_operand_passes_without_guard() -> TestResult {
    let mut input = secret_operation_input()?;
    input.reference[0].restriction = ValueRestriction::NonSecret;
    let report = evaluate(&input)?;
    assert!(report.diagnostics().is_empty());
    assert_eq!(report.status(), RuleReportStatus::Complete);
    Ok(())
}

#[test]
fn contradictory_resolved_call_and_complete_absence_fail_closed() -> TestResult {
    let mut input = empty_input();
    input.calls.push(call(
        "call:contradiction",
        "KnownApi",
        AnalyzerResolution::Resolved,
    )?);
    input.reference.push(reference(
        "reference:absent",
        "KnownApi",
        CoverageState::Complete,
        ApiPresence::Absent,
        CoverageState::Unavailable,
        ValueRestriction::Unknown,
    ));
    let error = evaluate(&input).err().ok_or("expected contradiction")?;
    assert_eq!(error.code(), RuleErrorCode::ContradictoryEvidence);
    Ok(())
}

#[test]
fn dangling_or_cross_file_relations_are_rejected() -> TestResult {
    let mut input = secret_operation_input()?;
    input.dominance.push(DominanceObservation {
        guard_fact_id: boxed("guard:missing"),
        operation_fact_id: boxed("operation:concat"),
    });
    let error = evaluate(&input).err().ok_or("expected dangling relation")?;
    assert_eq!(error.code(), RuleErrorCode::DanglingRelation);

    let mut input = secret_operation_input()?;
    input.bindings[0].source = SourceLocation::new(
        "main/other.lua",
        format!("sha256:{}", "b".repeat(64)),
        ByteSpan::new(0, 10).ok_or("invalid test span")?,
    );
    let error = evaluate(&input).err().ok_or("expected source mismatch")?;
    assert_eq!(error.code(), RuleErrorCode::SourceMismatch);
    Ok(())
}

#[test]
fn input_order_does_not_change_report_identity_or_bytes() -> TestResult {
    let mut left = secret_operation_input()?;
    left.calls.push(call(
        "call:known",
        "KnownApi",
        AnalyzerResolution::Resolved,
    )?);
    left.reference.push(reference(
        "reference:known",
        "KnownApi",
        CoverageState::Complete,
        ApiPresence::Present,
        CoverageState::Complete,
        ValueRestriction::NonSecret,
    ));
    let mut right = left.clone();
    right.calls.reverse();
    right.bindings.reverse();
    right.operations.reverse();
    right.reference.reverse();

    let left_report = evaluate(&left)?;
    let right_report = evaluate(&right)?;
    assert_eq!(left_report, right_report);
    assert_eq!(left_report.report_id(), right_report.report_id());
    assert_eq!(serde_json::to_vec(&left_report)?, serde_json::to_vec(&right_report)?);
    Ok(())
}
