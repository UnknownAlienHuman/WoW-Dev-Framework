//! Separate readonly observation owner for held physical data.
use super::{
    super::{
        RecoveryReport,
        database::{self, Lifetime},
        model::*,
        registry,
    },
    current,
    model::*,
};
use crate::{StoreErrorCode, StoreResult};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs::OpenOptions,
    path::{Path, PathBuf},
    rc::Rc,
    sync::atomic::AtomicBool,
};

pub struct QuarantinedStore {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
    pub(super) receipt: QuarantineReceipt,
    pub(super) life: Rc<Lifetime>,
}
impl QuarantinedStore {
    pub fn open(
        root: impl AsRef<Path>,
        catalog: &RecordCatalog,
        stop: &AtomicBool,
    ) -> StoreResult<Self> {
        checkpoint(stop)?;
        let (root, admitted, life, path) = open_owned(root.as_ref(), catalog)?;
        Self::from_parts(root, path, admitted.quarantine.ok_or_else(invalid)?, life)
    }
    pub(super) fn from_parts(
        root: PathBuf,
        path: PathBuf,
        record: QuarantineRecord,
        life: Rc<Lifetime>,
    ) -> StoreResult<Self> {
        Ok(Self {
            root,
            path,
            receipt: QuarantineReceipt::new(record)?,
            life,
        })
    }
    pub fn receipt(&self) -> &QuarantineReceipt {
        &self.receipt
    }
    pub fn epoch(&self) -> &EpochManifest {
        &self.receipt.record.epoch
    }
    pub fn current_observation(&self, stop: &AtomicBool) -> StoreResult<CurrentObservation> {
        self.ensure_selected()?;
        checkpoint(stop)?;
        let result = self
            .snapshot()
            .and_then(|connection| current::observe(&connection));
        let current = match result {
            Ok(value) => value,
            Err(_) => CurrentObservation::Unreadable {
                reason: PointerReadFailure::QueryUnavailable,
            },
        };
        checkpoint(stop)?;
        self.ensure_selected()?;
        Ok(current)
    }
    /// Physical classifications only. They cannot approve a Project/Graph pair.
    pub fn recovery_report(&self, stop: &AtomicBool) -> StoreResult<RecoveryReport> {
        self.ensure_selected()?;
        checkpoint(stop)?;
        let connection = self.snapshot()?;
        let report = super::super::recovery::inspect_snapshot(&connection, self.epoch(), stop)?;
        connection
            .execute_batch("ROLLBACK")
            .map_err(crate::StoreError::database)?;
        self.ensure_selected()?;
        Ok(report)
    }
    fn snapshot(&self) -> StoreResult<rusqlite::Connection> {
        // Retain the compiled lifetime; no mutable SQL or normal read capability.
        let _held = &self.life;
        database::regular(&self.path, 1024 * 1024 * 1024)?;
        let connection = database::connect(&self.path, true)?;
        connection
            .execute_batch("BEGIN DEFERRED")
            .map_err(crate::StoreError::database)?;
        database::validate_header(&connection, self.epoch())?;
        Ok(connection)
    }
    fn ensure_selected(&self) -> StoreResult<()> {
        let actual = registry::read(&self.root, &self.epoch().catalog)?;
        if actual.selection != self.receipt.selected
            || actual.quarantine.as_ref() != Some(&self.receipt.record)
        {
            return Err(failure(StoreErrorCode::CurrentConflict));
        }
        Ok(())
    }
}

type OwnedRoot = (PathBuf, registry::AdmittedRegistry, Rc<Lifetime>, PathBuf);
pub(super) fn open_owned(root: &Path, catalog: &RecordCatalog) -> StoreResult<OwnedRoot> {
    let root = database::admitted_root(root)?;
    let lock_path = root.join("writer.lock");
    database::regular(&lock_path, 0)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock_path)
        .map_err(|_| invalid())?;
    lock.try_lock()
        .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
    let admitted = registry::read(&root, catalog)?;
    let dir = admitted.selection.directory(&root, &admitted.epoch)?;
    let instance_lock = match admitted.selection.instance_root(&root) {
        None => None,
        Some(base) => {
            let path = base.join("writer.lock");
            database::regular(&path, 0)?;
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|_| invalid())?;
            file.try_lock()
                .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
            Some(Rc::new(file))
        }
    };
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
        match std::fs::symlink_metadata(&sidecar) {
            Ok(_) => database::regular(&sidecar, 128 * 1024 * 1024)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err(invalid()),
        }
    }
    let life = Rc::new(Lifetime {
        _lock: Rc::new(lock),
        _instance_lock: instance_lock,
        leases: RefCell::new(BTreeMap::new()),
        lease_revision: Cell::new(0),
        reader_admissions: Rc::new(Cell::new(0)),
    });
    Ok((root, admitted, life, path))
}
