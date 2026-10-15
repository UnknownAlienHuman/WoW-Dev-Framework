use super::*;
use std::{collections::BTreeMap, error::Error, sync::atomic::AtomicBool};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    MemberCallSessionQueryProfile, analyze_member_call_session_with_callable_queries,
    function_calls::{FunctionCallBudget, FunctionCallReport, PhysicalFunctionDeclarationRecipe},
    global_access::GlobalAccessBudget,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

const HOOK_LIBRARY: &str = concat!(
    "---@overload fun(name: string, hook: function)\n",
    "function hooksecurefunc(tbl, name, hook) end\n",
);

fn backend() -> TestResult<EmmyBackendIdentity> {
    let digest = |byte: char| format!("sha256:{}", byte.to_string().repeat(64));
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        digest('1'),
        digest('2'),
    )?)
}

fn workspace(path: &str, text: &str) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend()?,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(path, text)],
        LuaWorkspaceLimits::new(8, 1024, 64 * 1024, 128 * 1024)?,
    )?)
}

fn report(main_source: &str, query_names: &[&str]) -> TestResult<FunctionCallReport> {
    let main = workspace("main/hooks.lua", main_source)?;
    let library = workspace("library/Wiki.lua", HOOK_LIBRARY)?;
    let callables = query_names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let query_profile = MemberCallSessionQueryProfile::new(&[], &callables)
        .with_physical_function_declarations(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1);
    let session = analyze_member_call_session_with_callable_queries(
        &main,
        &[&library],
        query_profile,
        true,
        &AtomicBool::new(false),
    )?;
    Ok(session
        .function_calls
        .ok_or("missing function-call report")?)
}

fn report_with_local_constants(
    main_source: &str,
    query_names: &[&str],
) -> TestResult<FunctionCallReport> {
    let main = workspace("main/hooks.lua", main_source)?;
    let library = workspace("library/Wiki.lua", HOOK_LIBRARY)?;
    let callables = query_names
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let budget = FunctionCallBudget::default()
        .with_global_access_budget(GlobalAccessBudget::new(2_000_000, 8 * 1024 * 1024)?);
    let query_profile = MemberCallSessionQueryProfile::new(&[], &callables)
        .with_physical_function_declarations(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)
        .with_function_call_budget(budget);
    let session = analyze_member_call_session_with_callable_queries(
        &main,
        &[&library],
        query_profile,
        true,
        &AtomicBool::new(false),
    )?;
    Ok(session
        .function_calls
        .ok_or("missing function-call report")?)
}

fn function_proposals<'a>(
    report: &'a FunctionCallReport,
    proposal_ids: &'a [String],
) -> BTreeMap<&'a str, &'a str> {
    report
        .functions()
        .iter()
        .zip(proposal_ids)
        .map(|(function, proposal)| (function.fact_id(), proposal.as_str()))
        .collect()
}

fn proposal_ids(report: &FunctionCallReport) -> Vec<String> {
    report
        .functions()
        .iter()
        .enumerate()
        .map(|(index, _)| format!("test-function-proposal-{index}"))
        .collect()
}

fn secure_calls(report: &FunctionCallReport) -> Vec<&wow_emmy::function_calls::SourceCallFact> {
    report
        .calls()
        .iter()
        .filter(|call| call.resolved_callable_key() == Some(W5_SECURE_HOOK_CALLABLE))
        .collect()
}

