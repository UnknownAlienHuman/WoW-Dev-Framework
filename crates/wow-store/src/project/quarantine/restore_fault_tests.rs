//! Real selector sharing refusal and exact adoption, without repeating selection.
use super::tests::*;
use crate::{OperationId, StoreErrorCode, project::ReadSelector};
use std::{
    fs::{self, OpenOptions},
    os::windows::fs::OpenOptionsExt,
    sync::atomic::AtomicBool,
};

#[test]
fn sharing_refusal_preserves_hold_and_exact_selected_restore_is_adopted_once() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("restore-sharing")?;
    let (_, current) = publish(&mut store, "restore-sharing-base", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:restore-sharing-backup")?,
        &stop,
    )?;
    let inspection = store.quarantine_inspection(&stop)?;
    let hold = store.quarantine(
        &OperationId::new("fixture:restore-sharing-hold")?,
        &inspection,
        &stop,
    )?;
    let held = store.quarantined(&stop)?;
    drop(inspection);
    drop(store);
    let operation = OperationId::new("fixture:restore-sharing-select")?;
    let candidate = held.stage_restore(&backup, &operation, hold.selected(), &stop)?;
    let read = candidate.backup().read(&ReadSelector::Current, &stop)?;
    let checks = vec![check_fixture(&read, 11, &stop)?];
    drop(read);
    let selector = root.join("source").join("project-store-registry.json");
    let original = fs::read(&selector)?;
    let blocker = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&selector)?;
    rejected(
        held.activate_restore(candidate, checks, &stop),
        StoreErrorCode::OutcomeUnknown,
    )?;
    assert_eq!(fs::read(&selector)?, original);
    drop(blocker);
    let candidate = held.reopen_restore(
        &operation,
        hold.selected(),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    let read = candidate.backup().read(&ReadSelector::Current, &stop)?;
    let checks = vec![check_fixture(&read, 11, &stop)?];
    drop(read);
    let (restored, receipt) = held.activate_restore(candidate, checks, &stop)?;
    assert_eq!(restored.current()?, Some(current.clone()));
    let selected = fs::read(&selector)?;
    let staged = root.join("source").join(format!(
        "project-store-registry-{}.staged",
        super::super::registry::instance_id(&operation)?
    ));
    assert!(!staged.exists());
    drop(restored);
    let candidate = held.reopen_restore(
        &operation,
        hold.selected(),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    let read = candidate.backup().read(&ReadSelector::Current, &stop)?;
    let checks = vec![check_fixture(&read, 11, &stop)?];
    drop(read);
    let (adopted, adopted_receipt) = held.activate_restore(candidate, checks, &stop)?;
    assert_eq!(
        serde_json::to_value(adopted_receipt)?,
        serde_json::to_value(receipt)?
    );
    assert_eq!(adopted.current()?, Some(current));
    assert_eq!(fs::read(selector)?, selected);
    assert!(
        !staged.exists(),
        "adoption must not recreate or dispatch a selector"
    );
    drop(adopted);
    drop(held);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}
