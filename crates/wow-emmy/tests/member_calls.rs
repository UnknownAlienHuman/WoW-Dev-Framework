use wow_core::SourceSpan;
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyFactFileStatus, EmmyMemberCallErrorCode, EmmyReferenceResolution, LuaWorkspaceFileInput,
    LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse, analyze_member_calls,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn backend(version: &str) -> TestResult<EmmyBackendIdentity> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(version),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        digest('1'),
        digest('2'),
    )?)
}

fn snapshot(
    universe: LuaWorkspaceUniverse,
    version: &str,
    inputs: Vec<LuaWorkspaceFileInput>,
) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend(version)?,
        universe,
        inputs,
        LuaWorkspaceLimits::new(32, 1024, 64 * 1024, 512 * 1024)?,
    )?)
}

fn fixture_library() -> TestResult<LuaWorkspaceSnapshot> {
    snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/C_E0Fixture.lua",
            "---@meta _\n---@class C_E0Fixture\n---@field KnownApi fun(value: string): string\nC_E0Fixture = {}\n",
        )],
    )
}

fn source_slice(text: &str, span: SourceSpan) -> TestResult<&str> {
    let start = usize::try_from(span.byte_start().ok_or("missing byte start")?)?;
    let end = usize::try_from(span.byte_end().ok_or("missing byte end")?)?;
    text.get(start..end)
        .ok_or_else(|| "invalid source span".into())
}

#[test]
fn direct_member_calls_link_exact_reference_and_call_facts() -> TestResult {
    let source = concat!(
        "local known = C_E0Fixture.KnownApi(\"ok\")\n",
        "local missing = C_E0Fixture.RemovedApi()\n",
    );
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new("main/calls.lua", source)],
    )?;
    let library = fixture_library()?;
    let report = analyze_member_calls(&main, &[&library])?;

    assert_eq!(report.main_snapshot_id(), main.snapshot_id());
    assert_eq!(
        report.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );
    assert_eq!(report.files().len(), 1);
    assert_eq!(report.files()[0].path(), "main/calls.lua");
    assert_eq!(report.files()[0].status(), EmmyFactFileStatus::Complete);
    assert_eq!(report.files()[0].parse_error_count(), 0);
    assert_eq!(report.files()[0].reference_count(), 2);
    assert_eq!(report.files()[0].call_count(), 2);

    let known = report
        .references()
        .iter()
        .find(|fact| fact.member() == "KnownApi")
        .ok_or("KnownApi reference fact")?;
    let missing = report
        .references()
        .iter()
        .find(|fact| fact.member() == "RemovedApi")
        .ok_or("RemovedApi reference fact")?;
    assert_eq!(known.resolution(), EmmyReferenceResolution::Resolved);
    assert_eq!(missing.resolution(), EmmyReferenceResolution::Unresolved);
    assert_eq!(known.receiver(), "C_E0Fixture");
    assert_eq!(missing.receiver(), "C_E0Fixture");
    assert_eq!(source_slice(source, known.receiver_span())?, "C_E0Fixture");
    assert_eq!(source_slice(source, known.member_span())?, "KnownApi");
    assert_eq!(
        source_slice(source, known.reference_span())?,
        "C_E0Fixture.KnownApi"
    );
    assert_eq!(source_slice(source, missing.member_span())?, "RemovedApi");
    assert_eq!(
        source_slice(source, missing.reference_span())?,
        "C_E0Fixture.RemovedApi"
    );

    let known_call = report
        .calls()
        .iter()
        .find(|fact| fact.reference_fact_id() == known.fact_id())
        .ok_or("KnownApi call fact")?;
    let missing_call = report
        .calls()
        .iter()
        .find(|fact| fact.reference_fact_id() == missing.fact_id())
        .ok_or("RemovedApi call fact")?;
    assert_eq!(known_call.argument_count(), Some(1));
    assert_eq!(missing_call.argument_count(), Some(0));
    assert!(!known_call.is_colon_call());
    assert_eq!(
        source_slice(source, known_call.callee_span())?,
        "C_E0Fixture.KnownApi"
    );
    assert_eq!(
        source_slice(source, known_call.call_span())?,
        "C_E0Fixture.KnownApi(\"ok\")"
    );
    assert_eq!(
        source_slice(source, missing_call.call_span())?,
        "C_E0Fixture.RemovedApi()"
    );

    let serialized = serde_json::to_string(&report)?;
    assert!(!serialized.contains(std::env::temp_dir().to_string_lossy().as_ref()));
    assert!(!serialized.contains("API absent"));
    assert!(!serialized.contains("World of Warcraft availability"));
    Ok(())
}

