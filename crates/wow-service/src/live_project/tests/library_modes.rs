use super::*;

fn input_with_library(
    files: Vec<ProjectInputFile>,
    library: LuaWorkspaceSnapshot,
) -> TestResult<crate::LocalProjectInput> {
    let base = input_bundle_with_plans(files.clone(), None, None)?;
    let bundle = ProjectInputBundle::closed(base.configuration().clone(), files, vec![library])?;
    let reference = wow_reference::ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    Ok(crate::LocalProjectInput::new(bundle, reference)?)
}

#[test]
fn library_modes_keep_replace_and_clear_preserve_exact_intent() -> TestResult {
    let stop = AtomicBool::new(false);
    let store_root = root("library-modes")?;
    let graph = crate::graph::GraphBuildRequest::new("fixture-live-pair".into(), "current".into())?;
    let files = vec![ProjectInputFile::new("main.lua", "return External()\n")?];
    let base = input_bundle_with_plans(files.clone(), None, None)?;
    let original_library = base.libraries()[0].clone();
    let replacement = LuaWorkspaceSnapshot::build(
        base.configuration().analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/Core.lua",
            "---@meta\n---@return string\nfunction External() end\n",
        )],
        LuaWorkspaceLimits::new(8, 4096, 16384, 65536)?,
    )?;
    assert_ne!(original_library.snapshot_id(), replacement.snapshot_id());
    operations::publish_input(
        input_with_library(files.clone(), original_library)?,
        &graph,
        &store_root,
        &LiveProjectPublishRequest::new("fixture:library-base", "absent", true, true)?,
        &stop,
    )?;
    let mut store = LiveProjectStore::open(&store_root)?;
    let current = store.current()?.ok_or("missing Library base")?;
    let keep = LiveProjectUpdateRequest::new(
        "fixture:library-keep",
        current.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    assert_eq!(
        store
            .update(
                input_with_library(files.clone(), replacement.clone())?,
                &graph,
                &keep,
                &stop
            )
            .err()
            .ok_or("Keep accepted a mismatched final Library")?
            .code(),
        ServiceErrorCode::InternalContractViolation
    );
    assert_eq!(store.current()?, Some(current.clone()));
    assert!(store.reconcile("fixture:library-keep")?.is_none());
    let replace = LiveProjectUpdateRequest::new(
        "fixture:library-replace",
        current.record_id.as_str(),
        LiveProjectLibraryMode::Replace,
        true,
    )?;
    let receipt = store.update(
        input_with_library(files.clone(), replacement.clone())?,
        &graph,
        &replace,
        &stop,
    )?;
    assert_eq!(receipt.exit_code(), 2);
    let replaced = store.current()?.ok_or("missing Library replacement")?;
    assert_ne!(replaced.generation_id, current.generation_id);
    let live = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        live.project()
            .snapshot()
            .analyzer_binding()
            .library_snapshot_ids()
            .collect::<Vec<_>>(),
        vec![replacement.snapshot_id()]
    );
    drop(live);
    let clear = LiveProjectUpdateRequest::new(
        "fixture:library-clear",
        replaced.record_id.as_str(),
        LiveProjectLibraryMode::Clear,
        true,
    )?;
    assert_eq!(
        store
            .update(
                input_with_library(files, replacement)?,
                &graph,
                &clear,
                &stop
            )
            .err()
            .ok_or("Clear accepted a Library-free analyzer")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert_eq!(store.current()?, Some(replaced));
    assert!(store.reconcile("fixture:library-clear")?.is_none());
    drop(store);
    std::fs::remove_dir_all(store_root)?;
    Ok(())
}
