use super::*;
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits, LuaWorkspaceUniverse, MemberCallSessionQueryProfile,
    references::analyze_member_call_session_with_callable_queries,
};
use emmylua_parser::{LuaChunk, LuaLanguageLevel, LuaParser, ParserConfig};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn selected_function_budget_has_strict_wire_and_independent_target_access_bounds() -> TestResult {
    let budget = FunctionCallBudget::default();
    let wire = serde_json::to_value(budget)?;
    for field in wire.as_object().ok_or("budget")?.keys() {
        let mut missing = wire.clone();
        missing.as_object_mut().ok_or("budget")?.remove(field);
        assert!(serde_json::from_value::<FunctionCallBudget>(missing).is_err());
    }
    for (field, value) in [
        ("unknown", serde_json::json!(1)),
        ("max_ast_visits", serde_json::Value::Null),
        ("max_calls", serde_json::json!(-1)),
        ("max_report_bytes", serde_json::json!(1.5)),
    ] {
        let mut invalid = wire.clone();
        invalid[field] = value;
        assert!(serde_json::from_value::<FunctionCallBudget>(invalid).is_err());
    }
    let duplicate = format!(
        "{{\"max_calls\":1,{}",
        serde_json::to_string(&wire)?
            .strip_prefix('{')
            .ok_or("budget")?
    );
    assert!(serde_json::from_str::<FunctionCallBudget>(&duplicate).is_err());

    let library = snapshot(vec![LuaWorkspaceFileInput::new(
        "library/callables.lua",
        "---@meta _\nRealLib = {}\nfunction RealLib.Step() end\nfunction RealLib.Jump() end\n",
    )])?;
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/empty.lua",
        "return 1\n",
    )])?;
    let queries = ["RealLib.Step".to_owned(), "RealLib.Jump".to_owned()];
    let profile = MemberCallSessionQueryProfile::new(&[], &queries);
    let stop = AtomicBool::new(false);
    let exact = FunctionCallBudget::new(256, 16, 1, 1, 2, 64 * 1024)?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[&library],
        profile.with_function_call_budget(exact),
        true,
        &stop,
    )?
    .function_calls
    .ok_or("named targets")?;
    assert_eq!(report.named_targets().len(), 2);
    assert!(report.calls().is_empty() && report.global_accesses().is_empty());
    let short = FunctionCallBudget::new(256, 16, 1, 1, 1, 64 * 1024)?;
    assert_eq!(
        analyze_member_call_session_with_callable_queries(
            &main,
            &[&library],
            profile.with_function_call_budget(short),
            true,
            &stop,
        )
        .err()
        .ok_or("named target ceiling ignored")?
        .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );

    let globals = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/globals.lua",
        "return MissingOne, MissingTwo\n",
    )])?;
    let profile = MemberCallSessionQueryProfile::new(&[], &[]);
    let exact = FunctionCallBudget::new(64, 1, 1, 2, 1, 64 * 1024)?;
    let report = analyze_member_call_session_with_callable_queries(
        &globals,
        &[],
        profile.with_function_call_budget(exact),
        true,
        &stop,
    )?
    .function_calls
    .ok_or("global accesses")?;
    assert_eq!(report.global_accesses().len(), 2);
    assert!(report.calls().is_empty() && report.named_targets().is_empty());
    let short = FunctionCallBudget::new(64, 1, 1, 1, 1, 64 * 1024)?;
    assert_eq!(
        analyze_member_call_session_with_callable_queries(
            &globals,
            &[],
            profile.with_function_call_budget(short),
            true,
            &stop,
        )
        .err()
        .ok_or("global access ceiling ignored")?
        .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    Ok(())
}

fn snapshot(inputs: Vec<LuaWorkspaceFileInput>) -> TestResult<LuaWorkspaceSnapshot> {
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    Ok(LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        inputs,
        LuaWorkspaceLimits::new(16, 16_384, 1024 * 1024, 8 * 1024 * 1024)?,
    )?)
}

