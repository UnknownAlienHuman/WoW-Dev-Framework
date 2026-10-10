//! Explicit logical platform-store selection, separate from source generations.
use super::{
    LiveProjectRead, LiveProjectResult, LiveProjectStore, LiveProjectUpdateRequest, catalog, fail,
    store_error,
};
use crate::{LocalProjectInput, ServiceErrorCode, ServiceResult, graph::GraphBuildRequest};
use std::{path::Path, sync::atomic::AtomicBool};
use wow_core::ProfileId;
use wow_graph::GraphPartitionSnapshot;
use wow_project::{ProjectId, ProjectPublisher};
use wow_store::project::{
    CurrentRecordId, ProjectStore, ProjectStoreNamespace, ProjectStoreNamespaceRequest,
    PublicationOperation, ReadSelector,
};

/// A caller's exact logical source profile and native owner project selection.
/// Native publication/replay separately validates the genuine platform owners.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformStoreSelection {
    namespace: ProjectStoreNamespace,
}

impl PlatformStoreSelection {
    pub fn new(owner_project_id: ProjectId, source_profile_id: ProfileId) -> ServiceResult<Self> {
        let namespace = ProjectStoreNamespace::new(ProjectStoreNamespaceRequest {
            logical_namespace: source_profile_id.as_str().into(),
            owner_project_id: owner_project_id.as_str().into(),
        })
        .map_err(store_error)?;
        Ok(Self { namespace })
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
            store: ProjectStore::create_with_namespace(root, selection.namespace(), catalog()?)
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
        let store = Self::open(root)?;
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
        Ok(())
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
        self.update(input, graph_request, request, stop)
    }
}
