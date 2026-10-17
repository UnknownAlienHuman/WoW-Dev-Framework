use super::*;
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn input(text: &str) -> Result<LuaWorkspaceSnapshot, Box<dyn std::error::Error>> {
    // Synthetic report/surface metadata for the genuine compiled adapter.
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
        LuaWorkspaceLimits::new(1, 4096, 1048576, 1048576)?,
    )?)
}
fn limits() -> LocalDefinitionLimits {
    LocalDefinitionLimits {
        max_file_bytes: 1048576,
        max_work: 8388608,
        max_name_bytes: 256,
    }
}
#[test]
fn actual_native_lexical_shadow_parameter_and_function_targets_use_original_bytes() -> TestResult {
    let source = "--😀\r\nlocal item = 1\r\ndo local item = 2; print(item) end\r\nlocal function f(arg) return arg end\r\nprint(item, f)\r\n";
    let input = input(source)?;
    let stop = AtomicBool::new(false);
    let cases = [
        (
            source.find("print(item) end").ok_or("query")? + 6,
            source.find("local item = 2").ok_or("target")? + 6,
            "item",
        ),
        (
            source.rfind("item, f").ok_or("query")?,
            source.find("local item = 1").ok_or("target")? + 6,
            "item",
        ),
        (
            source.find("return arg").ok_or("query")? + 7,
            source.find("f(arg)").ok_or("target")? + 2,
            "arg",
        ),
        (
            source.rfind(", f").ok_or("query")? + 2,
            source.find("function f").ok_or("target")? + 9,
            "f",
        ),
    ];
    for (offset, start, name) in cases {
        let report = local_definition(&input, offset, limits(), &stop)?;
        assert_eq!(report.state, LocalDefinitionState::ResolvedLocal, "{name}");
        let target = report.target.as_ref().ok_or("missing native target")?;
        assert_eq!(target.name.as_ref(), name);
        assert_eq!(target.span.byte_start(), Some(start as u64));
        assert_eq!(target.span.byte_end(), Some((start + name.len()) as u64));
        assert!(!report.negative_authority);
        assert_eq!(
            report.analysis_id,
            local_definition(&input, offset, limits(), &stop)?.analysis_id
        );
    }
    Ok(())
}
#[test]
fn nonlocals_parse_failure_invalid_utf8_boundary_and_stop_never_become_clean_negative() -> TestResult
{
    let stop = AtomicBool::new(false);
    for source in [
        "Global = 1; print(Global)",
        "local obj = {}; print(obj.member)",
        "print(missing)",
    ] {
        let input = input(source)?;
        let offset = source.rfind('(').ok_or("query")? + 1;
        let offset = if source.contains(".member") {
            source.find("member").ok_or("query")?
        } else {
            offset
        };
        let report = local_definition(&input, offset, limits(), &stop)?;
        assert_eq!(report.state, LocalDefinitionState::NotEvaluated);
        assert!(report.target.is_none());
        assert!(!report.negative_authority);
    }
    let broken = input("local =")?;
    assert_eq!(
        local_definition(&broken, 0, limits(), &stop)?.state,
        LocalDefinitionState::SourceParseFailed
    );
    let unicode = input("--😀\nlocal x = 1\nprint(x)")?;
    assert_eq!(
        local_definition(&unicode, 3, limits(), &stop)
            .err()
            .map(|e| e.code()),
        Some(LocalDefinitionErrorCode::InvalidPosition)
    );
    let mut tiny = limits();
    tiny.max_file_bytes = 1;
    assert_eq!(
        local_definition(&unicode, 0, tiny, &stop)
            .err()
            .map(|e| e.code()),
        Some(LocalDefinitionErrorCode::Limit)
    );
    stop.store(true, Ordering::Release);
    assert_eq!(
        local_definition(&unicode, 0, limits(), &stop)
            .err()
            .map(|e| e.code()),
        Some(LocalDefinitionErrorCode::Cancelled)
    );
    Ok(())
}
