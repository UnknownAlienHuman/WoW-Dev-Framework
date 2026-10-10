//! Durable intent, checkpoint and reconciliation records for Reference Pack filesystem effects.
//!
//! The application still owns filesystem access and rename primitives. This module owns the
//! exact effect binding, journal transitions and recovery decision policy. It never opens an
//! arbitrary path, performs a rename or deletes a directory.

use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, ObjectId, OperationBegin,
    OperationId, OperationRecord, OperationState, PendingObject, RequestDigest, Store,
    StoreConfiguration, StoreError, StoreErrorCode, StoreLimits, WriteBatch,
};

pub const REFERENCE_PACK_MATERIALIZATION_SCHEMA: &str =
    "wow-service/reference-pack-materialization/e1-d/1";
const STATE_KIND: &str = "wow.service.reference_pack_materialization_state";
const STATE_SCHEMA_VERSION: u32 = 1;
const RECEIPT_KIND: &str = "wow.service.reference_pack_materialization_receipt";
const RECEIPT_SCHEMA_VERSION: u32 = 1;
const STATE_CATALOG: &str = "reference-pack-materialization-state";
const RECEIPT_CATALOG: &str = "reference-pack-materialization-results";
const MAX_PATH_BYTES: usize = 32 * 1024;

pub type ReferencePackMaterializationOperationId = OperationId;
pub type ReferencePackMaterializationStoreLimits = StoreLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackMaterializationErrorCode {
    ConfigurationInvalid,
    InvalidRequest,
    OperationConflict,
    OperationIncomplete,
    OutcomeUnknown,
    StateInvalid,
    ReceiptInvalid,
    StoreFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePackMaterializationError {
    code: ReferencePackMaterializationErrorCode,
    lower_code: Option<Box<str>>,
    message: Box<str>,
}

impl ReferencePackMaterializationError {
    fn new(code: ReferencePackMaterializationErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            lower_code: None,
            message: message.into(),
        }
    }

    fn lower(
        code: ReferencePackMaterializationErrorCode,
        lower_code: impl Into<Box<str>>,
        message: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            lower_code: Some(lower_code.into()),
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ReferencePackMaterializationErrorCode {
        self.code
    }

    #[must_use]
    pub fn lower_code(&self) -> Option<&str> {
        self.lower_code.as_deref()
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReferencePackMaterializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReferencePackMaterializationError {}

impl From<StoreError> for ReferencePackMaterializationError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::ConfigurationInvalid | StoreErrorCode::IdentifierInvalid => {
                ReferencePackMaterializationErrorCode::ConfigurationInvalid
            }
            StoreErrorCode::OperationConflict | StoreErrorCode::CatalogConflict => {
                ReferencePackMaterializationErrorCode::OperationConflict
            }
            StoreErrorCode::OperationStateInvalid => {
                ReferencePackMaterializationErrorCode::OperationIncomplete
            }
            StoreErrorCode::OutcomeUnknown => ReferencePackMaterializationErrorCode::OutcomeUnknown,
            _ => ReferencePackMaterializationErrorCode::StoreFailure,
        };
        Self::lower(code, store_error_code(source.code()), source.message())
    }
}

pub type ReferencePackMaterializationResult<T> = Result<T, ReferencePackMaterializationError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationConfiguration {
    schema: &'static str,
    store_configuration: StoreConfiguration,
    configuration_id: Box<str>,
}

