//! Explicit target-authorized recovery of a held physical instance.
use super::super::{
    ProjectStore, RegistrySelection, ReplacementCandidate, ReplacementReceipt, ValidatedRead,
    VerifiedBackup, backup,
    database::{self, Database, Lifetime},
    model::*,
    registry::{self, RegistryRecord, ReplacementIntent},
    replacement,
};
use super::{QuarantinedStore, archives, capture};
use crate::{OperationId, StoreErrorCode, StoreResult};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs,
    rc::Rc,
    sync::atomic::AtomicBool,
};

impl QuarantinedStore {
    /// Copy an explicit independently verified target. Held data contributes
    /// guard/evidence only and is never opened writable or used as target input.
    pub fn stage_restore(
        &self,
        backup: &VerifiedBackup,
        operation: &OperationId,
        expected: &RegistrySelection,
        stop: &AtomicBool,
    ) -> StoreResult<ReplacementCandidate> {
        let intent =
            self.restore_intent(operation, expected, backup.manifest().snapshot_digest())?;
        if self.require_restore_state(&intent, stop)?.is_some() {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        backup.verify(stop)?;
        if backup.manifest().epoch() != self.epoch() {
            return Err(invalid());
        }
        let parent = self.root.join("instances");
        archives::directory(&parent)?;
        let root = parent.join(intent.instance()?);
        match fs::symlink_metadata(&root) {
            Ok(_) => return Err(failure(StoreErrorCode::OperationConflict)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(invalid()),
        }
        let mut target = backup.restore_to_new(&root, operation, stop)?;
        replacement::share_admission(&mut target, &self.life);
        backup::write_new(&root.join("replacement-intent.json"), &intent.bytes()?)?;
        self.require_restore_state(&intent, stop)?;
        Ok(ReplacementCandidate {
            backup: target,
            intent,
        })
    }
    /// Reopen only the original complete candidate; copying and committed
    /// selection are never repeated by this reconciliation operation.
    pub fn reopen_restore(
        &self,
        operation: &OperationId,
        expected: &RegistrySelection,
        snapshot_digest: &str,
        stop: &AtomicBool,
    ) -> StoreResult<ReplacementCandidate> {
        let intent = self.restore_intent(operation, expected, snapshot_digest)?;
        self.require_restore_state(&intent, stop)?;
        let parent = self.root.join("instances");
        database::directory(&parent)?;
        let root = parent.join(intent.instance()?);
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
        replacement::share_admission(&mut backup, &self.life);
        Ok(ReplacementCandidate { backup, intent })
    }
    pub fn activate_restore(
        &self,
        candidate: ReplacementCandidate,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<(ProjectStore, ReplacementReceipt)> {
        let selected = self.require_restore_state(&candidate.intent, stop)?;
        candidate.backup.verify(stop)?;
        if candidate.backup.manifest().epoch() != self.epoch()
            || candidate.backup.manifest().snapshot_digest() != candidate.intent.snapshot_digest
            || candidate.backup.manifest().operation_id() != &candidate.intent.operation_id
            || candidate.backup.root
                != self
                    .root
                    .join("instances")
                    .join(candidate.intent.instance()?)
            || !Rc::ptr_eq(
                &candidate.backup.store.db.life.reader_admissions,
                &self.life.reader_admissions,
            )
        {
            return Err(invalid());
        }
        let owners = candidate.backup.restore_validation_digest(checks, stop)?;
        // Source archive closure is recovered from the exact original hold,
        // including its shallow preceding selector, even on adoption.
        let guard = candidate
            .intent
            .quarantine_guard
            .as_ref()
            .ok_or_else(invalid)?;
        let archive = self.receipt.record.archive(&self.root)?;
        let previous = registry::read_normal_shallow(
            &self.epoch().catalog,
            &registry::read_file(&archive.join("selection.json"), registry::MAX_REGISTRY)?,
        )?;
        let mut refs = previous.retained_quarantines;
        refs.push(guard.clone());
        refs.sort();
        let source = archives::read(&self.root, &self.epoch().catalog, &refs, stop)?;
        let target = archives::read(
            &candidate.backup.root,
            &self.epoch().catalog,
            candidate.backup.manifest().retained_quarantines(),
            stop,
        )?;
        let closure = source.merge(target)?;
        archives::write(&self.root, &closure, stop)?;
        let record = RegistryRecord::with_quarantines(
            self.epoch().clone(),
            candidate.intent.clone(),
            owners,
            candidate.backup.manifest().current().cloned(),
            closure.references().to_vec(),
            closure.max_revision(),
        )?;
        if selected.as_ref().is_some_and(|actual| actual != &record) {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        let bytes = record.bytes()?;
        let dir = database::epoch_directory(&candidate.backup.root, self.epoch())?;
        replacement::write_exact_or_new(
            &dir.join("epoch-manifest.json"),
            &encode(self.epoch(), 65536)?,
        )?;
        replacement::write_exact_or_new(
            &candidate.backup.root.join("replacement-record.json"),
            &bytes,
        )?;
        let staged = self
            .root
            .join(format!("project-store-registry-{}.staged", record.instance));
        if selected.is_none() {
            replacement::write_exact_or_new(&staged, &bytes)?;
        }
        let connection = database::connect(&candidate.backup.store.db.path, false)?;
        database::validate_header(&connection, self.epoch())?;
        database::enable_writer(&connection)?;
        if self.require_restore_state(&candidate.intent, stop)? != selected {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        checkpoint(stop)?;
        if selected.is_none() {
            fs::rename(&staged, self.root.join(registry::REGISTRY_FILE))
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        }
        let observed = registry::read(&self.root, &self.epoch().catalog)
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if observed.record.as_ref() != Some(&record) || observed.selection != record.selection()? {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let db = Database {
            connection,
            path: candidate.backup.store.db.path.clone(),
            epoch: self.epoch().clone(),
            root: self.root.clone(),
            selection: Some(observed.selection),
            life: Rc::new(Lifetime {
                _lock: Rc::clone(&self.life._lock),
                _instance_lock: Some(Rc::clone(&candidate.backup.store.db.life._lock)),
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::clone(&self.life.reader_admissions),
            }),
        };
        let fresh = db
            .read_connection()
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        let actual = backup::identity::capture(&fresh, &db.epoch, &AtomicBool::new(false))
            .and_then(|state| {
                state.digest_with_quarantines(candidate.backup.manifest().retained_quarantines())
            })
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if actual != candidate.intent.snapshot_digest {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        drop(fresh);
        Ok((
            ProjectStore { db },
            ReplacementReceipt::from_record(&record)?,
        ))
    }
    fn restore_intent(
        &self,
        operation: &OperationId,
        expected: &RegistrySelection,
        snapshot: &str,
    ) -> StoreResult<ReplacementIntent> {
        if expected != self.receipt.selected() {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        ReplacementIntent::restore(
            operation.clone(),
            expected.clone(),
            archives::QuarantineReference::from_record(&self.receipt.record)?,
            snapshot.to_owned(),
        )
    }
    fn require_restore_state(
        &self,
        intent: &ReplacementIntent,
        stop: &AtomicBool,
    ) -> StoreResult<Option<RegistryRecord>> {
        intent.validate()?;
        if intent.expected != self.receipt.selected
            || intent.quarantine_guard.as_ref()
                != Some(&archives::QuarantineReference::from_record(
                    &self.receipt.record,
                )?)
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let observed = registry::read(&self.root, &self.epoch().catalog)?;
        let selected = if observed.selection == intent.expected
            && observed.quarantine.as_ref() == Some(&self.receipt.record)
        {
            None
        } else if let Some(record) = observed.record
            && record.intent == *intent
        {
            Some(record)
        } else {
            return Err(failure(StoreErrorCode::CurrentConflict));
        };
        // Observational equality is an explicit whole-held-instance guard; it
        // never asserts byte identity or semantic absence for unreadable SQL.
        let (current, evidence) = capture(&self.path, self.epoch(), stop)?;
        if current != self.receipt.record.current
            || evidence.len() != self.receipt.record.evidence_length
            || digest("project-quarantine-evidence", &evidence)
                != self.receipt.record.evidence_digest
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        let archive = self.receipt.record.archive(&self.root)?;
        if registry::read_file(&archive.join("record.json"), registry::MAX_REGISTRY)?
            != self.receipt.record.bytes()?
            || registry::read_file(&archive.join("evidence.json"), super::model::MAX_EVIDENCE)?
                != evidence
        {
            return Err(invalid());
        }
        Ok(selected)
    }
}
impl ProjectStore {
    pub fn retained_quarantines(&self) -> StoreResult<Vec<archives::QuarantineReference>> {
        self.db.ensure_idle()?;
        Ok(registry::read(&self.db.root, &self.db.epoch.catalog)?.retained_quarantines)
    }
}
