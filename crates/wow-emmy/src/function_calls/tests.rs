use super::*;
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits, LuaWorkspaceUniverse, MemberCallSessionQueryProfile,
    references::analyze_member_call_session_with_callable_queries,
};
use emmylua_parser::{LuaChunk, LuaLanguageLevel, LuaParser, ParserConfig};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn lexical_global_access_indexed_global_prefix_keeps_its_own_cell() -> TestResult {
    use crate::global_access::{GlobalAccessKey, GlobalAccessKind, GlobalAccessResolution};
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/registry.lua",
        "_G.SlashCmdList = {}\n",
    )])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("report")?;
    let [access] = report.global_accesses() else {
        return Err("missing exact root access".into());
    };
    assert_eq!(access.root_name(), "_G");
    assert_eq!(
        access.keys(),
        &[GlobalAccessKey::String("SlashCmdList".into())]
    );
    assert_eq!(access.kind(), GlobalAccessKind::Write);
    assert_eq!(access.resolution(), GlobalAccessResolution::Unresolved);
    assert!(access.declaration().is_none() && !access.is_alias());
    report.validate()?;
    Ok(())
}

#[test]
fn lexical_global_access_indexed_prefix_preserves_environment_and_shadow_boundaries() -> TestResult
{
    use crate::global_access::{GlobalAccessKey, GlobalAccessResolution};
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/env.lua",
        "_ENV.Registry = {}\n_G[\"StringRegistry\"] = {}\n_G[key] = {}\nRegistry.Value = 1\n",
    )])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("environment report")?;
    let roots = report
        .global_accesses()
        .iter()
        .filter(|access| matches!(access.root_name(), "_G" | "_ENV"))
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 3);
    assert!(roots.iter().all(
        |access| access.resolution() == GlobalAccessResolution::Unresolved
            && access.declaration().is_none()
    ));
    assert!(roots.iter().any(|access| !access.path_complete()));
    assert!(
        report
            .global_accesses()
            .iter()
            .any(|access| access.root_name() == "Registry"
                && access.resolution() == GlobalAccessResolution::MainGlobal)
    );
    report.validate()?;
    let source = concat!(
        "Global = {}\ndo local _G = Global; _G.AliasSlot = {} end\n",
        "do local _G = {}; _G.Hidden = {} end\n_G = {}\n_G.OwnSlot = {}\n",
    );
    let main = snapshot(vec![LuaWorkspaceFileInput::new("main/shadow.lua", source)])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("shadow report")?;
    let alias = report
        .global_accesses()
        .iter()
        .find(|access| access.keys() == [GlobalAccessKey::String("AliasSlot".into())])
        .ok_or("lexical alias discarded")?;
    assert_eq!(alias.root_name(), "Global");
    assert_eq!(alias.aliases().len(), 1);
    assert_eq!(alias.aliases()[0].name, "_G");
    assert_eq!(alias.resolution(), GlobalAccessResolution::MainGlobal);
    assert!(
        !report
            .global_accesses()
            .iter()
            .any(|access| access.keys() == [GlobalAccessKey::String("Hidden".into())])
    );
    let own = report
        .global_accesses()
        .iter()
        .find(|access| access.keys() == [GlobalAccessKey::String("OwnSlot".into())])
        .ok_or("explicit root discarded")?;
    assert_eq!(own.root_name(), "_G");
    assert_eq!(own.resolution(), GlobalAccessResolution::MainGlobal);
    assert_eq!(
        own.declaration()
            .ok_or("root declaration")?
            .span
            .byte_start(),
        Some((source.find("\n_G = {}").ok_or("physical root")? + 1) as u64)
    );
    report.validate()?;
    let main = snapshot(vec![
        LuaWorkspaceFileInput::new("main/a.lua", "_G = {}\n"),
        LuaWorkspaceFileInput::new("main/b.lua", "_G = {}\n_G.AmbiguousRoot = {}\n"),
    ])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("ambiguous report")?;
    let ambiguous = report
        .global_accesses()
        .iter()
        .find(|access| access.keys() == [GlobalAccessKey::String("AmbiguousRoot".into())])
        .ok_or("ambiguous root omitted")?;
    assert_eq!(ambiguous.resolution(), GlobalAccessResolution::Unresolved);
    assert!(ambiguous.declaration().is_none());
    report.validate()?;
    Ok(())
}

