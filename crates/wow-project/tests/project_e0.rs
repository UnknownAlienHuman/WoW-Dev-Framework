use std::error::Error;
use std::sync::Arc;

use wow_core::{
    CanonicalResult, ContentDigest, ProfileId, ProfileIdentity, ProfileIdentityBuilder,
    ProfileKind, ReferenceGenerationId, SchemaId, SchemaVersionEntry, SourceKind,
    SourceLogicalSnapshot, ToolVersion,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyReferenceResolution, EmmySyntaxDiagnosticKind, LuaWorkspaceFileInput, LuaWorkspaceLimits,
    LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy, ProjectConfiguration,
    ProjectConfigurationBuilder, ProjectErrorCode, ProjectFileId, ProjectFileOperation,
    ProjectFileRole, ProjectId, ProjectInputBundle, ProjectInputFile, ProjectInputInventory,
    ProjectKind, ProjectLanguageKind, ProjectPublisher, ProjectSourceOriginId,
    ProjectUpdateOutcome, ProjectUpdateRequest, ProjectWorkspaceId,
};

type TestResult = Result<(), Box<dyn Error>>;

fn test_error(message: impl Into<String>) -> std::io::Error {
    std::io::Error::other(message.into())
}

const LIBRARY_SOURCE: &str = "---@meta _\n---@class C_E0Fixture\n---@field KnownApi fun(value: string): boolean\n---@field SecretText fun(): string\nC_E0Fixture = {}\n\n---@param value any\n---@return boolean accessible\nfunction canaccessvalue(value) end\n";
const CLEAN_SOURCE: &str = "local accepted = C_E0Fixture.KnownApi(\"ok\")\nreturn accepted\n";
const GENERIC_ERROR_SOURCE: &str = "---@type string\nlocal value = 42\nreturn value\n";
const MISSING_API_SOURCE: &str = "return C_E0Fixture.RemovedApi()\n";
const SECRET_LOCAL_SOURCE: &str = "local function unsafe_concat()\n    local text = C_E0Fixture.SecretText()\n    return text .. \"!\"\nend\n\nlocal function guarded_concat()\n    local text = C_E0Fixture.SecretText()\n    if canaccessvalue(text) then\n        return text .. \"!\"\n    end\nend\n\nlocal function guard_after_use()\n    local text = C_E0Fixture.SecretText()\n    local rendered = text .. \"!\"\n    if canaccessvalue(text) then\n        return rendered\n    end\nend\n\nlocal function different_value_guard()\n    local text = C_E0Fixture.SecretText()\n    local ordinary = \"ok\"\n    if canaccessvalue(ordinary) then\n        return text .. \"!\"\n    end\nend\n\nreturn unsafe_concat, guarded_concat, guard_after_use, different_value_guard\n";

fn backend() -> Result<EmmyBackendIdentity, Box<dyn Error>> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?)
}

fn fixture_profile() -> Result<ProfileIdentity, Box<dyn Error>> {
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
    .fixture_scope("wow-project-e0-d-fixture")
    .build()?)
}

fn configuration_with(
    capability_policy: ProjectCapabilityPolicy,
    budget_policy: ProjectBudgetPolicy,
) -> Result<ProjectConfiguration, Box<dyn Error>> {
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
        ProjectId::new("fixture-project-e0-v1")?,
        ProjectKind::Fixture,
        fixture_profile()?,
        ReferenceGenerationId::derive(&"fixture-reference-e0")?,
        analyzer_binding,
    )
    .workspace_id(ProjectWorkspaceId::new("workspace:main:e0")?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:fixture-project-e0-v1",
    )?)
    .logical_root("fixtures/e0/project/main")
    .capability_policy(capability_policy)
    .budget_policy(budget_policy)
    .build()?)
}

fn strict_configuration() -> Result<ProjectConfiguration, Box<dyn Error>> {
    configuration_with(
        ProjectCapabilityPolicy::strict_e0()?,
        ProjectBudgetPolicy::fixture_e0()?,
    )
}

fn degraded_configuration() -> Result<ProjectConfiguration, Box<dyn Error>> {
    configuration_with(
        ProjectCapabilityPolicy::degraded_e0()?,
        ProjectBudgetPolicy::fixture_e0()?,
    )
}

