//! Explicit logical platform-store selection, separate from source generations.
use super::{
    LiveProjectRead, LiveProjectResult, LiveProjectStore, LiveProjectUpdateRequest, catalog,
    catalog_for, fail, store_error,
};
use crate::{LocalProjectInput, ServiceErrorCode, ServiceResult, graph::GraphBuildRequest};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_core::ProfileId;
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
        })
    }

    /// Explicit fixed catalog intent; publication also checks the genuine owner.
    #[must_use]
    pub fn with_xml_classification_catalog(mut self) -> Self {
        self.xml_classification_catalog = true;
        self
    }

    fn catalog(&self) -> ServiceResult<RecordCatalog> {
        if self.xml_classification_catalog {
            catalog_for(publication::STORAGE_SCHEMAS_XML_CLASSIFICATION_V1)
        } else {
            catalog()
        }
    }

    fn require_catalog(&self, actual: &RecordCatalog) -> ServiceResult<()> {
        for &schemas in publication::STORAGE_SCHEMA_CATALOGS {
            if self.xml_classification_catalog
                == (schemas == publication::STORAGE_SCHEMAS_XML_CLASSIFICATION_V1)
                && &catalog_for(schemas)? == actual
            {
                return Ok(());
            }
        }
        Err(fail(ServiceErrorCode::IdentityMismatch))
    }

    /// Immutable configuration getters are not a receipt-derived capability.
    /// The existing graph/replay owners still validate the complete native closure.
    pub(super) fn require_configuration(
        &self,
        configuration: &ProjectConfiguration,
    ) -> ServiceResult<()> {
        self.validate()?;
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
        selection.validate()?;
        let store = Self::open_selected(root, Some(selection.xml_classification_catalog))?;
        store.require_namespace(selection)?;
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
        self.require_namespace(selection)?;
        self.read(selector, stop)
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
        self.update(input, graph_request, request, stop)
    }
}
