use super::{LiveProjectStore, project_error, store_error};
use crate::ServiceResult;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use wow_graph::GraphPartitionSnapshot;
use wow_project::replay::publication::{self, AcquiredProjectPair};
use wow_store::OperationId;
pub use wow_store::project::{CurrentState, RecoveryReport, ScopeState};
use wow_store::project::{ReadSelector, ValidatedRead, VerifiedBackup};

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

fn validate_owners(
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