impl ReferencePackMaterializationConfiguration {
    pub fn new(
        store_profile_id: impl Into<Box<str>>,
        store_limits: StoreLimits,
    ) -> ReferencePackMaterializationResult<Self> {
        let store_configuration = StoreConfiguration::new(store_profile_id, store_limits)?;
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            store_configuration_id: &'a str,
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
            store_configuration_id: store_configuration.configuration_id(),
        })
        .map_err(|_| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ConfigurationInvalid,
                "reference pack materialization configuration cannot be canonicalized",
            )
        })?;
        Ok(Self {
            schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
            store_configuration,
            configuration_id: format!(
                "reference-pack-materialization-configuration:sha256:{}",
                hex(&Sha256::digest(bytes))
            )
            .into(),
        })
    }

    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    #[must_use]
    pub fn store_configuration(&self) -> &StoreConfiguration {
        &self.store_configuration
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationRequest {
    operation_id: OperationId,
    output_path: Box<str>,
    staging_path: Box<str>,
    backup_path: Box<str>,
    quarantine_path: Box<str>,
    pack_id: Box<str>,
    plan_id: Box<str>,
    validation_report_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prior_destination_pack_id: Option<Box<str>>,
}

impl ReferencePackMaterializationRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        output_path: impl Into<Box<str>>,
        staging_path: impl Into<Box<str>>,
        backup_path: impl Into<Box<str>>,
        quarantine_path: impl Into<Box<str>>,
        pack_id: impl Into<Box<str>>,
        plan_id: impl Into<Box<str>>,
        validation_report_id: impl Into<Box<str>>,
        prior_destination_pack_id: Option<impl Into<Box<str>>>,
    ) -> ReferencePackMaterializationResult<Self> {
        let request = Self {
            operation_id,
            output_path: output_path.into(),
            staging_path: staging_path.into(),
            backup_path: backup_path.into(),
            quarantine_path: quarantine_path.into(),
            pack_id: pack_id.into(),
            plan_id: plan_id.into(),
            validation_report_id: validation_report_id.into(),
            prior_destination_pack_id: prior_destination_pack_id.map(Into::into),
        };
        request.validate()?;
        Ok(request)
    }

    fn validate(&self) -> ReferencePackMaterializationResult<()> {
        let paths = [
            self.output_path.as_ref(),
            self.staging_path.as_ref(),
            self.backup_path.as_ref(),
            self.quarantine_path.as_ref(),
        ];
        if paths.iter().any(|path| !valid_path_text(path))
            || paths
                .iter()
                .enumerate()
                .any(|(index, path)| paths[index + 1..].contains(path))
            || !valid_text(&self.pack_id)
            || !valid_text(&self.plan_id)
            || !valid_text(&self.validation_report_id)
            || self
                .prior_destination_pack_id
                .as_deref()
                .is_some_and(|value| !valid_text(value))
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::InvalidRequest,
                "reference pack materialization request is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    #[must_use]
    pub fn output_path(&self) -> &str {
        &self.output_path
    }

    #[must_use]
    pub fn staging_path(&self) -> &str {
        &self.staging_path
    }

    #[must_use]
    pub fn backup_path(&self) -> &str {
        &self.backup_path
    }

    #[must_use]
    pub fn quarantine_path(&self) -> &str {
        &self.quarantine_path
    }

    #[must_use]
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    #[must_use]
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    #[must_use]
    pub fn validation_report_id(&self) -> &str {
        &self.validation_report_id
    }

    #[must_use]
    pub fn prior_destination_pack_id(&self) -> Option<&str> {
        self.prior_destination_pack_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferencePackPathObservation {
    Absent,
    Pack {
        pack_id: Box<str>,
        validation_report_id: Box<str>,
    },
    Invalid {
        reason: Box<str>,
    },
}

impl ReferencePackPathObservation {
    #[must_use]
    pub const fn absent() -> Self {
        Self::Absent
    }

    pub fn pack(
        pack_id: impl Into<Box<str>>,
        validation_report_id: impl Into<Box<str>>,
    ) -> ReferencePackMaterializationResult<Self> {
        let pack_id = pack_id.into();
        let validation_report_id = validation_report_id.into();
        if !valid_text(&pack_id) || !valid_text(&validation_report_id) {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::InvalidRequest,
                "reference pack path observation identity is invalid",
            ));
        }
        Ok(Self::Pack {
            pack_id,
            validation_report_id,
        })
    }

    pub fn invalid(reason: impl Into<Box<str>>) -> ReferencePackMaterializationResult<Self> {
        let reason = reason.into();
        if !valid_text(&reason) {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::InvalidRequest,
                "reference pack invalid-path observation is empty",
            ));
        }
        Ok(Self::Invalid { reason })
    }

    fn validate(&self) -> bool {
        match self {
            Self::Absent => true,
            Self::Pack {
                pack_id,
                validation_report_id,
            } => valid_text(pack_id) && valid_text(validation_report_id),
            Self::Invalid { reason } => valid_text(reason),
        }
    }

    fn expected_pack(&self, request: &ReferencePackMaterializationRequest) -> bool {
        matches!(
            self,
            Self::Pack {
                pack_id,
                validation_report_id,
            } if pack_id.as_ref() == request.pack_id()
                && validation_report_id.as_ref() == request.validation_report_id()
        )
    }

    fn prior_pack(&self, request: &ReferencePackMaterializationRequest) -> bool {
        matches!(
            (self, request.prior_destination_pack_id()),
            (Self::Pack { pack_id, .. }, Some(prior)) if pack_id.as_ref() == prior
        )
    }

    fn absent_state(&self) -> bool {
        matches!(self, Self::Absent)
    }

    fn invalid_state(&self) -> bool {
        matches!(self, Self::Invalid { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationObservation {
    staging: ReferencePackPathObservation,
    destination: ReferencePackPathObservation,
    backup: ReferencePackPathObservation,
}

impl ReferencePackMaterializationObservation {
    pub fn new(
        staging: ReferencePackPathObservation,
        destination: ReferencePackPathObservation,
        backup: ReferencePackPathObservation,
    ) -> ReferencePackMaterializationResult<Self> {
        let observation = Self {
            staging,
            destination,
            backup,
        };
        if !observation.staging.validate()
            || !observation.destination.validate()
            || !observation.backup.validate()
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::InvalidRequest,
                "reference pack materialization observation is invalid",
            ));
        }
        Ok(observation)
    }

    #[must_use]
    pub fn staging(&self) -> &ReferencePackPathObservation {
        &self.staging
    }

    #[must_use]
    pub fn destination(&self) -> &ReferencePackPathObservation {
        &self.destination
    }

    #[must_use]
    pub fn backup(&self) -> &ReferencePackPathObservation {
        &self.backup
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackMaterializationStage {
    Registered,
    StagingValidated,
    BackupIntent,
    BackupMoved,
    InstallIntent,
    Installed,
    FinalValidated,
    CleanupIntent,
    RollbackIntent,
    RolledBack,
    OutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackMaterializationAction {
    CreateStaging,
    ResetStaging,
    AdoptStaging,
    MovePriorToBackup,
    AdoptMovedBackup,
    InstallStaging,
    AdoptInstalledDestination,
    ValidateDestination,
    RemoveBackup,
    Complete,
    PerformRollback,
    AdoptRollback,
    ReturnCompleted,
    TerminalNoEffect,
    TerminalFailed,
    OperatorReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationStateRecord {
    schema: Box<str>,
    state_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    request: ReferencePackMaterializationRequest,
    stage: ReferencePackMaterializationStage,
    observation: ReferencePackMaterializationObservation,
}

impl ReferencePackMaterializationStateRecord {
    fn build(
        configuration_id: &str,
        request_digest: RequestDigest,
        request: ReferencePackMaterializationRequest,
        stage: ReferencePackMaterializationStage,
        observation: ReferencePackMaterializationObservation,
    ) -> ReferencePackMaterializationResult<Self> {
        request.validate()?;
        let mut record = Self {
            schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA.into(),
            state_id: "pending".into(),
            configuration_id: configuration_id.into(),
            operation_id: request.operation_id().clone(),
            request_digest,
            request,
            stage,
            observation,
        };
        record.state_id = state_record_id(&record)?;
        record.validate(configuration_id)?;
        Ok(record)
    }

    fn validate(&self, configuration_id: &str) -> ReferencePackMaterializationResult<()> {
        self.request.validate()?;
        if self.schema.as_ref() != REFERENCE_PACK_MATERIALIZATION_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
            || &self.operation_id != self.request.operation_id()
            || self.request_digest
                != materialization_request_digest(configuration_id, &self.request)?
            || self.state_id != state_record_id(self)?
            || !self.observation.staging.validate()
            || !self.observation.destination.validate()
            || !self.observation.backup.validate()
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "stored reference pack materialization state is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn state_id(&self) -> &str {
        &self.state_id
    }

    #[must_use]
    pub const fn stage(&self) -> ReferencePackMaterializationStage {
        self.stage
    }

    #[must_use]
    pub fn observation(&self) -> &ReferencePackMaterializationObservation {
        &self.observation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackMaterializationDestinationState {
    Created,
    Replaced,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationReceipt {
    schema: Box<str>,
    receipt_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    pack_id: Box<str>,
    plan_id: Box<str>,
    final_validation_report_id: Box<str>,
    destination_state: ReferencePackMaterializationDestinationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    prior_destination_pack_id: Option<Box<str>>,
    member_count: u32,
    total_bytes: u64,
}

impl ReferencePackMaterializationReceipt {
    fn build(
        configuration_id: &str,
        request_digest: RequestDigest,
        request: &ReferencePackMaterializationRequest,
        member_count: u32,
        total_bytes: u64,
    ) -> ReferencePackMaterializationResult<Self> {
        if member_count == 0 || total_bytes == 0 {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "reference pack materialization receipt counts are invalid",
            ));
        }
        let mut receipt = Self {
            schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA.into(),
            receipt_id: "pending".into(),
            configuration_id: configuration_id.into(),
            operation_id: request.operation_id().clone(),
            request_digest,
            pack_id: request.pack_id().into(),
            plan_id: request.plan_id().into(),
            final_validation_report_id: request.validation_report_id().into(),
            destination_state: if request.prior_destination_pack_id().is_some() {
                ReferencePackMaterializationDestinationState::Replaced
            } else {
                ReferencePackMaterializationDestinationState::Created
            },
            prior_destination_pack_id: request.prior_destination_pack_id().map(Into::into),
            member_count,
            total_bytes,
        };
        receipt.receipt_id = materialization_receipt_id(&receipt)?;
        receipt.validate(configuration_id, request)?;
        Ok(receipt)
    }

    fn validate(
        &self,
        configuration_id: &str,
        request: &ReferencePackMaterializationRequest,
    ) -> ReferencePackMaterializationResult<()> {
        if self.schema.as_ref() != REFERENCE_PACK_MATERIALIZATION_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
            || &self.operation_id != request.operation_id()
            || self.request_digest != materialization_request_digest(configuration_id, request)?
            || self.pack_id.as_ref() != request.pack_id()
            || self.plan_id.as_ref() != request.plan_id()
            || self.final_validation_report_id.as_ref() != request.validation_report_id()
            || self.prior_destination_pack_id.as_deref() != request.prior_destination_pack_id()
            || self.destination_state
                != if request.prior_destination_pack_id().is_some() {
                    ReferencePackMaterializationDestinationState::Replaced
                } else {
                    ReferencePackMaterializationDestinationState::Created
                }
            || self.member_count == 0
            || self.total_bytes == 0
            || self.receipt_id != materialization_receipt_id(self)?
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "stored reference pack materialization receipt is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    #[must_use]
    pub fn pack_id(&self) -> &str {
        &self.pack_id
    }

    #[must_use]
    pub fn plan_id(&self) -> &str {
        &self.plan_id
    }

    #[must_use]
    pub fn final_validation_report_id(&self) -> &str {
        &self.final_validation_report_id
    }

    #[must_use]
    pub const fn destination_state(&self) -> ReferencePackMaterializationDestinationState {
        self.destination_state
    }

    #[must_use]
    pub fn prior_destination_pack_id(&self) -> Option<&str> {
        self.prior_destination_pack_id.as_deref()
    }

    #[must_use]
    pub const fn member_count(&self) -> u32 {
        self.member_count
    }

    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackMaterializationReconciliation {
    schema: &'static str,
    operation_state: OperationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    stage: Option<ReferencePackMaterializationStage>,
    action: ReferencePackMaterializationAction,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<ReferencePackMaterializationReceipt>,
    started: bool,
}

impl ReferencePackMaterializationReconciliation {
    #[must_use]
    pub const fn operation_state(&self) -> OperationState {
        self.operation_state
    }

    #[must_use]
    pub const fn stage(&self) -> Option<ReferencePackMaterializationStage> {
        self.stage
    }

    #[must_use]
    pub const fn action(&self) -> ReferencePackMaterializationAction {
        self.action
    }

    #[must_use]
    pub fn receipt(&self) -> Option<&ReferencePackMaterializationReceipt> {
        self.receipt.as_ref()
    }

    #[must_use]
    pub const fn started(&self) -> bool {
        self.started
    }
}

pub struct ReferencePackMaterializationService {
    configuration: ReferencePackMaterializationConfiguration,
    store: Store,
}

impl ReferencePackMaterializationService {
    pub fn open(
        path: impl AsRef<Path>,
        configuration: ReferencePackMaterializationConfiguration,
    ) -> ReferencePackMaterializationResult<Self> {
        let store = Store::open(path, configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &ReferencePackMaterializationConfiguration {
        &self.configuration
    }

    pub fn stored_request(
        &self,
        operation_id: &OperationId,
    ) -> ReferencePackMaterializationResult<Option<ReferencePackMaterializationRequest>> {
        let Some(operation) = self.store.operation(operation_id)? else {
            return Ok(None);
        };
        let Some(object_id) = catalog_object_id(&self.store, STATE_CATALOG, operation_id)? else {
            if operation.state() == OperationState::Prepared {
                return Err(ReferencePackMaterializationError::new(
                    ReferencePackMaterializationErrorCode::OutcomeUnknown,
                    "materialization operation exists without a durable state record",
                ));
            }
            return Ok(None);
        };
        let object = self.store.object(&object_id)?.ok_or_else(|| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object is missing",
            )
        })?;
        if object.kind() != STATE_KIND || object.schema_version() != STATE_SCHEMA_VERSION {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object has an incompatible type",
            ));
        }
        let record: ReferencePackMaterializationStateRecord = object.decode().map_err(|_| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object cannot be decoded",
            )
        })?;
        record.validate(self.configuration.configuration_id())?;
        if &record.operation_id != operation_id
            || &record.request_digest != operation.request_digest()
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state is not bound to the durable operation",
            ));
        }
        Ok(Some(record.request))
    }

    pub fn reconcile(
        &mut self,
        request: &ReferencePackMaterializationRequest,
        observation: ReferencePackMaterializationObservation,
    ) -> ReferencePackMaterializationResult<ReferencePackMaterializationReconciliation> {
        request.validate()?;
        let request_digest =
            materialization_request_digest(self.configuration.configuration_id(), request)?;
        let operation_id = request.operation_id().clone();
        let (mut operation, started) = match self
            .store
            .begin_operation(operation_id.clone(), request_digest.clone())?
        {
            OperationBegin::Started(record) => {
                let state = ReferencePackMaterializationStateRecord::build(
                    self.configuration.configuration_id(),
                    request_digest.clone(),
                    request.clone(),
                    ReferencePackMaterializationStage::Registered,
                    observation.clone(),
                )?;
                self.persist_state(None, &state)?;
                (record, true)
            }
            OperationBegin::Replay(record) => (record, false),
        };

        match operation.state() {
            OperationState::Completed => {
                let receipt = self.completed_receipt(&operation, request)?;
                let action = if completed_observation(request, &observation) {
                    ReferencePackMaterializationAction::ReturnCompleted
                } else {
                    ReferencePackMaterializationAction::OperatorReview
                };
                return Ok(ReferencePackMaterializationReconciliation {
                    schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
                    operation_state: operation.state(),
                    stage: None,
                    action,
                    receipt: Some(receipt),
                    started,
                });
            }
            OperationState::NoEffect => {
                return Ok(ReferencePackMaterializationReconciliation {
                    schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
                    operation_state: operation.state(),
                    stage: None,
                    action: ReferencePackMaterializationAction::TerminalNoEffect,
                    receipt: None,
                    started,
                });
            }
            OperationState::Failed => {
                return Ok(ReferencePackMaterializationReconciliation {
                    schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
                    operation_state: operation.state(),
                    stage: None,
                    action: ReferencePackMaterializationAction::TerminalFailed,
                    receipt: None,
                    started,
                });
            }
            OperationState::OutcomeUnknown => {
                return Ok(ReferencePackMaterializationReconciliation {
                    schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
                    operation_state: operation.state(),
                    stage: self
                        .read_state(request, &request_digest)?
                        .map(|value| value.0.stage),
                    action: ReferencePackMaterializationAction::OperatorReview,
                    receipt: None,
                    started,
                });
            }
            OperationState::Prepared => {}
        }

        let Some((state, _)) = self.read_state(request, &request_digest)? else {
            let _ = self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest);
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::OutcomeUnknown,
                "prepared materialization operation is missing its durable state",
            ));
        };
        let action = decide_action(state.stage(), request, &observation);
        if action == ReferencePackMaterializationAction::OperatorReview {
            let unknown = ReferencePackMaterializationStateRecord::build(
                self.configuration.configuration_id(),
                request_digest.clone(),
                request.clone(),
                ReferencePackMaterializationStage::OutcomeUnknown,
                observation,
            )?;
            self.persist_state(
                catalog_object_id(&self.store, STATE_CATALOG, &operation_id)?,
                &unknown,
            )?;
            operation = self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest)?;
        }
        Ok(ReferencePackMaterializationReconciliation {
            schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
            operation_state: operation.state(),
            stage: Some(state.stage()),
            action,
            receipt: None,
            started,
        })
    }

    pub fn checkpoint(
        &mut self,
        request: &ReferencePackMaterializationRequest,
        stage: ReferencePackMaterializationStage,
        observation: ReferencePackMaterializationObservation,
    ) -> ReferencePackMaterializationResult<ReferencePackMaterializationStateRecord> {
        request.validate()?;
        if stage == ReferencePackMaterializationStage::OutcomeUnknown {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::InvalidRequest,
                "outcome-unknown stage must be recorded through reconciliation",
            ));
        }
        let request_digest =
            materialization_request_digest(self.configuration.configuration_id(), request)?;
        let operation = self
            .store
            .operation(request.operation_id())?
            .ok_or_else(|| {
                ReferencePackMaterializationError::new(
                    ReferencePackMaterializationErrorCode::OperationIncomplete,
                    "materialization operation is not registered",
                )
            })?;
        ensure_prepared(&operation, &request_digest)?;
        let (current, current_object_id) =
            self.read_state(request, &request_digest)?.ok_or_else(|| {
                ReferencePackMaterializationError::new(
                    ReferencePackMaterializationErrorCode::StateInvalid,
                    "materialization operation state is missing",
                )
            })?;
        if !allowed_transition(current.stage(), stage) {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization checkpoint transition is invalid",
            ));
        }
        let next = ReferencePackMaterializationStateRecord::build(
            self.configuration.configuration_id(),
            request_digest,
            request.clone(),
            stage,
            observation,
        )?;
        self.persist_state(Some(current_object_id), &next)?;
        Ok(next)
    }

    pub fn complete(
        &mut self,
        request: &ReferencePackMaterializationRequest,
        observation: &ReferencePackMaterializationObservation,
        member_count: u32,
        total_bytes: u64,
    ) -> ReferencePackMaterializationResult<ReferencePackMaterializationReceipt> {
        request.validate()?;
        let request_digest =
            materialization_request_digest(self.configuration.configuration_id(), request)?;
        let operation = self
            .store
            .operation(request.operation_id())?
            .ok_or_else(|| {
                ReferencePackMaterializationError::new(
                    ReferencePackMaterializationErrorCode::OperationIncomplete,
                    "materialization operation is not registered",
                )
            })?;
        if operation.state() == OperationState::Completed {
            return self.completed_receipt(&operation, request);
        }
        ensure_prepared(&operation, &request_digest)?;
        let (state, _) = self.read_state(request, &request_digest)?.ok_or_else(|| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization operation state is missing",
            )
        })?;
        if !matches!(
            state.stage(),
            ReferencePackMaterializationStage::FinalValidated
                | ReferencePackMaterializationStage::CleanupIntent
        ) || decide_action(state.stage(), request, observation)
            != ReferencePackMaterializationAction::Complete
        {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization cannot complete before exact final read-back and backup cleanup",
            ));
        }
        let receipt = ReferencePackMaterializationReceipt::build(
            self.configuration.configuration_id(),
            request_digest.clone(),
            request,
            member_count,
            total_bytes,
        )?;
        self.persist_receipt(&receipt)?;
        let object_id = catalog_object_id(&self.store, RECEIPT_CATALOG, request.operation_id())?
            .ok_or_else(|| {
                ReferencePackMaterializationError::new(
                    ReferencePackMaterializationErrorCode::ReceiptInvalid,
                    "materialization receipt is not visible after commit",
                )
            })?;
        if let Err(source) =
            self.store
                .complete_operation(request.operation_id(), &request_digest, &object_id)
        {
            if self
                .store
                .operation(request.operation_id())?
                .is_some_and(|record| {
                    record.state() == OperationState::Completed
                        && record.result_object_id() == Some(&object_id)
                })
            {
                return Ok(receipt);
            }
            let _ = self
                .store
                .mark_outcome_unknown(request.operation_id(), &request_digest);
            return Err(ReferencePackMaterializationError::lower(
                ReferencePackMaterializationErrorCode::OutcomeUnknown,
                store_error_code(source.code()),
                "materialization receipt was committed but operation completion is uncertain",
            ));
        }
        Ok(receipt)
    }

    pub fn record_no_effect(
        &mut self,
        request: &ReferencePackMaterializationRequest,
    ) -> ReferencePackMaterializationResult<()> {
        let request_digest =
            materialization_request_digest(self.configuration.configuration_id(), request)?;
        self.store
            .record_no_effect(request.operation_id(), &request_digest)?;
        Ok(())
    }

    fn completed_receipt(
        &self,
        operation: &OperationRecord,
        request: &ReferencePackMaterializationRequest,
    ) -> ReferencePackMaterializationResult<ReferencePackMaterializationReceipt> {
        let object_id = operation.result_object_id().ok_or_else(|| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "completed materialization operation is missing its receipt object",
            )
        })?;
        self.read_receipt(object_id, request)
    }

    fn read_state(
        &self,
        request: &ReferencePackMaterializationRequest,
        request_digest: &RequestDigest,
    ) -> ReferencePackMaterializationResult<
        Option<(ReferencePackMaterializationStateRecord, ObjectId)>,
    > {
        let Some(object_id) =
            catalog_object_id(&self.store, STATE_CATALOG, request.operation_id())?
        else {
            return Ok(None);
        };
        let object = self.store.object(&object_id)?.ok_or_else(|| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object is missing",
            )
        })?;
        if object.kind() != STATE_KIND || object.schema_version() != STATE_SCHEMA_VERSION {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object has an incompatible type",
            ));
        }
        let record: ReferencePackMaterializationStateRecord = object.decode().map_err(|_| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state object cannot be decoded",
            )
        })?;
        record.validate(self.configuration.configuration_id())?;
        if &record.request_digest != request_digest || &record.request != request {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::StateInvalid,
                "materialization state belongs to another request",
            ));
        }
        Ok(Some((record, object_id)))
    }

    fn read_receipt(
        &self,
        object_id: &ObjectId,
        request: &ReferencePackMaterializationRequest,
    ) -> ReferencePackMaterializationResult<ReferencePackMaterializationReceipt> {
        let object = self.store.object(object_id)?.ok_or_else(|| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "materialization receipt object is missing",
            )
        })?;
        if object.kind() != RECEIPT_KIND || object.schema_version() != RECEIPT_SCHEMA_VERSION {
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "materialization receipt object has an incompatible type",
            ));
        }
        let receipt: ReferencePackMaterializationReceipt = object.decode().map_err(|_| {
            ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "materialization receipt object cannot be decoded",
            )
        })?;
        receipt.validate(self.configuration.configuration_id(), request)?;
        Ok(receipt)
    }

    fn persist_state(
        &mut self,
        expected: Option<ObjectId>,
        state: &ReferencePackMaterializationStateRecord,
    ) -> ReferencePackMaterializationResult<()> {
        let pending = PendingObject::from_json(
            STATE_KIND,
            STATE_SCHEMA_VERSION,
            state,
            self.configuration.store_configuration().limits(),
        )?;
        self.persist_catalog_object(
            STATE_CATALOG,
            state.request.operation_id(),
            expected,
            pending,
            ReferencePackMaterializationErrorCode::StateInvalid,
        )
    }

    fn persist_receipt(
        &mut self,
        receipt: &ReferencePackMaterializationReceipt,
    ) -> ReferencePackMaterializationResult<()> {
        let pending = PendingObject::from_json(
            RECEIPT_KIND,
            RECEIPT_SCHEMA_VERSION,
            receipt,
            self.configuration.store_configuration().limits(),
        )?;
        let current = catalog_object_id(&self.store, RECEIPT_CATALOG, receipt.operation_id())?;
        if let Some(existing) = current {
            if &existing == pending.object_id() {
                return Ok(());
            }
            return Err(ReferencePackMaterializationError::new(
                ReferencePackMaterializationErrorCode::ReceiptInvalid,
                "materialization receipt catalog is already bound to different content",
            ));
        }
        self.persist_catalog_object(
            RECEIPT_CATALOG,
            receipt.operation_id(),
            None,
            pending,
            ReferencePackMaterializationErrorCode::ReceiptInvalid,
        )
    }

    fn persist_catalog_object(
        &mut self,
        catalog: &str,
        operation_id: &OperationId,
        expected: Option<ObjectId>,
        pending: PendingObject,
        invalid_code: ReferencePackMaterializationErrorCode,
    ) -> ReferencePackMaterializationResult<()> {
        let target = pending.object_id().clone();
        if expected.as_ref() == Some(&target) {
            return Ok(());
        }
        let expectation = expected
            .clone()
            .map_or(CatalogExpectation::Absent, CatalogExpectation::Exact);
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        batch.add_catalog_mutation(CatalogMutation::set(
            CatalogName::new(catalog)?,
            CatalogPath::new(operation_id.as_str())?,
            expectation,
            target.clone(),
        ))?;
        match self.store.commit(batch) {
            Ok(_) => {}
            Err(source)
                if matches!(
                    source.code(),
                    StoreErrorCode::CatalogConflict | StoreErrorCode::OutcomeUnknown
                ) =>
            {
                if catalog_object_id(&self.store, catalog, operation_id)?.as_ref() != Some(&target)
                {
                    return Err(ReferencePackMaterializationError::lower(
                        if source.code() == StoreErrorCode::OutcomeUnknown {
                            ReferencePackMaterializationErrorCode::OutcomeUnknown
                        } else {
                            ReferencePackMaterializationErrorCode::OperationConflict
                        },
                        store_error_code(source.code()),
                        "materialization checkpoint commit could not be reconciled",
                    ));
                }
            }
            Err(source) => return Err(source.into()),
        }
        let object = self.store.object(&target)?.ok_or_else(|| {
            ReferencePackMaterializationError::new(
                invalid_code,
                "materialization checkpoint object is missing after commit",
            )
        })?;
        if object.object_id() != &target {
            return Err(ReferencePackMaterializationError::new(
                invalid_code,
                "materialization checkpoint failed exact read-back",
            ));
        }
        Ok(())
    }
}

