use super::super::{database, model::MAX_READERS};
use crate::project::{
    CurrentPublication, PHYSICAL_PROFILE, PartitionRecord, ProjectStore, PublicationRequest,
    ReadSelector, ReadSnapshot, RecordCatalog, RegistrySelection, ValidatedRead, VerifiedBackup,
};
use crate::{OperationId, StoreErrorCode};
use std::{
    error::Error,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn new_store(name: &str) -> TestResult<(PathBuf, ProjectStore, RecordCatalog)> {
    let root = std::env::temp_dir().join(format!(
        "wow-project-replacement-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let store = ProjectStore::create(root.join("source"), "fixture.replacement", catalog.clone())?;
    Ok((root, store, catalog))
}

fn check_fixture(
    read: &ReadSnapshot,
    expected: u32,
    stop: &AtomicBool,
) -> TestResult<ValidatedRead> {
    assert_eq!(read.manifest().members.len(), 1);
    assert_eq!(
        read.record("fixture.data", stop)?
            .ok_or("missing sealed fixture data")?
            .decode::<Vec<u32>>()?,
        vec![expected]
    );
    Ok(read.owner_validation(&["fixture.owner.v1"])?)
}

fn publish(
    store: &mut ProjectStore,
    name: &str,
    value: u32,
    stop: &AtomicBool,
) -> TestResult<(PublicationRequest, CurrentPublication)> {
    let request = PublicationRequest::new(
        store.epoch(),
        OperationId::new(format!("fixture:{name}"))?,
        store.current()?.map(|current| current.record_id),
        [("fixture.owner".into(), format!("fixture:{name}"))].into(),
        vec![PartitionRecord::new(
            "fixture.data",
            "fixture.partition.v1",
            &vec![value],
        )?],
    )?;
    store.prepare(&request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    assert_eq!(read.manifest(), request.generation());
    let validation = check_fixture(&read, value, stop)?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validation,
        stop,
    )?;
    let activated = store.activate(request.operation_id(), request.request_digest(), stop)?;
    let current = store.current()?.ok_or("missing activated current")?;
    assert_eq!(activated.activation.as_ref(), Some(&current));
    Ok((request, current))
}

fn owners(
    backup: &VerifiedBackup,
    request: &PublicationRequest,
    value: u32,
    stop: &AtomicBool,
) -> TestResult<Vec<ValidatedRead>> {
    assert_eq!(
        backup.manifest().generations(),
        std::slice::from_ref(&request.generation().generation_id)
    );
    let read = backup.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    assert_eq!(read.manifest(), request.generation());
    Ok(vec![check_fixture(&read, value, stop)?])
}

fn unchanged(
    store: &ProjectStore,
    selection: &RegistrySelection,
    current: &CurrentPublication,
) -> TestResult {
    assert_eq!(&store.registry_selection()?, selection);
    assert_eq!(store.current()?.as_ref(), Some(current));
    Ok(())
}

#[test]
fn replacement_preserves_semantic_ids_readers_budget_and_exact_retry() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, store, catalog) = new_store("readers")?;
    let source = root.join("source");
    let epoch = store.epoch().clone();
    let legacy_selection = store.registry_selection()?;
    assert_eq!(legacy_selection.revision(), 0);
    assert_eq!(epoch.physical_profile(), PHYSICAL_PROFILE);
    drop(store);
    let mut store = ProjectStore::open(&source, &catalog)?;
    assert_eq!(store.epoch(), &epoch);
    assert_eq!(store.registry_selection()?, legacy_selection);

    let (first, first_current) = publish(&mut store, "replacement-first", 11, &stop)?;
    let first_operation = store.operation(first.operation_id())?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:replacement-backup")?,
        &stop,
    )?;
    let first_reader = store.read(&ReadSelector::Current, &stop)?;
    let (second, second_current) = publish(&mut store, "replacement-second", 22, &stop)?;
    let old_reader = store.read(&ReadSelector::Current, &stop)?;
    let old_path = store.db.path.clone();
    let operation = OperationId::new("fixture:replacement-select")?;
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &legacy_selection,
        Some(second_current.record_id.clone()),
        &stop,
    )?;
    assert_eq!(candidate.backup().manifest().epoch(), &epoch);
    assert_eq!(
        candidate.backup().manifest().current(),
        Some(&first_current)
    );
    let request_digest = candidate.request_digest()?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;

    // Retain the actual old physical connection and lock to emulate a caller
    // losing the result after the selector was successfully replaced.
    let stale_db = database::Database {
        connection: database::connect(&store.db.path, false)?,
        path: store.db.path.clone(),
        epoch: store.db.epoch.clone(),
        root: store.db.root.clone(),
        selection: store.db.selection.clone(),
        life: Rc::clone(&store.db.life),
    };
    let receipt = store.activate_replacement(candidate, checks, &stop)?;
    let selected = store.registry_selection()?;
    assert_eq!(receipt.previous(), &legacy_selection);
    assert_eq!(receipt.selected(), &selected);
    assert_eq!(receipt.operation_id(), &operation);
    assert_eq!(receipt.request_digest(), request_digest);
    assert_eq!(receipt.activated_current(), Some(&first_current));
    assert_eq!(selected.revision(), 1);
    assert_eq!(selected.epoch(), epoch.epoch_id());
    assert_ne!(selected.digest(), legacy_selection.digest());
    assert_ne!(store.db.path, old_path);
    assert!(old_path.exists());
    let registry_bytes = fs::read(source.join("project-store-registry.json"))?;
    let selected_db = std::mem::replace(&mut store.db, stale_db);
    drop(selected_db);
    assert_eq!(
        store
            .current()
            .err()
            .ok_or("stale source handle remained writable")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    let candidate = store.reopen_replacement(
        &operation,
        &legacy_selection,
        Some(second_current.record_id.clone()),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    assert_eq!(candidate.request_digest()?, request_digest);
    let staged = source.join(format!(
        "project-store-registry-{}.staged",
        super::super::registry::instance_id(&operation)?
    ));
    assert!(!staged.exists());
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    assert_eq!(
        store.activate_replacement(candidate, checks, &stop)?,
        receipt
    );
    assert!(!staged.exists());
    assert_eq!(
        fs::read(source.join("project-store-registry.json"))?,
        registry_bytes
    );
    unchanged(&store, &selected, &first_current)?;
    assert_eq!(store.epoch(), &epoch);
    assert_eq!(store.operation(first.operation_id())?, first_operation);
    assert!(store.operation(second.operation_id())?.is_none());
    assert_eq!(
        store.replacement_receipt(&operation, &request_digest)?,
        Some(receipt.clone())
    );
    assert_eq!(first_reader.manifest(), first.generation());
    assert_eq!(first_reader.current_at_acquisition(), Some(&first_current));
    check_fixture(&first_reader, 11, &stop)?;
    assert_eq!(old_reader.manifest(), second.generation());
    assert_eq!(old_reader.current_at_acquisition(), Some(&second_current));
    check_fixture(&old_reader, 22, &stop)?;
    let new_reader = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(new_reader.manifest(), first.generation());
    check_fixture(&new_reader, 11, &stop)?;

    // Two retained old-instance readers plus selected-instance readers share
    // the same finite root admission limit, despite separate lease maps.
    assert_eq!(MAX_READERS, 16);
    let mut readers = Vec::new();
    for _ in 3..MAX_READERS {
        readers.push(store.read(&ReadSelector::Current, &stop)?);
    }
    assert_eq!(
        store
            .read(&ReadSelector::Current, &stop)
            .err()
            .ok_or("replacement reset the root reader budget")?
            .code(),
        StoreErrorCode::BudgetExceeded
    );
    drop(first_reader);
    let released_slot = store.read(&ReadSelector::Current, &stop)?;
    check_fixture(&released_slot, 11, &stop)?;
    drop(released_slot);
    drop(readers);
    drop(store);
    assert_eq!(
        ProjectStore::open(&source, &catalog)
            .err()
            .ok_or("old and new readers lost the root lock")?
            .code(),
        StoreErrorCode::WriterBusy
    );
    drop(old_reader);
    assert_eq!(
        ProjectStore::open(&source, &catalog)
            .err()
            .ok_or("selected reader lost the root lock")?
            .code(),
        StoreErrorCode::WriterBusy
    );
    drop(new_reader);
    let store = ProjectStore::open(&source, &catalog)?;
    unchanged(&store, &selected, &first_current)?;
    assert_eq!(store.epoch(), &epoch);
    assert_eq!(
        store.replacement_receipt(&operation, &request_digest)?,
        Some(receipt)
    );
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.manifest(), first.generation());
    check_fixture(&read, 11, &stop)?;
    drop(read);
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn stale_current_selector_and_reused_operation_preserve_the_live_base() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("guards")?;
    let (first, first_current) = publish(&mut store, "guards-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:guards-backup")?,
        &stop,
    )?;
    let selection = store.registry_selection()?;
    let stale_operation = OperationId::new("fixture:stale-replacement")?;
    let candidate = store.stage_replacement(
        &backup,
        &stale_operation,
        &selection,
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let stale_digest = candidate.request_digest()?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let (_, second_current) = publish(&mut store, "guards-second", 22, &stop)?;
    assert_eq!(
        store
            .activate_replacement(candidate, checks, &stop)
            .err()
            .ok_or("staged stale current was accepted")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    unchanged(&store, &selection, &second_current)?;
    assert_eq!(
        store
            .stage_replacement(
                &backup,
                &OperationId::new("fixture:stale-current")?,
                &selection,
                Some(first_current.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("stale current was copied")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    assert_eq!(
        store
            .reopen_replacement(
                &stale_operation,
                &selection,
                Some(second_current.record_id.clone()),
                backup.manifest().snapshot_digest(),
                &stop,
            )
            .err()
            .ok_or("staged operation accepted a substituted base")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    let operation = OperationId::new("fixture:guarded-replacement")?;
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &selection,
        Some(second_current.record_id.clone()),
        &stop,
    )?;
    assert_eq!(
        store
            .stage_replacement(
                &backup,
                &operation,
                &selection,
                Some(second_current.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("existing operation directory was overwritten")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let receipt = store.activate_replacement(candidate, checks, &stop)?;
    let selected = store.registry_selection()?;
    assert_eq!(
        store
            .stage_replacement(
                &backup,
                &OperationId::new("fixture:stale-selector")?,
                &selection,
                Some(first_current.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("stale outer selector was accepted")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    assert_eq!(
        store
            .replacement_receipt(&operation, &stale_digest)
            .err()
            .ok_or("receipt accepted another request digest")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    unchanged(&store, &selected, &first_current)?;
    assert_eq!(
        store.replacement_receipt(&operation, receipt.request_digest())?,
        Some(receipt)
    );
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn omitted_owner_validation_cannot_select_a_verified_physical_candidate() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("owner")?;
    let (_, current) = publish(&mut store, "owner-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:owner-backup")?,
        &stop,
    )?;
    let selection = store.registry_selection()?;
    let operation = OperationId::new("fixture:owner-replacement")?;
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &selection,
        Some(current.record_id.clone()),
        &stop,
    )?;
    candidate.backup().verify(&stop)?;
    let digest = candidate.request_digest()?;
    assert_eq!(
        store
            .activate_replacement(candidate, Vec::new(), &stop)
            .err()
            .ok_or("physical verification substituted for native owner checks")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    unchanged(&store, &selection, &current)?;
    assert!(store.replacement_receipt(&operation, &digest)?.is_none());
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn candidate_payload_mutation_rejects_previously_obtained_owner_checks() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("corruption")?;
    let (first, current) = publish(&mut store, "corruption-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:corruption-backup")?,
        &stop,
    )?;
    let selection = store.registry_selection()?;
    let operation = OperationId::new("fixture:corrupt-replacement")?;
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &selection,
        Some(current.record_id.clone()),
        &stop,
    )?;
    let digest = candidate.request_digest()?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let connection = database::connect(&candidate.backup().store.db.path, false)?;
    let version = &first.generation().members[0].version;
    let original: Vec<u8> = connection.query_row(
        "SELECT payload FROM partition_versions WHERE version=?1",
        [version.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(original.as_slice(), b"[11]");
    assert_eq!(original.len(), b"[12]".len());
    assert_eq!(
        connection.execute(
            "UPDATE partition_versions SET payload=?1 WHERE version=?2",
            rusqlite::params![b"[12]".as_slice(), version.as_str()],
        )?,
        1
    );
    let busy: i64 =
        connection.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))?;
    assert_eq!(busy, 0);
    drop(connection);
    assert_eq!(
        store
            .activate_replacement(candidate, checks, &stop)
            .err()
            .ok_or("mutated candidate accepted earlier owner checks")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    unchanged(&store, &selection, &current)?;
    assert!(store.replacement_receipt(&operation, &digest)?.is_none());
    let read = store.read(&ReadSelector::Current, &stop)?;
    check_fixture(&read, 11, &stop)?;
    drop(read);
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cancelled_activation_reopens_only_the_exact_staged_intent() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("cancelled")?;
    let (first, current) = publish(&mut store, "cancelled-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:cancelled-backup")?,
        &stop,
    )?;
    let selection = store.registry_selection()?;
    let operation = OperationId::new("fixture:cancelled-replacement")?;
    stop.store(true, Ordering::Release);
    assert_eq!(
        store
            .stage_replacement(
                &backup,
                &operation,
                &selection,
                Some(current.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("cancelled staging created a candidate")?
            .code(),
        StoreErrorCode::Cancelled
    );
    assert!(!root.join("source/instances").exists());
    stop.store(false, Ordering::Release);
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &selection,
        Some(current.record_id.clone()),
        &stop,
    )?;
    let digest = candidate.request_digest()?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    stop.store(true, Ordering::Release);
    assert_eq!(
        store
            .activate_replacement(candidate, checks, &stop)
            .err()
            .ok_or("cancelled activation selected the candidate")?
            .code(),
        StoreErrorCode::Cancelled
    );
    unchanged(&store, &selection, &current)?;
    assert!(store.replacement_receipt(&operation, &digest)?.is_none());
    drop(store);
    stop.store(false, Ordering::Release);
    let mut store = ProjectStore::open(root.join("source"), &catalog)?;
    unchanged(&store, &selection, &current)?;
    let candidate = store.reopen_replacement(
        &operation,
        &selection,
        Some(current.record_id.clone()),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    assert_eq!(candidate.request_digest()?, digest);
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let receipt = store.activate_replacement(candidate, checks, &stop)?;
    let selected = store.registry_selection()?;
    assert_eq!(selected.revision(), 1);
    unchanged(&store, &selected, &current)?;
    drop(store);
    let store = ProjectStore::open(root.join("source"), &catalog)?;
    unchanged(&store, &selected, &current)?;
    assert_eq!(
        store.replacement_receipt(&operation, &digest)?,
        Some(receipt)
    );
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.manifest(), first.generation());
    check_fixture(&read, 11, &stop)?;
    drop(read);
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn selected_instance_accepts_normal_publication_and_an_exact_second_replacement() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("second-instance")?;
    let (first, first_current) = publish(&mut store, "physical-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:physical-backup")?,
        &stop,
    )?;
    let epoch = store.epoch().clone();
    let selection = store.registry_selection()?;
    let candidate = store.stage_replacement(
        &backup,
        &OperationId::new("fixture:physical-one")?,
        &selection,
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let receipt_one = store.activate_replacement(candidate, checks, &stop)?;
    let selected_one = store.registry_selection()?;
    let path_one = store.db.path.clone();
    let (_, next_current) = publish(&mut store, "physical-next", 22, &stop)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(store.registry_selection()?, selected_one);
    assert_eq!(
        store.replacement_receipt(receipt_one.operation_id(), receipt_one.request_digest())?,
        Some(receipt_one.clone())
    );
    // Activation-time current is historical, and normal publication does not
    // rewrite the outer selector or erase that operation's original receipt.
    assert_eq!(receipt_one.activated_current(), Some(&first_current));
    let candidate = store.stage_replacement(
        &backup,
        &OperationId::new("fixture:physical-two")?,
        &selected_one,
        Some(next_current.record_id.clone()),
        &stop,
    )?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let receipt_two = store.activate_replacement(candidate, checks, &stop)?;
    let selected_two = store.registry_selection()?;
    assert_eq!(selected_two.revision(), 2);
    assert_eq!(receipt_two.previous(), &selected_one);
    assert_eq!(store.epoch(), &epoch);
    assert_ne!(store.db.path, path_one);
    assert!(path_one.exists());
    check_fixture(&held, 22, &stop)?;
    assert_eq!(held.current_at_acquisition(), Some(&next_current));
    assert!(
        store
            .replacement_receipt(receipt_one.operation_id(), receipt_one.request_digest())?
            .is_none()
    );
    unchanged(&store, &selected_two, &first_current)?;
    drop(store);
    assert_eq!(
        ProjectStore::open(root.join("source"), &catalog)
            .err()
            .ok_or("old selected instance lost root lock")?
            .code(),
        StoreErrorCode::WriterBusy
    );
    drop(held);
    let store = ProjectStore::open(root.join("source"), &catalog)?;
    unchanged(&store, &selected_two, &first_current)?;
    assert_eq!(
        store.replacement_receipt(receipt_two.operation_id(), receipt_two.request_digest())?,
        Some(receipt_two)
    );
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[cfg(windows)]
#[test]
fn selector_sharing_failure_preserves_current_and_requires_explicit_reconciliation() -> TestResult {
    use std::os::windows::fs::OpenOptionsExt;
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("selector-sharing")?;
    let (first, current) = publish(&mut store, "sharing-first", 11, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:sharing-backup")?,
        &stop,
    )?;
    let selection = store.registry_selection()?;
    let operation = OperationId::new("fixture:sharing-replacement")?;
    let candidate = store.stage_replacement(
        &backup,
        &operation,
        &selection,
        Some(current.record_id.clone()),
        &stop,
    )?;
    let request = candidate.request_digest()?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    // Permit the exact source guard to read, while the OS denies selector
    // deletion/replacement. No reader is force-closed and no retry spins.
    let sharing = fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(root.join("source/project-store-registry.json"))?;
    assert_eq!(
        store
            .activate_replacement(candidate, checks, &stop)
            .err()
            .ok_or("Windows sharing failed to block replacement")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    unchanged(&store, &selection, &current)?;
    assert!(store.replacement_receipt(&operation, &request)?.is_none());
    drop(sharing);
    let candidate = store.reopen_replacement(
        &operation,
        &selection,
        Some(current.record_id.clone()),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    let checks = owners(candidate.backup(), &first, 11, &stop)?;
    let receipt = store.activate_replacement(candidate, checks, &stop)?;
    assert_eq!(receipt.request_digest(), request);
    assert_eq!(receipt.selected().revision(), 1);
    assert_eq!(store.current()?, Some(current));
    drop(store);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}
