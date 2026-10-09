use super::{VerifiedBackup, database};
use crate::project::{
    CurrentPublication, PartitionRecord, ProjectStore, PublicationRequest, ReadSelector,
    ReadSnapshot, RecordCatalog, ValidatedRead,
};
use crate::{OperationId, StoreErrorCode};
use std::{error::Error, fs, path::PathBuf, sync::atomic::AtomicBool};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn new_store(name: &str) -> TestResult<(PathBuf, ProjectStore, RecordCatalog)> {
    let root = std::env::temp_dir().join(format!(
        "wow-project-backup-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let store =
        ProjectStore::create_with_gc(root.join("source"), "fixture.backup", catalog.clone())?;
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

#[test]
fn committed_wal_backup_reopens_and_restores_only_after_owner_checks() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("wal-restore")?;
    let (first, first_current) = publish(&mut store, "backup-first", 11, &stop)?;
    let leased = store.read(&ReadSelector::Current, &stop)?;
    let (second, second_current) = publish(&mut store, "backup-second", 22, &stop)?;
    assert_eq!(leased.current_at_acquisition(), Some(&first_current));
    check_fixture(&leased, 11, &stop)?;
    let checkpoint = store.checkpoint(&stop)?;
    assert_eq!(checkpoint.active_readers, 1);
    assert!(checkpoint.frames > checkpoint.checkpointed);
    let mut wal_path = store.db.path.as_os_str().to_os_string();
    wal_path.push("-wal");
    assert!(fs::metadata(PathBuf::from(wal_path))?.len() > 0);

    let backup_root = root.join("backup");
    let backup_id = OperationId::new("fixture:backup-wal")?;
    let backup = store.backup_to_new(&backup_root, &backup_id, &stop)?;
    backup.verify(&stop)?;
    assert_eq!(backup.manifest().epoch(), store.epoch());
    assert_eq!(backup.manifest().current(), Some(&second_current));
    let mut generations = vec![
        first.generation().generation_id.clone(),
        second.generation().generation_id.clone(),
    ];
    generations.sort();
    assert_eq!(backup.manifest().generations(), generations);
    {
        let current = backup.read(&ReadSelector::Current, &stop)?;
        assert_eq!(current.manifest(), second.generation());
        check_fixture(&current, 22, &stop)?;
        let old = backup.read(
            &ReadSelector::Exact(first.generation().generation_id.clone()),
            &stop,
        )?;
        assert_eq!(old.manifest(), first.generation());
        check_fixture(&old, 11, &stop)?;
    }
    // Inspect the finished destination body through the owner's read-only
    // connection; its main image includes the source's uncheckpointed commit.
    let body_path =
        database::epoch_directory(&backup_root, backup.manifest().epoch())?.join("project.sqlite");
    let inspect = database::connect(&body_path, true)?;
    let count: i64 = inspect.query_row("SELECT count(*) FROM generations", [], |row| row.get(0))?;
    assert_eq!(count, 2);
    let record_id: String = inspect.query_row(
        "SELECT record_id FROM current_publication WHERE id=1",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(record_id, second_current.record_id.as_str());
    drop(inspect);
    let snapshot_digest = backup.manifest().snapshot_digest().to_owned();
    drop(backup);
    assert_eq!(
        ProjectStore::open(&backup_root, &catalog)
            .err()
            .ok_or("unvalidated backup opened as a normal store")?
            .code(),
        StoreErrorCode::DatabaseUnavailable
    );
    let backup = VerifiedBackup::open(&backup_root, &catalog, &backup_id, &snapshot_digest, &stop)?;

    let restore_root = root.join("restore");
    let restore_id = OperationId::new("fixture:backup-restore")?;
    let candidate = backup.restore_to_new(&restore_root, &restore_id, &stop)?;
    assert_eq!(candidate.manifest().snapshot_digest(), snapshot_digest);
    let restore_digest = candidate.manifest().snapshot_digest().to_owned();
    drop(candidate);
    assert_eq!(
        ProjectStore::open(&restore_root, &catalog)
            .err()
            .ok_or("inactive restore opened before owner validation")?
            .code(),
        StoreErrorCode::DatabaseUnavailable
    );
    let candidate =
        VerifiedBackup::open(&restore_root, &catalog, &restore_id, &restore_digest, &stop)?;
    assert_eq!(candidate.manifest().generations(), generations);
    let mut checks = Vec::new();
    for generation in candidate.manifest().generations() {
        let (request, value) = if generation == &first.generation().generation_id {
            (&first, 11)
        } else {
            assert_eq!(generation, &second.generation().generation_id);
            (&second, 22)
        };
        let read = candidate.read(&ReadSelector::Exact(generation.clone()), &stop)?;
        assert_eq!(read.manifest(), request.generation());
        checks.push(check_fixture(&read, value, &stop)?);
    }
    let restored = candidate.finish_restore(checks, &stop)?;
    assert_eq!(restored.epoch(), store.epoch());
    assert_eq!(restored.current()?, Some(second_current.clone()));
    assert_eq!(
        restored.operation(first.operation_id())?,
        store.operation(first.operation_id())?
    );
    assert_eq!(
        restored.operation(second.operation_id())?,
        store.operation(second.operation_id())?
    );
    drop(restored);
    let restored = ProjectStore::open(&restore_root, &catalog)?;
    assert_eq!(restored.current()?, Some(second_current.clone()));
    assert_eq!(store.current()?, Some(second_current));
    assert_eq!(leased.current_at_acquisition(), Some(&first_current));
    assert_eq!(leased.manifest(), first.generation());
    check_fixture(&leased, 11, &stop)?;
    drop(restored);
    drop(backup);
    drop(leased);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn truncated_body_and_substituted_final_manifest_reject_independent_open() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("tamper")?;
    let (_, current) = publish(&mut store, "backup-tamper-source", 33, &stop)?;
    let body_root = root.join("truncated");
    let body_id = OperationId::new("fixture:backup-truncated")?;
    let backup = store.backup_to_new(&body_root, &body_id, &stop)?;
    let body_path =
        database::epoch_directory(&body_root, backup.manifest().epoch())?.join("project.sqlite");
    let digest = backup.manifest().snapshot_digest().to_owned();
    drop(backup);
    fs::OpenOptions::new()
        .write(true)
        .open(body_path)?
        .set_len(32)?;
    let error = VerifiedBackup::open(&body_root, &catalog, &body_id, &digest, &stop)
        .err()
        .ok_or("truncated backup body accepted")?;
    assert_ne!(error.code(), StoreErrorCode::WriterBusy);

    let manifest_root = root.join("manifest-tamper");
    let manifest_id = OperationId::new("fixture:backup-manifest-tamper")?;
    let backup = store.backup_to_new(&manifest_root, &manifest_id, &stop)?;
    let digest = backup.manifest().snapshot_digest().to_owned();
    let original = String::from_utf8(backup.manifest().canonical_bytes()?)?;
    drop(backup);
    let changed = original.replace(
        "self-contained-inline-partitions",
        "self-contained-inline-partitionx",
    );
    assert_ne!(changed, original);
    fs::write(manifest_root.join("backup-manifest.json"), changed)?;
    assert_eq!(
        VerifiedBackup::open(&manifest_root, &catalog, &manifest_id, &digest, &stop)
            .err()
            .ok_or("substituted final manifest accepted")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    assert_eq!(store.current()?, Some(current));
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cancelled_backup_creates_nothing_and_existing_destination_is_preserved() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("refusal")?;
    let (_, current) = publish(&mut store, "backup-refusal-source", 44, &stop)?;
    let leased = store.read(&ReadSelector::Current, &stop)?;
    let cancelled_root = root.join("cancelled");
    assert_eq!(
        store
            .backup_to_new(
                &cancelled_root,
                &OperationId::new("fixture:backup-cancelled")?,
                &AtomicBool::new(true),
            )
            .err()
            .ok_or("cancelled backup accepted")?
            .code(),
        StoreErrorCode::Cancelled
    );
    assert!(!cancelled_root.exists());
    let existing_root = root.join("existing");
    fs::create_dir(&existing_root)?;
    let sentinel = existing_root.join("sentinel.txt");
    fs::write(&sentinel, b"preserve existing destination")?;
    assert_eq!(
        store
            .backup_to_new(
                &existing_root,
                &OperationId::new("fixture:backup-existing")?,
                &stop,
            )
            .err()
            .ok_or("existing destination overwritten")?
            .code(),
        StoreErrorCode::DatabaseUnavailable
    );
    assert_eq!(fs::read(&sentinel)?, b"preserve existing destination");
    assert_eq!(fs::read_dir(&existing_root)?.count(), 1);
    assert_eq!(store.current()?, Some(current.clone()));
    assert_eq!(leased.current_at_acquisition(), Some(&current));
    check_fixture(&leased, 44, &stop)?;
    drop(leased);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
