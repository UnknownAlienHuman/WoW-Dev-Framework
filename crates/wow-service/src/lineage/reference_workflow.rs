//! Two genuine Reference carrier epochs; generation-local universes stay intact.
use super::reference_sources::{ReferenceHoldPlan, ReferenceSources};
use super::workflow::digest;
use super::*;
mod terminal;
pub use terminal::{abandon_reference_lineage_snapshot, retire_reference_lineage_snapshot};
mod read;
pub use read::{compare_reference_lineage_snapshot, query_reference_lineage_snapshot};

const BUILD_KIND: &str = "wow.service.reference_lineage.build";
const INTENT_KIND: &str = "wow.service.reference_lineage.intent";
const INTENTS: &str = "reference-lineage.intents";
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: Box<str>,
    request_digest: RequestDigest,
    request: LineageBuildRequest,
    holds: ReferenceHoldPlan,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    schema: Box<str>,
    request_digest: RequestDigest,
    request: LineageBuildRequest,
    holds: ReferenceHoldPlan,
    snapshot: SealedLineageHandle,
    reference_comparison: serde_json::Value,
    status: Box<str>,
    resource_closure: Box<str>,
    accepted_assertion_count: usize,
    negative_authority: bool,
}
fn require_reference(request: &LineageBuildRequest) -> ServiceResult<()> {
    request.validate()?;
    if request.reference_selection.is_none() || request.source_selection.is_some() {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    Ok(())
}
fn resources<T>(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    limits: LineageStoreLimits,
    create: bool,
    stop: &AtomicBool,
    run: impl FnOnce(&mut ReferenceSources, &mut Option<Store>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    checkpoint(stop)?;
    if !path.is_absolute() || path.as_os_str().len() > 32768 {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let configuration = StoreConfiguration::new_with_instance_identity(
        wow_graph::lineage::REFERENCE_LINEAGE_STORE_PROFILE,
        limits,
    )
    .map_err(store_error)?;
    let mut sources = ReferenceSources::open(roots)?;
    let opened = if create {
        Store::open(path, configuration)
    } else {
        Store::open_existing(path, configuration)
    };
    let mut slot = match opened {
        Ok(store) => Some(store),
        Err(error) => {
            return match sources.close() {
                Ok(()) => Err(store_error(error)),
                Err(_) => Err(unknown()),
            };
        }
    };
    let draft = run(&mut sources, &mut slot);
    let consumer_closed = slot.take().map_or(Ok(()), Store::close);
    let source_closed = sources.close();
    if consumer_closed.is_err() || source_closed.is_err() {
        return Err(unknown());
    }
    checkpoint(stop)?;
    draft
}
fn request_digest(
    sources: &ReferenceSources,
    store: &Store,
    request: &LineageBuildRequest,
) -> ServiceResult<RequestDigest> {
    digest(
        "wow-service/reference-lineage-build-request/1",
        &(
            sources.before.retained_owner().epoch().epoch_id(),
            sources.after.retained_owner().epoch().epoch_id(),
            store.store_instance_id().map_err(store_error)?,
            request,
        ),
    )
}
fn prepare(
    sources: &ReferenceSources,
    request: &LineageBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<(PreparedLineageSnapshot, serde_json::Value)> {
    sources.with_pair(request, stop, |before, after| {
        let selected = request
            .reference_selection
            .as_ref()
            .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
        let qualified = reference::build_input(before, after, selected, request.limits, stop)?;
        let comparison = serde_json::to_value(&qualified.comparison)
            .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
        let expected =
            PreparedLineageSnapshot::build(qualified.input, stop).map_err(graph_error)?;
        reference::validate_capture(Some(&comparison), &expected.input().binding)?;
        Ok((expected, comparison))
    })
}
fn retain_intent(
    store: &mut Store,
    operation: &wow_store::OperationId,
    intent: &Intent,
) -> ServiceResult<()> {
    let pending = PendingObject::from_json(INTENT_KIND, 1, intent, store.configuration().limits())
        .map_err(store_error)?;
    let catalog = CatalogName::new(INTENTS).map_err(store_error)?;
    let path = CatalogPath::new(operation.as_str()).map_err(store_error)?;
    if let Some(entry) = store.catalog_entry(&catalog, &path).map_err(store_error)? {
        let object = store
            .object(entry.object_id())
            .map_err(store_error)?
            .ok_or_else(unknown)?;
        if object.object_id() != pending.object_id()
            || object.kind() != INTENT_KIND
            || object.decode::<Intent>().map_err(store_error)? != *intent
        {
            return Err(failure(ServiceErrorCode::IdentityMismatch));
        }
    } else {
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
    }
    Ok(())
}
fn read_receipt(
    store: &Store,
    request: &LineageReadRequest,
) -> ServiceResult<(Receipt, Box<[u8]>)> {
    request.validate()?;
    if request.reference_selection.is_none() {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let operation =
        wow_store::OperationId::new(request.build_operation_id.clone()).map_err(store_error)?;
    let record = store
        .operation(&operation)
        .map_err(store_error)?
        .ok_or_else(|| failure(ServiceErrorCode::OperationReleased))?;
    if record.state() != OperationState::Completed
        || record.request_digest() != &request.build_request_digest
    {
        return Err(failure(ServiceErrorCode::OperationConflict));
    }
    let object = store
        .object(record.result_object_id().ok_or_else(unknown)?)
        .map_err(store_error)?
        .ok_or_else(unknown)?;
    if object.kind() != BUILD_KIND || object.schema_version() != 1 {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    let receipt: Receipt = object.decode().map_err(store_error)?;
    if receipt.schema.as_ref() != "wow-service/reference-lineage-build-result/1"
        || receipt.request_digest != request.build_request_digest
        || receipt.request.operation_id != request.build_operation_id
        || receipt.request.reference_selection != request.reference_selection
        || receipt.request.store_limits != request.store_limits
        || receipt.snapshot.snapshot_id != request.snapshot_id
        || receipt.status.as_ref() != "candidate_only"
        || receipt.resource_closure.as_ref() != "closed"
        || receipt.accepted_assertion_count != 0
        || receipt.negative_authority
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    require_reference(&receipt.request)?;
    Ok((receipt, object.canonical_json().into()))
}
fn qualify(
    sources: &ReferenceSources,
    store: &Store,
    receipt: &Receipt,
    stop: &AtomicBool,
) -> ServiceResult<PreparedLineageSnapshot> {
    if request_digest(sources, store, &receipt.request)? != receipt.request_digest {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    receipt.holds.verify(
        sources,
        store.store_instance_id().map_err(store_error)?,
        &receipt.request,
        stop,
    )?;
    let (expected, comparison) = prepare(sources, &receipt.request, stop)?;
    if comparison != receipt.reference_comparison {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    Ok(expected)
}
pub fn build_reference_lineage_snapshot(
    roots: ReferenceLineageRoots<'_>,
    path: &Path,
    request: &LineageBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    require_reference(request)?;
    resources(
        roots,
        path,
        request.store_limits,
        true,
        stop,
        |sources, slot| {
            let operation =
                wow_store::OperationId::new(request.operation_id.clone()).map_err(store_error)?;
            let hash = request_digest(sources, current(slot)?, request)?;
            if let Some(record) = current(slot)?.operation(&operation).map_err(store_error)? {
                if record.request_digest() != &hash {
                    return Err(failure(ServiceErrorCode::OperationConflict));
                }
                if record.state() == OperationState::Completed {
                    let object = current(slot)?
                        .object(record.result_object_id().ok_or_else(unknown)?)
                        .map_err(store_error)?
                        .ok_or_else(unknown)?;
                    let receipt: Receipt = object.decode().map_err(store_error)?;
                    let selector = LineageReadRequest {
                        schema: "wow-service/reference-lineage-read/1".into(),
                        build_operation_id: request.operation_id.clone(),
                        build_request_digest: hash.clone(),
                        snapshot_id: receipt.snapshot.snapshot_id.clone(),
                        store_limits: request.store_limits,
                        max_output_bytes: request.max_output_bytes,
                        source_selection: None,
                        reference_selection: request.reference_selection.clone(),
                    };
                    let (receipt, bytes) = read_receipt(current(slot)?, &selector)?;
                    let expected = qualify(sources, current(slot)?, &receipt, stop)?;
                    open_lineage_store(current(slot)?)
                        .map_err(graph_error)?
                        .validate_sealed(&receipt.snapshot, &expected, stop)
                        .map_err(graph_error)?;
                    return bounded(bytes, request.max_output_bytes);
                }
                if record.state() != OperationState::Prepared {
                    return Err(failure(ServiceErrorCode::OperationReleased));
                }
            } else {
                current(slot)?
                    .begin_operation(operation.clone(), hash.clone())
                    .map_err(store_error)?;
            }
            let consumer = current(slot)?
                .store_instance_id()
                .map_err(store_error)?
                .clone();
            let holds = ReferenceHoldPlan::prepare(sources, &consumer, request, &hash, stop)?;
            let intent = Intent {
                schema: "wow-service/reference-lineage-build-intent/1".into(),
                request_digest: hash.clone(),
                request: request.clone(),
                holds: holds.clone(),
            };
            retain_intent(current(slot)?, &operation, &intent)?;
            #[cfg(test)]
            eprintln!("reference-build stage=intent-retained");
            holds.install(sources, &consumer, request, stop)?;
            #[cfg(test)]
            eprintln!("reference-build stage=holds-installed");
            let (expected, reference_comparison) = prepare(sources, request, stop)?;
            #[cfg(test)]
            eprintln!("reference-build stage=native-prepared");
            let stored = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .store_inactive(operation.as_str(), &expected, &operation, &hash, stop)
                .map_err(graph_error)?;
            reopen(slot, path, request.store_limits)?;
            let validated = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .validate_inactive(stored, &expected, stop)
                .map_err(graph_error)?;
            holds.verify(sources, &consumer, request, stop)?;
            let plan = open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .plan_seal(validated, stop)
                .map_err(graph_error)?;
            let receipt = Receipt {
                schema: "wow-service/reference-lineage-build-result/1".into(),
                request_digest: hash.clone(),
                request: request.clone(),
                holds,
                snapshot: plan.handle,
                reference_comparison,
                status: "candidate_only".into(),
                resource_closure: "closed".into(),
                accepted_assertion_count: 0,
                negative_authority: false,
            };
            let bytes = encode(&receipt, request.max_output_bytes)?;
            let pending = PendingObject::from_json(BUILD_KIND, 1, &receipt, request.store_limits)
                .map_err(store_error)?;
            let id = pending.object_id().clone();
            let mut batch = plan.batch;
            batch.add_object(pending).map_err(store_error)?;
            checkpoint(stop)?;
            current(slot)?
                .commit_operation(batch, &operation, &hash, &id)
                .map_err(|_| unknown())?;
            reopen(slot, path, request.store_limits)?;
            open_lineage_store(current(slot)?)
                .map_err(graph_error)?
                .validate_sealed(&receipt.snapshot, &expected, stop)
                .map_err(graph_error)?;
            receipt.holds.verify(sources, &consumer, request, stop)?;
            Ok(bytes)
        },
    )
}
