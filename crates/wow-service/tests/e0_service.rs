use std::sync::atomic::AtomicBool;

use wow_service::{
    BlockerKind, CapabilityState, CausalRelation, CheckContext, CheckRequest, CheckScope,
    CleanEvaluation, ComponentHealth, ComponentSnapshot, ContextIdentity, DeferredOperation,
    ExactSourceLocation, GenerationSelector, GenericFinding, LocalProjectBackend,
    LocalProjectInput, OperationId, OwnedServiceBackend, PresentationRelationKind, RawFinding,
    RuleBlocker, RuleEvaluation, RuleEvaluationState, RuleFinding, Service, ServiceBackendStatus,
    ServiceConfiguration, ServiceErrorCode, ServiceSemanticStatus, StatusRequest,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const NATIVE_FLOW_LIBRARY_WITH_GUARD: &str = "---@meta _\n---@class C_E0Fixture\n---@field SecretText fun(): string\nC_E0Fixture = {}\n\n---@param value any\n---@return boolean accessible\nfunction canaccessvalue(value) end\n";
const NATIVE_FLOW_LIBRARY_WITHOUT_GUARD: &str =
    "---@meta _\n---@class C_E0Fixture\n---@field SecretText fun(): string\nC_E0Fixture = {}\n";

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

fn native_flow_service(source: &str) -> TestResult<Service<LocalProjectBackend>> {
    native_flow_service_with_library(source, NATIVE_FLOW_LIBRARY_WITH_GUARD)
}

fn native_flow_service_with_library(
    source: &str,
    library_source: &str,
) -> TestResult<Service<LocalProjectBackend>> {
    use wow_core::{
        CanonicalResult, ContentDigest, ProfileId, ProfileIdentityBuilder, ProfileKind,
        ReferenceGenerationId, SchemaId, SchemaVersionEntry, SourceKind, SourceLogicalSnapshot,
        ToolVersion,
    };
    use wow_emmy::{
        EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
        LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    };
    use wow_project::{
        AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
        ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectInputFile, ProjectKind,
        ProjectSourceOriginId, ProjectWorkspaceId,
    };
    use wow_reference::{
        CoverageStatus, ReferencePartition, ReferenceRecord, ReferenceRecordKind, ReferenceView,
        RestrictionFacet, RestrictionState,
    };

    let profile_id: ProfileId = "profile:fixture:retail-120100-e0-v1".parse()?;
    let schema_id: SchemaId = "schema:wow:fixture-e0".parse()?;
    let schema_version: ToolVersion = "1.0.0".parse()?;
    let profile = ProfileIdentityBuilder::new(
        profile_id,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7_u8; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(schema_id, schema_version)])
    .fixture_scope("wow-service-e0-flow-fixture")
    .build()?;
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        sha('1'),
        sha('2'),
    )?;
    let analyzer = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256().to_owned(),
        ContentDigest::<CanonicalResult>::from_bytes([3_u8; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    let reference_generation = ReferenceGenerationId::derive(&"fixture-service-flow-e0")?;
    let reference_generation_text = reference_generation.to_string();
    let configuration = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-service-flow-e0-v1")?,
        ProjectKind::Fixture,
        profile,
        reference_generation,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:main:service-flow-e0")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:service-flow-e0",
    )?)
    .logical_root("fixtures/e0/service-flow/main")
    .capability_policy(ProjectCapabilityPolicy::degraded_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            library_source,
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let main_files = vec![ProjectInputFile::new("main/secret-local.lua", source)?];
    let bundle = ProjectInputBundle::closed(configuration, main_files, vec![library])?;
    let api = ReferencePartition::new(
        "reference.fixture.apidoc.system:C_E0Fixture",
        CoverageStatus::Complete,
        vec![ReferenceRecord::new(
            "function:C_E0Fixture.SecretText",
            ReferenceRecordKind::Api,
            "fixture-api-contract",
            vec!["source:fixture:api".to_owned()],
            Vec::new(),
        )?],
    )?;
    let restriction = ReferencePartition::new(
        "reference.fixture.restriction:C_E0Fixture.SecretText",
        CoverageStatus::Complete,
        vec![ReferenceRecord::new(
            "function:C_E0Fixture.SecretText",
            ReferenceRecordKind::Restriction,
            "return_position:1;applicability:unconditional_fixture",
            vec!["source:fixture:restriction".to_owned()],
            vec![RestrictionFacet::new(
                "secret.return",
                RestrictionState::Restricted,
                vec!["evidence:fixture:secret-return".to_owned()],
            )?],
        )?],
    )?;
    let reference = ReferenceView::new(
        reference_generation_text,
        vec![api, restriction],
        Vec::new(),
    )?;
    let input = LocalProjectInput::new(bundle, reference)?;
    let backend = LocalProjectBackend::new(input)?;
    Ok(Service::new(backend.configuration().clone(), backend)?)
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
    let json = serde_json::to_string(&result)?;
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
fn cancellation_leaves_service_usable() -> TestResult {
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
        serde_json::to_vec(&left_result)?,
        serde_json::to_vec(&right_result)?
    );
    Ok(())
}

