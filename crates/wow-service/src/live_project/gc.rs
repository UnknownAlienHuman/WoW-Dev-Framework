use super::{LiveProjectStore, fail, store_error};
use crate::{ServiceErrorCode, ServiceResult};
use std::sync::atomic::AtomicBool;
use wow_store::OperationId;
use wow_store::project::{
    GC_PHYSICAL_PROFILE, ProjectGcPlan, ProjectGcPolicy, ProjectGcReceipt, PublicationOperation,
};

impl LiveProjectStore {
    pub fn gc_policy(&self) -> ServiceResult<Option<ProjectGcPolicy>> {
        self.require_gc()?;
        self.store.gc_policy().map_err(store_error)
    }
    pub fn select_gc_policy(
        &mut self,
        policy: &ProjectGcPolicy,
        expected_digest: Option<&str>,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectGcPolicy> {
        self.require_gc()?;
        self.store
            .select_gc_policy(policy, expected_digest, stop)
            .map_err(store_error)
    }
    pub fn release_publication(
        &mut self,
        operation_id: &str,
        expected_digest: &str,
        held_by: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<PublicationOperation> {
        self.require_gc()?;
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store
            .release_publication(&id, expected_digest, held_by, stop)
            .map_err(store_error)
    }
    pub fn plan_gc(
        &self,
        policy: &ProjectGcPolicy,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectGcPlan> {
        self.require_gc()?;
        self.store.plan_gc(policy, stop).map_err(store_error)
    }
    pub fn execute_gc(
        &mut self,
        plan: &ProjectGcPlan,
        operation_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<ProjectGcReceipt> {
        self.require_gc()?;
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store.execute_gc(plan, &id, stop).map_err(store_error)
    }
    pub fn reconcile_gc(&self, operation_id: &str) -> ServiceResult<Option<ProjectGcReceipt>> {
        self.require_gc()?;
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store.reconcile_gc(&id).map_err(store_error)
    }
    fn require_gc(&self) -> ServiceResult<()> {
        if self.store.epoch().physical_profile() != GC_PHYSICAL_PROFILE {
            return Err(fail(ServiceErrorCode::OperationNotImplementedForMilestone));
        }
        Ok(())
    }
}
