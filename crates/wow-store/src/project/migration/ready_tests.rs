//! Native READY preparation preserves the baseline and admits only its exact plan.
use super::*;
use crate::project::migration::{MigrationPreparation, ValidatedMigration, ready::PreparationBody};
use std::path::Path;

fn ready_baseline(
    root: &Path,
    store: &mut ProjectStore,
    stop: &AtomicBool,
) -> TestResult<(PublicationRequest, PublicationRequest, ValidatedMigration)> {
    let first = request(store, "fixture:ready-first", "fixture:ready-first", 91)?;
    activate(store, &first, 91, stop)?;
    let second = request(store, "fixture:ready-second", "fixture:ready-second", 92)?;
    activate(store, &second, 92, stop)?;
    for (id, kind, generation) in [
        (
            "fixture:ready-rollback",
            RetentionRootKind::Rollback,
            &first.generation().generation_id,
        ),
        (
            "fixture:ready-evidence",
            RetentionRootKind::Evidence,
            &second.generation().generation_id,
        ),
    ] {
        let pin = RetentionRoot::new(
            store.epoch().epoch_id().clone(),
            RetentionRootId::new(id)?,
            kind,
            generation.clone(),
            "fixture:ready-policy",
        )?;
        store.put_retention_root(&pin, stop)?;
    }
    let backup = store.backup_to_new(
        root.join("source-backup"),
        &OperationId::new("fixture:ready-source-backup")?,
        stop,
    )?;
    let candidate = at(
        "stage READY baseline",
        MigrationCandidate::stage(
            &backup,
            &root.join("baseline"),
            &OperationId::new("fixture:ready-migration")?,
            stop,
        ),
    )?;
    let checks = target_checks(&candidate, &[(&first, 91), (&second, 92)], stop)?;
    assert_eq!(checks.len(), 2);
    let baseline = at("finish READY baseline", candidate.finish(checks, stop))?;
    Ok((first, second, baseline))
}

fn ready_checks(
    preparation: &MigrationPreparation,
    baseline: &ValidatedMigration,
    expected: &[(&PublicationRequest, u32)],
    stop: &AtomicBool,
) -> TestResult<Vec<ValidatedRead>> {
    assert_eq!(
        preparation.target_generations(),
        baseline.target_generations()
    );
    assert_eq!(baseline.mappings().len(), expected.len());
    let mut checks = BTreeMap::new();
    for (source, value) in expected {
        let mapping = baseline
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == &source.generation().generation_id)
            .ok_or("missing READY source mapping")?;
        let read = preparation.read_generation(mapping.target_generation(), stop)?;
        let original = baseline.read_generation(mapping.target_generation(), stop)?;
        assert_eq!(read.manifest(), original.manifest());
        assert_eq!(read.manifest().members, source.generation().members);
        assert_eq!(read.manifest().bindings, source.generation().bindings);
        assert_eq!(
            read.manifest().epoch_id,
            *baseline.target_epoch().epoch_id()
        );
        assert!(read.manifest().expected_current.is_none());
        assert!(
            checks
                .insert(
                    mapping.target_generation().clone(),
                    check(&read, *value, stop)?
                )
                .is_none()
        );
    }
    assert_eq!(checks.len(), preparation.target_generations().len());
    Ok(checks.into_values().collect())
}

fn ready_inventory(path: &Path) -> TestResult<SqlInventory> {
    let connection = crate::project::database::connect(path, true)?;
    connection.execute_batch("BEGIN DEFERRED")?;
    sql_inventory(&connection)
}

