//! Queries use the actual source-absent native snapshot, not decoded proof DTOs.
use super::*;
use std::error::Error;
type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub(super) struct SavedQuery {
    pub request: LineageQueryRequest,
    pub bytes: Box<[u8]>,
}
pub(super) fn native_queries(
    source: &Path,
    store: &Path,
    receipt: &BuildReceipt,
    comparison: &[u8],
    stop: &AtomicBool,
) -> TestResult<SavedQuery> {
    let snapshot: serde_json::Value = serde_json::from_slice(comparison)?;
    let proposal = &snapshot["candidates"]["proposals"][0];
    let before: LineageEntityAddress = serde_json::from_value(proposal["before"].clone())?;
    let after: LineageEntityAddress = serde_json::from_value(proposal["after"].clone())?;
    let mut request = LineageQueryRequest {
        schema: "wow-service/lineage-query/1".into(),
        source_selection: None,
        operation_id: "native-lineage-explain-query".into(),
        build_operation_id: receipt.request.operation_id.clone(),
        build_request_digest: receipt.request_digest.clone(),
        snapshot_id: receipt.snapshot.snapshot_id.clone(),
        store_limits: receipt.request.store_limits,
        query: LineageCandidateQuery::Explain {
            target: LineageExplanationTarget::Proposal {
                id: proposal["proposal_id"].as_str().ok_or("proposal")?.into(),
            },
        },
        limits: LineageQueryLimits {
            max_work: 128_000_000,
            max_metadata_bytes: 32 * 1024 * 1024,
            max_output_bytes: 8 * 1024 * 1024,
            max_components: 8,
            max_entities: 32,
            max_proposals: 64,
        },
        lease_expires_after: 10,
    };
    let bytes = query_lineage_snapshot(source, store, &request, stop)?;
    let parsed: serde_json::Value = serde_json::from_slice(&bytes)?;
    let result: LineageQueryResult = serde_json::from_value(parsed["result"].clone())?;
    assert_eq!(parsed["resource_closure"], "closed");
    assert_eq!(result.status.as_ref(), "candidate_only");
    assert_eq!(result.full_e4_b_validation.as_ref(), "not_evaluated");
    assert_eq!(result.accepted_assertion_count, 0);
    assert!(!result.negative_authority);
    let closure = result.closure.as_ref().ok_or("explain closure")?;
    assert_eq!(closure.components.len(), 1);
    assert_eq!(closure.proposals.len(), 2);
    assert_eq!(closure.functions.len(), 3);
    assert_eq!(closure.native_support.len(), 3);
    assert!(closure.components[0].ambiguous);
    for native in &closure.native_support {
        assert_eq!(native.original_source_file_node.kind(), "source_file");
        for id in native
            .entity_proposal
            .evidence_ids()
            .iter()
            .chain(native.ownership_proposal.evidence_ids())
        {
            assert!(
                closure
                    .evidence
                    .iter()
                    .any(|e| e.role == native.entity.role && e.evidence.evidence_id() == *id)
            );
        }
    }
    for evidence in &closure.evidence {
        for handle in evidence.evidence.source_handle_ids() {
            assert!(
                closure
                    .sources
                    .iter()
                    .any(|s| s.role == evidence.role && s.source.handle_id() == *handle)
            );
        }
    }
    assert!(!String::from_utf8_lossy(&bytes).contains("local function Shared()"));
    assert_eq!(
        query_lineage_snapshot(source, store, &request, stop)?,
        bytes
    );
    let saved = SavedQuery {
        request: request.clone(),
        bytes,
    };
    let mut changed = request.clone();
    changed.limits.max_work -= 1;
    assert_eq!(
        query_lineage_snapshot(source, store, &changed, stop)
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::OperationConflict)
    );
    request.operation_id = "native-lineage-compare-query".into();
    request.query = LineageCandidateQuery::Compare {
        before: before.clone(),
        after: after.clone(),
    };
    let compared: serde_json::Value =
        serde_json::from_slice(&query_lineage_snapshot(source, store, &request, stop)?)?;
    assert_eq!(
        compared["result"]["closure"]["proposals"]
            .as_array()
            .ok_or("all alternatives")?
            .len(),
        2
    );
    let mut wrong = request.clone();
    wrong.operation_id = "native-lineage-wrong-role-query".into();
    let mut wrong_before = before;
    wrong_before.role = LineageRole::After;
    wrong.query = LineageCandidateQuery::Compare {
        before: wrong_before,
        after,
    };
    assert!(query_lineage_snapshot(source, store, &wrong, stop).is_err());
    for (operation, query) in [
        (
            "native-lineage-status-query",
            LineageCandidateQuery::Status {},
        ),
        (
            "native-lineage-validation-query",
            LineageCandidateQuery::Validate {},
        ),
    ] {
        request.operation_id = operation.into();
        request.query = query;
        let response: serde_json::Value =
            serde_json::from_slice(&query_lineage_snapshot(source, store, &request, stop)?)?;
        assert_eq!(response["result"]["summary"]["functions"], 6);
        assert_eq!(response["result"]["summary"]["proposals"], 4);
        assert_eq!(response["result"]["full_e4_b_validation"], "not_evaluated");
    }
    request.operation_id = "native-lineage-missing-query".into();
    request.query = LineageCandidateQuery::Explain {
        target: LineageExplanationTarget::Component {
            id: format!(
                "wow-graph/lineage-candidate-component/1:sha256:{}",
                "0".repeat(64)
            )
            .into(),
        },
    };
    assert!(query_lineage_snapshot(source, store, &request, stop).is_err());
    request.operation_id = "native-lineage-tiny-query".into();
    request.query = LineageCandidateQuery::Status {};
    request.limits.max_output_bytes = 1;
    assert_eq!(
        query_lineage_snapshot(source, store, &request, stop)
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    request.operation_id = "native-lineage-cancel-query".into();
    assert_eq!(
        query_lineage_snapshot(source, store, &request, &AtomicBool::new(true))
            .err()
            .map(|e| e.code()),
        Some(ServiceErrorCode::Cancelled)
    );
    let opened = Store::open_existing(store, configuration(receipt.request.store_limits)?)?;
    assert!(
        opened.logical_manifest()?.leases().is_empty(),
        "read leases close on success, missing IDs and output failure"
    );
    opened.close()?;
    Ok(saved)
}
pub(super) fn after_retire(
    source: &Path,
    store: &Path,
    saved: &SavedQuery,
    stop: &AtomicBool,
) -> TestResult {
    assert_eq!(
        query_lineage_snapshot(source, store, &saved.request, stop)?,
        saved.bytes,
        "replay is original capture after retirement"
    );
    let mut fresh = saved.request.clone();
    fresh.operation_id = "native-lineage-retired-fresh-query".into();
    assert!(
        query_lineage_snapshot(source, store, &fresh, stop).is_err(),
        "fresh query cannot bypass original retention"
    );
    Ok(())
}

