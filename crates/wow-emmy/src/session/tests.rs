use super::*;
use crate::{EmmyBackendIdentity, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceUniverse};

type TestResult = Result<(), Box<dyn std::error::Error>>;

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
