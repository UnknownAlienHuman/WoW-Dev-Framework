//! Exact Current domain replay preserves the independent physical observation.
use super::super::{CurrentDomainObservation, CurrentDomainPhase, LiveProjectStore};
use super::{owners, root};
use crate::ServiceErrorCode;
use std::sync::atomic::AtomicBool;
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, ProjectPublicationBundle};
use wow_store::OperationId;
use wow_store::project::{CurrentState, PartitionRecord, PublicationRequest, ReadSelector};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn current_observation_replays_exact_native_ids_and_preserves_physical_evidence() -> TestResult {
    let stop = AtomicBool::new(false);
    let (publisher, graph) = owners("local value = External() + 73\nreturn value\n")?;
    let root = root("current-domain-validated")?;
    let mut store = LiveProjectStore::create(&root, graph.snapshot().universe().as_str())?;
    let operation = store.publish(
        &publisher,
        &graph,
        "fixture:current-domain-validated",
        None,
        &stop,
    )?;
    let current = store.current()?.ok_or("missing published Current")?;
    assert_eq!(operation.activation.as_ref(), Some(&current));
    let held = store.read(&ReadSelector::Publication(current.record_id.clone()), &stop)?;
    let view = publisher.open_current()?;
    assert_eq!(held.project().snapshot_id(), view.snapshot_id());
    assert_eq!(
        held.project().analyzer_snapshot_id(),
        view.analyzer_snapshot_id()
    );
    assert_eq!(held.graph(), &graph);
    assert_eq!(held.store_generation_id(), &operation.generation_id);

    let physical = store.recovery_report(&stop)?;
    assert_eq!(physical.current_state(), CurrentState::Validated);
    assert_eq!(physical.current(), Some(&current));
    assert!(physical.incidents().is_empty());
    let observation = store.current_domain_observation(&stop)?;
    assert_eq!(observation.physical(), &physical);
    let CurrentDomainObservation::Validated(ids) = observation.current_domain() else {
        return Err(format!("native Current did not validate: {observation:?}").into());
    };
    assert_eq!(ids.publication_set_id(), held.publication_set_id());
    assert_eq!(ids.project_snapshot_id(), view.snapshot_id());
    assert_eq!(ids.analyzer_snapshot_id(), view.analyzer_snapshot_id());
    assert_eq!(
        ids.graph_snapshot_id(),
        graph.snapshot().snapshot_id().as_str()
    );
    assert_eq!(store.current()?, Some(current.clone()));
    assert_eq!(store.recovery_report(&stop)?, physical);
    assert_eq!(held.current_at_acquisition(), Some(&current));
    assert_eq!(held.graph(), &graph);

    let cancelled = AtomicBool::new(true);
    assert_eq!(
        store
            .current_domain_observation(&cancelled)
            .err()
            .ok_or("initially cancelled observation succeeded")?
            .code(),
        ServiceErrorCode::Cancelled
    );
    assert_eq!(store.current()?, Some(current));
    assert_eq!(store.recovery_report(&stop)?, physical);
    drop(held);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn empty_native_store_observes_absent_without_changing_physical_state() -> TestResult {
    let stop = AtomicBool::new(false);
    let (_, graph) = owners("return External()\n")?;
    let root = root("current-domain-absent")?;
    let store = LiveProjectStore::create(&root, graph.snapshot().universe().as_str())?;
    let physical = store.recovery_report(&stop)?;
    assert_eq!(physical.current_state(), CurrentState::Absent);
    assert!(physical.current().is_none());
    assert!(physical.operations().is_empty());
    assert!(physical.incidents().is_empty());
    let observation = store.current_domain_observation(&stop)?;
    assert_eq!(observation.physical(), &physical);
    assert_eq!(
        observation.current_domain(),
        &CurrentDomainObservation::Absent
    );
    assert!(store.current()?.is_none());
    assert_eq!(store.recovery_report(&stop)?, physical);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn unsupported_native_pair_retains_validated_physical_current_and_reports_replay_failure()
-> TestResult {
    let stop = AtomicBool::new(false);
    let (publisher, graph) = owners("return External() + 74\n")?;
    let root = root("current-domain-native-failure")?;
    let mut store = LiveProjectStore::create(&root, graph.snapshot().universe().as_str())?;
    let bundle = ProjectPublicationBundle::build(&publisher, &graph, &stop)?;
    let (mut records, bindings) = bundle.into_parts();
    let index = records
        .iter()
        .position(|record| record.key() == "live.project.header")
        .ok_or("missing native pair header")?;
    let mut header: serde_json::Value = records[index].decode()?;
    assert_eq!(header["schema"], "wow-project/live-project-graph-pair/1");
    header["schema"] = serde_json::Value::String("fixture:unsupported-native-pair".into());
    records[index] =
        PartitionRecord::new("live.project.header", "wow-project.live-pair.v1", &header)?;
    let request = PublicationRequest::new(
        store.store.epoch(),
        OperationId::new("fixture:current-domain-native-failure")?,
        None,
        bindings,
        records,
    )?;
    store.store.prepare(&request, &stop)?;
    let read = store.store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        &stop,
    )?;
    assert_eq!(read.manifest(), request.generation());
    assert_eq!(GraphPartitionSnapshot::read_stored(&read, &stop)?, graph);
    assert_eq!(
        read.record("live.project.header", &stop)?
            .ok_or("missing sealed unsupported header")?
            .decode::<serde_json::Value>()?,
        header
    );
    // Physical seals are valid; the separate native observer must replay the pair.
    let validation = read.owner_validation(&[
        GraphPartitionSnapshot::STORAGE_CHECK,
        publication::STORAGE_CHECK,
    ])?;
    drop(read);
    store.store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validation,
        &stop,
    )?;
    let activated =
        store
            .store
            .activate(request.operation_id(), request.request_digest(), &stop)?;
    let current = store.current()?.ok_or("missing physical Current")?;
    assert_eq!(activated.activation.as_ref(), Some(&current));
    let physical = store.recovery_report(&stop)?;
    assert_eq!(physical.current_state(), CurrentState::Validated);
    assert_eq!(physical.current(), Some(&current));
    assert!(physical.incidents().is_empty());
    let observation = store.current_domain_observation(&stop)?;
    assert_eq!(observation.physical(), &physical);
    let CurrentDomainObservation::Failed(failure) = observation.current_domain() else {
        return Err(format!("unsupported native pair lost its failure: {observation:?}").into());
    };
    assert_eq!(failure.phase(), CurrentDomainPhase::Replay);
    assert_eq!(failure.code(), ServiceErrorCode::IdentityMismatch);
    assert_eq!(store.current()?, Some(current));
    assert_eq!(store.recovery_report(&stop)?, physical);
    drop(store);
    std::fs::remove_dir_all(root)?;
    Ok(())
}
