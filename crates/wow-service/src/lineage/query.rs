//! One exact journaled query; replay is the original capture, never a fresh
//! validation. New captures hold native before/after owners until Graph closes.
use super::*;
pub use wow_graph::lineage::{
    LineageCandidateQuery, LineageEntityAddress, LineageExplanationTarget, LineageQueryLimits,
    LineageQueryResult, LineageRole,
};
use wow_graph::lineage::{LineageNativeSourceSide, LineageNativeSources, LineageReadLeaseRequest};
use wow_store::{LogicalEpoch, OperationBegin};
const KIND: &str = "wow.service.lineage.query";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageQueryRequest {
    pub schema: Box<str>,
    pub operation_id: Box<str>,
    pub build_operation_id: Box<str>,
    pub build_request_digest: RequestDigest,
    pub snapshot_id: Box<str>,
    pub store_limits: LineageStoreLimits,
    pub query: LineageCandidateQuery,
    pub limits: LineageQueryLimits,
    pub lease_expires_after: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "selection_option"
    )]
    pub source_selection: Option<PlatformProjectSelection>,
}
impl LineageQueryRequest {
    fn selector(&self) -> LineageReadRequest {
        LineageReadRequest {
            schema: if self.source_selection.is_some() {
                "wow-service/source-lineage-read/1"
            } else {
                "wow-service/lineage-read/1"
            }
            .into(),
            build_operation_id: self.build_operation_id.clone(),
            build_request_digest: self.build_request_digest.clone(),
            snapshot_id: self.snapshot_id.clone(),
            store_limits: self.store_limits,
            max_output_bytes: self.limits.max_output_bytes,
            source_selection: self.source_selection.clone(),
        }
    }
    fn validate(&self) -> ServiceResult<()> {
        selection_schema(
            &self.schema,
            self.source_selection.as_ref(),
            "wow-service/lineage-query/1",
            "wow-service/source-lineage-query/1",
        )?;
        if self.lease_expires_after == 0 || self.lease_expires_after > i64::MAX as u64 {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        self.selector().validate()?;
        wow_store::OperationId::new(self.operation_id.clone()).map_err(store_error)?;
        self.query.validate().map_err(graph_error)?;
        self.limits.validate().map_err(graph_error)?;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct QueryReceipt {
    schema: Box<str>,
    operation_id: Box<str>,
    request_digest: RequestDigest,
    request: LineageQueryRequest,
    result: LineageQueryResult,
    resource_closure: Box<str>,
    capture_kind: Box<str>,
}

pub fn query_lineage_snapshot(
    source_root: &Path,
    lineage_store: &Path,
    request: &LineageQueryRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    resources(
        source_root,
        lineage_store,
        request.store_limits,
        false,
        request.source_selection.as_ref(),
        stop,
        |source, slot| run(source, current(slot)?, request, stop),
    )
    .map_err(|error| {
        ServiceError::for_operation(error.code(), error.message(), &request.operation_id)
    })
}
fn run(
    source: &crate::live_project::LiveProjectStore,
    store: &mut Store,
    request: &LineageQueryRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    checkpoint(stop)?;
    let operation =
        wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
    let hash = wow_core::domain_separated_digest(
        "wow-service/lineage-query-request/1",
        &(
            source.retained_owner().epoch().epoch_id(),
            store.store_instance_id().map_err(store_error)?,
            request,
        ),
    )
    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    let digest = RequestDigest::new(
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(hash).to_string(),
    )
    .map_err(store_error)?;
    if let Some(record) = store.operation(&operation).map_err(store_error)? {
        if record.request_digest() != &digest {
            return Err(failure(ServiceErrorCode::OperationConflict));
        }
        if record.state() == OperationState::Completed {
            return replay(
                store,
                request,
                &digest,
                record.result_object_id().ok_or_else(unknown)?,
            );
        }
        if record.state() != OperationState::Prepared {
            return Err(failure(ServiceErrorCode::OperationReleased));
        }
    } else {
        match store
            .begin_operation(operation.clone(), digest.clone())
            .map_err(|_| unknown())?
        {
            OperationBegin::Started(_) => {}
            OperationBegin::Replay(_) => return Err(failure(ServiceErrorCode::OperationBusy)),
        }
    }
    let build = workflow::read_receipt(store, &request.selector())?;
    verify_source_binding(source, store, &build, stop)?;
    let result = with_prepared_pair(&build.request, source, stop, |expected, before, after| {
        let left = before.pair().graph_read(stop).map_err(graph_error)?;
        let right = after.pair().graph_read(stop).map_err(graph_error)?;
        let left_sources = native_provenance(build.request.source_selection.as_ref(), before)?;
        let right_sources = native_provenance(build.request.source_selection.as_ref(), after)?;
        let sources = LineageNativeSources {
            before: LineageNativeSourceSide {
                graph: &left,
                source_handles: left_sources.source_handles(),
                evidence: left_sources.evidence(),
            },
            after: LineageNativeSourceSide {
                graph: &right,
                source_handles: right_sources.source_handles(),
                evidence: right_sources.evidence(),
            },
        };
        let mut owner = open_lineage_store(store).map_err(graph_error)?;
        let mut view = owner
            .open_lineage_graph_view(
                &build.snapshot,
                &expected,
                LineageReadLeaseRequest {
                    operation: &operation,
                    request_digest: &digest,
                    expires_after: LogicalEpoch::new(request.lease_expires_after),
                },
                request.limits,
                stop,
            )
            .map_err(query_error)?;
        let draft = view.query(&request.query, &sources).map_err(graph_error);
        let closed = view.close().map_err(query_error);
        match (draft, closed) {
            (Ok(result), Ok(())) => Ok(result),
            (Err(error), Ok(())) => Err(error),
            (_, Err(error)) => Err(error),
        }
    })?;
    checkpoint(stop)?;
    let receipt = QueryReceipt {
        schema: if request.source_selection.is_some() {
            "wow-service/source-lineage-query-result/1"
        } else {
            "wow-service/lineage-query-result/1"
        }
        .into(),
        operation_id: request.operation_id.clone(),
        request_digest: digest.clone(),
        request: request.clone(),
        result,
        resource_closure: "closed".into(),
        capture_kind: "recorded_native_candidate_observation".into(),
    };
    let bytes = encode(&receipt, request.limits.max_output_bytes)?;
    let kind = if request.source_selection.is_some() {
        "wow.service.source_lineage.query"
    } else {
        KIND
    };
    let pending =
        PendingObject::from_json(kind, 1, &receipt, request.store_limits).map_err(store_error)?;
    let result_id = pending.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(pending).map_err(store_error)?;
    store
        .commit_operation(batch, &operation, &digest, &result_id)
        .map_err(|_| unknown())?;
    Ok(bytes)
}
fn replay(
    store: &Store,
    request: &LineageQueryRequest,
    digest: &RequestDigest,
    id: &ObjectId,
) -> ServiceResult<Box<[u8]>> {
    let object = store.object(id).map_err(store_error)?.ok_or_else(unknown)?;
    let kind = if request.source_selection.is_some() {
        "wow.service.source_lineage.query"
    } else {
        KIND
    };
    if object.kind() != kind || object.schema_version() != 1 {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let receipt: QueryReceipt = object.decode().map_err(store_error)?;
    let schema = if request.source_selection.is_some() {
        "wow-service/source-lineage-query-result/1"
    } else {
        "wow-service/lineage-query-result/1"
    };
    if receipt.schema.as_ref() != schema
        || receipt.operation_id != request.operation_id
        || receipt.request_digest != *digest
        || receipt.request != *request
        || receipt.resource_closure.as_ref() != "closed"
        || receipt.capture_kind.as_ref() != "recorded_native_candidate_observation"
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    if request.source_selection.is_some() {
        receipt
            .result
            .validate_source_capture(&request.snapshot_id, &request.query)
    } else {
        receipt
            .result
            .validate_capture(&request.snapshot_id, &request.query)
    }
    .map_err(graph_error)?;
    bounded(
        object.canonical_json().into(),
        request.limits.max_output_bytes,
    )
}
fn query_error(error: wow_graph::GraphError) -> ServiceError {
    if matches!(
        error.code(),
        wow_graph::GraphErrorCode::StoreFailure | wow_graph::GraphErrorCode::LineageReadCloseFailed
    ) {
        unknown()
    } else {
        graph_error(error)
    }
}
