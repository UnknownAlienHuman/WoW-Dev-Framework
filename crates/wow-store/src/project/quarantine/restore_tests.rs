//! Native restoration from an exact physical hold, with retained archive authority.
use super::super::{database, model::digest, registry};
use super::tests::*;
use crate::project::{
    CurrentObservation, PointerReadFailure, ProjectStore, PublicationRequest, QuarantineInspection,
    QuarantinedStore, ReadSelector, ValidatedRead, VerifiedBackup,
};
use crate::{OperationId, StoreErrorCode};
use std::{collections::BTreeSet, fs, path::Path, sync::atomic::AtomicBool};

fn owner_checks(
    backup: &VerifiedBackup,
    expected: &[(&PublicationRequest, u32)],
    stop: &AtomicBool,
) -> TestResult<Vec<ValidatedRead>> {
    let expected_ids: BTreeSet<_> = expected
        .iter()
        .map(|(request, _)| request.generation().generation_id.clone())
        .collect();
    assert_eq!(
        backup
            .manifest()
            .generations()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>(),
        expected_ids
    );
    assert_eq!(backup.manifest().generations().len(), expected.len());
    let mut checks = Vec::new();
    for (request, value) in expected {
        let read = backup.read(
            &ReadSelector::Exact(request.generation().generation_id.clone()),
            stop,
        )?;
        assert_eq!(read.manifest(), request.generation());
        checks.push(check_fixture(&read, *value, stop)?);
    }
    Ok(checks)
}

fn archive(root: &Path, operation: &OperationId) -> TestResult<[Vec<u8>; 3]> {
    let directory = root
        .join("quarantines")
        .join(registry::instance_id(operation)?);
    Ok([
        fs::read(directory.join("selection.json"))?,
        fs::read(directory.join("evidence.json"))?,
        fs::read(directory.join("record.json"))?,
    ])
}

fn damage_pointer(store: &ProjectStore) -> TestResult<CurrentObservation> {
    let malformed = b"restore-malformed-current";
    store
        .db
        .connection
        .execute_batch("PRAGMA foreign_keys=OFF")?;
    assert_eq!(
        store.db.connection.execute(
            "UPDATE current_publication SET record_id=?1 WHERE id=1",
            [std::str::from_utf8(malformed)?],
        )?,
        1
    );
    store
        .db
        .connection
        .execute_batch("PRAGMA foreign_keys=ON")?;
    Ok(CurrentObservation::Pointer {
        digest: digest("project-current-observation", malformed),
        byte_length: malformed.len(),
        record_id: None,
    })
}

