//! Guarded live-source staging of a separately retained inactive migration.

use super::MigrationCandidate;
use crate::project::{
    CurrentRecordId, ProjectStore, RegistrySelection, VerifiedBackup,
    backup::identity::capture,
    model::{checkpoint, failure},
    quarantine::archives,
    registry, source_authority,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::{path::Path, sync::atomic::AtomicBool};

impl ProjectStore {
    /// Stage an inactive migration from the exact currently held live source.
    ///
    /// The source selector, explicit optional current and complete persisted
    /// closure are checked before creation and again against the candidate's
    /// retained source. This never changes the live source or grants activation
    /// authority. [`MigrationCandidate::stage`] remains the offline builder.
    /// A failure after staging retains the inactive files for reconciliation;
    /// this operation does not rebase or remove them.
    pub fn stage_migration_to_new(
        &self,
        source: &VerifiedBackup,
        root: &Path,
        operation: &OperationId,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
        stop: &AtomicBool,
    ) -> StoreResult<MigrationCandidate> {
        self.require_migration_source(expected, expected_current, source, stop)?;
        let candidate = MigrationCandidate::stage(source, root, operation, stop)?;
        self.require_migration_source(expected, expected_current, candidate.source(), stop)?;
        Ok(candidate)
    }

    pub(in crate::project::migration) fn require_migration_source(
        &self,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
        source: &VerifiedBackup,
        stop: &AtomicBool,
    ) -> StoreResult<()> {
        checkpoint(stop)?;
        self.db.ensure_idle()?;
        expected.validate()?;
        source.verify(stop)?;

        let admitted = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        if admitted.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        if &admitted.selection != expected
            || self.db.selection.as_ref() != Some(expected)
            || admitted.epoch != self.db.epoch
            || source.manifest().epoch() != &self.db.epoch
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }

        let archives = archives::read(
            &self.db.root,
            &self.db.epoch.catalog,
            &admitted.retained_quarantines,
            stop,
        )?;
        archives.validate_epoch(&self.db.epoch)?;
        let sources = source_authority::read(&self.db.root, &admitted.source_authorities, stop)?;
        sources.admit_selected(&archives)?;

        {
            let connection = self.db.read_connection()?;
            let state = capture(&connection, &self.db.epoch, stop)?;
            if state.recovery.current().map(|current| &current.record_id) != expected_current
                || state.digest_with_authorities(archives.references(), sources.references())?
                    != source.manifest().snapshot_digest()
            {
                return Err(failure(StoreErrorCode::CurrentConflict));
            }
        }

        checkpoint(stop)
    }
}