fn decide_action(
    stage: ReferencePackMaterializationStage,
    request: &ReferencePackMaterializationRequest,
    observation: &ReferencePackMaterializationObservation,
) -> ReferencePackMaterializationAction {
    use ReferencePackMaterializationAction as Action;
    use ReferencePackMaterializationStage as Stage;

    let staging_expected = observation.staging.expected_pack(request);
    let staging_absent = observation.staging.absent_state();
    let staging_invalid = observation.staging.invalid_state();
    let destination_expected = observation.destination.expected_pack(request);
    let destination_initial = match request.prior_destination_pack_id() {
        Some(_) => observation.destination.prior_pack(request),
        None => observation.destination.absent_state(),
    };
    let destination_absent = observation.destination.absent_state();
    let backup_expected = match request.prior_destination_pack_id() {
        Some(_) => observation.backup.prior_pack(request),
        None => observation.backup.absent_state(),
    };
    let backup_absent = observation.backup.absent_state();

    if observation.backup.invalid_state()
        || (observation.destination.invalid_state() && stage != Stage::RollbackIntent)
    {
        return Action::OperatorReview;
    }

    match stage {
        Stage::Registered | Stage::RolledBack => {
            if destination_initial && backup_absent && staging_absent {
                Action::CreateStaging
            } else if destination_initial && backup_absent && staging_invalid {
                Action::ResetStaging
            } else if destination_initial && backup_absent && staging_expected {
                Action::AdoptStaging
            } else {
                Action::OperatorReview
            }
        }
        Stage::StagingValidated => {
            if staging_expected && destination_initial && backup_absent {
                if request.prior_destination_pack_id().is_some() {
                    Action::MovePriorToBackup
                } else {
                    Action::InstallStaging
                }
            } else {
                Action::OperatorReview
            }
        }
        Stage::BackupIntent => {
            if staging_expected && destination_initial && backup_absent {
                Action::MovePriorToBackup
            } else if staging_expected && destination_absent && backup_expected {
                Action::AdoptMovedBackup
            } else if staging_absent && destination_expected && backup_expected {
                Action::AdoptInstalledDestination
            } else {
                Action::OperatorReview
            }
        }
        Stage::BackupMoved => {
            if staging_expected && destination_absent && backup_expected {
                Action::InstallStaging
            } else if staging_absent && destination_expected && backup_expected {
                Action::AdoptInstalledDestination
            } else if staging_absent && destination_absent && backup_expected {
                Action::PerformRollback
            } else {
                Action::OperatorReview
            }
        }
        Stage::InstallIntent => {
            if staging_expected && destination_absent && backup_expected {
                Action::InstallStaging
            } else if staging_absent && destination_expected && backup_expected {
                Action::AdoptInstalledDestination
            } else if staging_absent && destination_absent && backup_expected {
                Action::PerformRollback
            } else {
                Action::OperatorReview
            }
        }
        Stage::Installed => {
            if staging_absent && destination_expected && backup_expected {
                Action::ValidateDestination
            } else if staging_absent && destination_absent && backup_expected {
                Action::PerformRollback
            } else {
                Action::OperatorReview
            }
        }
        Stage::FinalValidated => {
            if staging_absent && destination_expected && backup_expected {
                if request.prior_destination_pack_id().is_some() {
                    Action::RemoveBackup
                } else {
                    Action::Complete
                }
            } else {
                Action::OperatorReview
            }
        }
        Stage::CleanupIntent => {
            if staging_absent && destination_expected && backup_absent {
                Action::Complete
            } else if staging_absent && destination_expected && backup_expected {
                Action::RemoveBackup
            } else {
                Action::OperatorReview
            }
        }
        Stage::RollbackIntent => {
            if destination_initial && backup_absent {
                Action::AdoptRollback
            } else if (destination_absent && backup_expected)
                || destination_expected
                || observation.destination.invalid_state()
            {
                Action::PerformRollback
            } else {
                Action::OperatorReview
            }
        }
        Stage::OutcomeUnknown => Action::OperatorReview,
    }
}

