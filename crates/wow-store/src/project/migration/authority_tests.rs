//! Native source holds remain portable across migration and target recovery.
use super::*;
use crate::project::{MigrationPreparation, ReadyMigration, SourceAuthorityReference, registry};
use std::path::Path;

struct Fixture {
    root: PathBuf,
    store: ProjectStore,
    catalog: RecordCatalog,
    baseline: super::super::ValidatedMigration,
    ready: ReadyMigration,
}

fn backup_checks(backup: &VerifiedBackup, stop: &AtomicBool) -> TestResult<Vec<ValidatedRead>> {
    assert_eq!(backup.manifest().generations().len(), 1);
    let read = backup.read(
        &ReadSelector::Exact(backup.manifest().generations()[0].clone()),
        stop,
    )?;
    Ok(vec![check(&read, 71, stop)?])
}

fn source_state(store: &ProjectStore, stop: &AtomicBool) -> TestResult<Vec<u8>> {
    let connection = store.db.read_connection()?;
    let state = crate::project::backup::identity::capture(&connection, store.epoch(), stop)?;
    Ok(crate::project::model::encode(&state, 16 * 1024 * 1024)?)
}

#[test]
fn conflicting_native_hold_union_refuses_both_staging_owners_before_files() -> TestResult {
    let stop = AtomicBool::new(false);
    let Fixture {
        root,
        store,
        catalog,
        baseline,
        ready,
    } = fixture_with_hold("authority-preflight", &stop)?;
    let before_root = root.join("before-hold");
    let header: serde_json::Value =
        serde_json::from_slice(&fs::read(before_root.join("backup-manifest.json"))?)?;
    let before = VerifiedBackup::open(
        &before_root,
        &catalog,
        &OperationId::new("fixture:authority-before-hold")?,
        header["snapshot_digest"]
            .as_str()
            .ok_or("missing backup digest")?,
        &stop,
    )?;
    let copy = before.restore_to_new(
        root.join("different-hold"),
        &OperationId::new("fixture:authority-different-copy")?,
        &stop,
    )?;
    let checks = backup_checks(&copy, &stop)?;
    let mut branch = copy.finish_restore(checks, &stop)?;
    let second = request(
        &branch,
        "fixture:authority-different-current",
        "fixture:authority-different",
        72,
    )?;
    activate(&mut branch, &second, 72, &stop)?;
    let inspection = branch.quarantine_inspection(&stop)?;
    // Same original epoch/operation identity, genuinely different native evidence.
    let hold = inspection.quarantine(&OperationId::new("fixture:authority-source-hold")?, &stop)?;
    let held = branch.quarantined(&stop)?;
    drop(inspection);
    drop(branch);
    let candidate = held.stage_restore(
        &before,
        &OperationId::new("fixture:authority-different-restore")?,
        hold.selected(),
        &stop,
    )?;
    let checks = backup_checks(candidate.backup(), &stop)?;
    let (branch, _) = held.activate_restore(candidate, checks, &stop)?;
    drop(held);
    let incoming = branch.backup_to_new(
        root.join("different-backup"),
        &OperationId::new("fixture:authority-different-backup")?,
        &stop,
    )?;
    assert_ne!(
        incoming.manifest().retained_quarantines(),
        store.retained_quarantines()?
    );
    let source_root = root.join("source-restored");
    let registry_path = source_root.join(registry::REGISTRY_FILE);
    let selector = fs::read(&registry_path)?;
    rejected(
        store.stage_replacement(
            &incoming,
            &OperationId::new("fixture:authority-conflict-replacement")?,
            &store.registry_selection()?,
            store.current()?.map(|current| current.record_id),
            &stop,
        ),
        StoreErrorCode::OperationConflict,
    )?;
    assert!(!source_root.join("instances").exists());
    assert_eq!(fs::read(&registry_path)?, selector);
    let inspection = store.quarantine_inspection(&stop)?;
    let hold = inspection.quarantine(&OperationId::new("fixture:authority-next-hold")?, &stop)?;
    let held = store.quarantined(&stop)?;
    let held_selector = fs::read(&registry_path)?;
    rejected(
        held.stage_restore(
            &incoming,
            &OperationId::new("fixture:authority-conflict-restore")?,
            hold.selected(),
            &stop,
        ),
        StoreErrorCode::OperationConflict,
    )?;
    assert!(!source_root.join("instances").exists());
    assert_eq!(fs::read(&registry_path)?, held_selector);
    drop(held);
    drop(inspection);
    drop(store);
    drop(incoming);
    drop(branch);
    drop(before);
    drop(ready);
    drop(baseline);
    fs::remove_dir_all(root)?;
    Ok(())
}

