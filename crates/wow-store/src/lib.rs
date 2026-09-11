#![forbid(unsafe_code)]

//! Bounded synchronous durable storage primitives for exact framework artifacts.
//!
//! The crate owns physical SQLite details. Callers exchange validated typed records;
//! no raw connection, SQL, row identifier, PRAGMA, or transaction callback escapes.

mod error;
mod identity;
mod model;
mod store;

pub use error::{StoreError, StoreErrorCode, StoreResult};
pub use identity::{
    CatalogName, CatalogPath, LeaseId, LogicalEpoch, ObjectId, OperationId, RequestDigest,
};
pub use model::{
    CatalogChange, CatalogEntry, CatalogExpectation, CatalogMutation, CommitReceipt,
    GarbageCollectionReceipt, IntegrityReport, LeaseRecord, LogicalManifest, ObjectRecord,
    OperationBegin, OperationRecord, OperationState, PendingObject, StoreConfiguration,
    StoreLimits, WriteBatch,
};
pub use store::Store;
