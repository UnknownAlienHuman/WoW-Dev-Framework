//! Native identities survive physical migration into a validated inactive target.
use super::super::{
    LiveProjectStore, catalog, migrate_live_project_to_new, resume_live_project_migration,
};
use super::{owners, root};
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_store::project::{
    GC_PHYSICAL_PROFILE, PHYSICAL_PROFILE, ProjectStore, PublicationState, ReadSelector,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn native_v1_migration_preserves_pairs_current_and_exact_resumed_receipt() -> TestResult {
    let stop = AtomicBool::new(false);
    let (first, first_graph) = owners("local value = External() + 41\nreturn value\n")?;
    let (second, second_graph) = owners("local value = External() + 42\nreturn value\n")?;
    let source_root = root("migration-native-source")?;
    let backup_root = root("migration-native-backup")?;
    let target_root = root("migration-native-target")?;
    let mut live = LiveProjectStore {
        store: ProjectStore::create(
            &source_root,
            first_graph.snapshot().universe().as_str(),
            catalog()?,
        )?,
    };
    let one = live.publish(
        &first,
        &first_graph,
        "fixture:migration-native-first",
        None,
        &stop,
    )?;
    let first_current = one.activation.as_ref().ok_or("missing first Current")?;
    let held_first = live.read(&ReadSelector::Current, &stop)?;
    let first_ids = (
        held_first.project().snapshot_id().to_owned(),
        held_first.project().analyzer_snapshot_id().to_owned(),
        held_first.publication_set_id().to_owned(),
    );
    let two = live.publish(
        &second,
        &second_graph,
        "fixture:migration-native-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let second_current = two.activation.as_ref().ok_or("missing second Current")?;
    let second_ids = {
        let read = live.read(&ReadSelector::Current, &stop)?;
        assert_eq!(read.graph(), &second_graph);
        assert_eq!(read.store_generation_id(), &two.generation_id);
        (
            read.project().snapshot_id().to_owned(),
            read.project().analyzer_snapshot_id().to_owned(),
            read.publication_set_id().to_owned(),
        )
    };
    assert_ne!(first_ids.0, second_ids.0);
    assert_ne!(first_ids.2, second_ids.2);
    let backup = live.backup_to_new(&backup_root, "fixture:migration-native-backup", &stop)?;
    assert_eq!(backup.manifest().generations().len(), 2);
    assert_eq!(
        backup.manifest().epoch().physical_profile(),
        PHYSICAL_PROFILE
    );
    assert_eq!(backup.manifest().current(), Some(second_current));
    let source_snapshot = backup.manifest().snapshot_digest().to_owned();
    let operation = "fixture:migration-native";
    let target = migrate_live_project_to_new(&backup, &target_root, operation, &stop)?;
    let receipt = target.receipt().clone();
    assert_eq!(receipt.state(), PublicationState::ValidatedInactive);
    assert_eq!(receipt.source_epoch(), backup.manifest().epoch());
    assert_eq!(receipt.target_epoch(), target.target_epoch());
    assert_eq!(
        target.target_epoch().physical_profile(),
        GC_PHYSICAL_PROFILE
    );
    assert_ne!(
        receipt.source_epoch().epoch_id(),
        target.target_epoch().epoch_id()
    );
    assert_eq!(receipt.source_snapshot_digest(), source_snapshot);
    assert_eq!(target.mappings().len(), 2);
    assert_eq!(target.target_generations().len(), 2);
    assert_eq!(receipt.validations().len(), 2);

    for mapping in target.mappings() {
        let source = backup.read(
            &ReadSelector::Exact(mapping.source_generation().clone()),
            &stop,
        )?;
        let read = target.read_generation(mapping.target_generation(), &stop)?;
        assert_eq!(&read.manifest().generation_id, mapping.target_generation());
        assert_ne!(mapping.source_generation(), mapping.target_generation());
        assert_eq!(&read.manifest().epoch_id, target.target_epoch().epoch_id());
        assert_eq!(read.manifest().members, source.manifest().members);
        assert_eq!(read.manifest().bindings, source.manifest().bindings);
        assert!(read.manifest().expected_current.is_none());
        assert!(read.current_at_acquisition().is_none());
        let pair = AcquiredProjectPair::read(&read, &stop)?;
        let (ids, graph) = if mapping.source_generation() == &one.generation_id {
            (&first_ids, &first_graph)
        } else {
            assert_eq!(mapping.source_generation(), &two.generation_id);
            (&second_ids, &second_graph)
        };
        assert_eq!(pair.project().snapshot_id(), ids.0);
        assert_eq!(pair.project().analyzer_snapshot_id(), ids.1);
        assert_eq!(pair.publication_set_id(), ids.2);
        assert_eq!(pair.graph(), graph);
    }
    let current_mapping = receipt
        .current_mapping()
        .ok_or("missing inactive Current mapping")?;
    let mapped_second = receipt
        .mappings()
        .iter()
        .find(|mapping| mapping.source_generation() == &two.generation_id)
        .ok_or("missing second generation mapping")?;
    assert_eq!(current_mapping.source(), second_current);
    assert_eq!(
        current_mapping.target_generation(),
        mapped_second.target_generation()
    );
    assert_eq!(
        receipt
            .validations()
            .get(current_mapping.target_generation()),
        Some(current_mapping.target_validation())
    );
    assert!(!target_root.join("project-store-registry.json").exists());
    assert_eq!(live.current()?.as_ref(), Some(second_current));
    assert_eq!(held_first.project().snapshot_id(), first_ids.0);
    assert_eq!(held_first.project().analyzer_snapshot_id(), first_ids.1);
    assert_eq!(held_first.publication_set_id(), first_ids.2);
    assert_eq!(held_first.store_generation_id(), &one.generation_id);
    assert_eq!(held_first.graph(), &first_graph);

    drop(target);
    let resumed = resume_live_project_migration(&target_root, operation, &source_snapshot, &stop)?;
    assert_eq!(resumed.receipt(), &receipt);
    assert!(!target_root.join("project-store-registry.json").exists());
    assert_eq!(live.current()?.as_ref(), Some(second_current));
    assert_eq!(held_first.project().snapshot_id(), first_ids.0);
    assert_eq!(held_first.project().analyzer_snapshot_id(), first_ids.1);
    assert_eq!(held_first.publication_set_id(), first_ids.2);
    assert_eq!(held_first.store_generation_id(), &one.generation_id);
    assert_eq!(held_first.graph(), &first_graph);
    drop(resumed);
    drop(backup);
    drop(live);
    drop(held_first);
    for path in [source_root, backup_root, target_root] {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
