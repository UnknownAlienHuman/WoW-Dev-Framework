//! Typed guarded installation of one exact, independently validated READY epoch.
pub(in crate::project) mod io;
pub(in crate::project) mod model;
use super::{ReadyMigration, ValidatedMigration, plan};
use crate::project::{
    CurrentRecordId, ProjectStore, ReadSelector, ReadSnapshot, RegistrySelection,
    StoreGenerationId, ValidatedRead, VerifiedBackup, backup,
    database::{self, Database, Lifetime},
    model::{checkpoint, encode, failure, hashed, invalid},
    quarantine::archives,
    registry, replacement, source_authority,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
pub use model::MigrationSelectionReceipt;
use model::{EvidenceBinding, SelectionIntent, SelectionRecord};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicBool,
};

enum SelectionBody {
    Staged(ProjectStore),
    Selected(ProjectStore),
}
/// Held native target; decoded selection evidence cannot construct this capability.
pub struct MigrationSelectionCandidate {
    body: SelectionBody,
    intent: SelectionIntent,
    source_root: PathBuf,
    root: PathBuf,
    generations: Vec<StoreGenerationId>,
}
impl MigrationSelectionCandidate {
    pub fn request_digest(&self) -> StoreResult<String> {
        self.intent.digest()
    }
    pub fn target_generations(&self) -> Vec<StoreGenerationId> {
        self.generations.clone()
    }
    pub fn read_generation(
        &self,
        generation: &StoreGenerationId,
        stop: &AtomicBool,
    ) -> StoreResult<ReadSnapshot> {
        if self.generations.binary_search(generation).is_err() {
            return Err(failure(StoreErrorCode::GenerationMissing));
        }
        self.store()
            .read(&ReadSelector::Exact(generation.clone()), stop)
    }
    fn store(&self) -> &ProjectStore {
        match &self.body {
            SelectionBody::Staged(store) => store,
            SelectionBody::Selected(store) => store,
        }
    }
    fn verify_target(&self, ready: &ReadyMigration, stop: &AtomicBool) -> StoreResult<()> {
        if let SelectionBody::Staged(store) = &self.body {
            if store.db.root != self.root || store.db.selection.is_some() {
                return Err(invalid());
            }
            require_working_marker(&self.root, &self.intent)?;
        }
        if self.generations != ready.artifact().manifest().generations() {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        verify_body(self.store(), &self.intent, &self.generations, stop)
    }
}

impl ProjectStore {
    /// Persist an exact request, then copy the portable target once into a new instance.
    #[allow(clippy::too_many_arguments)]
    pub fn stage_ready_selection(
        &self,
        migration: &ValidatedMigration,
        ready: &ReadyMigration,
        operation: &OperationId,
        expected: &RegistrySelection,
        expected_current: Option<&CurrentRecordId>,
        stop: &AtomicBool,
    ) -> StoreResult<MigrationSelectionCandidate> {
        self.require_migration_source(expected, expected_current, migration.source(), stop)?;
        ready.verify_for_export(migration, stop)?;
        let admitted = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        let sources = source_authority::capture_source(
            &self.db.root,
            &admitted,
            migration.source().manifest().snapshot_digest(),
            stop,
        )?;
        let intent = intent_for(
            migration,
            ready,
            operation,
            expected,
            expected_current,
            &sources,
            stop,
        )?;
        let evidence = Evidence::capture(&self.db.root, migration, ready)?;
        // All shape, receipt and aggregate admission precedes the first directory effect.
        intent.bytes()?;
        let ledger_parent = self.db.root.join("migration-selections");
        let instance_parent = self.db.root.join("instances");
        admit_existing_parent(&ledger_parent)?;
        admit_existing_parent(&instance_parent)?;
        let ledger = ledger_parent.join(intent.instance()?);
        let root = instance_parent.join(intent.instance()?);
        require_missing(&ledger)?;
        require_missing(&root)?;
        checkpoint(stop)?;
        archives::directory(&ledger_parent)?;
        database::create_private_directory(&ledger)
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        evidence.write(&ledger, &intent)?;
        archives::directory(&instance_parent)?;
        let portable = self.export_ready_migration_to_new(
            migration,
            ready,
            &root,
            &intent.portable_operation,
            expected,
            expected_current,
            stop,
        )?;
        evidence.write(&root, &intent)?;
        let working = working_from_backup(portable, &intent, &self.db.life, stop)?;
        self.require_migration_source(expected, expected_current, migration.source(), stop)?;
        let candidate = MigrationSelectionCandidate {
            generations: ready.artifact().manifest().generations().to_vec(),
            body: SelectionBody::Staged(working),
            intent,
            source_root: self.db.root.clone(),
            root,
        };
        candidate.verify_target(ready, stop)?;
        Ok(candidate)
    }
    /// Reopen only the original complete candidate or the exact selected installation.
    /// A partial native copy is retained and refused, never recopied or deleted.
    pub fn reopen_ready_selection(
        &self,
        migration: &ValidatedMigration,
        ready: &ReadyMigration,
        operation: &OperationId,
        expected_request: &str,
        stop: &AtomicBool,
    ) -> StoreResult<MigrationSelectionCandidate> {
        checkpoint(stop)?;
        OperationId::new(operation.as_str())?;
        if !hashed(expected_request, "project-migration-selection-request") {
            return Err(invalid());
        }
        let id = registry::instance_id(operation)?;
        let ledger = self.db.root.join("migration-selections").join(&id);
        database::directory(&self.db.root.join("migration-selections"))?;
        database::directory(&ledger)?;
        let intent = io::read_intent(&ledger)?;
        if &intent.operation_id != operation || intent.digest()? != expected_request {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        io::read(&ledger, &intent)?;
        let root = self.db.root.join("instances").join(id);
        database::directory(&self.db.root.join("instances"))?;
        database::directory(&root)?;
        io::read(&root, &intent)?;
        require_binding(&root, &intent, migration, ready, stop)?;
        // Inspect committed selection before the old source's ensure_idle/guard.
        let observed = registry::read(&self.db.root, &intent.target_epoch.catalog)?;
        if observed.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        let body = if observed.selection == intent.expected {
            self.require_migration_source(
                &intent.expected,
                intent.expected_current.as_ref(),
                migration.source(),
                stop,
            )?;
            let working = match fs::symlink_metadata(root.join("migration-selection-working.json"))
            {
                Ok(_) => open_working_target(&root, &intent, &self.db.life, ready, stop)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let portable = VerifiedBackup::open(
                        &root,
                        &intent.target_epoch.catalog,
                        &intent.portable_operation,
                        &intent.portable_snapshot,
                        stop,
                    )?;
                    working_from_backup(portable, &intent, &self.db.life, stop)?
                }
                Err(_) => return Err(invalid()),
            };
            SelectionBody::Staged(working)
        } else if observed
            .migration
            .as_ref()
            .is_some_and(|record| record.intent == intent)
        {
            SelectionBody::Selected(self.selected_read_owner(&observed)?)
        } else {
            return Err(failure(StoreErrorCode::CurrentConflict));
        };
        let candidate = MigrationSelectionCandidate {
            body,
            intent,
            source_root: self.db.root.clone(),
            root,
            generations: ready.artifact().manifest().generations().to_vec(),
        };
        candidate.verify_target(ready, stop)?;
        Ok(candidate)
    }
    /// Select/adopt the exact target only after every compiled target owner validates.
    /// Already selected outcomes reconcile without a second selector or SQL activation.
    pub fn activate_ready_selection(
        &mut self,
        migration: &ValidatedMigration,
        ready: &ReadyMigration,
        candidate: MigrationSelectionCandidate,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<MigrationSelectionReceipt> {
        checkpoint(stop)?;
        if candidate.source_root != self.db.root
            || candidate.root
                != self
                    .db
                    .root
                    .join("instances")
                    .join(candidate.intent.instance()?)
            || !Rc::ptr_eq(
                &candidate.store().db.life.reader_admissions,
                &self.db.life.reader_admissions,
            )
            || !self.db.connection.is_autocommit()
        {
            return Err(invalid());
        }
        require_binding(&candidate.root, &candidate.intent, migration, ready, stop)?;
        io::read(&candidate.root, &candidate.intent)?;
        candidate.verify_target(ready, stop)?;
        let owners = backup::owner_validation_digest(
            candidate.store(),
            &candidate.generations,
            &candidate.intent.portable_snapshot,
            checks,
            stop,
        )?;
        let record = SelectionRecord::new(candidate.intent.clone(), owners)?;
        let observed = registry::read(&self.db.root, &record.epoch.catalog)?;
        if observed.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        let committed = if observed.selection == record.intent.expected {
            if !matches!(&candidate.body, SelectionBody::Staged(_)) {
                return Err(invalid());
            }
            self.require_migration_source(
                &record.intent.expected,
                record.intent.expected_current.as_ref(),
                migration.source(),
                stop,
            )?;
            false
        } else if observed.migration.as_ref() == Some(&record) {
            if !matches!(&candidate.body, SelectionBody::Selected(_)) {
                return Err(failure(StoreErrorCode::OutcomeUnknown));
            }
            true
        } else {
            return Err(failure(StoreErrorCode::CurrentConflict));
        };
        let bytes = record.bytes()?;
        let selection = record.selection()?;
        let staged = self
            .db
            .root
            .join(format!("project-store-registry-{}.staged", record.instance));
        if !committed {
            let sources =
                source_authority::read(&candidate.root, &record.intent.source_authorities, stop)?;
            let dir = database::epoch_directory(&candidate.root, &record.epoch)?;
            replacement::write_exact_or_new(
                &dir.join("epoch-manifest.json"),
                &encode(&record.epoch, 65536)?,
            )?;
            source_authority::write(&self.db.root, &sources, stop)?;
            replacement::write_exact_or_new(
                &candidate.root.join("migration-selection-record.json"),
                &bytes,
            )?;
            replacement::write_exact_or_new(&staged, &bytes)?;
        }
        let connection = database::connect(&candidate.store().db.path, false)?;
        database::validate_header(&connection, &record.epoch)?;
        database::enable_writer(&connection)?;
        if committed {
            if registry::read(&self.db.root, &record.epoch.catalog)?
                .migration
                .as_ref()
                != Some(&record)
            {
                return Err(failure(StoreErrorCode::CurrentConflict));
            }
        } else {
            self.require_migration_source(
                &record.intent.expected,
                record.intent.expected_current.as_ref(),
                migration.source(),
                stop,
            )?;
        }
        checkpoint(stop)?;
        // No writes or cancellation branches between the final guard and OS dispatch.
        if !committed {
            fs::rename(&staged, self.db.root.join(registry::REGISTRY_FILE))
                .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        }
        let observed = registry::read(&self.db.root, &record.epoch.catalog)
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if observed.selection != selection || observed.migration.as_ref() != Some(&record) {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        let life = match &candidate.body {
            SelectionBody::Staged(target) => Rc::clone(&target.db.life),
            SelectionBody::Selected(target) => Rc::clone(&target.db.life),
        };
        let db = Database {
            connection,
            path: candidate.store().db.path.clone(),
            epoch: record.epoch.clone(),
            root: self.db.root.clone(),
            selection: Some(selection),
            life,
        };
        let fresh = db
            .read_connection()
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        let installed = backup::identity::capture(&fresh, &db.epoch, &AtomicBool::new(false))
            .and_then(|state| state.digest_with_authorities(&[], &record.intent.source_authorities))
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        if installed != record.intent.portable_snapshot {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        drop(fresh);
        self.db = db;
        MigrationSelectionReceipt::from_record(&record)
    }
    /// Exact historical installation evidence; later publications do not change it.
    pub fn migration_selection_receipt(
        &self,
        operation: &OperationId,
        expected_request: &str,
    ) -> StoreResult<Option<MigrationSelectionReceipt>> {
        OperationId::new(operation.as_str())?;
        if !hashed(expected_request, "project-migration-selection-request") {
            return Err(invalid());
        }
        let observed = registry::read(&self.db.root, &self.db.epoch.catalog)?;
        if observed.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        match observed.migration {
            Some(record) if &record.intent.operation_id == operation => {
                if record.request_digest != expected_request {
                    return Err(failure(StoreErrorCode::OperationConflict));
                }
                MigrationSelectionReceipt::from_record(&record).map(Some)
            }
            _ => Ok(None),
        }
    }
    fn selected_read_owner(
        &self,
        admitted: &registry::AdmittedRegistry,
    ) -> StoreResult<ProjectStore> {
        let dir = admitted
            .selection
            .directory(&self.db.root, &admitted.epoch)?;
        if registry::read_file(&dir.join("epoch-manifest.json"), 65536)?
            != encode(&admitted.epoch, 65536)?
        {
            return Err(invalid());
        }
        let path = dir.join("project.sqlite");
        database::regular(&path, 1024 * 1024 * 1024)?;
        for name in [
            "project.sqlite-wal",
            "project.sqlite-shm",
            "project.sqlite-journal",
        ] {
            let sidecar = dir.join(name);
            match fs::symlink_metadata(&sidecar) {
                Ok(_) => database::regular(&sidecar, 128 * 1024 * 1024)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(invalid()),
            }
        }
        let life = if self.db.selection.as_ref() == Some(&admitted.selection)
            && self.db.epoch == admitted.epoch
            && self.db.path == path
        {
            Rc::clone(&self.db.life)
        } else {
            let instance = admitted
                .selection
                .instance_root(&self.db.root)
                .ok_or_else(invalid)?;
            let lock_path = instance.join("writer.lock");
            database::regular(&lock_path, 0)?;
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .open(lock_path)
                .map_err(|_| invalid())?;
            lock.try_lock()
                .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
            Rc::new(Lifetime {
                _lock: Rc::clone(&self.db.life._lock),
                _instance_lock: Some(Rc::new(lock)),
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::clone(&self.db.life.reader_admissions),
            })
        };
        let connection = database::connect(&path, true)?;
        database::validate_header(&connection, &admitted.epoch)?;
        Ok(ProjectStore {
            db: Database {
                connection,
                path,
                epoch: admitted.epoch.clone(),
                root: self.db.root.clone(),
                selection: Some(admitted.selection.clone()),
                life,
            },
        })
    }
}

struct Evidence {
    source: Vec<u8>,
    migration: Vec<u8>,
    ready: Vec<u8>,
}
fn working_marker(intent: &SelectionIntent) -> StoreResult<Vec<u8>> {
    encode(
        &(
            "wow-store/project-migration-selection-working/1",
            intent.digest()?,
            intent.target_epoch.epoch_id(),
            &intent.portable_snapshot,
        ),
        registry::MAX_REGISTRY,
    )
}
fn require_working_marker(root: &Path, intent: &SelectionIntent) -> StoreResult<()> {
    if registry::read_file(
        &root.join("migration-selection-working.json"),
        registry::MAX_REGISTRY,
    )? != working_marker(intent)?
    {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    Ok(())
}
fn working_from_backup(
    portable: VerifiedBackup,
    intent: &SelectionIntent,
    source: &Rc<Lifetime>,
    stop: &AtomicBool,
) -> StoreResult<ProjectStore> {
    portable.verify(stop)?;
    if portable.manifest().epoch() != &intent.target_epoch
        || portable.manifest().operation_id() != &intent.portable_operation
        || portable.manifest().snapshot_digest() != intent.portable_snapshot
        || portable.manifest().source_authorities() != intent.source_authorities
        || !portable.manifest().retained_quarantines().is_empty()
        || portable.manifest().current() != intent.activated_current.as_ref()
    {
        return Err(invalid());
    }
    let dir = database::epoch_directory(&portable.root, &intent.target_epoch)?;
    replacement::write_exact_or_new(
        &dir.join("epoch-manifest.json"),
        &encode(&intent.target_epoch, 65536)?,
    )?;
    // Persist the private handoff before writable WAL configuration.
    replacement::write_exact_or_new(
        &portable.root.join("migration-selection-working.json"),
        &working_marker(intent)?,
    )?;
    checkpoint(stop)?;
    let connection = database::connect(&portable.store.db.path, false)?;
    database::validate_header(&connection, &intent.target_epoch)?;
    database::enable_writer(&connection)?;
    Ok(ProjectStore {
        db: Database {
            connection,
            path: portable.store.db.path.clone(),
            epoch: intent.target_epoch.clone(),
            root: portable.root.clone(),
            selection: None,
            life: Rc::new(Lifetime {
                _lock: Rc::clone(&source._lock),
                _instance_lock: Some(Rc::clone(&portable.store.db.life._lock)),
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::clone(&source.reader_admissions),
            }),
        },
    })
}
fn open_working_target(
    root: &Path,
    intent: &SelectionIntent,
    source: &Rc<Lifetime>,
    ready: &ReadyMigration,
    stop: &AtomicBool,
) -> StoreResult<ProjectStore> {
    require_working_marker(root, intent)?;
    let root = database::admitted_root(root)?;
    require_missing(&root.join(registry::REGISTRY_FILE))?;
    if registry::read_file(&root.join("epoch-manifest.json"), 65536)?
        != encode(&intent.target_epoch, 65536)?
    {
        return Err(invalid());
    }
    database::directory(&root.join("epochs"))?;
    let dir = database::epoch_directory(&root, &intent.target_epoch)?;
    database::directory(&dir)?;
    if registry::read_file(&dir.join("epoch-manifest.json"), 65536)?
        != encode(&intent.target_epoch, 65536)?
    {
        return Err(invalid());
    }
    let path = dir.join("project.sqlite");
    database::regular(&path, 1024 * 1024 * 1024)?;
    for name in [
        "project.sqlite-wal",
        "project.sqlite-shm",
        "project.sqlite-journal",
    ] {
        match fs::symlink_metadata(dir.join(name)) {
            Ok(_) => database::regular(&dir.join(name), 128 * 1024 * 1024)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(invalid()),
        }
    }
    let lock_path = root.join("writer.lock");
    database::regular(&lock_path, 0)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock_path)
        .map_err(|_| invalid())?;
    lock.try_lock()
        .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
    let connection = database::connect(&path, true)?;
    database::validate_header(&connection, &intent.target_epoch)?;
    let mut store = ProjectStore {
        db: Database {
            connection,
            path: path.clone(),
            epoch: intent.target_epoch.clone(),
            root: root.clone(),
            selection: None,
            life: Rc::new(Lifetime {
                _lock: Rc::clone(&source._lock),
                _instance_lock: Some(Rc::new(lock)),
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::clone(&source.reader_admissions),
            }),
        },
    };
    // Refuse foreign or partial mutable bodies before any writer configuration.
    verify_body(
        &store,
        intent,
        ready.artifact().manifest().generations(),
        stop,
    )?;
    checkpoint(stop)?;
    let connection = database::connect(&path, false)?;
    database::validate_header(&connection, &intent.target_epoch)?;
    database::enable_writer(&connection)?;
    store.db.connection = connection;
    Ok(store)
}
fn verify_body(
    store: &ProjectStore,
    intent: &SelectionIntent,
    generations: &[StoreGenerationId],
    stop: &AtomicBool,
) -> StoreResult<()> {
    let connection = store.db.read_connection()?;
    let state = backup::identity::capture(&connection, store.epoch(), stop)?;
    if store.epoch() != &intent.target_epoch
        || state.generations != generations
        || state.recovery.current() != intent.activated_current.as_ref()
        || state.digest_with_authorities(&[], &intent.source_authorities)?
            != intent.portable_snapshot
    {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    Ok(())
}
impl Evidence {
    fn capture(
        root: &Path,
        migration: &ValidatedMigration,
        ready: &ReadyMigration,
    ) -> StoreResult<Self> {
        Ok(Self {
            source: registry::read_file(
                &root.join(registry::REGISTRY_FILE),
                registry::MAX_REGISTRY,
            )?,
            migration: encode(migration.receipt(), super::model::MAX_METADATA)?,
            ready: ready.receipt().canonical_bytes()?,
        })
    }
    fn write(&self, root: &Path, intent: &SelectionIntent) -> StoreResult<()> {
        io::write(root, intent, &self.source, &self.migration, &self.ready)
    }
}
#[allow(clippy::too_many_arguments)]
fn intent_for(
    migration: &ValidatedMigration,
    ready: &ReadyMigration,
    operation: &OperationId,
    expected: &RegistrySelection,
    expected_current: Option<&CurrentRecordId>,
    sources: &source_authority::AuthoritySet,
    stop: &AtomicBool,
) -> StoreResult<SelectionIntent> {
    ready.verify_for_export(migration, stop)?;
    let source = migration.source().manifest();
    let original_authority =
        sources.origin_reference(source.epoch(), expected, source.snapshot_digest())?;
    let connection = ready.artifact().store.db.read_connection()?;
    let state = backup::identity::capture(&connection, ready.artifact().manifest().epoch(), stop)?;
    let portable_snapshot = state.digest_with_authorities(&[], sources.references())?;
    let intent = SelectionIntent {
        schema: "wow-store/project-migration-selection-intent/1".into(),
        operation_id: operation.clone(),
        expected: expected.clone(),
        expected_current: expected_current.cloned(),
        source_epoch: source.epoch().clone(),
        target_epoch: ready.artifact().manifest().epoch().clone(),
        source_snapshot: source.snapshot_digest().to_owned(),
        portable_operation: plan::derived_operation(operation, "selection-copy")?,
        portable_snapshot,
        source_authorities: sources.references().to_vec(),
        original_authority,
        migration_evidence: EvidenceBinding::from_bytes(&encode(
            migration.receipt(),
            super::model::MAX_METADATA,
        )?)?,
        ready_evidence: EvidenceBinding::from_bytes(&ready.receipt().canonical_bytes()?)?,
        activated_current: ready.artifact().manifest().current().cloned(),
    };
    intent.validate()?;
    Ok(intent)
}
fn require_binding(
    root: &Path,
    intent: &SelectionIntent,
    migration: &ValidatedMigration,
    ready: &ReadyMigration,
    stop: &AtomicBool,
) -> StoreResult<()> {
    let sources = source_authority::read(root, &intent.source_authorities, stop)?;
    let expected = intent_for(
        migration,
        ready,
        &intent.operation_id,
        &intent.expected,
        intent.expected_current.as_ref(),
        &sources,
        stop,
    )?;
    if expected != *intent {
        return Err(failure(StoreErrorCode::OperationConflict));
    }
    Ok(())
}
fn require_missing(path: &Path) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(failure(StoreErrorCode::OperationConflict)),
        Err(_) => Err(invalid()),
    }
}
fn admit_existing_parent(path: &Path) -> StoreResult<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => database::directory(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(invalid()),
    }
}
