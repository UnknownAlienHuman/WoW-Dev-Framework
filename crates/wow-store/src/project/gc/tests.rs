use super::*;
use crate::project::{PartitionRecord, PublicationRequest, ReadSelector, RecordCatalog};
use std::{error::Error, fs, path::PathBuf};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

#[test]
fn gc_page_cap_enforces_native_full_and_restores_after_rollback() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "wow-native-gc-page-cap-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let mut store = ProjectStore::create_with_gc(&root, "fixture.gc-page-cap", catalog)?;
    let stop = AtomicBool::new(false);
    shared_publication(&mut store, 0, &stop)?;
    let policy = ProjectGcPolicy::new("fixture:page-cap", BTreeSet::new(), 1, 1, 1024)?;
    store.select_gc_policy(&policy, None, &stop)?;
    let plan = store.plan_gc(&policy, &stop)?;
    let connection = &mut store.db.connection;
    let original_limit: i64 =
        connection.pragma_query_value(None, "max_page_count", |row| row.get(0))?;
    let original_spill: i64 =
        connection.pragma_query_value(None, "cache_spill", |row| row.get(0))?;
    let before_wal = wal_bytes(&store)?;
    let limits = wal_budget::admit(&store.db, plan.report(), 0, &stop)?;
    let page_count: i64 = store
        .db
        .connection
        .pragma_query_value(None, "page_count", |row| row.get(0))?;
    let admitted: i64 = store
        .db
        .connection
        .pragma_query_value(None, "max_page_count", |row| row.get(0))?;
    assert_eq!(admitted, page_count + 64);
    // A deliberately oversized inert physical receipt probes the native page
    // allocator, not semantic receipt custody. It must fail and never commit.
    // The genuine public GC/refusal/receipt paths are covered separately below.
    let transaction = store
        .db
        .connection
        .transaction_with_behavior(TransactionBehavior::Immediate)?;
    let error = transaction
        .execute(
            "INSERT INTO gc_operations(operation_id,request_digest,record)
             VALUES('fixture:page-cap-probe','inert-allocation-probe',zeroblob(1048576))",
            [],
        )
        .err()
        .ok_or("native allocator exceeded the admitted page cap")?;
    assert!(matches!(
        &error,
        rusqlite::Error::SqliteFailure(code, _) if code.code == rusqlite::ErrorCode::DiskFull
    ));
    assert_eq!(
        StoreError::database(error).code(),
        StoreErrorCode::DatabaseUnavailable
    );
    // SQLITE_FULL may already have rolled back the outer native transaction.
    drop(transaction);
    limits.restore(&store.db.connection)?;
    assert!(store.db.connection.is_autocommit());
    assert_eq!(
        store
            .db
            .connection
            .pragma_query_value(None, "max_page_count", |row| row.get::<_, i64>(0))?,
        original_limit
    );
    assert_eq!(
        store
            .db
            .connection
            .pragma_query_value(None, "cache_spill", |row| row.get::<_, i64>(0))?,
        original_spill
    );
    assert_eq!(wal_bytes(&store)?, before_wal);
    let rows: i64 =
        store
            .db
            .connection
            .query_row("SELECT count(*) FROM gc_operations", [], |row| row.get(0))?;
    assert_eq!(rows, 0);
    assert_eq!(store.plan_gc(&policy, &stop)?.report(), plan.report());
    eprintln!(
        "native SQLite {}: enforced 64-page allocation cap -> SQLITE_FULL, rollback and settings restored",
        rusqlite::version()
    );
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}

fn wal_bytes(store: &ProjectStore) -> TestResult<u64> {
    let mut path = store.db.path.as_os_str().to_owned();
    path.push("-wal");
    match fs::metadata(PathBuf::from(path)) {
        Ok(metadata) => Ok(metadata.len()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.into()),
    }
}

fn shared_publication(store: &mut ProjectStore, sequence: usize, stop: &AtomicBool) -> TestResult {
    let records = (0..MAX_PARTITIONS)
        .map(|member| {
            PartitionRecord::new(
                format!("fixture.part.{member:03}.{}", "x".repeat(192)),
                "fixture.partition.v1",
                &1u8,
            )
        })
        .collect::<StoreResult<Vec<_>>>()?;
    let request = PublicationRequest::new(
        store.epoch(),
        OperationId::new(format!("fixture:shared-{sequence}"))?,
        store.current()?.map(|current| current.record_id),
        [("fixture.owner".into(), format!("fixture:shared-{sequence}"))].into(),
        records,
    )?;
    store.prepare(&request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    for member in &request.generation().members {
        assert_eq!(
            read.record(&member.key, stop)?
                .ok_or("missing shared partition")?
                .decode::<u8>()?,
            1
        );
    }
    let validated = read.owner_validation(&["fixture.owner.v1"])?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validated,
        stop,
    )?;
    let operation = store.activate(request.operation_id(), request.request_digest(), stop)?;
    store.release_publication(
        &operation.operation_id,
        &operation.canonical_digest()?,
        "fixture:gc-owner",
        stop,
    )?;
    Ok(())
}

