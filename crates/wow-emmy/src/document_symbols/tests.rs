use super::*;
use crate::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn snapshot(source: &str) -> Result<LuaWorkspaceSnapshot, Box<dyn std::error::Error>> {
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
        vec![LuaWorkspaceFileInput::new("main.lua", source)],
        LuaWorkspaceLimits::new(1, 4096, 1048576, 1048576)?,
    )?)
}
fn limits() -> DocumentSymbolsLimits {
    DocumentSymbolsLimits {
        max_file_bytes: 1048576,
        max_work: 8388608,
        max_nodes: 131072,
        max_symbols: 2048,
        max_depth: 12,
        max_ast_depth: 256,
        max_name_bytes: 1024,
        max_detail_bytes: 2048,
        max_report_bytes: 1048576,
    }
}
#[test]
fn native_donor_families_emit_qualified_original_ranges() -> TestResult {
    let stop = AtomicBool::new(false);
    let cases = [
        ("local", "local x=1"),
        (
            "closure",
            "local a,f=1,function(p,...) local inner=p return inner end",
        ),
        (
            "table",
            "local t={key=1,callback=function(q) local z=q return z end}",
        ),
        (
            "function",
            "function Exposed.method(first) local v=first return v end",
        ),
        (
            "loops",
            "for i=1,2 do local per=i end; for k,v in pairs({}) do local item=v end",
        ),
        (
            "branches",
            "if true then local a=1 elseif false then local b=2 else local c=3 end; do local d=4 end",
        ),
        (
            "region",
            "--region OutlineC0\n--😀\nlocal x=1\n--endregion\n",
        ),
    ];
    for (case, source) in cases {
        let input = snapshot(source)?;
        let outcome = document_symbols(&input, limits(), &stop);
        let report =
            outcome.map_err(|error| format!("native outline case {case}: {:?}", error.code()))?;
        assert_eq!(
            report.state,
            DocumentSymbolsState::Ready,
            "{case}: {:?}",
            report.reason
        );
        assert!(!report.symbols.is_empty(), "{case}");
        for (index, symbol) in report.symbols.iter().enumerate() {
            let start = symbol.range.byte_start().ok_or("range")? as usize;
            let end = symbol.range.byte_end().ok_or("range")? as usize;
            assert!(source.get(start..end).is_some(), "{case}");
            assert!(
                symbol.selection_range.byte_start() >= symbol.range.byte_start(),
                "{case}"
            );
            assert!(
                symbol.selection_range.byte_end() <= symbol.range.byte_end(),
                "{case}"
            );
            assert!(
                symbol.parent_index.is_none_or(|parent| parent < index),
                "{case}"
            );
        }
    }
    Ok(())
}
