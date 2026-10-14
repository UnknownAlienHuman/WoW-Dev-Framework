use super::*;
use crate::{EmmyBackendIdentity, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceUniverse};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn native_member_failure_retains_typed_source_without_panic_payload() -> TestResult {
    let main = snapshot(&[("main.lua", "return true\n")])?;
    let limits = crate::function_calls::FunctionCallBudget::new(1, 1, 1, 1, 1, 64 * 1024)?;
    let failure = analyze_member_call_session_with_local_flow(
        &main,
        &[],
        None,
        MemberCallSessionQueryProfile::new(&[], &[]).with_function_call_budget(limits),
        true,
        &AtomicBool::new(false),
    )
    .err()
    .ok_or("bounded collection admitted")?;
    assert_eq!(
        failure.code(),
        AnalyzerSessionErrorCode::OutputBudgetExceeded
    );
    let cause = std::error::Error::source(&failure)
        .ok_or("missing member cause")?
        .downcast_ref::<crate::EmmyMemberCallError>()
        .ok_or("wrong typed cause")?;
    assert_eq!(
        cause.code(),
        crate::EmmyMemberCallErrorCode::FactBudgetExceeded
    );
    assert!(failure.to_string().contains(cause.message()));
    let panic = error(AnalyzerSessionErrorCode::NativeFailure);
    assert!(std::error::Error::source(&panic).is_none());
    assert_eq!(
        panic.to_string(),
        "native analysis failed; session unavailable"
    );
    Ok(())
}

fn snapshot(files: &[(&str, &str)]) -> Result<LuaWorkspaceSnapshot, Box<dyn std::error::Error>> {
    Ok(LuaWorkspaceSnapshot::build(
        EmmyBackendIdentity::new(
            "emmylua_code_analysis",
            Some(crate::EMMYLUA_CODE_ANALYSIS_VERSION),
            crate::EMMYLUA_REVISION,
            crate::EMMYLUA_TREE,
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
        )?,
        LuaWorkspaceUniverse::Fixture,
        files
            .iter()
            .map(|(path, text)| LuaWorkspaceFileInput::new(*path, *text))
            .collect(),
        LuaWorkspaceLimits::new(32, 1024, 65536, 524288)?,
    )?)
}

fn green(
    analysis: &EmmyLuaAnalysis,
    root: &Path,
    path: &str,
) -> Result<rowan::GreenNode, Box<dyn std::error::Error>> {
    let uri = file_path_to_uri(&root.join(path)).ok_or("invalid URI")?;
    let id = analysis.get_file_id(&uri).ok_or("missing file")?;
    Ok(analysis
        .compilation
        .get_db()
        .get_vfs()
        .get_syntax_tree(&id)
        .ok_or("missing tree")?
        .get_red_root()
        .green()
        .to_owned())
}

#[test]
fn unchanged_green_allocations_survive_and_removal_drops_uri_mapping() -> TestResult {
    let old = snapshot(&[
        ("main/keep.lua", "local k = 1\nreturn k\n"),
        ("main/change.lua", "local x = 1\n"),
        ("main/remove.lua", "function Removed() end\n"),
    ])?;
    let target = snapshot(&[
        ("main/keep.lua", "local k = 1\nreturn k\n"),
        ("main/change.lua", "local x = 2\n"),
        ("main/add.lua", "local a = 3\n"),
    ])?;
    let library = snapshot(&[("lib/api.lua", "---@meta _\nfunction Api() end\n")])?;
    let stop = AtomicBool::new(false);
    let mut session = AnalyzerSession::open(old.clone(), &[library], &stop)?;
    let syntax_before = green(&session.syntax, &session.syntax_root, "main/keep.lua")?;
    let semantic_before = green(
        &session.semantic.analysis,
        &session.semantic.main_root,
        "main/keep.lua",
    )?;
    let batch =
        AnalyzerUpdateBatch::between(&old, target, ProjectGenerationId::derive(&"green-reuse")?)?;
    session.apply_update(
        batch,
        MemberCallSessionQueryProfile::new(&[], &[]),
        false,
        &stop,
    )?;
    assert!(std::ptr::eq(
        &*syntax_before,
        &*green(&session.syntax, &session.syntax_root, "main/keep.lua")?
    ));
    assert!(std::ptr::eq(
        &*semantic_before,
        &*green(
            &session.semantic.analysis,
            &session.semantic.main_root,
            "main/keep.lua"
        )?
    ));
    for (analysis, root) in [
        (&session.syntax, &session.syntax_root),
        (&session.semantic.analysis, &session.semantic.main_root),
    ] {
        let uri = file_path_to_uri(&root.join("main/remove.lua")).ok_or("invalid URI")?;
        assert!(analysis.get_file_id(&uri).is_none());
    }
    assert_eq!(
        session.last_work(),
        AnalyzerUpdateWork {
            added_files: 1,
            updated_files: 1,
            removed_files: 1,
            reused_files: 1
        }
    );
    Ok(())
}

