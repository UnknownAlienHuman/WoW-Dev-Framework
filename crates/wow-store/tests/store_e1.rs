use std::{
    error::Error,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};
use serde_json::json;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, LeaseId, LogicalEpoch,
    ObjectId, OperationBegin, OperationId, OperationState, PendingObject, RequestDigest, Store,
    StoreConfiguration, StoreErrorCode, StoreLimits, WriteBatch,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

static PATH_SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn configuration(profile: &str) -> Result<StoreConfiguration, wow_store::StoreError> {
    StoreConfiguration::new(profile, StoreLimits::default())
}

fn pending(value: serde_json::Value) -> Result<PendingObject, wow_store::StoreError> {
    PendingObject::from_json("fixture.record", 1, &value, StoreLimits::default())
}

fn write_object(store: &mut Store, object: PendingObject) -> TestResult<ObjectId> {
    let object_id = object.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(object)?;
    store.commit(batch)?;
    Ok(object_id)
}

fn temporary_database(label: &str) -> PathBuf {
    let sequence = PATH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "wow-store-e1-{label}-{}-{sequence}.sqlite3",
        std::process::id()
    ))
}

fn remove_database(path: &PathBuf) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(path.with_extension("sqlite3-wal"));
    let _ = fs::remove_file(path.with_extension("sqlite3-shm"));
}

#[test]
fn immutable_objects_are_canonical_deduplicated_and_typed() -> TestResult {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Record {
        zeta: u32,
        alpha: Box<str>,
    }
    let configuration = configuration("fixture-store")?;
    let mut store = Store::open_in_memory(configuration)?;
    let left = PendingObject::from_json(
        "fixture.record",
        1,
        &json!({"zeta":7,"alpha":"value"}),
        StoreLimits::default(),
    )?;
    let right = PendingObject::from_json(
        "fixture.record",
        1,
        &json!({"alpha":"value","zeta":7}),
        StoreLimits::default(),
    )?;
    assert_eq!(left.object_id(), right.object_id());
    assert_eq!(left.canonical_json(), right.canonical_json());

    let object_id = left.object_id().clone();
    let mut first = WriteBatch::new();
    first.add_object(left)?;
    let first_receipt = store.commit(first)?;
    assert_eq!(first_receipt.inserted(), std::slice::from_ref(&object_id));

    let mut second = WriteBatch::new();
    second.add_object(right)?;
    let second_receipt = store.commit(second)?;
    assert_eq!(
        second_receipt.deduplicated(),
        std::slice::from_ref(&object_id)
    );
    assert_ne!(first_receipt.receipt_id(), second_receipt.receipt_id());

    let record = store.object(&object_id)?.ok_or("missing stored object")?;
    assert_eq!(record.kind(), "fixture.record");
    assert_eq!(record.schema_version(), 1);
    assert_eq!(
        record.decode::<Record>()?,
        Record {
            zeta: 7,
            alpha: "value".into(),
        }
    );
    assert!(record.content_sha256().starts_with("sha256:"));
    Ok(())
}

#[test]
fn catalog_compare_and_swap_is_atomic_with_object_insertion() -> TestResult {
    let mut store = Store::open_in_memory(configuration("catalog-cas")?)?;
    let catalog = CatalogName::new("reference.current")?;
    let path = CatalogPath::new("retail/mainline")?;
    let first = pending(json!({"generation":1}))?;
    let first_id = first.object_id().clone();
    let mut initial = WriteBatch::new();
    initial.add_object(first)?;
    initial.add_catalog_mutation(CatalogMutation::set(
        catalog.clone(),
        path.clone(),
        CatalogExpectation::Absent,
        first_id.clone(),
    ))?;
    store.commit(initial)?;

    let rejected = pending(json!({"generation":2}))?;
    let rejected_id = rejected.object_id().clone();
    let mut conflict = WriteBatch::new();
    conflict.add_object(rejected)?;
    conflict.add_catalog_mutation(CatalogMutation::set(
        catalog.clone(),
        path.clone(),
        CatalogExpectation::Absent,
        rejected_id.clone(),
    ))?;
    let error = store
        .commit(conflict)
        .err()
        .ok_or("expected catalog conflict")?;
    assert_eq!(error.code(), StoreErrorCode::CatalogConflict);
    assert!(store.object(&rejected_id)?.is_none());
    assert_eq!(
        store
            .catalog_entry(&catalog, &path)?
            .ok_or("missing catalog entry")?
            .object_id(),
        &first_id
    );

    let replacement = pending(json!({"generation":3}))?;
    let replacement_id = replacement.object_id().clone();
    let mut update = WriteBatch::new();
    update.add_object(replacement)?;
    update.add_catalog_mutation(CatalogMutation::set(
        catalog.clone(),
        path.clone(),
        CatalogExpectation::Exact(first_id),
        replacement_id.clone(),
    ))?;
    store.commit(update)?;
    assert_eq!(
        store
            .catalog_entry(&catalog, &path)?
            .ok_or("missing updated catalog entry")?
            .object_id(),
        &replacement_id
    );
    Ok(())
}

