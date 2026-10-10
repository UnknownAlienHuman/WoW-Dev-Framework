//! Native inactive-epoch migration preserves opaque owner records and source history.
use super::{MigrationCandidate, MigrationMapping};
use crate::project::{
    CurrentPublication, GC_PHYSICAL_PROFILE, PHYSICAL_PROFILE, PartitionRecord, ProjectStore,
    PublicationRequest, PublicationState, RETAINED_PHYSICAL_PROFILE, ReadSelector, ReadSnapshot,
    RecordCatalog, RetentionRoot, RetentionRootId, RetentionRootKind, StoreGenerationId,
    ValidatedRead,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    path::PathBuf,
    sync::atomic::AtomicBool,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
type SqlInventory = BTreeMap<&'static str, Vec<Vec<rusqlite::types::Value>>>;

fn at<T, E: std::fmt::Debug>(stage: &str, result: Result<T, E>) -> TestResult<T> {
    result.map_err(|error| format!("{stage}: {error:?}").into())
}

fn fixture(name: &str, retained: bool) -> TestResult<(PathBuf, ProjectStore, RecordCatalog)> {
    let root = std::env::temp_dir().join(format!(
        "wow-project-migration-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let source = root.join("source");
    let store = if retained {
        ProjectStore::create_with_retention(&source, "fixture.migration", catalog.clone())?
    } else {
        ProjectStore::create(&source, "fixture.migration", catalog.clone())?
    };
    Ok((root, store, catalog))
}

fn check(read: &ReadSnapshot, value: u32, stop: &AtomicBool) -> TestResult<ValidatedRead> {
    assert_eq!(read.manifest().members.len(), 1);
    let record = read
        .record("fixture.data", stop)?
        .ok_or("missing migration fixture partition")?;
    assert_eq!(record.decode::<Vec<u32>>()?, vec![value]);
    assert_eq!(record.version(), &read.manifest().members[0].version);
    Ok(read.owner_validation(&["fixture.owner.v1"])?)
}

fn activate(
    store: &mut ProjectStore,
    request: &PublicationRequest,
    value: u32,
    stop: &AtomicBool,
) -> TestResult<CurrentPublication> {
    store.prepare(request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    assert_eq!(read.manifest(), request.generation());
    let validation = check(&read, value, stop)?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validation,
        stop,
    )?;
    let operation = store.activate(request.operation_id(), request.request_digest(), stop)?;
    let current = store
        .current()?
        .ok_or("missing migration fixture current")?;
    assert_eq!(operation.activation.as_ref(), Some(&current));
    Ok(current)
}

fn request(
    store: &ProjectStore,
    operation: &str,
    binding: &str,
    value: u32,
) -> TestResult<PublicationRequest> {
    Ok(PublicationRequest::new(
        store.epoch(),
        OperationId::new(operation)?,
        store.current()?.map(|current| current.record_id),
        [("fixture.owner".into(), binding.into())].into(),
        vec![PartitionRecord::new(
            "fixture.data",
            "fixture.partition.v1",
            &vec![value],
        )?],
    )?)
}

fn rejected<T>(result: StoreResult<T>, code: StoreErrorCode) -> TestResult {
    assert_eq!(
        result
            .err()
            .ok_or("migration unexpectedly succeeded")?
            .code(),
        code
    );
    Ok(())
}

fn target_checks(
    candidate: &MigrationCandidate,
    expected: &[(&PublicationRequest, u32)],
    stop: &AtomicBool,
) -> TestResult<Vec<ValidatedRead>> {
    let sources: BTreeSet<_> = expected
        .iter()
        .map(|(request, _)| request.generation().generation_id.clone())
        .collect();
    assert_eq!(candidate.mappings().len(), sources.len());
    assert_eq!(
        candidate
            .mappings()
            .iter()
            .map(|mapping| mapping.source_generation().clone())
            .collect::<BTreeSet<_>>(),
        sources
    );
    let mut targets: BTreeMap<StoreGenerationId, u32> = BTreeMap::new();
    for (request, value) in expected {
        let mapping = candidate
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == &request.generation().generation_id)
            .ok_or("missing source generation mapping")?;
        assert!(!mapping.operation_id().as_str().is_empty());
        assert!(!mapping.request_digest().is_empty());
        let operation = candidate
            .store
            .operation(mapping.operation_id())?
            .ok_or("missing mapped inactive operation")?;
        assert_eq!(operation.generation_id, *mapping.target_generation());
        assert_eq!(operation.request_digest, mapping.request_digest());
        assert!(operation.activation.is_none());
        let read = candidate.read_generation(mapping.target_generation(), stop)?;
        assert_eq!(
            read.manifest().epoch_id,
            *candidate.target_epoch().epoch_id()
        );
        assert_eq!(read.manifest().bindings, request.generation().bindings);
        assert_eq!(read.manifest().members, request.generation().members);
        assert!(read.manifest().expected_current.is_none());
        assert!(read.current_at_acquisition().is_none());
        check(&read, *value, stop)?;
        if let Some(previous) = targets.insert(mapping.target_generation().clone(), *value) {
            assert_eq!(previous, *value, "aliased target changed its owner payload");
        }
    }
    assert_eq!(
        candidate
            .target_generations()
            .into_iter()
            .collect::<BTreeSet<_>>(),
        targets.keys().cloned().collect::<BTreeSet<_>>()
    );
    let mut checks = Vec::new();
    for (generation, value) in targets {
        let read = candidate.read_generation(&generation, stop)?;
        checks.push(check(&read, value, stop)?);
    }
    Ok(checks)
}

fn sql_inventory(connection: &rusqlite::Connection) -> TestResult<SqlInventory> {
    let mut result = BTreeMap::new();
    for (table, sql) in [
        (
            "epoch_metadata",
            "SELECT id,manifest FROM epoch_metadata ORDER BY id LIMIT 17",
        ),
        (
            "partition_versions",
            "SELECT version,logical_key,schema_id,byte_length,payload FROM partition_versions ORDER BY version LIMIT 17",
        ),
        (
            "generations",
            "SELECT generation_id,manifest FROM generations ORDER BY generation_id LIMIT 17",
        ),
        (
            "membership",
            "SELECT generation_id,logical_key,version FROM membership ORDER BY generation_id,logical_key LIMIT 17",
        ),
        (
            "operations",
            "SELECT operation_id,request_digest,manifest,record FROM operations ORDER BY operation_id LIMIT 17",
        ),
        (
            "validations",
            "SELECT validation_id,generation_id,record FROM validations ORDER BY validation_id LIMIT 17",
        ),
        (
            "publication_history",
            "SELECT record_id,generation_id,validation_id,record FROM publication_history ORDER BY record_id LIMIT 17",
        ),
        (
            "current_publication",
            "SELECT id,record_id FROM current_publication ORDER BY id LIMIT 17",
        ),
        (
            "retention_roots",
            "SELECT root_id,generation_id,record FROM retention_roots ORDER BY root_id LIMIT 17",
        ),
        (
            "gc_policy",
            "SELECT id,policy_digest,record FROM gc_policy ORDER BY id LIMIT 17",
        ),
        (
            "gc_operations",
            "SELECT operation_id,request_digest,record FROM gc_operations ORDER BY operation_id LIMIT 17",
        ),
    ] {
        let mut statement = connection.prepare(sql)?;
        let columns = statement.column_count();
        let rows = statement
            .query_map([], |row| {
                (0..columns)
                    .map(|column| row.get::<_, rusqlite::types::Value>(column))
                    .collect::<rusqlite::Result<Vec<_>>>()
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        assert!(
            rows.len() < 17,
            "fixture inventory was truncated for {table}"
        );
        result.insert(table, rows);
    }
    Ok(result)
}

fn remove_declared_generation(
    candidate: &mut MigrationCandidate,
    mapping: &MigrationMapping,
) -> TestResult {
    let transaction = candidate
        .store
        .db
        .connection
        .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    assert_eq!(
        transaction.execute(
            "DELETE FROM operations WHERE operation_id=?1",
            [mapping.operation_id().as_str()],
        )?,
        1
    );
    assert_eq!(
        transaction.execute(
            "DELETE FROM membership WHERE generation_id=?1",
            [mapping.target_generation().as_str()],
        )?,
        1
    );
    assert_eq!(
        transaction.execute(
            "DELETE FROM generations WHERE generation_id=?1",
            [mapping.target_generation().as_str()],
        )?,
        1
    );
    transaction.commit()?;
    Ok(())
}

#[test]
fn v1_migration_aliases_predecessor_only_generations_without_rewriting_source_history() -> TestResult
{
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at("create v1 source", fixture("v1-alias", false))?;
    assert_eq!(store.epoch().physical_profile(), PHYSICAL_PROFILE);
    let first = request(&store, "fixture:migration-first", "fixture:shared", 11)?;
    let first_current = activate(&mut store, &first, 11, &stop)?;
    let second = request(&store, "fixture:migration-second", "fixture:shared", 11)?;
    let second_current = activate(&mut store, &second, 11, &stop)?;
    assert_eq!(first.generation().members, second.generation().members);
    assert_eq!(first.generation().bindings, second.generation().bindings);
    assert_ne!(
        first.generation().generation_id,
        second.generation().generation_id
    );
    assert_eq!(
        second_current.predecessor.as_ref(),
        Some(&first_current.record_id)
    );
    let third = request(&store, "fixture:migration-third", "fixture:distinct", 22)?;
    let third_current = activate(&mut store, &third, 22, &stop)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    let backup = at(
        "capture v1 history",
        store.backup_to_new(
            root.join("backup"),
            &OperationId::new("fixture:migration-backup")?,
            &stop,
        ),
    )?;
    let source_digest = backup.manifest().snapshot_digest().to_owned();
    let migration_root = root.join("migration");
    let operation = OperationId::new("fixture:migration-v1")?;
    let candidate = at(
        "stage v1 migration",
        MigrationCandidate::stage(&backup, &migration_root, &operation, &stop),
    )?;
    assert_eq!(
        candidate.target_epoch().physical_profile(),
        GC_PHYSICAL_PROFILE
    );
    assert_ne!(
        candidate.target_epoch().epoch_id(),
        store.epoch().epoch_id()
    );
    assert_eq!(
        candidate.source().manifest().snapshot_digest(),
        source_digest
    );
    assert_eq!(
        candidate.source().manifest().current(),
        Some(&third_current)
    );
    assert!(candidate.store.current()?.is_none());
    assert_eq!(candidate.target_generations().len(), 2);
    let expected = [(&first, 11), (&second, 11), (&third, 22)];
    let checks = target_checks(&candidate, &expected, &stop)?;
    let mapped = |id: &StoreGenerationId| {
        candidate
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == id)
            .map(|mapping| mapping.target_generation().clone())
    };
    assert_eq!(
        mapped(&first.generation().generation_id),
        mapped(&second.generation().generation_id)
    );
    assert_ne!(
        mapped(&first.generation().generation_id),
        mapped(&third.generation().generation_id)
    );
    let mapped_current =
        mapped(&third.generation().generation_id).ok_or("missing mapped original Current")?;
    let first_mapping = candidate
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == &first.generation().generation_id)
        .ok_or("missing first alias")?;
    let second_mapping = candidate
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == &second.generation().generation_id)
        .ok_or("missing second alias")?;
    assert_eq!(first_mapping.operation_id(), second_mapping.operation_id());
    assert_eq!(
        first_mapping.request_digest(),
        second_mapping.request_digest()
    );
    for (current, original, value) in [
        (&first_current, &first, 11),
        (&second_current, &second, 11),
        (&third_current, &third, 22),
    ] {
        let read = candidate
            .source()
            .read(&ReadSelector::Publication(current.record_id.clone()), &stop)?;
        assert_eq!(read.manifest(), original.generation());
        check(&read, value, &stop)?;
    }
    let validated = at(
        "finish v1 owner validation",
        candidate.finish(checks, &stop),
    )?;
    assert_eq!(validated.receipt().source_snapshot_digest(), source_digest);
    assert!(!validated.receipt().target_snapshot_digest().is_empty());
    assert_eq!(validated.receipt().source_epoch(), store.epoch());
    assert_eq!(validated.receipt().target_epoch(), validated.target_epoch());
    assert_eq!(validated.receipt().mappings(), validated.mappings());
    assert_eq!(validated.target_generations().len(), 2);
    assert_eq!(validated.receipt().validations().len(), 2);
    assert_eq!(
        validated.receipt().state(),
        PublicationState::ValidatedInactive
    );
    assert!(validated.candidate.store.current()?.is_none());
    let current_mapping = validated
        .receipt()
        .current_mapping()
        .ok_or("missing inactive Current mapping")?;
    assert_eq!(current_mapping.source(), &third_current);
    assert_eq!(current_mapping.target_generation(), &mapped_current);
    assert_eq!(
        validated.receipt().validations().get(&mapped_current),
        Some(current_mapping.target_validation())
    );
    for id in validated.target_generations() {
        assert!(
            validated
                .read_generation(&id, &stop)?
                .current_at_acquisition()
                .is_none()
        );
    }
    assert!(!migration_root.join("project-store-registry.json").exists());
    assert!(ProjectStore::open(&migration_root, &catalog).is_err());
    validated.source().verify(&stop)?;
    backup.verify(&stop)?;
    assert_eq!(store.current()?.as_ref(), Some(&third_current));
    check(&held, 22, &stop)?;
    drop(held);
    drop(validated);
    drop(backup);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn v2_migration_reopens_exact_intent_and_requires_real_owner_checks() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at("create v2 source", fixture("v2-reopen", true))?;
    assert_eq!(store.epoch().physical_profile(), RETAINED_PHYSICAL_PROFILE);
    let first = request(&store, "fixture:migration-v2-first", "fixture:first", 31)?;
    activate(&mut store, &first, 31, &stop)?;
    let second = request(&store, "fixture:migration-v2-second", "fixture:second", 32)?;
    let original_current = activate(&mut store, &second, 32, &stop)?;
    let pin = RetentionRoot::new(
        store.epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:migration-rollback")?,
        RetentionRootKind::Rollback,
        first.generation().generation_id.clone(),
        "fixture:policy",
    )?;
    assert_eq!(store.put_retention_root(&pin, &stop)?, pin);
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:migration-v2-backup")?,
        &stop,
    )?;
    let snapshot = backup.manifest().snapshot_digest().to_owned();
    let operation = OperationId::new("fixture:migration-v2")?;
    let migration_root = root.join("migration");
    let candidate = at(
        "stage v2 migration",
        MigrationCandidate::stage(&backup, &migration_root, &operation, &stop),
    )?;
    let target_epoch = candidate.target_epoch().clone();
    assert_eq!(target_epoch.physical_profile(), GC_PHYSICAL_PROFILE);
    assert_eq!(candidate.target_generations().len(), 2);
    assert!(migration_root.join("source-archive").is_dir());
    assert_eq!(
        candidate.source().store.retention_roots(&stop)?,
        vec![pin.clone()]
    );
    assert!(candidate.store.retention_roots(&stop)?.is_empty());
    drop(candidate);
    let forbidden_registry = migration_root.join("project-store-registry.json");
    fs::write(&forbidden_registry, b"{}")?;
    rejected(
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
        StoreErrorCode::CurrentConflict,
    )?;
    assert_eq!(fs::read(&forbidden_registry)?, b"{}");
    fs::remove_file(&forbidden_registry)?;
    rejected(
        MigrationCandidate::open(
            &migration_root,
            &catalog,
            &OperationId::new("fixture:substituted-migration")?,
            &snapshot,
            &stop,
        ),
        StoreErrorCode::OperationConflict,
    )?;
    let cancelled = AtomicBool::new(true);
    rejected(
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &cancelled),
        StoreErrorCode::Cancelled,
    )?;
    let reopened = at(
        "reopen exact v2 intent",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    assert_eq!(reopened.target_epoch(), &target_epoch);
    rejected(
        reopened.read_generation(&first.generation().generation_id, &stop),
        StoreErrorCode::GenerationMissing,
    )?;
    rejected(
        reopened.finish(Vec::new(), &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    let reopened = at(
        "reopen after omitted owner checks",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    let mut source_checks = Vec::new();
    for (request, value) in [(&first, 31), (&second, 32)] {
        let read = reopened.source().read(
            &ReadSelector::Exact(request.generation().generation_id.clone()),
            &stop,
        )?;
        source_checks.push(check(&read, value, &stop)?);
    }
    rejected(
        reopened.finish(source_checks, &stop),
        StoreErrorCode::GenerationMissing,
    )?;
    let reopened = at(
        "reopen after substituted source-epoch capabilities",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    let checks = target_checks(&reopened, &[(&first, 31), (&second, 32)], &stop)?;
    rejected(
        reopened.finish(checks, &cancelled),
        StoreErrorCode::Cancelled,
    )?;
    let reopened = at(
        "reopen after cancelled owner completion",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    let checks = target_checks(&reopened, &[(&first, 31), (&second, 32)], &stop)?;
    let validated = at(
        "finish reopened v2 migration",
        reopened.finish(checks, &stop),
    )?;
    let target_snapshot = validated.receipt().target_snapshot_digest().to_owned();
    assert!(!target_snapshot.is_empty());
    assert_eq!(validated.receipt().source_snapshot_digest(), snapshot);
    for id in validated.target_generations() {
        let read = validated.read_generation(&id, &stop)?;
        assert!(read.current_at_acquisition().is_none());
    }
    assert!(!migration_root.join("project-store-registry.json").exists());
    drop(validated);
    let reopened = at(
        "reopen completed inactive migration",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    let checks = target_checks(&reopened, &[(&first, 31), (&second, 32)], &stop)?;
    let validated = reopened.finish(checks, &stop)?;
    assert_eq!(
        validated.receipt().target_snapshot_digest(),
        target_snapshot
    );
    validated.source().verify(&stop)?;
    backup.verify(&stop)?;
    assert_eq!(store.current()?.as_ref(), Some(&original_current));
    assert_eq!(
        validated.source().store.retention_roots(&stop)?,
        vec![pin.clone()]
    );
    assert_eq!(store.retention_roots(&stop)?, vec![pin]);
    let expected_receipt = validated.receipt().clone();
    let target_path = validated.candidate.store.db.path.clone();
    let before = sql_inventory(&validated.candidate.store.db.connection)?;
    drop(validated);
    let record_path = migration_root.join("migration-record.json");
    let record_bytes = fs::read(&record_path)?;
    fs::write(&record_path, b"{}")?;
    rejected(
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    let connection = super::super::database::connect(&target_path, true)?;
    connection.execute_batch("BEGIN DEFERRED")?;
    assert_eq!(
        sql_inventory(&connection)?,
        before,
        "receipt rejection altered the completed target"
    );
    drop(connection);
    fs::write(&record_path, &record_bytes)?;
    let reopened = at(
        "reopen after restoring exact migration receipt",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    let checks = target_checks(&reopened, &[(&first, 31), (&second, 32)], &stop)?;
    let validated = at(
        "finish with fresh checks after receipt restoration",
        reopened.finish(checks, &stop),
    )?;
    assert_eq!(validated.receipt(), &expected_receipt);
    assert_eq!(fs::read(&record_path)?, record_bytes);
    assert_eq!(
        sql_inventory(&validated.candidate.store.db.connection)?,
        before
    );
    drop(validated);
    drop(backup);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn mutated_target_seal_blocks_finish_and_reopen_without_changing_the_source() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at("create corruption source", fixture("target-seal", true))?;
    let original = request(&store, "fixture:migration-seal-source", "fixture:seal", 41)?;
    let current = activate(&mut store, &original, 41, &stop)?;
    let held = store.read(&ReadSelector::Current, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:migration-seal-backup")?,
        &stop,
    )?;
    let source_snapshot = backup.manifest().snapshot_digest().to_owned();
    let operation = OperationId::new("fixture:migration-seal")?;
    let migration_root = root.join("migration");
    let mut candidate = at(
        "stage target to corrupt",
        MigrationCandidate::stage(&backup, &migration_root, &operation, &stop),
    )?;
    let target_path = candidate.store.db.path.clone();
    let epoch_dir =
        super::super::database::epoch_directory(&migration_root, candidate.target_epoch())?;
    for manifest_path in [
        migration_root.join("epoch-manifest.json"),
        epoch_dir.join("epoch-manifest.json"),
    ] {
        let checks = target_checks(&candidate, &[(&original, 41)], &stop)?;
        let before = sql_inventory(&candidate.store.db.connection)?;
        let bytes = fs::read(&manifest_path)?;
        fs::remove_file(&manifest_path)?;
        rejected(
            candidate.finish(checks, &stop),
            StoreErrorCode::DatabaseUnavailable,
        )?;
        let connection = super::super::database::connect(&target_path, true)?;
        connection.execute_batch("BEGIN DEFERRED")?;
        assert_eq!(
            sql_inventory(&connection)?,
            before,
            "missing epoch manifest allowed SQL validation mutation"
        );
        drop(connection);
        assert_eq!(store.current()?.as_ref(), Some(&current));
        check(&held, 41, &stop)?;
        fs::write(&manifest_path, &bytes)?;
        candidate = at(
            "reopen after restoring exact epoch manifest",
            MigrationCandidate::open(
                &migration_root,
                &catalog,
                &operation,
                &source_snapshot,
                &stop,
            ),
        )?;
        assert_eq!(sql_inventory(&candidate.store.db.connection)?, before);
        assert!(candidate.store.current()?.is_none());
    }
    let checks = target_checks(&candidate, &[(&original, 41)], &stop)?;
    let member = {
        let targets = candidate.target_generations();
        assert_eq!(targets.len(), 1);
        candidate
            .read_generation(&targets[0], &stop)?
            .manifest()
            .members[0]
            .clone()
    };
    let replacement = serde_json::to_vec(&vec![42u32])?;
    assert_eq!(replacement.len(), member.byte_length);
    assert_eq!(
        candidate.store.db.connection.execute(
            "UPDATE partition_versions SET payload=?2 WHERE version=?1",
            rusqlite::params![member.version.as_str(), &replacement],
        )?,
        1
    );
    rejected(
        candidate.finish(checks, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    rejected(
        MigrationCandidate::open(
            &migration_root,
            &catalog,
            &operation,
            &source_snapshot,
            &stop,
        ),
        StoreErrorCode::IntegrityViolation,
    )?;
    let connection = super::super::database::connect(&target_path, true)?;
    let actual: Vec<u8> = connection.query_row(
        "SELECT payload FROM partition_versions WHERE version=?1",
        [member.version.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(
        actual, replacement,
        "rejection silently repaired the target"
    );
    drop(connection);
    assert!(!migration_root.join("project-store-registry.json").exists());
    backup.verify(&stop)?;
    assert_eq!(store.current()?.as_ref(), Some(&current));
    check(&held, 41, &stop)?;
    drop(held);
    drop(backup);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn missing_declared_work_resumes_but_foreign_prepared_work_rejects_before_any_resume_write()
-> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at(
        "create partial-state source",
        fixture("partial-state", false),
    )?;
    let first = request(&store, "fixture:partial-first", "fixture:partial-first", 61)?;
    activate(&mut store, &first, 61, &stop)?;
    let second = request(
        &store,
        "fixture:partial-second",
        "fixture:partial-second",
        62,
    )?;
    let current = activate(&mut store, &second, 62, &stop)?;
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:partial-backup")?,
        &stop,
    )?;
    let snapshot = backup.manifest().snapshot_digest().to_owned();
    let operation = OperationId::new("fixture:partial-migration")?;
    let migration_root = root.join("migration");
    let mut candidate = at(
        "stage two distinct targets",
        MigrationCandidate::stage(&backup, &migration_root, &operation, &stop),
    )?;
    assert_eq!(candidate.target_generations().len(), 2);
    let keep = candidate
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == &first.generation().generation_id)
        .ok_or("missing first declared mapping")?
        .clone();
    let missing = candidate
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == &second.generation().generation_id)
        .ok_or("missing second declared mapping")?
        .clone();
    assert_ne!(keep.target_generation(), missing.target_generation());
    let complete = sql_inventory(&candidate.store.db.connection)?;
    at(
        "remove declared-only operation and membership",
        remove_declared_generation(&mut candidate, &missing),
    )?;
    assert!(candidate.store.operation(missing.operation_id())?.is_none());
    rejected(
        candidate.read_generation(missing.target_generation(), &stop),
        StoreErrorCode::GenerationMissing,
    )?;
    drop(candidate);
    let mut candidate = at(
        "resume missing declared-only work",
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
    )?;
    assert_eq!(sql_inventory(&candidate.store.db.connection)?, complete);
    assert_eq!(
        target_checks(&candidate, &[(&first, 61), (&second, 62)], &stop)?.len(),
        2
    );

    at(
        "remove declared work before foreign receipt",
        remove_declared_generation(&mut candidate, &missing),
    )?;
    let record = {
        let read = candidate.read_generation(keep.target_generation(), &stop)?;
        read.record("fixture.data", &stop)?
            .ok_or("missing retained seal")?
    };
    let foreign = PublicationRequest::new(
        candidate.target_epoch(),
        OperationId::new("fixture:foreign-prepared")?,
        None,
        [("fixture.owner".into(), "fixture:foreign-prepared".into())].into(),
        vec![record.clone()],
    )?;
    assert!(
        !candidate
            .target_generations()
            .contains(&foreign.generation().generation_id)
    );
    let original_bytes: Vec<u8> = candidate.store.db.connection.query_row(
        "SELECT payload FROM partition_versions WHERE version=?1",
        [record.version().as_str()],
        |row| row.get(0),
    )?;
    let temporary_bytes = serde_json::to_vec(&vec![99u32])?;
    assert_eq!(temporary_bytes.len(), original_bytes.len());
    assert_eq!(
        candidate.store.db.connection.execute(
            "UPDATE partition_versions SET payload=?2 WHERE version=?1",
            rusqlite::params![record.version().as_str(), &temporary_bytes],
        )?,
        1
    );
    let interrupted_prepare = candidate.store.prepare(&foreign, &stop);
    assert_eq!(
        candidate.store.db.connection.execute(
            "UPDATE partition_versions SET payload=?2 WHERE version=?1",
            rusqlite::params![record.version().as_str(), &original_bytes],
        )?,
        1
    );
    rejected(interrupted_prepare, StoreErrorCode::IntegrityViolation)?;
    let retained_foreign = candidate
        .store
        .operation(foreign.operation_id())?
        .ok_or("prepare did not retain its genuine foreign receipt")?;
    assert_eq!(retained_foreign.state, PublicationState::Prepared);
    assert_eq!(retained_foreign.request_digest, foreign.request_digest());
    assert_eq!(
        retained_foreign.generation_id,
        foreign.generation().generation_id
    );
    assert!(retained_foreign.activation.is_none());
    assert!(
        candidate
            .store
            .recovery_report(&stop)?
            .incidents()
            .is_empty()
    );
    assert!(candidate.store.operation(missing.operation_id())?.is_none());
    let before = sql_inventory(&candidate.store.db.connection)?;
    let target_path = candidate.store.db.path.clone();
    drop(candidate);
    rejected(
        MigrationCandidate::open(&migration_root, &catalog, &operation, &snapshot, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    let connection = super::super::database::connect(&target_path, true)?;
    connection.execute_batch("BEGIN DEFERRED")?;
    assert_eq!(
        sql_inventory(&connection)?,
        before,
        "rejection recreated declared work or altered foreign evidence"
    );
    let missing_generations: i64 = connection.query_row(
        "SELECT count(*) FROM generations WHERE generation_id=?1",
        [missing.target_generation().as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(missing_generations, 0);
    drop(connection);
    backup.verify(&stop)?;
    assert_eq!(store.current()?.as_ref(), Some(&current));
    drop(backup);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