fn completed_observation(
    request: &ReferencePackMaterializationRequest,
    observation: &ReferencePackMaterializationObservation,
) -> bool {
    observation.staging.absent_state()
        && observation.destination.expected_pack(request)
        && observation.backup.absent_state()
}

fn allowed_transition(
    current: ReferencePackMaterializationStage,
    next: ReferencePackMaterializationStage,
) -> bool {
    use ReferencePackMaterializationStage as Stage;
    current == next
        || matches!(
            (current, next),
            (
                Stage::Registered | Stage::RolledBack,
                Stage::StagingValidated
            ) | (Stage::Registered, Stage::RolledBack)
                | (
                    Stage::StagingValidated,
                    Stage::BackupIntent | Stage::InstallIntent
                )
                | (Stage::StagingValidated, Stage::RollbackIntent)
                | (
                    Stage::BackupIntent,
                    Stage::BackupMoved | Stage::Installed | Stage::RollbackIntent
                )
                | (
                    Stage::BackupMoved,
                    Stage::InstallIntent | Stage::Installed | Stage::RollbackIntent
                )
                | (
                    Stage::InstallIntent,
                    Stage::Installed | Stage::RollbackIntent
                )
                | (
                    Stage::Installed,
                    Stage::FinalValidated | Stage::RollbackIntent
                )
                | (
                    Stage::FinalValidated,
                    Stage::CleanupIntent | Stage::RollbackIntent
                )
                | (Stage::CleanupIntent, Stage::RollbackIntent)
                | (Stage::RollbackIntent, Stage::RolledBack)
        )
}