#[test]
fn secure_posthook_requires_the_selected_library_api_and_resolves_only_function_endpoints()
-> TestResult {
    let source = concat!(
        "function Target() end\n",
        "function Callback() end\n",
        "TargetTable = {}\n",
        "function TargetTable.Method() end\n",
        "hooksecurefunc(\"Target\", Callback)\n",
        "hooksecurefunc(TargetTable, \"Method\", function() end)\n",
    );
    let report = report(source, &["hooksecurefunc", "Target", "TargetTable.Method"])?;
    let proposals = proposal_ids(&report);
    let function_proposals = function_proposals(&report, &proposals);
    let calls = secure_calls(&report);
    assert_eq!(calls.len(), 2);

    let global = calls
        .iter()
        .copied()
        .find(|call| call.arguments().len() == 2)
        .ok_or("global-name overload missing")?;
    let global_semantics = w5_secure_posthook_semantics(&report, &function_proposals, global);
    assert!(global_semantics.api_provenance);
    assert_eq!(
        global_semantics.signature_kind,
        W5_SECURE_SIGNATURE_GLOBAL_STRING
    );
    assert_eq!(global_semantics.target_form, W5_SECURE_FORM_GLOBAL);
    assert!(global_semantics.target_endpoint.is_some());
    assert!(global_semantics.callback_endpoint.is_some());
    assert_eq!(global_semantics.callback_form, W5_CALLBACK_EXACT);
    assert!(global_semantics.is_derived());
    let global_receipt = W5SecurePosthookMatch {
        call_id: "global-call".into(),
        caller_proposal_id: "caller-proposal".into(),
        target_form: global_semantics.target_form,
        target_proposal_id: global_semantics.target_endpoint.clone(),
        member_name: global_semantics.member_name.clone(),
        callback_form: global_semantics.callback_form,
        callback_proposal_id: global_semantics.callback_endpoint.clone(),
        confidence: GraphConfidence::Derived,
        hook_kind: W5_HOOK_KIND_SECURE_POSTHOOK,
        relation_proposal_id: "relation-proposal".into(),
    };
    let receipt_json = serde_json::to_value(&global_receipt)?;
    assert!(
        receipt_json.get("member_name").is_none(),
        "the valid global-string form has typed-absent member_name, which must be omitted from canonical receipt JSON"
    );
    assert_eq!(
        receipt_json["target_proposal_id"].as_str(),
        global_semantics.target_endpoint.as_deref()
    );
    assert_eq!(
        receipt_json["callback_proposal_id"].as_str(),
        global_semantics.callback_endpoint.as_deref()
    );
    assert_eq!(receipt_json["target_form"], W5_SECURE_FORM_GLOBAL);
    assert_eq!(receipt_json["callback_form"], W5_CALLBACK_EXACT);
    assert_eq!(
        receipt_json["confidence"],
        serde_json::to_value(GraphConfidence::Derived)?
    );
    wow_core::canonical_json_bytes(&global_receipt)?;

    let unresolved_receipt = W5SecurePosthookMatch {
        call_id: "dynamic-call".into(),
        caller_proposal_id: "caller-proposal".into(),
        target_form: W5_SECURE_FORM_DYNAMIC,
        target_proposal_id: None,
        member_name: None,
        callback_form: W5_CALLBACK_DYNAMIC,
        callback_proposal_id: None,
        confidence: GraphConfidence::Possible,
        hook_kind: W5_HOOK_KIND_SECURE_POSTHOOK,
        relation_proposal_id: "possible-relation".into(),
    };
    let unresolved_json = serde_json::to_value(&unresolved_receipt)?;
    for field in ["target_proposal_id", "member_name", "callback_proposal_id"] {
        assert!(
            unresolved_json.get(field).is_none(),
            "typed absence must be omitted for {field}"
        );
    }
    assert_eq!(
        unresolved_json["confidence"],
        serde_json::to_value(GraphConfidence::Possible)?
    );
    wow_core::canonical_json_bytes(&unresolved_receipt)?;

    let table_member = calls
        .iter()
        .copied()
        .find(|call| call.arguments().len() == 3)
        .ok_or("table/member overload missing")?;
    let table_semantics = w5_secure_posthook_semantics(&report, &function_proposals, table_member);
    assert!(table_semantics.api_provenance);
    assert_eq!(
        table_semantics.signature_kind,
        W5_SECURE_SIGNATURE_TABLE_MEMBER_STRING
    );
    assert_eq!(table_semantics.target_form, W5_SECURE_FORM_TABLE_MEMBER);
    assert_eq!(table_semantics.member_name.as_deref(), Some("Method"));
    assert!(table_semantics.target_endpoint.is_some());
    assert!(table_semantics.callback_endpoint.is_some());
    assert!(table_semantics.is_derived());
    Ok(())
}

