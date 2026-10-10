//! Guarded immutable target export with portable original-epoch authority.
use super::{ReadyMigration, ValidatedMigration};
use crate::project::{
    CurrentRecordId, ProjectStore, RegistrySelection, VerifiedBackup,
    model::{checkpoint, failure},
    registry, source_authority,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::{path::Path, sync::atomic::AtomicBool};

impl ProjectStore {
    /// Export the held ready target with exact original selector and hold evidence.
    /// Source guards run before effects and against the same snapshot after copying.
    /// This writes a new immutable artifact and never selects a live epoch.
    #[allow(clippy::too_many_arguments)]
    pub fn export_ready_migration_to_new(
        &self,
        migration: &ValidatedMigration,
        ready: &ReadyMigration,
        root: &Path,
        operation: &OperationId,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
        stop: &AtomicBool,
    ) -> StoreResult<VerifiedBackup> {
        self.require_migration_source(expected, expected_current, migration.source(), stop)?;
        ready.verify_for_export(migration, stop)?;
        let admitted = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        let sources = source_authority::capture_source(
            &self.db.root,
            &admitted,
            migration.source().manifest().snapshot_digest(),
            stop,
        )?;
        checkpoint(stop)?;
        let backup = ready
            .artifact()
            .restore_to_new_with_authorities(root, operation, sources, stop)?;
        self.require_migration_source(expected, expected_current, migration.source(), stop)?;
        if backup.manifest().epoch() != ready.artifact().manifest().epoch()
            || backup.manifest().current() != ready.artifact().manifest().current()
            || backup.manifest().generations() != ready.artifact().manifest().generations()
            || backup.manifest().source_authorities().is_empty()
        {
            return Err(failure(StoreErrorCode::IntegrityViolation));
        }
        checkpoint(stop)?;
        Ok(backup)
    }
}
