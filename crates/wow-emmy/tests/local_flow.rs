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

fn library_without_guard() -> TestResult<LuaWorkspaceSnapshot> {
    snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            concat!(
                "---@meta _\n",
                "---@class C_E0Fixture\n",
                "---@field SecretText fun(): string\n",
                "C_E0Fixture = {}\n",
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
fn guard_dominance_requires_one_selected_library_declaration() -> TestResult {
    let unguarded_library = library_without_guard()?;
    let guarded_source = concat!(
        "local text = C_E0Fixture.SecretText()\n",
        "if canaccessvalue(text) then\n",
        "    local rendered = text .. \"!\"\n",
        "end\n",
    );

    let main_defined = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/main-guard.lua",
            format!("function canaccessvalue(value) return true end\n{guarded_source}"),
        )],
    )?;
    let main_report = analyze_local_flow(&main_defined, &[&unguarded_library])?;
    assert_eq!(
        main_report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert!(main_report.guards().is_empty());
    assert!(main_report.control_flow().is_empty());

    let unresolved_main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/unresolved-guard.lua",
            guarded_source,
        )],
    )?;
    let unresolved_report = analyze_local_flow(&unresolved_main, &[&unguarded_library])?;
    assert_eq!(
        unresolved_report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert!(unresolved_report.guards().is_empty());
    assert!(unresolved_report.control_flow().is_empty());

    let ambiguous_library = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![
            LuaWorkspaceFileInput::new(
                "library/C_E0Fixture.lua",
                concat!(
                    "---@meta _\n",
                    "---@class C_E0Fixture\n",
                    "---@field SecretText fun(): string\n",
                    "C_E0Fixture = {}\n",
                ),
            ),
            LuaWorkspaceFileInput::new(
                "library/guard-a.lua",
                "function canaccessvalue(value) end\n",
            ),
            LuaWorkspaceFileInput::new(
                "library/guard-b.lua",
                "function canaccessvalue(value) end\n",
            ),
        ],
    )?;
    let ambiguous_main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/ambiguous-guard.lua",
            guarded_source,
        )],
    )?;
    let ambiguous_report = analyze_local_flow(&ambiguous_main, &[&ambiguous_library])?;
    assert_eq!(
        ambiguous_report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert!(ambiguous_report.guards().is_empty());
    assert!(ambiguous_report.control_flow().is_empty());

    let operation_outside_block = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/outside-guard.lua",
            concat!(
                "function canaccessvalue(value) return true end\n",
                "local text = C_E0Fixture.SecretText()\n",
                "if canaccessvalue(text) then local accepted = true end\n",
                "local rendered = text .. \"!\"\n",
            ),
        )],
    )?;
    let outside_report = analyze_local_flow(&operation_outside_block, &[&unguarded_library])?;
    assert_eq!(
        outside_report.files()[0].status(),
        EmmyLocalFlowFileStatus::Complete
    );
    assert!(outside_report.guards().is_empty());
    assert!(outside_report.control_flow().is_empty());
    Ok(())
}