fn fixture_with_hold(name: &str, stop: &AtomicBool) -> TestResult<Fixture> {
    let (root, mut store, catalog) = fixture(name, true)?;
    let publication = request(&store, "fixture:authority-one", "fixture:authority", 71)?;
    activate(&mut store, &publication, 71, stop)?;
    store.put_retention_root(
        &RetentionRoot::new(
            store.epoch().epoch_id().clone(),
            RetentionRootId::new("fixture:authority-pin")?,
            RetentionRootKind::Evidence,
            publication.generation().generation_id.clone(),
            "fixture:authority-policy",
        )?,
        stop,
    )?;
    let backup = store.backup_to_new(
        root.join("before-hold"),
        &OperationId::new("fixture:authority-before-hold")?,
        stop,
    )?;
    let inspection = store.quarantine_inspection(stop)?;
    let hold = inspection.quarantine(&OperationId::new("fixture:authority-source-hold")?, stop)?;
    let held = store.quarantined(stop)?;
    drop(inspection);
    drop(store);
    let candidate = held.stage_restore(
        &backup,
        &OperationId::new("fixture:authority-source-restore")?,
        hold.selected(),
        stop,
    )?;
    let checks = backup_checks(candidate.backup(), stop)?;
    let (restored, _) = held.activate_restore(candidate, checks, stop)?;
    drop(held);
    drop(backup);
    // A standalone restored /5 selector retains a genuine v2 hold without
    // depending on a replacement instance or its original root.
    let transport = restored.backup_to_new(
        root.join("source-restored"),
        &OperationId::new("fixture:authority-source-transport")?,
        stop,
    )?;
    let checks = backup_checks(&transport, stop)?;
    let store = transport.finish_restore(checks, stop)?;
    drop(restored);
    assert_eq!(store.epoch().physical_profile(), RETAINED_PHYSICAL_PROFILE);
    assert_eq!(store.retained_quarantines()?.len(), 1);
    let selector: serde_json::Value = serde_json::from_slice(&fs::read(
        root.join("source-restored").join(registry::REGISTRY_FILE),
    )?)?;
    assert_eq!(selector["schema"], "wow-store/project-registry/5");
    let source = store.backup_to_new(
        root.join("source-backup"),
        &OperationId::new("fixture:authority-source-backup")?,
        stop,
    )?;
    let candidate = MigrationCandidate::stage(
        &source,
        &root.join("baseline"),
        &OperationId::new("fixture:authority-migration")?,
        stop,
    )?;
    let checks = target_checks(&candidate, &[(&publication, 71)], stop)?;
    let baseline = candidate.finish(checks, stop)?;
    drop(source);
    let preparation = MigrationPreparation::create(
        &baseline,
        &root.join("preparation"),
        &OperationId::new("fixture:authority-ready")?,
        stop,
    )?;
    let mut checks = Vec::new();
    for generation in preparation.target_generations() {
        checks.push(check(
            &preparation.read_generation(&generation, stop)?,
            71,
            stop,
        )?);
    }
    let ready = preparation.finish(&baseline, checks, stop)?;
    Ok(Fixture {
        root,
        store,
        catalog,
        baseline,
        ready,
    })
}

fn authority_directory(root: &Path, reference: &SourceAuthorityReference) -> TestResult<PathBuf> {
    let hash = reference
        .manifest_digest()
        .strip_prefix("project-source-authority:sha256:")
        .ok_or("unexpected authority digest")?;
    Ok(root.join("source-authorities").join(hash))
}

fn hold_bytes(root: &Path, operation: &OperationId) -> TestResult<[Vec<u8>; 3]> {
    let directory = root
        .join("quarantines")
        .join(registry::instance_id(operation)?);
    Ok([
        fs::read(directory.join("selection.json"))?,
        fs::read(directory.join("evidence.json"))?,
        fs::read(directory.join("record.json"))?,
    ])
}

