use super::super::{database, model::digest, registry};
use crate::project::{
    CurrentObservation, CurrentPublication, CurrentState, PartitionRecord, PointerReadFailure,
    ProjectGcPolicy, ProjectStore, PublicationRequest, QuarantineInspection, QuarantinedStore,
    ReadSelector, ReadSnapshot, RecordCatalog, ValidatedRead,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn at<T, E: std::fmt::Debug>(stage: &str, result: Result<T, E>) -> TestResult<T> {
    result.map_err(|error| format!("{stage}: {error:?}").into())
}

fn new_store(name: &str) -> TestResult<(PathBuf, ProjectStore, RecordCatalog)> {
    let root = std::env::temp_dir().join(format!(
        "wow-project-quarantine-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let store =
        ProjectStore::create_with_gc(root.join("source"), "fixture.quarantine", catalog.clone())?;
    Ok((root, store, catalog))
}

fn request(store: &ProjectStore, name: &str, value: u32) -> TestResult<PublicationRequest> {
    Ok(PublicationRequest::new(
        store.epoch(),
        OperationId::new(format!("fixture:{name}"))?,
        store.current()?.map(|current| current.record_id),
        [("fixture.owner".into(), format!("fixture:{name}"))].into(),
        vec![PartitionRecord::new(
            "fixture.data",
            "fixture.partition.v1",
            &vec![value],
        )?],
    )?)
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
    let request = request(store, name, value)?;
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

fn rejected<T>(result: StoreResult<T>, expected: StoreErrorCode) -> TestResult {
    assert_eq!(
        result
            .err()
            .ok_or("operation unexpectedly succeeded")?
            .code(),
        expected
    );
    Ok(())
}

#[test]
fn malformed_current_hold_blocks_normal_effects_and_preserves_healthy_leased_data() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at("create fixture store", new_store("malformed-current"))?;
    let source = root.join("source");
    let epoch = store.epoch().clone();
    let (first, current) = at(
        "publish healthy generation",
        publish(&mut store, "held-healthy", 11, &stop),
    )?;
    let held = at(
        "lease healthy current",
        store.read(&ReadSelector::Current, &stop),
    )?;
    let pending = at(
        "construct inactive request",
        request(&store, "held-inactive", 22),
    )?;
    at(
        "prepare inactive generation",
        store.prepare(&pending, &stop),
    )?;
    let read = at(
        "read inactive generation",
        store.read(
            &ReadSelector::Exact(pending.generation().generation_id.clone()),
            &stop,
        ),
    )?;
    let validation = at("validate inactive payload", check_fixture(&read, 22, &stop))?;
    drop(read);
    at(
        "record inactive validation",
        store.validate_inactive(
            pending.operation_id(),
            pending.request_digest(),
            validation,
            &stop,
        ),
    )?;
    let next = at(
        "construct blocked publication request",
        request(&store, "blocked-publication", 33),
    )?;
    let policy = at(
        "construct GC policy",
        ProjectGcPolicy::new("fixture:quarantine-policy", BTreeSet::new(), 8, 32, 4096),
    )?;
    at(
        "select GC policy",
        store.select_gc_policy(&policy, None, &stop),
    )?;
    let plan = at(
        "plan GC before pointer damage",
        store.plan_gc(&policy, &stop),
    )?;
    assert_eq!(
        at(
            "recover healthy store before damage",
            store.recovery_report(&stop)
        )?
        .current_state(),
        CurrentState::Validated
    );
    let selection = at("read normal selector", store.registry_selection())?;
    let selection_bytes = at(
        "read normal selector bytes",
        fs::read(source.join("project-store-registry.json")),
    )?;

    // Deliberately damage only the fixture's raw Current pointer. Restoring FK
    // enforcement does not repair the orphan or change the leased WAL snapshot.
    let malformed = b"malformed-current";
    at(
        "disable fixture FK enforcement",
        store.db.connection.execute_batch("PRAGMA foreign_keys=OFF"),
    )?;
    assert_eq!(
        at(
            "damage fixture current pointer",
            store.db.connection.execute(
                "UPDATE current_publication SET record_id=?1 WHERE id=1",
                [std::str::from_utf8(malformed)?],
            ),
        )?,
        1
    );
    at(
        "restore fixture FK enforcement",
        store.db.connection.execute_batch("PRAGMA foreign_keys=ON"),
    )?;
    let expected = CurrentObservation::Pointer {
        digest: digest("project-current-observation", malformed),
        byte_length: malformed.len(),
        record_id: None,
    };
    let inspection = at(
        "inspect malformed current",
        store.quarantine_inspection(&stop),
    )?;
    assert_eq!(inspection.selection(), &selection);
    assert_eq!(inspection.current(), &expected);
    at(
        "canonicalize malformed pointer observation before quarantine",
        wow_core::canonical_json_bytes(inspection.current()),
    )?;
    let evidence_digest = inspection.evidence_digest();
    let operation = OperationId::new("fixture:hold-malformed")?;
    let receipt = at(
        "hold malformed current",
        store.quarantine(&operation, &inspection, &stop),
    )?;
    assert_eq!(receipt.previous(), &selection);
    assert_eq!(receipt.operation_id(), &operation);
    assert_eq!(receipt.current(), &expected);
    assert_eq!(receipt.evidence_digest(), evidence_digest);
    assert!(receipt.selected().is_quarantined());
    assert_eq!(receipt.selected().revision(), selection.revision() + 1);
    assert_eq!(receipt.selected().epoch(), epoch.epoch_id());
    assert_eq!(store.quarantine(&operation, &inspection, &stop)?, receipt);

    let archive = source
        .join("quarantines")
        .join(registry::instance_id(&operation)?);
    assert_eq!(fs::read(archive.join("selection.json"))?, selection_bytes);
    assert_eq!(
        fs::read(archive.join("record.json"))?,
        fs::read(source.join("project-store-registry.json"))?
    );
    assert_eq!(
        digest(
            "project-quarantine-evidence",
            &fs::read(archive.join("evidence.json"))?
        ),
        evidence_digest
    );
    rejected(store.current(), StoreErrorCode::Quarantined)?;
    rejected(
        store.read(&ReadSelector::Current, &stop),
        StoreErrorCode::Quarantined,
    )?;
    rejected(
        store.read(
            &ReadSelector::Exact(first.generation().generation_id.clone()),
            &stop,
        ),
        StoreErrorCode::Quarantined,
    )?;
    rejected(store.prepare(&next, &stop), StoreErrorCode::Quarantined)?;
    rejected(
        store.activate(pending.operation_id(), pending.request_digest(), &stop),
        StoreErrorCode::Quarantined,
    )?;
    rejected(store.plan_gc(&policy, &stop), StoreErrorCode::Quarantined)?;
    rejected(
        store.execute_gc(&plan, &OperationId::new("fixture:blocked-gc")?, &stop),
        StoreErrorCode::Quarantined,
    )?;
    let refused_backup = root.join("refused-backup");
    rejected(
        store.backup_to_new(
            &refused_backup,
            &OperationId::new("fixture:blocked-backup")?,
            &stop,
        ),
        StoreErrorCode::Quarantined,
    )?;
    assert!(!refused_backup.exists());
    let quarantined = store.quarantined(&stop)?;
    assert_eq!(quarantined.receipt(), &receipt);
    assert_eq!(quarantined.current_observation(&stop)?, expected);
    let report = quarantined.recovery_report(&stop)?;
    assert_eq!(report.current_state(), CurrentState::Corrupt);
    assert!(!report.incidents().is_empty());
    assert_eq!(held.manifest(), first.generation());
    assert_eq!(held.current_at_acquisition(), Some(&current));
    check_fixture(&held, 11, &stop)?;
    drop(quarantined);
    drop(inspection);
    drop(store);
    check_fixture(&held, 11, &stop)?;
    rejected(
        QuarantinedStore::open(&source, &catalog, &stop),
        StoreErrorCode::WriterBusy,
    )?;
    drop(held);
    rejected(
        ProjectStore::open(&source, &catalog),
        StoreErrorCode::Quarantined,
    )?;
    let reopened = QuarantinedStore::open(&source, &catalog, &stop)?;
    assert_eq!(reopened.receipt(), &receipt);
    assert_eq!(reopened.epoch(), &epoch);
    assert_eq!(reopened.current_observation(&stop)?, expected);
    assert_eq!(
        reopened.recovery_report(&stop)?.current_state(),
        CurrentState::Corrupt
    );
    drop(reopened);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn stale_history_evidence_cancellation_and_operation_conflicts_do_not_change_the_selector()
-> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("stale-evidence")?;
    let source = root.join("source");
    let (first, current) = publish(&mut store, "stale-first", 11, &stop)?;
    assert_eq!(
        store.db.connection.execute(
            "UPDATE publication_history SET record=?1 WHERE record_id=?2",
            rusqlite::params![b"{}".as_slice(), current.record_id.as_str()],
        )?,
        1
    );
    assert_eq!(
        store.recovery_report(&stop)?.current_state(),
        CurrentState::Corrupt
    );
    let stale = store.quarantine_inspection(&stop)?;
    assert!(matches!(
        stale.current(), CurrentObservation::Pointer { record_id: Some(id), .. }
            if id == &current.record_id
    ));
    let selection_bytes = fs::read(source.join("project-store-registry.json"))?;
    // The pointer is unchanged, but an additional real payload defect changes
    // recovery evidence and must invalidate the earlier inspection.
    assert_eq!(
        store.db.connection.execute(
            "UPDATE partition_versions SET payload=?1 WHERE version=?2",
            rusqlite::params![
                b"[12]".as_slice(),
                first.generation().members[0].version.as_str()
            ],
        )?,
        1
    );
    let operation = OperationId::new("fixture:hold-stale")?;
    rejected(
        store.quarantine(&operation, &stale, &stop),
        StoreErrorCode::CurrentConflict,
    )?;
    assert_eq!(
        fs::read(source.join("project-store-registry.json"))?,
        selection_bytes
    );
    assert!(!source.join("quarantines").exists());
    let fresh = store.quarantine_inspection(&stop)?;
    assert_eq!(fresh.current(), stale.current());
    assert_ne!(fresh.evidence_digest(), stale.evidence_digest());
    stop.store(true, Ordering::Release);
    rejected(
        store.quarantine(&operation, &fresh, &stop),
        StoreErrorCode::Cancelled,
    )?;
    rejected(
        store.quarantine_inspection(&stop),
        StoreErrorCode::Cancelled,
    )?;
    assert_eq!(
        fs::read(source.join("project-store-registry.json"))?,
        selection_bytes
    );
    assert!(!source.join("quarantines").exists());
    stop.store(false, Ordering::Release);
    let receipt = store.quarantine(&operation, &fresh, &stop)?;
    let selected_bytes = fs::read(source.join("project-store-registry.json"))?;
    assert_eq!(store.quarantine(&operation, &fresh, &stop)?, receipt);
    rejected(
        store.quarantine(&operation, &stale, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    rejected(
        store.quarantine(&OperationId::new("fixture:another-hold")?, &fresh, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    assert_eq!(
        fs::read(source.join("project-store-registry.json"))?,
        selected_bytes
    );
    drop(stale);
    drop(fresh);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn altered_archived_selection_rejects_quarantined_and_normal_reopen() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("archive-mutation")?;
    let source = root.join("source");
    let (_, current) = publish(&mut store, "archive-first", 11, &stop)?;
    assert_eq!(
        store.db.connection.execute(
            "UPDATE publication_history SET record=?1 WHERE record_id=?2",
            rusqlite::params![b"{}".as_slice(), current.record_id.as_str()],
        )?,
        1
    );
    let inspection = store.quarantine_inspection(&stop)?;
    let operation = OperationId::new("fixture:hold-archive")?;
    let receipt = store.quarantine(&operation, &inspection, &stop)?;
    drop(inspection);
    drop(store);
    let reopened = QuarantinedStore::open(&source, &catalog, &stop)?;
    assert_eq!(reopened.receipt(), &receipt);
    assert_eq!(
        reopened.recovery_report(&stop)?.current_state(),
        CurrentState::Corrupt
    );
    drop(reopened);
    let selected = fs::read(source.join("project-store-registry.json"))?;
    let archive = source
        .join("quarantines")
        .join(registry::instance_id(&operation)?);
    let path = archive.join("selection.json");
    let mut changed = fs::read(&path)?;
    changed.push(b'\n');
    fs::write(&path, &changed)?;
    rejected(
        QuarantinedStore::open(&source, &catalog, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    rejected(
        ProjectStore::open(&source, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(
        fs::read(source.join("project-store-registry.json"))?,
        selected
    );
    assert_eq!(fs::read(path)?, changed);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn corrupt_header_allows_only_readonly_inspection_and_an_explicit_durable_hold() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("corrupt-header")?;
    let source = root.join("source");
    publish(&mut store, "header-first", 11, &stop)?;
    let epoch = store.epoch().clone();
    let selection = store.registry_selection()?;
    let selection_bytes = fs::read(source.join("project-store-registry.json"))?;
    let path = store.db.path.clone();
    drop(store);
    let connection = database::connect(&path, false)?;
    connection.pragma_update(None, "application_id", 0i64)?;
    drop(connection);
    rejected(
        ProjectStore::open(&source, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;
    let inspection = QuarantineInspection::open(&source, &catalog, &stop)?;
    let expected = CurrentObservation::Unreadable {
        reason: PointerReadFailure::QueryUnavailable,
    };
    assert_eq!(inspection.selection(), &selection);
    assert_eq!(inspection.current(), &expected);
    let operation = OperationId::new("fixture:hold-header")?;
    let receipt = inspection.quarantine(&operation, &stop)?;
    assert_eq!(receipt.previous(), &selection);
    assert_eq!(receipt.current(), &expected);
    assert_eq!(receipt.evidence_digest(), inspection.evidence_digest());
    let archive = source
        .join("quarantines")
        .join(registry::instance_id(&operation)?);
    assert_eq!(fs::read(archive.join("selection.json"))?, selection_bytes);
    assert_eq!(
        fs::read(archive.join("record.json"))?,
        fs::read(source.join("project-store-registry.json"))?
    );
    let evidence_bytes = fs::read(archive.join("evidence.json"))?;
    assert_eq!(
        digest("project-quarantine-evidence", &evidence_bytes),
        receipt.evidence_digest()
    );
    let evidence: serde_json::Value = serde_json::from_slice(&evidence_bytes)?;
    assert_eq!(evidence["state"], "unavailable");
    assert_eq!(
        evidence["code"],
        serde_json::to_value(StoreErrorCode::IntegrityViolation)?
    );
    assert!(evidence.get("report").is_none());
    drop(inspection);
    rejected(
        ProjectStore::open(&source, &catalog),
        StoreErrorCode::Quarantined,
    )?;
    let reopened = QuarantinedStore::open(&source, &catalog, &stop)?;
    assert_eq!(reopened.epoch(), &epoch);
    assert_eq!(reopened.receipt(), &receipt);
    assert_eq!(reopened.current_observation(&stop)?, expected);
    rejected(
        reopened.recovery_report(&stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    let connection = database::connect(&path, true)?;
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    assert_eq!(application_id, 0);
    drop(connection);
    drop(reopened);
    fs::remove_dir_all(root)?;
    Ok(())
}
