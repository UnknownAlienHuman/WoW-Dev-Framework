use super::*;
use wow_store::OperationBegin;

pub fn compare_reference_lineage_snapshot(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    request: &LineageReadRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    resources(
        roots,
        path,
        request.store_limits,
        false,
        stop,
        |sources, slot| {
            let (receipt, _) = read_receipt(current(slot)?, request)?;
            let expected = qualify(sources, current(slot)?, &receipt, stop)?;
            let bytes = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .read_sealed(&receipt.snapshot, &expected, stop)
                .map_err(graph_error)?;
            bounded(bytes, request.max_output_bytes)
        },
    )
}
pub fn query_reference_lineage_snapshot(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    request: &LineageQueryRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    if request.reference_selection.is_none() {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    resources(
        roots,
        path,
        request.store_limits,
        false,
        stop,
        |sources, slot| {
            let operation =
                wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
            let hash = digest(
                "wow-service/reference-lineage-query-request/1",
                &(
                    sources.before.retained_owner().epoch().epoch_id(),
                    sources.after.retained_owner().epoch().epoch_id(),
                    current(slot)?.store_instance_id().map_err(store_error)?,
                    request,
                ),
            )?;
            if let Some(record) = current(slot)?.operation(&operation).map_err(store_error)? {
                if record.request_digest() != &hash {
                    return Err(failure(ServiceErrorCode::OperationConflict));
                }
                if record.state() == OperationState::Completed {
                    return crate::lineage::query::replay(
                        current(slot)?,
                        request,
                        &hash,
                        record.result_object_id().ok_or_else(unknown)?,
                    );
                }
                if record.state() != OperationState::Prepared {
                    return Err(failure(ServiceErrorCode::OperationReleased));
                }
            } else {
                match current(slot)?
                    .begin_operation(operation.clone(), hash.clone())
                    .map_err(store_error)?
                {
                    OperationBegin::Started(_) => {}
                    OperationBegin::Replay(_) => {
                        return Err(failure(ServiceErrorCode::OperationBusy));
                    }
                }
            }
            let (receipt, _) = read_receipt(current(slot)?, &request.selector())?;
            if request_digest(sources, current(slot)?, &receipt.request)? != receipt.request_digest
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            receipt.holds.verify(
                sources,
                current(slot)?.store_instance_id().map_err(store_error)?,
                &receipt.request,
                stop,
            )?;
            let result = sources.with_pair(&receipt.request, stop, |before, after| {
                let qualified = reference::build_input(
                    before,
                    after,
                    receipt
                        .request
                        .reference_selection
                        .as_ref()
                        .ok_or_else(unknown)?,
                    receipt.request.limits,
                    stop,
                )?;
                if serde_json::to_value(&qualified.comparison)
                    .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?
                    != receipt.reference_comparison
                {
                    return Err(failure(ServiceErrorCode::IdentityMismatch));
                }
                let expected =
                    PreparedLineageSnapshot::build(qualified.input, stop).map_err(graph_error)?;
                let left = before.pair().graph_read(stop).map_err(graph_error)?;
                let right = after.pair().graph_read(stop).map_err(graph_error)?;
                let selected = LineageSelections {
                    source: None,
                    reference: request.reference_selection.as_ref(),
                };
                let native = wow_graph::lineage::LineageNativeSources {
                    before: native_source_side(selected, before, &left)?,
                    after: native_source_side(selected, after, &right)?,
                };
                let mut owner = open_lineage_store(current(slot)?).map_err(graph_error)?;
                let mut view = owner
                    .open_lineage_graph_view(
                        &receipt.snapshot,
                        &expected,
                        wow_graph::lineage::LineageReadLeaseRequest {
                            operation: &operation,
                            request_digest: &hash,
                            expires_after: wow_store::LogicalEpoch::new(
                                request.lease_expires_after,
                            ),
                        },
                        request.limits,
                        stop,
                    )
                    .map_err(graph_error)?;
                let draft = view.query(&request.query, &native).map_err(graph_error);
                let closed = view.close().map_err(graph_error);
                match (draft, closed) {
                    (Ok(value), Ok(())) => Ok(value),
                    (Err(error), Ok(())) => Err(error),
                    (_, Err(error)) => Err(error),
                }
            })?;
            let capture = crate::lineage::query::QueryReceipt {
                schema: "wow-service/reference-lineage-query-result/1".into(),
                operation_id: request.operation_id.clone(),
                request_digest: hash.clone(),
                request: request.clone(),
                result,
                resource_closure: "closed".into(),
                capture_kind: "recorded_native_candidate_observation".into(),
                reference_comparison: Some(receipt.reference_comparison),
            };
            let bytes = encode(&capture, request.limits.max_output_bytes)?;
            let pending = PendingObject::from_json(
                "wow.service.reference_lineage.query",
                1,
                &capture,
                request.store_limits,
            )
            .map_err(store_error)?;
            let id = pending.object_id().clone();
            let mut batch = WriteBatch::new();
            batch.add_object(pending).map_err(store_error)?;
            current(slot)?
                .commit_operation(batch, &operation, &hash, &id)
                .map_err(|_| unknown())?;
            Ok(bytes)
        },
    )
}
