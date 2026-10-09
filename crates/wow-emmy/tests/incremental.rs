//! Incremental analyzer-session parity against the independent cold APIs.
//!
//! Each test builds a target snapshot, opens a session over the previous state,
//! applies the derived delta, and requires every extracted report to equal the
//! report the existing one-shot owner produces from the same final snapshot.
use std::sync::atomic::AtomicBool;

use wow_core::ProjectGenerationId;
use wow_emmy::session::{
    AnalyzerSession, AnalyzerSessionErrorCode, AnalyzerSessionReports, AnalyzerUpdateBatch,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    MemberCallSessionQueryProfile, analyze_local_flow, analyze_member_calls, analyze_syntax,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const READER: &str = concat!(
    "local function read()\n",
    "    return C_E0Fixture.KnownApi(\"ok\")\n",
    "end\n",
    "return read\n",
);

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

fn snapshot(inputs: Vec<LuaWorkspaceFileInput>) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend()?,
        LuaWorkspaceUniverse::Fixture,
        inputs,
        LuaWorkspaceLimits::new(32, 1024, 64 * 1024, 512 * 1024)?,
    )?)
}

fn library() -> TestResult<LuaWorkspaceSnapshot> {
    snapshot(vec![LuaWorkspaceFileInput::new(
        "library/C_E0Fixture.lua",
        concat!(
            "---@meta _\n",
            "---@class C_E0Fixture\n",
            "---@field KnownApi fun(value: string): string\n",
            "C_E0Fixture = {}\n",
        ),
    )])
}

fn generation(label: &str) -> TestResult<ProjectGenerationId> {
    Ok(ProjectGenerationId::derive(&format!(
        "incremental-emmy-session-{label}"
    ))?)
}

fn queries() -> Vec<String> {
    Vec::new()
}

/// Compare every extracted surface against the independent one-shot owners.
fn assert_cold_parity(
    incremental: &AnalyzerSessionReports,
    target: &LuaWorkspaceSnapshot,
    library: &LuaWorkspaceSnapshot,
) -> TestResult {
    assert_cold_parity_with_queries(
        incremental,
        target,
        library,
        MemberCallSessionQueryProfile::new(&[], &[]),
    )
}

