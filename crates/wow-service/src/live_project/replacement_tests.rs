//! Native Project/Graph read-back across an explicit live physical replacement.
use super::super::LiveProjectStore;
use super::{owners, root};
use crate::ServiceErrorCode;
use std::sync::atomic::AtomicBool;
use wow_project::replay::publication::AcquiredProjectPair;
use wow_store::project::ReadSelector;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn live_replacement_replays_all_backup_pairs_and_preserves_third_generation_readers() -> TestResult
{
    let stop = AtomicBool::new(false);
    let (first, first_graph) = owners("local value = External() + 1\nreturn value\n")?;
    let (second, second_graph) = owners("local value = External() + 2\nreturn value\n")?;
    let (third, third_graph) = owners("local value = External() + 3\nreturn value\n")?;
    assert_eq!(
        first_graph.snapshot().universe(),
        second_graph.snapshot().universe()
    );
    assert_eq!(
        second_graph.snapshot().universe(),
        third_graph.snapshot().universe()
    );
    assert_ne!(
        first_graph.snapshot().snapshot_id(),
        second_graph.snapshot().snapshot_id()
    );
    assert_ne!(
        second_graph.snapshot().snapshot_id(),
        third_graph.snapshot().snapshot_id()
    );

    let source_root = root("replacement-service-source")?;
    let backup_root = root("replacement-service-backup")?;
    let mut store =
        LiveProjectStore::create(&source_root, first_graph.snapshot().universe().as_str())?;
    let epoch = store.storage_epoch().clone();
    let selection = store.registry_selection()?;
    let one = store.publish(&first, &first_graph, "fixture:replace-first", None, &stop)?;
    let first_current = one.activation.as_ref().ok_or("missing first activation")?;
    let first_ids = {
        let read = store.read(&ReadSelector::Current, &stop)?;
        assert_eq!(read.graph(), &first_graph);
        assert_eq!(read.store_generation_id(), &one.generation_id);
        (
            read.project().snapshot_id().to_owned(),
            read.project().analyzer_snapshot_id().to_owned(),
            read.publication_set_id().to_owned(),
        )
    };
    let two = store.publish(
        &second,
        &second_graph,
        "fixture:replace-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let second_current = two.activation.as_ref().ok_or("missing second activation")?;
    assert_eq!(
        second_current.predecessor.as_ref(),
        Some(&first_current.record_id)
    );
    let second_ids = {
        let read = store.read(&ReadSelector::Current, &stop)?;
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
    let backup = store.backup_to_new(&backup_root, "fixture:replace-backup", &stop)?;
    assert_eq!(backup.manifest().epoch(), &epoch);
    assert_eq!(backup.manifest().current(), Some(second_current));
    let mut expected_generations = vec![one.generation_id.clone(), two.generation_id.clone()];
    expected_generations.sort();
    assert_eq!(backup.manifest().generations(), expected_generations);
    for generation in backup.manifest().generations() {
        let read = backup.read(&ReadSelector::Exact(generation.clone()), &stop)?;
        assert_eq!(&read.manifest().generation_id, generation);
        let pair = AcquiredProjectPair::read(&read, &stop)?;
        let (ids, graph) = if generation == &one.generation_id {
            (&first_ids, &first_graph)
        } else {
            assert_eq!(generation, &two.generation_id);
            (&second_ids, &second_graph)
        };
        assert_eq!(pair.project().snapshot_id(), ids.0.as_str());
        assert_eq!(pair.project().analyzer_snapshot_id(), ids.1.as_str());
        assert_eq!(pair.publication_set_id(), ids.2.as_str());
        assert_eq!(pair.graph(), graph);
    }

    let three = store.publish(
        &third,
        &third_graph,
        "fixture:replace-third",
        Some(second_current.record_id.clone()),
        &stop,
    )?;
    let third_current = three
        .activation
        .as_ref()
        .ok_or("missing third activation")?;
    assert_eq!(
        third_current.predecessor.as_ref(),
        Some(&second_current.record_id)
    );
    let held_second = store.read(&ReadSelector::Exact(two.generation_id.clone()), &stop)?;
    let held_third = store.read(&ReadSelector::Current, &stop)?;
    let third_ids = (
        held_third.project().snapshot_id().to_owned(),
        held_third.project().analyzer_snapshot_id().to_owned(),
        held_third.publication_set_id().to_owned(),
    );
    assert_ne!(third_ids.0, second_ids.0);
    assert_eq!(held_second.current_at_acquisition(), Some(third_current));
    assert_eq!(held_third.current_at_acquisition(), Some(third_current));
    assert_eq!(held_second.graph(), &second_graph);
    assert_eq!(held_third.graph(), &third_graph);
    assert_eq!(store.registry_selection()?, selection);
    assert_eq!(
        store
            .restore_replace(
                &backup,
                "fixture:replace-stale-current",
                &selection,
                Some(second_current.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("service replacement accepted a stale current")?
            .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    assert_eq!(store.registry_selection()?, selection);
    assert_eq!(store.current()?.as_ref(), Some(third_current));

    let operation = "fixture:replace-live";
    let receipt = store.restore_replace(
        &backup,
        operation,
        &selection,
        Some(third_current.record_id.clone()),
        &stop,
    )?;
    let selected = store.registry_selection()?;
    assert_eq!(receipt.previous(), &selection);
    assert_eq!(receipt.selected(), &selected);
    assert_eq!(receipt.operation_id().as_str(), operation);
    assert_eq!(receipt.activated_current(), Some(second_current));
    assert_eq!(selected.revision(), selection.revision() + 1);
    assert_ne!(selected.digest(), selection.digest());
    assert_eq!(selected.epoch(), epoch.epoch_id());
    assert_eq!(store.storage_epoch(), &epoch);
    assert_eq!(store.current()?.as_ref(), Some(second_current));
    assert_eq!(
        store.replacement_receipt(operation, receipt.request_digest())?,
        Some(receipt.clone())
    );
    for generation in backup.manifest().generations() {
        let read = store.read(&ReadSelector::Exact(generation.clone()), &stop)?;
        let (ids, graph) = if generation == &one.generation_id {
            (&first_ids, &first_graph)
        } else {
            assert_eq!(generation, &two.generation_id);
            (&second_ids, &second_graph)
        };
        assert_eq!(read.store_generation_id(), generation);
        assert_eq!(read.project().snapshot_id(), ids.0.as_str());
        assert_eq!(read.project().analyzer_snapshot_id(), ids.1.as_str());
        assert_eq!(read.publication_set_id(), ids.2.as_str());
        assert_eq!(read.graph(), graph);
    }
    let target = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(target.store_generation_id(), &two.generation_id);
    assert_eq!(target.current_at_acquisition(), Some(second_current));
    assert_eq!(target.project().snapshot_id(), second_ids.0.as_str());
    assert_eq!(
        target.project().analyzer_snapshot_id(),
        second_ids.1.as_str()
    );
    assert_eq!(target.publication_set_id(), second_ids.2.as_str());
    assert_eq!(target.graph(), &second_graph);
    assert_eq!(held_second.store_generation_id(), &two.generation_id);
    assert_eq!(held_second.project().snapshot_id(), second_ids.0.as_str());
    assert_eq!(held_second.graph(), &second_graph);
    assert_eq!(held_second.current_at_acquisition(), Some(third_current));
    assert_eq!(held_third.store_generation_id(), &three.generation_id);
    assert_eq!(held_third.project().snapshot_id(), third_ids.0.as_str());
    assert_eq!(
        held_third.project().analyzer_snapshot_id(),
        third_ids.1.as_str()
    );
    assert_eq!(held_third.publication_set_id(), third_ids.2.as_str());
    assert_eq!(held_third.graph(), &third_graph);
    assert_eq!(held_third.current_at_acquisition(), Some(third_current));

    drop(store);
    assert_eq!(
        LiveProjectStore::open(&source_root)
            .err()
            .ok_or("replacement readers failed to retain the root lock")?
            .code(),
        ServiceErrorCode::OperationBusy
    );
    drop(held_second);
    drop(held_third);
    drop(target);
    let reopened = LiveProjectStore::open(&source_root)?;
    assert_eq!(reopened.registry_selection()?, selected);
    assert_eq!(reopened.storage_epoch(), &epoch);
    assert_eq!(reopened.current()?.as_ref(), Some(second_current));
    assert_eq!(
        reopened.replacement_receipt(operation, receipt.request_digest())?,
        Some(receipt)
    );
    let read = reopened.read(&ReadSelector::Current, &stop)?;
    assert_eq!(read.store_generation_id(), &two.generation_id);
    assert_eq!(read.project().snapshot_id(), second_ids.0.as_str());
    assert_eq!(read.project().analyzer_snapshot_id(), second_ids.1.as_str());
    assert_eq!(read.publication_set_id(), second_ids.2.as_str());
    assert_eq!(read.graph(), &second_graph);
    drop(read);
    drop(reopened);
    drop(backup);
    std::fs::remove_dir_all(source_root)?;
    std::fs::remove_dir_all(backup_root)?;
    Ok(())
}