fn ensure_prepared(
    operation: &OperationRecord,
    request_digest: &RequestDigest,
) -> ReferencePackMaterializationResult<()> {
    if operation.request_digest() != request_digest {
        return Err(ReferencePackMaterializationError::new(
            ReferencePackMaterializationErrorCode::OperationConflict,
            "materialization operation request digest differs",
        ));
    }
    if operation.state() != OperationState::Prepared {
        return Err(ReferencePackMaterializationError::new(
            ReferencePackMaterializationErrorCode::OperationIncomplete,
            "materialization operation is not prepared",
        ));
    }
    Ok(())
}

fn materialization_request_digest(
    configuration_id: &str,
    request: &ReferencePackMaterializationRequest,
) -> ReferencePackMaterializationResult<RequestDigest> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        request: &'a ReferencePackMaterializationRequest,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: REFERENCE_PACK_MATERIALIZATION_SCHEMA,
        configuration_id,
        request,
    })
    .map_err(|_| {
        ReferencePackMaterializationError::new(
            ReferencePackMaterializationErrorCode::InvalidRequest,
            "materialization request cannot be canonicalized",
        )
    })?;
    Ok(RequestDigest::new(format!(
        "sha256:{}",
        hex(&Sha256::digest(bytes))
    ))?)
}