fn assert_cold_parity_with_queries(
    incremental: &AnalyzerSessionReports,
    target: &LuaWorkspaceSnapshot,
    library: &LuaWorkspaceSnapshot,
    queries: MemberCallSessionQueryProfile<'_>,
) -> TestResult {
    let libraries = [library];
    let cold_syntax = analyze_syntax(target)?;
    let cold_members = analyze_member_calls(target, &libraries)?;
    let cold_flow = analyze_local_flow(target, &libraries)?;
    assert_eq!(incremental.syntax, cold_syntax);
    assert_eq!(incremental.member_calls, cold_members);
    assert_eq!(incremental.local_flow, cold_flow);
    let cold = wow_emmy::references::analyze_member_call_session_with_callable_queries(
        target,
        &libraries,
        queries,
        incremental.function_calls.is_some(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(incremental.function_calls, cold.function_calls);
    assert_eq!(incremental.symbol_lookup, cold.symbol_lookup);
    Ok(())
}

/// Open a session over `previous`, apply the derived batch to `target`, and
/// compare the result with an independent cold build of the same target.
fn transition(
    previous: &LuaWorkspaceSnapshot,
    target: LuaWorkspaceSnapshot,
    library: &LuaWorkspaceSnapshot,
) -> TestResult<AnalyzerSessionReports> {
    let stop = AtomicBool::new(false);
    let queries = queries();
    let profile = MemberCallSessionQueryProfile::new(&queries, &queries);
    let mut session =
        AnalyzerSession::open(previous.clone(), std::slice::from_ref(library), &stop)?;
    let batch = AnalyzerUpdateBatch::between(previous, target.clone(), generation("target")?)?;
    let reports = session.apply_update(batch, profile, true, &stop)?;
    assert_eq!(session.main().snapshot_id(), target.snapshot_id());
    assert_cold_parity(&reports, &target, library)?;
    Ok(reports)
}

// Add, update and remove in one delta. Reextracted reports must equal the
// independent cold build of the identical final snapshot.
#[test]
fn add_update_and_remove_match_a_cold_target_build() -> TestResult {
    let library = library()?;
    let previous = snapshot(vec![
        LuaWorkspaceFileInput::new("main/stale.lua", "local value = 1\n"),
        LuaWorkspaceFileInput::new("main/kept.lua", "local old = 1\n"),
    ])?;
    let target = snapshot(vec![
        LuaWorkspaceFileInput::new("main/added.lua", READER),
        LuaWorkspaceFileInput::new(
            "main/kept.lua",
            concat!(
                "local function kept()\n",
                "    return C_E0Fixture.KnownApi(\"kept\")\n",
                "end\n",
                "return kept\n",
            ),
        ),
    ])?;
    let reports = transition(&previous, target.clone(), &library)?;
    assert_eq!(reports.member_calls.files().len(), 2);
    assert_eq!(reports.syntax.files().len(), 2);
    assert_eq!(
        reports
            .member_calls
            .references()
            .iter()
            .filter(|fact| fact.member() == "KnownApi")
            .count(),
        2
    );
    Ok(())
}

// A removed Main file that defined a global must leave no stale entry, and the
// unchanged consumer in another file must match a cold
// build of the same final snapshot reports it.
#[test]
fn removing_a_cross_file_global_definition_matches_cold_parity() -> TestResult {
    let library = library()?;
    let definition = concat!(
        "---@class SharedDefinition\n",
        "---@field KnownApi fun(value: string): string\n",
        "SharedDefinition = {}\n",
        "return SharedDefinition\n",
    );
    let consumer = concat!(
        "local function consume()\n",
        "    return SharedDefinition.KnownApi(\"ok\")\n",
        "end\n",
        "return consume\n",
    );
    let previous = snapshot(vec![
        LuaWorkspaceFileInput::new("main/definition.lua", definition),
        LuaWorkspaceFileInput::new("main/consumer.lua", consumer),
    ])?;
    let query = vec!["SharedDefinition".to_owned()];
    let profile = MemberCallSessionQueryProfile::new(&query, &[]);
    let stop = AtomicBool::new(false);
    let mut session =
        AnalyzerSession::open(previous.clone(), std::slice::from_ref(&library), &stop)?;
    let baseline = session.reports(profile, true, &stop)?;
    assert_eq!(
        baseline
            .symbol_lookup
            .as_ref()
            .ok_or("missing initial lookup")?
            .lookups()["SharedDefinition"]
            .state,
        wow_emmy::bindings::SymbolLookupState::UniqueAnalyzerDeclaration
    );
    let target = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/consumer.lua",
        consumer,
    )])?;
    let batch =
        AnalyzerUpdateBatch::between(&previous, target.clone(), generation("remove-definition")?)?;
    let reports = session.apply_update(batch, profile, true, &stop)?;
    assert_cold_parity_with_queries(&reports, &target, &library, profile)?;
    assert_eq!(reports.member_calls.files().len(), 1);
    assert_eq!(reports.local_flow.files().len(), 1);
    let lookup = &reports
        .symbol_lookup
        .as_ref()
        .ok_or("missing target lookup")?
        .lookups()["SharedDefinition"];
    assert_eq!(
        lookup.state,
        wow_emmy::bindings::SymbolLookupState::NotObserved
    );
    assert!(lookup.targets.is_empty());
    // One fact per unchanged consumer file, with no definition-side residue.
    let cold = analyze_member_calls(&target, &[&library])?;
    assert_eq!(reports.member_calls, cold);
    Ok(())
}

