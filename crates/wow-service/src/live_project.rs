//! Coherent native project/graph publication through the existing manifested
//! store. Current resolves once; actual replay and all owner checks hold its lease.
mod current_recovery;
mod gc;
pub use current_recovery::{
    CurrentDomainFailure, CurrentDomainIds, CurrentDomainObservation, CurrentDomainPhase,
    LiveProjectRecoveryObservation, recover_current_live_project,
};
mod migration;
pub use migration::{
    export_live_project_migration, migrate_live_project_to_new, prepare_live_project_migration,
    resume_live_project_migration, resume_live_project_migration_preparation,
};
mod namespace;
pub use namespace::PlatformStoreSelection;
mod operations;
mod quarantine;
pub use quarantine::{LiveProjectQuarantineInspection, QuarantinedLiveProject};
mod recovery;
pub use recovery::{
    CurrentState, RecoveryReport, ScopeState, recover_live_project, restore_live_project_to_new,
};
mod retention;
#[cfg(test)]
mod tests;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
pub use operations::{
    LiveProjectLibraryMode, LiveProjectPublishRequest, LiveProjectResult, LiveProjectUpdateRequest,
    publish_input_in_namespace, publish_local_project, read_live_project,
    read_live_project_in_namespace, reconcile_live_project, reconcile_live_project_in_namespace,
    update_input_in_namespace, update_local_project,
};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair, ProjectPublicationBundle};
use wow_project::{ProjectPublisher, ProjectView};
use wow_store::project::{
    CurrentPublication, CurrentRecordId, ProjectStore, ProjectStoreNamespace, PublicationOperation,
    PublicationRequest, PublicationState, ReadSelector, ReadSnapshot, RecordCatalog,
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
    pub fn namespace(&self) -> Option<&ProjectStoreNamespace> {
        self.read.epoch().namespace()
    }
    fn into_update_publisher(self) -> ServiceResult<(ProjectPublisher, ReadSnapshot)> {
        let publisher = self.pair.into_update_publisher().map_err(project_error)?;
        Ok((publisher, self.read))
    }
}
impl LiveProjectStore {
    pub fn create(root: &Path, owner: &str) -> ServiceResult<Self> {
        Ok(Self {
            store: ProjectStore::create_with_gc(root, owner, catalog()?).map_err(store_error)?,
        })
    }
    pub fn open(root: &Path) -> ServiceResult<Self> {
        // Each open requires an exact registered epoch. The store rejects a
        // catalog mismatch before opening SQLite writable; no migration occurs.
        for schemas in [
            publication::STORAGE_SCHEMAS,
            publication::STORAGE_SCHEMAS_V7,
            publication::STORAGE_SCHEMAS_V6,
            publication::STORAGE_SCHEMAS_V5,
            publication::STORAGE_SCHEMAS_V4,
            publication::STORAGE_SCHEMAS_V3,
            publication::STORAGE_SCHEMAS_V2,
            publication::STORAGE_SCHEMAS_V1,
        ] {
            match ProjectStore::open(root, &catalog_for(schemas)?) {
                Ok(store) => return Ok(Self { store }),
                Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {}
                Err(error) => return Err(store_error(error)),
            }
        }
        Err(fail(ServiceErrorCode::IdentityMismatch))
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
        let bundle = match self.store.epoch().namespace() {
            Some(namespace) => {
                ProjectPublicationBundle::build_in_namespace(publisher, graph, namespace, stop)
            }
            None => {
                if self.store.epoch().owner() != graph.snapshot().universe().as_str() {
                    return Err(fail(ServiceErrorCode::IdentityMismatch));
                }
                ProjectPublicationBundle::build(publisher, graph, stop)
            }
        }
        .map_err(project_error)?;
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
        if let Some(operation) = self.store.operation(&id).map_err(store_error)?
            && operation.release.is_some()
        {
            return Err(fail(
                if operation.request_digest == request.request_digest() {
                    ServiceErrorCode::OperationReleased
                } else {
                    ServiceErrorCode::OperationConflict
                },
            ));
        }
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
                    && operation.release.is_none()
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
        wow_project::ProjectErrorCode::StoreReadUnavailable => {
            ServiceErrorCode::ComponentUnavailable
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
        StoreErrorCode::Quarantined => ServiceErrorCode::StoreQuarantined,
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