#[test]
fn library_order_does_not_change_fact_or_report_identity() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/calls.lua",
            "return C_E0Fixture.KnownApi(\"ok\")\n",
        )],
    )?;
    let fixture = fixture_library()?;
    let unrelated = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/OtherFixture.lua",
            "---@meta _\n---@class OtherFixture\n---@field Ping fun()\nOtherFixture = {}\n",
        )],
    )?;

    let left = analyze_member_calls(&main, &[&fixture, &unrelated])?;
    let right = analyze_member_calls(&main, &[&unrelated, &fixture])?;
    assert_eq!(left, right);
    assert_eq!(serde_json::to_vec(&left)?, serde_json::to_vec(&right)?);
    Ok(())
}

#[test]
fn broken_library_blocks_resolution_dependent_facts() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/calls.lua",
            "return C_E0Fixture.KnownApi()\n",
        )],
    )?;
    let broken = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "library/broken.lua",
            "local function broken(..., value) end\n",
        )],
    )?;

    let error = analyze_member_calls(&main, &[&broken])
        .err()
        .ok_or("expected library health failure")?;
    assert_eq!(error.code(), EmmyMemberCallErrorCode::LibraryHealthFailed);
    assert_eq!(error.path(), Some("library/broken.lua"));
    let cold_error = wow_emmy::session::analyze_member_call_session_with_local_flow(
        &main,
        &[&broken],
        None,
        wow_emmy::MemberCallSessionQueryProfile::new(&[], &[]).with_member_pair_limit(65_537)?,
        false,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .err()
    .ok_or("cold flow route admitted a broken Library")?;
    assert_eq!(
        cold_error.code(),
        wow_emmy::session::AnalyzerSessionErrorCode::NativeFailure
    );
    Ok(())
}

#[test]
fn malformed_main_file_reports_failed_capability_without_facts() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        EMMYLUA_CODE_ANALYSIS_VERSION,
        vec![LuaWorkspaceFileInput::new(
            "main/broken.lua",
            "local function broken(..., value) end\n",
        )],
    )?;
    let report = analyze_member_calls(&main, &[])?;
    assert_eq!(report.files().len(), 1);
    assert_eq!(report.files()[0].status(), EmmyFactFileStatus::FailedParse);
    assert!(report.files()[0].parse_error_count() > 0);
    assert!(report.references().is_empty());
    assert!(report.calls().is_empty());
    Ok(())
}

#[test]
fn compiled_adapter_rejects_mismatched_backend_before_semantics() -> TestResult {
    let main = snapshot(
        LuaWorkspaceUniverse::Fixture,
        "0.25.0",
        vec![LuaWorkspaceFileInput::new(
            "main/clean.lua",
            "return true\n",
        )],
    )?;
    let error = analyze_member_calls(&main, &[])
        .err()
        .ok_or("expected incompatible backend")?;
    assert_eq!(error.code(), EmmyMemberCallErrorCode::IncompatibleBackend);
    assert!(error.path().is_none());
    Ok(())
}