// Removing and re-adding a path with new bytes converges on the final state.
// Cold parity is the acceptance criteria; no file ID is compared.
#[test]
fn readding_a_path_with_new_bytes_matches_cold_parity() -> TestResult {
    let library = library()?;
    let original = "local value = 1\n";
    let replacement = concat!(
        "local function read()\n",
        "    return C_E0Fixture.KnownApi(\"readded\")\n",
        "end\n",
        "return read\n",
    );
    let previous = snapshot(vec![LuaWorkspaceFileInput::new("main/slot.lua", original)])?;
    let removed = snapshot(Vec::new())?;
    let stop = AtomicBool::new(false);
    let mut session =
        AnalyzerSession::open(previous.clone(), std::slice::from_ref(&library), &stop)?;
    let profile = MemberCallSessionQueryProfile::new(&[], &[]);
    let remove = AnalyzerUpdateBatch::between(&previous, removed.clone(), generation("remove")?)?;
    let reports = session.apply_update(remove, profile, true, &stop)?;
    assert_cold_parity(&reports, &removed, &library)?;
    assert_eq!(reports.member_calls.files().len(), 0);
    let readded = snapshot(vec![LuaWorkspaceFileInput::new(
        "main/slot.lua",
        replacement,
    )])?;
    let add = AnalyzerUpdateBatch::between(&removed, readded.clone(), generation("readd")?)?;
    let reports = session.apply_update(add, profile, true, &stop)?;
    assert_cold_parity(&reports, &readded, &library)?;
    assert_eq!(reports.member_calls.files().len(), 1);
    assert_eq!(
        reports
            .member_calls
            .references()
            .iter()
            .filter(|fact| fact.member() == "KnownApi")
            .count(),
        1
    );
    Ok(())
}

// A batch naming a previous snapshot this session never published is refused
// before either VFS mutates, so the session keeps serving its last state.
#[test]
fn stale_batch_is_refused_without_mutating_the_session() -> TestResult {
    let library = library()?;
    let previous = snapshot(vec![LuaWorkspaceFileInput::new("main/one.lua", READER)])?;
    let target = snapshot(vec![LuaWorkspaceFileInput::new("main/two.lua", READER)])?;
    let stop = AtomicBool::new(false);
    let queries = queries();
    let profile = MemberCallSessionQueryProfile::new(&queries, &queries);
    let mut session =
        AnalyzerSession::open(previous.clone(), std::slice::from_ref(&library), &stop)?;
    let foreign = snapshot(vec![LuaWorkspaceFileInput::new("main/foreign.lua", READER)])?;
    let stale = AnalyzerUpdateBatch::between(&foreign, target.clone(), generation("stale")?)?;
    let error = session
        .apply_update(stale, profile, false, &stop)
        .err()
        .ok_or("a batch naming another previous snapshot applied")?;
    assert_eq!(error.code(), AnalyzerSessionErrorCode::StaleSnapshot);
    assert_eq!(session.main().snapshot_id(), previous.snapshot_id());
    // The session remains usable for its own real target.
    let real = AnalyzerUpdateBatch::between(&previous, target.clone(), generation("real")?)?;
    let reports = session.apply_update(real, profile, false, &stop)?;
    assert_cold_parity(&reports, &target, &library)?;
    Ok(())
}

// Cancelling before the transition applies stops before any report is built.
#[test]
fn pre_cancelled_update_reports_cancellation() -> TestResult {
    let library = library()?;
    let previous = snapshot(vec![LuaWorkspaceFileInput::new("main/one.lua", READER)])?;
    let target = snapshot(vec![LuaWorkspaceFileInput::new("main/two.lua", READER)])?;
    let stop = AtomicBool::new(true);
    let queries = queries();
    let profile = MemberCallSessionQueryProfile::new(&queries, &queries);
    let session = AnalyzerSession::open(previous.clone(), std::slice::from_ref(&library), &stop)
        .err()
        .ok_or("pre-cancelled session analyzed input")?;
    assert_eq!(session.code(), AnalyzerSessionErrorCode::Cancelled);
    // With the session unusable, reopen healthily and confirm a cancelled apply.
    let live = AtomicBool::new(false);
    let mut session = AnalyzerSession::open(previous.clone(), &[library], &live)?;
    let batch = AnalyzerUpdateBatch::between(&previous, target, generation("cancel")?)?;
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let error = session
        .apply_update(batch, profile, false, &stop)
        .err()
        .ok_or("cancelled apply produced a report")?;
    assert_eq!(error.code(), AnalyzerSessionErrorCode::Cancelled);
    Ok(())
}