#[test]
fn secure_posthook_rejects_main_shadow_and_known_wrong_types_or_arities() -> TestResult {
    let shadow = report(
        concat!(
            "function Target() end\n",
            "function hooksecurefunc(name, hook) end\n",
            "hooksecurefunc(\"Target\", function() end)\n",
        ),
        &["hooksecurefunc", "Target"],
    )?;
    let shadow_proposals = proposal_ids(&shadow);
    let shadow_function_proposals = function_proposals(&shadow, &shadow_proposals);
    let shadow_calls = secure_calls(&shadow);
    let [shadow_call] = shadow_calls.as_slice() else {
        return Err("expected one Main-shadow call".into());
    };
    assert!(matches!(
        shadow_call.target(),
        SourceCallTarget::MainFunction { .. }
    ));
    let shadow_semantics =
        w5_secure_posthook_semantics(&shadow, &shadow_function_proposals, shadow_call);
    assert!(!shadow_semantics.api_provenance);
    assert!(!shadow_semantics.is_derived());

    let invalid = report(
        concat!(
            "function Target() end\n",
            "TargetTable = {}\n",
            "hooksecurefunc(true, function() end)\n",
            "hooksecurefunc(nil, function() end)\n",
            "hooksecurefunc(Target, \"Method\")\n",
            "hooksecurefunc(Target, function() end)\n",
            "hooksecurefunc(Target, \"Method\", function() end)\n",
            "hooksecurefunc(TargetTable, true, function() end)\n",
            "hooksecurefunc(TargetTable, nil, function() end)\n",
            "hooksecurefunc(\"Target\", false)\n",
            "hooksecurefunc(\"Target\", nil)\n",
            "hooksecurefunc(\"Target\")\n",
            "hooksecurefunc(TargetTable, \"Method\", function() end, 4)\n",
        ),
        &["hooksecurefunc", "Target"],
    )?;
    let invalid_proposals = proposal_ids(&invalid);
    let invalid_function_proposals = function_proposals(&invalid, &invalid_proposals);
    let calls = secure_calls(&invalid);
    assert_eq!(calls.len(), 11);
    for call in calls {
        let semantics = w5_secure_posthook_semantics(&invalid, &invalid_function_proposals, call);
        assert_eq!(
            semantics.signature_kind,
            W5_SECURE_SIGNATURE_INVALID,
            "unexpected admitted signature at {:?}",
            call.call_span()
        );
        assert!(!semantics.is_derived());
    }
    Ok(())
}

#[test]
fn secure_posthook_typed_integer_slots_are_invalid_and_local_strings_stay_unresolved() -> TestResult
{
    let report = report_with_local_constants(
        concat!(
            "function Target() end\n",
            "function Callback() end\n",
            "TargetTable = {}\n",
            "function TargetTable.Method() end\n",
            "local integerTarget = 17\n",
            "local integerMember = 18\n",
            "local integerCallback = 19\n",
            "local globalName = \"Target\"\n",
            "local memberName = \"Method\"\n",
            "hooksecurefunc(integerTarget, Callback)\n",
            "hooksecurefunc(TargetTable, integerMember, Callback)\n",
            "hooksecurefunc(\"Target\", integerCallback)\n",
            "hooksecurefunc(globalName, Callback)\n",
            "hooksecurefunc(TargetTable, memberName, Callback)\n",
        ),
        &["hooksecurefunc", "Target", "TargetTable.Method"],
    )?;
    let proposals = proposal_ids(&report);
    let function_proposals = function_proposals(&report, &proposals);
    let calls = secure_calls(&report);
    assert_eq!(calls.len(), 5);

    let integer_target = calls
        .iter()
        .copied()
        .find(|call| {
            call.arguments()[0]
                .local_constant()
                .is_some_and(|value| value.name() == "integerTarget")
        })
        .ok_or("typed integer global-name target")?;
    let integer_member = calls
        .iter()
        .copied()
        .find(|call| {
            call.arguments()[1]
                .local_constant()
                .is_some_and(|value| value.name() == "integerMember")
        })
        .ok_or("typed integer table member")?;
    let integer_callback = calls
        .iter()
        .copied()
        .find(|call| {
            call.arguments()[1]
                .local_constant()
                .is_some_and(|value| value.name() == "integerCallback")
        })
        .ok_or("typed integer callback")?;
    for call in [integer_target, integer_member, integer_callback] {
        let semantics = w5_secure_posthook_semantics(&report, &function_proposals, call);
        assert_eq!(semantics.signature_kind, W5_SECURE_SIGNATURE_INVALID);
        assert!(!semantics.is_derived());
    }
    assert_eq!(
        w5_argument_literal_kind(&integer_target.arguments()[0]),
        Some("integer")
    );
    assert_eq!(
        w5_argument_literal_kind(&integer_member.arguments()[1]),
        Some("integer")
    );
    assert_eq!(
        w5_argument_literal_kind(&integer_callback.arguments()[1]),
        Some("integer")
    );
    assert_eq!(
        integer_target.arguments()[0]
            .local_constant()
            .map(|value| value.value()),
        Some(&SourceCallConstant::Integer(17))
    );
    assert_eq!(
        integer_member.arguments()[1]
            .local_constant()
            .map(|value| value.value()),
        Some(&SourceCallConstant::Integer(18))
    );
    assert_eq!(
        integer_callback.arguments()[1]
            .local_constant()
            .map(|value| value.value()),
        Some(&SourceCallConstant::Integer(19))
    );

    for (name, member, signature) in [
        ("globalName", None, W5_SECURE_SIGNATURE_GLOBAL_STRING),
        (
            "memberName",
            Some("Method"),
            W5_SECURE_SIGNATURE_TABLE_MEMBER_STRING,
        ),
    ] {
        let call = calls
            .iter()
            .copied()
            .find(|call| {
                let ordinal = if member.is_some() { 1 } else { 0 };
                call.arguments()[ordinal]
                    .local_constant()
                    .is_some_and(|value| value.name() == name)
            })
            .ok_or("typed local string argument")?;
        let semantics = w5_secure_posthook_semantics(&report, &function_proposals, call);
        assert_eq!(semantics.signature_kind, signature);
        assert!(semantics.target_endpoint.is_none());
        assert!(semantics.callback_endpoint.is_some());
        assert!(!semantics.is_derived());
        if let Some(member) = member {
            assert_eq!(semantics.member_name, None);
            assert_eq!(semantics.target_form, W5_SECURE_FORM_TABLE_MEMBER);
            assert!(!member.is_empty());
        } else {
            assert_eq!(semantics.target_form, W5_SECURE_FORM_GLOBAL);
        }
    }
    Ok(())
}