#[test]
fn original_hold_survives_independent_reopen_restore_replacement_and_target_hold() -> TestResult {
    let stop = AtomicBool::new(false);
    let Fixture {
        root,
        store,
        catalog,
        baseline,
        ready,
    } = fixture_with_hold("authority-portable", &stop)?;
    let source_root = root.join("source-restored");
    let selection = store.registry_selection()?;
    let current = store.current()?.ok_or("missing source Current")?;
    let epoch = store.epoch().clone();
    let source_selector = fs::read(source_root.join(registry::REGISTRY_FILE))?;
    let source_holds = store.retained_quarantines()?;
    let source_archive = hold_bytes(&source_root, source_holds[0].operation_id())?;
    let portable_root = root.join("portable");
    let operation = OperationId::new("fixture:authority-portable")?;
    let portable = store.export_ready_migration_to_new(
        &baseline,
        &ready,
        &portable_root,
        &operation,
        &selection,
        Some(&current.record_id),
        &stop,
    )?;
    let manifest = portable.manifest().clone();
    assert_eq!(manifest.source_authorities().len(), 1);
    assert!(manifest.retained_quarantines().is_empty());
    assert_ne!(manifest.epoch(), &epoch);
    assert_eq!(manifest.epoch(), ready.artifact().manifest().epoch());
    let directory = authority_directory(&portable_root, &manifest.source_authorities()[0])?;
    assert_eq!(fs::read(directory.join("selection.json"))?, source_selector);
    assert_eq!(
        hold_bytes(&directory, source_holds[0].operation_id())?,
        source_archive
    );
    let authority: serde_json::Value =
        serde_json::from_slice(&fs::read(directory.join("authority.json"))?)?;
    assert_eq!(authority["epoch"], serde_json::to_value(&epoch)?);
    assert_eq!(authority["selection"], serde_json::to_value(&selection)?);
    assert_eq!(
        authority["source_snapshot"],
        baseline.source().manifest().snapshot_digest()
    );
    assert_eq!(
        fs::read(source_root.join(registry::REGISTRY_FILE))?,
        source_selector
    );
    drop(portable);
    drop(ready);
    drop(baseline);
    drop(store);
    for name in [
        "source",
        "source-restored",
        "source-backup",
        "before-hold",
        "baseline",
        "preparation",
    ] {
        fs::rename(root.join(name), root.join(format!("unavailable-{name}")))?;
    }
    let portable = VerifiedBackup::open(
        &portable_root,
        &catalog,
        &operation,
        manifest.snapshot_digest(),
        &stop,
    )?;
    assert_eq!(portable.manifest(), &manifest);
    portable.verify(&stop)?;
    let target_root = root.join("target");
    let copy = portable.restore_to_new(
        &target_root,
        &OperationId::new("fixture:authority-target")?,
        &stop,
    )?;
    let checks = backup_checks(&copy, &stop)?;
    let mut target = copy.finish_restore(checks, &stop)?;
    assert_eq!(
        target.retained_source_authorities()?,
        manifest.source_authorities()
    );
    assert!(target.retained_quarantines()?.is_empty());
    assert_eq!(target.current()?.as_ref(), manifest.current());
    let selected: serde_json::Value =
        serde_json::from_slice(&fs::read(target_root.join(registry::REGISTRY_FILE))?)?;
    assert_eq!(selected["schema"], "wow-store/project-registry/7");
    let candidate = target.stage_replacement(
        &portable,
        &OperationId::new("fixture:authority-replace")?,
        &target.registry_selection()?,
        target.current()?.map(|value| value.record_id),
        &stop,
    )?;
    let checks = backup_checks(candidate.backup(), &stop)?;
    target.activate_replacement(candidate, checks, &stop)?;
    assert_eq!(
        target.retained_source_authorities()?,
        manifest.source_authorities()
    );
    let selected: serde_json::Value =
        serde_json::from_slice(&fs::read(target_root.join(registry::REGISTRY_FILE))?)?;
    assert_eq!(selected["schema"], "wow-store/project-registry/8");
    let inspection = target.quarantine_inspection(&stop)?;
    let target_hold =
        inspection.quarantine(&OperationId::new("fixture:authority-target-hold")?, &stop)?;
    let held = target.quarantined(&stop)?;
    drop(inspection);
    drop(target);
    let candidate = held.stage_restore(
        &portable,
        &OperationId::new("fixture:authority-target-restore")?,
        target_hold.selected(),
        &stop,
    )?;
    let checks = backup_checks(candidate.backup(), &stop)?;
    let (target, _) = held.activate_restore(candidate, checks, &stop)?;
    drop(held);
    assert_eq!(
        target.retained_source_authorities()?,
        manifest.source_authorities()
    );
    let target_holds = target.retained_quarantines()?;
    assert_eq!(target_holds.len(), 1);
    assert_eq!(
        target_holds[0].operation_id(),
        &OperationId::new("fixture:authority-target-hold")?
    );
    let recopied = target.backup_to_new(
        root.join("recopied"),
        &OperationId::new("fixture:authority-recopy")?,
        &stop,
    )?;
    assert_eq!(
        recopied.manifest().source_authorities(),
        manifest.source_authorities()
    );
    assert_eq!(recopied.manifest().retained_quarantines(), target_holds);
    let directory = authority_directory(&root.join("recopied"), &manifest.source_authorities()[0])?;
    assert_eq!(
        hold_bytes(&directory, source_holds[0].operation_id())?,
        source_archive
    );
    recopied.verify(&stop)?;
    drop(recopied);
    drop(target);
    drop(portable);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn stale_source_and_substituted_authority_refuse_without_destination_or_selector_effects()
-> TestResult {
    let stop = AtomicBool::new(false);
    let Fixture {
        root,
        mut store,
        catalog,
        baseline,
        ready,
    } = fixture_with_hold("authority-refusal", &stop)?;
    let selection = store.registry_selection()?;
    let current = store.current()?.ok_or("missing source Current")?;
    let before = source_state(&store, &stop)?;
    let source_selector = fs::read(root.join("source-restored").join(registry::REGISTRY_FILE))?;
    let refused = root.join("refused");
    let operation = OperationId::new("fixture:authority-refused")?;
    rejected(
        store.export_ready_migration_to_new(
            &baseline, &ready, &refused, &operation, &selection, None, &stop,
        ),
        StoreErrorCode::CurrentConflict,
    )?;
    assert!(!refused.exists());
    assert_eq!(source_state(&store, &stop)?, before);
    let portable_root = root.join("portable");
    let portable_operation = OperationId::new("fixture:authority-refusal-portable")?;
    let portable = store.export_ready_migration_to_new(
        &baseline,
        &ready,
        &portable_root,
        &portable_operation,
        &selection,
        Some(&current.record_id),
        &stop,
    )?;
    let manifest = portable.manifest().clone();
    let copy_root = root.join("restored");
    let copy = portable.restore_to_new(
        &copy_root,
        &OperationId::new("fixture:authority-refusal-restore")?,
        &stop,
    )?;
    let checks = backup_checks(&copy, &stop)?;
    let restored = copy.finish_restore(checks, &stop)?;
    let restored_epoch = restored.epoch().clone();
    drop(restored);
    // A canonical bare epoch cannot omit the source authority retained by the marker.
    let registry_path = copy_root.join(registry::REGISTRY_FILE);
    let original = fs::read(&registry_path)?;
    let bare_epoch = crate::project::model::encode(&restored_epoch, 65536)?;
    fs::write(&registry_path, &bare_epoch)?;
    rejected(
        ProjectStore::open(&copy_root, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(fs::read(&registry_path)?, bare_epoch);
    fs::write(&registry_path, &original)?;
    drop(ProjectStore::open(&copy_root, &catalog)?);
    drop(portable);
    let directory = authority_directory(&portable_root, &manifest.source_authorities()[0])?;
    let hold = store.retained_quarantines()?.remove(0);
    for path in [
        directory.join("authority.json"),
        directory
            .join("quarantines")
            .join(registry::instance_id(hold.operation_id())?)
            .join("evidence.json"),
    ] {
        let original = fs::read(&path)?;
        let mut changed = original.clone();
        changed.push(b' ');
        fs::write(&path, &changed)?;
        rejected(
            VerifiedBackup::open(
                &portable_root,
                &catalog,
                &portable_operation,
                manifest.snapshot_digest(),
                &stop,
            ),
            StoreErrorCode::IntegrityViolation,
        )?;
        assert_eq!(fs::read(&path)?, changed);
        fs::write(&path, original)?;
    }
    drop(VerifiedBackup::open(
        &portable_root,
        &catalog,
        &portable_operation,
        manifest.snapshot_digest(),
        &stop,
    )?);
    store.put_retention_root(
        &RetentionRoot::new(
            store.epoch().epoch_id().clone(),
            RetentionRootId::new("fixture:authority-late-pin")?,
            RetentionRootKind::Evidence,
            current.generation_id.clone(),
            "fixture:authority-policy",
        )?,
        &stop,
    )?;
    let after_pin = source_state(&store, &stop)?;
    assert_ne!(after_pin, before);
    rejected(
        store.export_ready_migration_to_new(
            &baseline,
            &ready,
            &refused,
            &operation,
            &selection,
            Some(&current.record_id),
            &stop,
        ),
        StoreErrorCode::CurrentConflict,
    )?;
    assert!(!refused.exists());
    assert_eq!(source_state(&store, &stop)?, after_pin);
    assert_eq!(
        fs::read(root.join("source-restored").join(registry::REGISTRY_FILE))?,
        source_selector
    );
    drop(ready);
    drop(baseline);
    drop(store);
    fs::remove_dir_all(root)?;
    Ok(())
}
