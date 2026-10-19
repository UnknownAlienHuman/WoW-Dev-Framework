use super::{LiveProjectStore, store_error};
use crate::ServiceResult;
use std::{path::Path, sync::atomic::AtomicBool};
pub use wow_store::project::{CurrentState, RecoveryReport, ScopeState};

impl LiveProjectStore {
    /// Observe physical seals, membership and receipts on one held snapshot.
    /// This does not authorize repair, replacement or another storage lifecycle.
    pub fn recovery_report(&self, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
        self.store.recovery_report(stop).map_err(store_error)
    }
}

/// Open only an exactly registered live-project profile, then observe it once.
pub fn recover_live_project(root: &Path, stop: &AtomicBool) -> ServiceResult<RecoveryReport> {
    LiveProjectStore::open(root)?.recovery_report(stop)
}
