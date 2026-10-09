use super::*;
use wow_store::project::{
    RETAINED_PHYSICAL_PROFILE, RetentionRoot, RetentionRootId, RetentionRootKind,
};

#[test]
fn service_pins_survive_reopen_and_legacy_physical_epochs_remain_readable() -> TestResult {
    let stop = AtomicBool::new(false);
    let root = root("retention-service")?;
    std::fs::create_dir(&root)?;
    let retained = root.join("retained");
    let legacy = root.join("legacy");
    let (publisher, graph) = owners("return External()")?;
    let owner = graph.snapshot().universe().as_str();
    let mut store = LiveProjectStore::create(&retained, owner)?;
    assert_eq!(
        store.storage_epoch().physical_profile(),
        RETAINED_PHYSICAL_PROFILE
    );
    let operation = store.publish(&publisher, &graph, "fixture:retained", None, &stop)?;
    let current = store.current()?.ok_or("missing retained current")?;
    let pin = RetentionRoot::new(
        store.storage_epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:rollback")?,
        RetentionRootKind::Rollback,
        operation.generation_id,
        "fixture:operator",
    )?;
    assert_eq!(store.put_retention_root(&pin, &stop)?, pin);
    drop(store);
    let mut store = LiveProjectStore::open(&retained)?;
    assert_eq!(store.retention_roots(&stop)?, vec![pin.clone()]);
    assert_eq!(store.current()?, Some(current));
    let read = store.read(&ReadSelector::Exact(pin.generation_id().clone()), &stop)?;
    assert_eq!(read.graph(), &graph);
    drop(read);
    assert!(store.remove_retention_root(pin.root_id(), pin.pin_digest(), &stop)?);
    drop(store);

    // The same registered owner catalog in a frozen v1 physical epoch remains
    // readable; retaining it must never add a table or rewrite its identity.
    let mut store = LiveProjectStore {
        store: ProjectStore::create(&legacy, owner, catalog()?)?,
    };
    let epoch = store.storage_epoch().clone();
    store.publish(&publisher, &graph, "fixture:legacy-retention", None, &stop)?;
    let legacy_current = store.current()?.ok_or("missing legacy current")?;
    assert_eq!(
        store
            .retention_roots(&stop)
            .err()
            .ok_or("v1 retention unexpectedly supported")?
            .code(),
        ServiceErrorCode::OperationNotImplementedForMilestone
    );
    assert_eq!(
        store
            .put_retention_root(&pin, &stop)
            .err()
            .ok_or("v1 pin unexpectedly accepted")?
            .code(),
        ServiceErrorCode::OperationNotImplementedForMilestone
    );
    assert_eq!(store.storage_epoch(), &epoch);
    drop(store);
    let store = LiveProjectStore::open(&legacy)?;
    assert_eq!(store.storage_epoch(), &epoch);
    assert_eq!(store.current()?, Some(legacy_current));
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.graph(), &graph);
    drop(read);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
