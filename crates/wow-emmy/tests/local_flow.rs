use wow_core::SourceSpan;
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyControlFlowRelationKind, EmmyGuardKind, EmmyLocalFlowFileStatus, EmmyOperationKind,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    analyze_local_flow,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn backend() -> TestResult<EmmyBackendIdentity> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        digest('1'),
        digest('2'),
    )?)
}

fn snapshot(
    universe: LuaWorkspaceUniverse,
    inputs: Vec<LuaWorkspaceFileInput>,
) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend()?,
        universe,
        inputs,
        LuaWorkspaceLimits::new(32, 1024, 64 * 1024, 512 * 1024)?,
    )?)
}

fn library() -> TestResult<LuaWorkspaceSnapshot> {
    snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            concat!(
                "---@meta _\n",
                "---@class C_E0Fixture\n",
                "---@field SecretText fun(): string\n",
                "C_E0Fixture = {}\n",
                "---@param value any\n",
                "---@return boolean\n",
                "function canaccessvalue(value) end\n",
            ),
        )],
    )
}

fn secret_source() -> &'static str {
    concat!(
        "local function unsafe_concat()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    return text .. \"!\"\n",
        "end\n\n",
        "local function guarded_concat()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    if canaccessvalue(text) then\n",
        "        return text .. \"!\"\n",
        "    end\n",
        "end\n\n",
        "local function guard_after_use()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    local rendered = text .. \"!\"\n",
        "    if canaccessvalue(text) then\n",
        "        return rendered\n",
        "    end\n",
        "end\n\n",
        "local function different_value_guard()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    local ordinary = \"ok\"\n",
        "    if canaccessvalue(ordinary) then\n",
        "        return text .. \"!\"\n",
        "    end\n",
        "end\n",
    )
}

fn source_slice(text: &str, span: SourceSpan) -> TestResult<&str> {
    let start = usize::try_from(span.byte_start().ok_or("missing start")?)?;
    let end = usize::try_from(span.byte_end().ok_or("missing end")?)?;
    text.get(start..end)
        .ok_or_else(|| "invalid source span".into())
}

#[test]
fn exact_local_flow_distinguishes_dominating_and_non_dominating_guards() -> TestResult {
    let source = secret_source();
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new("main/secret-local.lua", source)],
    )?;
    let library = library()?;
    let report = analyze_local_flow(&main, &[&library])?;

    assert_eq!(report.main_snapshot_id(), main.snapshot_id());
    assert_eq!(report.files().len(), 1);
    assert_eq!(
        report.files()[0].status(),
        EmmyLocalFlowFileStatus::Complete
    );
    assert_eq!(report.files()[0].operation_count(), 4);
    assert_eq!(report.files()[0].guard_count(), 3);
    assert_eq!(report.files()[0].control_flow_count(), 1);
    assert_eq!(
        report
            .bindings()
            .iter()
            .filter(|fact| fact.name() == "text")
            .count(),
        4
    );
    assert_eq!(
        report
            .bindings()
            .iter()
            .filter(|fact| fact.initializer_member() == Some("SecretText"))
            .count(),
        4
    );
    assert!(
        report
            .operations()
            .iter()
            .all(|fact| fact.kind() == EmmyOperationKind::Concatenation)
    );
    assert!(
        report
            .guards()
            .iter()
            .all(|fact| fact.kind() == EmmyGuardKind::AccessSingle)
    );

    let relation = report.control_flow().first().ok_or("dominance relation")?;
    assert_eq!(relation.relation(), EmmyControlFlowRelationKind::Dominates);
    let guard = report
        .guards()
        .iter()
        .find(|fact| fact.fact_id() == relation.guard_fact_id())
        .ok_or("linked guard")?;
    let operation = report
        .operations()
        .iter()
        .find(|fact| fact.fact_id() == relation.operation_fact_id())
        .ok_or("linked operation")?;
    assert_eq!(guard.guarded_name(), "text");
    assert_eq!(
        source_slice(source, operation.operation_span())?,
        "text .. \"!\""
    );
    assert!(source_slice(source, guard.guard_span())?.starts_with("canaccessvalue(text)"));

    let serialized = serde_json::to_string(&report)?;
    assert!(!serialized.contains("secret_value"));
    assert!(!serialized.contains("operation_is_safe"));
    assert!(!serialized.contains(std::env::temp_dir().to_string_lossy().as_ref()));
    Ok(())
}

#[test]
fn local_flow_is_independent_of_library_input_order() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/secret-local.lua",
            secret_source(),
        )],
    )?;
    let fixture = library()?;
    let unrelated = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/Other.lua",
            "---@meta _\nOther = {}\n",
        )],
    )?;
    let left = analyze_local_flow(&main, &[&fixture, &unrelated])?;
    let right = analyze_local_flow(&main, &[&unrelated, &fixture])?;
    assert_eq!(left, right);
    assert_eq!(serde_json::to_vec(&left)?, serde_json::to_vec(&right)?);
    Ok(())
}

#[test]
fn malformed_main_has_failed_capability_and_no_facts() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/broken.lua",
            "local function broken(..., value) end\n",
        )],
    )?;
    let report = analyze_local_flow(&main, &[])?;
    assert_eq!(
        report.files()[0].status(),
        EmmyLocalFlowFileStatus::FailedParse
    );
    assert!(report.files()[0].parse_error_count() > 0);
    assert!(report.bindings().is_empty());
    assert!(report.operations().is_empty());
    assert!(report.guards().is_empty());
    assert!(report.control_flow().is_empty());
    Ok(())
}
