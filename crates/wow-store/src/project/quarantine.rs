//! Explicit physical-instance hold. No SQL mutation, cleanup or automatic repair.
mod current;
#[cfg(all(test, windows))]
mod fault_tests;
pub(super) mod model;
mod owner;
#[cfg(test)]
mod selected_tests;
#[cfg(test)]
mod tests;
use super::{
    ProjectStore,
    database::{self, Lifetime},
    model::*,
    registry, replacement,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use model::QuarantineRecord;
pub use model::{CurrentObservation, PointerReadFailure, QuarantineReceipt};
pub use owner::QuarantinedStore;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicBool,
};

/// Unserializable observation bound to the admitted root's actual writer lease.
/// Changing Current or recovery evidence makes this inspection stale.
pub struct QuarantineInspection {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
    pub(super) epoch: EpochManifest,
    pub(super) life: Rc<Lifetime>,
    pub(super) selection: registry::RegistrySelection,
    current: CurrentObservation,
    evidence: Vec<u8>,
}
impl QuarantineInspection {
    /// Readonly admission permits observing a damaged SQL body without opening
    /// it writable. The normal canonical registry and confined files must admit.
    pub fn open(
        root: impl AsRef<Path>,
        catalog: &RecordCatalog,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        let (root, admitted, life, path) = owner::open_owned(root.as_ref(), catalog)?;
        if admitted.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        Self::capture(root, path, admitted.epoch, life, admitted.selection, stop)
    }
    fn capture(
        root: PathBuf,
        path: PathBuf,
        epoch: EpochManifest,
        life: Rc<Lifetime>,
        selection: registry::RegistrySelection,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        let (current, evidence) = capture(&path, &epoch, stop)?;
        Ok(Self {
            root,
            path,
            epoch,
            life,
            selection,
            current,
            evidence,
        })
    }
    pub fn selection(&self) -> &registry::RegistrySelection {
        &self.selection
    }
    pub fn current(&self) -> &CurrentObservation {
        &self.current
    }
    pub fn evidence_digest(&self) -> String {
        digest("project-quarantine-evidence", &self.evidence)
    }
    /// An explicit operator hold, not a physical report granting domain approval.
    /// Exact selected receipts reconcile without redispatching the rename.
    pub fn quarantine(
        &self,
        operation: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<QuarantineReceipt> {
        checkpoint(stop)?;
        let record = QuarantineRecord::new(
            self.epoch.clone(),
            operation.clone(),
            self.selection.clone(),
            self.current.clone(),
            &self.evidence,
        )?;
        let observed = registry::read(&self.root, &self.epoch.catalog)?;
        if let Some(selected) = observed.quarantine {
            if selected == record {
                return QuarantineReceipt::new(selected);
            }
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        if observed.selection != self.selection {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        self.recheck(stop)?;
        let parent = self.root.join("quarantines");
        match fs::symlink_metadata(&parent) {
            Ok(_) => database::directory(&parent)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                database::create_private_directory(&parent)
                    .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
            }
            Err(_) => return Err(invalid()),
        }
        let archive = record.archive(&self.root)?;
        match fs::symlink_metadata(&archive) {
            Ok(_) => database::directory(&archive)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                database::create_private_directory(&archive)
                    .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
            }
            Err(_) => return Err(invalid()),
        }
        let previous = registry::read_file(
            &self.root.join(registry::REGISTRY_FILE),
            registry::MAX_REGISTRY,
        )?;
        if digest("project-registry", &previous) != self.selection.digest() {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        replacement::write_exact_or_new(&archive.join("selection.json"), &previous)?;
        // Evidence has a distinct finite budget; never feed it to registry's smaller reader.
        write_evidence(&archive.join("evidence.json"), &self.evidence)?;
        let bytes = record.bytes()?;
        replacement::write_exact_or_new(&archive.join("record.json"), &bytes)?;
        let staged = archive.join("selector.staged");
        replacement::write_exact_or_new(&staged, &bytes)?;
        self.recheck(stop)?;
        if registry::read(&self.root, &self.epoch.catalog)?.selection != self.selection {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        checkpoint(stop)?;
        // One dispatch, followed by non-cancellable classification. No remove gap.
        fs::rename(&staged, self.root.join(registry::REGISTRY_FILE))
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        let selected = registry::read(&self.root, &self.epoch.catalog)
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if selected.quarantine.as_ref() != Some(&record) {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        QuarantineReceipt::new(record)
    }
    fn recheck(&self, stop: &AtomicBool) -> StoreResult<()> {
        let (current, evidence) = capture(&self.path, &self.epoch, stop)?;
        if current != self.current || evidence != self.evidence {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        Ok(())
    }
}
impl ProjectStore {
    pub fn quarantine_inspection(&self, stop: &AtomicBool) -> StoreResult<QuarantineInspection> {
        self.db.ensure_idle()?;
        QuarantineInspection::capture(
            self.db.root.clone(),
            self.db.path.clone(),
            self.db.epoch.clone(),
            Rc::clone(&self.db.life),
            self.db.selection.clone().ok_or_else(invalid)?,
            stop,
        )
    }
    pub fn quarantine(
        &self,
        operation: &OperationId,
        inspection: &QuarantineInspection,
        stop: &AtomicBool,
    ) -> StoreResult<QuarantineReceipt> {
        if !self.db.connection.is_autocommit() {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        if !Rc::ptr_eq(&self.db.life, &inspection.life)
            || self.db.root != inspection.root
            || self.db.path != inspection.path
            || self.db.epoch != inspection.epoch
        {
            return Err(invalid());
        }
        inspection.quarantine(operation, stop)
    }
    pub fn quarantined(&self, stop: &AtomicBool) -> StoreResult<QuarantinedStore> {
        checkpoint(stop)?;
        if !self.db.connection.is_autocommit() {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let admitted = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        let record = admitted.quarantine.ok_or_else(invalid)?;
        let path = record
            .previous
            .directory(&self.db.root, &record.epoch)?
            .join("project.sqlite");
        if path != self.db.path || record.epoch != self.db.epoch {
            return Err(invalid());
        }
        QuarantinedStore::from_parts(self.db.root.clone(), path, record, Rc::clone(&self.db.life))
    }
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
enum PhysicalEvidence {
    Report { report: Box<super::RecoveryReport> },
    Unavailable { code: StoreErrorCode },
}

fn capture(
    path: &Path,
    epoch: &EpochManifest,
    stop: &AtomicBool,
) -> StoreResult<(CurrentObservation, Vec<u8>)> {
    checkpoint(stop)?;
    database::regular(path, 1024 * 1024 * 1024)?;
    let read = (|| {
        let connection = database::connect(path, true)?;
        connection
            .execute_batch("BEGIN DEFERRED")
            .map_err(crate::StoreError::database)?;
        // Only the exact supported schema is queried. A bad header is evidence
        // of unavailable inspection, never pointer absence or permission to repair.
        database::validate_header(&connection, epoch)?;
        let current = current::observe(&connection)?;
        let evidence = match super::recovery::inspect_snapshot(&connection, epoch, stop) {
            Ok(report) => PhysicalEvidence::Report {
                report: Box::new(report),
            },
            Err(error) if error.code() == StoreErrorCode::Cancelled => return Err(error),
            Err(error) => PhysicalEvidence::Unavailable { code: error.code() },
        };
        connection
            .execute_batch("ROLLBACK")
            .map_err(crate::StoreError::database)?;
        Ok((current, evidence))
    })();
    let (current, evidence) = match read {
        Ok(value) => value,
        Err(error) if error.code() == StoreErrorCode::Cancelled => return Err(error),
        Err(error) => (
            CurrentObservation::Unreadable {
                reason: PointerReadFailure::QueryUnavailable,
            },
            PhysicalEvidence::Unavailable { code: error.code() },
        ),
    };
    checkpoint(stop)?;
    Ok((current, encode(&evidence, model::MAX_EVIDENCE)?))
}
fn write_evidence(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            if registry::read_file(path, model::MAX_EVIDENCE)? != bytes {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .and_then(|file| file.sync_all())
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => super::backup::write_new(path, bytes),
        Err(_) => Err(invalid()),
    }
}
