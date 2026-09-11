use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use wow_service::{
    BlockerKind, CapabilityState, CausalRelation, CheckContext, CheckRequest, CheckScope,
    CleanEvaluation, ComponentHealth, ComponentSnapshot, ContextIdentity, DeferredOperation,
    ExactSourceLocation, GenerationSelector, GenericFinding, OperationId, OwnedServiceBackend,
    PresentationRelationKind, RawFinding, RuleBlocker, RuleEvaluation, RuleFinding, Service,
    ServiceBackendStatus, ServiceConfiguration, ServiceErrorCode, ServiceSemanticStatus,
    StatusRequest,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn sha(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn configuration() -> TestResult<ServiceConfiguration> {
    Ok(ServiceConfiguration::builder()
        .project_id("project:fixture")
        .profile_id("profile:retail-e0")
        .reference_generation_id("reference:g1")
        .analyzer_pin_id("emmy:aaaca684")
        .rule_registry_id("rules:e0")
        .build()?)
}

fn components(order_reversed: bool) -> TestResult<Vec<ComponentSnapshot>> {
    let mut components = [
        "wow-core",
        "wow-reference",
        "wow-emmy",
        "wow-project",
        "wow-rules",
    ]
    .into_iter()
    .enumerate()
    .map(|(index, name)| {
        ComponentSnapshot::new(
            name,
            "0.1.0",
            format!("{name}:snapshot:{index}"),
            ComponentHealth::Ready,
        )?
        .with_capability(format!("{name}.e0.available"), CapabilityState::Available)
    })
    .collect::<Result<Vec<_>, _>>()?;
    if order_reversed {
        components.reverse();
    }
    Ok(components)
}

fn identity(configuration: &ServiceConfiguration) -> TestResult<ContextIdentity> {
    Ok(ContextIdentity::builder()
        .configuration_id(configuration.configuration_id())
        .project_id(configuration.project_id())
        .profile_id(configuration.profile_id())
        .reference_generation_id(configuration.reference_generation_id())
        .project_generation_id("project-generation:g1")
        .project_snapshot_id("project-snapshot:g1")
        .analyzer_snapshot_id("analyzer-snapshot:g1")
        .analyzer_pin_id(configuration.analyzer_pin_id())
        .rule_registry_id(configuration.rule_registry_id())
        .build()?)
}

fn generic_finding() -> TestResult<GenericFinding> {
    Ok(GenericFinding::new(
        "finding:generic:type-mismatch",
        "emmy.generic.fixture_error",
        "assign-type-mismatch",
        "error",
        ExactSourceLocation::new("main/generic-error.lua", sha('1'), 22, 27)?,
    )?)
}

fn rule_finding(
    id: &str,
    rule_id: &str,
    category: &str,
    path: &str,
    start: u64,
) -> TestResult<RuleFinding> {
    Ok(RuleFinding::new(
        id,
        rule_id,
        category,
        "error",
        ExactSourceLocation::new(path, sha('2'), start, start + 5)?,
        format!("source-handle:{path}"),
        vec![format!("evidence:{id}").into_boxed_str()],
    )?)
}

fn full_context(configuration: &ServiceConfiguration, reverse: bool) -> TestResult<CheckContext> {
    let generic = generic_finding()?;
    let api = rule_finding(
        "finding:api:removed",
        "wow.api.exists",
        "wow.api.missing",
        "main/missing-api.lua",
        7,
    )?;
    let unsafe_concat = rule_finding(
        "finding:secret:unsafe-concat",
        "wow.secret.local_operation",
        "wow.secret.unguarded_operation",
        "main/secret-local.lua",
        10,
    )?;
    let after_use = rule_finding(
        "finding:secret:guard-after-use",
        "wow.secret.local_operation",
        "wow.secret.guard_after_use",
        "main/secret-local.lua",
        30,
    )?;
    let different = rule_finding(
        "finding:secret:different-value",
        "wow.secret.local_operation",
        "wow.secret.different_value_guard",
        "main/secret-local.lua",
        50,
    )?;
    let api_evaluation = RuleEvaluation::findings(
        "evaluation:api:missing",
        "wow.api.exists",
        "scope:missing-api",
        vec![api.clone()],
    )?;
    let secret_evaluation = RuleEvaluation::findings(
        "evaluation:secret:findings",
        "wow.secret.local_operation",
        "scope:secret-findings",
        vec![unsafe_concat, after_use, different],
    )?;
    let guarded = CleanEvaluation::new(
        "clean:secret:guarded",
        "wow.secret.local_operation",
        "scope:guarded-concat",
        "secret_fixture_operation_guarded_for_exact_value_and_scope",
        vec!["coverage:secret:guarded".into()],
    )?;
    let clean_evaluation = RuleEvaluation::clean(
        "evaluation:secret:clean",
        "wow.secret.local_operation",
        "scope:guarded-concat",
        vec![guarded],
    )?;
    let mut evaluations = vec![api_evaluation, secret_evaluation, clean_evaluation];
    if reverse {
        evaluations.reverse();
    }
    Ok(CheckContext::new(
        identity(configuration)?,
        CheckScope::WholeProject,
        components(reverse)?,
        vec![generic.clone()],
        evaluations,
        vec![CausalRelation::new(
            api.finding_id(),
            generic.finding_id(),
            PresentationRelationKind::CausesOrExplains,
        )?],
    ))
}

fn service_with_context(
    configuration: ServiceConfiguration,
    context: CheckContext,
) -> TestResult<Service<OwnedServiceBackend>> {
    let status = ServiceBackendStatus::new(
        Some(context.identity().clone()),
        context.components().to_vec(),
    )?;
    Ok(Service::new(
        configuration,
        OwnedServiceBackend::new(status, vec![context])?,
    )?)
}

#[test]
fn status_reports_exact_state_without_claiming_a_check_passed() -> TestResult {
    let configuration = configuration()?;
    let context = full_context(&configuration, false)?;
    let service = service_with_context(configuration, context)?;
    let result = service.status(
        &StatusRequest::new(OperationId::new("operation:status:1")?),
        &AtomicBool::new(false),
    )?;
    assert_eq!(result.health(), ComponentHealth::Ready);
    assert_eq!(result.components().len(), 5);
    assert_eq!(result.deferred_operations().len(), 12);
    assert!(
        result
            .deferred_operations()
            .contains(&DeferredOperation::Release)
    );
    assert_eq!(
        result
            .current_context()
            .ok_or("current context")?
            .project_generation_id(),
        "project-generation:g1"
    );
    let json = serde_json::to_string(&*result)?;
    assert!(!json.contains("tests_passed"));
    assert!(!json.contains("clean"));
    Ok(())
}

#[test]
fn full_check_preserves_all_raw_findings_and_structured_folding() -> TestResult {
    let configuration = configuration()?;
    let context = full_context(&configuration, false)?;
    let service = service_with_context(configuration, context)?;
    let request = CheckRequest::new(
        OperationId::new("operation:check:full")?,
        GenerationSelector::current_published("project:fixture")?,
        CheckScope::WholeProject,
    );
    let result = service.check(&request, &AtomicBool::new(false))?;
    assert_eq!(result.semantic_status(), ServiceSemanticStatus::Findings);
    assert_eq!(result.raw_findings().len(), 5);
    assert_eq!(result.rule_evaluations().len(), 3);
    assert_eq!(result.presentation_graph().nodes().len(), 5);
    assert_eq!(result.presentation_graph().relations().len(), 1);
    assert_eq!(result.presentation_graph().display_root_ids().len(), 4);
    assert!(
        result
            .raw_findings()
            .iter()
            .any(|finding| matches!(finding, RawFinding::Generic(_)))
    );
    assert!(
        result
            .rule_evaluations()
            .iter()
            .any(|evaluation| !evaluation.clean_records().is_empty())
    );
    Ok(())
}

#[test]
fn same_operation_replays_exact_result_and_digest_conflict_fails() -> TestResult {
    let configuration = configuration()?;
    let context = full_context(&configuration, false)?;
    let service = service_with_context(configuration, context)?;
    let operation_id = OperationId::new("operation:check:replay")?;
    let request = CheckRequest::new(
        operation_id.clone(),
        GenerationSelector::exact("project-generation:g1")?,
        CheckScope::WholeProject,
    );
    let first = service.check(&request, &AtomicBool::new(false))?;
    let second = service.check(&request, &AtomicBool::new(false))?;
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(service.operation_registry()?.records().len(), 1);

    let conflict = CheckRequest::new(
        operation_id,
        GenerationSelector::exact("project-generation:other")?,
        CheckScope::WholeProject,
    );
    let error = service
        .check(&conflict, &AtomicBool::new(false))
        .err()
        .ok_or("expected operation conflict")?;
    assert_eq!(error.code(), ServiceErrorCode::OperationConflict);
    Ok(())
}

#[test]
fn partial_result_retains_findings_and_not_evaluated_state() -> TestResult {
    let configuration = configuration()?;
    let generic = generic_finding()?;
    let blocker = RuleBlocker::new(
        "blocker:coverage:api",
        BlockerKind::IncompleteCoverage,
        Some("reference.api.existence.complete".into()),
    )?;
    let context = CheckContext::new(
        identity(&configuration)?,
        CheckScope::WholeProject,
        components(false)?,
        vec![generic],
        vec![RuleEvaluation::not_evaluated(
            "evaluation:api:blocked",
            "wow.api.exists",
            "scope:whole-project",
            blocker,
        )?],
        Vec::new(),
    );
    let service = service_with_context(configuration, context)?;
    let result = service.check(
        &CheckRequest::new(
            OperationId::new("operation:check:partial")?,
            GenerationSelector::exact("project-generation:g1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    assert_eq!(result.semantic_status(), ServiceSemanticStatus::Partial);
    assert_eq!(result.raw_findings().len(), 1);
    assert_eq!(result.presentation_graph().nodes().len(), 2);
    Ok(())
}

#[test]
fn clean_requires_explicit_clean_authority() -> TestResult {
    let configuration = configuration()?;
    let clean = CleanEvaluation::new(
        "clean:api:known",
        "wow.api.exists",
        "scope:known-api",
        "api_exists_for_exact_use",
        vec!["coverage:api:complete".into()],
    )?;
    let context = CheckContext::new(
        identity(&configuration)?,
        CheckScope::WholeProject,
        components(false)?,
        Vec::new(),
        vec![RuleEvaluation::clean(
            "evaluation:api:clean",
            "wow.api.exists",
            "scope:known-api",
            vec![clean],
        )?],
        Vec::new(),
    );
    let service = service_with_context(configuration, context)?;
    let result = service.check(
        &CheckRequest::new(
            OperationId::new("operation:check:clean")?,
            GenerationSelector::exact("project-generation:g1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    assert_eq!(result.semantic_status(), ServiceSemanticStatus::Clean);
    assert!(result.raw_findings().is_empty());
    Ok(())
}

#[test]
fn cancellation_does_not_publish_or_poison_operation_id() -> TestResult {
    let configuration = configuration()?;
    let context = full_context(&configuration, false)?;
    let service = service_with_context(configuration, context)?;
    let request = CheckRequest::new(
        OperationId::new("operation:check:cancel")?,
        GenerationSelector::exact("project-generation:g1")?,
        CheckScope::WholeProject,
    );
    let error = service
        .check(&request, &AtomicBool::new(true))
        .err()
        .ok_or("expected cancellation")?;
    assert_eq!(error.code(), ServiceErrorCode::Cancelled);
    assert!(service.operation_registry()?.records().is_empty());
    let result = service.check(&request, &AtomicBool::new(false))?;
    assert_eq!(result.semantic_status(), ServiceSemanticStatus::Findings);
    Ok(())
}

#[test]
fn canonical_result_is_invariant_to_owner_input_order() -> TestResult {
    let left_configuration = configuration()?;
    let right_configuration = configuration()?;
    let left = service_with_context(
        left_configuration.clone(),
        full_context(&left_configuration, false)?,
    )?;
    let right = service_with_context(
        right_configuration.clone(),
        full_context(&right_configuration, true)?,
    )?;
    let left_result = left.check(
        &CheckRequest::new(
            OperationId::new("operation:check:deterministic")?,
            GenerationSelector::exact("project-generation:g1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    let right_result = right.check(
        &CheckRequest::new(
            OperationId::new("operation:check:deterministic")?,
            GenerationSelector::exact("project-generation:g1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    assert_eq!(left_result.result_id(), right_result.result_id());
    assert_eq!(
        serde_json::to_vec(&*left_result)?,
        serde_json::to_vec(&*right_result)?
    );
    Ok(())
}

#[test]
fn deferred_operations_fail_explicitly() -> TestResult {
    let configuration = configuration()?;
    let context = full_context(&configuration, false)?;
    let service = service_with_context(configuration, context)?;
    let error = service
        .deferred(DeferredOperation::Search)
        .err()
        .ok_or("expected deferred operation error")?;
    assert_eq!(
        error.code(),
        ServiceErrorCode::OperationNotImplementedForMilestone
    );
    Ok(())
}
