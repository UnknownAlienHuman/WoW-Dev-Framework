use wow_core::SourceSpan;
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyFactFileStatus, EmmyMemberCallErrorCode, EmmyReferenceResolution, LuaWorkspaceFileInput,
    LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse, analyze_member_calls,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn backend(version: &str) -> TestResult<EmmyBackendIdentity> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(version),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        digest('1'),
        digest('2'),
    )?)
}

fn snapshot(
    universe: LuaWorkspaceUniverse,
    version: &str,
    inputs: Vec<LuaWorkspaceFileInput>,
) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend(version)?,
        universe,
        inputs,
        LuaWorkspaceLimits::new(32, 1024, 64 * 1024, 512 * 1024)?,
    )?)
}

fn fixture_library() -> TestResult<LuaWorkspaceSnapshot> {
    snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            "---@meta _\n---@class C_E0Fixture\n---@field KnownApi fun(value: string): string\nC_E0Fixture = {}\n",
        )],
    )
}

fn source_slice(text: &str, span: SourceSpan) -> TestResult<&str> {
    let start = usize::try_from(span.byte_start().ok_or("missing byte start")?)?;
    let end = usize::try_from(span.byte_end().ok_or("missing byte end")?)?;
    text.get(start..end)
        .ok_or_else(|| "invalid source span".into())
}

#[test]
fn direct_member_calls_link_exact_reference_and_call_facts() -> TestResult {
    let source = concat!(
        "local known = C_E0Fixture.KnownApi(\"ok\")\n",
        "local missing = C_E0Fixture.RemovedApi()\n",
    );
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new("main/calls.lua", source)],
    )?;
    let library = fixture_library()?;
    let report = analyze_member_calls(&main, &[&library])?;

    assert_eq!(report.main_snapshot_id(), main.snapshot_id());
    assert_eq!(
        report.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );
    assert_eq!(report.files().len(), 1);
    assert_eq!(report.files()[0].path(), "main/calls.lua");
    assert_eq!(report.files()[0].status(), EmmyFactFileStatus::Complete);
    assert_eq!(report.files()[0].parse_error_count(), 0);
    assert_eq!(report.files()[0].reference_count(), 2);
    assert_eq!(report.files()[0].call_count(), 2);

    let known = report
        .references()
        .iter()
        .find(|fact| fact.member() == "KnownApi")
        .ok_or("KnownApi reference fact")?;
    let missing = report
        .references()
        .iter()
        .find(|fact| fact.member() == "RemovedApi")
        .ok_or("RemovedApi reference fact")?;
    assert_eq!(known.resolution(), EmmyReferenceResolution::Resolved);
    assert_eq!(missing.resolution(), EmmyReferenceResolution::Unresolved);
    assert_eq!(known.receiver(), "C_E0Fixture");
    assert_eq!(missing.receiver(), "C_E0Fixture");
    assert_eq!(source_slice(source, known.receiver_span())?, "C_E0Fixture");
    assert_eq!(source_slice(source, known.member_span())?, "KnownApi");
    assert_eq!(
        source_slice(source, known.reference_span())?,
        "C_E0Fixture.KnownApi"
    );
    assert_eq!(source_slice(source, missing.member_span())?, "RemovedApi");
    assert_eq!(
        source_slice(source, missing.reference_span())?,
        "C_E0Fixture.RemovedApi"
    );

    let known_call = report
        .calls()
        .iter()
        .find(|fact| fact.reference_fact_id() == known.fact_id())
        .ok_or("KnownApi call fact")?;
    let missing_call = report
        .calls()
        .iter()
        .find(|fact| fact.reference_fact_id() == missing.fact_id())
        .ok_or("RemovedApi call fact")?;
    assert_eq!(known_call.argument_count(), Some(1));
    assert_eq!(missing_call.argument_count(), Some(0));
    assert!(!known_call.is_colon_call());
    assert_eq!(
        source_slice(source, known_call.callee_span())?,
        "C_E0Fixture.KnownApi"
    );
    assert_eq!(
        source_slice(source, known_call.call_span())?,
        "C_E0Fixture.KnownApi(\"ok\")"
    );
    assert_eq!(
        source_slice(source, missing_call.call_span())?,
        "C_E0Fixture.RemovedApi()"
    );

    let serialized = serde_json::to_string(&report)?;
    assert!(!serialized.contains(std::env::temp_dir().to_string_lossy().as_ref()));
    assert!(!serialized.contains("API absent"));
    assert!(!serialized.contains("World of Warcraft availability"));
    Ok(())
}

#[test]
fn library_order_does_not_change_fact_or_report_identity() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/calls.lua",
            "return C_E0Fixture.KnownApi(\"ok\")\n",
        )],
    )?;
    let fixture = fixture_library()?;
    let unrelated = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/OtherFixture.lua",
            "---@meta _\n---@class OtherFixture\n---@field Ping fun()\nOtherFixture = {}\n",
        )],
    )?;

    let left = analyze_member_calls(&main, &[&fixture, &unrelated])?;
    let right = analyze_member_calls(&main, &[&unrelated, &fixture])?;
    assert_eq!(left, right);
    assert_eq!(serde_json::to_vec(&left)?, serde_json::to_vec(&right)?);
    Ok(())
}

#[test]
fn broken_library_blocks_resolution_dependent_facts() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/calls.lua",
            "return C_E0Fixture.KnownApi()\n",
        )],
    )?;
    let broken = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/broken.lua",
            "local function broken(..., value) end\n",
        )],
    )?;

    let error = analyze_member_calls(&main, &[&broken])
        .err()
        .ok_or("expected library health failure")?;
    assert_eq!(error.code(), EmmyMemberCallErrorCode::LibraryHealthFailed);
    assert_eq!(error.path(), Some("library/broken.lua"));
    Ok(())
}

#[test]
fn malformed_main_file_reports_failed_capability_without_facts() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/broken.lua",
            "local function broken(..., value) end\n",
        )],
    )?;
    let report = analyze_member_calls(&main, &[])?;
    assert_eq!(report.files().len(), 1);
    assert_eq!(report.files()[0].status(), EmmyFactFileStatus::FailedParse);
    assert!(report.files()[0].parse_error_count() > 0);
    assert!(report.references().is_empty());
    assert!(report.calls().is_empty());
    Ok(())
}

#[test]
fn compiled_adapter_rejects_mismatched_backend_before_semantics() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        "0.25.0",
        vec![LuaWorkspaceFileInput::new(
            "main/clean.lua",
            "return true\n",
        )],
    )?;
    let error = analyze_member_calls(&main, &[])
        .err()
        .ok_or("expected incompatible backend")?;
    assert_eq!(error.code(), EmmyMemberCallErrorCode::IncompatibleBackend);
    assert!(error.path().is_none());
    Ok(())
}
