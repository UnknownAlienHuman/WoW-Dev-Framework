use super::{LiveProjectStore, fail, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use std::sync::atomic::AtomicBool;
use wow_store::project::{
    EpochManifest, RETAINED_PHYSICAL_PROFILE, RetentionRoot, RetentionRootId,
};

impl LiveProjectStore {
    pub fn storage_epoch(&self) -> &EpochManifest {
        self.store.epoch()
    }

    pub fn retention_roots(&self, stop: &AtomicBool) -> ServiceResult<Vec<RetentionRoot>> {
        self.require_retention()?;
        self.store.retention_roots(stop).map_err(store_error)
    }

    pub fn put_retention_root(
        &mut self,
        root: &RetentionRoot,
        stop: &AtomicBool,
    ) -> ServiceResult<RetentionRoot> {
        self.require_retention()?;
        self.store
            .put_retention_root(root, stop)
            .map_err(store_error)
    }

    pub fn remove_retention_root(
        &mut self,
        id: &RetentionRootId,
        expected_digest: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<bool> {
        self.require_retention()?;
        self.store
            .remove_retention_root(id, expected_digest, stop)
            .map_err(store_error)
    }

    fn require_retention(&self) -> ServiceResult<()> {
        if self.store.epoch().physical_profile() != RETAINED_PHYSICAL_PROFILE {
            return Err(fail(ServiceErrorCode::OperationNotImplementedForMilestone));
        }
        Ok(())
    }
}