fn state_record_id(
    record: &ReferencePackMaterializationStateRecord,
) -> ReferencePackMaterializationResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'a str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        request: &'a ReferencePackMaterializationRequest,
        stage: ReferencePackMaterializationStage,
        observation: &'a ReferencePackMaterializationObservation,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: &record.schema,
        configuration_id: &record.configuration_id,
        operation_id: &record.operation_id,
        request_digest: &record.request_digest,
        request: &record.request,
        stage: record.stage,
        observation: &record.observation,
    })
    .map_err(|_| {
        ReferencePackMaterializationError::new(
            ReferencePackMaterializationErrorCode::StateInvalid,
            "materialization state cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "reference-pack-materialization-state:sha256:{}",
        hex(&Sha256::digest(bytes))
    )
    .into())
}

fn materialization_receipt_id(
    receipt: &ReferencePackMaterializationReceipt,
) -> ReferencePackMaterializationResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'a str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        pack_id: &'a str,
        plan_id: &'a str,
        final_validation_report_id: &'a str,
        destination_state: ReferencePackMaterializationDestinationState,
        prior_destination_pack_id: Option<&'a str>,
        member_count: u32,
        total_bytes: u64,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: &receipt.schema,
        configuration_id: &receipt.configuration_id,
        operation_id: &receipt.operation_id,
        request_digest: &receipt.request_digest,
        pack_id: &receipt.pack_id,
        plan_id: &receipt.plan_id,
        final_validation_report_id: &receipt.final_validation_report_id,
        destination_state: receipt.destination_state,
        prior_destination_pack_id: receipt.prior_destination_pack_id.as_deref(),
        member_count: receipt.member_count,
        total_bytes: receipt.total_bytes,
    })
    .map_err(|_| {
        ReferencePackMaterializationError::new(
            ReferencePackMaterializationErrorCode::ReceiptInvalid,
            "materialization receipt cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "reference-pack-materialization-receipt:sha256:{}",
        hex(&Sha256::digest(bytes))
    )
    .into())
}

