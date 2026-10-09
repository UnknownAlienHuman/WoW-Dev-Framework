use std::{collections::BTreeSet, error::Error, sync::atomic::AtomicBool};
use wow_store::project::{
    PartitionRecord, ProjectGcPolicy, ProjectStore, PublicationOperation, PublicationRequest,
    ReadSelector, RecordCatalog, RetentionRoot, RetentionRootId, RetentionRootKind,
};
use wow_store::{OperationId, StoreErrorCode};
type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn publish(
    store: &mut ProjectStore,
    name: &str,
    value: u32,
    stop: &AtomicBool,
) -> TestResult<(PublicationRequest, PublicationOperation)> {
    let request = PublicationRequest::new(
        store.epoch(),
        OperationId::new(format!("fixture:{name}"))?,
        store.current()?.map(|c| c.record_id),
        [("fixture.owner".into(), format!("fixture:{name}"))].into(),
        vec![
            PartitionRecord::new("fixture.shared", "fixture.partition.v1", &vec![7u32])?,
            PartitionRecord::new("fixture.data", "fixture.partition.v1", &vec![value])?,
        ],
    )?;
    store.prepare(&request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    // This fixture owner actually checks its two sealed data records before
    // issuing the compiled owner-validation capability; no serialized success.
    assert_eq!(
        read.record("fixture.data", stop)?
            .ok_or("missing data")?
            .decode::<Vec<u32>>()?,
        vec![value]
    );
    assert_eq!(
        read.record("fixture.shared", stop)?
            .ok_or("missing shared data")?
            .decode::<Vec<u32>>()?,
        vec![7]
    );
    let validation = read.owner_validation(&["fixture.owner.v1"])?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validation,
        stop,
    )?;
    let operation = store.activate(request.operation_id(), request.request_digest(), stop)?;
    Ok((request, operation))
}

