//! Exact native Project -> retained Candidate lineage orchestration.
//! Every durable effect uses the existing typed Store/retention owners.
mod source_holds;
#[cfg(test)]
mod tests;
mod workflow;
use crate::{OperationId, ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use source_holds::SourceHoldPlan;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
pub use wow_graph::lineage::LineageLimits;
use wow_graph::lineage::{PersistentLineageStore, PreparedLineageSnapshot, SealedLineageHandle};
pub use wow_store::StoreLimits as LineageStoreLimits;
use wow_store::project::StoreGenerationId;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, ObjectId, OperationState,
    PendingObject, RequestDigest, Store, StoreConfiguration, WriteBatch,
};

/// Closed, exact-generation request. Physical roots are supplied separately and
/// never enter semantic output or source-fingerprint identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageBuildRequest {
    pub schema: Box<str>,
    pub operation_id: Box<str>,
    pub before_generation: Box<str>,
    pub after_generation: Box<str>,
    pub limits: LineageLimits,
    pub max_source_bytes: usize,
    pub store_limits: LineageStoreLimits,
    pub max_output_bytes: usize,
}
impl LineageBuildRequest {
    fn validate(&self) -> ServiceResult<()> {
        if self.schema.as_ref() != "wow-service/lineage-build/1"
            || self.max_source_bytes == 0
            || self.max_source_bytes > 64 * 1024 * 1024
            || self.max_output_bytes == 0
            || self.max_output_bytes > 64 * 1024 * 1024
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        OperationId::new(self.operation_id.clone())?;
        StoreGenerationId::parse(self.before_generation.to_string()).map_err(store_error)?;
        StoreGenerationId::parse(self.after_generation.to_string()).map_err(store_error)?;
        self.limits.validate().map_err(graph_error)?;
        configuration(self.store_limits)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineageReadRequest {
    pub schema: Box<str>,
    pub build_operation_id: Box<str>,
    pub build_request_digest: RequestDigest,
    pub snapshot_id: Box<str>,
    pub store_limits: LineageStoreLimits,
    pub max_output_bytes: usize,
}
impl LineageReadRequest {
    fn validate(&self) -> ServiceResult<()> {
        if self.schema.as_ref() != "wow-service/lineage-read/1"
            || self.max_output_bytes == 0
            || self.max_output_bytes > 64 * 1024 * 1024
            || self.snapshot_id.is_empty()
            || self.snapshot_id.len() > 256
        {
            return Err(failure(ServiceErrorCode::InvalidRequest));
        }
        OperationId::new(self.build_operation_id.clone())?;
        configuration(self.store_limits)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BuildReceipt {
    schema: Box<str>,
    operation: Box<str>,
    request_digest: RequestDigest,
    request: LineageBuildRequest,
    source_holds: SourceHoldPlan,
    snapshot: SealedLineageHandle,
    status: Box<str>,
    resource_closure: Box<str>,
    accepted_assertion_count: usize,
    promotion: Box<str>,
    negative_authority: bool,
}

/// Publishes one separate inactive immutable overlay and admits it only after
/// native qualification, fresh physical readback and exact source retention.
pub fn build_lineage_snapshot(
    source_root: &Path,
    lineage_store: &Path,
    request: &LineageBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    resources(
        source_root,
        lineage_store,
        request.store_limits,
        true,
        stop,
        |source, slot| workflow::build(lineage_store, source, slot, request, stop),
    )
}

/// Returns the exact stored candidate comparison after native owner and catalog
/// validation. A new observation never reselects Current or repairs a snapshot.
pub fn compare_lineage_snapshot(
    source_root: &Path,
    lineage_store: &Path,
    request: &LineageReadRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    resources(
        source_root,
        lineage_store,
        request.store_limits,
        false,
        stop,
        |source, slot| {
            let store = current(slot)?;
            let receipt = workflow::read_receipt(store, request)?;
            let expected = qualify(source, store, &receipt, stop)?;
            let bytes = PersistentLineageStore::new(store)
                .map_err(graph_error)?
                .read_sealed(&receipt.snapshot, &expected, stop)
                .map_err(graph_error)?;
            bounded(bytes, request.max_output_bytes)
        },
    )
}

/// Retires exact snapshot roots before releasing its two independent original
/// generation holds. A retry resumes the recorded retirement, never rebuilds.
pub fn retire_lineage_snapshot(
    source_root: &Path,
    lineage_store: &Path,
    request: &LineageReadRequest,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    let operation_id = OperationId::new(operation_id.to_owned())?;
    resources(
        source_root,
        lineage_store,
        request.store_limits,
        false,
        stop,
        |source, slot| workflow::retire(source, current(slot)?, request, &operation_id, stop),
    )
}

/// Explicitly stops one exact unfinished build and releases its source holds
/// only after its inactive target is removed and sealed publication is excluded.
pub fn abandon_lineage_snapshot(
    source_root: &Path,
    lineage_store: &Path,
    request: &LineageBuildRequest,
    operation_id: &str,
    max_output_bytes: usize,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    request.validate()?;
    if max_output_bytes == 0 || max_output_bytes > 64 * 1024 * 1024 {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let operation_id = OperationId::new(operation_id.to_owned())?;
    resources(
        source_root,
        lineage_store,
        request.store_limits,
        false,
        stop,
        |source, slot| {
            workflow::abandon(
                source,
                current(slot)?,
                request,
                &operation_id,
                max_output_bytes,
                stop,
            )
        },
    )
}

fn prepare(
    request: &LineageBuildRequest,
    source: &crate::live_project::LiveProjectStore,
    stop: &AtomicBool,
) -> ServiceResult<PreparedLineageSnapshot> {
    let pair_request = crate::live_project::ProjectGenerationPairRequest::new(
        &request.before_generation,
        &request.after_generation,
        1,
    )?;
    crate::live_project::with_native_generation_pair(
        source,
        &pair_request,
        stop,
        &std::cell::Cell::new("pending"),
        |before, after| {
            let pair = wow_project::replay::publication::RetainedGenerationPair::bind(
                before.pair(),
                after.pair(),
                before.custody(),
                after.custody(),
                stop,
            )
            .map_err(crate::live_project::project_error)?;
            let fingerprint_limits =
                wow_project::graph::fingerprints::FunctionFingerprintLimits::new(
                    request.limits.max_entities,
                    request.max_source_bytes,
                    request.limits.max_metadata_bytes,
                )
                .map_err(crate::live_project::project_error)?;
            let input =
                wow_project::graph::lineage_input::build_project_source_fingerprint_partition(
                    &pair,
                    fingerprint_limits,
                    request.limits,
                    stop,
                )
                .map_err(crate::live_project::project_error)?;
            if input.data().binding.before.store_epoch_id.as_ref()
                != source.retained_owner().epoch().epoch_id().as_str()
                || input.data().binding.after.store_epoch_id.as_ref()
                    != source.retained_owner().epoch().epoch_id().as_str()
            {
                return Err(failure(ServiceErrorCode::IdentityMismatch));
            }
            PreparedLineageSnapshot::build(input, stop).map_err(graph_error)
        },
    )
}
fn qualify(
    source: &crate::live_project::LiveProjectStore,
    store: &Store,
    receipt: &BuildReceipt,
    stop: &AtomicBool,
) -> ServiceResult<PreparedLineageSnapshot> {
    receipt.request.validate()?;
    receipt.source_holds.verify(
        source.retained_owner(),
        store.store_instance_id().map_err(store_error)?,
        stop,
    )?;
    if receipt.source_holds.source_epoch_id() != source.retained_owner().epoch().epoch_id()
        || receipt.source_holds.before_generation().as_str()
            != receipt.request.before_generation.as_ref()
        || receipt.source_holds.after_generation().as_str()
            != receipt.request.after_generation.as_ref()
        || workflow::build_digest(source, store, &receipt.request)? != receipt.request_digest
    {
        return Err(failure(ServiceErrorCode::IdentityMismatch));
    }
    prepare(&receipt.request, source, stop)
}
fn resources<T>(
    source_root: &Path,
    lineage_store: &Path,
    limits: LineageStoreLimits,
    create: bool,
    stop: &AtomicBool,
    run: impl FnOnce(&mut crate::live_project::LiveProjectStore, &mut Option<Store>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    checkpoint(stop)?;
    if !source_root.is_absolute()
        || !lineage_store.is_absolute()
        || source_root.as_os_str().len() > 32768
        || lineage_store.as_os_str().len() > 32768
    {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let mut source = crate::live_project::LiveProjectStore::open(source_root)?;
    let opened = if create {
        Store::open(lineage_store, configuration(limits)?)
    } else {
        Store::open_existing(lineage_store, configuration(limits)?)
    };
    let mut slot = match opened {
        Ok(store) => Some(store),
        Err(error) => {
            return match source.close() {
                Ok(()) => Err(store_error(error)),
                Err(_) => Err(unknown()),
            };
        }
    };
    let result = run(&mut source, &mut slot);
    let consumer_closed = slot.take().map_or(Ok(()), Store::close);
    let source_closed = source.close();
    if consumer_closed.is_err() || source_closed.is_err() {
        return Err(unknown());
    }
    result
}
fn reopen(slot: &mut Option<Store>, path: &Path, limits: LineageStoreLimits) -> ServiceResult<()> {
    let original = current(slot)?
        .store_instance_id()
        .map_err(store_error)?
        .clone();
    slot.take()
        .ok_or_else(unknown)?
        .close()
        .map_err(|_| unknown())?;
    *slot = Some(Store::open_existing(path, configuration(limits)?).map_err(|_| unknown())?);
    if current(slot)?.store_instance_id().map_err(store_error)? != &original {
        return Err(unknown());
    }
    Ok(())
}
fn current(slot: &mut Option<Store>) -> ServiceResult<&mut Store> {
    slot.as_mut().ok_or_else(unknown)
}
fn configuration(limits: LineageStoreLimits) -> ServiceResult<StoreConfiguration> {
    StoreConfiguration::new_with_instance_identity(
        wow_graph::lineage::LINEAGE_STORE_PROFILE,
        limits,
    )
    .map_err(store_error)
}
fn bounded(bytes: Box<[u8]>, limit: usize) -> ServiceResult<Box<[u8]>> {
    if bytes.len().checked_add(1).is_none_or(|n| n > limit) {
        return Err(failure(ServiceErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
fn encode<T: Serialize>(value: &T, limit: usize) -> ServiceResult<Box<[u8]>> {
    // All records are already charged by their owners or bounded Store objects.
    let bytes = wow_core::canonical_json_bytes(value)
        .map_err(|_| failure(ServiceErrorCode::CanonicalizationFailed))?;
    bounded(bytes.into_boxed_slice(), limit)
}
fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn graph_error(error: wow_graph::GraphError) -> ServiceError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::IdentityMismatch,
    })
}
fn store_error(error: wow_store::StoreError) -> ServiceError {
    crate::live_project::store_error(error)
}
fn unknown() -> ServiceError {
    failure(ServiceErrorCode::StoreOutcomeUnknown)
}
fn failure(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "exact Candidate lineage operation is unavailable or inconsistent",
    )
}
