//! Durable intent, exact publication, response-loss replay and retirement.
use super::*;

const RECEIPT_KIND: &str = "wow.service.lineage.build";
const INTENT_KIND: &str = "wow.service.lineage.intent";
const INTENTS: &str = "lineage.intents";
fn chosen(source: bool, ordinary: &'static str, selected: &'static str) -> &'static str {
    if source { selected } else { ordinary }
}
fn receipt_kind(source: bool) -> &'static str {
    chosen(source, RECEIPT_KIND, "wow.service.source_lineage.build")
}
fn receipt_schema(source: bool) -> &'static str {
    chosen(
        source,
        "wow-service/lineage-build-result/1",
        "wow-service/source-lineage-build-result/1",
    )
}
fn intent_kind(source: bool) -> &'static str {
    chosen(source, INTENT_KIND, "wow.service.source_lineage.intent")
}
fn intent_schema(source: bool) -> &'static str {
    chosen(
        source,
        "wow-service/lineage-build-intent/1",
        "wow-service/source-lineage-build-intent/1",
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildIntent {
    schema: Box<str>,
    request_digest: RequestDigest,
    request: LineageBuildRequest,
    source_holds: SourceHoldPlan,
}

pub(super) fn build_digest(
    source: &crate::live_project::LiveProjectStore,
    store: &Store,
    request: &LineageBuildRequest,
) -> ServiceResult<RequestDigest> {
    digest(
        "wow-service/lineage-build-request/1",
        &(
            source.retained_owner().epoch().epoch_id(),
            store.store_instance_id().map_err(store_error)?,
            request,
        ),
    )
}

pub(super) fn build(
    lineage_path: &Path,
    source: &mut crate::live_project::LiveProjectStore,
    slot: &mut Option<Store>,
    request: &LineageBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    checkpoint(stop)?;
    let store = current(slot)?;
    let operation =
        wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
    let request_digest = build_digest(source, store, request)?;
    if let Some(record) = store.operation(&operation).map_err(store_error)? {
        if record.request_digest() != &request_digest {
            return Err(failure(ServiceErrorCode::OperationConflict));
        }
        match record.state() {
            OperationState::Completed => {
                let selector = LineageReadRequest {
                    schema: chosen(
                        request.source_selection.is_some(),
                        "wow-service/lineage-read/1",
                        "wow-service/source-lineage-read/1",
                    )
                    .into(),
                    build_operation_id: request.operation_id.clone(),
                    build_request_digest: request_digest,
                    snapshot_id: "".into(),
                    store_limits: request.store_limits,
                    max_output_bytes: request.max_output_bytes,
                    source_selection: request.source_selection.clone(),
                    reference_selection: None,
                };
                let (receipt, bytes) = recorded(store, &selector, false)?;
                let expected = qualify(source, store, &receipt, stop)?;
                open_lineage_store(store)
                    .map_err(graph_error)?
                    .validate_sealed(&receipt.snapshot, &expected, stop)
                    .map_err(|_| failure(ServiceErrorCode::OperationReleased))?;
                return bounded(bytes, request.max_output_bytes);
            }
            OperationState::Prepared => {}
            OperationState::Failed | OperationState::NoEffect => {
                return Err(failure(ServiceErrorCode::OperationReleased));
            }
            _ => return Err(unknown()),
        }
    } else {
        store
            .begin_operation(operation.clone(), request_digest.clone())
            .map_err(|_| unknown())?;
    }
    let consumer = store.store_instance_id().map_err(store_error)?.clone();
    let hold = if request.source_selection.is_some() {
        SourceHoldPlan::prepare_source
    } else {
        SourceHoldPlan::prepare
    };
    let holds = hold(
        source.retained_owner(),
        &consumer,
        &OperationId::new(request.operation_id.clone())?,
        &request_digest,
        &StoreGenerationId::parse(request.before_generation.to_string()).map_err(store_error)?,
        &StoreGenerationId::parse(request.after_generation.to_string()).map_err(store_error)?,
        stop,
    )?;
    let intent = BuildIntent {
        schema: intent_schema(request.source_selection.is_some()).into(),
        request_digest: request_digest.clone(),
        request: request.clone(),
        source_holds: holds.clone(),
    };
    retain_intent(store, &operation, &intent)?;
    // The exact plan is durable before either source-owner dispatch. Any failure
    // leaves Prepared intent and installed roots for explicit same-request resume.
    holds
        .install(source.retained_owner_mut(), &consumer, stop)
        .map_err(|_| unknown())?;
    let expected = prepare(request, source, stop)?;
    let stored = open_lineage_store(current(slot)?)
        .map_err(graph_error)?
        .store_inactive(
            operation.as_str(),
            &expected,
            &operation,
            &request_digest,
            stop,
        )
        .map_err(|_| unknown())?;
    reopen(slot, lineage_path, request.store_limits)?;
    let validated = open_lineage_store(current(slot)?)
        .map_err(graph_error)?
        .validate_inactive(stored, &expected, stop)
        .map_err(graph_error)?;
    holds.verify(
        source.retained_owner(),
        current(slot)?.store_instance_id().map_err(store_error)?,
        stop,
    )?;
    let plan = open_lineage_store(current(slot)?)
        .map_err(graph_error)?
        .plan_seal(validated, stop)
        .map_err(graph_error)?;
    let receipt = BuildReceipt {
        schema: receipt_schema(request.source_selection.is_some()).into(),
        operation: "lineage_build".into(),
        request_digest: request_digest.clone(),
        request: request.clone(),
        source_holds: holds,
        snapshot: plan.handle,
        status: "candidate_only".into(),
        resource_closure: "closed".into(),
        accepted_assertion_count: 0,
        promotion: "not_evaluated".into(),
        negative_authority: false,
    };
    let bytes = encode(&receipt, request.max_output_bytes)?;
    let pending = PendingObject::from_json(
        receipt_kind(request.source_selection.is_some()),
        1,
        &receipt,
        request.store_limits,
    )
    .map_err(store_error)?;
    let result_id = pending.object_id().clone();
    let mut batch = plan.batch;
    batch.add_object(pending).map_err(store_error)?;
    checkpoint(stop)?;
    current(slot)?
        .commit_operation(batch, &operation, &request_digest, &result_id)
        .map_err(|_| unknown())?;
    reopen(slot, lineage_path, request.store_limits)?;
    let store = current(slot)?;
    open_lineage_store(store)
        .map_err(graph_error)?
        .validate_sealed(&receipt.snapshot, &expected, stop)
        .map_err(graph_error)?;
    receipt.source_holds.verify(
        source.retained_owner(),
        store.store_instance_id().map_err(store_error)?,
        stop,
    )?;
    Ok(bytes)
}

fn retain_intent(
    store: &mut Store,
    operation: &wow_store::OperationId,
    intent: &BuildIntent,
) -> ServiceResult<()> {
    let kind = intent_kind(intent.request.source_selection.is_some());
    let pending = PendingObject::from_json(kind, 1, intent, store.configuration().limits())
        .map_err(store_error)?;
    let catalog = CatalogName::new(INTENTS).map_err(store_error)?;
    let path = CatalogPath::new(operation.as_str().to_owned()).map_err(store_error)?;
    if let Some(entry) = store.catalog_entry(&catalog, &path).map_err(store_error)? {
        let object = store
            .object(entry.object_id())
            .map_err(store_error)?
            .ok_or_else(unknown)?;
        if object.object_id() != pending.object_id()
            || object.kind() != kind
            || object.schema_version() != 1
            || object.decode::<BuildIntent>().map_err(store_error)? != *intent
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
        return Ok(());
    }
    let id = pending.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(pending).map_err(store_error)?;
    batch
        .add_catalog_mutation(CatalogMutation::set(
            catalog,
            path,
            CatalogExpectation::Absent,
            id,
        ))
        .map_err(store_error)?;
    store.commit(batch).map_err(|_| unknown())?;
    Ok(())
}

pub(super) fn read_receipt(
    store: &Store,
    request: &LineageReadRequest,
) -> ServiceResult<BuildReceipt> {
    recorded(store, request, true).map(|(receipt, _)| receipt)
}
fn recorded(
    store: &Store,
    request: &LineageReadRequest,
    check_snapshot: bool,
) -> ServiceResult<(BuildReceipt, Box<[u8]>)> {
    let operation =
        wow_store::OperationId::new(request.build_operation_id.clone()).map_err(store_error)?;
    let record = store
        .operation(&operation)
        .map_err(store_error)?
        .ok_or_else(|| failure(ServiceErrorCode::ExactGenerationUnavailable))?;
    if record.request_digest() != &request.build_request_digest
        || record.state() != OperationState::Completed
    {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    let object = store
        .object(record.result_object_id().ok_or_else(unknown)?)
        .map_err(store_error)?
        .ok_or_else(unknown)?;
    if object.kind() != receipt_kind(request.source_selection.is_some())
        || object.schema_version() != 1
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let receipt: BuildReceipt = object.decode().map_err(store_error)?;
    if receipt.schema.as_ref() != receipt_schema(request.source_selection.is_some())
        || receipt.operation.as_ref() != "lineage_build"
        || receipt.request.operation_id != request.build_operation_id
        || receipt.request_digest != request.build_request_digest
        || receipt.request.store_limits != request.store_limits
        || receipt.request.source_selection != request.source_selection
        || receipt.request.reference_selection != request.reference_selection
        || receipt.status.as_ref() != "candidate_only"
        || receipt.accepted_assertion_count != 0
        || receipt.promotion.as_ref() != "not_evaluated"
        || receipt.negative_authority
        || receipt.resource_closure.as_ref() != "closed"
        || (check_snapshot && receipt.snapshot.snapshot_id != request.snapshot_id)
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    Ok((receipt, object.canonical_json().into()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retirement {
    schema: Box<str>,
    operation_id: Box<str>,
    request_digest: RequestDigest,
    build_operation_id: Box<str>,
    snapshot_id: Box<str>,
    snapshot_object_id: ObjectId,
    status: Box<str>,
    resource_closure: Box<str>,
}
pub(super) fn retire(
    source: &mut crate::live_project::LiveProjectStore,
    store: &mut Store,
    request: &LineageReadRequest,
    operation: &OperationId,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let receipt = read_receipt(store, request)?;
    let consumer = store.store_instance_id().map_err(store_error)?.clone();
    receipt
        .source_holds
        .validate(source.retained_owner(), &consumer, stop)?;
    if build_digest(source, store, &receipt.request)? != receipt.request_digest {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let digest = digest(
        "wow-service/lineage-retire-request/1",
        &(
            operation.as_str(),
            request,
            &consumer,
            receipt.source_holds.source_epoch_id(),
        ),
    )?;
    let operation =
        wow_store::OperationId::new(operation.as_str().to_owned()).map_err(store_error)?;
    let result = Retirement {
        schema: chosen(
            request.source_selection.is_some(),
            "wow-service/lineage-retire-result/1",
            "wow-service/source-lineage-retire-result/1",
        )
        .into(),
        operation_id: operation.as_str().into(),
        request_digest: digest.clone(),
        build_operation_id: request.build_operation_id.clone(),
        snapshot_id: request.snapshot_id.clone(),
        snapshot_object_id: receipt.snapshot.object_id.clone(),
        status: "retired".into(),
        resource_closure: "closed".into(),
    };
    let bytes = encode(&result, request.max_output_bytes)?;
    let pending = PendingObject::from_json(
        "wow.service.lineage.retirement",
        1,
        &result,
        request.store_limits,
    )
    .map_err(store_error)?;
    let id = pending.object_id().clone();
    if let Some(record) = store.operation(&operation).map_err(store_error)? {
        if record.request_digest() != &digest {
            return Err(failure(ServiceErrorCode::OperationConflict));
        }
        if record.state() == OperationState::Completed {
            if record.result_object_id() != Some(&id)
                || store
                    .object(&id)
                    .map_err(store_error)?
                    .is_none_or(|o| o.canonical_json() != bytes.as_ref())
            {
                return Err(unknown());
            }
            receipt
                .source_holds
                .release(source.retained_owner_mut(), &consumer, stop)
                .map_err(|_| unknown())?;
            return Ok(bytes);
        }
        if record.state() != OperationState::Prepared {
            return Err(unknown());
        }
    } else {
        store
            .begin_operation(operation.clone(), digest.clone())
            .map_err(|_| unknown())?;
    }
    receipt
        .source_holds
        .verify(source.retained_owner(), &consumer, stop)?;
    let mut batch = open_lineage_store(store)
        .map_err(graph_error)?
        .plan_retire(&receipt.snapshot)
        .map_err(graph_error)?;
    batch.add_object(pending).map_err(store_error)?;
    checkpoint(stop)?;
    store
        .commit_operation(batch, &operation, &digest, &id)
        .map_err(|_| unknown())?;
    receipt
        .source_holds
        .release(source.retained_owner_mut(), &consumer, stop)
        .map_err(|_| unknown())?;
    Ok(bytes)
}
pub(super) fn digest<T: Serialize>(domain: &str, value: &T) -> ServiceResult<RequestDigest> {
    let hash = wow_core::domain_separated_digest(domain, value)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    RequestDigest::new(
        wow_core::ContentDigest::<wow_core::CanonicalResult>::from_bytes(hash).to_string(),
    )
    .map_err(store_error)
}

pub(super) fn abandon(
    source: &mut crate::live_project::LiveProjectStore,
    store: &mut Store,
    request: &LineageBuildRequest,
    operation: &OperationId,
    max_output_bytes: usize,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    checkpoint(stop)?;
    let original =
        wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
    let original_digest = build_digest(source, store, request)?;
    let record = store
        .operation(&original)
        .map_err(store_error)?
        .ok_or_else(|| failure(ServiceErrorCode::ExactGenerationUnavailable))?;
    if record.request_digest() != &original_digest {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    if !matches!(
        record.state(),
        OperationState::Prepared | OperationState::Failed
    ) {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    let operation =
        wow_store::OperationId::new(operation.as_str().to_owned()).map_err(store_error)?;
    if operation == original {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    let consumer = store.store_instance_id().map_err(store_error)?.clone();
    let hold = if request.source_selection.is_some() {
        SourceHoldPlan::prepare_source
    } else {
        SourceHoldPlan::prepare
    };
    let holds = hold(
        source.retained_owner(),
        &consumer,
        &OperationId::new(request.operation_id.clone())?,
        &original_digest,
        &StoreGenerationId::parse(request.before_generation.to_string()).map_err(store_error)?,
        &StoreGenerationId::parse(request.after_generation.to_string()).map_err(store_error)?,
        stop,
    )?;
    let intent = BuildIntent {
        schema: intent_schema(request.source_selection.is_some()).into(),
        request_digest: original_digest.clone(),
        request: request.clone(),
        source_holds: holds.clone(),
    };
    // This validates a surviving intent in full; an operation that stopped before
    // intent publication has dispatched no hold and can record the same plan.
    retain_intent(store, &original, &intent)?;
    let request_digest = digest(
        "wow-service/lineage-abandon-request/1",
        &(
            operation.as_str(),
            &original,
            &original_digest,
            &consumer,
            max_output_bytes,
        ),
    )?;
    #[derive(Serialize)]
    struct Result<'a> {
        schema: &'static str,
        operation_id: &'a str,
        request_digest: &'a RequestDigest,
        build_operation_id: &'a str,
        build_request_digest: &'a RequestDigest,
        status: &'static str,
        resource_closure: &'static str,
    }
    let result = Result {
        schema: chosen(
            request.source_selection.is_some(),
            "wow-service/lineage-abandon-result/1",
            "wow-service/source-lineage-abandon-result/1",
        ),
        operation_id: operation.as_str(),
        request_digest: &request_digest,
        build_operation_id: original.as_str(),
        build_request_digest: &original_digest,
        status: "abandoned",
        resource_closure: "closed",
    };
    let bytes = encode(&result, max_output_bytes)?;
    let pending = PendingObject::from_json(
        "wow.service.lineage.abandonment",
        1,
        &result,
        request.store_limits,
    )
    .map_err(store_error)?;
    let id = pending.object_id().clone();
    if let Some(record) = store.operation(&operation).map_err(store_error)? {
        if record.request_digest() != &request_digest {
            return Err(failure(ServiceErrorCode::OperationConflict));
        }
        if record.state() == OperationState::Completed {
            if record.result_object_id() != Some(&id)
                || store
                    .object(&id)
                    .map_err(store_error)?
                    .is_none_or(|o| o.canonical_json() != bytes.as_ref())
            {
                return Err(unknown());
            }
            let remaining = open_lineage_store(store)
                .map_err(graph_error)?
                .plan_abandon(original.as_str())
                .map_err(graph_error)?;
            if !remaining.catalog_mutations().is_empty() {
                return Err(unknown());
            }
            holds
                .release(source.retained_owner_mut(), &consumer, stop)
                .map_err(|_| unknown())?;
            return Ok(bytes);
        }
        if record.state() != OperationState::Prepared {
            return Err(unknown());
        }
    } else {
        store
            .begin_operation(operation.clone(), request_digest.clone())
            .map_err(|_| unknown())?;
    }
    // Terminal original state fences future build resumes before deleting any
    // inactive root. Failed means effects may exist; it never asserts no-effect.
    let mut batch = open_lineage_store(store)
        .map_err(graph_error)?
        .plan_abandon(original.as_str())
        .map_err(graph_error)?;
    store
        .record_failed(&original, &original_digest)
        .map_err(|_| unknown())?;
    batch.add_object(pending).map_err(store_error)?;
    checkpoint(stop)?;
    store
        .commit_operation(batch, &operation, &request_digest, &id)
        .map_err(|_| unknown())?;
    holds
        .release(source.retained_owner_mut(), &consumer, stop)
        .map_err(|_| unknown())?;
    Ok(bytes)
}
