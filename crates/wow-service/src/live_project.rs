//! Coherent native project/graph publication through the existing manifested
//! store. Current resolves once; actual replay and all owner checks hold its lease.
mod operations;
#[cfg(test)]
mod tests;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
pub use operations::{
    LiveProjectPublishRequest, LiveProjectResult, publish_local_project, read_live_project,
    reconcile_live_project,
};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair, ProjectPublicationBundle};
use wow_project::{ProjectPublisher, ProjectView};
use wow_store::project::{
    CurrentPublication, CurrentRecordId, ProjectStore, PublicationOperation, PublicationRequest,
    PublicationState, ReadSelector, ReadSnapshot, RecordCatalog,
};
use wow_store::{OperationId, StoreError, StoreErrorCode};

/// One writer for a separately registered live-project epoch. Retained graph
/// bundle epochs remain a distinct profile; this API never adopts or relabels them.
pub struct LiveProjectStore {
    store: ProjectStore,
}
/// Immutable executable pair plus the original transaction and generation lease.
pub struct LiveProjectRead {
    pair: AcquiredProjectPair,
    read: ReadSnapshot,
}
impl LiveProjectRead {
    pub fn project(&self) -> &ProjectView {
        self.pair.project()
    }
    pub fn graph(&self) -> &GraphPartitionSnapshot {
        self.pair.graph()
    }
    pub fn publication_set_id(&self) -> &str {
        self.pair.publication_set_id()
    }
    pub fn store_generation_id(&self) -> &wow_store::project::StoreGenerationId {
        &self.read.manifest().generation_id
    }
    pub fn current_at_acquisition(&self) -> Option<&CurrentPublication> {
        self.read.current_at_acquisition()
    }
}
impl LiveProjectStore {
    pub fn create(root: &Path, owner: &str) -> ServiceResult<Self> {
        Ok(Self {
            store: ProjectStore::create(root, owner, catalog()?).map_err(store_error)?,
        })
    }
    pub fn open(root: &Path) -> ServiceResult<Self> {
        // Both opens require an exact registered epoch. The store rejects a
        // catalog mismatch before opening SQLite writable; no migration occurs.
        let store = match ProjectStore::open(root, &catalog()?) {
            Ok(store) => store,
            Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {
                match ProjectStore::open(root, &catalog_for(publication::STORAGE_SCHEMAS_V2)?) {
                    Ok(store) => store,
                    Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {
                        ProjectStore::open(root, &catalog_for(publication::STORAGE_SCHEMAS_V1)?)
                            .map_err(store_error)?
                    }
                    Err(error) => return Err(store_error(error)),
                }
            }
            Err(error) => return Err(store_error(error)),
        };
        Ok(Self { store })
    }
    pub fn current(&self) -> ServiceResult<Option<CurrentPublication>> {
        self.store.current().map_err(store_error)
    }
    /// Compose from actual live owners, prepare inactive membership, read back
    /// under one snapshot, validate all owners, then CAS the expected current.
    pub fn publish(
        &mut self,
        publisher: &ProjectPublisher,
        graph: &GraphPartitionSnapshot,
        operation_id: &str,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
        if self.store.epoch().owner() != graph.snapshot().universe().as_str() {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let bundle =
            ProjectPublicationBundle::build(publisher, graph, stop).map_err(project_error)?;
        self.publish_bundle(bundle, operation_id, expected_current, stop)
    }
    fn publish_bundle(
        &mut self,
        bundle: ProjectPublicationBundle,
        operation_id: &str,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
        let (records, bindings) = bundle.into_parts();
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let request = PublicationRequest::new(
            self.store.epoch(),
            id.clone(),
            expected_current,
            bindings,
            records,
        )
        .map_err(store_error)?;
        let outcome = (|| {
            let operation = self.store.prepare(&request, stop).map_err(store_error)?;
            let read = self
                .store
                .read(
                    &ReadSelector::Exact(request.generation().generation_id.clone()),
                    stop,
                )
                .map_err(store_error)?;
            AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
            let validated = read
                .owner_validation(&[
                    GraphPartitionSnapshot::STORAGE_CHECK,
                    publication::STORAGE_CHECK,
                ])
                .map_err(store_error)?;
            drop(read);
            if operation.state == PublicationState::Activated {
                return Ok(operation);
            }
            self.store
                .validate_inactive(&id, request.request_digest(), validated, stop)
                .map_err(store_error)?;
            self.store
                .activate(&id, request.request_digest(), stop)
                .map_err(store_error)
        })();
        match outcome {
            Ok(operation) => Ok(operation),
            Err(error) => {
                // Observe an uncertain commit once. Never retry effects or
                // downgrade an exact committed activation to cancellation.
                if let Ok(Some(operation)) = self.store.reconcile(&id)
                    && operation.request_digest == request.request_digest()
                    && operation.state == PublicationState::Activated
                {
                    return Ok(operation);
                }
                Err(error)
            }
        }
    }
    pub fn read(
        &self,
        selector: &ReadSelector,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectRead> {
        let read = self.store.read(selector, stop).map_err(store_error)?;
        let pair = AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
        Ok(LiveProjectRead { pair, read })
    }
    pub fn reconcile(&self, operation_id: &str) -> ServiceResult<Option<PublicationOperation>> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store.reconcile(&id).map_err(store_error)
    }
}
fn catalog() -> ServiceResult<RecordCatalog> {
    catalog_for(publication::STORAGE_SCHEMAS)
}
fn catalog_for(project_schemas: &[&'static str]) -> ServiceResult<RecordCatalog> {
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(project_schemas);
    RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )
    .map_err(store_error)
}
pub(crate) fn project_error(error: wow_project::ProjectError) -> ServiceError {
    fail(match error.code() {
        wow_project::ProjectErrorCode::AnalysisCancelled => ServiceErrorCode::Cancelled,
        wow_project::ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        wow_project::ProjectErrorCode::DeferredCapability => {
            ServiceErrorCode::OperationNotImplementedForMilestone
        }
        _ => ServiceErrorCode::IdentityMismatch,
    })
}
fn store_error(error: StoreError) -> ServiceError {
    fail(match error.code() {
        StoreErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        StoreErrorCode::CurrentConflict => ServiceErrorCode::StoreCurrentConflict,
        StoreErrorCode::OperationConflict => ServiceErrorCode::OperationConflict,
        StoreErrorCode::OutcomeUnknown => ServiceErrorCode::StoreOutcomeUnknown,
        StoreErrorCode::WriterBusy => ServiceErrorCode::OperationBusy,
        StoreErrorCode::GenerationMissing => ServiceErrorCode::ExactGenerationUnavailable,
        StoreErrorCode::BudgetExceeded | StoreErrorCode::ObjectTooLarge => {
            ServiceErrorCode::BudgetExceeded
        }
        StoreErrorCode::IntegrityViolation => ServiceErrorCode::IdentityMismatch,
        _ => ServiceErrorCode::ComponentUnavailable,
    })
}
fn fail(code: ServiceErrorCode) -> ServiceError {
    ServiceError::new(
        code,
        "native live project pair is unavailable; inspect its typed outcome",
    )
}
fn publication_checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(std::sync::atomic::Ordering::Acquire) {
        Err(fail(ServiceErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
