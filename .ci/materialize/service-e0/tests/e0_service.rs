use std::error::Error;

use wow_rules::{
    AnalyzerResolution, ApiCallObservation, ApiPresence, ByteSpan, CoverageState,
    ReferenceApiEvidence, RuleDecision, RuleEvaluationInput, SourceLocation, ValueRestriction,
};
use wow_service::{
    OperationRegistry, OperationSelector, ServiceErrorCode, ServiceFailureCode, ServiceHost,
    ServiceRequest, ServiceResponseStatus,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn boxed(value: &str) -> Box<str> {
    value.into()
}

fn input(generation: &str, member: &str, presence: ApiPresence) -> TestResult<RuleEvaluationInput> {
    let digest = format!("sha256:{}", "a".repeat(64));
    let restriction = if presence == ApiPresence::Present {
        ValueRestriction::NonSecret
    } else {
        ValueRestriction::Unknown
    };
    let restriction_coverage = if presence == ApiPresence::Present {
        CoverageState::Complete
    } else {
        CoverageState::Unavailable
    };
    Ok(RuleEvaluationInput {
        project_snapshot_id: boxed(&format!("project-snapshot:{generation}")),
        project_generation: boxed(generation),
        reference_view_id: boxed("reference-view:fixed"),
        target_profile: boxed("mainline-12.1.5"),
        calls: vec![ApiCallObservation {
            fact_id: boxed("call:one"),
            source: SourceLocation::new(
                "main/file.lua",
                digest,
                ByteSpan::new(0, 20).ok_or("span")?,
            ),
            receiver: boxed("C_Test"),
            member: boxed(member),
            resolution: if presence == ApiPresence::Absent {
                AnalyzerResolution::Unresolved
            } else {
                AnalyzerResolution::Resolved
            },
        }],
        bindings: Vec::new(),
        operations: Vec::new(),
        guards: Vec::new(),
        dominance: Vec::new(),
        reference: vec![ReferenceApiEvidence {
            evidence_id: boxed("reference:one"),
            target_profile: boxed("mainline-12.1.5"),
            receiver: boxed("C_Test"),
            member: boxed(member),
            presence_coverage: CoverageState::Complete,
            presence,
            restriction_coverage,
            restriction,
        }],
    })
}

#[test]
fn registry_exposes_one_versioned_read_only_operation() {
    let descriptors = OperationRegistry::descriptors();
    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].operation_id, "rules.evaluate");
    assert_eq!(descriptors[0].version, 1);
    assert!(descriptors[0].deterministic);
    assert!(descriptors[0].read_only);
}

#[test]
fn request_executes_against_one_exact_snapshot() -> TestResult {
    let host = ServiceHost::new(input("generation:one", "KnownApi", ApiPresence::Present)?)?;
    let view = host.read()?;
    let request = ServiceRequest::new(
        "request:one",
        view.snapshot_id(),
        OperationSelector::evaluate_rules(),
    )?;
    let response = host.execute(&request)?;
    assert_eq!(response.status(), ServiceResponseStatus::Completed);
    assert_eq!(response.service_snapshot_id(), view.snapshot_id());
    assert!(response.failure().is_none());
    assert!(response.result().is_some_and(|report| {
        report
            .outcomes()
            .iter()
            .all(|outcome| outcome.decision == RuleDecision::Pass)
    }));
    Ok(())
}

#[test]
fn stale_snapshot_and_unknown_operations_use_stable_rejection_envelopes() -> TestResult {
    let host = ServiceHost::new(input("generation:one", "KnownApi", ApiPresence::Present)?)?;
    let stale = ServiceRequest::new(
        "request:stale",
        "service-snapshot:stale",
        OperationSelector::evaluate_rules(),
    )?;
    let response = host.execute(&stale)?;
    assert_eq!(response.status(), ServiceResponseStatus::Rejected);
    assert_eq!(
        response.failure().map(|failure| failure.code),
        Some(ServiceFailureCode::StaleServiceSnapshot)
    );
    assert!(response.result().is_none());

    let current = host.read()?;
    let unknown = ServiceRequest::new(
        "request:unknown",
        current.snapshot_id(),
        OperationSelector::new("unknown.operation", 1)?,
    )?;
    let response = host.execute(&unknown)?;
    assert_eq!(
        response.failure().map(|failure| failure.code),
        Some(ServiceFailureCode::UnknownOperation)
    );

    let unsupported = ServiceRequest::new(
        "request:version",
        current.snapshot_id(),
        OperationSelector::new("rules.evaluate", 2)?,
    )?;
    let response = host.execute(&unsupported)?;
    assert_eq!(
        response.failure().map(|failure| failure.code),
        Some(ServiceFailureCode::UnsupportedOperationVersion)
    );
    Ok(())
}

#[test]
fn publication_is_atomic_and_old_read_views_remain_immutable() -> TestResult {
    let host = ServiceHost::new(input("generation:one", "KnownApi", ApiPresence::Present)?)?;
    let old = host.read()?;
    let published = host.publish(
        old.snapshot_id(),
        input("generation:two", "RemovedApi", ApiPresence::Absent)?,
    )?;
    assert_ne!(old.snapshot_id(), published.snapshot_id());
    assert_eq!(old.snapshot().project_generation(), "generation:one");
    assert_eq!(published.snapshot().project_generation(), "generation:two");
    assert_eq!(host.read()?.snapshot_id(), published.snapshot_id());
    assert_eq!(published.snapshot().rule_report().diagnostics().len(), 1);
    Ok(())
}

#[test]
fn no_change_publication_preserves_the_same_arc() -> TestResult {
    let original_input = input("generation:one", "KnownApi", ApiPresence::Present)?;
    let host = ServiceHost::new(original_input.clone())?;
    let old = host.read()?;
    let same = host.publish(old.snapshot_id(), original_input)?;
    assert!(old.is_same_snapshot(&same));
    Ok(())
}

#[test]
fn stale_publication_cannot_replace_current_snapshot() -> TestResult {
    let host = ServiceHost::new(input("generation:one", "KnownApi", ApiPresence::Present)?)?;
    let before = host.read()?;
    let error = host
        .publish(
            "service-snapshot:stale",
            input("generation:two", "KnownApi", ApiPresence::Present)?,
        )
        .err()
        .ok_or("expected publication conflict")?;
    assert_eq!(error.code(), ServiceErrorCode::PublicationConflict);
    assert_eq!(host.read()?.snapshot_id(), before.snapshot_id());
    Ok(())
}

#[test]
fn repeated_execution_is_byte_deterministic() -> TestResult {
    let host = ServiceHost::new(input("generation:one", "KnownApi", ApiPresence::Present)?)?;
    let view = host.read()?;
    let request = ServiceRequest::new(
        "request:deterministic",
        view.snapshot_id(),
        OperationSelector::evaluate_rules(),
    )?;
    let first = host.execute(&request)?;
    let second = host.execute(&request)?;
    assert_eq!(first, second);
    assert_eq!(first.response_id(), second.response_id());
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}