fn library_for(
    selected_backend: EmmyBackendIdentity,
) -> Result<LuaWorkspaceSnapshot, Box<dyn Error>> {
    Ok(LuaWorkspaceSnapshot::build(
        selected_backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            LIBRARY_SOURCE,
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?)
}

fn baseline_files() -> Result<Vec<ProjectInputFile>, Box<dyn Error>> {
    Ok(vec![
        ProjectInputFile::declared(
            "main/clean.lua",
            CLEAN_SOURCE,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            Some("wow-emmy/workspace-fixture:main/clean.lua"),
        )?,
        ProjectInputFile::declared(
            "main/generic-error.lua",
            GENERIC_ERROR_SOURCE,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            Some("wow-emmy/workspace-fixture:main/generic-error.lua"),
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
    ])
}

fn bundle(
    configuration: ProjectConfiguration,
    mut files: Vec<ProjectInputFile>,
    reverse: bool,
) -> Result<ProjectInputBundle, Box<dyn Error>> {
    if reverse {
        files.reverse();
    }
    let library = library_for(configuration.analyzer_binding().backend().clone())?;
    Ok(ProjectInputBundle::closed(
        configuration,
        files,
        vec![library],
    )?)
}

fn publish_strict() -> Result<(ProjectPublisher, Arc<wow_project::ProjectSnapshot>), Box<dyn Error>>
{
    let configuration = strict_configuration()?;
    let mut publisher = ProjectPublisher::new();
    let snapshot = publisher.publish_initial(bundle(configuration, baseline_files()?, false)?)?;
    Ok((publisher, snapshot))
}

fn record_by_path<'a>(
    snapshot: &'a wow_project::ProjectSnapshot,
    path: &str,
) -> Result<&'a wow_project::ProjectFileRecord, Box<dyn Error>> {
    Ok(snapshot
        .source_registry()
        .file_by_path(path)?
        .ok_or_else(|| test_error(format!("missing project file {path}")))?)
}

#[test]
fn baseline_publication_binds_one_exact_generation() -> TestResult {
    let (publisher, snapshot) = publish_strict()?;
    snapshot.validate()?;
    assert_eq!(snapshot.file_manifest().len(), 4);
    assert_eq!(
        snapshot.generation_context().project_generation(),
        Some(snapshot.project_generation())
    );
    assert_eq!(
        snapshot.analyzer_binding().main_workspace().files().len(),
        snapshot.file_manifest().len()
    );
    assert!(
        snapshot
            .project_coverage_records()
            .iter()
            .all(|record| record.context_id() == snapshot.generation_context().context_id())
    );
    assert_eq!(snapshot.deferred_capabilities().len(), 8);

    let generic = record_by_path(&snapshot, "main/generic-error.lua")?;
    let view = publisher.open_current()?;
    let generic_diagnostics = view.generic_diagnostics_for_file(generic.file_id());
    assert_eq!(generic_diagnostics.len(), 1);
    assert_eq!(
        generic_diagnostics[0].kind(),
        EmmySyntaxDiagnosticKind::AssignmentTypeMismatch
    );

    let missing = record_by_path(&snapshot, "main/missing-api.lua")?;
    let removed = view
        .member_references_for_file(missing.file_id())
        .into_iter()
        .find(|fact| fact.member() == "RemovedApi")
        .ok_or_else(|| test_error("missing RemovedApi reference fact"))?;
    assert_eq!(removed.receiver(), "C_E0Fixture");
    assert_eq!(removed.resolution(), EmmyReferenceResolution::Unresolved);

    let secret = record_by_path(&snapshot, "main/secret-local.lua")?;
    let flow = view.local_flow_report();
    let secret_bindings = view.local_bindings_for_file(secret.file_id());
    assert!(!secret_bindings.is_empty());
    assert_eq!(flow.operations().len(), 4);
    assert_eq!(flow.guards().len(), 3);
    assert_eq!(flow.control_flow().len(), 1);
    assert!(secret_bindings.iter().all(|binding| {
        binding
            .initializer_member()
            .is_none_or(|member| member == "SecretText")
    }));

    for record in snapshot.file_manifest() {
        snapshot
            .source_registry()
            .validate_source_handle(record.source_handle_base())?;
        assert_eq!(record.project_generation(), snapshot.project_generation());
    }
    Ok(())
}

