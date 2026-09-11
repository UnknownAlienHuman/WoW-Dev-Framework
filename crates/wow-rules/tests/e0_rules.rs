use std::error::Error;
use std::sync::atomic::AtomicBool;

use wow_core::{
    CanonicalResult, ContentDigest, ProfileId, ProfileIdentity, ProfileIdentityBuilder,
    ProfileKind, ReferenceGenerationId, SchemaId, SchemaVersionEntry, SourceKind,
    SourceLogicalSnapshot, ToolVersion,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfiguration,
    ProjectConfigurationBuilder, ProjectFileRole, ProjectId, ProjectInputBundle, ProjectInputFile,
    ProjectKind, ProjectLanguageKind, ProjectPublisher, ProjectSourceOriginId, ProjectView,
    ProjectWorkspaceId,
};
use wow_reference::{
    CoverageStatus as ReferenceCoverageStatus, ReferenceConflict, ReferencePartition,
    ReferenceRecord, ReferenceRecordKind, ReferenceView, RestrictionFacet, RestrictionState,
};
use wow_rules::{
    API_EXISTS_RULE, CleanEvaluationRecord, RuleBlockerKind, RuleEvaluationOutcome,
    RuleExecutionBudget, RuleExecutionContext, RuleFixturePolicy, RuleGuardClassification,
    RuleRegistry, RuleScope, SECRET_LOCAL_RULE, execute_e0,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const API_PARTITION: &str = "reference.fixture.apidoc.system:C_E0Fixture";
const SECRET_PARTITION: &str = "reference.fixture.restriction:C_E0Fixture.SecretText";
const SECRET_ENTITY: &str = "function:C_E0Fixture.SecretText";
const SECRET_PAYLOAD: &str = "return_position:1;applicability:unconditional_fixture";

const LIBRARY_SOURCE: &str = "---@meta _\n---@class C_E0Fixture\n---@field KnownApi fun(value: string): boolean\n---@field SecretText fun(): string\nC_E0Fixture = {}\n\n---@param value any\n---@return boolean accessible\nfunction canaccessvalue(value) end\n";
const CLEAN_SOURCE: &str = "local accepted = C_E0Fixture.KnownApi(\"ok\")\nreturn accepted\n";
const MISSING_API_SOURCE: &str = "return C_E0Fixture.RemovedApi()\n";
const SECRET_LOCAL_SOURCE: &str = "local function unsafe_concat()\n    local text = C_E0Fixture.SecretText()\n    return text .. \"!\"\nend\n\nlocal function guarded_concat()\n    local text = C_E0Fixture.SecretText()\n    if canaccessvalue(text) then\n        return text .. \"!\"\n    end\nend\n\nlocal function guard_after_use()\n    local text = C_E0Fixture.SecretText()\n    local rendered = text .. \"!\"\n    if canaccessvalue(text) then\n        return rendered\n    end\nend\n\nlocal function different_value_guard()\n    local text = C_E0Fixture.SecretText()\n    local ordinary = \"ok\"\n    if canaccessvalue(ordinary) then\n        return text .. \"!\"\n    end\nend\n\nreturn unsafe_concat, guarded_concat, guard_after_use, different_value_guard\n";

fn backend() -> TestResult<EmmyBackendIdentity> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?)
}

fn fixture_profile() -> TestResult<ProfileIdentity> {
    let profile_id: ProfileId = "profile:fixture:retail-120100-e0-v1".parse()?;
    let schema_id: SchemaId = "schema:wow:fixture-e0".parse()?;
    let schema_version: ToolVersion = "1.0.0".parse()?;
    Ok(ProfileIdentityBuilder::new(
        profile_id,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7_u8; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(schema_id, schema_version)])
    .fixture_scope("wow-rules-e0-e-fixture")
    .build()?)
}

fn configuration() -> TestResult<ProjectConfiguration> {
    let backend = backend()?;
    let compatibility_report_sha256 = backend.compatibility_report_sha256().to_owned();
    let analyzer_binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        compatibility_report_sha256,
        ContentDigest::<CanonicalResult>::from_bytes([3_u8; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend,
    )?;
    Ok(ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-rules-e0-v1")?,
        ProjectKind::Fixture,
        fixture_profile()?,
        ReferenceGenerationId::derive(&"fixture-reference-rules-e0")?,
        analyzer_binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:main:rules-e0")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:fixture-rules-e0-v1",
    )?)
    .logical_root("fixtures/e0/rules/main")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .build()?)
}

