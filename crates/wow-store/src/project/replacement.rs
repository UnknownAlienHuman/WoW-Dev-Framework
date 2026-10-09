//! Explicit same-semantic-epoch replacement by a new confined physical instance.
#[cfg(test)]
mod tests;
use super::{
    AcknowledgmentState, CurrentRecordId, ProjectStore, RegistrySelection, ValidatedRead,
    VerifiedBackup, backup,
    database::{self, Database, Lifetime},
    model::*,
    registry::{self, RegistryRecord, ReplacementIntent},
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::Serialize;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs,
    rc::Rc,
    sync::atomic::AtomicBool,
};

/// An immutable native copy with exact source guards and durable intent. A
/// serialized report cannot manufacture this held candidate or owner checks.
pub struct ReplacementCandidate {
    backup: VerifiedBackup,
    intent: ReplacementIntent,
}
impl ReplacementCandidate {
    pub fn backup(&self) -> &VerifiedBackup {
        &self.backup
    }
    pub fn request_digest(&self) -> StoreResult<String> {
        self.intent.digest()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplacementReceipt {
    operation_id: OperationId,
    request_digest: String,
    previous: RegistrySelection,
    selected: RegistrySelection,
    activated_current: Option<CurrentPublication>,
    snapshot_digest: String,
    owner_validation_digest: String,
    acknowledgment: AcknowledgmentState,
    durability: String,
}
impl ReplacementReceipt {
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
    pub fn selected(&self) -> &RegistrySelection {
        &self.selected
    }
    pub fn previous(&self) -> &RegistrySelection {
        &self.previous
    }
    pub fn activated_current(&self) -> Option<&CurrentPublication> {
        self.activated_current.as_ref()
    }
    fn from_record(record: &RegistryRecord) -> StoreResult<Self> {
        Ok(Self {
            operation_id: record.intent.operation_id.clone(),
            request_digest: record.request_digest.clone(),
            previous: record.intent.expected.clone(),
            selected: record.selection()?,
            activated_current: record.activated_current.clone(),
            snapshot_digest: record.intent.snapshot_digest.clone(),
            owner_validation_digest: record.owner_validation_digest.clone(),
            acknowledgment: AcknowledgmentState::Unknown,
            durability: "file-synced-selector-replaced-and-read-back;power-loss-not-evaluated"
                .into(),
        })
    }
}
impl ProjectStore {
    /// Observe the exact canonical selector under the existing root writer lock.
    pub fn registry_selection(&self) -> StoreResult<RegistrySelection> {
        self.db.ensure_idle()?;
        self.db.selection.clone().ok_or_else(invalid)
    }
    /// Reconcile the selected replacement receipt only; no effect is redispatched.
    /// A later replacement has a different selector and cannot masquerade as it.
    pub fn replacement_receipt(
        &self,
        operation: &OperationId,
        request_digest: &str,
    ) -> StoreResult<Option<ReplacementReceipt>> {
        self.db.ensure_idle()?;
        OperationId::new(operation.as_str())?;
        if !hashed(request_digest, "project-replacement-request") {
            return Err(invalid());
        }
        let admitted = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        match admitted.record {
            Some(record) if &record.intent.operation_id == operation => {
                if record.request_digest != request_digest {
                    return Err(failure(StoreErrorCode::OperationConflict));
                }
                ReplacementReceipt::from_record(&record).map(Some)
            }
            _ => Ok(None),
        }
    }
    /// Make an inactive native copy within the owned root. Both source guards
    /// are explicit; an existing operation directory is never overwritten.
    pub fn stage_replacement(
        &self,
        backup: &VerifiedBackup,
        operation: &OperationId,
        expected: &RegistrySelection,
        expected_current: Option<CurrentRecordId>,
        stop: &AtomicBool,
    ) -> StoreResult<ReplacementCandidate> {
        self.require_replacement_base(expected, expected_current.as_ref())?;
        checkpoint(stop)?;
        backup.verify(stop)?;
        if backup.manifest().epoch() != self.epoch() {
            return Err(invalid());
        }
        let intent = ReplacementIntent::new(
            operation.clone(),
            expected.clone(),
            expected_current,
            self.epoch().epoch_id.clone(),
            backup.manifest().snapshot_digest().to_owned(),
        )?;
        let instances = self.db.root.join("instances");
        match fs::symlink_metadata(&instances) {
            Ok(_) => database::directory(&instances)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                database::create_private_directory(&instances)
                    .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
            }
            Err(_) => return Err(invalid()),
        }
        let root = instances.join(intent.instance()?);
        match fs::symlink_metadata(&root) {
            Ok(_) => return Err(failure(StoreErrorCode::OperationConflict)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(invalid()),
        }
        let mut candidate = backup.restore_to_new(&root, operation, stop)?;
        share_admission(&mut candidate, &self.db.life);
        backup::write_new(&root.join("replacement-intent.json"), &intent.bytes()?)?;
        checkpoint(stop)?;
        Ok(ReplacementCandidate {
            backup: candidate,
            intent,
        })
    }
    /// Explicitly reconcile an already staged operation. Never repeat copying,
    /// select newest data or loosen original expected-current/selector guards.
    pub fn reopen_replacement(
        &self,
        operation: &OperationId,
        expected: &RegistrySelection,
        expected_current: Option<CurrentRecordId>,
        snapshot_digest: &str,
        stop: &AtomicBool,
    ) -> StoreResult<ReplacementCandidate> {
        let intent = ReplacementIntent::new(
            operation.clone(),
            expected.clone(),
            expected_current,
            self.epoch().epoch_id.clone(),
            snapshot_digest.to_owned(),
        )?;
        self.require_replacement_state(&intent)?;
        checkpoint(stop)?;
        database::directory(&self.db.root.join("instances"))?;
        let root = self.db.root.join("instances").join(intent.instance()?);
        database::directory(&root)?;
        if registry::read_file(
            &root.join("replacement-intent.json"),
            registry::MAX_REGISTRY,
        )? != intent.bytes()?
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let mut backup = VerifiedBackup::open(
            &root,
            &self.epoch().catalog,
            operation,
            snapshot_digest,
            stop,
        )?;
        share_admission(&mut backup, &self.db.life);
        Ok(ReplacementCandidate { backup, intent })
    }
    /// Validate an exact immutable target, publish the outer selector once, then
    /// switch future reads. Existing leases retain their old connection and the
    /// shared root OS lock; this operation never deletes old physical instances.
    pub fn activate_replacement(
        &mut self,
        candidate: ReplacementCandidate,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<ReplacementReceipt> {
        let already_selected = self.require_replacement_state(&candidate.intent)?;
        candidate.backup.verify(stop)?;
        if candidate.backup.manifest().epoch() != self.epoch()
            || candidate.backup.manifest().snapshot_digest() != candidate.intent.snapshot_digest
            || candidate.backup.manifest().operation_id() != &candidate.intent.operation_id
            || candidate.backup.root
                != self
                    .db
                    .root
                    .join("instances")
                    .join(candidate.intent.instance()?)
        {
            return Err(invalid());
        }
        let owners = candidate.backup.restore_validation_digest(checks, stop)?;
        let record = RegistryRecord::new(
            self.db.epoch.clone(),
            candidate.intent.clone(),
            owners,
            candidate.backup.manifest().current().cloned(),
        )?;
        let bytes = record.bytes()?;
        let selection = record.selection()?;
        if already_selected
            .as_ref()
            .is_some_and(|selected| selected != &record)
        {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let dir = database::epoch_directory(&candidate.backup.root, self.epoch())?;
        let epoch_bytes = encode(self.epoch(), 65536)?;
        write_exact_or_new(&dir.join("epoch-manifest.json"), &epoch_bytes)?;
        write_exact_or_new(
            &candidate.backup.root.join("replacement-record.json"),
            &bytes,
        )?;
        let staged = self
            .db
            .root
            .join(format!("project-store-registry-{}.staged", record.instance));
        if already_selected.is_none() {
            write_exact_or_new(&staged, &bytes)?;
        }

        // All candidate body/owner checks precede the writable open. No database
        // writes occur between final source CAS and the noninterruptible replace.
        let connection = database::connect(&candidate.backup.store.db.path, false)?;
        database::validate_header(&connection, self.epoch())?;
        database::enable_writer(&connection)?;
        if self.require_replacement_state(&candidate.intent)? != already_selected {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        checkpoint(stop)?;
        // Same-root file replacement is one OS call. No remove-then-rename gap,
        // cleanup, retry loop or cancellation after dispatch. Failure stays
        // unknown; ensure_idle prevents subsequent writes to a stale selector.
        if already_selected.is_none() {
            fs::rename(&staged, self.db.root.join(registry::REGISTRY_FILE))
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        }
        let observed = registry::read(&self.db.root, &self.epoch().catalog)
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if observed.selection != selection || observed.record.as_ref() != Some(&record) {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let db = Database {
            connection,
            path: candidate.backup.store.db.path.clone(),
            epoch: self.db.epoch.clone(),
            root: self.db.root.clone(),
            selection: Some(selection),
            life: Rc::new(Lifetime {
                _lock: Rc::clone(&self.db.life._lock),
                _instance_lock: Some(Rc::clone(&candidate.backup.store.db.life._lock)),
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::clone(&self.db.life.reader_admissions),
            }),
        };
        let fresh = db
            .read_connection()
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        let actual = backup::identity::capture(&fresh, &db.epoch, &AtomicBool::new(false))
            .and_then(|state| state.digest())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if actual != record.intent.snapshot_digest {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        drop(fresh);
        self.db = db;
        ReplacementReceipt::from_record(&record)
    }
    fn require_replacement_base(
        &self,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
    ) -> StoreResult<()> {
        if &self.registry_selection()? != expected
            || self.current()?.as_ref().map(|c| &c.record_id) != expected_current
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        Ok(())
    }
    /// The old owner may reconcile a replaced-but-unacknowledged selector while
    /// its old readers still hold the root lock. It can only adopt this exact
    /// original request after repeat physical and native owner validation.
    fn require_replacement_state(
        &self,
        intent: &ReplacementIntent,
    ) -> StoreResult<Option<RegistryRecord>> {
        if !self.db.connection.is_autocommit() {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let observed = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        if observed.selection == intent.expected {
            self.require_replacement_base(&intent.expected, intent.expected_current.as_ref())?;
            return Ok(None);
        }
        if self.db.selection.as_ref() == Some(&intent.expected)
            && observed.epoch == self.db.epoch
            && let Some(record) = observed.record
            && record.intent == *intent
        {
            let current = super::read::read_current(&self.db.connection, &self.db.epoch)?;
            if current.as_ref().map(|c| &c.record_id) != intent.expected_current.as_ref() {
                return Err(failure(StoreErrorCode::CurrentConflict));
            }
            return Ok(Some(record));
        }
        Err(failure(StoreErrorCode::CurrentConflict))
    }
}
fn share_admission(backup: &mut VerifiedBackup, source: &Rc<Lifetime>) {
    backup.store.db.life = Rc::new(Lifetime {
        _lock: Rc::clone(&backup.store.db.life._lock),
        _instance_lock: Some(Rc::clone(&source._lock)),
        leases: RefCell::new(BTreeMap::new()),
        lease_revision: Cell::new(0),
        reader_admissions: Rc::clone(&source.reader_admissions),
    });
}
fn write_exact_or_new(path: &std::path::Path, bytes: &[u8]) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => {
            if registry::read_file(path, registry::MAX_REGISTRY)? != bytes {
                return Err(failure(StoreErrorCode::OperationConflict));
            }
            // Exact bytes reconcile content, not an earlier failed flush. Flush
            // again before reporting file-sync or dispatching the selector.
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .and_then(|file| file.sync_all())
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => backup::write_new(path, bytes),
        Err(_) => Err(invalid()),
    }
}