pub(super) fn native_view_leases(
    source_root: &Path,
    store_path: &Path,
    base: &LineageBuildRequest,
    stop: &AtomicBool,
) -> TestResult {
    use wow_graph::lineage::{
        LineageNativeSourceSide, LineageNativeSources, LineageReadLeaseRequest,
    };
    use wow_store::{LogicalEpoch, OperationId as StoreOperationId};
    let mut request = base.clone();
    request.operation_id = "native-lineage-view-lease-build".into();
    let receipt: BuildReceipt = serde_json::from_slice(&build_lineage_snapshot(
        source_root,
        store_path,
        &request,
        stop,
    )?)?;
    let mut source = crate::live_project::LiveProjectStore::open(source_root)?;
    let mut store = Store::open_existing(store_path, configuration(request.store_limits)?)?;
    let instance = store.store_instance_id()?.clone();
    with_prepared_pair(&request, &source, stop, |expected, before, after| {
        let left = before.pair().graph_read(stop).map_err(graph_error)?;
        let right = after.pair().graph_read(stop).map_err(graph_error)?;
        let l = before
            .pair()
            .source_provenance()
            .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
        let r = after
            .pair()
            .source_provenance()
            .ok_or_else(|| failure(ServiceErrorCode::ComponentUnavailable))?;
        let sources = LineageNativeSources {
            before: LineageNativeSourceSide {
                graph: &left,
                source_handles: l.source_handles(),
                evidence: l.evidence(),
            },
            after: LineageNativeSourceSide {
                graph: &right,
                source_handles: r.source_handles(),
                evidence: r.evidence(),
            },
        };
        let limits = LineageQueryLimits {
            max_work: 128_000_000,
            max_metadata_bytes: 32 * 1024 * 1024,
            max_output_bytes: 8 * 1024 * 1024,
            max_components: 8,
            max_entities: 32,
            max_proposals: 64,
        };
        let digest =
            RequestDigest::new(format!("sha256:{}", "4".repeat(64))).map_err(store_error)?;
        let operation = StoreOperationId::new("native-view-close-guard").map_err(store_error)?;
        store
            .begin_operation(operation.clone(), digest.clone())
            .map_err(store_error)?;
        {
            let mut owner = PersistentLineageStore::new(&mut store).map_err(graph_error)?;
            let view = owner
                .open_lineage_graph_view(
                    &receipt.snapshot,
                    &expected,
                    LineageReadLeaseRequest {
                        operation: &operation,
                        request_digest: &digest,
                        expires_after: LogicalEpoch::new(10),
                    },
                    limits,
                    stop,
                )
                .map_err(graph_error)?;
            let mut external =
                Store::open_existing(store_path, configuration(request.store_limits)?)
                    .map_err(store_error)?;
            let manifest = external.logical_manifest().map_err(store_error)?;
            assert_eq!(manifest.leases().len(), 2);
            let guard = &manifest.leases()[0];
            external
                .release_lease(guard.lease_id(), guard.holder())
                .map_err(store_error)?;
            assert_eq!(
                view.close().err().map(|e| e.code()),
                Some(wow_graph::GraphErrorCode::LineageReadCloseFailed)
            );
            assert!(
                external
                    .logical_manifest()
                    .map_err(store_error)?
                    .leases()
                    .is_empty(),
                "close attempts both roles after a missing guard"
            );
            external.close().map_err(store_error)?;
        }
        let operation = StoreOperationId::new("native-view-gc-guard").map_err(store_error)?;
        store
            .begin_operation(operation.clone(), digest.clone())
            .map_err(store_error)?;
        let mut owner = PersistentLineageStore::new(&mut store).map_err(graph_error)?;
        let mut view = owner
            .open_lineage_graph_view(
                &receipt.snapshot,
                &expected,
                LineageReadLeaseRequest {
                    operation: &operation,
                    request_digest: &digest,
                    expires_after: LogicalEpoch::new(10),
                },
                limits,
                stop,
            )
            .map_err(graph_error)?;
        let mut external = Store::open_existing(store_path, configuration(request.store_limits)?)
            .map_err(store_error)?;
        let retire = PersistentLineageStore::new(&mut external)
            .map_err(graph_error)?
            .plan_retire(&receipt.snapshot)
            .map_err(graph_error)?;
        external.commit(retire).map_err(store_error)?;
        external
            .collect_garbage(LogicalEpoch::new(1), 100)
            .map_err(store_error)?;
        for id in [
            &receipt.snapshot.object_id,
            &receipt.snapshot.validation_object_id,
        ] {
            assert!(
                external.object(id).map_err(store_error)?.is_some(),
                "view leases retain both objects after unpublish"
            );
        }
        let observed = view
            .query(&LineageCandidateQuery::Status {}, &sources)
            .map_err(graph_error)?;
        assert_eq!(observed.summary.functions, 6);
        view.close().map_err(graph_error)?;
        assert!(
            external
                .logical_manifest()
                .map_err(store_error)?
                .leases()
                .is_empty()
        );
        external
            .collect_garbage(LogicalEpoch::new(1), 100)
            .map_err(store_error)?;
        for id in [
            &receipt.snapshot.object_id,
            &receipt.snapshot.validation_object_id,
        ] {
            assert!(
                external.object(id).map_err(store_error)?.is_none(),
                "unpublished objects become collectable after checked close"
            );
        }
        external.close().map_err(store_error)?;
        Ok(())
    })?;
    receipt
        .source_holds
        .release(source.retained_owner_mut(), &instance, stop)?;
    source.close()?;
    store.close()?;
    Ok(())
}
