//! Data-only hold identity. Only a compiled inspection can establish the hold.
use super::super::{
    model::*,
    registry::{self, RegistrySelection},
};
use crate::{OperationId, StoreResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub(in crate::project) const MAX_EVIDENCE: usize = 16 * 1024 * 1024 + 4096;

/// Raw pointer observation, independent of history or domain validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum CurrentObservation {
    Absent,
    Pointer {
        digest: String,
        byte_length: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        record_id: Option<CurrentRecordId>,
    },
    Unreadable {
        reason: PointerReadFailure,
    },
}
impl CurrentObservation {
    fn validate(&self) -> StoreResult<()> {
        if let Self::Pointer {
            digest,
            byte_length,
            record_id,
        } = self
        {
            if !hashed(digest, "project-current-observation") || *byte_length > 4096 {
                return Err(invalid());
            }
            if let Some(id) = record_id {
                CurrentRecordId::parse(id.as_str())?;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PointerReadFailure {
    QueryUnavailable,
    InvalidShape,
    BudgetExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::project) struct QuarantineRecord {
    pub schema: String,
    pub epoch: EpochManifest,
    pub operation_id: OperationId,
    pub previous: RegistrySelection,
    pub current: CurrentObservation,
    pub evidence_digest: String,
    pub evidence_length: usize,
    pub revision: u64,
    pub request_digest: String,
}
impl QuarantineRecord {
    pub fn new(
        epoch: EpochManifest,
        operation_id: OperationId,
        previous: RegistrySelection,
        current: CurrentObservation,
        evidence: &[u8],
    ) -> StoreResult<Self> {
        let mut record = Self {
            schema: "wow-store/project-registry/3".into(),
            revision: previous.revision().checked_add(1).ok_or_else(invalid)?,
            epoch,
            operation_id,
            previous,
            current,
            evidence_digest: digest("project-quarantine-evidence", evidence),
            evidence_length: evidence.len(),
            request_digest: String::new(),
        };
        record.request_digest = record.request()?;
        record.validate()?;
        Ok(record)
    }
    fn request(&self) -> StoreResult<String> {
        Ok(digest(
            "project-quarantine-request",
            &encode(
                &(
                    "wow-store/project-quarantine-intent/1",
                    &self.operation_id,
                    &self.previous,
                    &self.epoch,
                    &self.current,
                    &self.evidence_digest,
                    self.evidence_length,
                ),
                registry::MAX_REGISTRY,
            )?,
        ))
    }
    pub fn validate(&self) -> StoreResult<()> {
        self.previous.validate()?;
        self.current.validate()?;
        OperationId::new(self.operation_id.as_str())?;
        if self.schema != "wow-store/project-registry/3"
            || self.previous.is_quarantined()
            || self.epoch.epoch_id != *self.previous.epoch()
            || self.previous.revision().checked_add(1) != Some(self.revision)
            || !hashed(&self.evidence_digest, "project-quarantine-evidence")
            || self.evidence_length == 0
            || self.evidence_length > MAX_EVIDENCE
            || self.request_digest != self.request()?
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, registry::MAX_REGISTRY)
    }
    pub fn archive(&self, root: &Path) -> StoreResult<PathBuf> {
        Ok(root
            .join("quarantines")
            .join(registry::instance_id(&self.operation_id)?))
    }
    pub fn selection(&self) -> StoreResult<RegistrySelection> {
        Ok(self.previous.quarantined(
            &self.bytes()?,
            self.revision,
            registry::instance_id(&self.operation_id)?,
        ))
    }
}

/// Attributable selected hold, with unknown acknowledgment and bounded durability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct QuarantineReceipt {
    pub(super) record: QuarantineRecord,
    pub(super) selected: RegistrySelection,
    pub(super) acknowledgment: super::super::AcknowledgmentState,
    pub(super) durability: String,
}
impl QuarantineReceipt {
    pub(super) fn new(record: QuarantineRecord) -> StoreResult<Self> {
        Ok(Self {
            selected: record.selection()?,
            record,
            acknowledgment: super::super::AcknowledgmentState::Unknown,
            durability: "file-synced-selector-replaced-and-read-back;power-loss-not-evaluated"
                .into(),
        })
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.record.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.record.request_digest
    }
    pub fn previous(&self) -> &RegistrySelection {
        &self.record.previous
    }
    pub fn selected(&self) -> &RegistrySelection {
        &self.selected
    }
    pub fn current(&self) -> &CurrentObservation {
        &self.record.current
    }
    pub fn evidence_digest(&self) -> &str {
        &self.record.evidence_digest
    }
}