#[test]
fn unresolved_valid_shape_is_possible_and_pack_targets_the_normalized_endpoint() -> TestResult {
    let report = report(
        concat!(
            "function Target() end\n",
            "local name = GetTargetName()\n",
            "local callback = GetCallback()\n",
            "hooksecurefunc(name, callback)\n",
            "hooksecurefunc(\"Target\", callback)\n",
        ),
        &["hooksecurefunc", "Target"],
    )?;
    let proposals = proposal_ids(&report);
    let function_proposals = function_proposals(&report, &proposals);
    let calls = secure_calls(&report);
    assert_eq!(calls.len(), 2);
    let call = calls
        .iter()
        .copied()
        .find(|call| call.arguments()[0].literal().is_none())
        .ok_or("dynamic-target secure-hook call")?;
    let semantics = w5_secure_posthook_semantics(&report, &function_proposals, call);
    assert!(semantics.api_provenance);
    assert_eq!(semantics.signature_kind, W5_SECURE_SIGNATURE_GLOBAL_DYNAMIC);
    assert_eq!(semantics.target_form, W5_SECURE_FORM_DYNAMIC);
    assert!(semantics.target_endpoint.is_none());
    assert!(semantics.callback_endpoint.is_none());
    assert!(!semantics.is_derived());

    let known_target = calls
        .iter()
        .copied()
        .find(|call| {
            matches!(
                call.arguments()[0].literal(),
                Some(SourceCallLiteral::String(value)) if value == "Target"
            )
        })
        .ok_or("known-target/dynamic-callback secure-hook call")?;
    let known_target_semantics =
        w5_secure_posthook_semantics(&report, &function_proposals, known_target);
    assert_eq!(
        known_target_semantics.signature_kind,
        W5_SECURE_SIGNATURE_GLOBAL_STRING
    );
    assert!(known_target_semantics.target_endpoint.is_some());
    assert!(known_target_semantics.callback_endpoint.is_none());
    assert!(!known_target_semantics.is_derived());

    let pack = w5_hook_pack("test-graph-registry")?;
    let rule = pack
        .document()
        .pack
        .rules
        .iter()
        .find(|rule| rule.rule_id.as_ref() == W5_SECURE_POSTHOOK_RULE)
        .ok_or("secure-posthook pack rule missing")?;
    assert_eq!(rule.version, W5_SECURE_POSTHOOK_RULE_VERSION);
    let [RecognizerOutput::RelationAssertion { source, target, .. }] = rule.outputs.as_slice()
    else {
        return Err("expected one secure-posthook relation output".into());
    };
    assert_eq!(source.as_ref(), "call.caller");
    assert_eq!(target.as_ref(), "call.target_endpoint");
    assert_eq!(
        w5_output_endpoint(&RecognizerFactValue::Reference("target-node".into()))?,
        Some("target-node".to_owned())
    );
    assert_eq!(
        w5_output_endpoint(&RecognizerFactValue::String(
            W5_SECURE_ENDPOINT_UNRESOLVED.into()
        ))?,
        None
    );
    Ok(())
}
