//! Typed read-only recovery observations and bounded scope coverage.
use serde::Serialize;

use super::super::model::{
    CurrentPublication, CurrentRecordId, EpochId, PublicationOperation, encode,
};
use crate::{StoreErrorCode, StoreResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryScope {
    Current,
    Generations,
    Membership,
    Partitions,
    Operations,
    Validations,
    History,
    RetentionRoots,
    GcPolicy,
    GcReceipts,
    ForeignKeys,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScopeState {
    Validated,
    Invalid,
    Incomplete,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentState {
    Absent,
    Validated,
    Corrupt,
    Unverified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDisposition {
    Prepared,
    Published,
    Validated,
    ActivatedReceiptAvailable,
    Released,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AcknowledgmentState {
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScopeCoverage {
    pub(super) scope: RecoveryScope,
    pub(super) state: ScopeState,
}

impl ScopeCoverage {
    pub fn scope(&self) -> RecoveryScope {
        self.scope.clone()
    }
    pub fn state(&self) -> ScopeState {
        self.state.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryIncident {
    pub(super) scope: RecoveryScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) subject_id: Option<String>,
    pub(super) code: StoreErrorCode,
}

impl RecoveryIncident {
    pub fn scope(&self) -> RecoveryScope {
        self.scope.clone()
    }
    pub fn subject_id(&self) -> Option<&str> {
        self.subject_id.as_deref()
    }
    pub fn code(&self) -> StoreErrorCode {
        self.code
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryOperation {
    pub(super) operation: PublicationOperation,
    pub(super) disposition: RecoveryDisposition,
    pub(super) target_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) expected_current: Option<CurrentRecordId>,
    pub(super) matches_current: bool,
    pub(super) acknowledgment: AcknowledgmentState,
}

impl RecoveryOperation {
    pub fn operation(&self) -> &PublicationOperation {
        &self.operation
    }
    pub fn disposition(&self) -> RecoveryDisposition {
        self.disposition.clone()
    }
    pub fn target_present(&self) -> bool {
        self.target_present
    }
    pub fn expected_current(&self) -> Option<&CurrentRecordId> {
        self.expected_current.as_ref()
    }
    pub fn matches_current(&self) -> bool {
        self.matches_current
    }
    pub fn acknowledgment(&self) -> AcknowledgmentState {
        self.acknowledgment.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecoveryReport {
    pub(super) schema: String,
    pub(super) epoch_id: EpochId,
    pub(super) physical_profile: String,
    pub(super) current_state: CurrentState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) current: Option<CurrentPublication>,
    pub(super) operations: Vec<RecoveryOperation>,
    pub(super) coverage: Vec<ScopeCoverage>,
    pub(super) incidents: Vec<RecoveryIncident>,
}

impl RecoveryReport {
    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn epoch_id(&self) -> &EpochId {
        &self.epoch_id
    }
    pub fn physical_profile(&self) -> &str {
        &self.physical_profile
    }
    pub fn current_state(&self) -> CurrentState {
        self.current_state.clone()
    }
    pub fn current(&self) -> Option<&CurrentPublication> {
        self.current.as_ref()
    }
    pub fn operations(&self) -> &[RecoveryOperation] {
        &self.operations
    }
    pub fn coverage(&self) -> &[ScopeCoverage] {
        &self.coverage
    }
    pub fn incidents(&self) -> &[RecoveryIncident] {
        &self.incidents
    }
    pub fn canonical_bytes(&self) -> StoreResult<Vec<u8>> {
        encode(self, 16 * 1024 * 1024)
    }
}