#[test]
fn failed_mutation_poison_requires_a_fresh_owner() -> TestResult {
    let old = snapshot(&[("main/remove.lua", "local a = 1\n")])?;
    let target = snapshot(&[("main/new.lua", "local b = 2\n")])?;
    let stop = AtomicBool::new(false);
    let mut session = AnalyzerSession::open(old.clone(), &[], &stop)?;
    // A genuinely inconsistent native cache cannot be certified by the batch.
    let uri = file_path_to_uri(&session.semantic.main_root.join("main/remove.lua"))
        .ok_or("invalid URI")?;
    session
        .semantic
        .analysis
        .remove_file_by_uri(&uri)
        .ok_or("missing file")?;
    let batch =
        AnalyzerUpdateBatch::between(&old, target, ProjectGenerationId::derive(&"poison")?)?;
    let result = session.apply_update(
        batch,
        MemberCallSessionQueryProfile::new(&[], &[]),
        false,
        &stop,
    );
    assert_eq!(
        result.err().ok_or("corrupt cache accepted")?.code(),
        AnalyzerSessionErrorCode::NativeFailure
    );
    assert_eq!(
        session
            .reports(MemberCallSessionQueryProfile::new(&[], &[]), false, &stop)
            .err()
            .ok_or("poisoned cache accepted")?
            .code(),
        AnalyzerSessionErrorCode::Poisoned
    );
    Ok(())
}

