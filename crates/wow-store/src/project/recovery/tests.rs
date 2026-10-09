use crate::project::{
    AcknowledgmentState, CurrentState, PartitionRecord, ProjectStore, PublicationOperation,
    PublicationRequest, PublicationState, ReadSelector, RecordCatalog, RecoveryDisposition,
    RecoveryScope, ScopeState,
};
use crate::{OperationId, StoreErrorCode};
use rusqlite::{TransactionBehavior, params};
use std::{error::Error, path::PathBuf, sync::atomic::AtomicBool};

use super::super::model::encode;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn new_store(name: &str) -> TestResult<(PathBuf, ProjectStore)> {
    let root = std::env::temp_dir().join(format!(
        "wow-project-recovery-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let store = ProjectStore::create_with_gc(&root, "fixture.recovery", catalog)?;
    Ok((root, store))
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

fn publish(
    store: &mut ProjectStore,
    name: &str,
    value: u32,
    stop: &AtomicBool,
) -> TestResult<(PublicationRequest, PublicationOperation)> {
    let request = request(store, name, value)?;
    store.prepare(&request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    assert_eq!(
        read.record("fixture.data", stop)?
            .ok_or("missing sealed data")?
            .decode::<Vec<u32>>()?,
        vec![value]
    );
    let validated = read.owner_validation(&["fixture.owner.v1"])?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validated,
        stop,
    )?;
    let operation = store.activate(request.operation_id(), request.request_digest(), stop)?;
    Ok((request, operation))
}

#[test]
fn empty_v3_reports_absent_current_and_valid_unselected_gc_policy() -> TestResult {
    let (root, store) = new_store("empty")?;
    assert!(store.gc_policy()?.is_none());
    let changes = store.db.connection.total_changes();
    let report = store.recovery_report(&AtomicBool::new(false))?;
    assert_eq!(report.current_state(), CurrentState::Absent);
    assert!(report.current().is_none());
    assert!(report.operations().is_empty());
    assert!(report.incidents().is_empty(), "{report:?}");
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::GcPolicy && coverage.state() == ScopeState::Validated
    }));
    assert_eq!(store.db.connection.total_changes(), changes);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn prepared_intent_without_target_retains_unknown_acknowledgment() -> TestResult {
    let (root, mut store) = new_store("prepared")?;
    let request = request(&store, "recovery-prepared", 1)?;
    let operation = PublicationOperation {
        operation_id: request.operation_id().clone(),
        request_digest: request.request_digest().into(),
        generation_id: request.generation().generation_id.clone(),
        state: PublicationState::Prepared,
        validation_id: None,
        activation: None,
        release: None,
    };
    // Execute the durable intent prefix used by prepare, stopping before any
    // partition seals or generation membership. No validation was performed.
    let tx = store
        .db
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "INSERT INTO operations(operation_id,request_digest,manifest,record) VALUES(?1,?2,?3,?4)",
        params![
            request.operation_id().as_str(),
            request.request_digest(),
            encode(request.generation(), 256 * 1024)?,
            encode(&operation, 65536)?
        ],
    )?;
    tx.commit()?;
    assert_eq!(
        store.operation(request.operation_id())?,
        Some(operation.clone())
    );
    let report = store.recovery_report(&AtomicBool::new(false))?;
    assert_eq!(report.current_state(), CurrentState::Absent);
    assert!(report.current().is_none());
    assert_eq!(report.operations().len(), 1);
    let observed = report
        .operations()
        .first()
        .ok_or("missing prepared observation")?;
    assert_eq!(observed.operation(), &operation);
    assert_eq!(observed.operation().state, PublicationState::Prepared);
    assert_eq!(observed.disposition(), RecoveryDisposition::Prepared);
    assert!(!observed.target_present());
    assert!(!observed.matches_current());
    assert_eq!(observed.acknowledgment(), AcknowledgmentState::Unknown);
    assert!(report.incidents().is_empty());
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn missing_membership_marks_real_activated_current_corrupt() -> TestResult {
    let (root, mut store) = new_store("missing-membership")?;
    let stop = AtomicBool::new(false);
    let (_, operation) = publish(&mut store, "recovery-membership", 1, &stop)?;
    let current = store.current()?.ok_or("missing activated current")?;
    let before = store.recovery_report(&stop)?;
    assert_eq!(before.current_state(), CurrentState::Validated);
    assert_eq!(before.current(), Some(&current));
    let observed = before
        .operations()
        .first()
        .ok_or("missing activated observation")?;
    assert_eq!(observed.operation(), &operation);
    assert_eq!(
        observed.disposition(),
        RecoveryDisposition::ActivatedReceiptAvailable
    );
    assert!(observed.matches_current());
    assert_eq!(observed.acknowledgment(), AcknowledgmentState::Unknown);
    assert_eq!(
        store.db.connection.execute(
            "DELETE FROM membership WHERE generation_id=?1 AND logical_key=?2",
            params![operation.generation_id.as_str(), "fixture.data"],
        )?,
        1
    );
    let changes = store.db.connection.total_changes();
    let report = store.recovery_report(&stop)?;
    assert_eq!(report.current_state(), CurrentState::Corrupt);
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::Membership && coverage.state() == ScopeState::Invalid
    }));
    assert!(report.incidents().iter().any(|incident| {
        incident.scope() == RecoveryScope::Membership
            && incident.code() == StoreErrorCode::IntegrityViolation
    }));
    assert_eq!(store.db.connection.total_changes(), changes);
    let retained_current: String = store.db.connection.query_row(
        "SELECT record_id FROM current_publication WHERE id=1",
        [],
        |row| row.get(0),
    )?;
    assert_eq!(retained_current, current.record_id.as_str());
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn same_length_partition_mutation_marks_real_activated_current_corrupt() -> TestResult {
    let (root, mut store) = new_store("partition-digest")?;
    let stop = AtomicBool::new(false);
    let (request, _) = publish(&mut store, "recovery-payload", 1, &stop)?;
    assert_eq!(
        store.recovery_report(&stop)?.current_state(),
        CurrentState::Validated
    );
    let member = request
        .generation()
        .members
        .first()
        .ok_or("missing member")?;
    let original: Vec<u8> = store.db.connection.query_row(
        "SELECT payload FROM partition_versions WHERE version=?1",
        [member.version.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(original.as_slice(), b"[1]");
    assert_eq!(original.len(), b"[2]".len());
    assert_eq!(
        store.db.connection.execute(
            "UPDATE partition_versions SET payload=?1 WHERE version=?2",
            params![b"[2]".as_slice(), member.version.as_str()],
        )?,
        1
    );
    let changes = store.db.connection.total_changes();
    let report = store.recovery_report(&stop)?;
    assert_eq!(report.current_state(), CurrentState::Corrupt);
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::Partitions && coverage.state() == ScopeState::Invalid
    }));
    assert!(report.incidents().iter().any(|incident| {
        incident.scope() == RecoveryScope::Partitions
            && incident.code() == StoreErrorCode::IntegrityViolation
    }));
    assert_eq!(store.db.connection.total_changes(), changes);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn recovery_refuses_pending_writer_state_and_honors_cancellation() -> TestResult {
    let (root, store) = new_store("pending-cancelled")?;
    store.db.connection.execute_batch("BEGIN IMMEDIATE")?;
    assert_eq!(
        store
            .recovery_report(&AtomicBool::new(false))
            .err()
            .ok_or("pending writer transaction accepted")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    store.db.connection.execute_batch("ROLLBACK")?;
    assert_eq!(
        store
            .recovery_report(&AtomicBool::new(true))
            .err()
            .ok_or("cancelled recovery scan accepted")?
            .code(),
        StoreErrorCode::Cancelled
    );
    assert!(store.current()?.is_none());
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn orphan_generation_length_mismatch_invalidates_membership_and_blocks_backup() -> TestResult {
    use crate::project::{RECORD_PROFILE, StoreGenerationId};

    let (root, mut store) = new_store("orphan-length")?;
    let stop = AtomicBool::new(false);
    let (current_request, _) = publish(&mut store, "recovery-current", 1, &stop)?;
    let current = store.current()?.ok_or("missing valid current")?;
    let orphan_request = request(&store, "recovery-orphan", 1)?;
    let mut orphan = orphan_request.generation().clone();
    assert_eq!(orphan.members.len(), 1);
    assert_eq!(orphan.members[0].byte_length, 3);
    assert_eq!(
        orphan.members[0].version,
        current_request.generation().members[0].version
    );
    orphan.members[0].byte_length = 4;
    // Keep the manifest canonical and identity-valid: only the declared member
    // length disagrees with the independently valid shared seal.
    let identity_bytes = encode(
        &(
            RECORD_PROFILE,
            &orphan.epoch_id,
            &orphan.owner,
            &orphan.bindings,
            &orphan.members,
            orphan.expected_current.iter().collect::<Vec<_>>(),
        ),
        256 * 1024,
    )?;
    orphan.generation_id = StoreGenerationId::derive(&identity_bytes);
    orphan.validate(store.epoch())?;
    assert_ne!(orphan.generation_id, current.generation_id);
    let tx = store
        .db
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    tx.execute(
        "INSERT INTO generations(generation_id,manifest) VALUES(?1,?2)",
        params![orphan.generation_id.as_str(), encode(&orphan, 256 * 1024)?],
    )?;
    let member = &orphan.members[0];
    tx.execute(
        "INSERT INTO membership(generation_id,logical_key,version) VALUES(?1,?2,?3)",
        params![
            orphan.generation_id.as_str(),
            &member.key,
            member.version.as_str()
        ],
    )?;
    tx.commit()?;
    assert_eq!(
        super::super::read::read_manifest(
            &store.db.connection,
            &orphan.generation_id,
            store.epoch(),
        )?,
        orphan
    );
    let operation_count: i64 =
        store
            .db
            .connection
            .query_row("SELECT count(*) FROM operations", [], |row| row.get(0))?;
    assert_eq!(operation_count, 1);
    assert!(store.operation(orphan_request.operation_id())?.is_none());

    let report = store.recovery_report(&stop)?;
    assert!(
        report
            .operations()
            .iter()
            .all(|observation| { observation.operation().generation_id != orphan.generation_id })
    );
    assert_eq!(report.current_state(), CurrentState::Validated);
    assert_eq!(report.current(), Some(&current));
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::Membership && coverage.state() == ScopeState::Invalid
    }));
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::Generations && coverage.state() == ScopeState::Validated
    }));
    assert!(report.coverage().iter().any(|coverage| {
        coverage.scope() == RecoveryScope::Partitions && coverage.state() == ScopeState::Validated
    }));
    assert!(report.incidents().iter().any(|incident| {
        incident.scope() == RecoveryScope::Membership
            && incident.subject_id() == Some(orphan.generation_id.as_str())
            && incident.code() == StoreErrorCode::IntegrityViolation
    }));
    let backup_root = root.join("refused-backup");
    assert_eq!(
        store
            .backup_to_new(
                &backup_root,
                &OperationId::new("fixture:orphan-backup")?,
                &stop,
            )
            .err()
            .ok_or("orphan member mismatch was accepted for backup")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    assert!(!backup_root.exists());
    assert_eq!(store.current()?, Some(current));
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn legacy_empty_profiles_recover_and_reopen_backups_without_absent_tables() -> TestResult {
    use crate::project::{PHYSICAL_PROFILE, RETAINED_PHYSICAL_PROFILE, VerifiedBackup};

    let root = std::env::temp_dir().join(format!(
        "wow-project-recovery-legacy-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&root)?;
    let stop = AtomicBool::new(false);
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    for (name, profile) in [("v1", PHYSICAL_PROFILE), ("v2", RETAINED_PHYSICAL_PROFILE)] {
        let store_root = root.join(name);
        let store = if profile == PHYSICAL_PROFILE {
            ProjectStore::create(&store_root, "fixture.recovery-legacy", catalog.clone())?
        } else {
            ProjectStore::create_with_retention(
                &store_root,
                "fixture.recovery-legacy",
                catalog.clone(),
            )?
        };
        assert_eq!(store.epoch().physical_profile(), profile);
        let report = store.recovery_report(&stop)?;
        assert_eq!(report.current_state(), CurrentState::Absent);
        assert!(report.current().is_none());
        assert!(report.operations().is_empty());
        assert!(
            report.incidents().is_empty(),
            "{name}: {:?}",
            report.incidents()
        );
        let retention_state = if profile == PHYSICAL_PROFILE {
            ScopeState::NotApplicable
        } else {
            ScopeState::Validated
        };
        assert!(report.coverage().iter().any(|coverage| {
            coverage.scope() == RecoveryScope::RetentionRoots && coverage.state() == retention_state
        }));
        for scope in [RecoveryScope::GcPolicy, RecoveryScope::GcReceipts] {
            assert!(report.coverage().iter().any(|coverage| {
                coverage.scope() == scope && coverage.state() == ScopeState::NotApplicable
            }));
        }

        let backup_root = root.join(format!("backup-{name}"));
        let backup_id = OperationId::new(format!("fixture:legacy-backup-{name}"))?;
        let backup = store.backup_to_new(&backup_root, &backup_id, &stop)?;
        assert_eq!(backup.manifest().epoch(), store.epoch());
        assert_eq!(backup.manifest().recovery(), &report);
        let manifest = backup.manifest().clone();
        drop(backup);
        let reopened = VerifiedBackup::open(
            &backup_root,
            &catalog,
            &backup_id,
            manifest.snapshot_digest(),
            &stop,
        )?;
        assert_eq!(reopened.manifest(), &manifest);
        assert_eq!(reopened.manifest().recovery(), &report);
        assert!(reopened.manifest().current().is_none());
        assert_eq!(store.current()?, None);
        drop(reopened);
        drop(store);
    }
    std::fs::remove_dir_all(root)?;
    Ok(())
}