fn catalog_object_id(
    store: &Store,
    catalog: &str,
    operation_id: &OperationId,
) -> ReferencePackMaterializationResult<Option<ObjectId>> {
    Ok(store
        .catalog_entry(
            &CatalogName::new(catalog)?,
            &CatalogPath::new(operation_id.as_str())?,
        )?
        .map(|entry| entry.object_id().clone()))
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && value
            .chars()
            .all(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
}

fn valid_path_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_PATH_BYTES
        && !value.contains('\0')
        && value.chars().all(|character| !character.is_control())
}

fn store_error_code(code: StoreErrorCode) -> &'static str {
    match code {
        StoreErrorCode::ConfigurationInvalid => "configuration_invalid",
        StoreErrorCode::IdentifierInvalid => "identifier_invalid",
        StoreErrorCode::ObjectTooLarge => "object_too_large",
        StoreErrorCode::BatchTooLarge => "batch_too_large",
        StoreErrorCode::JsonInvalid => "json_invalid",
        StoreErrorCode::ObjectConflict => "object_conflict",
        StoreErrorCode::ObjectMissing => "object_missing",
        StoreErrorCode::CatalogConflict => "catalog_conflict",
        StoreErrorCode::OperationConflict => "operation_conflict",
        StoreErrorCode::OperationStateInvalid => "operation_state_invalid",
        StoreErrorCode::LeaseConflict => "lease_conflict",
        StoreErrorCode::LeaseInvalid => "lease_invalid",
        StoreErrorCode::BudgetExceeded => "budget_exceeded",
        StoreErrorCode::IntegrityViolation => "integrity_violation",
        StoreErrorCode::DatabaseUnavailable => "database_unavailable",
        StoreErrorCode::Cancelled => "cancelled",
        StoreErrorCode::WriterBusy => "writer_busy",
        StoreErrorCode::CurrentConflict => "current_conflict",
        StoreErrorCode::GenerationMissing => "generation_missing",
        StoreErrorCode::OutcomeUnknown => "outcome_unknown",
        StoreErrorCode::Quarantined => "quarantined",
    }
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(TABLE[(byte >> 4) as usize] as char);
        output.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    output
}
