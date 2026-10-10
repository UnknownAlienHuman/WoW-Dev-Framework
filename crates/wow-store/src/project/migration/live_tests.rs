//! A source pin change must reject before creating any inactive target.
use super::*;

#[test]
fn guarded_live_migration_binds_pins_beside_unchanged_current_and_selector() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = fixture("guarded-pins", true)?;
    let publication = request(&store, "fixture:guarded-source", "fixture:guarded", 37)?;
    let current = activate(&mut store, &publication, 37, &stop)?;
    let first_pin = RetentionRoot::new(
        store.epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:first-pin")?,
        RetentionRootKind::Rollback,
        publication.generation().generation_id.clone(),
        "fixture.guard",
    )?;
    store.put_retention_root(&first_pin, &stop)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    let expected = store.registry_selection()?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:guarded-backup")?,
        &stop,
    )?;
    let candidate = store.stage_migration_to_new(
        &backup,
        &root.join("accepted"),
        &OperationId::new("fixture:guarded-migration")?,
        &expected,
        Some(&current.record_id),
        &stop,
    )?;
    assert_eq!(
        candidate.source().manifest().snapshot_digest(),
        backup.manifest().snapshot_digest()
    );
    assert!(candidate.store.current()?.is_none());
    assert!(!root.join("accepted/project-store-registry.json").exists());
    drop(candidate);

    let second_pin = RetentionRoot::new(
        store.epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:second-pin")?,
        RetentionRootKind::Evidence,
        publication.generation().generation_id.clone(),
        "fixture.guard",
    )?;
    store.put_retention_root(&second_pin, &stop)?;
    assert_eq!(store.registry_selection()?, expected);
    assert_eq!(store.current()?.as_ref(), Some(&current));
    let before = store.retention_roots(&stop)?;
    let refused_root = root.join("refused");
    rejected(
        store.stage_migration_to_new(
            &backup,
            &refused_root,
            &OperationId::new("fixture:stale-guard")?,
            &expected,
            Some(&current.record_id),
            &stop,
        ),
        StoreErrorCode::CurrentConflict,
    )?;
    assert!(!refused_root.exists());
    assert_eq!(store.retention_roots(&stop)?, before);
    assert_eq!(store.registry_selection()?, expected);
    assert_eq!(store.current()?.as_ref(), Some(&current));
    check(&held, 37, &stop)?;
    backup.verify(&stop)?;
    let cancelled_root = root.join("cancelled");
    rejected(
        store.stage_migration_to_new(
            &backup,
            &cancelled_root,
            &OperationId::new("fixture:cancel-guard")?,
            &expected,
            Some(&current.record_id),
            &AtomicBool::new(true),
        ),
        StoreErrorCode::Cancelled,
    )?;
    assert!(!cancelled_root.exists());
    drop(backup);
    drop(held);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
