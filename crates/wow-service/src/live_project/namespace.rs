//! Explicit logical platform-store selection, separate from source generations.
use super::{
    LiveProjectRead, LiveProjectReadFailure, LiveProjectResult, LiveProjectStore,
    LiveProjectUpdateRequest, ReadFailureClosure, catalog_for, fail, store_error,
};
use crate::{LocalProjectInput, ServiceErrorCode, ServiceResult, graph::GraphBuildRequest};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_core::ProfileId;
use wow_emmy::function_calls::{
    PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
};
use wow_graph::GraphPartitionSnapshot;
use wow_project::{ProjectConfiguration, ProjectId, ProjectPublisher, replay::publication};
use wow_store::project::{
    CurrentRecordId, ProjectStore, ProjectStoreNamespace, ProjectStoreNamespaceRequest,
    PublicationOperation, ReadSelector, RecordCatalog,
};

/// A caller's exact logical source profile and native owner project selection.
/// Native publication/replay separately validates the genuine platform owners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStoreSelection {
    namespace: ProjectStoreNamespace,
    xml_classification_catalog: bool,
    physical_function_declarations_catalog: Option<PhysicalFunctionDeclarationRecipe>,
    physical_function_signatures_catalog: Option<PhysicalFunctionSignatureRecipe>,
    composed_candidate_catalog: bool,
}

impl PlatformStoreSelection {
    pub fn new(owner_project_id: ProjectId, source_profile_id: ProfileId) -> ServiceResult<Self> {
        let namespace = ProjectStoreNamespace::new(ProjectStoreNamespaceRequest {
            logical_namespace: source_profile_id.as_str().into(),
            owner_project_id: owner_project_id.as_str().into(),
        })
        .map_err(store_error)?;
        Ok(Self {
            namespace,
            xml_classification_catalog: false,
            physical_function_declarations_catalog: None,
            physical_function_signatures_catalog: None,
            composed_candidate_catalog: false,
        })
    }

    /// Explicit fixed catalog intent; publication also checks the genuine owner.
    #[must_use]
    pub fn with_xml_classification_catalog(mut self) -> Self {
        self.xml_classification_catalog = true;
        self
    }

