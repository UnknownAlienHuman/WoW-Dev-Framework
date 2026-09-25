//! First executable manifested-partition ProjectStore profile. Domain adapters
//! own their records and validation; this owner supplies WAL, exact membership,
//! read leases, durable operation state and a single coherent current CAS.
mod database;
mod model;
mod publication;
mod read;
use crate::{OperationId, StoreError, StoreResult};
use database::Database;
pub use model::{
    CurrentPublication, CurrentRecordId, EpochId, EpochManifest, GenerationManifest,
    PHYSICAL_PROFILE, PartitionMember, PartitionRecord, PartitionVersionId, PublicationOperation,
    PublicationRequest, PublicationState, RECORD_PROFILE, RecordCatalog, StoreGenerationId,
    ValidationId,
};
pub use read::{ReadSelector, ReadSnapshot, ValidatedRead};
use std::{path::Path, sync::atomic::AtomicBool};

/// One process-local owner. All read snapshots share its OS-held writer lock;
/// no raw connection, SQL callback, source path or external reader is exposed.
pub struct ProjectStore {
    db: Database,
}
impl ProjectStore {
    pub fn create(
        root: impl AsRef<Path>,
        owner: &str,
        catalog: RecordCatalog,
    ) -> StoreResult<Self> {
        Database::create(root.as_ref(), owner, catalog).map(|db| Self { db })
    }
    pub fn open(root: impl AsRef<Path>, catalog: &RecordCatalog) -> StoreResult<Self> {
        Database::open(root.as_ref(), catalog).map(|db| Self { db })
    }
    /// Explicitly authorized creation; retries only reopen an already valid,
    /// exactly registered owned root, never an arbitrary database.
    pub fn open_or_create(
        root: impl AsRef<Path>,
        owner: &str,
        catalog: RecordCatalog,
    ) -> StoreResult<Self> {
        match std::fs::symlink_metadata(root.as_ref()) {
            Ok(_) => {
                let store = Self::open(root, &catalog)?;
                if store.epoch().owner() != owner {
                    return Err(model::invalid());
                }
                Ok(store)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Self::create(root, owner, catalog)
            }
            Err(_) => Err(model::invalid()),
        }
    }
    pub fn epoch(&self) -> &EpochManifest {
        &self.db.epoch
    }
    pub fn current(&self) -> StoreResult<Option<CurrentPublication>> {
        read::read_current(&self.db.connection, &self.db.epoch)
    }
    pub fn operation(&self, id: &OperationId) -> StoreResult<Option<PublicationOperation>> {
        read::read_operation(&self.db.connection, id, &self.db.epoch)
    }
    pub fn read(&self, selector: &ReadSelector, stop: &AtomicBool) -> StoreResult<ReadSnapshot> {
        ReadSnapshot::acquire(&self.db, selector, stop)
    }
    /// Explicit nonblocking checkpoint. Busy/remaining frames are reported; no
    /// loop revokes readers or grows a query's budget to make it succeed.
    pub fn checkpoint(&self, stop: &AtomicBool) -> StoreResult<CheckpointReport> {
        model::checkpoint(stop)?;
        let (busy, frames, checkpointed) = self
            .db
            .connection
            .query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, i64>(2)?,
                ))
            })
            .map_err(StoreError::database)?;
        Ok(CheckpointReport {
            busy: busy != 0,
            frames: frames.max(0) as u64,
            checkpointed: checkpointed.max(0) as u64,
            active_readers: self.db.life.leases.borrow().values().sum(),
        })
    }
}
#[derive(Debug, serde::Serialize)]
pub struct CheckpointReport {
    pub busy: bool,
    pub frames: u64,
    pub checkpointed: u64,
    pub active_readers: usize,
}
