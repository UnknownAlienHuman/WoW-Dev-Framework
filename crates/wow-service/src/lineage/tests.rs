//! Real retained Project/Graph -> independently persisted Candidate snapshot.
use super::*;
use crate::live_project::{LiveProjectStore, tests::owners_with_files};
use std::error::Error;
use wow_project::ProjectInputFile;

#[test]
fn native_lineage_snapshot_survives_reopen_replay_and_exact_retirement()
-> Result<(), Box<dyn Error>> {
    let stop = AtomicBool::new(false);
    let root = std::env::temp_dir().join(format!(
        "wow-native-lineage-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    std::fs::create_dir(&root)?;
    let source_root = root.join("source");
    let store_path = root.join("lineage.sqlite");
    let text = "local function Shared() return 1 end\nreturn Shared()\n";
    let (first, first_graph) = owners_with_files(vec![ProjectInputFile::new("main.lua", text)?])?;
    let (second, second_graph) = owners_with_files(vec![
        ProjectInputFile::new("moved.lua", text)?,
        ProjectInputFile::new("copy.lua", text)?,
    ])?;
    let old_graph = wow_core::canonical_json_bytes(first_graph.snapshot())?;
    let mut source =
        LiveProjectStore::create(&source_root, first_graph.snapshot().universe().as_str())?;
    let before = source.publish(&first, &first_graph, "native-lineage-before", None, &stop)?;
    let expected = source.current()?.ok_or("Current before")?.record_id;
    let after = source.publish(
        &second,
        &second_graph,
        "native-lineage-after",
        Some(expected),
        &stop,
    )?;
    source.close()?;
    drop(first);
    drop(first_graph);
    drop(second);
    drop(second_graph);
    let request = LineageBuildRequest {
        schema: "wow-service/lineage-build/1".into(),
        source_selection: None,
        operation_id: "native-lineage-build".into(),
        before_generation: before.generation_id.as_str().into(),
        after_generation: after.generation_id.as_str().into(),
        limits: LineageLimits {
            max_entities: 32,
            max_bucket_members: 16,
            max_pairs: 64,
            max_component_nodes: 32,
            max_component_edges: 64,
            max_work: 128_000_000,
            max_metadata_bytes: 16 * 1024 * 1024,
        },
        max_source_bytes: 4 * 1024 * 1024,
        store_limits: LineageStoreLimits::default(),
        max_output_bytes: 8 * 1024 * 1024,
    };
    let bytes = build_lineage_snapshot(&source_root, &store_path, &request, &stop)?;
    let result: BuildReceipt = serde_json::from_slice(&bytes)?;
    assert_eq!(result.accepted_assertion_count, 0);
    assert_eq!(result.promotion.as_ref(), "not_evaluated");
    assert!(!result.negative_authority);
    let selector = LineageReadRequest {
        schema: "wow-service/lineage-read/1".into(),
        source_selection: None,
        build_operation_id: request.operation_id.clone(),
        build_request_digest: result.request_digest.clone(),
        snapshot_id: result.snapshot.snapshot_id.clone(),
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes,
    };
    let comparison = compare_lineage_snapshot(&source_root, &store_path, &selector, &stop)?;
    let value: serde_json::Value = serde_json::from_slice(&comparison)?;
    assert_eq!(value["accepted_assertion_count"], 0);
    assert_eq!(
        value["input"]["records"].as_array().ok_or("input")?.len(),
        6
    );
    assert_eq!(
        value["candidates"]["proposals"]
            .as_array()
            .ok_or("proposals")?
            .len(),
        4
    );
    assert!(!String::from_utf8_lossy(&comparison).contains(text));
    let saved_query =
        super::query_tests::native_queries(&source_root, &store_path, &result, &comparison, &stop)?;
    let consumer = Store::open_existing(&store_path, configuration(request.store_limits)?)?;
    let saved = consumer
        .object(&result.snapshot.object_id)?
        .ok_or("stored snapshot")?;
    assert_eq!(saved.canonical_json(), comparison.as_ref());
    consumer.close()?;
    let source = LiveProjectStore::open_retained_owner_store(&source_root)?;
    assert_eq!(source.retention_roots(&stop)?.len(), 2);
    source.close()?;
    assert_eq!(
        build_lineage_snapshot(&source_root, &store_path, &request, &stop)?,
        bytes,
        "same request returns original result after physical reopen/response loss"
    );
    let mut conflict = request.clone();
    conflict.max_source_bytes -= 1;
    assert_eq!(
        build_lineage_snapshot(&source_root, &store_path, &conflict, &stop)
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::OperationConflict)
    );
    let mut wrong = selector.clone();
    wrong.snapshot_id = "other-snapshot".into();
    assert!(compare_lineage_snapshot(&source_root, &store_path, &wrong, &stop).is_err());
    assert_eq!(
        compare_lineage_snapshot(&source_root, &store_path, &selector, &AtomicBool::new(true))
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::Cancelled)
    );
    let retirement = retire_lineage_snapshot(
        &source_root,
        &store_path,
        &selector,
        "native-lineage-retire",
        &stop,
    )?;
    super::query_tests::after_retire(&source_root, &store_path, &saved_query, &stop)?;
    assert_eq!(
        retire_lineage_snapshot(
            &source_root,
            &store_path,
            &selector,
            "native-lineage-retire",
            &stop
        )?,
        retirement
    );
    assert!(compare_lineage_snapshot(&source_root, &store_path, &selector, &stop).is_err());
    assert!(
        build_lineage_snapshot(&source_root, &store_path, &request, &stop).is_err(),
        "retired build cannot restore its roots by replay"
    );
    let consumer = Store::open_existing(&store_path, configuration(request.store_limits)?)?;
    assert_eq!(
        consumer
            .object(&result.snapshot.object_id)?
            .ok_or("retained old snapshot bytes")?
            .canonical_json(),
        comparison.as_ref()
    );
    consumer.close()?;
    let source = LiveProjectStore::open_retained_owner_store(&source_root)?;
    assert!(source.retention_roots(&stop)?.is_empty());
    source.close()?;
    let source = LiveProjectStore::open(&source_root)?;
    let retained = source.read(
        &wow_store::project::ReadSelector::Exact(before.generation_id),
        &stop,
    )?;
    assert_eq!(
        wow_core::canonical_json_bytes(retained.graph().snapshot())?,
        old_graph
    );
    retained.close()?;
    source.close()?;
    let mut failing = request.clone();
    failing.operation_id = "native-lineage-failed-build".into();
    failing.max_source_bytes = 1;
    assert!(build_lineage_snapshot(&source_root, &store_path, &failing, &stop).is_err());
    let source = LiveProjectStore::open_retained_owner_store(&source_root)?;
    assert_eq!(source.retention_roots(&stop)?.len(), 2);
    source.close()?;
    let abandoned = abandon_lineage_snapshot(
        &source_root,
        &store_path,
        &failing,
        "native-lineage-abandon",
        65536,
        &stop,
    )?;
    assert_eq!(
        abandon_lineage_snapshot(
            &source_root,
            &store_path,
            &failing,
            "native-lineage-abandon",
            65536,
            &stop
        )?,
        abandoned
    );
    assert_eq!(
        build_lineage_snapshot(&source_root, &store_path, &failing, &stop)
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::OperationReleased)
    );
    let source = LiveProjectStore::open_retained_owner_store(&source_root)?;
    assert!(source.retention_roots(&stop)?.is_empty());
    source.close()?;
    let mut tiny_output = request.clone();
    tiny_output.operation_id = "native-lineage-tiny-output".into();
    tiny_output.max_output_bytes = 1;
    assert_eq!(
        build_lineage_snapshot(&source_root, &store_path, &tiny_output, &stop)
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    abandon_lineage_snapshot(
        &source_root,
        &store_path,
        &tiny_output,
        "native-lineage-abandon-tiny-output",
        65536,
        &stop,
    )?;
    let source = LiveProjectStore::open_retained_owner_store(&source_root)?;
    assert!(source.retention_roots(&stop)?.is_empty());
    source.close()?;
    super::query_tests::native_view_leases(&source_root, &store_path, &request, &stop)?;
    std::fs::remove_dir_all(root)?;
    Ok(())
}
