//! Structural mutation fixtures for the E2-B core recognizer families.
//!
//! Each case applies one Mutation/evaluation-contract vector to an exact fact
//! bundle and asserts the declared outcome. The bundle is produced by the real
//! Emmy analyzer, never hand-written, so mutating the Lua source mutates the
//! observed structure.

use std::error::Error;
use std::sync::atomic::AtomicBool;

use wow_emmy::references::analyze_member_call_session;
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, LuaWorkspaceFileInput,
    LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn sha(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn workspace(path: &str, text: &str) -> TestResult<LuaWorkspaceSnapshot> {
    let backend = wow_emmy::EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        sha('1'),
        sha('2'),
    )?;
    Ok(LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(path, text)],
        LuaWorkspaceLimits::new(16, 1024, 64 * 1024, 256 * 1024)?,
    )?)
}

fn callable_keys(text: &str) -> TestResult<Vec<String>> {
    let main = workspace("main/events.lua", text)?;
    let stop = AtomicBool::new(false);
    let queries = vec![String::from("EventRegistry.RegisterCallback")];
    let session = analyze_member_call_session(&main, &[], &queries, true, &stop)?;
    let report = session
        .function_calls
        .ok_or("the function-call sidecar is required")?;
    Ok(report
        .calls()
        .iter()
        .filter_map(|call| call.resolved_callable_key())
        .map(str::to_owned)
        .collect())
}

const POSITIVE: &str = "EventRegistry = {}\nfunction EventRegistry:RegisterCallback(key, callback) end\nlocal function onEvent() end\nEventRegistry:RegisterCallback(\"Fixture.Event\", onEvent)\n";

const MUTATED: &str = "EventRegistry = {}\nfunction EventRegistry:RegisterCallback(key, callback) end\nlocal function onEvent() end\nOtherRegistry:RegisterCallback(\"Fixture.Event\", onEvent)\n";

/// RECOG-MUT-004: changing the decisive convention literal must stop the match.
#[test]
fn decisive_convention_literal_stops_the_match() -> TestResult {
    let positive = callable_keys(POSITIVE)?;
    assert!(
        positive.contains(&"EventRegistry.RegisterCallback".to_owned()),
        "the positive fixture must resolve the decisive literal, got {positive:?}"
    );
    let mutated = callable_keys(MUTATED)?;
    assert!(
        !mutated.contains(&"EventRegistry.RegisterCallback".to_owned()),
        "the mutated fixture must not resolve the decisive literal, got {mutated:?}"
    );
    Ok(())
}
