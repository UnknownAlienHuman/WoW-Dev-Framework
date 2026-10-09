//! One held, bounded SQLite observation. Recovery never repeats effects or
//! selects a replacement current. Domain replay remains an owner responsibility.
mod model;
mod scan;
#[cfg(test)]
mod tests;
use super::ProjectStore;
use crate::StoreResult;
pub use model::*;
pub(super) use scan::inspect as inspect_snapshot;
use std::sync::atomic::AtomicBool;

impl ProjectStore {
    /// Inspect admitted durable state without repairing or activating it.
    /// Coverage qualifies every negative; receipt availability does not prove ACK.
    pub fn recovery_report(&self, stop: &AtomicBool) -> StoreResult<RecoveryReport> {
        self.db.ensure_idle()?;
        super::model::checkpoint(stop)?;
        let connection = self.db.read_connection()?;
        let report = scan::inspect(&connection, &self.db.epoch, stop)?;
        // Explicitly release the snapshot only after every scope was observed.
        connection
            .execute_batch("ROLLBACK")
            .map_err(crate::StoreError::database)?;
        Ok(report)
    }
}