#[test]
fn operation_journal_replays_exact_requests_and_rejects_conflicts() -> TestResult {
    let mut store = Store::open_in_memory(configuration("operations")?)?;
    let result_object = pending(json!({"result":"complete"}))?;
    let result_id = write_object(&mut store, result_object)?;
    let operation_id = OperationId::new("operation:fixture:1")?;
    let request_digest = RequestDigest::new(format!("sha256:{}", "1".repeat(64)))?;
    let other_digest = RequestDigest::new(format!("sha256:{}", "2".repeat(64)))?;

    let OperationBegin::Started(prepared) =
        store.begin_operation(operation_id.clone(), request_digest.clone())?
    else {
        return Err("first operation registration was not started".into());
    };
    assert_eq!(prepared.state(), OperationState::Prepared);
    let OperationBegin::Replay(replayed) =
        store.begin_operation(operation_id.clone(), request_digest.clone())?
    else {
        return Err("same request did not replay".into());
    };
    assert_eq!(replayed, prepared);
    assert_eq!(
        store
            .begin_operation(operation_id.clone(), other_digest)
            .err()
            .ok_or("expected operation conflict")?
            .code(),
        StoreErrorCode::OperationConflict
    );

    let completed = store.complete_operation(&operation_id, &request_digest, &result_id)?;
    assert_eq!(completed.state(), OperationState::Completed);
    assert_eq!(completed.result_object_id(), Some(&result_id));
    assert_eq!(
        store.complete_operation(&operation_id, &request_digest, &result_id)?,
        completed
    );
    assert_eq!(
        store
            .record_failed(&operation_id, &request_digest)
            .err()
            .ok_or("terminal transition should fail")?
            .code(),
        StoreErrorCode::OperationStateInvalid
    );
    Ok(())
}

#[test]
fn leases_and_durable_roots_bound_garbage_collection() -> TestResult {
    let mut store = Store::open_in_memory(configuration("gc")?)?;
    let catalog_root = pending(json!({"root":"catalog"}))?;
    let catalog_root_id = catalog_root.object_id().clone();
    let leased = pending(json!({"root":"lease"}))?;
    let leased_id = leased.object_id().clone();
    let unrooted = pending(json!({"root":"none"}))?;
    let unrooted_id = unrooted.object_id().clone();
    let operation_root = pending(json!({"root":"operation"}))?;
    let operation_root_id = operation_root.object_id().clone();

    let mut batch = WriteBatch::new();
    for object in [catalog_root, leased, unrooted, operation_root] {
        batch.add_object(object)?;
    }
    batch.add_catalog_mutation(CatalogMutation::set(
        CatalogName::new("project.current")?,
        CatalogPath::new("fixture")?,
        CatalogExpectation::Absent,
        catalog_root_id.clone(),
    ))?;
    store.commit(batch)?;
    store.acquire_lease(
        LeaseId::new("lease:fixture:1")?,
        leased_id.clone(),
        "test-holder",
        LogicalEpoch::new(10),
    )?;
    let operation_id = OperationId::new("operation:gc:1")?;
    let request_digest = RequestDigest::new(format!("sha256:{}", "3".repeat(64)))?;
    store.begin_operation(operation_id.clone(), request_digest.clone())?;
    store.complete_operation(&operation_id, &request_digest, &operation_root_id)?;

    let first = store.collect_garbage(LogicalEpoch::new(5), 10)?;
    assert_eq!(first.deleted(), std::slice::from_ref(&unrooted_id));
    assert!(store.object(&catalog_root_id)?.is_some());
    assert!(store.object(&leased_id)?.is_some());
    assert!(store.object(&operation_root_id)?.is_some());

    let second = store.collect_garbage(LogicalEpoch::new(10), 10)?;
    assert_eq!(second.expired_leases_deleted(), 1);
    assert_eq!(second.deleted(), std::slice::from_ref(&leased_id));
    assert!(store.object(&catalog_root_id)?.is_some());
    assert!(store.object(&operation_root_id)?.is_some());
    Ok(())
}

#[test]
fn logical_manifest_is_order_invariant_and_persists_across_reopen() -> TestResult {
    let path = temporary_database("reopen");
    remove_database(&path);
    let configuration = configuration("persistent")?;
    let manifest_before = {
        let mut store = Store::open(&path, configuration.clone())?;
        let first = pending(json!({"value":"a"}))?;
        let second = pending(json!({"value":"b"}))?;
        let mut batch = WriteBatch::new();
        batch.add_object(second)?;
        batch.add_object(first)?;
        store.commit(batch)?;
        let report = store.validate_integrity(100)?;
        assert!(report.complete());
        store.logical_manifest()?
    };
    let reopened = Store::open(&path, configuration.clone())?;
    let manifest_after = reopened.logical_manifest()?;
    assert_eq!(manifest_before, manifest_after);
    assert_eq!(manifest_before.objects().len(), 2);
    assert!(
        manifest_before
            .manifest_id()
            .starts_with("store-manifest:sha256:")
    );
    drop(reopened);

    let incompatible = configuration("other-profile")?;
    assert_eq!(
        Store::open(&path, incompatible)
            .err()
            .ok_or("expected configuration mismatch")?
            .code(),
        StoreErrorCode::ConfigurationInvalid
    );
    remove_database(&path);
    Ok(())
}

#[test]
fn bounded_gc_reports_continuation_without_deleting_roots() -> TestResult {
    let mut store = Store::open_in_memory(configuration("bounded-gc")?)?;
    for value in 0..3 {
        let object = pending(json!({"value":value}))?;
        write_object(&mut store, object)?;
    }
    let first = store.collect_garbage(LogicalEpoch::new(0), 2)?;
    assert_eq!(first.deleted().len(), 2);
    assert!(first.more_available());
    let second = store.collect_garbage(LogicalEpoch::new(0), 2)?;
    assert_eq!(second.deleted().len(), 1);
    assert!(!second.more_available());
    assert!(store.logical_manifest()?.objects().is_empty());
    Ok(())
}
