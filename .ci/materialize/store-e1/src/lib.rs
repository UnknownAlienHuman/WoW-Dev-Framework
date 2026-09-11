#![forbid(unsafe_code)]

//! Persistent, content-addressed E1 storage with immutable snapshots and CAS refs.
//!
//! The caller supplies an explicit trusted local root. Every object and manifest is
//! hashed on read. Mutable publication uses an append-only, checksummed ref journal
//! under an inter-process lock; incomplete trailing records are ignored.

mod error;
mod identity;
mod model;
mod store;

pub use error::{StoreError, StoreErrorCode, StoreResult};
pub use model::{
    BlobId, RefName, RefUpdate, SnapshotEntry, SnapshotEntryInput, SnapshotId, StoreLimits,
    StoreSnapshot,
};
pub use store::Store;