#[test]
fn concatenation_facts_keep_direct_operands_and_mark_unsupported_result_flow_partial() -> TestResult
{
    let library = library()?;
    let nested_operands = [
        "helper(other)",
        "(function() return other end)",
        "{ value = other }",
    ];
    for (index, nested_operand) in nested_operands.into_iter().enumerate() {
        let source = format!(
            "local text = C_E0Fixture.SecretText()\nlocal other = C_E0Fixture.SecretText()\nlocal rendered = text .. {nested_operand}\n"
        );
        let main = snapshot(
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                format!("main/unsupported-{index}.lua"),
                &source,
            )],
        )?;
        let report = analyze_local_flow(&main, &[&library])?;
        assert_eq!(
            report.files()[0].status(),
            EmmyLocalFlowFileStatus::PartialUnresolvedFlow
        );
        assert_eq!(report.operations().len(), 1);
        assert_eq!(
            report.operations()[0].operand_names().collect::<Vec<_>>(),
            vec!["text"]
        );
        let operation_id = report.operations()[0].fact_id();
        let uses = report
            .uses()
            .iter()
            .filter(|fact| fact.operation_fact_id() == operation_id)
            .collect::<Vec<_>>();
        assert_eq!(uses.len(), 1);
        assert_eq!(uses[0].name(), "text");
        assert_eq!(source_slice(&source, uses[0].use_span())?, "text");
    }

    let helper_only_source = concat!(
        "local function helper(value) return \"constant\" end\n",
        "local text = C_E0Fixture.SecretText()\n",
        "local rendered = helper(text) .. \"!\"\n",
    );
    let helper_only_main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/helper-only.lua",
            helper_only_source,
        )],
    )?;
    let helper_only_report = analyze_local_flow(&helper_only_main, &[&library])?;
    assert_eq!(
        helper_only_report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert!(helper_only_report.operations().is_empty());
    assert!(helper_only_report.uses().is_empty());

    let parenthesized_source = concat!(
        "local text = C_E0Fixture.SecretText()\n",
        "local rendered = ((text)) .. \"!\"\n",
    );
    let parenthesized_main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/parenthesized.lua",
            parenthesized_source,
        )],
    )?;
    let parenthesized_report = analyze_local_flow(&parenthesized_main, &[&library])?;
    assert_eq!(
        parenthesized_report.files()[0].status(),
        EmmyLocalFlowFileStatus::Complete
    );
    assert_eq!(parenthesized_report.operations().len(), 1);
    assert_eq!(
        parenthesized_report.operations()[0]
            .operand_names()
            .collect::<Vec<_>>(),
        vec!["text"]
    );
    assert_eq!(parenthesized_report.uses().len(), 1);
    assert_eq!(
        source_slice(
            parenthesized_source,
            parenthesized_report.uses()[0].use_span()
        )?,
        "text"
    );

    let nested_concat_source = concat!(
        "local text = C_E0Fixture.SecretText()\n",
        "local other = C_E0Fixture.SecretText()\n",
        "local rendered = text .. (other .. \"!\")\n",
    );
    let nested_concat_main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "main/nested-concat.lua",
            nested_concat_source,
        )],
    )?;
    let nested_concat_report = analyze_local_flow(&nested_concat_main, &[&library])?;
    assert_eq!(
        nested_concat_report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert_eq!(nested_concat_report.operations().len(), 2);
    for expected_name in ["text", "other"] {
        let operation = nested_concat_report
            .operations()
            .iter()
            .find(|operation| operation.operand_names().any(|name| name == expected_name))
            .ok_or("expected nested-concatenation operand")?;
        assert_eq!(
            operation.operand_names().collect::<Vec<_>>(),
            vec![expected_name]
        );
        assert_eq!(
            nested_concat_report
                .uses()
                .iter()
                .filter(|fact| fact.operation_fact_id() == operation.fact_id())
                .count(),
            1
        );
    }
    Ok(())
}

#[test]
fn reassignment_after_guard_is_partial_without_dominance() -> TestResult {
    let source = concat!(
        "local function changed_after_guard()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    if canaccessvalue(text) then\n",
        "        text = C_E0Fixture.SecretText()\n",
        "        return text .. \"!\"\n",
        "    end\n",
        "end\n",
    );
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new("main/reassigned.lua", source)],
    )?;
    let library = library()?;
    let report = analyze_local_flow(&main, &[&library])?;

    assert_eq!(
        report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert_eq!(report.files()[0].operation_count(), 1);
    assert_eq!(report.files()[0].guard_count(), 1);
    assert!(report.control_flow().is_empty());
    assert_eq!(report.guards()[0].kind(), EmmyGuardKind::AccessSingle);
    Ok(())
}