#[test]
fn selected_query_limit_preserves_session_lifecycle_and_default_refusals() -> TestResult {
    use crate::bindings::SymbolLookupState;
    use crate::function_calls::SourceCallTarget;

    let library = snapshot(&[(
        "lib/api.lua",
        "---@meta _\nfunction RealApi(value) return value end\n",
    )])?;
    let old = snapshot(&[
        (
            "main/keep.lua",
            "function MainCallable() return RealApi(1) end\n",
        ),
        ("main/drop.lua", "return RealApi(2)\n"),
    ])?;
    let target = snapshot(&[(
        "main/keep.lua",
        "function MainCallable() return RealApi(3) end\n",
    )])?;
    let stop = AtomicBool::new(false);
    let mut names = (0..4095)
        .map(|index| format!("FixtureAbsent_{index}"))
        .collect::<Vec<_>>();
    names.extend(["RealApi".to_owned(), "MainCallable".to_owned()]);
    let profile = MemberCallSessionQueryProfile::new(&names, &names).with_query_limit(4097)?;
    assert_eq!(
        MemberCallSessionQueryProfile::new(&[], &[])
            .with_query_limit(0)
            .err()
            .ok_or("zero accepted")?
            .code(),
        crate::EmmyMemberCallErrorCode::FactBudgetExceeded
    );

    // Each default lane refuses independently; a failed extraction poisons its owner.
    for (symbols, callables) in [(names.as_slice(), &[][..]), (&[][..], names.as_slice())] {
        let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
        assert_eq!(
            owner
                .reports(
                    MemberCallSessionQueryProfile::new(symbols, callables),
                    true,
                    &stop
                )
                .err()
                .ok_or("default accepted4097")?
                .code(),
            AnalyzerSessionErrorCode::OutputBudgetExceeded
        );
        assert_eq!(
            owner
                .reports(profile, true, &stop)
                .err()
                .ok_or("poisoned owner accepted")?
                .code(),
            AnalyzerSessionErrorCode::Poisoned
        );
    }
    let mut extra = names.clone();
    extra.push("FixtureLimitPlusOne".into());
    let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
    assert_eq!(
        owner
            .reports(
                MemberCallSessionQueryProfile::new(&extra, &[]).with_query_limit(4097)?,
                false,
                &stop
            )
            .err()
            .ok_or("selected limit+1 accepted")?
            .code(),
        AnalyzerSessionErrorCode::OutputBudgetExceeded
    );

    // Repeats consume input count even though lookup output is distinct.
    let repeats = vec!["RealApi".to_owned(); 4097];
    let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
    assert_eq!(
        owner
            .reports(
                MemberCallSessionQueryProfile::new(&repeats, &[]),
                false,
                &stop
            )
            .err()
            .ok_or("default repeats accepted")?
            .code(),
        AnalyzerSessionErrorCode::OutputBudgetExceeded
    );
    let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
    assert_eq!(
        owner
            .reports(
                MemberCallSessionQueryProfile::new(&repeats[..4096], &[]),
                false,
                &stop
            )?
            .symbol_lookup
            .ok_or("default exact lookup missing")?
            .lookups()
            .len(),
        1
    );
    assert_eq!(
        owner
            .reports(
                MemberCallSessionQueryProfile::new(&repeats, &[]).with_query_limit(4097)?,
                false,
                &stop
            )?
            .symbol_lookup
            .ok_or("selected repeated lookup missing")?
            .lookups()
            .len(),
        1
    );

    let long_names = vec!["x".repeat(4096); 4097];
    let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
    assert_eq!(
        owner
            .reports(
                MemberCallSessionQueryProfile::new(&long_names, &[]).with_query_limit(4097)?,
                false,
                &stop
            )
            .err()
            .ok_or("input text overflow accepted")?
            .code(),
        AnalyzerSessionErrorCode::OutputBudgetExceeded
    );

    let mut owner = AnalyzerSession::open(old.clone(), std::slice::from_ref(&library), &stop)?;
    let reports = owner.reports(profile, true, &stop)?;
    let lookup = reports.symbol_lookup.ok_or("large lookup missing")?;
    assert_eq!(lookup.lookups().len(), 4097);
    assert_eq!(
        lookup.lookups()["RealApi"].state,
        SymbolLookupState::UniqueAnalyzerDeclaration
    );
    let calls = reports.function_calls.ok_or("callable sidecar missing")?;
    assert!(matches!(
        calls.named_targets()["RealApi"],
        SourceCallTarget::LibraryFunction { .. }
    ));
    assert!(matches!(
        calls.named_targets()["MainCallable"],
        SourceCallTarget::MainFunction { .. }
    ));
    assert!(
        calls
            .symbol_lookup_analysis_ids()
            .iter()
            .any(|id| id == lookup.analysis_id())
    );

    let small = vec!["RealApi".into(), "MainCallable".into()];
    let ordinary = owner.reports(
        MemberCallSessionQueryProfile::new(&small, &small),
        true,
        &stop,
    )?;
    let selected = owner.reports(
        MemberCallSessionQueryProfile::new(&small, &small).with_query_limit(4097)?,
        true,
        &stop,
    )?;
    assert_eq!(ordinary.symbol_lookup, selected.symbol_lookup);
    assert_eq!(ordinary.function_calls, selected.function_calls);
    assert_eq!(ordinary.member_calls, selected.member_calls);
    assert_eq!(ordinary.local_flow, selected.local_flow);

    let updated = owner.apply_update(
        AnalyzerUpdateBatch::between(
            &old,
            target.clone(),
            ProjectGenerationId::derive(&"query-limit-update")?,
        )?,
        profile,
        true,
        &stop,
    )?;
    assert_eq!(
        updated
            .function_calls
            .ok_or("updated callable report missing")?
            .main_snapshot_id(),
        target.snapshot_id()
    );
    assert_eq!(
        updated
            .symbol_lookup
            .ok_or("updated lookup missing")?
            .lookups()
            .len(),
        4097
    );
    assert_eq!(
        owner.last_work(),
        AnalyzerUpdateWork {
            added_files: 0,
            updated_files: 1,
            removed_files: 1,
            reused_files: 0
        }
    );

    let virtual_units = snapshot(&[("virtual/inline.lua", "local value = MainCallable()\n")])?;
    let virtual_report = crate::analyze_member_call_session_with_virtual_and_callable_queries(
        &target,
        &[&library],
        &virtual_units,
        ProjectGenerationId::derive(&"query-limit-virtual")?,
        profile,
        true,
        &stop,
    )?;
    assert_eq!(
        virtual_report
            .symbol_lookup
            .ok_or("virtual lookup missing")?
            .lookups()
            .len(),
        4097
    );
    assert_eq!(
        virtual_report
            .virtual_semantics
            .ok_or("virtual semantics missing")?
            .virtual_snapshot_id(),
        virtual_units.snapshot_id()
    );
    assert!(matches!(
        virtual_report
            .function_calls
            .ok_or("virtual callable sidecar missing")?
            .named_targets()["RealApi"],
        SourceCallTarget::LibraryFunction { .. }
    ));

    assert_eq!(
        owner
            .reports(profile, true, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled reports accepted")?
            .code(),
        AnalyzerSessionErrorCode::Cancelled
    );
    assert_eq!(
        crate::analyze_member_call_session_with_virtual_and_callable_queries(
            &target,
            &[&library],
            &virtual_units,
            ProjectGenerationId::derive(&"query-limit-cancelled")?,
            profile,
            true,
            &AtomicBool::new(true)
        )
        .err()
        .ok_or("cancelled virtual accepted")?
        .code(),
        crate::EmmyMemberCallErrorCode::Cancelled
    );
    Ok(())
}

/// The real multi-file native witness for the Issue #135 collector gap.
///
/// The stop is armed through the deterministic after-first-native-call seam, so
/// one native file is diagnosed and the run stops before the next file enters
/// the collector. No timer, sleep, or polling thread is involved, and no extra
/// source callback runs in production.
///
/// This proves cancellation reaches the diagnostic collector and that the
/// between-file stop is typed `Cancelled` rather than a native failure or
/// unavailable diagnostics.
#[test]
fn cancelled_collection_stops_between_native_files() -> TestResult {
    use crate::syntax::analyzer::{
        arm_stop_after_diagnostic_call, diagnostic_calls, reset_diagnostic_calls,
    };
    use std::sync::atomic::AtomicBool;

    let main = snapshot(&[
        ("main/a.lua", "local value = 1\n"),
        ("main/b.lua", "local text = 'x\n"),
        ("main/c.lua", "return 1\n"),
    ])?;
    reset_diagnostic_calls();
    arm_stop_after_diagnostic_call(1);
    let stop = AtomicBool::new(false);
    let error = crate::analyze_syntax_with_stop(&main, &stop)
        .err()
        .ok_or("cancelled collection published a report")?;
    assert_eq!(error.code(), crate::EmmySyntaxErrorCode::Cancelled);
    // The first file entered upstream, and the stop prevented every later file.
    assert_eq!(diagnostic_calls(), 1);
    reset_diagnostic_calls();

    // An operation cancelled before entry never reaches the native collector.
    let stopped = AtomicBool::new(true);
    let error = crate::analyze_syntax_with_stop(&main, &stopped)
        .err()
        .ok_or("pre-cancelled collection published a report")?;
    assert_eq!(error.code(), crate::EmmySyntaxErrorCode::Cancelled);

    // The uncancelled canonical path is unchanged and still reaches every file.
    let running = AtomicBool::new(false);
    let report = crate::analyze_syntax_with_stop(&main, &running)?;
    assert_eq!(report.files().len(), 3);
    assert!(report.files().iter().all(|f| f.status() == "complete"));
    assert_eq!(
        report.analysis_id(),
        crate::analyze_syntax(&main)?.analysis_id()
    );
    Ok(())
}