#[test]
fn shuffled_input_order_is_identity_invariant() -> TestResult {
    let configuration = strict_configuration()?;
    let mut left = ProjectPublisher::new();
    let mut right = ProjectPublisher::new();
    let left_snapshot =
        left.publish_initial(bundle(configuration.clone(), baseline_files()?, false)?)?;
    let right_snapshot = right.publish_initial(bundle(configuration, baseline_files()?, true)?)?;
    assert_eq!(
        left_snapshot.project_generation(),
        right_snapshot.project_generation()
    );
    assert_eq!(left_snapshot.snapshot_id(), right_snapshot.snapshot_id());
    assert_eq!(
        left_snapshot.canonical_snapshot_digest(),
        right_snapshot.canonical_snapshot_digest()
    );
    assert_eq!(
        left_snapshot.analyzer_binding().analyzer_snapshot_id(),
        right_snapshot.analyzer_binding().analyzer_snapshot_id()
    );
    Ok(())
}

#[test]
fn no_change_preserves_the_exact_arc_and_analyzer_snapshot() -> TestResult {
    let (mut publisher, current) = publish_strict()?;
    let request = ProjectUpdateRequest::new(current.configuration().clone(), Vec::new())
        .expected_generation(current.project_generation())
        .expected_snapshot_digest(current.canonical_snapshot_digest());
    let outcome = publisher.apply_update(request)?;
    assert!(!outcome.changed());
    assert!(matches!(&outcome, ProjectUpdateOutcome::NoChange(_)));
    assert!(Arc::ptr_eq(&current, outcome.snapshot()));
    assert_eq!(
        current.analyzer_binding().analyzer_snapshot_id(),
        outcome.snapshot().analyzer_binding().analyzer_snapshot_id()
    );
    Ok(())
}

#[test]
fn successful_update_publishes_atomically_and_keeps_old_view_immutable() -> TestResult {
    let (mut publisher, old_snapshot) = publish_strict()?;
    let old_view = old_snapshot.open_view();
    let clean = record_by_path(&old_snapshot, "main/clean.lua")?;
    let operation = ProjectFileOperation::update(
        clean.file_id().clone(),
        clean.content_digest(),
        "local accepted = C_E0Fixture.KnownApi(\"changed\")\nreturn accepted\n",
    );
    let request = ProjectUpdateRequest::new(old_snapshot.configuration().clone(), vec![operation])
        .expected_generation(old_snapshot.project_generation())
        .expected_snapshot_digest(old_snapshot.canonical_snapshot_digest());
    let outcome = publisher.apply_update(request)?;
    assert!(outcome.changed());
    let new_snapshot = outcome.snapshot();
    assert_ne!(
        old_snapshot.project_generation(),
        new_snapshot.project_generation()
    );
    assert_ne!(old_snapshot.snapshot_id(), new_snapshot.snapshot_id());
    assert_eq!(
        old_view.project_generation(),
        old_snapshot.project_generation()
    );
    assert_eq!(old_view.snapshot_id(), old_snapshot.snapshot_id());
    assert!(
        new_snapshot
            .source_registry()
            .validate_source_handle(clean.source_handle_base())
            .is_err()
    );
    assert!(Arc::ptr_eq(
        publisher
            .current_snapshot()
            .ok_or_else(|| test_error("missing current snapshot"))?,
        new_snapshot
    ));
    Ok(())
}

#[test]
fn stale_transaction_guards_fail_before_publication() -> TestResult {
    let (mut publisher, current) = publish_strict()?;
    let wrong_generation = wow_core::ProjectGenerationId::derive(&"wrong-project-generation")?;
    let request = ProjectUpdateRequest::new(current.configuration().clone(), Vec::new())
        .expected_generation(wrong_generation);
    let error = publisher
        .apply_update(request)
        .err()
        .ok_or_else(|| test_error("expected stale generation error"))?;
    assert_eq!(error.code(), ProjectErrorCode::ExpectedGenerationMismatch);
    assert_eq!(
        error.current_generation(),
        Some(current.project_generation())
    );
    assert!(Arc::ptr_eq(
        publisher
            .current_snapshot()
            .ok_or_else(|| test_error("missing current snapshot"))?,
        &current
    ));

    let wrong_digest = ContentDigest::<CanonicalResult>::from_bytes([9_u8; 32]);
    let request = ProjectUpdateRequest::new(current.configuration().clone(), Vec::new())
        .expected_snapshot_digest(wrong_digest);
    let error = publisher
        .apply_update(request)
        .err()
        .ok_or_else(|| test_error("expected stale digest error"))?;
    assert_eq!(
        error.code(),
        ProjectErrorCode::ExpectedSnapshotDigestMismatch
    );
    assert_eq!(
        error.current_snapshot_digest(),
        Some(current.canonical_snapshot_digest())
    );
    assert!(Arc::ptr_eq(
        publisher
            .last_known_good()
            .ok_or_else(|| test_error("missing LKG"))?,
        &current
    ));
    Ok(())
}