#[test]
fn exact_gc_preserves_roots_shared_versions_and_released_idempotency() -> TestResult {
    let root = std::env::temp_dir().join(format!(
        "wow-project-gc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let mut store = ProjectStore::create_with_gc(&root, "fixture.gc", catalog.clone())?;
    let stop = AtomicBool::new(false);
    let (a, a_op) = publish(&mut store, "a", 10, &stop)?;
    let old = store.read(
        &ReadSelector::Exact(a.generation().generation_id.clone()),
        &stop,
    )?;
    let (b, b_op) = publish(&mut store, "b", 20, &stop)?;
    let (_, c_op) = publish(&mut store, "c", 30, &stop)?;
    let current = store.current()?.ok_or("missing current")?;
    for op in [&a_op, &b_op, &c_op] {
        let released = store.release_publication(
            &op.operation_id,
            &op.canonical_digest()?,
            "fixture:policy",
            &stop,
        )?;
        assert_eq!(released.activation, op.activation);
        assert!(released.release.is_some());
    }
    let pin = RetentionRoot::new(
        store.epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:pin-b")?,
        RetentionRootKind::User,
        b_op.generation_id.clone(),
        "fixture:operator",
    )?;
    store.put_retention_root(&pin, &stop)?;
    let policy = ProjectGcPolicy::new("fixture:gc-policy", BTreeSet::new(), 16, 32, 1024 * 1024)?;
    store.select_gc_policy(&policy, None, &stop)?;
    assert_eq!(
        store
            .select_gc_policy(&policy, None, &stop)
            .err()
            .ok_or("absence CAS ignored for existing policy")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    let plan = store.plan_gc(&policy, &stop)?;
    assert!(plan.report().delete_generations().is_empty());
    let changed_policy = ProjectGcPolicy::new(
        "fixture:changed-policy",
        BTreeSet::new(),
        8,
        16,
        1024 * 1024,
    )?;
    store.select_gc_policy(&changed_policy, Some(&policy.canonical_digest()?), &stop)?;
    assert_eq!(
        store
            .execute_gc(&plan, &OperationId::new("fixture:stale-policy")?, &stop)
            .err()
            .ok_or("stale policy plan accepted")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    store.select_gc_policy(&policy, Some(&changed_policy.canonical_digest()?), &stop)?;
    let plan = store.plan_gc(&policy, &stop)?;
    store.remove_retention_root(pin.root_id(), pin.pin_digest(), &stop)?;
    assert_eq!(
        store
            .execute_gc(&plan, &OperationId::new("fixture:stale-root")?, &stop)
            .err()
            .ok_or("stale root plan accepted")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    let plan = store.plan_gc(&policy, &stop)?;
    assert_eq!(
        plan.report().delete_generations(),
        std::slice::from_ref(&b_op.generation_id)
    );
    let transient = store.read(&ReadSelector::Current, &stop)?;
    drop(transient);
    assert_eq!(
        store
            .execute_gc(&plan, &OperationId::new("fixture:stale-lease")?, &stop)
            .err()
            .ok_or("lease ABA plan accepted")?
            .code(),
        StoreErrorCode::CurrentConflict
    );
    let plan = store.plan_gc(&policy, &stop)?;
    let gc_id = OperationId::new("fixture:gc-b")?;
    let receipt = store.execute_gc(&plan, &gc_id, &stop)?;
    assert_eq!(
        receipt.report().delete_generations(),
        std::slice::from_ref(&b_op.generation_id)
    );
    assert_eq!(
        receipt.report().delete_versions().len(),
        1,
        "shared partition must survive"
    );
    assert_eq!(store.execute_gc(&plan, &gc_id, &stop)?, receipt);
    assert_eq!(store.reconcile_gc(&gc_id)?, Some(receipt.clone()));
    assert_eq!(store.current()?, Some(current.clone()));
    assert_eq!(
        old.record("fixture.data", &stop)?
            .ok_or("leased data lost")?
            .decode::<Vec<u32>>()?,
        vec![10]
    );
    assert_eq!(
        store
            .read(&ReadSelector::Exact(b_op.generation_id.clone()), &stop)
            .err()
            .ok_or("collected generation still readable")?
            .code(),
        StoreErrorCode::GenerationMissing
    );
    let released_b = store
        .reconcile(&b_op.operation_id)?
        .ok_or("release evidence lost")?;
    assert_eq!(released_b.activation, b_op.activation);
    assert!(released_b.release.is_some());
    assert_eq!(
        store
            .prepare(&b, &stop)
            .err()
            .ok_or("released retry resumed")?
            .code(),
        StoreErrorCode::OperationStateInvalid
    );
    let substitute = PublicationRequest::new(
        store.epoch(),
        b_op.operation_id.clone(),
        Some(current.record_id.clone()),
        [("fixture.owner".into(), "fixture:substitute".into())].into(),
        vec![PartitionRecord::new(
            "fixture.data",
            "fixture.partition.v1",
            &vec![999u32],
        )?],
    )?;
    assert_eq!(
        store
            .prepare(&substitute, &stop)
            .err()
            .ok_or("released ID reused")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    drop(old);
    let plan = store.plan_gc(&policy, &stop)?;
    assert_eq!(
        plan.report().delete_generations(),
        std::slice::from_ref(&a_op.generation_id)
    );
    assert_eq!(
        store
            .execute_gc(
                &plan,
                &OperationId::new("fixture:cancelled")?,
                &AtomicBool::new(true)
            )
            .err()
            .ok_or("cancelled GC accepted")?
            .code(),
        StoreErrorCode::Cancelled
    );
    store.execute_gc(&plan, &OperationId::new("fixture:gc-a")?, &stop)?;
    drop(store);
    let store = ProjectStore::open(&root, &catalog)?;
    assert_eq!(store.current()?, Some(current));
    assert_eq!(store.reconcile_gc(&gc_id)?, Some(receipt));
    assert!(
        store
            .reconcile(&a_op.operation_id)?
            .ok_or("tombstone lost on reopen")?
            .release
            .is_some()
    );
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        read.record("fixture.shared", &stop)?
            .ok_or("shared data lost")?
            .decode::<Vec<u32>>()?,
        vec![7]
    );
    drop(read);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
