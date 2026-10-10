//! Serialize-only manifest for an exact physical project-store backup.
use serde::Serialize;

use super::super::{
    RecoveryReport,
    model::{CurrentPublication, EpochManifest, PartitionVersionId, StoreGenerationId, encode},
    quarantine::archives::QuarantineReference,
};
use crate::project::source_authority::SourceAuthorityReference;
use crate::{OperationId, StoreResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackupManifest {
    pub(super) schema: String,
    pub(super) operation_id: OperationId,
    pub(super) request_digest: String,
    pub(super) epoch: EpochManifest,
    pub(super) snapshot_digest: String,
    pub(super) generations: Vec<StoreGenerationId>,
    pub(super) partitions: Vec<PartitionVersionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) current: Option<CurrentPublication>,
    pub(super) payload_digest: String,
    pub(super) payload_bytes: u64,
    pub(super) recovery: RecoveryReport,
    pub(super) object_closure: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) retained_quarantines: Vec<QuarantineReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(super) source_authorities: Vec<SourceAuthorityReference>,
}

impl BackupManifest {
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
    pub fn epoch(&self) -> &EpochManifest {
        &self.epoch
    }
    pub fn snapshot_digest(&self) -> &str {
        &self.snapshot_digest
    }
    pub fn generations(&self) -> &[StoreGenerationId] {
        &self.generations
    }
    pub fn partitions(&self) -> &[PartitionVersionId] {
        &self.partitions
    }
    pub fn current(&self) -> Option<&CurrentPublication> {
        self.current.as_ref()
    }
    pub fn payload_digest(&self) -> &str {
        &self.payload_digest
    }
    pub fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }
    pub fn recovery(&self) -> &RecoveryReport {
        &self.recovery
    }
    pub fn object_closure(&self) -> &str {
        &self.object_closure
    }
    pub fn retained_quarantines(&self) -> &[QuarantineReference] {
        &self.retained_quarantines
    }
    pub fn canonical_bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, 16 * 1024 * 1024)
    }
    pub fn source_authorities(&self) -> &[SourceAuthorityReference] {
        &self.source_authorities
    }
}