fn nodes(source: &str) -> TestResult<u64> {
    let tree = LuaParser::parse(source, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    assert!(tree.get_errors().is_empty());
    let chunk = LuaChunk::cast(tree.get_red_root()).ok_or("Lua chunk")?;
    Ok(u64::try_from(chunk.descendants::<LuaAst>().count())?)
}

#[test]
fn selected_function_work_budget_counts_both_main_passes_and_library() -> TestResult {
    let one = nodes("do end\n")?;
    let two = nodes("do end\ndo end\n")?;
    let three = nodes("do end\ndo end\ndo end\n")?;
    let per_block = two.checked_sub(one).ok_or("AST density")?;
    assert!(per_block > 0);
    assert_eq!(three.checked_sub(two), Some(per_block));
    // Measure the native AST shape, then split sequential empty blocks under
    // the existing one-MiB file bound. Calls/functions/globals remain tiny.
    let repeats = usize::try_from(MAX_AST_VISITS as u64 / 2 / per_block + 1)?;
    let per_file = 1024 * 1024 / "do end\n".len();
    let mut inputs = Vec::new();
    let mut main_nodes = 0u64;
    let mut remaining = repeats;
    while remaining > 0 {
        let count = remaining.min(per_file);
        let text = "do end\n".repeat(count);
        main_nodes = main_nodes
            .checked_add(nodes(&text)?)
            .ok_or("AST count overflow")?;
        inputs.push(LuaWorkspaceFileInput::new(
            format!("main/blocks-{}.lua", inputs.len()),
            text,
        ));
        remaining -= count;
    }
    let main = snapshot(inputs)?;
    let library_text = "return true\n";
    let library = snapshot(vec![LuaWorkspaceFileInput::new(
        "library/empty.lua",
        library_text,
    )])?;
    let library_nodes = nodes(library_text)?;
    let required = main_nodes
        .checked_mul(2)
        .and_then(|count| count.checked_add(library_nodes))
        .ok_or("visit overflow")?;
    assert!(
        required > MAX_AST_VISITS as u64,
        "required visits: {required}"
    );
    let stop = AtomicBool::new(false);
    let profile = MemberCallSessionQueryProfile::new(&[], &[]);
    assert_eq!(
        analyze_member_call_session_with_callable_queries(&main, &[&library], profile, true, &stop)
            .err()
            .ok_or("default work limit ignored")?
            .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    let exact = FunctionCallBudget::new(required, 16, 1, 1, 1, 64 * 1024)?;
    let result = analyze_member_call_session_with_callable_queries(
        &main,
        &[&library],
        profile.with_function_call_budget(exact),
        true,
        &stop,
    )?;
    let report = result.function_calls.ok_or("sidecar")?;
    assert_eq!(report.budget(), Some(exact));
    assert_eq!(report.functions().len(), main.files().len());
    assert!(report.calls().is_empty() && report.global_accesses().is_empty());
    report.validate()?;
    let over = FunctionCallBudget::new(required - 1, 16, 1, 1, 1, 64 * 1024)?;
    assert_eq!(
        analyze_member_call_session_with_callable_queries(
            &main,
            &[&library],
            profile.with_function_call_budget(over),
            true,
            &stop
        )
        .err()
        .ok_or("selected work limit ignored")?
        .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    assert_eq!(
        analyze_member_call_session_with_callable_queries(
            &main,
            &[&library],
            profile.with_function_call_budget(exact),
            true,
            &AtomicBool::new(true)
        )
        .err()
        .ok_or("cancellation ignored")?
        .code(),
        EmmyMemberCallErrorCode::Cancelled
    );
    let mut overflow = u64::MAX;
    assert_eq!(
        visit(&mut overflow, u64::MAX)
            .err()
            .ok_or("visit overflow ignored")?
            .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    Ok(())
}

#[test]
fn selected_function_counts_and_output_keep_default_facts_and_refuse_limits() -> TestResult {
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/calls.lua",
        "local f = function() return true end\nf()\n",
    )])?;
    let profile = MemberCallSessionQueryProfile::new(&[], &[]);
    let stop = AtomicBool::new(false);
    let ordinary =
        analyze_member_call_session_with_callable_queries(&main, &[], profile, true, &stop)?;
    let budget = FunctionCallBudget::default();
    let selected = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        profile.with_function_call_budget(budget),
        true,
        &stop,
    )?;
    assert_eq!(ordinary.member_calls, selected.member_calls);
    let ordinary = ordinary.function_calls.ok_or("ordinary sidecar")?;
    let selected = selected.function_calls.ok_or("selected sidecar")?;
    let mut ordinary_wire = serde_json::to_value(&ordinary)?;
    let mut selected_wire = serde_json::to_value(&selected)?;
    assert_eq!(ordinary_wire["profile"], FUNCTION_CALL_PROFILE);
    assert_eq!(selected_wire["profile"], FUNCTION_CALL_BUDGET_PROFILE);
    assert!(ordinary_wire.get("budget").is_none());
    assert_eq!(selected_wire["budget"], serde_json::to_value(budget)?);
    assert_ne!(ordinary.analysis_id(), selected.analysis_id());
    for key in ["profile", "budget", "analysis_id"] {
        ordinary_wire.as_object_mut().ok_or("report")?.remove(key);
        selected_wire.as_object_mut().ok_or("report")?.remove(key);
    }
    assert_eq!(ordinary_wire, selected_wire);
    let tiny = FunctionCallBudget::new(2_000_000, 1, 1, 1, 1, 1)?;
    for limits in [tiny, FunctionCallBudget::new(2_000_000, 64, 64, 64, 64, 1)?] {
        assert_eq!(
            analyze_member_call_session_with_callable_queries(
                &main,
                &[],
                profile.with_function_call_budget(limits),
                true,
                &stop
            )
            .err()
            .ok_or("count/output limit ignored")?
            .code(),
            EmmyMemberCallErrorCode::FactBudgetExceeded
        );
    }
    let wire = serde_json::to_value(budget)?;
    for field in wire.as_object().ok_or("budget")?.keys() {
        let mut invalid = wire.clone();
        invalid[field] = serde_json::json!(0);
        assert!(serde_json::from_value::<FunctionCallBudget>(invalid).is_err());
    }
    let text = format!(
        "local f = function() return true end\n{}",
        "f()\n".repeat(65_537)
    );
    let many = snapshot(vec![LuaWorkspaceFileInput::new("main/many.lua", text)])?;
    assert_eq!(
        analyze_member_call_session_with_callable_queries(&many, &[], profile, true, &stop)
            .err()
            .ok_or("default calls limit ignored")?
            .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    let exact = FunctionCallBudget::new(2_000_000, 2, 65_537, 1, 1, 128 * 1024 * 1024)?;
    let selected = analyze_member_call_session_with_callable_queries(
        &many,
        &[],
        profile.with_function_call_budget(exact),
        true,
        &stop,
    )?
    .function_calls
    .ok_or("large sidecar")?;
    assert_eq!(selected.calls().len(), 65_537);
    assert_eq!(selected.functions().len(), 2);
    assert!(selected.global_accesses().is_empty());
    selected.validate()?;
    let too_few = FunctionCallBudget::new(2_000_000, 2, 65_536, 1, 1, 128 * 1024 * 1024)?;
    assert!(
        analyze_member_call_session_with_callable_queries(
            &many,
            &[],
            profile.with_function_call_budget(too_few),
            true,
            &stop
        )
        .is_err()
    );
    Ok(())
}
