//! Bounded cross-epoch selection identities; serialized data grants no capability.
use crate::project::{
    AcknowledgmentState, CurrentPublication, CurrentRecordId, EpochManifest, RegistrySelection,
    database,
    migration::plan,
    model::{
        GC_PHYSICAL_PROFILE, PHYSICAL_PROFILE, RETAINED_PHYSICAL_PROFILE, digest, encode, failure,
        hashed, invalid,
    },
    registry,
    source_authority::{self, SourceAuthorityReference},
};
use crate::{OperationId, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const MAX_EVIDENCE: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::project) struct EvidenceBinding {
    pub(in crate::project) digest: String,
    pub(in crate::project) byte_length: usize,
}
impl EvidenceBinding {
    pub(in crate::project) fn from_bytes(bytes: &[u8]) -> StoreResult<Self> {
        if bytes.len() > MAX_EVIDENCE {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let binding = Self {
            digest: digest("project-migration-selection-evidence", bytes),
            byte_length: bytes.len(),
        };
        binding.validate()?;
        Ok(binding)
    }
    pub(in crate::project) fn validate(&self) -> StoreResult<()> {
        if !hashed(&self.digest, "project-migration-selection-evidence")
            || !(1..=MAX_EVIDENCE).contains(&self.byte_length)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(in crate::project) fn verify_bytes(&self, bytes: &[u8]) -> StoreResult<()> {
        self.validate()?;
        if Self::from_bytes(bytes)? != *self {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::project) struct SelectionIntent {
    pub(in crate::project) schema: String,
    pub(in crate::project) operation_id: OperationId,
    pub(in crate::project) expected: RegistrySelection,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::project) expected_current: Option<CurrentRecordId>,
    pub(in crate::project) source_epoch: EpochManifest,
    pub(in crate::project) target_epoch: EpochManifest,
    pub(in crate::project) source_snapshot: String,
    pub(in crate::project) portable_operation: OperationId,
    pub(in crate::project) portable_snapshot: String,
    pub(in crate::project) source_authorities: Vec<SourceAuthorityReference>,
    pub(in crate::project) original_authority: SourceAuthorityReference,
    pub(in crate::project) migration_evidence: EvidenceBinding,
    pub(in crate::project) ready_evidence: EvidenceBinding,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(in crate::project) activated_current: Option<CurrentPublication>,
}
impl SelectionIntent {
    pub(in crate::project) fn validate(&self) -> StoreResult<()> {
        OperationId::new(self.operation_id.as_str())?;
        OperationId::new(self.portable_operation.as_str())?;
        self.expected.validate()?;
        if let Some(current) = &self.expected_current {
            CurrentRecordId::parse(current.as_str())?;
        }
        if self.schema != "wow-store/project-migration-selection-intent/1"
            || self.expected.is_quarantined()
            || self.source_epoch.epoch_id() != self.expected.epoch()
            || !matches!(
                self.source_epoch.physical_profile(),
                PHYSICAL_PROFILE | RETAINED_PHYSICAL_PROFILE
            )
            || self.target_epoch.physical_profile() != GC_PHYSICAL_PROFILE
            || self.source_epoch.epoch_id() == self.target_epoch.epoch_id()
            || self.source_epoch.owner != self.target_epoch.owner
            || self.source_epoch.catalog != self.target_epoch.catalog
            || !hashed(&self.source_snapshot, "project-backup-snapshot")
            || !hashed(&self.portable_snapshot, "project-backup-snapshot")
            || self.portable_operation
                != plan::derived_operation(&self.operation_id, "selection-copy")?
            || self.source_authorities.is_empty()
        {
            return Err(invalid());
        }
        database::admit_epoch(
            &encode(&self.source_epoch, 65536)?,
            &self.source_epoch.catalog,
        )?;
        database::admit_epoch(
            &encode(&self.target_epoch, 65536)?,
            &self.target_epoch.catalog,
        )?;
        source_authority::validate_references(&self.source_authorities)?;
        self.original_authority.validate()?;
        if !self.source_authorities.contains(&self.original_authority) {
            return Err(invalid());
        }
        self.migration_evidence.validate()?;
        self.ready_evidence.validate()?;
        if let Some(current) = &self.activated_current {
            if current.epoch_id != self.target_epoch.epoch_id || current.predecessor.is_some() {
                return Err(invalid());
            }
            let bytes = encode(
                &(
                    &current.epoch_id,
                    &current.generation_id,
                    &current.validation_id,
                    current.predecessor.iter().collect::<Vec<_>>(),
                ),
                4096,
            )?;
            if current.record_id != CurrentRecordId::derive(&bytes) {
                return Err(invalid());
            }
        }
        Ok(())
    }
    pub(in crate::project) fn bytes(&self) -> StoreResult<Vec<u8>> {
        self.validate()?;
        encode(self, registry::MAX_REGISTRY)
    }
    pub(in crate::project) fn digest(&self) -> StoreResult<String> {
        Ok(digest(
            "project-migration-selection-request",
            &self.bytes()?,
        ))
    }
    pub(in crate::project) fn instance(&self) -> StoreResult<String> {
        registry::instance_id(&self.operation_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::project) struct SelectionRecord {
    pub(in crate::project) schema: String,
    pub(in crate::project) epoch: EpochManifest,
    pub(in crate::project) intent: SelectionIntent,
    pub(in crate::project) revision: u64,
    pub(in crate::project) instance: String,
    pub(in crate::project) request_digest: String,
    pub(in crate::project) owner_validation_digest: String,
}
impl SelectionRecord {
    pub(in crate::project) fn new(intent: SelectionIntent, owners: String) -> StoreResult<Self> {
        intent.validate()?;
        let record = Self {
            schema: "wow-store/project-registry/6".into(),
            epoch: intent.target_epoch.clone(),
            revision: intent
                .expected
                .revision()
                .checked_add(1)
                .ok_or_else(invalid)?,
            instance: intent.instance()?,
            request_digest: intent.digest()?,
            owner_validation_digest: owners,
            intent,
        };
        record.validate()?;
        Ok(record)
    }
    pub(in crate::project) fn validate(&self) -> StoreResult<()> {
        self.intent.validate()?;
        let revision = self
            .intent
            .expected
            .revision()
            .checked_add(1)
            .ok_or_else(invalid)?;
        if self.schema != "wow-store/project-registry/6"
            || self.epoch != self.intent.target_epoch
            || self.revision != revision
            || self.instance != self.intent.instance()?
            || self.request_digest != self.intent.digest()?
            || !hashed(&self.owner_validation_digest, "project-replacement-owners")
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(in crate::project) fn bytes(&self) -> StoreResult<Vec<u8>> {
        self.validate()?;
        encode(self, registry::MAX_REGISTRY)
    }
    pub(in crate::project) fn selection(&self) -> StoreResult<RegistrySelection> {
        Ok(RegistrySelection::from_bytes(
            &self.bytes()?,
            &self.epoch,
            self.revision,
            Some(self.instance.clone()),
        ))
    }
    pub(in crate::project) fn instance_root(&self, root: &Path) -> PathBuf {
        root.join("instances").join(&self.instance)
    }
}

/// Immutable installation evidence; it does not assert caller acknowledgment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MigrationSelectionReceipt {
    operation_id: OperationId,
    request_digest: String,
    previous: RegistrySelection,
    selected: RegistrySelection,
    source_epoch: EpochManifest,
    target_epoch: EpochManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    activated_current: Option<CurrentPublication>,
    portable_snapshot_digest: String,
    acknowledgment: AcknowledgmentState,
    durability: String,
}
impl MigrationSelectionReceipt {
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }
    pub fn previous(&self) -> &RegistrySelection {
        &self.previous
    }
    pub fn selected(&self) -> &RegistrySelection {
        &self.selected
    }
    pub fn source_epoch(&self) -> &EpochManifest {
        &self.source_epoch
    }
    pub fn target_epoch(&self) -> &EpochManifest {
        &self.target_epoch
    }
    pub fn activated_current(&self) -> Option<&CurrentPublication> {
        self.activated_current.as_ref()
    }
    pub fn portable_snapshot_digest(&self) -> &str {
        &self.portable_snapshot_digest
    }
    pub fn acknowledgment(&self) -> AcknowledgmentState {
        self.acknowledgment.clone()
    }
    pub(in crate::project) fn from_record(record: &SelectionRecord) -> StoreResult<Self> {
        record.validate()?;
        Ok(Self {
            operation_id: record.intent.operation_id.clone(),
            request_digest: record.request_digest.clone(),
            previous: record.intent.expected.clone(),
            selected: record.selection()?,
            source_epoch: record.intent.source_epoch.clone(),
            target_epoch: record.intent.target_epoch.clone(),
            activated_current: record.intent.activated_current.clone(),
            portable_snapshot_digest: record.intent.portable_snapshot.clone(),
            acknowledgment: AcknowledgmentState::Unknown,
            durability: "file-synced-selector-replaced-and-read-back;power-loss-not-evaluated"
                .into(),
        })
    }
}