#[test]
fn ready_maps_pins_and_current_then_retries_without_changing_any_completed_effect() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut source, catalog) = fixture("ready-lifecycle", true)?;
    let (first, second, baseline) = ready_baseline(&root, &mut source, &stop)?;
    let source_current = source.current()?.ok_or("missing READY source Current")?;
    let source_roots = source.retention_roots(&stop)?;
    let held_first = source.read(
        &ReadSelector::Exact(first.generation().generation_id.clone()),
        &stop,
    )?;
    let held_current = source.read(&ReadSelector::Current, &stop)?;
    let baseline_inventory = sql_inventory(&baseline.candidate.store.db.connection)?;
    let baseline_receipt = baseline.receipt().clone();
    let baseline_record = fs::read(root.join("baseline/migration-record.json"))?;
    let working_root = root.join("preparation");
    let operation = OperationId::new("fixture:ready-prepare")?;
    let export = baseline.export_target(
        &working_root,
        &OperationId::new("fixture:ready-export")?,
        &stop,
    )?;
    let working_path = export.store.db.path.clone();
    let preparation = at(
        "stage exact READY request",
        MigrationPreparation::stage(&baseline, export, &operation, &stop),
    )?;
    let request_digest = preparation.request_digest()?;
    let initial_inventory = ready_inventory(&working_path)?;
    rejected(
        preparation.finish(&baseline, Vec::new(), &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(ready_inventory(&working_path)?, initial_inventory);
    assert!(!working_root.join("project-store-registry.json").exists());
    assert!(!working_root.join("ready-artifact").exists());

    let mut preparation = at(
        "reopen preparation after omitted owner checks",
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop),
    )?;
    let checks = ready_checks(
        &preparation,
        &baseline,
        &[(&first, 91), (&second, 92)],
        &stop,
    )?;
    preparation.body = match preparation.body {
        PreparationBody::Export(export) => PreparationBody::Working(Box::new(at(
            "owner-checked handoff before partial mapped pin",
            (*export).finish_restore(checks, &stop),
        )?)),
        PreparationBody::Working(_) => return Err("expected fresh READY export".into()),
    };
    let source_root = source_roots.first().ok_or("missing first source pin")?;
    let mapping = baseline
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == source_root.generation_id())
        .ok_or("missing partial pin generation mapping")?;
    let partial_root = RetentionRoot::new(
        baseline.target_epoch().epoch_id().clone(),
        source_root.root_id().clone(),
        source_root.kind(),
        mapping.target_generation().clone(),
        source_root.held_by(),
    )?;
    let partial_inventory = match &mut preparation.body {
        PreparationBody::Working(store) => {
            assert!(store.retention_roots(&stop)?.is_empty());
            assert_eq!(
                store.put_retention_root(&partial_root, &stop)?,
                partial_root
            );
            assert_eq!(store.retention_roots(&stop)?, vec![partial_root.clone()]);
            assert!(store.current()?.is_none());
            sql_inventory(&store.db.connection)?
        }
        PreparationBody::Export(_) => return Err("partial pin requires Working body".into()),
    };
    assert_eq!(partial_inventory["retention_roots"].len(), 1);
    assert!(partial_inventory["publication_history"].is_empty());
    let partial_registry = fs::read(working_root.join("project-store-registry.json"))?;
    drop(preparation);
    let preparation = at(
        "resume exact READY request after one mapped pin",
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop),
    )?;
    assert_eq!(preparation.request_digest()?, request_digest);
    match &preparation.body {
        PreparationBody::Working(store) => {
            assert_eq!(store.retention_roots(&stop)?, vec![partial_root]);
            assert!(store.current()?.is_none());
            assert_eq!(sql_inventory(&store.db.connection)?, partial_inventory);
        }
        PreparationBody::Export(_) => return Err("reopen lost the partial Working body".into()),
    }
    assert_eq!(
        fs::read(working_root.join("project-store-registry.json"))?,
        partial_registry
    );
    let checks = ready_checks(
        &preparation,
        &baseline,
        &[(&first, 91), (&second, 92)],
        &stop,
    )?;
    let ready = at(
        "finish genuine READY preparation",
        preparation.finish(&baseline, checks, &stop),
    )?;
    let receipt = ready.receipt().clone();
    assert_eq!(receipt.request_digest(), request_digest);
    assert_eq!(receipt.source_epoch(), source.epoch());
    assert_eq!(receipt.target_epoch(), baseline.target_epoch());
    assert_eq!(
        receipt.baseline_snapshot_digest(),
        baseline_receipt.target_snapshot_digest()
    );
    assert_eq!(receipt.mapped_roots().len(), 2);
    assert_eq!(
        receipt
            .mapped_roots()
            .iter()
            .map(|mapped| mapped.source().clone())
            .collect::<Vec<_>>(),
        source_roots
    );
    let mut mapped_roots = Vec::new();
    for mapped in receipt.mapped_roots() {
        let mapping = baseline
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == mapped.source().generation_id())
            .ok_or("missing READY pin generation mapping")?;
        let expected = RetentionRoot::new(
            baseline.target_epoch().epoch_id().clone(),
            mapped.source().root_id().clone(),
            mapped.source().kind(),
            mapping.target_generation().clone(),
            mapped.source().held_by(),
        )?;
        assert_eq!(mapped.target(), &expected);
        assert_ne!(mapped.target().pin_digest(), mapped.source().pin_digest());
        mapped_roots.push(expected);
    }
    let current = receipt
        .target_current()
        .ok_or("missing READY target Current")?;
    let current_mapping = baseline_receipt
        .current_mapping()
        .ok_or("missing baseline Current mapping")?;
    assert_eq!(current_mapping.source(), &source_current);
    assert_eq!(&current.generation_id, current_mapping.target_generation());
    assert_eq!(&current.validation_id, current_mapping.target_validation());
    assert_eq!(current.epoch_id, *baseline.target_epoch().epoch_id());
    assert!(current.predecessor.is_none());

    let artifact_root = working_root.join("ready-artifact");
    assert_eq!(
        fs::canonicalize(&ready.artifact().root)?,
        fs::canonicalize(&artifact_root)?
    );
    assert_eq!(ready.artifact().manifest().epoch(), baseline.target_epoch());
    assert_eq!(
        ready.artifact().manifest().generations(),
        baseline.target_generations()
    );
    assert_eq!(ready.artifact().manifest().current(), Some(current));
    assert_eq!(ready.artifact().store.current()?.as_ref(), Some(current));
    assert_eq!(ready.artifact().store.retention_roots(&stop)?, mapped_roots);
    assert_eq!(
        receipt.artifact_snapshot_digest(),
        ready.artifact().manifest().snapshot_digest()
    );
    assert_ne!(
        receipt.artifact_snapshot_digest(),
        receipt.baseline_snapshot_digest()
    );
    for (request, value) in [(&first, 91), (&second, 92)] {
        let mapping = baseline
            .mappings()
            .iter()
            .find(|mapping| mapping.source_generation() == &request.generation().generation_id)
            .ok_or("missing final READY mapping")?;
        let read = ready.artifact().read(
            &ReadSelector::Exact(mapping.target_generation().clone()),
            &stop,
        )?;
        let original = baseline.read_generation(mapping.target_generation(), &stop)?;
        assert_eq!(read.manifest(), original.manifest());
        assert_eq!(read.current_at_acquisition(), Some(current));
        check(&read, value, &stop)?;
    }
    ready.artifact().verify(&stop)?;
    let working_inventory = ready_inventory(&working_path)?;
    assert_eq!(working_inventory["publication_history"].len(), 1);
    assert_eq!(working_inventory["operations"].len(), 2);
    assert_eq!(working_inventory["retention_roots"].len(), 2);
    let artifact_inventory = sql_inventory(&ready.artifact().store.db.connection)?;
    assert_eq!(artifact_inventory, working_inventory);
    let artifact_manifest = ready.artifact().manifest().clone();
    let artifact_path = ready.artifact().store.db.path.clone();
    let artifact_bytes = fs::read(&artifact_path)?;
    let artifact_modified = fs::metadata(&artifact_path)?.modified()?;
    let manifest_bytes = fs::read(artifact_root.join("backup-manifest.json"))?;
    let record_bytes = fs::read(artifact_root.join("migration-ready-record.json"))?;
    assert_eq!(record_bytes, receipt.canonical_bytes()?);
    drop(ready);

    fs::write(
        artifact_root.join("migration-ready-record.json"),
        b"foreign ready record",
    )?;
    let preparation =
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop)?;
    let checks = ready_checks(
        &preparation,
        &baseline,
        &[(&first, 91), (&second, 92)],
        &stop,
    )?;
    rejected(
        preparation.finish(&baseline, checks, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    assert_eq!(ready_inventory(&working_path)?, working_inventory);
    assert_eq!(fs::read(&artifact_path)?, artifact_bytes);
    assert_eq!(
        fs::read(artifact_root.join("migration-ready-record.json"))?,
        b"foreign ready record"
    );
    fs::write(
        artifact_root.join("migration-ready-record.json"),
        &record_bytes,
    )?;

    let preparation = at(
        "reopen completed READY request",
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop),
    )?;
    let checks = ready_checks(
        &preparation,
        &baseline,
        &[(&first, 91), (&second, 92)],
        &stop,
    )?;
    let retried = at(
        "retry completed READY with fresh owners",
        preparation.finish(&baseline, checks, &stop),
    )?;
    assert_eq!(retried.receipt(), &receipt);
    assert_eq!(retried.artifact().manifest(), &artifact_manifest);
    assert_eq!(ready_inventory(&working_path)?, working_inventory);
    assert_eq!(
        sql_inventory(&retried.artifact().store.db.connection)?,
        artifact_inventory
    );
    assert_eq!(fs::read(&artifact_path)?, artifact_bytes);
    assert_eq!(fs::metadata(&artifact_path)?.modified()?, artifact_modified);
    assert_eq!(
        fs::read(artifact_root.join("backup-manifest.json"))?,
        manifest_bytes
    );
    assert_eq!(
        fs::read(artifact_root.join("migration-ready-record.json"))?,
        record_bytes
    );
    drop(retried);
    let artifact = at(
        "independently reopen final READY backup",
        VerifiedBackup::open(
            &artifact_root,
            &catalog,
            artifact_manifest.operation_id(),
            artifact_manifest.snapshot_digest(),
            &stop,
        ),
    )?;
    assert_eq!(artifact.manifest(), &artifact_manifest);
    assert!(baseline.candidate.store.current()?.is_none());
    assert!(baseline.candidate.store.retention_roots(&stop)?.is_empty());
    assert_eq!(
        sql_inventory(&baseline.candidate.store.db.connection)?,
        baseline_inventory
    );
    assert_eq!(baseline.receipt(), &baseline_receipt);
    assert_eq!(
        fs::read(root.join("baseline/migration-record.json"))?,
        baseline_record
    );
    assert_eq!(source.current()?.as_ref(), Some(&source_current));
    assert_eq!(source.retention_roots(&stop)?, source_roots);
    assert_eq!(
        baseline.source().store.retention_roots(&stop)?,
        source_roots
    );
    baseline.source().verify(&stop)?;
    assert_eq!(held_first.manifest(), first.generation());
    assert_eq!(held_current.manifest(), second.generation());
    assert_eq!(held_current.current_at_acquisition(), Some(&source_current));
    check(&held_first, 91, &stop)?;
    check(&held_current, 92, &stop)?;
    drop(artifact);
    drop(baseline);
    drop(held_current);
    drop(held_first);
    drop(source);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn ready_preflight_preserves_empty_output_and_working_missing_inner_manifest() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut source, _) = fixture("ready-interrupted-preflight", true)?;
    let publication = request(
        &source,
        "fixture:ready-interrupted-source",
        "fixture:ready-interrupted",
        93,
    )?;
    let source_current = activate(&mut source, &publication, 93, &stop)?;
    let held = source.read(&ReadSelector::Current, &stop)?;
    // No pins: a complete pin inventory must not mask the empty-output handoff defect.
    assert!(source.retention_roots(&stop)?.is_empty());
    let backup = source.backup_to_new(
        root.join("source-backup"),
        &OperationId::new("fixture:ready-interrupted-backup")?,
        &stop,
    )?;
    let candidate = MigrationCandidate::stage(
        &backup,
        &root.join("baseline"),
        &OperationId::new("fixture:ready-interrupted-migration")?,
        &stop,
    )?;
    let checks = target_checks(&candidate, &[(&publication, 93)], &stop)?;
    let baseline = candidate.finish(checks, &stop)?;
    let baseline_inventory = sql_inventory(&baseline.candidate.store.db.connection)?;
    let working_root = root.join("preparation");
    let operation = OperationId::new("fixture:ready-interrupted-prepare")?;
    let export = baseline.export_target(
        &working_root,
        &OperationId::new("fixture:ready-interrupted-export")?,
        &stop,
    )?;
    let working_path = export.store.db.path.clone();
    let preparation = MigrationPreparation::stage(&baseline, export, &operation, &stop)?;
    let request_digest = preparation.request_digest()?;
    let before_export = ready_inventory(&working_path)?;
    let intent_bytes = fs::read(working_root.join("migration-ready-intent.json"))?;
    let copy_manifest_bytes = fs::read(working_root.join("backup-manifest.json"))?;
    let output = working_root.join("ready-artifact");
    fs::create_dir(&output)?;
    let checks = ready_checks(&preparation, &baseline, &[(&publication, 93)], &stop)?;
    rejected(
        preparation.finish(&baseline, checks, &stop),
        StoreErrorCode::OutcomeUnknown,
    )?;
    assert_eq!(ready_inventory(&working_path)?, before_export);
    assert!(
        !working_root.join("project-store-registry.json").exists(),
        "rejection performed the private handoff"
    );
    assert!(output.is_dir());
    assert_eq!(
        fs::read_dir(&output)?.count(),
        0,
        "rejection changed the retained empty output"
    );
    assert_eq!(
        fs::read(working_root.join("migration-ready-intent.json"))?,
        intent_bytes
    );
    assert_eq!(
        fs::read(working_root.join("backup-manifest.json"))?,
        copy_manifest_bytes
    );

    // Remove only this known empty fixture directory, then model an interrupted handoff.
    fs::remove_dir(&output)?;
    let mut preparation =
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop)?;
    let checks = ready_checks(&preparation, &baseline, &[(&publication, 93)], &stop)?;
    preparation.body = match preparation.body {
        PreparationBody::Export(export) => PreparationBody::Working(Box::new(at(
            "owner-checked interrupted handoff",
            (*export).finish_restore(checks, &stop),
        )?)),
        PreparationBody::Working(_) => return Err("expected untouched interrupted export".into()),
    };
    let checks = ready_checks(&preparation, &baseline, &[(&publication, 93)], &stop)?;
    let before_working = ready_inventory(&working_path)?;
    assert!(before_working["current_publication"].is_empty());
    assert!(before_working["publication_history"].is_empty());
    let registry_bytes = fs::read(working_root.join("project-store-registry.json"))?;
    let inner_manifest =
        crate::project::database::epoch_directory(&working_root, baseline.target_epoch())?
            .join("epoch-manifest.json");
    let inner_bytes = fs::read(&inner_manifest)?;
    fs::remove_file(&inner_manifest)?;
    rejected(
        preparation.finish(&baseline, checks, &stop),
        StoreErrorCode::DatabaseUnavailable,
    )?;
    assert!(
        !inner_manifest.exists(),
        "rejection recreated the missing inner manifest"
    );
    assert!(!output.exists());
    assert_eq!(
        ready_inventory(&working_path)?,
        before_working,
        "rejection activated Current or changed working SQL"
    );
    assert_eq!(
        fs::read(working_root.join("project-store-registry.json"))?,
        registry_bytes
    );
    assert_eq!(
        fs::read(working_root.join("migration-ready-intent.json"))?,
        intent_bytes
    );
    assert_eq!(
        fs::read(working_root.join("backup-manifest.json"))?,
        copy_manifest_bytes
    );
    fs::write(&inner_manifest, &inner_bytes)?;
    assert_eq!(
        sql_inventory(&baseline.candidate.store.db.connection)?,
        baseline_inventory
    );
    assert_eq!(source.current()?.as_ref(), Some(&source_current));
    assert!(source.retention_roots(&stop)?.is_empty());
    assert_eq!(held.manifest(), publication.generation());
    assert_eq!(held.current_at_acquisition(), Some(&source_current));
    check(&held, 93, &stop)?;
    backup.verify(&stop)?;
    baseline.source().verify(&stop)?;
    drop(baseline);
    drop(backup);
    drop(held);
    drop(source);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn ready_reopen_rejects_wrong_request_and_foreign_pin_before_any_resume_write() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut source, _) = fixture("ready-foreign-pin", true)?;
    let (first, second, baseline) = ready_baseline(&root, &mut source, &stop)?;
    let baseline_inventory = sql_inventory(&baseline.candidate.store.db.connection)?;
    let source_current = source.current()?;
    let source_roots = source.retention_roots(&stop)?;
    let working_root = root.join("preparation");
    let operation = OperationId::new("fixture:ready-foreign-prepare")?;
    let export = baseline.export_target(
        &working_root,
        &OperationId::new("fixture:ready-foreign-export")?,
        &stop,
    )?;
    let working_path = export.store.db.path.clone();
    let preparation = MigrationPreparation::stage(&baseline, export, &operation, &stop)?;
    let request_digest = preparation.request_digest()?;
    let initial_inventory = ready_inventory(&working_path)?;
    let intent_bytes = fs::read(working_root.join("migration-ready-intent.json"))?;
    drop(preparation);
    let wrong_digest =
        crate::project::model::digest("project-migration-ready-request", b"wrong READY request");
    assert_ne!(wrong_digest, request_digest);
    rejected(
        MigrationPreparation::open(&baseline, &working_root, &operation, &wrong_digest, &stop),
        StoreErrorCode::OperationConflict,
    )?;
    assert_eq!(ready_inventory(&working_path)?, initial_inventory);
    assert_eq!(
        fs::read(working_root.join("migration-ready-intent.json"))?,
        intent_bytes
    );
    assert!(!working_root.join("project-store-registry.json").exists());

    let mut preparation =
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop)?;
    let checks = ready_checks(
        &preparation,
        &baseline,
        &[(&first, 91), (&second, 92)],
        &stop,
    )?;
    preparation.body = match preparation.body {
        PreparationBody::Export(export) => PreparationBody::Working(Box::new(at(
            "owner-checked handoff before foreign pin",
            (*export).finish_restore(checks, &stop),
        )?)),
        PreparationBody::Working(_) => return Err("expected fresh READY export".into()),
    };
    let foreign = RetentionRoot::new(
        baseline.target_epoch().epoch_id().clone(),
        RetentionRootId::new("fixture:foreign-ready-pin")?,
        RetentionRootKind::User,
        baseline
            .target_generations()
            .first()
            .ok_or("missing READY target")?
            .clone(),
        "fixture:foreign-policy",
    )?;
    let before = match &mut preparation.body {
        PreparationBody::Working(store) => {
            assert!(store.retention_roots(&stop)?.is_empty());
            assert_eq!(store.put_retention_root(&foreign, &stop)?, foreign);
            assert!(store.current()?.is_none());
            sql_inventory(&store.db.connection)?
        }
        PreparationBody::Export(_) => {
            return Err("READY handoff did not create working store".into());
        }
    };
    assert_eq!(before["retention_roots"].len(), 1);
    assert!(before["publication_history"].is_empty());
    let registry_bytes = fs::read(working_root.join("project-store-registry.json"))?;
    drop(preparation);
    rejected(
        MigrationPreparation::open(&baseline, &working_root, &operation, &request_digest, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(
        ready_inventory(&working_path)?,
        before,
        "rejected resume changed native SQL"
    );
    assert_eq!(
        fs::read(working_root.join("migration-ready-intent.json"))?,
        intent_bytes
    );
    assert_eq!(
        fs::read(working_root.join("project-store-registry.json"))?,
        registry_bytes
    );
    assert!(!working_root.join("ready-artifact").exists());
    assert_eq!(
        sql_inventory(&baseline.candidate.store.db.connection)?,
        baseline_inventory
    );
    assert_eq!(source.current()?, source_current);
    assert_eq!(source.retention_roots(&stop)?, source_roots);
    baseline.source().verify(&stop)?;
    drop(baseline);
    drop(source);
    fs::remove_dir_all(root)?;
    Ok(())
}
