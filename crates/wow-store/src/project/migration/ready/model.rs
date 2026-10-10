//! Data-only preparation intent and mapped roots; owner authority is held separately.
use crate::project::{
    AcknowledgmentState, CurrentPublication, EpochManifest, RetentionRoot, StoreGenerationId,
    model::{MAX_GENERATIONS, digest, encode, failure, hashed, invalid},
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};

pub(super) const MAX_METADATA: usize = 4 * 1024 * 1024;
pub(super) const INTENT_FILE: &str = "migration-ready-intent.json";
pub(super) const RECORD_FILE: &str = "migration-ready-record.json";
pub(super) const OUTPUT_DIRECTORY: &str = "ready-artifact";

/// Original archived pin and its exact reconstructed target-epoch counterpart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappedMigrationRoot {
    pub(super) source: RetentionRoot,
    pub(super) target: RetentionRoot,
}
impl MappedMigrationRoot {
    pub fn source(&self) -> &RetentionRoot {
        &self.source
    }
    pub fn target(&self) -> &RetentionRoot {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CopyBinding {
    pub operation: OperationId,
    pub request_digest: String,
    pub manifest_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CurrentPlan {
    pub source: CurrentPublication,
    pub target: CurrentPublication,
    pub operation: OperationId,
    pub request_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PreparationIntent {
    pub schema: String,
    pub operation: OperationId,
    pub migration_request: String,
    pub migration_receipt_digest: String,
    pub source_epoch: EpochManifest,
    pub target_epoch: EpochManifest,
    pub source_snapshot: String,
    pub baseline_snapshot: String,
    pub copy: CopyBinding,
    pub generations: Vec<StoreGenerationId>,
    pub roots: Vec<MappedMigrationRoot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentPlan>,
    pub output_operation: OperationId,
}
impl PreparationIntent {
    pub fn bytes(&self) -> StoreResult<Vec<u8>> {
        if self.schema != "wow-store/project-migration-ready-intent/1"
            || !hashed(&self.migration_request, "project-migration-request")
            || !hashed(&self.migration_receipt_digest, "project-migration-receipt")
            || !hashed(&self.source_snapshot, "project-backup-snapshot")
            || !hashed(&self.baseline_snapshot, "project-backup-snapshot")
            || !hashed(&self.copy.manifest_digest, "project-migration-copy")
            || self.source_epoch.epoch_id() == self.target_epoch.epoch_id()
            || self.generations.len() > MAX_GENERATIONS as usize
            || self.generations.windows(2).any(|w| w[0] >= w[1])
            || self.roots.len() > 1024
            || self
                .roots
                .windows(2)
                .any(|w| w[0].target.root_id() >= w[1].target.root_id())
        {
            return Err(invalid());
        }
        encode(self, MAX_METADATA)
    }
    pub fn digest(&self) -> StoreResult<String> {
        Ok(digest("project-migration-ready-request", &self.bytes()?))
    }
}

/// Durable preparation evidence. It cannot create owner validation capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationReadyReceipt {
    pub(super) schema: String,
    pub(super) intent: PreparationIntent,
    pub(super) request_digest: String,
    pub(super) artifact_snapshot: String,
    pub(super) artifact: CopyBinding,
    pub(super) owner_validation_digest: String,
    pub(super) acknowledgment: AcknowledgmentState,
}
impl MigrationReadyReceipt {
    pub fn operation_id(&self) -> &OperationId {
        &self.intent.operation
    }
    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
    pub fn source_epoch(&self) -> &EpochManifest {
        &self.intent.source_epoch
    }
    pub fn target_epoch(&self) -> &EpochManifest {
        &self.intent.target_epoch
    }
    pub fn baseline_snapshot_digest(&self) -> &str {
        &self.intent.baseline_snapshot
    }
    pub fn artifact_snapshot_digest(&self) -> &str {
        &self.artifact_snapshot
    }
    pub fn mapped_roots(&self) -> &[MappedMigrationRoot] {
        &self.intent.roots
    }
    pub fn target_current(&self) -> Option<&CurrentPublication> {
        self.intent.current.as_ref().map(|p| &p.target)
    }
    pub fn canonical_bytes(&self) -> StoreResult<Vec<u8>> {
        if !hashed(&self.artifact_snapshot, "project-backup-snapshot") {
            return Err(failure(StoreErrorCode::IntegrityViolation));
        }
        encode(self, MAX_METADATA)
    }
}
