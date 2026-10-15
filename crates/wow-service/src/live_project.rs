//! Native project/graph and explicitly selected candidate publication through
//! the existing manifested store. Current resolves once; replay, candidate and
//! Skeleton reads retain the exact generation lease through owner validation.
pub(crate) mod candidate;
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
mod selection;
pub(crate) use selection::SelectionDocument;
mod skeleton;
pub use selection::{
    PLATFORM_PROJECT_SELECTION_MAX_BYTES, PLATFORM_PROJECT_SELECTION_SCHEMA,
    PlatformProjectSelection, SelectedLiveProjectReadResult, publish_selected_local_project,
    read_selected_live_project, update_selected_local_project,
};
pub use skeleton::{
    LIVE_SKELETON_REQUEST_MAX_BYTES, LIVE_SKELETON_REQUEST_SCHEMA, LIVE_SKELETON_RESULT_SCHEMA,
    LiveSkeletonReadResult, execute_live_skeleton_read,
};
mod quarantine;
pub use quarantine::{LiveProjectQuarantineInspection, QuarantinedLiveProject};
mod recovery;
pub use recovery::{
    CurrentState, RecoveryReport, ScopeState, recover_live_project, restore_live_project_to_new,
};
mod retention;
#[cfg(test)]
pub(crate) mod tests;
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
pub use operations::{
    LiveProjectLibraryMode, LiveProjectPublishRequest, LiveProjectResult, LiveProjectUpdateRequest,
    publish_input_in_namespace, publish_local_project, read_live_project,
    read_live_project_in_namespace, reconcile_live_project, reconcile_live_project_in_namespace,
    update_input_in_namespace, update_local_project,
};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_emmy::function_calls::{
    PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
};
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
    candidate: Option<crate::graph::BlizzardUiIndexCandidate>,
    native_api: Option<crate::graph::NativeApiProvenance>,
    read: ReadSnapshot,
}
/// Checked reconstruction unwind is distinct from native acquisition failures,
/// whose lower-level close outcome is not exposed by the current store API.
pub(crate) enum ReadFailureClosure {
    Closed,
    Failed,
    NotEvaluated,
}
pub(crate) struct LiveProjectReadFailure {
    pub error: ServiceError,
    pub closure: ReadFailureClosure,
}
impl LiveProjectRead {
    /// Release retained domain owners before checking transaction/lease closure.
    pub fn close(self) -> ServiceResult<()> {
        let Self {
            pair,
            candidate,
            native_api,
            read,
        } = self;
        drop(native_api);
        drop(candidate);
        drop(pair);
        read.close().map_err(store_error)
    }
    /// Genuine immutable pair held by this exact store read and generation lease.
    pub fn pair(&self) -> &AcquiredProjectPair {
        &self.pair
    }
    /// Later Skeleton owners consume this genuine sealed candidate, never a DTO.
    pub fn candidate(&self) -> Option<&crate::graph::BlizzardUiIndexCandidate> {
        self.candidate.as_ref()
    }
    pub(crate) fn native_api(&self) -> Option<&crate::graph::NativeApiProvenance> {
        self.native_api.as_ref()
    }
    pub fn project(&self) -> &ProjectView {
        self.pair.project()
    }
    /// Re-run the native producers over exact retained inputs after publication.
    /// No cached report or generic graph proposal grants native callable authority.
    pub fn requalify_native_library(
        &self,
        stop: &AtomicBool,
    ) -> ServiceResult<crate::local::RequalifiedNativeLibrary> {
        crate::local::native_library_input::rebuild_for_project(self.project(), stop)
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
    // Only a selected, rebuilt candidate can enter the native update seam.
    // Keep its original exact lease; the receipt is comparison data, not authority.
    fn into_candidate_update_publisher(
        self,
        stop: &AtomicBool,
    ) -> ServiceResult<(
        ProjectPublisher,
        ReadSnapshot,
        wow_store::project::PartitionRecord,
    )> {
        publication_checkpoint(stop)?;
        let Self {
            pair,
            candidate,
            native_api,
            read,
        } = self;
        drop(native_api);
        let candidate = candidate.ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?;
        let receipt = crate::graph::candidate_storage_record(&candidate, stop)?;
        let publisher = pair
            .into_platform_update_publisher(candidate.native(), stop)
            .map_err(project_error)?;
        drop(candidate);
        publication_checkpoint(stop)?;
        Ok((publisher, read, receipt))
    }
    fn into_update_publisher(self) -> ServiceResult<(ProjectPublisher, ReadSnapshot)> {
        let Self {
            pair,
            candidate,
            native_api,
            read,
        } = self;
        drop(native_api);
        drop(candidate);
        let publisher = pair.into_update_publisher().map_err(project_error)?;
        Ok((publisher, read))
    }
}
impl LiveProjectStore {
    /// Check native store closure without revoking another live reader.
    pub fn close(self) -> ServiceResult<()> {
        self.store.close().map_err(store_error)
    }
    pub fn create(root: &Path, owner: &str) -> ServiceResult<Self> {
        Ok(Self {
            store: ProjectStore::create_with_gc(root, owner, catalog()?).map_err(store_error)?,
        })
    }
    // This ordinary recipe family never selects a platform or candidate namespace.
    fn create_primary_callables(root: &Path, owner: &str) -> ServiceResult<Self> {
        Ok(Self {
            store: ProjectStore::create_with_gc(
                root,
                owner,
                catalog_for(publication::STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1)?,
            )
            .map_err(store_error)?,
        })
    }
    fn create_native_library_input(root: &Path, owner: &str) -> ServiceResult<Self> {
        Ok(Self {
            store: ProjectStore::create_with_gc(
                root,
                owner,
                catalog_for(publication::STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1)?,
            )
            .map_err(store_error)?,
        })
    }
    pub fn open(root: &Path) -> ServiceResult<Self> {
        Self::open_selected(root, None, None)
    }
    /// Retained Service consumers share the same exact registered-catalog
    /// admission. Extracting this typed Store owner does not relabel its epoch.
    pub(crate) fn open_retained_owner_store(root: &Path) -> ServiceResult<ProjectStore> {
        Self::open(root).map(|owner| owner.store)
    }
    // None preserves general open/recovery across every registered catalog.
    // Old catalogs bind the XML/declaration pair; selected-component catalogs
    // retain exact recipes in configuration, validated on native pair readback.
    fn open_selected(
        root: &Path,
        components: Option<(
            bool,
            Option<PhysicalFunctionDeclarationRecipe>,
            Option<PhysicalFunctionSignatureRecipe>,
        )>,
        candidate_catalog: Option<bool>,
    ) -> ServiceResult<Self> {
        // Each open requires an exact registered epoch. The store rejects a
        // catalog mismatch before opening SQLite writable; no migration occurs.
        for &schemas in publication::STORAGE_SCHEMA_CATALOGS {
            if components.is_some_and(|selected| !catalog_accepts_components(schemas, selected)) {
                continue;
            }
            for selected in [false, true] {
                if selected
                    && (schemas == publication::STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1
                        || schemas == publication::STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1
                        || schemas == publication::STORAGE_SCHEMAS_NATIVE_API_USES_V1)
                {
                    continue;
                }
                if candidate_catalog.is_some_and(|expected| expected != selected) {
                    continue;
                }
                let catalog = if selected {
                    candidate::catalog(schemas)?
                } else {
                    catalog_for(schemas)?
                };
                match ProjectStore::open(root, &catalog) {
                    Ok(store) => return Ok(Self { store }),
                    Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {}
                    Err(error) => return Err(store_error(error)),
                }
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
        if candidate::is_selected(self.store.epoch().catalog())? {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
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
        if candidate::is_selected(self.store.epoch().catalog())? {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let (records, bindings) = bundle.into_parts();
        self.publish_parts(records, bindings, operation_id, expected_current, stop)
    }
    // One physical publication path for ordinary and explicitly selected owners.
    fn publish_parts(
        &mut self,
        records: Vec<wow_store::project::PartitionRecord>,
        bindings: std::collections::BTreeMap<String, String>,
        operation_id: &str,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
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
            let validation = (|| {
                let (pair, candidate) = candidate::read_owners(&read, stop)?;
                let native_api = crate::graph::validate_native_api_reopen(&pair, stop)?;
                let validated = read
                    .owner_validation(&candidate::completed_checks(candidate.is_some()))
                    .map_err(store_error);
                drop(native_api);
                drop(candidate);
                drop(pair);
                validated
            })();
            read.close().map_err(store_error)?;
            let validated = validation?;
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
        self.read_checked(selector, stop)
            .map_err(|failure| failure.error)
    }
    pub(crate) fn read_checked(
        &self,
        selector: &ReadSelector,
        stop: &AtomicBool,
    ) -> Result<LiveProjectRead, LiveProjectReadFailure> {
        let read = self
            .store
            .read(selector, stop)
            .map_err(|error| LiveProjectReadFailure {
                error: store_error(error),
                closure: ReadFailureClosure::NotEvaluated,
            })?;
        let qualified = candidate::read_owners(&read, stop).and_then(|(pair, candidate)| {
            let native_api = crate::graph::validate_native_api_reopen(&pair, stop)?;
            Ok((pair, candidate, native_api))
        });
        let (pair, candidate, native_api) = match qualified {
            Ok(owners) => owners,
            Err(error) => {
                return Err(match read.close() {
                    Ok(()) => LiveProjectReadFailure {
                        error,
                        closure: ReadFailureClosure::Closed,
                    },
                    Err(_) => LiveProjectReadFailure {
                        error: fail(ServiceErrorCode::InternalContractViolation),
                        closure: ReadFailureClosure::Failed,
                    },
                });
            }
        };
        Ok(LiveProjectRead {
            pair,
            candidate,
            native_api,
            read,
        })
    }
    pub fn reconcile(&self, operation_id: &str) -> ServiceResult<Option<PublicationOperation>> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store.reconcile(&id).map_err(store_error)
    }
}
// Ordinary migration/restore/export paths need the same selected qualification
// before issuing native owner checks. Consume and close the exact held read once.
fn validate_ordinary_read(
    read: ReadSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<wow_store::project::ValidatedRead> {
    let validation = (|| {
        let pair = AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
        let native_api = crate::graph::validate_native_api_reopen(&pair, stop)?;
        let validated = read
            .owner_validation(&candidate::completed_checks(false))
            .map_err(store_error);
        drop(native_api);
        drop(pair);
        validated
    })();
    read.close().map_err(store_error)?;
    validation
}
fn catalog() -> ServiceResult<RecordCatalog> {
    catalog_for(publication::STORAGE_SCHEMAS)
}
// The selected-component family cannot attest XML/declaration choices itself.
// require_configuration checks those exact choices against reconstructed owners.
fn catalog_accepts_components(
    schemas: &[&str],
    components: (
        bool,
        Option<PhysicalFunctionDeclarationRecipe>,
        Option<PhysicalFunctionSignatureRecipe>,
    ),
) -> bool {
    if schemas == publication::STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1
        || schemas == publication::STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1
        || schemas == publication::STORAGE_SCHEMAS_NATIVE_API_USES_V1
    {
        return false;
    }
    match components.2 {
        Some(PhysicalFunctionSignatureRecipe::IndexedTypesV1) => {
            schemas == publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1
        }
        None => {
            schemas != publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1
                && catalog_components(schemas) == (components.0, components.1)
        }
    }
}
// Retained signature-free catalogs bind exact XML/declaration recipes.
// Called only after excluding the selected-component family.
fn catalog_components(schemas: &[&str]) -> (bool, Option<PhysicalFunctionDeclarationRecipe>) {
    use PhysicalFunctionDeclarationRecipe::{DeclaredSyntaxAndBodyShapeV1, DeclaredSyntaxV1};
    if schemas == publication::STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_V1 {
        (true, Some(DeclaredSyntaxAndBodyShapeV1))
    } else if schemas == publication::STORAGE_SCHEMAS_PHYSICAL_FUNCTION_BODY_SHAPE_V1 {
        (false, Some(DeclaredSyntaxAndBodyShapeV1))
    } else if schemas
        == publication::STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_V1
    {
        (true, Some(DeclaredSyntaxV1))
    } else if schemas == publication::STORAGE_SCHEMAS_PHYSICAL_FUNCTION_DECLARATIONS_V1 {
        (false, Some(DeclaredSyntaxV1))
    } else if schemas == publication::STORAGE_SCHEMAS_XML_CLASSIFICATION_V1 {
        (true, None)
    } else {
        (false, None)
    }
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
pub(crate) fn store_error(error: StoreError) -> ServiceError {
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