#[test]
fn stale_file_digest_and_conflicting_operations_do_not_move_current() -> TestResult {
    let (mut publisher, current) = publish_strict()?;
    let clean = record_by_path(&current, "main/clean.lua")?;
    let stale = ProjectFileOperation::update(
        clean.file_id().clone(),
        ContentDigest::from_bytes([8_u8; 32]),
        CLEAN_SOURCE,
    );
    let error = publisher
        .apply_update(ProjectUpdateRequest::new(
            current.configuration().clone(),
            vec![stale],
        ))
        .err()
        .ok_or_else(|| test_error("expected stale file digest error"))?;
    assert_eq!(error.code(), ProjectErrorCode::ExpectedFileDigestMismatch);

    let first = ProjectFileOperation::update(
        clean.file_id().clone(),
        clean.content_digest(),
        CLEAN_SOURCE,
    );
    let second = ProjectFileOperation::remove(clean.file_id().clone(), clean.content_digest());
    let error = publisher
        .apply_update(ProjectUpdateRequest::new(
            current.configuration().clone(),
            vec![first, second],
        ))
        .err()
        .ok_or_else(|| test_error("expected conflicting-operation error"))?;
    assert_eq!(error.code(), ProjectErrorCode::ConflictingOperations);
    assert!(Arc::ptr_eq(
        publisher
            .current_snapshot()
            .ok_or_else(|| test_error("missing current snapshot"))?,
        &current
    ));
    Ok(())
}

#[test]
fn strict_failure_retains_lkg_while_degraded_policy_publishes_failed_file_coverage() -> TestResult {
    let (mut strict_publisher, strict_current) = publish_strict()?;
    let clean = record_by_path(&strict_current, "main/clean.lua")?;
    let malformed =
        ProjectFileOperation::update(clean.file_id().clone(), clean.content_digest(), "local =\n");
    let error = strict_publisher
        .apply_update(ProjectUpdateRequest::new(
            strict_current.configuration().clone(),
            vec![malformed],
        ))
        .err()
        .ok_or_else(|| test_error("expected strict analyzer failure"))?;
    assert_eq!(
        error.code(),
        ProjectErrorCode::MandatoryCapabilityUnavailable
    );
    assert!(Arc::ptr_eq(
        strict_publisher
            .last_known_good()
            .ok_or_else(|| test_error("missing strict LKG"))?,
        &strict_current
    ));

    let configuration = degraded_configuration()?;
    let mut files = baseline_files()?;
    let clean_input = files
        .iter_mut()
        .find(|file| file.relative_path().as_str() == "main/clean.lua")
        .ok_or_else(|| test_error("missing clean input"))?;
    *clean_input = ProjectInputFile::new("main/clean.lua", "local =\n")?;
    let mut degraded_publisher = ProjectPublisher::new();
    let degraded = degraded_publisher.publish_initial(bundle(configuration, files, false)?)?;
    let degraded_clean = record_by_path(&degraded, "main/clean.lua")?;
    assert!(
        !degraded
            .analyzer_binding()
            .file_facts_complete(degraded_clean.file_id())
    );
    assert!(degraded.project_coverage_records().iter().any(|coverage| {
        coverage.capability_id().as_str() == "project.analyzer.facts.available"
            && coverage.status() == wow_core::CoverageStatus::Failed
    }));
    Ok(())
}