#[test]
fn selected_member_pair_limit_bounds_physical_virtual_and_combined_reports() -> TestResult {
    use std::sync::atomic::AtomicBool;
    use wow_core::ProjectGenerationId;
    use wow_emmy::{
        MemberCallSessionQueryProfile,
        references::{
            analyze_member_call_session_with_callable_queries,
            analyze_member_call_session_with_virtual_and_callable_queries,
        },
        session::analyze_member_call_session_with_local_flow,
    };

    let stop = AtomicBool::new(false);
    let profile = MemberCallSessionQueryProfile::new(&[], &[]);
    assert_eq!(
        profile
            .with_member_pair_limit(0)
            .err()
            .ok_or("zero limit admitted")?
            .code(),
        EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    let build = |inputs| -> TestResult<LuaWorkspaceSnapshot> {
        Ok(LuaWorkspaceSnapshot::build(
            backend(EMMYLUA_CODE_ANALYSIS_VERSION)?,
            LuaWorkspaceUniverse::Fixture,
            inputs,
            LuaWorkspaceLimits::new(8, 16_384, 1024 * 1024, 4 * 1024 * 1024)?,
        )?)
    };
    let library = build(vec![LuaWorkspaceFileInput::new(
        "library/RealLib.lua",
        "---@meta _\nRealLib = {}\nfunction RealLib.Step(value) return value end\n",
    )])?;
    let large = build(vec![
        LuaWorkspaceFileInput::new("main/a.lua", "RealLib.Step(1)\n".repeat(32_768)),
        LuaWorkspaceFileInput::new("main/b.lua", "RealLib.Step(1)\n".repeat(32_769)),
    ])?;
    let small = build(vec![LuaWorkspaceFileInput::new(
        "main/small.lua",
        "RealLib.Step(1)\n",
    )])?;
    let pairs = 65_537;
    let selected = profile.with_member_pair_limit(pairs)?;
    let refused = analyze_member_call_session_with_callable_queries(
        &large,
        &[&library],
        profile,
        false,
        &stop,
    )
    .err()
    .ok_or("ordinary limit+1 admitted")?;
    assert_eq!(refused.code(), EmmyMemberCallErrorCode::FactBudgetExceeded);
    assert_eq!(refused.path(), Some("main/b.lua"));
    assert_eq!(
        refused.message(),
        "direct-member fact count exceeds the adapter budget"
    );

    // The cold project consumer uses this same registered analysis and physical
    // member prerequisite; no second default member collector can reimpose65536.
    let (exact, flow) = analyze_member_call_session_with_local_flow(
        &large,
        &[&library],
        None,
        selected,
        false,
        &stop,
    )?;
    assert_eq!(exact.member_calls.references().len(), pairs);
    assert_eq!(exact.member_calls.calls().len(), pairs);
    assert!(
        exact
            .member_calls
            .references()
            .iter()
            .all(|reference| reference.resolution() == EmmyReferenceResolution::Resolved)
    );
    assert_eq!(exact.member_calls.main_snapshot_id(), large.snapshot_id());
    assert_eq!(flow.main_snapshot_id(), large.snapshot_id());
    assert!(flow.bindings().is_empty());

    let generation = ProjectGenerationId::derive(&"member-pair-limit-fixture")?;
    let combined = analyze_member_call_session_with_virtual_and_callable_queries(
        &large,
        &[&library],
        &small,
        generation,
        selected,
        false,
        &stop,
    )
    .err()
    .ok_or("combined limit+1 admitted")?;
    assert_eq!(combined.code(), EmmyMemberCallErrorCode::FactBudgetExceeded);
    assert_eq!(combined.path(), None);
    assert_eq!(
        combined.message(),
        "physical and virtual direct-member facts exceed the session budget"
    );

    // Reverse roles: the virtual collector itself crosses the original cap,
    // while physical+virtual is admitted exactly at the selected shared limit.
    let virtual_exact = analyze_member_call_session_with_virtual_and_callable_queries(
        &small,
        &[&library],
        &large,
        generation,
        profile.with_member_pair_limit(pairs + 1)?,
        false,
        &stop,
    )?;
    let virtual_report = virtual_exact
        .virtual_semantics
        .ok_or("virtual report missing")?;
    assert_eq!(virtual_exact.member_calls.calls().len(), 1);
    assert_eq!(
        virtual_report.member_call_report().references().len(),
        pairs
    );
    assert_eq!(virtual_report.member_call_report().calls().len(), pairs);
    assert_eq!(virtual_report.project_generation(), generation);

    let cancelled = AtomicBool::new(true);
    assert_eq!(
        analyze_member_call_session_with_callable_queries(
            &large,
            &[&library],
            selected,
            false,
            &cancelled,
        )
        .err()
        .ok_or("cancelled report admitted")?
        .code(),
        EmmyMemberCallErrorCode::Cancelled
    );
    let ordinary_small = analyze_member_call_session_with_callable_queries(
        &small,
        &[&library],
        profile,
        false,
        &stop,
    )?;
    let (selected_small, selected_flow) = analyze_member_call_session_with_local_flow(
        &small,
        &[&library],
        None,
        selected,
        false,
        &stop,
    )?;
    assert_eq!(ordinary_small.member_calls, selected_small.member_calls);
    assert_eq!(
        serde_json::to_vec(&ordinary_small.member_calls)?,
        serde_json::to_vec(&selected_small.member_calls)?
    );
    assert_eq!(ordinary_small.symbol_lookup, selected_small.symbol_lookup);
    assert_eq!(ordinary_small.function_calls, selected_small.function_calls);
    assert_eq!(
        ordinary_small.virtual_semantics,
        selected_small.virtual_semantics
    );
    assert_eq!(
        selected_flow,
        wow_emmy::analyze_local_flow(&small, &[&library])?
    );
    Ok(())
}