#[test]
fn native_service_does_not_clean_a_reassigned_guard_value() -> TestResult {
    for source in [
        "local function changed_after_guard()\n    local text = C_E0Fixture.SecretText()\n    if canaccessvalue(text) then\n        text = C_E0Fixture.SecretText()\n        return text .. \"!\"\n    end\nend\n",
        "local function make_renderer()\n    local text = C_E0Fixture.SecretText()\n    local render\n    if canaccessvalue(text) then\n        render = function() return text .. \"!\" end\n    end\n    text = C_E0Fixture.SecretText()\n    return render\nend\n",
        "local function render_for()\n    local text = C_E0Fixture.SecretText()\n    local rendered\n    if canaccessvalue(text) then\n        for i = 1, 2 do\n            rendered = text .. \"!\"\n            text = C_E0Fixture.SecretText()\n        end\n    end\n    return rendered\nend\n",
        "local function render_generic_for()\n    local text = C_E0Fixture.SecretText()\n    local rendered\n    if canaccessvalue(text) then\n        for _, value in ipairs({ 1, 2 }) do\n            rendered = text .. \"!\"\n            text = C_E0Fixture.SecretText()\n        end\n    end\n    return rendered\nend\n",
        "local function render_while()\n    local text = C_E0Fixture.SecretText()\n    local rendered\n    local i = 0\n    if canaccessvalue(text) then\n        while i < 2 do\n            rendered = text .. \"!\"\n            text = C_E0Fixture.SecretText()\n            i = i + 1\n        end\n    end\n    return rendered\nend\n",
        "local function render_repeat()\n    local text = C_E0Fixture.SecretText()\n    local rendered\n    local i = 0\n    if canaccessvalue(text) then\n        repeat\n            rendered = text .. \"!\"\n            text = C_E0Fixture.SecretText()\n            i = i + 1\n        until i >= 2\n    end\n    return rendered\nend\n",
    ] {
        let service = native_flow_service(source)?;
        let result = service.check(
            &CheckRequest::new(
                OperationId::new("operation:check:native-flow-soundness")?,
                GenerationSelector::current_published("fixture-service-flow-e0-v1")?,
                CheckScope::WholeProject,
            ),
            &AtomicBool::new(false),
        )?;

        assert_eq!(result.semantic_status(), ServiceSemanticStatus::Partial);
        let secret_evaluations = result
            .rule_evaluations()
            .iter()
            .filter(|evaluation| evaluation.rule_id() == "wow.secret.local_operation@1")
            .collect::<Vec<_>>();
        assert!(
            secret_evaluations
                .iter()
                .any(|evaluation| evaluation.state() == RuleEvaluationState::NotEvaluated)
        );
        assert!(
            !secret_evaluations
                .iter()
                .any(|evaluation| evaluation.state() == RuleEvaluationState::Findings)
        );
    }
    Ok(())
}

#[test]
fn native_service_requires_library_guard_and_direct_operand_flow() -> TestResult {
    let inexact_source = concat!(
        "function canaccessvalue(value) return true end\n",
        "local function helper(value) return \"constant\" end\n",
        "local text = C_E0Fixture.SecretText()\n",
        "local other = C_E0Fixture.SecretText()\n",
        "if canaccessvalue(text) then\n",
        "    local rendered = text .. helper(other)\n",
        "end\n",
    );
    let inexact_service =
        native_flow_service_with_library(inexact_source, NATIVE_FLOW_LIBRARY_WITHOUT_GUARD)?;
    let inexact_result = inexact_service.check(
        &CheckRequest::new(
            OperationId::new("operation:check:native-inexact-flow")?,
            GenerationSelector::current_published("fixture-service-flow-e0-v1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        inexact_result.semantic_status(),
        ServiceSemanticStatus::Partial
    );
    let secret_evaluations = inexact_result
        .rule_evaluations()
        .iter()
        .filter(|evaluation| evaluation.rule_id() == "wow.secret.local_operation@1")
        .collect::<Vec<_>>();
    assert!(!secret_evaluations.is_empty());
    assert!(
        secret_evaluations
            .iter()
            .all(|evaluation| evaluation.state() == RuleEvaluationState::NotEvaluated)
    );

    let exact_source = concat!(
        "local text = C_E0Fixture.SecretText()\n",
        "if canaccessvalue(text) then\n",
        "    local rendered = text .. \"!\"\n",
        "end\n",
    );
    let exact_service = native_flow_service(exact_source)?;
    let exact_result = exact_service.check(
        &CheckRequest::new(
            OperationId::new("operation:check:native-library-guard")?,
            GenerationSelector::current_published("fixture-service-flow-e0-v1")?,
            CheckScope::WholeProject,
        ),
        &AtomicBool::new(false),
    )?;
    assert_eq!(exact_result.semantic_status(), ServiceSemanticStatus::Clean);
    assert!(exact_result.rule_evaluations().iter().any(|evaluation| {
        evaluation.rule_id() == "wow.secret.local_operation@1"
            && evaluation.state() == RuleEvaluationState::EvaluatedClean
    }));
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
