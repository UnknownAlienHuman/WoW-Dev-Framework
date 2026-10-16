use super::*;
use crate::function_calls::names::{self, SourceFunctionBindingState};

#[test]
fn native_binding_names_preserve_token_spans_and_omit_assigned_closure_names() -> TestResult {
    let source = "-- café 😀\r\nfunction A . B : Method() end\r\nlocal function LocalOne() end\r\nlocal assigned = function() end\r\n";
    let main = snapshot(vec![LuaWorkspaceFileInput::new("main/names.lua", source)])?;
    let file = main.files().first().ok_or("file")?;
    let tree = LuaParser::parse(source, ParserConfig::with_level(LuaLanguageLevel::Lua54));
    assert!(tree.get_errors().is_empty());
    let chunk = LuaChunk::cast(tree.get_red_root()).ok_or("chunk")?;
    let observed = chunk
        .descendants::<LuaClosureExpr>()
        .map(|closure| names::observe(&closure, file, &AtomicBool::new(false)))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(observed.len(), 3);
    let member = observed[0].name().ok_or("named member")?;
    assert_eq!(member.qualified_name(), "A.B:Method");
    assert_eq!(member.receiver(), Some("A.B"));
    assert_eq!(member.short_name(), "Method");
    assert!(member.colon_definition());
    assert_eq!(source_slice(source, member.name_span())?, "A . B : Method");
    assert_eq!(
        observed[1].name().ok_or("local")?.qualified_name(),
        "LocalOne"
    );
    assert_eq!(
        observed[2].state(),
        SourceFunctionBindingState::AnonymousClosure
    );
    assert!(observed[2].name().is_none());
    Ok(())
}

#[test]
fn native_binding_names_bound_before_copy_and_observe_cancellation() -> TestResult {
    for source in [
        format!("function {}F() end", "A.".repeat(128)),
        format!("function {}() end", "A".repeat(4097)),
    ] {
        let main = snapshot(vec![LuaWorkspaceFileInput::new(
            "main/large-name.lua",
            &source,
        )])?;
        let file = main.files().first().ok_or("file")?;
        let tree = LuaParser::parse(&source, ParserConfig::with_level(LuaLanguageLevel::Lua54));
        assert!(tree.get_errors().is_empty());
        let chunk = LuaChunk::cast(tree.get_red_root()).ok_or("chunk")?;
        let closure = chunk
            .descendants::<LuaClosureExpr>()
            .next()
            .ok_or("closure")?;
        assert_eq!(
            names::observe(&closure, file, &AtomicBool::new(false))?.state(),
            SourceFunctionBindingState::OverCapacity
        );
        assert_eq!(
            names::observe(&closure, file, &AtomicBool::new(true))
                .err()
                .ok_or("cancelled")?
                .code(),
            crate::EmmyMemberCallErrorCode::Cancelled
        );
    }
    Ok(())
}

#[test]
fn native_binding_names_are_separate_from_legacy_report_bytes_and_identity() -> TestResult {
    let main = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/report-name.lua",
        "local function LocalOne() end\nfunction GlobalOne() end\n",
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
    report.validate()?;
    assert_eq!(
        report
            .functions()
            .iter()
            .filter(|f| f.binding_name().is_some())
            .count(),
        2
    );
    let bytes = wow_core::canonical_json_bytes(&report)?;
    let mut stripped = report.clone();
    for fact in &mut stripped.functions {
        fact.binding_name = names::BindingNameObservation::chunk();
    }
    assert_eq!(wow_core::canonical_json_bytes(&stripped)?, bytes);
    assert_eq!(stripped.analysis_id(), report.analysis_id());
    assert!(
        stripped.validate().is_err(),
        "serialized equality cannot reconstruct valid native custody"
    );
    Ok(())
}
