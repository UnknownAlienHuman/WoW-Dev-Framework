use super::*;
use std::collections::BTreeSet;
use wow_store::project::{GC_PHYSICAL_PROFILE, ProjectGcPolicy};

#[test]
fn service_gc_preserves_leases_released_receipts_and_current_after_reopen() -> TestResult {
    let stop = AtomicBool::new(false);
    let root = root("gc-service")?;
    let (publisher_a, graph_a) = owners("return External()")?;
    let (publisher_b, graph_b) = owners("return External()+1")?;
    assert_eq!(graph_a.snapshot().universe(), graph_b.snapshot().universe());
    assert_ne!(
        graph_a.snapshot().snapshot_id(),
        graph_b.snapshot().snapshot_id()
    );

    let mut store = LiveProjectStore::create(&root, graph_a.snapshot().universe().as_str())?;
    assert_eq!(
        store.storage_epoch().physical_profile(),
        GC_PHYSICAL_PROFILE
    );
    let operation_a = store.publish(&publisher_a, &graph_a, "fixture:gc-a", None, &stop)?;
    assert_eq!(operation_a.state, PublicationState::Activated);
    let activation_a = operation_a
        .activation
        .clone()
        .ok_or("missing activation a")?;
    assert_eq!(store.current()?, Some(activation_a.clone()));
    let old = store.read(
        &ReadSelector::Exact(operation_a.generation_id.clone()),
        &stop,
    )?;
    assert_eq!(old.graph(), &graph_a);

    let operation_b = store.publish(
        &publisher_b,
        &graph_b,
        "fixture:gc-b",
        Some(activation_a.record_id.clone()),
        &stop,
    )?;
    assert_eq!(operation_b.state, PublicationState::Activated);
    let activation_b = operation_b
        .activation
        .clone()
        .ok_or("missing activation b")?;
    assert_eq!(
        activation_b.predecessor,
        Some(activation_a.record_id.clone())
    );
    assert_eq!(store.current()?, Some(activation_b.clone()));
    assert_eq!(old.graph(), &graph_a);

    let released_a = store.release_publication(
        "fixture:gc-a",
        &operation_a.canonical_digest()?,
        "fixture:policy",
        &stop,
    )?;
    let released_b = store.release_publication(
        "fixture:gc-b",
        &operation_b.canonical_digest()?,
        "fixture:policy",
        &stop,
    )?;
    assert_eq!(released_a.activation, operation_a.activation);
    assert_eq!(released_b.activation, operation_b.activation);
    assert_eq!(
        released_a
            .release
            .as_ref()
            .ok_or("missing release a")?
            .held_by(),
        "fixture:policy"
    );
    assert_eq!(
        released_b
            .release
            .as_ref()
            .ok_or("missing release b")?
            .held_by(),
        "fixture:policy"
    );

    let policy = ProjectGcPolicy::new(
        "fixture:gc-policy",
        BTreeSet::new(),
        1024,
        8192,
        64 * 1024 * 1024,
    )?;
    store.select_gc_policy(&policy, None, &stop)?;
    let leased_plan = store.plan_gc(&policy, &stop)?;
    assert!(
        leased_plan
            .report()
            .protected_generations()
            .contains(&operation_a.generation_id)
    );
    assert!(
        leased_plan
            .report()
            .protected_generations()
            .contains(&operation_b.generation_id)
    );
    assert!(leased_plan.report().delete_generations().is_empty());
    assert!(leased_plan.report().delete_versions().is_empty());

    drop(old);
    assert_eq!(
        store
            .execute_gc(&leased_plan, "fixture:gc-stale", &stop)
            .err()
            .ok_or("stale leased plan accepted")?
            .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    assert!(store.reconcile_gc("fixture:gc-stale")?.is_none());
    assert_eq!(store.current()?, Some(activation_b.clone()));

    let plan = store.plan_gc(&policy, &stop)?;
    assert_eq!(
        plan.report().delete_generations(),
        std::slice::from_ref(&operation_a.generation_id)
    );
    assert!(!plan.report().delete_versions().is_empty());
    assert!(plan.report().payload_bytes() > 0);
    let receipt = store.execute_gc(&plan, "fixture:gc-collect", &stop)?;
    assert_eq!(receipt.report(), plan.report());
    assert_eq!(
        store.reconcile_gc("fixture:gc-collect")?,
        Some(receipt.clone())
    );
    assert_eq!(
        store.execute_gc(&plan, "fixture:gc-collect", &stop)?,
        receipt
    );
    assert_eq!(store.current()?, Some(activation_b.clone()));
    let current = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(current.store_generation_id(), &operation_b.generation_id);
    assert_eq!(current.graph(), &graph_b);
    drop(current);

    let reconciled_a = store
        .reconcile("fixture:gc-a")?
        .ok_or("lost released receipt a")?;
    assert_eq!(reconciled_a, released_a);
    assert_eq!(reconciled_a.activation, Some(activation_a));
    assert!(reconciled_a.release.is_some());
    assert_eq!(
        store
            .publish(&publisher_a, &graph_a, "fixture:gc-a", None, &stop)
            .err()
            .ok_or("released publication returned a false activation")?
            .code(),
        ServiceErrorCode::OperationReleased
    );
    assert_eq!(store.current()?, Some(activation_b.clone()));
    drop(store);

    let reopened = LiveProjectStore::open(&root)?;
    assert_eq!(
        reopened.storage_epoch().physical_profile(),
        GC_PHYSICAL_PROFILE
    );
    assert_eq!(reopened.reconcile_gc("fixture:gc-collect")?, Some(receipt));
    assert_eq!(reopened.current()?, Some(activation_b));
    assert_eq!(reopened.reconcile("fixture:gc-a")?, Some(released_a));
    let current = reopened.read(&ReadSelector::Current, &stop)?;
    assert_eq!(current.store_generation_id(), &operation_b.generation_id);
    assert_eq!(current.graph(), &graph_b);
    drop(current);
    drop(reopened);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
