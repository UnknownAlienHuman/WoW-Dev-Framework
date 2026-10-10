//! Real native pairs retain identity while a root hold blocks new admission.
use super::super::{LiveProjectStore, QuarantinedLiveProject};
use super::{owners, root};
use crate::{ServiceErrorCode, ServiceResult};
use std::sync::atomic::AtomicBool;
use wow_store::project::{CurrentState, ReadSelector, ScopeState};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn rejected<T>(result: ServiceResult<T>, expected: ServiceErrorCode) -> TestResult {
    assert_eq!(
        result
            .err()
            .ok_or("operation unexpectedly succeeded")?
            .code(),
        expected
    );
    Ok(())
}

#[test]
fn native_quarantine_rejects_stale_inspection_preserves_pairs_and_reopens_readonly() -> TestResult {
    let stop = AtomicBool::new(false);
    let (first, first_graph) = owners("local value = External() + 1\nreturn value\n")?;
    let (second, second_graph) = owners("local value = External() + 2\nreturn value\n")?;
    let source = root("quarantine-service-source")?;
    let backup = root("quarantine-service-refused-backup")?;
    let mut store = LiveProjectStore::create(&source, first_graph.snapshot().universe().as_str())?;
    let one = store.publish(
        &first,
        &first_graph,
        "fixture:quarantine-first",
        None,
        &stop,
    )?;
    let first_current = one.activation.as_ref().ok_or("missing first activation")?;
    let held_first = store.read(&ReadSelector::Current, &stop)?;
    let first_project_id = held_first.project().snapshot_id().to_owned();
    let stale = store.quarantine_inspection(&stop)?;
    let two = store.publish(
        &second,
        &second_graph,
        "fixture:quarantine-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    rejected(
        store.quarantine("fixture:stale-hold", &stale, &stop),
        ServiceErrorCode::StoreCurrentConflict,
    )?;
    let held_second = store.read(&ReadSelector::Current, &stop)?;
    let second_project_id = held_second.project().snapshot_id().to_owned();
    assert_ne!(first_project_id, second_project_id);
    let fresh = store.quarantine_inspection(&stop)?;
    assert_ne!(fresh.current(), stale.current());
    let receipt = store.quarantine("fixture:native-hold", &fresh, &stop)?;
    assert_eq!(receipt.previous(), fresh.selection());
    assert!(receipt.selected().is_quarantined());
    assert_eq!(
        store.quarantine("fixture:native-hold", &fresh, &stop)?,
        receipt
    );

    rejected(store.current(), ServiceErrorCode::StoreQuarantined)?;
    rejected(
        store.read(&ReadSelector::Current, &stop),
        ServiceErrorCode::StoreQuarantined,
    )?;
    rejected(
        store.publish(
            &second,
            &second_graph,
            "fixture:blocked-native",
            None,
            &stop,
        ),
        ServiceErrorCode::StoreQuarantined,
    )?;
    rejected(
        store.backup_to_new(&backup, "fixture:blocked-native-backup", &stop),
        ServiceErrorCode::StoreQuarantined,
    )?;
    assert!(!backup.exists());
    assert_eq!(held_first.project().snapshot_id(), first_project_id);
    assert_eq!(held_first.graph(), &first_graph);
    assert_eq!(held_first.store_generation_id(), &one.generation_id);
    assert_eq!(held_second.project().snapshot_id(), second_project_id);
    assert_eq!(held_second.graph(), &second_graph);
    assert_eq!(held_second.store_generation_id(), &two.generation_id);
    let held = store.quarantined(&stop)?;
    assert_eq!(held.receipt(), &receipt);
    assert_eq!(held.current_observation(&stop)?, *fresh.current());
    let report = held.recovery_report(&stop)?;
    assert_eq!(report.current_state(), CurrentState::Validated);
    assert!(
        report
            .coverage()
            .iter()
            .all(|c| matches!(c.state(), ScopeState::Validated | ScopeState::NotApplicable))
    );
    // A physically valid report does not release the explicitly selected hold.
    rejected(store.current(), ServiceErrorCode::StoreQuarantined)?;
    drop(held);
    drop(stale);
    drop(fresh);
    drop(store);
    rejected(
        QuarantinedLiveProject::open(&source, &stop),
        ServiceErrorCode::OperationBusy,
    )?;
    assert_eq!(held_first.graph(), &first_graph);
    assert_eq!(held_second.graph(), &second_graph);
    drop(held_first);
    drop(held_second);
    rejected(
        LiveProjectStore::open(&source),
        ServiceErrorCode::StoreQuarantined,
    )?;
    let reopened = QuarantinedLiveProject::open(&source, &stop)?;
    assert_eq!(reopened.receipt(), &receipt);
    assert_eq!(reopened.recovery_report(&stop)?, report);
    assert_eq!(reopened.current_observation(&stop)?, *receipt.current());
    drop(reopened);
    std::fs::remove_dir_all(source)?;
    Ok(())
}