#[test]
fn captured_operation_is_partial_without_cross_function_dominance() -> TestResult {
    let source = concat!(
        "local function make_renderer()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    local render\n",
        "    if canaccessvalue(text) then\n",
        "        render = function()\n",
        "            return text .. \"!\"\n",
        "        end\n",
        "    end\n",
        "    text = C_E0Fixture.SecretText()\n",
        "    return render\n",
        "end\n",
    );
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new("main/captured.lua", source)],
    )?;
    let library = library()?;
    let report = analyze_local_flow(&main, &[&library])?;

    assert_eq!(
        report.files()[0].status(),
        EmmyLocalFlowFileStatus::PartialUnresolvedFlow
    );
    assert_eq!(report.files()[0].operation_count(), 1);
    assert_eq!(report.files()[0].guard_count(), 1);
    assert!(report.control_flow().is_empty());
    Ok(())
}

#[test]
fn loop_carried_reassignment_is_partial_for_each_loop_form() -> TestResult {
    let sources = [
        concat!(
            "local function render_for()\n",
            "    local text = C_E0Fixture.SecretText()\n",
            "    local rendered\n",
            "    if canaccessvalue(text) then\n",
            "        for i = 1, 2 do\n",
            "            rendered = text .. \"!\"\n",
            "            text = C_E0Fixture.SecretText()\n",
            "        end\n",
            "    end\n",
            "    return rendered\n",
            "end\n",
        ),
        concat!(
            "local function render_generic_for()\n",
            "    local text = C_E0Fixture.SecretText()\n",
            "    local rendered\n",
            "    if canaccessvalue(text) then\n",
            "        for _, value in ipairs({ 1, 2 }) do\n",
            "            rendered = text .. \"!\"\n",
            "            text = C_E0Fixture.SecretText()\n",
            "        end\n",
            "    end\n",
            "    return rendered\n",
            "end\n",
        ),
        concat!(
            "local function render_while()\n",
            "    local text = C_E0Fixture.SecretText()\n",
            "    local rendered\n",
            "    local i = 0\n",
            "    if canaccessvalue(text) then\n",
            "        while i < 2 do\n",
            "            rendered = text .. \"!\"\n",
            "            text = C_E0Fixture.SecretText()\n",
            "            i = i + 1\n",
            "        end\n",
            "    end\n",
            "    return rendered\n",
            "end\n",
        ),
        concat!(
            "local function render_repeat()\n",
            "    local text = C_E0Fixture.SecretText()\n",
            "    local rendered\n",
            "    local i = 0\n",
            "    if canaccessvalue(text) then\n",
            "        repeat\n",
            "            rendered = text .. \"!\"\n",
            "            text = C_E0Fixture.SecretText()\n",
            "            i = i + 1\n",
            "        until i >= 2\n",
            "    end\n",
            "    return rendered\n",
            "end\n",
        ),
    ];
    let library = library()?;

    for source in sources {
        let main = snapshot(
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new("main/loop-carried.lua", source)],
        )?;
        let report = analyze_local_flow(&main, &[&library])?;

        assert_eq!(
            report.files()[0].status(),
            EmmyLocalFlowFileStatus::PartialUnresolvedFlow
        );
        assert_eq!(report.files()[0].operation_count(), 1);
        assert_eq!(report.files()[0].guard_count(), 1);
        assert!(report.control_flow().is_empty());
    }
    Ok(())
}

#[test]
fn locally_shadowed_predicate_is_not_a_guard() -> TestResult {
    let source = concat!(
        "local function shadowed_guard()\n",
        "    local text = C_E0Fixture.SecretText()\n",
        "    local canaccessvalue = function(value) return true end\n",
        "    if canaccessvalue(text) then\n",
        "        return text .. \"!\"\n",
        "    end\n",
        "end\n",
    );
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new("main/shadowed.lua", source)],
    )?;
    let library = library()?;
    let report = analyze_local_flow(&main, &[&library])?;

    assert_eq!(
        report.files()[0].status(),
        EmmyLocalFlowFileStatus::Complete
    );
    assert_eq!(report.files()[0].operation_count(), 1);
    assert_eq!(report.files()[0].guard_count(), 0);
    assert!(report.control_flow().is_empty());
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
