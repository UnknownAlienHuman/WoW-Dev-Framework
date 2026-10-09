use std::{error::Error, path::PathBuf, sync::atomic::AtomicBool};
use wow_store::project::{
    EpochId, PartitionRecord, ProjectStore, PublicationRequest, RETAINED_PHYSICAL_PROFILE,
    ReadSelector, RecordCatalog, RetentionRoot, RetentionRootId, RetentionRootKind,
    StoreGenerationId,
};
use wow_store::{OperationId, StoreErrorCode};
type TestResult = Result<(), Box<dyn Error>>;

fn root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(std::env::temp_dir().join(format!(
        "wow-retention-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    )))
}

#[test]
fn persistent_roots_bind_exact_epoch_generation_and_digest_without_migrating_v1() -> TestResult {
    let stop = AtomicBool::new(false);
    let store_root = root("v2")?;
    let legacy_root = root("v1")?;
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let mut store =
        ProjectStore::create_with_retention(&store_root, "fixture.retention", catalog.clone())?;
    assert_eq!(store.epoch().physical_profile(), RETAINED_PHYSICAL_PROFILE);
    let epoch = store.epoch().clone();
    // Real sealed records and inactive complete membership, not an invented generation.
    let request = PublicationRequest::new(
        store.epoch(),
        OperationId::new("fixture:retention")?,
        None,
        [("fixture.generation".into(), "fixture:retained".into())].into(),
        vec![PartitionRecord::new(
            "fixture.data",
            "fixture.partition.v1",
            &vec![1u32, 2],
        )?],
    )?;
    store.prepare(&request, &stop)?;
    let generation = request.generation().generation_id.clone();
    let read = store.read(&ReadSelector::Exact(generation.clone()), &stop)?;
    assert_eq!(
        read.record("fixture.data", &stop)?
            .ok_or("missing sealed member")?
            .decode::<Vec<u32>>()?,
        vec![1, 2]
    );
    drop(read);
    let id = RetentionRootId::new("fixture:user-hold")?;
    let pin = RetentionRoot::new(
        epoch.epoch_id().clone(),
        id.clone(),
        RetentionRootKind::User,
        generation.clone(),
        "fixture:owner",
    )?;
    assert_eq!(store.put_retention_root(&pin, &stop)?, pin);
    assert_eq!(store.put_retention_root(&pin, &stop)?, pin);
    assert_eq!(store.retention_roots(&stop)?, vec![pin.clone()]);
    let substitution = RetentionRoot::new(
        epoch.epoch_id().clone(),
        id.clone(),
        RetentionRootKind::Debug,
        generation.clone(),
        "fixture:owner",
    )?;
    assert_eq!(
        store
            .put_retention_root(&substitution, &stop)
            .err()
            .ok_or("pin substitution accepted")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    let foreign = RetentionRoot::new(
        EpochId::parse(format!("project-epoch:sha256:{}", "0".repeat(64)))?,
        RetentionRootId::new("fixture:foreign")?,
        RetentionRootKind::User,
        generation.clone(),
        "fixture:owner",
    )?;
    assert_eq!(
        store
            .put_retention_root(&foreign, &stop)
            .err()
            .ok_or("foreign epoch accepted")?
            .code(),
        StoreErrorCode::IntegrityViolation
    );
    let missing = RetentionRoot::new(
        epoch.epoch_id().clone(),
        RetentionRootId::new("fixture:missing")?,
        RetentionRootKind::User,
        StoreGenerationId::parse(format!(
            "project-store-generation:sha256:{}",
            "0".repeat(64)
        ))?,
        "fixture:owner",
    )?;
    assert_eq!(
        store
            .put_retention_root(&missing, &stop)
            .err()
            .ok_or("unknown generation accepted")?
            .code(),
        StoreErrorCode::GenerationMissing
    );
    assert_eq!(
        store
            .put_retention_root(&substitution, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled pin accepted")?
            .code(),
        StoreErrorCode::Cancelled
    );
    assert_eq!(store.retention_roots(&stop)?, vec![pin.clone()]);
    assert!(
        store.current()?.is_none(),
        "pin must not activate a publication"
    );
    drop(store);
    let mut store = ProjectStore::open(&store_root, &catalog)?;
    assert_eq!(store.epoch(), &epoch);
    assert_eq!(store.retention_roots(&stop)?, vec![pin.clone()]);
    assert_eq!(
        store
            .remove_retention_root(&id, "different-digest", &stop)
            .err()
            .ok_or("stale removal accepted")?
            .code(),
        StoreErrorCode::OperationConflict
    );
    assert_eq!(store.retention_roots(&stop)?, vec![pin.clone()]);
    assert!(store.remove_retention_root(&id, pin.pin_digest(), &stop)?);
    assert!(!store.remove_retention_root(&id, pin.pin_digest(), &stop)?);
    assert!(store.retention_roots(&stop)?.is_empty());
    drop(store);

    let mut legacy = ProjectStore::create(&legacy_root, "fixture.retention", catalog.clone())?;
    let legacy_epoch = legacy.epoch().clone();
    assert_eq!(
        legacy
            .retention_roots(&stop)
            .err()
            .ok_or("v1 root table assumed")?
            .code(),
        StoreErrorCode::ConfigurationInvalid
    );
    assert_eq!(
        legacy
            .put_retention_root(&pin, &stop)
            .err()
            .ok_or("v1 root write accepted")?
            .code(),
        StoreErrorCode::ConfigurationInvalid
    );
    assert_eq!(legacy.epoch(), &legacy_epoch);
    drop(legacy);
    let legacy = ProjectStore::open(&legacy_root, &catalog)?;
    assert_eq!(legacy.epoch(), &legacy_epoch);
    assert!(legacy.current()?.is_none());
    drop(legacy);
    std::fs::remove_dir_all(store_root)?;
    std::fs::remove_dir_all(legacy_root)?;
    Ok(())
}
