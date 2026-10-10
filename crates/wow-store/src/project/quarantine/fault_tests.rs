//! Native Windows refusal followed by explicit exact-intent reconciliation.
use super::*;
use std::{error::Error, fs::OpenOptions, os::windows::fs::OpenOptionsExt};

#[test]
fn refused_selector_rename_preserves_normal_current_and_reconciles_exact_staged_intent()
-> Result<(), Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!(
        "wow-quarantine-sharing-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let store = ProjectStore::create(&root, "fixture.quarantine.windows", catalog.clone())?;
    let stop = AtomicBool::new(false);
    let expected = store.registry_selection()?;
    let inspection = store.quarantine_inspection(&stop)?;
    assert_eq!(inspection.current(), &CurrentObservation::Absent);
    // Exclude FILE_SHARE_DELETE using a real native handle, not a synthetic fault flag.
    let blocker = OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(root.join(registry::REGISTRY_FILE))?;
    let operation = OperationId::new("fixture:quarantine-sharing")?;
    assert_eq!(
        store
            .quarantine(&operation, &inspection, &stop)
            .err()
            .ok_or("rename was not refused")?
            .code(),
        StoreErrorCode::OutcomeUnknown
    );
    assert_eq!(store.registry_selection()?, expected);
    assert!(store.current()?.is_none());
    let archive = root
        .join("quarantines")
        .join(registry::instance_id(&operation)?);
    let retained = fs::read(archive.join("record.json"))?;
    assert_eq!(fs::read(archive.join("selector.staged"))?, retained);
    drop(blocker);
    let receipt = store.quarantine(&operation, &inspection, &stop)?;
    assert_eq!(receipt.previous(), &expected);
    assert_eq!(fs::read(root.join(registry::REGISTRY_FILE))?, retained);
    assert!(!archive.join("selector.staged").exists());
    assert_eq!(store.quarantine(&operation, &inspection, &stop)?, receipt);
    assert!(!archive.join("selector.staged").exists());
    assert_eq!(
        store
            .current()
            .err()
            .ok_or("quarantined source remained normal")?
            .code(),
        StoreErrorCode::Quarantined
    );
    drop(inspection);
    drop(store);
    let reopened = QuarantinedStore::open(&root, &catalog, &stop)?;
    assert_eq!(reopened.receipt(), &receipt);
    assert_eq!(
        reopened.current_observation(&stop)?,
        CurrentObservation::Absent
    );
    drop(reopened);
    fs::remove_dir_all(root)?;
    Ok(())
}
