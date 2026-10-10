//! Native cross-epoch installation and exact adoption preserve both reader domains.
use super::*;
use crate::project::{AcknowledgmentState, MigrationSelectionCandidate, database};
use std::rc::Rc;

fn checks(
    candidate: &MigrationSelectionCandidate,
    stop: &AtomicBool,
) -> TestResult<Vec<ValidatedRead>> {
    candidate
        .target_generations()
        .iter()
        .map(|generation| check(&candidate.read_generation(generation, stop)?, 71, stop))
        .collect()
}

#[test]
fn native_selection_preserves_old_readers_reconciles_both_owner_states_and_reopens() -> TestResult {
    let stop = AtomicBool::new(false);
    let Fixture {
        root,
        mut store,
        catalog,
        baseline,
        ready,
    } = fixture_with_hold("selection-lifecycle", &stop)?;
    let source_root = root.join("source-restored");
    let old_epoch = store.epoch().clone();
    let old_path = store.db.path.clone();
    let source_state_before = source_state(&store, &stop)?;
    let root_manifest_before = fs::read(source_root.join("epoch-manifest.json"))?;
    let expected = store.registry_selection()?;
    let old_current = store.current()?.ok_or("missing source Current")?;
    let old_reader = store.read(&ReadSelector::Current, &stop)?;
    let original_life = Rc::clone(&store.db.life);
    // Retain the real pre-dispatch source connection/cache to exercise adoption
    // by an old owner after a committed result is no longer held in memory.
    let mut old_owner = ProjectStore {
        db: database::Database {
            connection: database::connect(&old_path, true)?,
            path: old_path.clone(),
            epoch: old_epoch.clone(),
            root: store.db.root.clone(),
            selection: Some(expected.clone()),
            life: Rc::clone(&original_life),
        },
    };
    let operation = OperationId::new("fixture:selection")?;
    let candidate = at(
        "stage selection",
        store.stage_ready_selection(
            &baseline,
            &ready,
            &operation,
            &expected,
            Some(&old_current.record_id),
            &stop,
        ),
    )?;
    let request_digest = candidate.request_digest()?;
    let selector_before = fs::read(source_root.join(registry::REGISTRY_FILE))?;
    rejected(
        store.activate_ready_selection(&baseline, &ready, candidate, Vec::new(), &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(store.current()?, Some(old_current.clone()));
    assert_eq!(
        fs::read(source_root.join(registry::REGISTRY_FILE))?,
        selector_before
    );
    let candidate =
        store.reopen_ready_selection(&baseline, &ready, &operation, &request_digest, &stop)?;
    let owners = checks(&candidate, &stop)?;
    let staged_reader = candidate.read_generation(&candidate.target_generations()[0], &stop)?;
    let receipt = at(
        "initial selection",
        store.activate_ready_selection(&baseline, &ready, candidate, owners, &stop),
    )?;
    assert_eq!(receipt.previous(), &expected);
    assert_eq!(receipt.source_epoch(), &old_epoch);
    assert_eq!(receipt.target_epoch(), ready.artifact().manifest().epoch());
    assert_eq!(receipt.selected().revision(), expected.revision() + 1);
    assert_eq!(receipt.acknowledgment(), AcknowledgmentState::Unknown);
    assert_eq!(store.current()?.as_ref(), ready.receipt().target_current());
    assert_eq!(store.retained_source_authorities()?.len(), 1);
    assert!(store.retained_quarantines()?.is_empty());
    assert_ne!(store.db.path, old_path);
    check(&staged_reader, 71, &stop)?;
    assert_eq!(
        store
            .db
            .life
            .leases
            .borrow()
            .get(&staged_reader.manifest().generation_id),
        Some(&1)
    );
    drop(staged_reader);
    assert!(old_path.exists());
    assert_eq!(
        fs::read(source_root.join("epoch-manifest.json"))?,
        root_manifest_before
    );
    assert_eq!(old_reader.manifest().epoch_id, *old_epoch.epoch_id());
    assert_eq!(old_reader.current_at_acquisition(), Some(&old_current));
    check(&old_reader, 71, &stop)?;
    let selected = fs::read(source_root.join(registry::REGISTRY_FILE))?;
    let record: serde_json::Value = serde_json::from_slice(&selected)?;
    assert_eq!(record["schema"], "wow-store/project-registry/6");
    let inventory = sql_inventory(&store.db.connection)?;
    let target_reader = store.read(&ReadSelector::Current, &stop)?;
    let selected_life = Rc::clone(&store.db.life);
    assert!(Rc::ptr_eq(
        &selected_life.reader_admissions,
        &original_life.reader_admissions
    ));
    let candidate = at(
        "adopted owner reopen",
        store.reopen_ready_selection(&baseline, &ready, &operation, &request_digest, &stop),
    )?;
    let owners = checks(&candidate, &stop)?;
    assert_eq!(
        at(
            "adopted owner retry",
            store.activate_ready_selection(&baseline, &ready, candidate, owners, &stop)
        )?,
        receipt
    );
    assert!(Rc::ptr_eq(&store.db.life, &selected_life));
    assert_eq!(sql_inventory(&store.db.connection)?, inventory);
    assert_eq!(
        fs::read(source_root.join(registry::REGISTRY_FILE))?,
        selected
    );
    check(&target_reader, 71, &stop)?;
    drop(target_reader);
    drop(selected_life);
    drop(store);
    // The obsolete source owner branches on the committed selector before its
    // stale-source guard and acquires only the target instance lock.
    let candidate = at(
        "obsolete source owner reopen",
        old_owner.reopen_ready_selection(&baseline, &ready, &operation, &request_digest, &stop),
    )?;
    let owners = checks(&candidate, &stop)?;
    assert_eq!(
        at(
            "obsolete source owner adoption",
            old_owner.activate_ready_selection(&baseline, &ready, candidate, owners, &stop)
        )?,
        receipt
    );
    assert_eq!(sql_inventory(&old_owner.db.connection)?, inventory);
    assert_eq!(
        fs::read(source_root.join(registry::REGISTRY_FILE))?,
        selected
    );
    check(&old_reader, 71, &stop)?;
    let copy = old_owner.backup_to_new(
        root.join("selected-backup"),
        &OperationId::new("fixture:selection-backup")?,
        &stop,
    )?;
    assert_eq!(
        copy.manifest().snapshot_digest(),
        receipt.portable_snapshot_digest()
    );
    let owner_checks = backup_checks(&copy, &stop)?;
    let isolated = copy.finish_restore(owner_checks, &stop)?;
    assert_eq!(isolated.current()?.as_ref(), receipt.activated_current());
    drop(isolated);
    let next = request(
        &old_owner,
        "fixture:selection-later",
        "fixture:selection-later",
        72,
    )?;
    activate(&mut old_owner, &next, 72, &stop)?;
    assert_eq!(
        old_owner.migration_selection_receipt(&operation, &request_digest)?,
        Some(receipt.clone())
    );
    rejected(
        old_owner.reopen_ready_selection(&baseline, &ready, &operation, &request_digest, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    let source_connection = database::connect(&old_path, true)?;
    let original_state =
        crate::project::backup::identity::capture(&source_connection, &old_epoch, &stop)?;
    assert_eq!(
        crate::project::model::encode(&original_state, 16 * 1024 * 1024)?,
        source_state_before
    );
    drop(source_connection);
    drop(old_reader);
    drop(original_life);
    drop(old_owner);
    let reopened = ProjectStore::open(&source_root, &catalog)?;
    assert_eq!(reopened.epoch(), receipt.target_epoch());
    assert_eq!(
        reopened.migration_selection_receipt(&operation, &request_digest)?,
        Some(receipt)
    );
    let read = reopened.read(&ReadSelector::Current, &stop)?;
    check(&read, 72, &stop)?;
    drop(read);
    drop(reopened);
    drop(ready);
    drop(baseline);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn stale_source_guards_refuse_selection_before_ledger_or_instance_creation() -> TestResult {
    let stop = AtomicBool::new(false);
    let Fixture {
        root,
        mut store,
        baseline,
        ready,
        ..
    } = fixture_with_hold("selection-stale", &stop)?;
    let source_root = root.join("source-restored");
    let expected = store.registry_selection()?;
    let current = store.current()?.ok_or("missing source Current")?;
    let operation = OperationId::new("fixture:selection-stale")?;
    rejected(
        store.stage_ready_selection(&baseline, &ready, &operation, &expected, None, &stop),
        StoreErrorCode::CurrentConflict,
    )?;
    assert!(!source_root.join("migration-selections").exists());
    assert!(!source_root.join("instances").exists());
    let instances = source_root.join("instances");
    let foreign = instances.join(registry::instance_id(&operation)?);
    fs::create_dir(&instances)?;
    fs::create_dir(&foreign)?;
    rejected(
        store.stage_ready_selection(
            &baseline,
            &ready,
            &operation,
            &expected,
            Some(&current.record_id),
            &stop,
        ),
        StoreErrorCode::OperationConflict,
    )?;
    assert!(!source_root.join("migration-selections").exists());
    assert!(foreign.is_dir());
    fs::remove_dir(foreign)?;
    fs::remove_dir(instances)?;
    store.put_retention_root(
        &RetentionRoot::new(
            store.epoch().epoch_id().clone(),
            RetentionRootId::new("fixture:selection-late-pin")?,
            RetentionRootKind::Evidence,
            current.generation_id.clone(),
            "fixture:selection-policy",
        )?,
        &stop,
    )?;
    let before = source_state(&store, &stop)?;
    let selector = fs::read(source_root.join(registry::REGISTRY_FILE))?;
    rejected(
        store.stage_ready_selection(
            &baseline,
            &ready,
            &operation,
            &expected,
            Some(&current.record_id),
            &stop,
        ),
        StoreErrorCode::CurrentConflict,
    )?;
    assert!(!source_root.join("migration-selections").exists());
    assert!(!source_root.join("instances").exists());
    assert_eq!(source_state(&store, &stop)?, before);
    assert_eq!(
        fs::read(source_root.join(registry::REGISTRY_FILE))?,
        selector
    );
    drop(ready);
    drop(baseline);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