fn project() -> TestResult<ProjectView> {
    let configuration = configuration()?;
    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            LIBRARY_SOURCE,
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let files = vec![
        ProjectInputFile::declared(
            "main/clean.lua",
            CLEAN_SOURCE,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            Some("wow-emmy/workspace-fixture:main/clean.lua"),
        )?,
        ProjectInputFile::declared(
            "main/missing-api.lua",
            MISSING_API_SOURCE,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            Some("wow-emmy/workspace-fixture:main/missing-api.lua"),
        )?,
        ProjectInputFile::declared(
            "main/secret-local.lua",
            SECRET_LOCAL_SOURCE,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            Some("wow-emmy/workspace-fixture:main/secret-local.lua"),
        )?,
    ];
    let bundle = ProjectInputBundle::closed(configuration, files, vec![library])?;
    let mut publisher = ProjectPublisher::new();
    let snapshot = publisher.publish_initial(bundle)?;
    Ok(snapshot.open_view())
}

fn api_record(key: &str) -> TestResult<ReferenceRecord> {
    Ok(ReferenceRecord::new(
        key,
        ReferenceRecordKind::Api,
        "fixture-api-contract",
        vec!["source:fixture:api".to_owned()],
        Vec::new(),
    )?)
}

fn restriction_record(include_facet: bool) -> TestResult<ReferenceRecord> {
    let restrictions = if include_facet {
        vec![RestrictionFacet::new(
            "secret.return",
            RestrictionState::Restricted,
            vec!["evidence:fixture:secret-return".to_owned()],
        )?]
    } else {
        Vec::new()
    };
    Ok(ReferenceRecord::new(
        SECRET_ENTITY,
        ReferenceRecordKind::Restriction,
        SECRET_PAYLOAD,
        vec!["source:fixture:restriction".to_owned()],
        restrictions,
    )?)
}

fn reference(
    project: &ProjectView,
    api_coverage: ReferenceCoverageStatus,
    secret_coverage: ReferenceCoverageStatus,
    include_secret_facet: bool,
    secret_conflict: bool,
) -> TestResult<ReferenceView> {
    let api = ReferencePartition::new(
        API_PARTITION,
        api_coverage,
        vec![
            api_record("function:C_E0Fixture.KnownApi")?,
            api_record(SECRET_ENTITY)?,
        ],
    )?;
    let restriction = ReferencePartition::new(
        SECRET_PARTITION,
        secret_coverage,
        vec![restriction_record(include_secret_facet)?],
    )?;
    let conflicts = if secret_conflict {
        vec![ReferenceConflict::new(
            SECRET_PARTITION,
            SECRET_ENTITY,
            vec![
                format!("sha256:{}", "a".repeat(64)),
                format!("sha256:{}", "b".repeat(64)),
            ],
            vec!["source:fixture:conflict".to_owned()],
        )?]
    } else {
        Vec::new()
    };
    Ok(ReferenceView::new(
        project.configuration().reference_generation().to_string(),
        vec![api, restriction],
        conflicts,
    )?)
}

fn run(
    project: &ProjectView,
    reference: &ReferenceView,
    cancelled: &AtomicBool,
) -> TestResult<wow_rules::RuleExecutionReport> {
    let registry = RuleRegistry::e0()?;
    let policy = RuleFixturePolicy::e0()?;
    let budget = RuleExecutionBudget::fixture_e0()?;
    let context =
        RuleExecutionContext::new(&registry, &policy, project, reference, budget, cancelled);
    Ok(execute_e0(&context, &RuleScope::all())?)
}

fn clean_record(outcome: &RuleEvaluationOutcome) -> Option<&CleanEvaluationRecord> {
    match outcome {
        RuleEvaluationOutcome::EvaluatedClean { record } => Some(record),
        _ => None,
    }
}

#[test]
fn registry_is_exactly_the_two_closed_e0_rules() -> TestResult {
    let registry = RuleRegistry::e0()?;
    assert_eq!(registry.descriptors().len(), 2);
    assert_eq!(
        registry.descriptors()[0].rule_id().as_str(),
        API_EXISTS_RULE
    );
    assert_eq!(
        registry.descriptors()[1].rule_id().as_str(),
        SECRET_LOCAL_RULE
    );
    assert_eq!(registry, RuleRegistry::e0()?);
    Ok(())
}