#[test]
fn add_remove_order_is_canonical() -> TestResult {
    let configuration = strict_configuration()?;
    let mut left = ProjectPublisher::new();
    let mut right = ProjectPublisher::new();
    let left_current =
        left.publish_initial(bundle(configuration.clone(), baseline_files()?, false)?)?;
    let right_current = right.publish_initial(bundle(configuration, baseline_files()?, true)?)?;
    let clean_left = record_by_path(&left_current, "main/clean.lua")?;
    let clean_right = record_by_path(&right_current, "main/clean.lua")?;
    let added_left =
        ProjectInputFile::new("main/added.lua", "return C_E0Fixture.KnownApi(\"new\")\n")?;
    let added_right = added_left.clone();
    let left_ops = vec![
        ProjectFileOperation::add(added_left),
        ProjectFileOperation::remove(clean_left.file_id().clone(), clean_left.content_digest()),
    ];
    let right_ops = vec![
        ProjectFileOperation::remove(clean_right.file_id().clone(), clean_right.content_digest()),
        ProjectFileOperation::add(added_right),
    ];
    let left_outcome = left.apply_update(ProjectUpdateRequest::new(
        left_current.configuration().clone(),
        left_ops,
    ))?;
    let right_outcome = right.apply_update(ProjectUpdateRequest::new(
        right_current.configuration().clone(),
        right_ops,
    ))?;
    assert_eq!(
        left_outcome.snapshot().project_generation(),
        right_outcome.snapshot().project_generation()
    );
    assert_eq!(
        left_outcome.snapshot().snapshot_id(),
        right_outcome.snapshot().snapshot_id()
    );
    Ok(())
}

#[test]
fn inventory_rejects_missing_undeclared_case_collision_role_language_and_limits() -> TestResult {
    let configuration = strict_configuration()?;
    let one = ProjectInputFile::new("main/one.lua", "return 1\n")?;
    let missing = ProjectInputInventory::build(
        &configuration,
        vec!["main/one.lua".into(), "main/two.lua".into()],
        vec![one.clone()],
    )
    .err()
    .ok_or_else(|| test_error("expected missing-declared-file error"))?;
    assert_eq!(missing.code(), ProjectErrorCode::MissingDeclaredFile);

    let undeclared = ProjectInputInventory::build(
        &configuration,
        vec!["main/one.lua".into()],
        vec![
            one.clone(),
            ProjectInputFile::new("main/two.lua", "return 2\n")?,
        ],
    )
    .err()
    .ok_or_else(|| test_error("expected undeclared-file error"))?;
    assert_eq!(undeclared.code(), ProjectErrorCode::UndeclaredFile);

    let upper = ProjectInputFile::new("main/A.lua", "return 1\n")?;
    let lower = ProjectInputFile::new("main/a.lua", "return 2\n")?;
    let collision = ProjectInputInventory::build(
        &configuration,
        vec!["main/A.lua".into(), "main/a.lua".into()],
        vec![upper, lower],
    )
    .err()
    .ok_or_else(|| test_error("expected path case collision"))?;
    assert_eq!(collision.code(), ProjectErrorCode::FileCaseCollision);

    let library_role = ProjectInputFile::declared(
        "main/library.lua",
        "return 1\n",
        ProjectLanguageKind::Lua,
        ProjectFileRole::Library,
        None::<String>,
    )?;
    let error = ProjectInputInventory::build(
        &configuration,
        vec!["main/library.lua".into()],
        vec![library_role],
    )
    .err()
    .ok_or_else(|| test_error("expected Library-role rejection"))?;
    assert_eq!(error.code(), ProjectErrorCode::InvalidFileRole);

    let other_language = ProjectInputFile::declared(
        "main/other.lua",
        "return 1\n",
        ProjectLanguageKind::Other,
        ProjectFileRole::FirstPartyMain,
        None::<String>,
    )?;
    let error = ProjectInputInventory::build(
        &configuration,
        vec!["main/other.lua".into()],
        vec![other_language],
    )
    .err()
    .ok_or_else(|| test_error("expected language rejection"))?;
    assert_eq!(error.code(), ProjectErrorCode::InvalidFileLanguage);

    let tiny_budget = ProjectBudgetPolicy::new(1, 8, 8, 1, 1, 1, 1)?;
    let tiny_configuration =
        configuration_with(ProjectCapabilityPolicy::strict_e0()?, tiny_budget)?;
    let oversized = ProjectInputFile::new("main/large.lua", "return 123456789\n")?;
    let error = ProjectInputInventory::build(
        &tiny_configuration,
        vec!["main/large.lua".into()],
        vec![oversized],
    )
    .err()
    .ok_or_else(|| test_error("expected source budget rejection"))?;
    assert_eq!(error.code(), ProjectErrorCode::UpdateBudgetExceeded);
    Ok(())
}