#[test]
fn shared_membership_gc_refuses_wal_pressure_before_mutation() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "wow-native-gc-wal-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let mut store = ProjectStore::create_with_gc(&root, "fixture.gc-wal", catalog.clone())?;
    let stop = AtomicBool::new(false);
    for sequence in 0..81 {
        shared_publication(&mut store, sequence, &stop)?;
    }
    let current = store.current()?.ok_or("missing current")?;
    let reader = store.read(&ReadSelector::Current, &stop)?;
    // These are real sealed native partitions, complete manifests and unreleased
    // operations. The pinned last generation remains both current and a reader
    // root while its predecessors are eligible for collection.
    for sequence in 0..160 {
        if wal_bytes(&store)? >= 120 * 1024 * 1024 {
            break;
        }
        let value = format!("{sequence:04}{}", "w".repeat(1024 * 1024 - 4));
        let request = PublicationRequest::new(
            store.epoch(),
            OperationId::new(format!("fixture:wal-fill-{sequence}"))?,
            Some(current.record_id.clone()),
            [(
                "fixture.owner".into(),
                format!("fixture:wal-fill-{sequence}"),
            )]
            .into(),
            vec![PartitionRecord::new(
                "fixture.fill",
                "fixture.partition.v1",
                &value,
            )?],
        )?;
        store.prepare(&request, &stop)?;
    }
    let policy = ProjectGcPolicy::new("fixture:wal-gc", BTreeSet::new(), 80, 256, 1024 * 1024)?;
    store.select_gc_policy(&policy, None, &stop)?;
    let plan = store.plan_gc(&policy, &stop)?;
    assert_eq!(plan.report().delete_generations().len(), 80);
    assert!(plan.report().delete_versions().is_empty());
    let before_wal = wal_bytes(&store)?;
    assert!(before_wal >= 120 * 1024 * 1024);
    assert!(before_wal + 7_340_032 <= 128 * 1024 * 1024);
    let before_changes = store.db.connection.total_changes();
    let operation = OperationId::new("fixture:wal-gc-refusal")?;
    let error = store
        .execute_gc(&plan, &operation, &stop)
        .err()
        .ok_or("GC ignored membership WAL pressure")?;
    assert_eq!(error.code(), StoreErrorCode::BudgetExceeded);
    assert_eq!(store.db.connection.total_changes(), before_changes);
    assert_eq!(wal_bytes(&store)?, before_wal);
    assert!(store.reconcile_gc(&operation)?.is_none());
    assert_eq!(store.current()?, Some(current.clone()));
    assert_eq!(store.plan_gc(&policy, &stop)?.report(), plan.report());
    let shared = format!("fixture.part.000.{}", "x".repeat(192));
    assert_eq!(
        reader
            .record(&shared, &stop)?
            .ok_or("reader lost retained shared data")?
            .decode::<u8>()?,
        1
    );
    eprintln!(
        "native SQLite {}: 80x256 selected memberships, WAL {before_wal}, refusal unchanged",
        rusqlite::version()
    );
    drop(reader);
    drop(store);
    // Checked close/reopen ends the old WAL pressure. The same large batch must
    // still execute with a real retained reader when its physical pages fit.
    let mut store = ProjectStore::open(&root, &catalog)?;
    assert_eq!(store.current()?, Some(current.clone()));
    let reader = store.read(&ReadSelector::Current, &stop)?;
    let page_limit: i64 =
        store
            .db
            .connection
            .pragma_query_value(None, "max_page_count", |row| row.get(0))?;
    let spill: i64 = store
        .db
        .connection
        .pragma_query_value(None, "cache_spill", |row| row.get(0))?;
    let plan = store.plan_gc(&policy, &stop)?;
    assert_eq!(plan.report().delete_generations().len(), 80);
    let operation = OperationId::new("fixture:wal-gc-positive")?;
    let receipt = store.execute_gc(&plan, &operation, &stop)?;
    assert_eq!(store.reconcile_gc(&operation)?, Some(receipt.clone()));
    assert_eq!(store.execute_gc(&plan, &operation, &stop)?, receipt);
    assert_eq!(store.current()?, Some(current));
    assert!(wal_bytes(&store)? <= 128 * 1024 * 1024);
    assert_eq!(
        store
            .db
            .connection
            .pragma_query_value(None, "max_page_count", |row| row.get::<_, i64>(0))?,
        page_limit
    );
    assert_eq!(
        store
            .db
            .connection
            .pragma_query_value(None, "cache_spill", |row| row.get::<_, i64>(0))?,
        spill
    );
    assert_eq!(
        reader
            .record(&shared, &stop)?
            .ok_or("GC lost retained reader data")?
            .decode::<u8>()?,
        1
    );
    eprintln!(
        "same native 80x256 batch committed at WAL {}",
        wal_bytes(&store)?
    );
    drop(reader);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
