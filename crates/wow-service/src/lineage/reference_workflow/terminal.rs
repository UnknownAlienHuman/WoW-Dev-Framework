use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalReceipt {
    schema: Box<str>,
    operation_id: Box<str>,
    request_digest: RequestDigest,
    build_operation_id: Box<str>,
    build_request_digest: RequestDigest,
    snapshot_id: Box<str>,
    status: Box<str>,
    resource_closure: Box<str>,
}
pub fn retire_reference_lineage_snapshot(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    request: &LineageReadRequest,
    operation_id: &str,
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
            let operation = wow_store::OperationId::new(operation_id).map_err(store_error)?;
            let (receipt, _) = read_receipt(current(slot)?, request)?;
            let consumer = current(slot)?
                .store_instance_id()
                .map_err(store_error)?
                .clone();
            let hash = digest(
                "wow-service/reference-lineage-retire-request/1",
                &(&operation, request, &consumer),
            )?;
            let result = TerminalReceipt {
                schema: "wow-service/reference-lineage-retire-result/1".into(),
                operation_id: operation.as_str().into(),
                request_digest: hash.clone(),
                build_operation_id: request.build_operation_id.clone(),
                build_request_digest: request.build_request_digest.clone(),
                snapshot_id: request.snapshot_id.clone(),
                status: "retired".into(),
                resource_closure: "closed".into(),
            };
            let bytes = encode(&result, request.max_output_bytes)?;
            let pending = PendingObject::from_json(
                "wow.service.reference_lineage.retirement",
                1,
                &result,
                request.store_limits,
            )
            .map_err(store_error)?;
            let id = pending.object_id().clone();
            if let Some(record) = current(slot)?.operation(&operation).map_err(store_error)? {
                if record.request_digest() != &hash {
                    return Err(failure(ServiceErrorCode::OperationConflict));
                }
                if record.state() == OperationState::Completed {
                    if record.result_object_id() != Some(&id)
                        || current(slot)?
                            .object(&id)
                            .map_err(store_error)?
                            .is_none_or(|value| value.canonical_json() != bytes.as_ref())
                    {
                        return Err(unknown());
                    }
                    receipt
                        .holds
                        .release(sources, &consumer, &receipt.request, stop)?;
                    return Ok(bytes);
                }
                if record.state() != OperationState::Prepared {
                    return Err(failure(ServiceErrorCode::OperationReleased));
                }
            } else {
                current(slot)?
                    .begin_operation(operation.clone(), hash.clone())
                    .map_err(store_error)?;
            }
            let expected = qualify(sources, current(slot)?, &receipt, stop)?;
            open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .validate_sealed(&receipt.snapshot, &expected, stop)
                .map_err(graph_error)?;
            let mut batch = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .plan_retire(&receipt.snapshot)
                .map_err(graph_error)?;
            batch.add_object(pending).map_err(store_error)?;
            current(slot)?
                .commit_operation(batch, &operation, &hash, &id)
                .map_err(|_| unknown())?;
            receipt
                .holds
                .release(sources, &consumer, &receipt.request, stop)?;
            Ok(bytes)
        },
    )
}
pub fn abandon_reference_lineage_snapshot(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    request: &LineageBuildRequest,
    operation_id: &str,
    max_output_bytes: usize,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    require_reference(request)?;
    if max_output_bytes == 0 || max_output_bytes > 64 * 1024 * 1024 {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    resources(
        roots,
        path,
        request.store_limits,
        false,
        stop,
        |sources, slot| {
            let original =
                wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
            let operation = wow_store::OperationId::new(operation_id).map_err(store_error)?;
            if original == operation {
                return Err(failure(ServiceErrorCode::InvalidRequest));
            }
            let original_hash = request_digest(sources, current(slot)?, request)?;
            if let Some(record) = current(slot)?.operation(&original).map_err(store_error)? {
                if record.request_digest() != &original_hash {
                    return Err(failure(ServiceErrorCode::OperationConflict));
                }
                if record.state() == OperationState::Completed {
                    return Err(failure(ServiceErrorCode::InvalidRequest));
                }
            } else {
                current(slot)?
                    .begin_operation(original.clone(), original_hash.clone())
                    .map_err(store_error)?;
            }
            let consumer = current(slot)?
                .store_instance_id()
                .map_err(store_error)?
                .clone();
            let holds =
                ReferenceHoldPlan::prepare(sources, &consumer, request, &original_hash, stop)?;
            retain_intent(
                current(slot)?,
                &original,
                &Intent {
                    schema: "wow-service/reference-lineage-build-intent/1".into(),
                    request_digest: original_hash.clone(),
                    request: request.clone(),
                    holds: holds.clone(),
                },
            )?;
            let hash = digest(
                "wow-service/reference-lineage-abandon-request/1",
                &(&operation, &original_hash, &consumer, max_output_bytes),
            )?;
            let result = TerminalReceipt {
                schema: "wow-service/reference-lineage-abandon-result/1".into(),
                operation_id: operation.as_str().into(),
                request_digest: hash.clone(),
                build_operation_id: request.operation_id.clone(),
                build_request_digest: original_hash.clone(),
                snapshot_id: "not_published".into(),
                status: "abandoned".into(),
                resource_closure: "closed".into(),
            };
            let bytes = encode(&result, max_output_bytes)?;
            let pending = PendingObject::from_json(
                "wow.service.reference_lineage.abandonment",
                1,
                &result,
                request.store_limits,
            )
            .map_err(store_error)?;
            let id = pending.object_id().clone();
            if let Some(record) = current(slot)?.operation(&operation).map_err(store_error)? {
                if record.request_digest() != &hash {
                    return Err(failure(ServiceErrorCode::OperationConflict));
                }
                if record.state() == OperationState::Completed {
                    if record.result_object_id() != Some(&id)
                        || current(slot)?
                            .object(&id)
                            .map_err(store_error)?
                            .is_none_or(|value| value.canonical_json() != bytes.as_ref())
                    {
                        return Err(unknown());
                    }
                    if !open_lineage_store(current(slot)?)
                        .map_err(graph_error)?
                        .plan_abandon(original.as_str())
                        .map_err(graph_error)?
                        .catalog_mutations()
                        .is_empty()
                    {
                        return Err(unknown());
                    }
                    holds.release(sources, &consumer, request, stop)?;
                    return Ok(bytes);
                }
                if record.state() != OperationState::Prepared {
                    return Err(failure(ServiceErrorCode::OperationReleased));
                }
            } else {
                current(slot)?
                    .begin_operation(operation.clone(), hash.clone())
                    .map_err(store_error)?;
            }
            current(slot)?
                .record_failed(&original, &original_hash)
                .map_err(|_| unknown())?;
            let mut batch = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .plan_abandon(original.as_str())
                .map_err(graph_error)?;
            batch.add_object(pending).map_err(store_error)?;
            current(slot)?
                .commit_operation(batch, &operation, &hash, &id)
                .map_err(|_| unknown())?;
            holds.release(sources, &consumer, request, stop)?;
            Ok(bytes)
        },
    )
}
