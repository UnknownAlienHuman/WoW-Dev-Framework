use super::*;

const SOURCE: &str = concat!(
    "function ArbitraryTarget() end\n",
    "TargetTable = {}\n",
    "function TargetTable.ArbitraryMember() end\n",
    "function Callback() end\n",
    "function Install(dynamic)\n",
    "hooksecurefunc(\"ArbitraryTarget\", Callback)\n",
    "hooksecurefunc(TargetTable, \"ArbitraryMember\", Callback)\n",
    "hooksecurefunc(dynamic, Callback)\n",
    "hooksecurefunc(true, Callback)\n",
    "hooksecurefunc(\"TargetTable.ArbitraryMember\", Callback)\n",
    "hooksecurefunc(TargetTable, dynamic, Callback)\n",
    "hooksecurefunc(TargetTable, \"ArbitraryMember\", false)\n",
    "local TargetTable = {}\n",
    "hooksecurefunc(TargetTable, \"ArbitraryMember\", Callback)\n",
    "local function hooksecurefunc(name, callback) end\n",
    "hooksecurefunc(\"ArbitraryTarget\", Callback)\n",
    "end\n",
);

fn report(
    source: &str,
    selected: bool,
    query_limit: usize,
    limits: FunctionCallBudget,
    stop: &AtomicBool,
) -> TestResult<FunctionCallReport> {
    let main = snapshot(vec![LuaWorkspaceFileInput::new("main/hooks.lua", source)])?;
    let library = snapshot(vec![LuaWorkspaceFileInput::new(
        "library/hook.lua",
        "function hooksecurefunc(...) end\n",
    )])?;
    let queries = ["hooksecurefunc".to_owned()];
    let mut profile = MemberCallSessionQueryProfile::new(&[], &queries)
        .with_function_call_budget(limits)
        .with_physical_function_declarations(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)
        .with_query_limit(query_limit)?;
    if selected {
        profile = profile.with_hook_target_recipe(HookTargetRecipe::HookTargetLiteralsV1);
    }
    Ok(
        analyze_member_call_session_with_callable_queries(&main, &[&library], profile, true, stop)?
            .function_calls
            .ok_or("hook report")?,
    )
}

#[test]
fn hook_literal_targets_keep_original_scope_and_default_query_identity() -> TestResult {
    let stop = AtomicBool::new(false);
    let old = report(SOURCE, false, 4096, FunctionCallBudget::default(), &stop)?;
    let selected = report(SOURCE, true, 4096, FunctionCallBudget::default(), &stop)?;
    old.validate()?;
    selected.validate()?;
    assert!(old.calls().iter().all(|call| call.hook_target().is_none()));
    assert_eq!(old.named_targets(), selected.named_targets());
    assert_eq!(
        old.symbol_lookup_analysis_ids(),
        selected.symbol_lookup_analysis_ids()
    );
    assert_ne!(old.analysis_id(), selected.analysis_id());
    let targets = selected
        .calls()
        .iter()
        .filter_map(|call| call.hook_target().map(|target| (call, target)))
        .collect::<Vec<_>>();
    assert_eq!(targets.len(), 2, "{targets:?}");
    for (call, target) in targets {
        assert_eq!(call.path(), "main/hooks.lua");
        assert!(
            selected
                .functions()
                .iter()
                .any(|function| function.fact_id() == target.function_id())
        );
        match target.form() {
            SourceHookTargetForm::GlobalName => {
                assert_eq!(target.query(), "ArbitraryTarget");
                assert_eq!(
                    source_slice(SOURCE, call.call_span())?,
                    "hooksecurefunc(\"ArbitraryTarget\", Callback)"
                );
            }
            SourceHookTargetForm::TableMember => {
                assert_eq!(target.query(), "TargetTable.ArbitraryMember");
                assert_eq!(
                    source_slice(SOURCE, call.call_span())?,
                    "hooksecurefunc(TargetTable, \"ArbitraryMember\", Callback)"
                );
            }
        }
    }
    // Explicit absence serializes no new identity input.
    assert!(
        serde_json::to_value(&old)?
            .get("hook_target_recipe")
            .is_none()
    );
    assert_eq!(
        old.analysis_id(),
        report(SOURCE, false, 4096, FunctionCallBudget::default(), &stop)?.analysis_id()
    );
    Ok(())
}

#[test]
fn hook_literal_targets_refuse_ambiguous_reassigned_and_wrong_api() -> TestResult {
    let stop = AtomicBool::new(false);
    for source in [
        "function Target() end\nTarget = function() end\nhooksecurefunc(\"Target\", function() end)\n",
        "TargetTable = {}\nfunction TargetTable.Method() end\nTargetTable = {}\nhooksecurefunc(TargetTable, \"Method\", function() end)\n",
        "TargetTable = {}\nfunction TargetTable.Method() end\nTargetTable.Method = function() end\nhooksecurefunc(TargetTable, \"Method\", function() end)\n",
        "function Target() end\nfunction Callback() end\nCallback = function() end\nhooksecurefunc(\"Target\", Callback)\n",
        "function hooksecurefunc(...) end\nfunction Target() end\nhooksecurefunc(\"Target\", function() end)\n",
    ] {
        let report = report(source, true, 4096, FunctionCallBudget::default(), &stop)?;
        assert!(
            report
                .calls()
                .iter()
                .all(|call| call.hook_target().is_none()),
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn hook_literal_targets_share_query_ast_output_and_cancel_limits() -> TestResult {
    let stop = AtomicBool::new(false);
    for (query_limit, limits) in [
        (1, FunctionCallBudget::default()),
        (
            4096,
            FunctionCallBudget::new(1, 100, 100, 100, 100, 1_000_000)?,
        ),
        (
            4096,
            FunctionCallBudget::new(100_000, 100, 100, 100, 100, 1)?,
        ),
        (
            4096,
            FunctionCallBudget::new(100_000, 100, 100, 100, 1, 1_000_000)?,
        ),
    ] {
        let error = report(SOURCE, true, query_limit, limits, &stop)
            .err()
            .ok_or("missing bounded refusal")?;
        let error = error
            .downcast_ref::<EmmyMemberCallError>()
            .ok_or("typed refusal")?;
        assert_eq!(error.code(), EmmyMemberCallErrorCode::FactBudgetExceeded);
    }
    let error = report(
        SOURCE,
        true,
        4096,
        FunctionCallBudget::default(),
        &AtomicBool::new(true),
    )
    .err()
    .ok_or("missing cancellation")?;
    assert_eq!(
        error
            .downcast_ref::<EmmyMemberCallError>()
            .ok_or("typed cancellation")?
            .code(),
        EmmyMemberCallErrorCode::Cancelled
    );
    Ok(())
}