#[test]
fn complete_reference_produces_one_api_and_three_secret_findings() -> TestResult {
    let project = project()?;
    let reference = reference(
        &project,
        ReferenceCoverageStatus::Complete,
        ReferenceCoverageStatus::Complete,
        true,
        false,
    )?;
    let cancelled = AtomicBool::new(false);
    let report = run(&project, &reference, &cancelled)?;
    assert_eq!(report.finding_count(), 4);
    assert_eq!(report, run(&project, &reference, &cancelled)?);
    assert_eq!(
        serde_json::to_vec(&report)?,
        serde_json::to_vec(&run(&project, &reference, &cancelled)?)?
    );

    let mut api_findings = 0;
    let mut secret_findings = Vec::new();
    let mut guarded_clean = 0;
    for evaluation in report.evaluations() {
        match evaluation.outcome() {
            RuleEvaluationOutcome::Findings { result }
                if evaluation.rule_id().as_str() == API_EXISTS_RULE =>
            {
                api_findings += result.findings().len();
                assert!(!result.coverage_ids().is_empty());
                assert_eq!(
                    result.findings()[0].finding_code().as_str(),
                    "wow.api.missing"
                );
            }
            RuleEvaluationOutcome::Findings { result }
                if evaluation.rule_id().as_str() == SECRET_LOCAL_RULE =>
            {
                secret_findings.push(result.guard_classification().ok_or("guard class")?);
                assert!(!result.coverage_ids().is_empty());
                assert_eq!(
                    result.findings()[0].finding_code().as_str(),
                    "wow.secret.unsafe_local_operation"
                );
            }
            RuleEvaluationOutcome::EvaluatedClean { record }
                if evaluation.rule_id().as_str() == SECRET_LOCAL_RULE
                    && record.guard_classification()
                        == Some(RuleGuardClassification::DominatingExactValue) =>
            {
                guarded_clean += 1;
            }
            _ => {}
        }
    }
    secret_findings.sort();
    assert_eq!(api_findings, 1);
    assert_eq!(guarded_clean, 1);
    assert_eq!(
        secret_findings,
        vec![
            RuleGuardClassification::Absent,
            RuleGuardClassification::AfterUse,
            RuleGuardClassification::DifferentValue,
        ]
    );
    Ok(())
}

#[test]
fn partial_api_partition_blocks_only_negative_authority() -> TestResult {
    let project = project()?;
    let reference = reference(
        &project,
        ReferenceCoverageStatus::Partial,
        ReferenceCoverageStatus::Complete,
        true,
        false,
    )?;
    let report = run(&project, &reference, &AtomicBool::new(false))?;
    assert_eq!(
        report
            .evaluations()
            .iter()
            .filter(|evaluation| {
                evaluation.rule_id().as_str() == API_EXISTS_RULE
                    && matches!(evaluation.outcome(), RuleEvaluationOutcome::Findings { .. })
            })
            .count(),
        0
    );
    let blocked = report
        .evaluations()
        .iter()
        .filter_map(|evaluation| match evaluation.outcome() {
            RuleEvaluationOutcome::NotEvaluated { detail }
                if evaluation.rule_id().as_str() == API_EXISTS_RULE =>
            {
                Some(detail)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocked.len(), 1);
    assert!(
        blocked[0]
            .blockers()
            .contains(&RuleBlockerKind::PartialCoverage)
    );
    Ok(())
}

#[test]
fn restriction_conflict_never_becomes_secret_finding_or_clean() -> TestResult {
    let project = project()?;
    let reference = reference(
        &project,
        ReferenceCoverageStatus::Complete,
        ReferenceCoverageStatus::Complete,
        true,
        true,
    )?;
    let report = run(&project, &reference, &AtomicBool::new(false))?;
    let secret = report
        .evaluations()
        .iter()
        .filter(|evaluation| evaluation.rule_id().as_str() == SECRET_LOCAL_RULE)
        .collect::<Vec<_>>();
    assert_eq!(secret.len(), 4);
    assert!(secret.iter().all(|evaluation| match evaluation.outcome() {
        RuleEvaluationOutcome::NotEvaluated { detail } => {
            detail
                .blockers()
                .contains(&RuleBlockerKind::ReferenceConflict)
        }
        _ => false,
    }));
    Ok(())
}

#[test]
fn authoritative_no_facet_is_narrowly_clean() -> TestResult {
    let project = project()?;
    let reference = reference(
        &project,
        ReferenceCoverageStatus::Complete,
        ReferenceCoverageStatus::Complete,
        false,
        false,
    )?;
    let report = run(&project, &reference, &AtomicBool::new(false))?;
    assert_eq!(report.finding_count(), 1);
    let clean_secret = report
        .evaluations()
        .iter()
        .filter(|evaluation| evaluation.rule_id().as_str() == SECRET_LOCAL_RULE)
        .filter_map(|evaluation| clean_record(evaluation.outcome()))
        .count();
    assert_eq!(clean_secret, 4);
    Ok(())
}

#[test]
fn preflight_cancellation_is_an_explicit_two_rule_report() -> TestResult {
    let project = project()?;
    let reference = reference(
        &project,
        ReferenceCoverageStatus::Complete,
        ReferenceCoverageStatus::Complete,
        true,
        false,
    )?;
    let report = run(&project, &reference, &AtomicBool::new(true))?;
    assert_eq!(report.evaluations().len(), 2);
    assert!(
        report
            .evaluations()
            .iter()
            .all(|evaluation| matches!(evaluation.outcome(), RuleEvaluationOutcome::Cancelled))
    );
    Ok(())
}
