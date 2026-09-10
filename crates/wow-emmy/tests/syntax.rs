use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyDiagnosticSeverity, EmmySyntaxErrorCode, LuaWorkspaceFileInput, LuaWorkspaceLimits,
    LuaWorkspaceSnapshot, LuaWorkspaceUniverse, analyze_syntax,
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

fn snapshot(inputs: Vec<LuaWorkspaceFileInput>) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend(EMMYLUA_CODE_ANALYSIS_VERSION)?,
        LuaWorkspaceUniverse::Fixture,
        inputs,
        LuaWorkspaceLimits::new(16, 1024, 64 * 1024, 256 * 1024)?,
    )?)
}

#[test]
fn exact_snapshot_produces_clean_and_malformed_file_results() -> TestResult {
    let malformed = "local text = 'é🦀'; local function broken(..., value)\r\nend\r\n";
    let workspace = snapshot(vec![
        LuaWorkspaceFileInput::new("clean.lua", "local value = 1\n"),
        LuaWorkspaceFileInput::new("malformed.lua", malformed),
    ])?;
    let report = analyze_syntax(&workspace)?;
    assert_eq!(report.workspace_snapshot_id(), workspace.snapshot_id());
    assert_eq!(report.files().len(), 2);
    assert_eq!(report.files()[0].path(), "clean.lua");
    assert_eq!(report.files()[0].status(), "complete");
    assert_eq!(report.files()[0].diagnostic_count(), 0);
    assert_eq!(report.files()[1].path(), "malformed.lua");
    assert!(report.files()[1].diagnostic_count() > 0);

    let file = workspace.file("malformed.lua").ok_or("malformed input")?;
    for diagnostic in report.diagnostics() {
        assert_eq!(diagnostic.path(), "malformed.lua");
        assert_eq!(diagnostic.content_sha256(), file.content_sha256());
        assert_eq!(diagnostic.category(), "emmy.generic.fixture_error");
        assert!(matches!(
            diagnostic.upstream_code(),
            "syntax-error" | "doc-syntax-error"
        ));
        assert_eq!(
            diagnostic.normalized_severity(),
            EmmyDiagnosticSeverity::Error
        );
        let start = usize::try_from(diagnostic.span().byte_start().ok_or("range start")?)?;
        let end = usize::try_from(diagnostic.span().byte_end().ok_or("range end")?)?;
        assert!(start <= end && end <= malformed.len());
        assert!(malformed.is_char_boundary(start));
        assert!(malformed.is_char_boundary(end));
    }
    let json = serde_json::to_string(&report)?;
    let temp = std::env::temp_dir();
    assert!(!json.contains(temp.to_string_lossy().as_ref()));
    Ok(())
}

#[test]
fn file_order_does_not_change_report_identity_or_bytes() -> TestResult {
    let first = LuaWorkspaceFileInput::new("a.lua", "local function broken(..., value) end\n");
    let second = LuaWorkspaceFileInput::new("b.lua", "local value = 1\n");
    let left = snapshot(vec![first.clone(), second.clone()])?;
    let right = snapshot(vec![second, first])?;
    assert_eq!(left.snapshot_id(), right.snapshot_id());
    let left_report = analyze_syntax(&left)?;
    let right_report = analyze_syntax(&right)?;
    assert_eq!(left_report, right_report);
    assert_eq!(
        serde_json::to_vec(&left_report)?,
        serde_json::to_vec(&right_report)?
    );
    Ok(())
}

#[test]
fn compiled_adapter_rejects_a_different_backend_before_analysis() -> TestResult {
    let workspace = LuaWorkspaceSnapshot::build(
        backend("0.25.0")?,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new("clean.lua", "return true\n")],
        LuaWorkspaceLimits::new(4, 256, 4096, 4096)?,
    )?;
    let error = analyze_syntax(&workspace)
        .err()
        .ok_or("expected incompatible backend")?;
    assert_eq!(error.code(), EmmySyntaxErrorCode::IncompatibleBackend);
    assert!(error.path().is_none());
    Ok(())
}
