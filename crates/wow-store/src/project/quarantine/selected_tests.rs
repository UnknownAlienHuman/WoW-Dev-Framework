//! Physical selector scope is independent of an absent semantic Current.
use super::*;
use std::error::Error;

#[test]
fn empty_replacement_and_quarantine_retain_the_exact_selected_instance()
-> Result<(), Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!(
        "wow-quarantine-selected-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    let source = root.join("source");
    let catalog = RecordCatalog::new(&["fixture.partition.v1"], &["fixture.owner.v1"])?;
    let mut store = ProjectStore::create(&source, "fixture.quarantine.selected", catalog.clone())?;
    let stop = AtomicBool::new(false);
    let original = store.registry_selection()?;
    let epoch = store.epoch().clone();
    let backup = store.backup_to_new(
        root.join("backup"),
        &OperationId::new("fixture:empty-backup")?,
        &stop,
    )?;
    assert!(backup.manifest().current().is_none());
    assert!(backup.manifest().generations().is_empty());
    let replacement = store.stage_replacement(
        &backup,
        &OperationId::new("fixture:empty-replacement")?,
        &original,
        None,
        &stop,
    )?;
    let receipt = store.activate_replacement(replacement, Vec::new(), &stop)?;
    let selected = store.registry_selection()?;
    assert_eq!(selected.revision(), 1);
    assert_eq!(receipt.selected(), &selected);
    assert!(store.current()?.is_none());
    wow_core::canonical_json_bytes(&receipt)?;
    let path = store.db.path.clone();
    assert!(path.starts_with(fs::canonicalize(source.join("instances"))?));
    let inspection = store.quarantine_inspection(&stop)?;
    assert_eq!(inspection.current(), &CurrentObservation::Absent);
    let held = store.quarantine(
        &OperationId::new("fixture:selected-hold")?,
        &inspection,
        &stop,
    )?;
    assert_eq!(held.previous(), &selected);
    assert_eq!(held.selected().revision(), 2);
    assert_eq!(held.selected().epoch(), epoch.epoch_id());
    wow_core::canonical_json_bytes(&held)?;
    assert_eq!(store.quarantined(&stop)?.path, path);
    drop(inspection);
    drop(store);
    let reopened = QuarantinedStore::open(&source, &catalog, &stop)?;
    assert_eq!(reopened.path, path);
    assert_eq!(reopened.receipt(), &held);
    assert_eq!(
        reopened.current_observation(&stop)?,
        CurrentObservation::Absent
    );
    drop(reopened);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}