#[test]
fn pointer_restore_preserves_old_leases_and_hold_authority_through_publication_replacement_and_backup()
-> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = at("create pointer fixture", new_store("restore-pointer"))?;
    let source = root.join("source");
    let epoch = store.epoch().clone();
    let (first, first_current) = at(
        "publish backup base",
        publish(&mut store, "restore-first", 11, &stop),
    )?;
    let backup_root = root.join("before-incident");
    let backup_operation = OperationId::new("fixture:restore-backup")?;
    let backup = at(
        "capture pre-incident backup",
        store.backup_to_new(&backup_root, &backup_operation, &stop),
    )?;
    let backup_digest = backup.manifest().snapshot_digest().to_owned();
    drop(backup);
    let backup = at(
        "independently reopen pre-incident target",
        VerifiedBackup::open(
            &backup_root,
            &catalog,
            &backup_operation,
            &backup_digest,
            &stop,
        ),
    )?;
    let (second, second_current) = at(
        "publish incident base",
        publish(&mut store, "restore-second", 22, &stop),
    )?;
    let leased = store.read(&ReadSelector::Current, &stop)?;
    let old_path = store.db.path.clone();
    let observation = at("damage current pointer", damage_pointer(&store))?;
    let inspection = store.quarantine_inspection(&stop)?;
    assert_eq!(inspection.current(), &observation);
    let hold_operation = OperationId::new("fixture:restore-pointer-hold")?;
    let hold = at(
        "hold pointer damage",
        store.quarantine(&hold_operation, &inspection, &stop),
    )?;
    let original_archive = archive(&source, &hold_operation)?;
    let quarantined = store.quarantined(&stop)?;
    drop(inspection);
    drop(store);

    let restore_operation = OperationId::new("fixture:restore-pointer-select")?;
    let candidate = at(
        "stage pointer restore",
        quarantined.stage_restore(&backup, &restore_operation, hold.selected(), &stop),
    )?;
    let request_digest = candidate.request_digest()?;
    let checks = owner_checks(candidate.backup(), &[(&first, 11)], &stop)?;
    let (mut restored, receipt) = at(
        "activate pointer restore",
        quarantined.activate_restore(candidate, checks, &stop),
    )?;
    assert_eq!(receipt.operation_id(), &restore_operation);
    assert_eq!(receipt.request_digest(), request_digest);
    assert_eq!(receipt.previous(), hold.selected());
    assert_eq!(receipt.selected(), &restored.registry_selection()?);
    assert!(!receipt.selected().is_quarantined());
    assert_eq!(
        receipt.selected().revision(),
        hold.selected().revision() + 1
    );
    assert_eq!(receipt.activated_current(), Some(&first_current));
    assert_eq!(restored.epoch(), &epoch);
    assert_eq!(restored.current()?, Some(first_current.clone()));
    let selected: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join(registry::REGISTRY_FILE))?)?;
    assert_eq!(selected["schema"], "wow-store/project-registry/4");
    assert_ne!(restored.db.path, old_path);
    assert!(old_path.exists());
    assert_eq!(leased.manifest(), second.generation());
    assert_eq!(leased.current_at_acquisition(), Some(&second_current));
    check_fixture(&leased, 22, &stop)?;
    let read = restored.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.manifest(), first.generation());
    check_fixture(&read, 11, &stop)?;
    drop(read);
    let retained = restored.retained_quarantines()?;
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0].operation_id(), &hold_operation);
    assert_eq!(
        retained[0].record_digest(),
        digest("project-registry", &original_archive[2])
    );
    assert_eq!(archive(&source, &hold_operation)?, original_archive);

    let transport_root = root.join("transport");
    let transport_operation = OperationId::new("fixture:restore-transport")?;
    let transport = at(
        "capture retained-hold backup",
        restored.backup_to_new(&transport_root, &transport_operation, &stop),
    )?;
    assert_eq!(
        transport.manifest().retained_quarantines(),
        retained.as_slice()
    );
    assert_eq!(
        serde_json::to_value(transport.manifest())?["schema"],
        "wow-store/project-backup/2"
    );
    assert_eq!(archive(&transport_root, &hold_operation)?, original_archive);
    assert!(!transport_root.join("instances").exists());
    let transport_digest = transport.manifest().snapshot_digest().to_owned();
    let (_, third_current) = publish(&mut restored, "restore-normal-publication", 33, &stop)?;
    assert_eq!(restored.retained_quarantines()?, retained);
    assert_eq!(archive(&source, &hold_operation)?, original_archive);
    let expected_selection = restored.registry_selection()?;
    let replacement_operation = OperationId::new("fixture:restore-normal-replacement")?;
    let replacement = restored.stage_replacement(
        &transport,
        &replacement_operation,
        &expected_selection,
        Some(third_current.record_id),
        &stop,
    )?;
    let checks = owner_checks(replacement.backup(), &[(&first, 11)], &stop)?;
    restored.activate_replacement(replacement, checks, &stop)?;
    assert_eq!(restored.current()?, Some(first_current.clone()));
    assert_eq!(restored.retained_quarantines()?, retained);
    assert_eq!(archive(&source, &hold_operation)?, original_archive);
    check_fixture(&leased, 22, &stop)?;
    let registry_path = source.join(registry::REGISTRY_FILE);
    let normal_bytes = fs::read(&registry_path)?;
    let selected: serde_json::Value = serde_json::from_slice(&normal_bytes)?;
    assert_eq!(selected["schema"], "wow-store/project-registry/4");
    // Matching canonical selector/record bytes cannot discard source hold authority.
    let replacement_record = source
        .join("instances")
        .join(registry::instance_id(&replacement_operation)?)
        .join("replacement-record.json");
    let original_record = fs::read(&replacement_record)?;
    let mut dropped = selected;
    let fields = dropped
        .as_object_mut()
        .ok_or("replacement registry is not an object")?;
    fields.insert("schema".into(), "wow-store/project-registry/2".into());
    assert!(fields.remove("retained_quarantines").is_some());
    let dropped_bytes = wow_core::canonical_json_bytes(&dropped)?;
    fs::write(&registry_path, &dropped_bytes)?;
    fs::write(&replacement_record, &dropped_bytes)?;
    rejected(
        registry::read(&source, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;
    fs::write(&replacement_record, original_record)?;
    fs::write(&registry_path, normal_bytes)?;
    assert_eq!(restored.retained_quarantines()?, retained);
    drop(transport);

    drop(restored);
    drop(quarantined);
    check_fixture(&leased, 22, &stop)?;
    drop(leased);
    let reopened = ProjectStore::open(&source, &catalog)?;
    assert_eq!(reopened.current()?, Some(first_current.clone()));
    assert_eq!(reopened.retained_quarantines()?, retained);
    drop(reopened);
    // All source leases are released before moving this disposable fixture.
    // Artifact admission must work with the original source root unavailable.
    let offline_source = root.join("source-offline");
    fs::rename(&source, &offline_source)?;
    assert!(!source.exists());
    assert_eq!(archive(&offline_source, &hold_operation)?, original_archive);
    let transport = at(
        "independently reopen backup without original source root",
        VerifiedBackup::open(
            &transport_root,
            &catalog,
            &transport_operation,
            &transport_digest,
            &stop,
        ),
    )?;
    transport.verify(&stop)?;
    assert_eq!(
        serde_json::to_value(transport.manifest())?["schema"],
        "wow-store/project-backup/2"
    );
    assert_eq!(
        transport.manifest().retained_quarantines(),
        retained.as_slice()
    );
    assert_eq!(transport.manifest().current(), Some(&first_current));

    let roundtrip_root = root.join("roundtrip");
    let roundtrip_operation = OperationId::new("fixture:restore-roundtrip")?;
    let roundtrip = transport.restore_to_new(&roundtrip_root, &roundtrip_operation, &stop)?;
    let roundtrip_digest = roundtrip.manifest().snapshot_digest().to_owned();
    drop(roundtrip);
    let roundtrip = at(
        "independently reopen transported candidate",
        VerifiedBackup::open(
            &roundtrip_root,
            &catalog,
            &roundtrip_operation,
            &roundtrip_digest,
            &stop,
        ),
    )?;
    let checks = owner_checks(&roundtrip, &[(&first, 11)], &stop)?;
    let roundtrip = at(
        "finish transported restore",
        roundtrip.finish_restore(checks, &stop),
    )?;
    assert_eq!(roundtrip.current()?, Some(first_current.clone()));
    assert_eq!(roundtrip.retained_quarantines()?, retained);
    assert_eq!(archive(&roundtrip_root, &hold_operation)?, original_archive);
    let selected: serde_json::Value =
        serde_json::from_slice(&fs::read(roundtrip_root.join(registry::REGISTRY_FILE))?)?;
    assert_eq!(selected["schema"], "wow-store/project-registry/5");
    drop(roundtrip);
    let reopened = ProjectStore::open(&roundtrip_root, &catalog)?;
    assert_eq!(reopened.current()?, Some(first_current.clone()));
    assert_eq!(reopened.retained_quarantines()?, retained);
    let read = reopened.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.manifest(), first.generation());
    check_fixture(&read, 11, &stop)?;
    drop(read);
    drop(reopened);
    // The transported hold remains admission authority, not an unreferenced copy.
    let evidence_path = roundtrip_root
        .join("quarantines")
        .join(registry::instance_id(&hold_operation)?)
        .join("evidence.json");
    let mut changed = original_archive[1].clone();
    changed.push(b'\n');
    fs::write(evidence_path, changed)?;
    rejected(
        ProjectStore::open(&roundtrip_root, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;

    drop(transport);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn restore_rejects_stale_hold_omitted_owners_and_changed_archive_then_reopens_exact_intent()
-> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, _) = new_store("restore-guards")?;
    let source = root.join("source");
    let (first, _) = publish(&mut store, "restore-guards-first", 11, &stop)?;
    let (second, second_current) = publish(&mut store, "restore-guards-second", 22, &stop)?;
    let backup = store.backup_to_new(
        root.join("before-incident"),
        &OperationId::new("fixture:restore-guards-backup")?,
        &stop,
    )?;
    let stale_selection = store.registry_selection()?;
    damage_pointer(&store)?;
    let inspection = store.quarantine_inspection(&stop)?;
    let hold_operation = OperationId::new("fixture:restore-guards-hold")?;
    let hold = store.quarantine(&hold_operation, &inspection, &stop)?;
    let quarantined = store.quarantined(&stop)?;
    let selector_bytes = fs::read(source.join(registry::REGISTRY_FILE))?;
    let original_archive = archive(&source, &hold_operation)?;
    drop(inspection);
    drop(store);

    let operation = OperationId::new("fixture:restore-guards-select")?;
    rejected(
        quarantined.stage_restore(&backup, &operation, &stale_selection, &stop),
        StoreErrorCode::CurrentConflict,
    )?;
    assert_eq!(
        fs::read(source.join(registry::REGISTRY_FILE))?,
        selector_bytes
    );
    let candidate = quarantined.stage_restore(&backup, &operation, hold.selected(), &stop)?;
    let request_digest = candidate.request_digest()?;
    let read = candidate.backup().read(
        &ReadSelector::Exact(first.generation().generation_id.clone()),
        &stop,
    )?;
    let incomplete = vec![check_fixture(&read, 11, &stop)?];
    drop(read);
    rejected(
        quarantined.activate_restore(candidate, incomplete, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(
        fs::read(source.join(registry::REGISTRY_FILE))?,
        selector_bytes
    );

    let candidate = at(
        "reopen after omitted owner",
        quarantined.reopen_restore(
            &operation,
            hold.selected(),
            backup.manifest().snapshot_digest(),
            &stop,
        ),
    )?;
    assert_eq!(candidate.request_digest()?, request_digest);
    let checks = owner_checks(candidate.backup(), &[(&first, 11), (&second, 22)], &stop)?;
    // The selected hold is unchanged, but its live recovery evidence is stale.
    let version = &second.generation().members[0].version;
    let connection = database::connect(&quarantined.path, false)?;
    let original_payload: Vec<u8> = connection.query_row(
        "SELECT payload FROM partition_versions WHERE version=?1",
        [version.as_str()],
        |row| row.get(0),
    )?;
    assert_eq!(
        connection.execute(
            "UPDATE partition_versions SET payload=?1 WHERE version=?2",
            rusqlite::params![
                wow_core::canonical_json_bytes(&vec![23u32])?,
                version.as_str()
            ],
        )?,
        1
    );
    drop(connection);
    rejected(
        quarantined.activate_restore(candidate, checks, &stop),
        StoreErrorCode::CurrentConflict,
    )?;
    assert_eq!(
        fs::read(source.join(registry::REGISTRY_FILE))?,
        selector_bytes
    );
    assert_eq!(archive(&source, &hold_operation)?, original_archive);
    let connection = database::connect(&quarantined.path, false)?;
    assert_eq!(
        connection.execute(
            "UPDATE partition_versions SET payload=?1 WHERE version=?2",
            rusqlite::params![original_payload, version.as_str()],
        )?,
        1
    );
    drop(connection);
    let candidate = quarantined.reopen_restore(
        &operation,
        hold.selected(),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    assert_eq!(candidate.request_digest()?, request_digest);
    let checks = owner_checks(candidate.backup(), &[(&first, 11), (&second, 22)], &stop)?;
    let evidence_path = source
        .join("quarantines")
        .join(registry::instance_id(&hold_operation)?)
        .join("evidence.json");
    let mut changed = original_archive[1].clone();
    changed.push(b'\n');
    fs::write(&evidence_path, changed)?;
    rejected(
        quarantined.activate_restore(candidate, checks, &stop),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(
        fs::read(source.join(registry::REGISTRY_FILE))?,
        selector_bytes
    );
    fs::write(evidence_path, &original_archive[1])?;
    let candidate = quarantined.reopen_restore(
        &operation,
        hold.selected(),
        backup.manifest().snapshot_digest(),
        &stop,
    )?;
    assert_eq!(candidate.request_digest()?, request_digest);
    let checks = owner_checks(candidate.backup(), &[(&first, 11), (&second, 22)], &stop)?;
    let (restored, receipt) = quarantined.activate_restore(candidate, checks, &stop)?;
    assert_eq!(receipt.previous(), hold.selected());
    assert_eq!(receipt.activated_current(), Some(&second_current));
    assert_eq!(restored.current()?, Some(second_current));
    assert_eq!(restored.retained_quarantines()?.len(), 1);
    assert_eq!(archive(&source, &hold_operation)?, original_archive);
    drop(restored);
    drop(quarantined);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn unreadable_header_restore_uses_the_hold_guard_and_preserves_unavailable_evidence() -> TestResult
{
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("restore-header")?;
    let source = root.join("source");
    let (first, first_current) = publish(&mut store, "restore-header-first", 11, &stop)?;
    let epoch = store.epoch().clone();
    let backup = store.backup_to_new(
        root.join("before-incident"),
        &OperationId::new("fixture:restore-header-backup")?,
        &stop,
    )?;
    let old_path = store.db.path.clone();
    drop(store);
    let connection = database::connect(&old_path, false)?;
    connection.pragma_update(None, "application_id", 0i64)?;
    drop(connection);
    rejected(
        ProjectStore::open(&source, &catalog),
        StoreErrorCode::IntegrityViolation,
    )?;
    let inspection = QuarantineInspection::open(&source, &catalog, &stop)?;
    let observation = CurrentObservation::Unreadable {
        reason: PointerReadFailure::QueryUnavailable,
    };
    assert_eq!(inspection.current(), &observation);
    let operation = OperationId::new("fixture:restore-header-hold")?;
    let hold = inspection.quarantine(&operation, &stop)?;
    let original_archive = archive(&source, &operation)?;
    let evidence: serde_json::Value = serde_json::from_slice(&original_archive[1])?;
    assert_eq!(evidence["state"], "unavailable");
    assert!(evidence.get("report").is_none());
    drop(inspection);
    let quarantined = QuarantinedStore::open(&source, &catalog, &stop)?;
    assert_eq!(quarantined.current_observation(&stop)?, observation);
    let candidate = at(
        "stage unreadable-header restore",
        quarantined.stage_restore(
            &backup,
            &OperationId::new("fixture:restore-header-select")?,
            hold.selected(),
            &stop,
        ),
    )?;
    let checks = owner_checks(candidate.backup(), &[(&first, 11)], &stop)?;
    let (restored, receipt) = at(
        "activate unreadable-header restore",
        quarantined.activate_restore(candidate, checks, &stop),
    )?;
    assert_eq!(receipt.previous(), hold.selected());
    assert_eq!(restored.epoch(), &epoch);
    assert_eq!(restored.current()?, Some(first_current.clone()));
    assert_eq!(restored.retained_quarantines()?.len(), 1);
    assert_eq!(archive(&source, &operation)?, original_archive);
    let archived: super::model::QuarantineRecord = serde_json::from_slice(&original_archive[2])?;
    assert_eq!(archived.current, observation);
    let connection = database::connect(&old_path, true)?;
    let application_id: i64 =
        connection.query_row("PRAGMA application_id", [], |row| row.get(0))?;
    assert_eq!(
        application_id, 0,
        "restoration must preserve the held damaged instance"
    );
    drop(connection);
    drop(restored);
    drop(quarantined);
    let reopened = ProjectStore::open(&source, &catalog)?;
    assert_eq!(reopened.current()?, Some(first_current));
    assert_eq!(reopened.retained_quarantines()?.len(), 1);
    drop(reopened);
    drop(backup);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn repeated_holds_survive_empty_target_adoption_and_portable_archive_roundtrip() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut store, catalog) = new_store("restore-repeated-holds")?;
    let source = root.join("source");
    let (first, current) = publish(&mut store, "repeated-first", 11, &stop)?;
    let original = store.backup_to_new(
        root.join("original"),
        &OperationId::new("fixture:repeated-original")?,
        &stop,
    )?;
    assert!(original.manifest().retained_quarantines().is_empty());
    let mut next_target = None;
    let mut saved_archives = Vec::new();
    for (index, name) in ["first", "second"].into_iter().enumerate() {
        let previous_refs = store.retained_quarantines()?;
        damage_pointer(&store)?;
        let inspection = store.quarantine_inspection(&stop)?;
        let operation = OperationId::new(format!("fixture:repeated-hold-{name}"))?;
        let hold = store.quarantine(&operation, &inspection, &stop)?;
        let bytes = archive(&source, &operation)?;
        let previous = registry::read_normal_shallow(&catalog, &bytes[0])?;
        assert_eq!(previous.retained_quarantines, previous_refs);
        assert_eq!(previous.selection, *hold.previous());
        saved_archives.push((operation, bytes));
        let quarantined = store.quarantined(&stop)?;
        drop(inspection);
        let target = match next_target.as_ref() {
            Some(target) => target,
            None => &original,
        };
        let candidate = quarantined.stage_restore(
            target,
            &OperationId::new(format!("fixture:repeated-restore-{name}"))?,
            hold.selected(),
            &stop,
        )?;
        let checks = owner_checks(candidate.backup(), &[(&first, 11)], &stop)?;
        let (restored, receipt) = quarantined.activate_restore(candidate, checks, &stop)?;
        assert_eq!(receipt.previous(), hold.selected());
        assert_eq!(restored.current()?, Some(current.clone()));
        let refs = restored.retained_quarantines()?;
        assert_eq!(refs.len(), index + 1);
        for (operation, bytes) in &saved_archives {
            let reference = refs
                .iter()
                .find(|r| r.operation_id() == operation)
                .ok_or("restoration dropped a historical hold")?;
            assert_eq!(
                reference.record_digest(),
                digest("project-registry", &bytes[2])
            );
            assert_eq!(archive(&source, operation)?, *bytes);
        }
        store = restored;
        drop(quarantined);
        if index == 0 {
            next_target = Some(store.backup_to_new(
                root.join("after-first-hold"),
                &OperationId::new("fixture:repeated-second-target")?,
                &stop,
            )?);
        }
    }
    let refs = store.retained_quarantines()?;
    assert_eq!(refs.len(), 2);
    let expected = store.registry_selection()?;
    let replacement_operation = OperationId::new("fixture:repeated-empty-target")?;
    let candidate = store.stage_replacement(
        &original,
        &replacement_operation,
        &expected,
        Some(current.record_id.clone()),
        &stop,
    )?;
    assert!(
        candidate
            .backup()
            .manifest()
            .retained_quarantines()
            .is_empty()
    );
    // Retain the real original connection to model loss of a successful result.
    let stale_db = database::Database {
        connection: database::connect(&store.db.path, false)?,
        path: store.db.path.clone(),
        epoch: store.db.epoch.clone(),
        root: store.db.root.clone(),
        selection: store.db.selection.clone(),
        life: std::rc::Rc::clone(&store.db.life),
    };
    let checks = owner_checks(candidate.backup(), &[(&first, 11)], &stop)?;
    let receipt = store.activate_replacement(candidate, checks, &stop)?;
    assert_eq!(store.retained_quarantines()?, refs);
    let selected_db = std::mem::replace(&mut store.db, stale_db);
    drop(selected_db);
    let candidate = store.reopen_replacement(
        &replacement_operation,
        &expected,
        Some(current.record_id.clone()),
        original.manifest().snapshot_digest(),
        &stop,
    )?;
    let checks = owner_checks(candidate.backup(), &[(&first, 11)], &stop)?;
    assert_eq!(
        store.activate_replacement(candidate, checks, &stop)?,
        receipt
    );
    assert_eq!(store.retained_quarantines()?, refs);
    let sidecar = source
        .join("instances")
        .join(registry::instance_id(&replacement_operation)?)
        .join("replacement-source.json");
    let omitted = root.join("omitted-source.json");
    fs::rename(&sidecar, &omitted)?;
    rejected(
        registry::read(&source, &catalog),
        StoreErrorCode::DatabaseUnavailable,
    )?;
    fs::rename(omitted, sidecar)?;
    assert_eq!(store.retained_quarantines()?, refs);

    let transport_root = root.join("two-hold-transport");
    let transport_operation = OperationId::new("fixture:repeated-transport")?;
    let transport = store.backup_to_new(&transport_root, &transport_operation, &stop)?;
    let snapshot = transport.manifest().snapshot_digest().to_owned();
    assert_eq!(transport.manifest().retained_quarantines(), refs.as_slice());
    drop(transport);
    drop(store);
    drop(next_target);
    drop(original);
    fs::rename(&source, root.join("source-offline"))?;
    assert!(!source.exists());
    assert!(!transport_root.join("instances").exists());
    let transport = at(
        "reopen two-hold artifact without source instances",
        VerifiedBackup::open(
            &transport_root,
            &catalog,
            &transport_operation,
            &snapshot,
            &stop,
        ),
    )?;
    assert_eq!(
        serde_json::to_value(transport.manifest())?["schema"],
        "wow-store/project-backup/2"
    );
    assert_eq!(transport.manifest().retained_quarantines(), refs.as_slice());
    let isolated_root = root.join("two-hold-isolated");
    let isolated = transport.restore_to_new(
        &isolated_root,
        &OperationId::new("fixture:repeated-isolated")?,
        &stop,
    )?;
    let checks = owner_checks(&isolated, &[(&first, 11)], &stop)?;
    let isolated = isolated.finish_restore(checks, &stop)?;
    assert_eq!(isolated.current()?, Some(current));
    assert_eq!(isolated.retained_quarantines()?, refs);
    assert!(!isolated_root.join("instances").exists());
    for (operation, bytes) in &saved_archives {
        assert_eq!(archive(&transport_root, operation)?, *bytes);
        assert_eq!(archive(&isolated_root, operation)?, *bytes);
    }
    drop(isolated);
    let reopened = ProjectStore::open(&isolated_root, &catalog)?;
    assert_eq!(reopened.retained_quarantines()?, refs);
    let read = reopened.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.manifest(), first.generation());
    check_fixture(&read, 11, &stop)?;
    drop(read);
    drop(reopened);
    drop(transport);
    fs::remove_dir_all(root)?;
    Ok(())
}