    /// Explicit declaration catalog intent; configuration must select the same recipe.
    #[must_use]
    pub fn with_physical_function_declarations_catalog(mut self) -> Self {
        self.physical_function_declarations_catalog =
            Some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1);
        self
    }

    /// Select one exact native declaration recipe without widening old catalogs.
    #[must_use]
    pub fn with_physical_function_declarations_recipe_catalog(
        mut self,
        recipe: PhysicalFunctionDeclarationRecipe,
    ) -> Self {
        self.physical_function_declarations_catalog = Some(recipe);
        self
    }

    /// Explicit signature recipe; configuration must retain exactly the same selection.
    #[must_use]
    pub fn with_physical_function_signatures_catalog(
        mut self,
        recipe: PhysicalFunctionSignatureRecipe,
    ) -> Self {
        self.physical_function_signatures_catalog = Some(recipe);
        self
    }

    /// Require the selected service candidate owner on publication and reopen.
    #[must_use]
    pub fn with_composed_candidate_catalog(mut self) -> Self {
        self.composed_candidate_catalog = true;
        self
    }
    pub(super) const fn composed_candidate_selected(&self) -> bool {
        self.composed_candidate_catalog
    }
    fn selected_catalog(&self) -> ServiceResult<RecordCatalog> {
        if self.composed_candidate_catalog {
            super::candidate::catalog()
        } else {
            catalog_for(publication::STORAGE_SCHEMAS)
        }
    }

    fn catalog(&self) -> ServiceResult<RecordCatalog> {
        self.selected_catalog()
    }

    fn require_catalog(&self, actual: &RecordCatalog) -> ServiceResult<()> {
        if &self.selected_catalog()? == actual {
            Ok(())
        } else {
            Err(fail(ServiceErrorCode::IdentityMismatch))
        }
    }

    /// Immutable configuration getters are not a receipt-derived capability.
    /// The existing graph/replay owners still validate the complete native closure.
    pub(super) fn require_configuration(
        &self,
        configuration: &ProjectConfiguration,
    ) -> ServiceResult<()> {
        self.validate()?;
        if self.composed_candidate_catalog
            && !matches!(
                configuration.platform_graph_profile(),
                Some(wow_project::PlatformGraphProfile::DirectCapabilitiesV1(_))
            )
        {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        let classification_selected = matches!(
            configuration.platform_graph_profile(),
            Some(wow_project::PlatformGraphProfile::DirectCapabilitiesV1(capabilities))
                if capabilities.contains(wow_project::PlatformGraphCapability::XmlSchemaClassificationV1)
        );
        let packages = configuration
            .platform_packages()
            .ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?;
        if self.xml_classification_catalog != classification_selected
            || self.xml_classification_catalog
                != configuration.xml_classification_profile().is_some()
            || self.physical_function_declarations_catalog
                != configuration.physical_function_declarations()
            || self.physical_function_signatures_catalog
                != configuration.physical_function_signatures()
            || self.namespace.owner_project_id() != configuration.project_id().as_str()
            || self.namespace.logical_namespace()
                != packages.source().profile().profile_id().as_str()
        {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        Ok(())
    }

    pub fn namespace(&self) -> &ProjectStoreNamespace {
        &self.namespace
    }

    pub fn validate(&self) -> ServiceResult<()> {
        self.namespace.validate().map_err(store_error)
    }
}

impl LiveProjectStore {
    pub fn create_in_namespace(
        root: &Path,
        selection: &PlatformStoreSelection,
    ) -> ServiceResult<Self> {
        selection.validate()?;
        let store = Self {
            store: ProjectStore::create_with_namespace(
                root,
                selection.namespace(),
                selection.catalog()?,
            )
            .map_err(store_error)?,
        };
        store.require_namespace(selection)?;
        Ok(store)
    }

    pub fn open_in_namespace(
        root: &Path,
        selection: &PlatformStoreSelection,
    ) -> ServiceResult<Self> {
        Self::open_in_namespace_checked(root, selection).map_err(|failure| failure.error)
    }

    /// Preserve exact selected admission and the explicit close outcome for a
    /// post-open refusal. Lower open failures expose no closure receipt.
    pub(crate) fn open_in_namespace_checked(
        root: &Path,
        selection: &PlatformStoreSelection,
    ) -> Result<Self, LiveProjectReadFailure> {
        selection.validate().map_err(namespace_unacquired)?;
        let store = Self::open_selected(root, Some(selection.composed_candidate_catalog))
            .map_err(namespace_unacquired)?;
        if let Err(error) = store.require_namespace(selection) {
            return Err(namespace_closed_failure(error, store.close()));
        }
        Ok(store)
    }

    pub(super) fn require_namespace(
        &self,
        selection: &PlatformStoreSelection,
    ) -> ServiceResult<()> {
        selection.validate()?;
        let epoch = self.store.epoch();
        if epoch.namespace() != Some(selection.namespace())
            || epoch.owner() != selection.namespace().id().as_str()
        {
            return Err(fail(ServiceErrorCode::IdentityMismatch));
        }
        selection.require_catalog(epoch.catalog())
    }

    pub fn publish_in_namespace(
        &mut self,
        selection: &PlatformStoreSelection,
        publisher: &ProjectPublisher,
        graph: &GraphPartitionSnapshot,
        operation_id: &str,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
        self.require_namespace(selection)?;
        let configuration = publisher
            .current_snapshot()
            .ok_or_else(|| fail(ServiceErrorCode::IdentityMismatch))?
            .configuration();
        selection.require_configuration(configuration)?;
        self.publish(publisher, graph, operation_id, expected_current, stop)
    }

    pub fn read_in_namespace(
        &self,
        selection: &PlatformStoreSelection,
        selector: &ReadSelector,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectRead> {
        self.read_in_namespace_checked(selection, selector, stop)
            .map_err(|failure| failure.error)
    }

    /// One original checked reconstruction; never reread to obtain a close receipt.
    pub(crate) fn read_in_namespace_checked(
        &self,
        selection: &PlatformStoreSelection,
        selector: &ReadSelector,
        stop: &AtomicBool,
    ) -> Result<LiveProjectRead, LiveProjectReadFailure> {
        self.require_namespace(selection)
            .map_err(namespace_unacquired)?;
        let read = self.read_checked(selector, stop)?;
        if let Err(error) = selection.require_configuration(read.project().configuration()) {
            return Err(namespace_closed_failure(error, read.close()));
        }
        Ok(read)
    }

    pub fn reconcile_in_namespace(
        &self,
        selection: &PlatformStoreSelection,
        operation_id: &str,
    ) -> ServiceResult<Option<PublicationOperation>> {
        self.require_namespace(selection)?;
        self.reconcile(operation_id)
    }

    /// Guard the selected epoch before any update outcome, including NoChange.
    pub fn update_in_namespace(
        &mut self,
        selection: &PlatformStoreSelection,
        input: LocalProjectInput,
        graph_request: &GraphBuildRequest,
        request: &LiveProjectUpdateRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LiveProjectResult> {
        self.require_namespace(selection)?;
        selection.require_configuration(input.project_bundle().configuration())?;
        if selection.composed_candidate_selected() {
            return self.update_candidate(selection, input, graph_request, request, stop);
        }
        self.update(input, graph_request, request, stop)
    }
}

fn namespace_unacquired(error: crate::ServiceError) -> LiveProjectReadFailure {
    LiveProjectReadFailure {
        error,
        closure: ReadFailureClosure::NotEvaluated,
    }
}
fn namespace_closed_failure(
    error: crate::ServiceError,
    closed: ServiceResult<()>,
) -> LiveProjectReadFailure {
    match closed {
        Ok(()) => LiveProjectReadFailure {
            error,
            closure: ReadFailureClosure::Closed,
        },
        Err(_) => LiveProjectReadFailure {
            error: fail(ServiceErrorCode::InternalContractViolation),
            closure: ReadFailureClosure::Failed,
        },
    }
}