#[test]
fn selected_global_budget_strict_wire_preserves_omitted_shape() -> TestResult {
    use crate::global_access::GlobalAccessBudget;
    let plain = FunctionCallBudget::default();
    let old_bytes = serde_json::to_vec(&plain)?;
    let old_wire = serde_json::to_value(plain)?;
    assert_eq!(old_wire.as_object().ok_or("budget")?.len(), 6);
    assert_eq!(
        serde_json::to_vec(&serde_json::from_slice::<FunctionCallBudget>(&old_bytes)?)?,
        old_bytes
    );
    let component = GlobalAccessBudget::new(5_000_000, 128 * 1024 * 1024)?;
    let selected = plain.with_global_access_budget(component);
    let wire = serde_json::to_value(selected)?;
    assert_eq!(
        serde_json::from_value::<FunctionCallBudget>(wire.clone())?,
        selected
    );
    let global = wire["global_access"].clone();
    for (field, value) in [
        ("max_work", serde_json::json!(0)),
        ("max_text_bytes", serde_json::json!(-1)),
        ("max_work", serde_json::json!(1.5)),
        ("max_text_bytes", serde_json::Value::Null),
        ("unknown", serde_json::json!(1)),
    ] {
        let mut bad = wire.clone();
        bad["global_access"][field] = value;
        assert!(serde_json::from_value::<FunctionCallBudget>(bad).is_err());
    }
    for field in ["max_work", "max_text_bytes"] {
        let mut missing = global.clone();
        missing
            .as_object_mut()
            .ok_or("global budget")?
            .remove(field);
        assert!(serde_json::from_value::<GlobalAccessBudget>(missing).is_err());
    }
    let mut null = wire.clone();
    null["global_access"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<FunctionCallBudget>(null).is_err());
    let duplicate_global = format!(
        "{{\"max_work\":1,{}",
        serde_json::to_string(&global)?
            .strip_prefix('{')
            .ok_or("global")?
    );
    assert!(serde_json::from_str::<GlobalAccessBudget>(&duplicate_global).is_err());
    let duplicate = format!(
        "{{\"global_access\":{},{}",
        global,
        serde_json::to_string(&wire)?
            .strip_prefix('{')
            .ok_or("budget")?
    );
    assert!(serde_json::from_str::<FunctionCallBudget>(&duplicate).is_err());
    assert!(GlobalAccessBudget::new(0, 1).is_err());
    assert!(GlobalAccessBudget::new(1, 0).is_err());
    Ok(())
}

#[test]
fn selected_global_budget_admits_native_text_above_default() -> TestResult {
    use crate::global_access::GlobalAccessBudget;
    let text = "do local value = Global end\n".repeat(9000);
    let main = snapshot(vec![
        LuaWorkspaceFileInput::new("main/a.lua", format!("Global = {{}}\n{text}")),
        LuaWorkspaceFileInput::new("main/b.lua", text),
    ])?;
    let stop = AtomicBool::new(false);
    let default = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &stop,
    )
    .err()
    .ok_or("default text guard not reached")?;
    assert_eq!(default.code(), EmmyMemberCallErrorCode::FactBudgetExceeded);
    assert!(default.message().contains("text limit"), "{default:?}");
    let selected = FunctionCallBudget::default()
        .with_global_access_budget(GlobalAccessBudget::new(2_000_000, 32 * 1024 * 1024)?);
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]).with_function_call_budget(selected),
        true,
        &stop,
    )?
    .function_calls
    .ok_or("selected report")?;
    assert_eq!(report.profile, FUNCTION_CALL_GLOBAL_BUDGET_PROFILE);
    assert_eq!(report.budget(), Some(selected));
    assert_eq!(report.global_accesses().len(), 18_001);
    assert_eq!(report.files().len(), 2);
    report.validate()?;
    let mut downgrade = report.clone();
    downgrade.profile = FUNCTION_CALL_BUDGET_PROFILE;
    downgrade.analysis_id = downgrade.identity()?;
    assert!(downgrade.validate().is_err());
    downgrade = report.clone();
    downgrade.budget = Some(FunctionCallBudget::default());
    downgrade.analysis_id = downgrade.identity()?;
    assert!(downgrade.validate().is_err());
    Ok(())
}

