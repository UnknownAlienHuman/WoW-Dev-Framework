use super::*;
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn input(text: &str) -> TestResult<LuaWorkspaceSnapshot> {
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
        LuaWorkspaceUniverse::Project,
        vec![LuaWorkspaceFileInput::new("main.lua", text)],
        LuaWorkspaceLimits::new(1, 4096, 1_048_576, 1_048_576)?,
    )?)
}

fn limits() -> LocalReferencesLimits {
    LocalReferencesLimits {
        max_file_bytes: 1_048_576,
        max_work: 8_388_608,
        max_name_bytes: 256,
        max_references: 2048,
    }
}

fn occurrence(source: &str, needle: &str, index: usize) -> TestResult<usize> {
    source
        .match_indices(needle)
        .nth(index)
        .map(|(start, _)| start)
        .ok_or_else(|| "expected source occurrence is missing".into())
}

fn expected_span(source: &str, needle: &str, index: usize) -> TestResult<SourceSpan> {
    let start = occurrence(source, needle, index)?;
    let end = start
        .checked_add(needle.len())
        .ok_or("expected span overflow")?;
    Ok(SourceSpan::byte_range(
        u64::try_from(start)?,
        u64::try_from(end)?,
    )?)
}

#[test]
fn native_index_preserves_shadowing_closures_parameters_and_local_functions() -> TestResult {
    let source = "local value = 0\n\
local function outer(param)\n\
    local value = param\n\
    local function inner()\n\
        local value = 2\n\
        return value, param\n\
    end\n\
    return value, inner()\n\
end\n\
return value, outer(1)\n";
    let snapshot = input(source)?;
    let stop = AtomicBool::new(false);

    let inner_read = occurrence(source, "value", 3)?;
    let inner = local_references(&snapshot, inner_read, false, limits(), &stop)?;
    assert_eq!(inner.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(
        inner.declaration_span,
        Some(expected_span(source, "value", 2)?)
    );
    assert_eq!(inner.references, vec![expected_span(source, "value", 3)?]);

    let outer_read = occurrence(source, "value", 4)?;
    let outer = local_references(&snapshot, outer_read, false, limits(), &stop)?;
    assert_eq!(outer.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(
        outer.declaration_span,
        Some(expected_span(source, "value", 1)?)
    );
    assert_eq!(outer.references, vec![expected_span(source, "value", 4)?]);

    let top_read = occurrence(source, "value", 5)?;
    let top = local_references(&snapshot, top_read, false, limits(), &stop)?;
    assert_eq!(top.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(
        top.declaration_span,
        Some(expected_span(source, "value", 0)?)
    );
    assert_eq!(top.references, vec![expected_span(source, "value", 5)?]);

    let param_read = occurrence(source, "param", 2)?;
    let parameter = local_references(&snapshot, param_read, false, limits(), &stop)?;
    assert_eq!(parameter.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(
        parameter.declaration_span,
        Some(expected_span(source, "param", 0)?)
    );
    assert_eq!(
        parameter.references,
        vec![
            expected_span(source, "param", 1)?,
            expected_span(source, "param", 2)?,
        ]
    );

    let inner_call = occurrence(source, "inner", 1)?;
    let function = local_references(&snapshot, inner_call, false, limits(), &stop)?;
    assert_eq!(function.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(
        function.declaration_span,
        Some(expected_span(source, "inner", 0)?)
    );
    assert_eq!(
        function.references,
        vec![expected_span(source, "inner", 1)?]
    );

    let repeated = local_references(&snapshot, inner_read, false, limits(), &stop)?;
    assert_eq!(inner.analysis_id, repeated.analysis_id);
    assert_eq!(serde_json::to_vec(&inner)?, serde_json::to_vec(&repeated)?);
    assert!(inner.references.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(!inner.negative_authority);
    Ok(())
}

#[test]
fn declaration_is_opt_in_and_unreferenced_local_is_observed_without_negative_authority()
-> TestResult {
    let source = "local lonely = 1\nreturn 0\n";
    let snapshot = input(source)?;
    let stop = AtomicBool::new(false);
    let query = occurrence(source, "lonely", 0)?;
    let declaration = expected_span(source, "lonely", 0)?;

    let omitted = local_references(&snapshot, query, false, limits(), &stop)?;
    assert_eq!(omitted.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(omitted.declaration_span, Some(declaration));
    assert!(omitted.references.is_empty());
    assert!(!omitted.negative_authority);

    let included = local_references(&snapshot, query, true, limits(), &stop)?;
    assert_eq!(included.state, LocalReferencesState::ResolvedLocal);
    assert_eq!(included.references, vec![declaration]);
    let serialized = serde_json::to_string(&included)?;
    assert!(!serialized.contains("lonely"));
    assert!(!serialized.contains("decl_id"));
    Ok(())
}

#[test]
fn used_local_references_are_sorted_unique_and_include_declaration_only_on_request() -> TestResult {
    let source = "local x = 1\nreturn x, x\n";
    let snapshot = input(source)?;
    let stop = AtomicBool::new(false);
    let query = occurrence(source, "x", 1)?;
    let expected_uses = vec![
        expected_span(source, "x", 1)?,
        expected_span(source, "x", 2)?,
    ];

    let omitted = local_references(&snapshot, query, false, limits(), &stop)?;
    assert_eq!(omitted.references, expected_uses);

    let included = local_references(&snapshot, query, true, limits(), &stop)?;
    assert_eq!(
        included.references,
        vec![
            expected_span(source, "x", 0)?,
            expected_span(source, "x", 1)?,
            expected_span(source, "x", 2)?,
        ]
    );
    assert!(!included.negative_authority);
    Ok(())
}

#[test]
fn globals_members_parse_errors_and_invalid_utf8_offsets_stay_conservative() -> TestResult {
    let stop = AtomicBool::new(false);
    let global_source = "Global = 1\nreturn Global\n";
    let global = input(global_source)?;
    let global_report = local_references(
        &global,
        occurrence(global_source, "Global", 0)?,
        false,
        limits(),
        &stop,
    )?;
    assert_eq!(global_report.state, LocalReferencesState::NotEvaluated);
    assert!(global_report.declaration_span.is_none());
    assert!(!global_report.negative_authority);

    let member_source = "local object = {}\nreturn object.member\n";
    let member = input(member_source)?;
    let member_report = local_references(
        &member,
        occurrence(member_source, "member", 0)?,
        false,
        limits(),
        &stop,
    )?;
    assert_eq!(member_report.state, LocalReferencesState::NotEvaluated);
    assert!(member_report.declaration_span.is_none());

    let malformed = input("local =")?;
    assert_eq!(
        local_references(&malformed, 0, false, limits(), &stop)?.state,
        LocalReferencesState::SourceParseFailed
    );

    let unicode = input("--😀\nlocal x = 1\nreturn x")?;
    assert_eq!(
        local_references(&unicode, 3, false, limits(), &stop)
            .err()
            .map(|error| error.code()),
        Some(LocalReferencesErrorCode::InvalidPosition)
    );
    Ok(())
}

#[test]
fn cancellation_and_budget_exhaustion_refuse_the_whole_request() -> TestResult {
    let source = "local x = 1\nreturn x, x\n";
    let snapshot = input(source)?;
    let query = occurrence(source, "x", 1)?;
    let stopped = AtomicBool::new(true);
    assert_eq!(
        local_references(&snapshot, query, false, limits(), &stopped)
            .err()
            .map(|error| error.code()),
        Some(LocalReferencesErrorCode::Cancelled)
    );

    let stop = AtomicBool::new(false);
    let mut one_reference = limits();
    one_reference.max_references = 1;
    assert_eq!(
        local_references(&snapshot, query, false, one_reference, &stop)
            .err()
            .map(|error| error.code()),
        Some(LocalReferencesErrorCode::Limit)
    );

    let mut tiny_file = limits();
    tiny_file.max_file_bytes = 1;
    assert_eq!(
        local_references(&snapshot, query, false, tiny_file, &stop)
            .err()
            .map(|error| error.code()),
        Some(LocalReferencesErrorCode::Limit)
    );
    Ok(())
}
