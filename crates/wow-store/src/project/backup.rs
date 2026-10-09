//! Explicit new-root backup artifacts and owner-validated isolated restoration.
//! The SQLite backup API includes the held committed WAL view. Artifact roots
//! have no normal project registry until an explicit validated restore finishes.
pub(super) mod copy;
pub(super) mod identity;
mod model;
#[cfg(test)]
mod tests;
use super::{
    ProjectStore, ReadSelector, ReadSnapshot, ValidatedRead,
    database::{self, Database, Lifetime},
    model::*,
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use identity::{BackupState, capture};
pub use model::BackupManifest;
use sha2::{Digest, Sha256};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicBool,
};

/// An independently reopened artifact under an OS-held owner lock. Public
/// methods expose only verified reads; ordinary ProjectStore::open cannot adopt
/// this root, which has no project-store registry.
pub struct VerifiedBackup {
    pub(super) store: ProjectStore,
    pub(super) root: PathBuf,
    pub(super) manifest: BackupManifest,
}
impl ProjectStore {
    /// Write one whole inline snapshot to an explicitly selected new directory.
    /// Failed/cancelled artifacts remain for reconciliation; no blind cleanup or
    /// overwrite is performed. An existing root always refuses a new write.
    pub fn backup_to_new(
        &self,
        root: impl AsRef<Path>,
        operation_id: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<VerifiedBackup> {
        self.db.ensure_idle()?;
        checkpoint(stop)?;
        OperationId::new(operation_id.as_str())?;
        let source = self.db.read_connection()?;
        let state = capture(&source, &self.db.epoch, stop)?;
        let snapshot_digest = state.digest()?;
        let root = root.as_ref();
        database::create_private_directory(root)
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let root = database::admitted_root(root)?;
        let lock = new_lock(&root)?;
        write_new(
            &root.join("epoch-manifest.json"),
            &encode(&state.epoch, 65536)?,
        )?;
        let request_digest = request_digest(operation_id, &state.epoch, &snapshot_digest)?;
        // Durable intent precedes the first destination database write. A missing
        // final manifest never constitutes an accepted backup.
        write_new(
            &root.join("backup-intent.json"),
            &intent(operation_id, &request_digest, &snapshot_digest)?,
        )?;
        fs::create_dir(root.join("epochs"))
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let dir = database::epoch_directory(&root, &state.epoch)?;
        fs::create_dir(&dir).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let path = dir.join("project.sqlite");
        let file =
            File::create_new(&path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        file.sync_all()
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        drop(file);
        copy::copy(&source, &path, &state.epoch, stop)?;
        let db = open_artifact(&root, state.epoch.clone(), lock)?;
        let fresh = db.read_connection()?;
        let independent = capture(&fresh, &db.epoch, stop)?;
        if independent.digest()? != snapshot_digest {
            return Err(invalid());
        }
        drop(fresh);
        let (payload_digest, payload_bytes) = payload(&path, stop)?;
        let manifest = manifest(operation_id, state, payload_digest, payload_bytes)?;
        checkpoint(stop)?;
        write_new(
            &root.join("backup-manifest.json"),
            &manifest.canonical_bytes()?,
        )?;
        let result = VerifiedBackup {
            store: Self { db },
            root,
            manifest,
        };
        result.verify(stop)?;
        Ok(result)
    }
}
impl VerifiedBackup {
    pub fn manifest(&self) -> &BackupManifest {
        &self.manifest
    }

    /// Reconcile only the named operation and exact previously observed snapshot.
    /// All serialized observations are recomputed; flags in JSON authorize nothing.
    pub fn open(
        root: impl AsRef<Path>,
        catalog: &RecordCatalog,
        operation_id: &OperationId,
        expected_snapshot_digest: &str,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        OperationId::new(operation_id.as_str())?;
        if !hashed(expected_snapshot_digest, "project-backup-snapshot") {
            return Err(failure(StoreErrorCode::IdentifierInvalid));
        }
        let root = database::admitted_root(root.as_ref())?;
        if root
            .join("project-store-registry.json")
            .try_exists()
            .map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        let lock_path = root.join("writer.lock");
        database::regular(&lock_path, 0)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        lock.try_lock()
            .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
        let epoch = database::admit_epoch(
            &read_file(&root.join("epoch-manifest.json"), 65536)?,
            catalog,
        )?;
        let db = open_artifact(&root, epoch, lock)?;
        let snapshot = db.read_connection()?;
        let state = capture(&snapshot, &db.epoch, stop)?;
        if state.digest()? != expected_snapshot_digest {
            return Err(failure(StoreErrorCode::OperationConflict));
        }
        drop(snapshot);
        let (payload_digest, payload_bytes) = payload(&db.path, stop)?;
        let expected = manifest(operation_id, state, payload_digest, payload_bytes)?;
        if read_file(&root.join("backup-manifest.json"), 16 * 1024 * 1024)?
            != expected.canonical_bytes()?
        {
            return Err(invalid());
        }
        let backup = Self {
            store: ProjectStore { db },
            root,
            manifest: expected,
        };
        backup.verify(stop)?;
        Ok(backup)
    }

    /// Recompute body bytes, complete physical closure and the exact manifest.
    pub fn verify(&self, stop: &AtomicBool) -> StoreResult<()> {
        checkpoint(stop)?;
        if self
            .root
            .join("project-store-registry.json")
            .try_exists()
            .map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        check_sidecars(self.store.db.path.parent().ok_or_else(invalid)?)?;
        let (digest, bytes) = payload(&self.store.db.path, stop)?;
        if digest != self.manifest.payload_digest || bytes != self.manifest.payload_bytes {
            return Err(invalid());
        }
        if read_file(&self.root.join("backup-intent.json"), 65536)?
            != intent(
                &self.manifest.operation_id,
                &self.manifest.request_digest,
                &self.manifest.snapshot_digest,
            )?
            || read_file(&self.root.join("epoch-manifest.json"), 65536)?
                != encode(&self.store.db.epoch, 65536)?
            || read_file(&self.root.join("backup-manifest.json"), 16 * 1024 * 1024)?
                != self.manifest.canonical_bytes()?
        {
            return Err(invalid());
        }
        let source = self.store.db.read_connection()?;
        let actual = capture(&source, &self.store.db.epoch, stop)?;
        if actual.digest()? != self.manifest.snapshot_digest {
            return Err(invalid());
        }
        checkpoint(stop)
    }
    pub fn read(&self, selector: &ReadSelector, stop: &AtomicBool) -> StoreResult<ReadSnapshot> {
        // The immutable artifact owner keeps its lock; every selected manifest
        // and seal is revalidated by the ordinary held read. Whole-artifact
        // verification remains explicit, rather than repeated for each member.
        self.store.read(selector, stop)
    }

    /// Produce a separate, inactive private recovery path using the same native
    /// copy/verification route. This never replaces an existing root or epoch.
    pub fn restore_to_new(
        &self,
        root: impl AsRef<Path>,
        operation_id: &OperationId,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        self.verify(stop)?;
        self.store.backup_to_new(root, operation_id, stop)
    }

    /// Finish a new private candidate only after compiled adapters independently
    /// validated every retained generation. Original semantic IDs and durable
    /// attestations survive unchanged; no live registry is replaced.
    pub fn finish_restore(
        self,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<ProjectStore> {
        self.verify(stop)?;
        self.restore_validation_digest(checks, stop)?;
        let dir = database::epoch_directory(&self.root, &self.store.db.epoch)?;
        // A previous failed finish may have written this exact confined file.
        // Reconcile its bytes rather than deleting or silently substituting it.
        let epoch_path = dir.join("epoch-manifest.json");
        let epoch_bytes = encode(&self.store.db.epoch, 65536)?;
        if epoch_path.try_exists().map_err(|_| invalid())? {
            if read_file(&epoch_path, 65536)? != epoch_bytes {
                return Err(invalid());
            }
        } else {
            write_new(&epoch_path, &epoch_bytes)?;
        }
        let connection = database::connect(&self.store.db.path, false)?;
        database::enable_writer(&connection)?;
        database::validate_header(&connection, &self.store.db.epoch)?;
        checkpoint(stop)?;
        // This is a new private registry after both physical and owner checks.
        write_new(
            &self.root.join("project-store-registry.json"),
            &encode(&self.store.db.epoch, 65536)?,
        )?;
        Ok(ProjectStore {
            db: Database {
                connection,
                path: self.store.db.path.clone(),
                epoch: self.store.db.epoch.clone(),
                life: Rc::clone(&self.store.db.life),
                root: self.root.clone(),
                selection: Some(super::registry::RegistrySelection::from_bytes(
                    &epoch_bytes,
                    &self.store.db.epoch,
                    0,
                    None,
                )),
            },
        })
    }
    pub(super) fn restore_validation_digest(
        &self,
        checks: Vec<ValidatedRead>,
        stop: &AtomicBool,
    ) -> StoreResult<String> {
        let mut verified = BTreeSet::new();
        let mut validations = BTreeMap::new();
        for check in checks {
            checkpoint(stop)?;
            if check.epoch != self.store.db.epoch.epoch_id
                || &check.validation.checks != self.store.db.epoch.catalog.checks()
                || !verified.insert(check.validation.generation_id.clone())
            {
                return Err(invalid());
            }
            let read = self.read(
                &ReadSelector::Exact(check.validation.generation_id.clone()),
                stop,
            )?;
            if ValidationRecord::new(
                read.manifest(),
                self.store.db.epoch.catalog.checks().clone(),
            )? != check.validation
            {
                return Err(invalid());
            }
            validations.insert(
                check.validation.generation_id,
                check.validation.validation_id,
            );
        }
        if verified != self.manifest.generations.iter().cloned().collect() {
            return Err(invalid());
        }
        Ok(digest(
            "project-replacement-owners",
            &encode(&(self.manifest.snapshot_digest(), validations), 262144)?,
        ))
    }
}
fn manifest(
    operation_id: &OperationId,
    state: BackupState,
    payload_digest: String,
    payload_bytes: u64,
) -> StoreResult<BackupManifest> {
    let snapshot_digest = state.digest()?;
    Ok(BackupManifest {
        schema: "wow-store/project-backup/1".into(),
        operation_id: operation_id.clone(),
        request_digest: request_digest(operation_id, &state.epoch, &snapshot_digest)?,
        epoch: state.epoch,
        snapshot_digest,
        generations: state.generations,
        partitions: state.partitions,
        current: state.recovery.current().cloned(),
        payload_digest,
        payload_bytes,
        recovery: state.recovery,
        object_closure: "self-contained-inline-partitions".into(),
    })
}
fn request_digest(
    operation_id: &OperationId,
    epoch: &EpochManifest,
    snapshot_digest: &str,
) -> StoreResult<String> {
    Ok(digest(
        "project-backup-request",
        &encode(
            &(
                "wow-store/project-backup/1",
                operation_id,
                &epoch.epoch_id,
                snapshot_digest,
            ),
            65536,
        )?,
    ))
}
fn open_artifact(root: &Path, epoch: EpochManifest, lock: File) -> StoreResult<Database> {
    database::directory(&root.join("epochs"))?;
    let dir = database::epoch_directory(root, &epoch)?;
    database::directory(&dir)?;
    let path = dir.join("project.sqlite");
    database::regular(&path, 1024 * 1024 * 1024)?;
    // A completed artifact has flushed its whole image. Foreign WAL/journal
    // state cannot turn a modified artifact into the recorded backup body.
    check_sidecars(&dir)?;
    let connection = database::connect(&path, true)?;
    database::validate_header(&connection, &epoch)?;
    Ok(Database {
        connection,
        path,
        epoch,
        root: root.to_owned(),
        selection: None,
        life: Rc::new(Lifetime {
            _lock: Rc::new(lock),
            _instance_lock: None,
            leases: RefCell::new(BTreeMap::new()),
            lease_revision: Cell::new(0),
            reader_admissions: Rc::new(Cell::new(0)),
        }),
    })
}
fn check_sidecars(dir: &Path) -> StoreResult<()> {
    for suffix in ["-wal", "-shm", "-journal"] {
        let sidecar = dir.join(format!("project.sqlite{suffix}"));
        if sidecar.try_exists().map_err(|_| invalid())? {
            database::regular(
                &sidecar,
                if suffix == "-shm" {
                    128 * 1024 * 1024
                } else {
                    0
                },
            )?;
        }
    }
    Ok(())
}
fn intent(operation: &OperationId, request: &str, snapshot: &str) -> StoreResult<Vec<u8>> {
    encode(
        &(
            "wow-store/project-backup-intent/1",
            operation,
            request,
            snapshot,
        ),
        65536,
    )
}
fn new_lock(root: &Path) -> StoreResult<File> {
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(root.join("writer.lock"))
        .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    lock.try_lock()
        .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
    Ok(lock)
}
pub(super) fn write_new(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    let mut file =
        File::create_new(path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
}
fn read_file(path: &Path, max: usize) -> StoreResult<Vec<u8>> {
    database::regular(path, max as u64)?;
    let mut bytes = Vec::new();
    File::open(path)
        .and_then(|file| file.take(max as u64 + 1).read_to_end(&mut bytes))
        .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    if bytes.len() > max {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
fn payload(path: &Path, stop: &AtomicBool) -> StoreResult<(String, u64)> {
    database::regular(path, 1024 * 1024 * 1024)?;
    let mut file = File::open(path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    let mut hasher = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        checkpoint(stop)?;
        let n = file
            .read(&mut buffer)
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        if n == 0 {
            break;
        }
        total = total
            .checked_add(n as u64)
            .filter(|n| *n <= 1024 * 1024 * 1024)
            .ok_or_else(|| failure(StoreErrorCode::BudgetExceeded))?;
        hasher.update(&buffer[..n]);
    }
    let hex = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    Ok((format!("project-backup-payload:sha256:{hex}"), total))
}