#[test]
fn selected_global_budget_location_index_preserves_exact_joins_and_ambiguity() -> TestResult {
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/calls.lua",
        "Receiver = {}\nfunction Receiver:Handle(value) end\nTarget = {}\nReceiver:Handle(Target)\nlocal receiver = Receiver\nreceiver:Handle(Target)\n",
    )])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("report")?;
    for call in report.calls() {
        for span in std::iter::once(call.callee_span())
            .chain(call.arguments().iter().map(SourceCallArgument::span))
        {
            let scan = report
                .global_accesses()
                .iter()
                .filter(|access| {
                    access.path() == call.path()
                        && access.function_id() == call.caller_function_id()
                        && access.span() == span
                })
                .collect::<Vec<_>>();
            assert_eq!(
                report
                    .access_locations
                    .at(
                        report.global_accesses(),
                        call.path(),
                        call.caller_function_id(),
                        span
                    )
                    .collect::<Vec<_>>(),
                scan
            );
        }
    }
    let call = report
        .calls()
        .iter()
        .find(|call| report.exact_call_receiver(call).is_some())
        .ok_or("global receiver")?;
    assert_eq!(
        report.exact_call_receiver(call).ok_or("receiver")?.key(),
        "Receiver"
    );
    assert!(
        report
            .calls()
            .iter()
            .any(|call| report.exact_call_receiver(call).is_none())
    );
    let bytes = serde_json::to_vec(&report)?;
    let mut rebuilt = report.clone();
    rebuilt.access_locations = std::sync::Arc::new(AccessLocationIndex::build(
        rebuilt.global_accesses(),
        u64::MAX,
        &AtomicBool::new(false),
    )?);
    assert_eq!(serde_json::to_vec(&rebuilt)?, bytes);
    assert_eq!(rebuilt.identity()?, report.analysis_id());
    rebuilt.validate()?;
    let access = rebuilt
        .access_locations
        .at(
            rebuilt.global_accesses(),
            call.path(),
            call.caller_function_id(),
            call.callee_span(),
        )
        .next()
        .ok_or("callee access")?
        .clone();
    rebuilt.global_accesses.push(access);
    rebuilt.access_locations = std::sync::Arc::new(AccessLocationIndex::build(
        rebuilt.global_accesses(),
        u64::MAX,
        &AtomicBool::new(false),
    )?);
    assert!(
        rebuilt.exact_call_receiver(call).is_none(),
        "duplicate exact receiver was deduplicated"
    );
    let mut stale = report.clone();
    stale.global_accesses.swap(0, 1);
    assert!(
        stale
            .access_locations
            .validate(stale.global_accesses())
            .is_err()
    );
    assert_eq!(
        AccessLocationIndex::build(report.global_accesses(), u64::MAX, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled index")?
            .code(),
        EmmyMemberCallErrorCode::Cancelled
    );
    Ok(())
}

#[test]
fn lexical_global_access_excludes_implicit_and_explicit_method_parameters() -> TestResult {
    let source = concat!(
        "Object = {}\n",
        "function Object:GetName()\n",
        " local alias = self\n",
        " return function() return self.Name, alias.Name end\n",
        "end\n",
        "function Object.Explicit(self) return self.Name end\n",
        "do\n local self = Object\n self.Name = 1\n",
        " do local self = {} self.Other = 2 end\nend\n",
        "self = {}\nreturn self.Name\n",
    );
    let main = snapshot(vec![LuaWorkspaceFileInput::new("main/method.lua", source)])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("function report")?;
    assert!(
        report
            .global_accesses()
            .iter()
            .any(|access| access.root_name() == "Object")
    );
    // Only the explicit local alias has global provenance. Method parameters,
    // their nested captures/aliases and a shadowing table local do not.
    let aliases = report
        .global_accesses()
        .iter()
        .filter(|access| access.is_alias())
        .collect::<Vec<_>>();
    assert_eq!(aliases.len(), 1);
    assert_eq!(aliases[0].root_name(), "Object");
    assert_eq!(aliases[0].aliases()[0].name, "self");
    assert_eq!(
        aliases[0].keys(),
        &[crate::global_access::GlobalAccessKey::String("Name".into())]
    );
    assert!(
        report
            .global_accesses()
            .iter()
            .any(|access| access.root_name() == "self")
    );
    Ok(())
}

#[test]
fn lexical_global_access_retains_function_alias_and_rebinding_provenance() -> TestResult {
    use crate::global_access::{AliasAccessBlocker, GlobalAccessKind};
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/alias.lua",
        "function GlobalFunction() end\nlocal alias = GlobalFunction\nalias()\nalias = GlobalFunction\nalias()\n",
    )])?;
    let report = analyze_member_call_session_with_callable_queries(
        &main,
        &[],
        MemberCallSessionQueryProfile::new(&[], &[]),
        true,
        &AtomicBool::new(false),
    )?
    .function_calls
    .ok_or("function report")?;
    let aliases = report
        .global_accesses()
        .iter()
        .filter(|access| access.is_alias())
        .collect::<Vec<_>>();
    assert_eq!(aliases.len(), 3);
    for access in aliases {
        assert_eq!(access.root_name(), "GlobalFunction");
        assert_eq!(access.aliases().len(), 1);
        assert_eq!(access.aliases()[0].name, "alias");
        assert!(access.aliases()[0].first_reassignment.is_some());
        assert_eq!(
            access.alias_blocker(),
            Some(if access.kind() == GlobalAccessKind::Read {
                AliasAccessBlocker::ReassignedBinding
            } else {
                AliasAccessBlocker::LocalBindingWrite
            })
        );
    }
    Ok(())
}

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