#[test]
fn inventory_rejects_unknown_or_mismatched_fixture_provenance() -> TestResult {
    let configuration = strict_configuration()?;
    let unknown = ProjectInputFile::declared(
        "main/unknown.lua",
        "return 1\n",
        ProjectLanguageKind::Lua,
        ProjectFileRole::FirstPartyMain,
        Some("wow-emmy/workspace-fixture:main/unknown.lua"),
    )?;
    let error = ProjectInputInventory::build(
        &configuration,
        vec!["main/unknown.lua".into()],
        vec![unknown],
    )
    .err()
    .ok_or_else(|| test_error("expected unknown fixture-member rejection"))?;
    assert_eq!(error.code(), ProjectErrorCode::InvalidInputInventory);

    let mismatched = ProjectInputFile::declared(
        "main/clean.lua",
        CLEAN_SOURCE,
        ProjectLanguageKind::Lua,
        ProjectFileRole::FirstPartyMain,
        Some("wow-emmy/workspace-fixture:main/missing-api.lua"),
    )?;
    let error = ProjectInputInventory::build(
        &configuration,
        vec!["main/clean.lua".into()],
        vec![mismatched],
    )
    .err()
    .ok_or_else(|| test_error("expected mismatched fixture-member rejection"))?;
    assert_eq!(error.code(), ProjectErrorCode::InvalidInputInventory);
    Ok(())
}

#[test]
fn mismatched_library_backend_is_rejected_without_publication() -> TestResult {
    let configuration = strict_configuration()?;
    let different_backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "3".repeat(64)),
        configuration
            .analyzer_binding()
            .backend()
            .compatibility_report_sha256(),
    )?;
    let library = library_for(different_backend)?;
    let mut publisher = ProjectPublisher::new();
    let error = publisher
        .publish_initial(ProjectInputBundle::closed(
            configuration,
            baseline_files()?,
            vec![library],
        )?)
        .err()
        .ok_or_else(|| test_error("expected Library backend mismatch"))?;
    assert_eq!(error.code(), ProjectErrorCode::AnalyzerSnapshotMismatch);
    assert!(publisher.current_snapshot().is_none());
    assert!(publisher.last_known_good().is_none());
    Ok(())
}

#[test]
fn public_identity_and_errors_do_not_expose_source_or_host_paths() -> TestResult {
    let (mut publisher, snapshot) = publish_strict()?;
    let public_json = serde_json::to_string(&(
        snapshot.configuration(),
        snapshot.generation_candidate(),
        snapshot.file_manifest(),
        snapshot.analyzer_binding().syntax_report(),
        snapshot.analyzer_binding().member_call_report(),
        snapshot.analyzer_binding().local_flow_report(),
    ))?;
    assert!(!public_json.contains("/tmp/"));
    assert!(!public_json.contains("C:\\"));
    assert!(!public_json.contains(SECRET_LOCAL_SOURCE));

    let clean = record_by_path(&snapshot, "main/clean.lua")?;
    let error = publisher
        .apply_update(ProjectUpdateRequest::new(
            snapshot.configuration().clone(),
            vec![ProjectFileOperation::update(
                clean.file_id().clone(),
                ContentDigest::from_bytes([4_u8; 32]),
                "private source must not appear",
            )],
        ))
        .err()
        .ok_or_else(|| test_error("expected digest mismatch"))?;
    let rendered = error.to_string();
    assert!(!rendered.contains("private source"));
    assert!(!rendered.contains("/tmp/"));
    Ok(())
}

#[test]
fn canonical_path_and_file_identity_guards_are_strict() -> TestResult {
    assert!(ProjectInputFile::new("main/../escape.lua", "return 1\n").is_err());
    assert!(ProjectInputFile::new("main\\wrong.lua", "return 1\n").is_err());
    assert!(ProjectFileId::parse("project-file:main/../escape.lua").is_err());
    assert!(ProjectWorkspaceId::new("workspace:current:e0").is_err());
    assert!(ProjectSourceOriginId::new("project-origin:latest").is_err());
    Ok(())
}
