//! Versioned source-to-target identity mapping; no serialized activation capability.
use super::super::model::*;
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub(super) const MAX_METADATA: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MigrationMapping {
    pub(super) source_generation: StoreGenerationId,
    pub(super) target_generation: StoreGenerationId,
    pub(super) operation_id: OperationId,
    pub(super) request_digest: String,
}
impl MigrationMapping {
    pub fn source_generation(&self) -> &StoreGenerationId {
        &self.source_generation
    }
    pub fn target_generation(&self) -> &StoreGenerationId {
        &self.target_generation
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MigrationIntent {
    pub schema: String,
    pub operation_id: OperationId,
    pub source_archive_operation: OperationId,
    pub source_epoch: EpochManifest,
    pub source_snapshot_digest: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_current: Option<CurrentPublication>,
    pub target_epoch: EpochManifest,
    pub mappings: Vec<MigrationMapping>,
}
impl MigrationIntent {
    pub fn validate(&self) -> StoreResult<()> {
        OperationId::new(self.operation_id.as_str())?;
        OperationId::new(self.source_archive_operation.as_str())?;
        if self.schema != "wow-store/project-migration-intent/1"
            || !matches!(
                self.source_epoch.physical_profile(),
                PHYSICAL_PROFILE | RETAINED_PHYSICAL_PROFILE
            )
            || self.target_epoch.physical_profile() != GC_PHYSICAL_PROFILE
            || self.source_epoch.epoch_id == self.target_epoch.epoch_id
            || self.source_epoch.owner != self.target_epoch.owner
            || self.source_epoch.catalog != self.target_epoch.catalog
            || !hashed(&self.source_snapshot_digest, "project-backup-snapshot")
            || self.mappings.len() > MAX_GENERATIONS as usize
        {
            return Err(invalid());
        }
        let mut unique = BTreeMap::new();
        for mapping in &self.mappings {
            StoreGenerationId::parse(mapping.source_generation.as_str())?;
            StoreGenerationId::parse(mapping.target_generation.as_str())?;
            OperationId::new(mapping.operation_id.as_str())?;
            if !hashed(&mapping.request_digest, "project-request") {
                return Err(invalid());
            }
            let value = (&mapping.operation_id, &mapping.request_digest);
            if unique
                .insert(&mapping.target_generation, value)
                .is_some_and(|old| old != value)
            {
                return Err(invalid());
            }
        }
        if self
            .mappings
            .windows(2)
            .any(|w| w[0].source_generation >= w[1].source_generation)
        {
            return Err(invalid());
        }
        if let Some(current) = &self.source_current
            && (current.epoch_id != self.source_epoch.epoch_id
                || !self
                    .mappings
                    .iter()
                    .any(|m| m.source_generation == current.generation_id))
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn bytes(&self) -> StoreResult<Vec<u8>> {
        self.validate()?;
        encode(self, MAX_METADATA)
    }
    pub fn digest(&self) -> StoreResult<String> {
        Ok(digest("project-migration-request", &self.bytes()?))
    }
}

/// Original Current remains source evidence; this target pair is unactivated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationCurrentMapping {
    pub(super) source: CurrentPublication,
    pub(super) target_generation: StoreGenerationId,
    pub(super) target_validation: ValidationId,
}
impl MigrationCurrentMapping {
    pub fn source(&self) -> &CurrentPublication {
        &self.source
    }
    pub fn target_generation(&self) -> &StoreGenerationId {
        &self.target_generation
    }
    pub fn target_validation(&self) -> &ValidationId {
        &self.target_validation
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationReceipt {
    pub(super) schema: String,
    pub(super) intent: MigrationIntent,
    pub(super) request_digest: String,
    pub(super) target_snapshot_digest: String,
    pub(super) validations: BTreeMap<StoreGenerationId, ValidationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) current_mapping: Option<MigrationCurrentMapping>,
    pub(super) owner_validation_digest: String,
    pub(super) state: PublicationState,
    pub(super) acknowledgment: super::super::AcknowledgmentState,
}
impl MigrationReceipt {
    pub fn operation_id(&self) -> &OperationId {
        &self.intent.operation_id
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
    pub fn source_snapshot_digest(&self) -> &str {
        &self.intent.source_snapshot_digest
    }
    pub fn target_snapshot_digest(&self) -> &str {
        &self.target_snapshot_digest
    }
    pub fn mappings(&self) -> &[MigrationMapping] {
        &self.intent.mappings
    }
    pub fn validations(&self) -> &BTreeMap<StoreGenerationId, ValidationId> {
        &self.validations
    }
    pub fn current_mapping(&self) -> Option<&MigrationCurrentMapping> {
        self.current_mapping.as_ref()
    }
    pub fn state(&self) -> PublicationState {
        self.state
    }
    pub(super) fn bytes(&self) -> StoreResult<Vec<u8>> {
        if !hashed(&self.target_snapshot_digest, "project-backup-snapshot") {
            return Err(failure(StoreErrorCode::IntegrityViolation));
        }
        encode(self, MAX_METADATA)
    }
}
