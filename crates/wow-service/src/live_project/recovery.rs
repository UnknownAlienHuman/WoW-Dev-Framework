use super::{LiveProjectStore, project_error, store_error};
use crate::ServiceResult;
use std::{path::Path, sync::atomic::AtomicBool};
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair};
use wow_store::OperationId;
use wow_store::project::{
    CurrentRecordId, ReadSelector, RegistrySelection, ReplacementReceipt, ValidatedRead,
    VerifiedBackup,
};
pub use wow_store::project::{CurrentState, RecoveryReport, ScopeState};

impl LiveProjectStore {
    /// Observe physical seals, membership and receipts on one held snapshot.
    /// This does not certify domain replay or authorize repair/activation.
    pub fn recovery_report(&self, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
        self.store.recovery_report(stop).map_err(store_error)
    }

    /// Create and independently replay every retained native project/graph pair.
    /// A failed owner check leaves the physical artifact for explicit inspection.
    pub fn backup_to_new(
        &self,
        root: &Path,
        operation_id: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<VerifiedBackup> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let backup = self
            .store
            .backup_to_new(root, &id, stop)
            .map_err(store_error)?;
        validate_owners(&backup, stop)?;
        Ok(backup)
    }

    pub fn registry_selection(&self) -> ServiceResult<RegistrySelection> {
        self.store.registry_selection().map_err(store_error)
    }

    /// Observe the selected original receipt without dispatching another effect.
    pub fn replacement_receipt(
        &self,
        operation_id: &str,
        request_digest: &str,
    ) -> ServiceResult<Option<ReplacementReceipt>> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        self.store
            .replacement_receipt(&id, request_digest)
            .map_err(store_error)
    }

    /// Restore one explicit backup and select it under both exact live guards.
    /// Every retained native Project/Graph pair is replayed before activation.
    pub fn restore_replace(
        &mut self,
        backup: &VerifiedBackup,
        operation_id: &str,
        expected: &RegistrySelection,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> ServiceResult<ReplacementReceipt> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let candidate = self
            .store
            .stage_replacement(backup, &id, expected, expected_current, stop)
            .map_err(store_error)?;
        let checks = validate_owners(candidate.backup(), stop)?;
        self.store
            .activate_replacement(candidate, checks, stop)
            .map_err(store_error)
    }

    /// Explicitly reconcile the original staged/published intent. No copy or
    /// selector change is repeated for an already selected exact operation.
    pub fn resume_replacement(
        &mut self,
        operation_id: &str,
        expected: &RegistrySelection,
        expected_current: Option<CurrentRecordId>,
        snapshot_digest: &str,
        stop: &AtomicBool,
    ) -> ServiceResult<ReplacementReceipt> {
        let id = OperationId::new(operation_id).map_err(store_error)?;
        let candidate = self
            .store
            .reopen_replacement(&id, expected, expected_current, snapshot_digest, stop)
            .map_err(store_error)?;
        let checks = validate_owners(candidate.backup(), stop)?;
        self.store
            .activate_replacement(candidate, checks, stop)
            .map_err(store_error)
    }
}

/// Open only an exactly registered live-project profile, then observe it once.
pub fn recover_live_project(root: &Path, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
    LiveProjectStore::open(root)?.recovery_report(stop)
}

/// Restore only to a new private path. All native owners run before its registry
/// is finalized. The source/live root, current and previously leased pairs survive.
pub fn restore_live_project_to_new(
    backup: &VerifiedBackup,
    root: &Path,
    operation_id: &str,
    stop: &AtomicBool,
) -> ServiceResult<LiveProjectStore> {
    let id = OperationId::new(operation_id).map_err(store_error)?;
    let candidate = backup
        .restore_to_new(root, &id, stop)
        .map_err(store_error)?;
    let checks = validate_owners(&candidate, stop)?;
    let store = candidate
        .finish_restore(checks, stop)
        .map_err(store_error)?;
    Ok(LiveProjectStore { store })
}

pub(super) fn validate_owners(
    backup: &VerifiedBackup,
    stop: &AtomicBool,
) -> ServiceResult<Vec<ValidatedRead>> {
    backup.verify(stop).map_err(store_error)?;
    let mut checks = Vec::new();
    for id in backup.manifest().generations() {
        let read = backup
            .read(&ReadSelector::Exact(id.clone()), stop)
            .map_err(store_error)?;
        AcquiredProjectPair::read(&read, stop).map_err(project_error)?;
        checks.push(
            read.owner_validation(&[
                GraphPartitionSnapshot::STORAGE_CHECK,
                publication::STORAGE_CHECK,
            ])
            .map_err(store_error)?,
        );
    }
    Ok(checks)
}
