//! Native domain read-back of every retained generation in a verified backup.
use super::super::{LiveProjectStore, restore_live_project_to_new};
use super::{owners, root};
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_store::project::ReadSelector;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn backup_and_restore_replay_every_retained_generation_and_preserve_live_readers() -> TestResult {
    let stop = AtomicBool::new(false);
    let (first, first_graph) = owners("local x = 1\nreturn x\n")?;
    let (second, second_graph) = owners("local x = 2\nreturn x\n")?;
    let source_root = root("backup-source")?;
    let backup_root = root("backup-artifact")?;
    let restore_root = root("backup-restore")?;
    let mut store =
        LiveProjectStore::create(&source_root, first_graph.snapshot().universe().as_str())?;
    let one = store.publish(&first, &first_graph, "fixture:backup-first", None, &stop)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    let old_id = held.project().snapshot_id().to_owned();
    let first_current = store.current()?.ok_or("missing first current")?;
    store.publish(
        &second,
        &second_graph,
        "fixture:backup-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let active = store.current()?.ok_or("missing current before backup")?;
    let current_id = store
        .read(&ReadSelector::Current, &stop)?
        .project()
        .snapshot_id()
        .to_owned();
    let backup = store.backup_to_new(&backup_root, "fixture:backup-write", &stop)?;
    assert_eq!(store.current()?, Some(active.clone()));
    assert_eq!(backup.manifest().generations().len(), 2);
    for id in backup.manifest().generations() {
        let read = backup.read(&ReadSelector::Exact(id.clone()), &stop)?;
        let pair = AcquiredProjectPair::read(&read, &stop)?;
        let (expected_id, graph) = if id == &one.generation_id {
            (&old_id, &first_graph)
        } else {
            assert_eq!(id, &active.generation_id);
            (&current_id, &second_graph)
        };
        assert_eq!(pair.project().snapshot_id(), expected_id);
        assert_eq!(pair.graph(), graph);
    }
    let restored =
        restore_live_project_to_new(&backup, &restore_root, "fixture:restore-write", &stop)?;
    assert_eq!(restored.storage_epoch(), store.storage_epoch());
    assert_eq!(restored.current()?, Some(active.clone()));
    for id in backup.manifest().generations() {
        let read = restored.read(&ReadSelector::Exact(id.clone()), &stop)?;
        let (expected_id, graph) = if id == &one.generation_id {
            (&old_id, &first_graph)
        } else {
            (&current_id, &second_graph)
        };
        assert_eq!(read.project().snapshot_id(), expected_id);
        assert_eq!(read.graph(), graph);
    }
    assert_eq!(held.project().snapshot_id(), old_id);
    assert_eq!(held.graph(), &first_graph);
    assert_eq!(held.current_at_acquisition(), Some(&first_current));
    assert_eq!(store.current()?, Some(active.clone()));
    drop(held);
    drop(restored);
    drop(backup);
    drop(store);
    let reopened = LiveProjectStore::open(&source_root)?;
    assert_eq!(reopened.current()?, Some(active));
    drop(reopened);
    std::fs::remove_dir_all(source_root)?;
    std::fs::remove_dir_all(backup_root)?;
    std::fs::remove_dir_all(restore_root)?;
    Ok(())
}
