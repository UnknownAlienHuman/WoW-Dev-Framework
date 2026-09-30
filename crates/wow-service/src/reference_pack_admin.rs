//! Durable operation registration and reconciliation for Reference Pack use cases.
//!
//! The pure `reference_pack` owner still performs build, validation and comparison.
//! This module binds those deterministic operations to a durable operation journal,
//! stores compact exact result receipts, and records typed recovery guidance. It does
//! not materialize files, mutate a final destination, or retry unknown external effects.

use std::{fmt, path::Path, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, ObjectId, OperationBegin,
    OperationId, OperationRecord, OperationState, PendingObject, RequestDigest, Store,
    StoreConfiguration, StoreError, StoreErrorCode, StoreLimits, WriteBatch,
};

use crate::local::LocalProjectInput;
use crate::reference_pack::{
    RebuildComparisonStatus, ReferencePackBuildOutcome, ReferencePackBuildRequest,
    ReferencePackBuildStatus, ReferencePackError, ReferencePackErrorCode, ReferencePackImage,
    ReferencePackRebuildComparisonReport, ReferencePackRebuildComparisonRequest,
    ReferencePackService, ReferencePackValidationReport, ReferencePackValidationRequest,
};

pub const REFERENCE_PACK_ADMIN_SCHEMA: &str = "wow-service/reference-pack-admin/e1-d/1";
const RESULT_KIND: &str = "wow.service.reference_pack_operation_result";
const RESULT_SCHEMA_VERSION: u32 = 1;
const RECOVERY_KIND: &str = "wow.service.reference_pack_recovery_record";
const RECOVERY_SCHEMA_VERSION: u32 = 1;
const RESULT_CATALOG: &str = "reference-pack-operation-results";
const RECOVERY_CATALOG: &str = "reference-pack-operation-recovery";

pub type ReferencePackAdminOperationId = OperationId;
pub type ReferencePackAdminStoreLimits = StoreLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackAdminErrorCode {
    ConfigurationInvalid,
    InvalidRequest,
    OperationConflict,
    OperationIncomplete,
    OutcomeUnknown,
    ResultInvalid,
    RecoveryInvalid,
    StoreFailure,
    ReferencePackFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferencePackAdminError {
    code: ReferencePackAdminErrorCode,
    lower_code: Option<Box<str>>,
    message: Box<str>,
}

impl ReferencePackAdminError {
    fn new(code: ReferencePackAdminErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            lower_code: None,
            message: message.into(),
        }
    }

    fn lower(
        code: ReferencePackAdminErrorCode,
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
    pub const fn code(&self) -> ReferencePackAdminErrorCode {
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

impl fmt::Display for ReferencePackAdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReferencePackAdminError {}

impl From<StoreError> for ReferencePackAdminError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::OperationConflict => ReferencePackAdminErrorCode::OperationConflict,
            StoreErrorCode::OperationStateInvalid => {
                ReferencePackAdminErrorCode::OperationIncomplete
            }
            StoreErrorCode::OutcomeUnknown => ReferencePackAdminErrorCode::OutcomeUnknown,
            StoreErrorCode::ConfigurationInvalid | StoreErrorCode::IdentifierInvalid => {
                ReferencePackAdminErrorCode::ConfigurationInvalid
            }
            _ => ReferencePackAdminErrorCode::StoreFailure,
        };
        Self::lower(code, store_error_code(source.code()), source.message())
    }
}

impl From<ReferencePackError> for ReferencePackAdminError {
    fn from(source: ReferencePackError) -> Self {
        let code = match source.code() {
            ReferencePackErrorCode::InvalidRequest
            | ReferencePackErrorCode::UnsupportedLayout
            | ReferencePackErrorCode::IdentityMismatch
            | ReferencePackErrorCode::MemberInvalid
            | ReferencePackErrorCode::ManifestInvalid => {
                ReferencePackAdminErrorCode::InvalidRequest
            }
            ReferencePackErrorCode::Cancelled => ReferencePackAdminErrorCode::OperationIncomplete,
            ReferencePackErrorCode::SourceInputUnavailable
            | ReferencePackErrorCode::BudgetExceeded
            | ReferencePackErrorCode::ValidationFailed
            | ReferencePackErrorCode::SerializationFailed => {
                ReferencePackAdminErrorCode::ReferencePackFailure
            }
        };
        Self::lower(code, pack_error_code(source.code()), source.message())
    }
}

pub type ReferencePackAdminResult<T> = Result<T, ReferencePackAdminError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackAdminConfiguration {
    schema: &'static str,
    store_configuration: StoreConfiguration,
    configuration_id: Box<str>,
}

impl ReferencePackAdminConfiguration {
    pub fn new(
        store_profile_id: impl Into<Box<str>>,
        store_limits: StoreLimits,
    ) -> ReferencePackAdminResult<Self> {
        let store_configuration = StoreConfiguration::new(store_profile_id, store_limits)?;
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            store_configuration_id: &'a str,
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: REFERENCE_PACK_ADMIN_SCHEMA,
            store_configuration_id: store_configuration.configuration_id(),
        })
        .map_err(|_| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ConfigurationInvalid,
                "reference pack administration configuration cannot be canonicalized",
            )
        })?;
        Ok(Self {
            schema: REFERENCE_PACK_ADMIN_SCHEMA,
            store_configuration,
            configuration_id: format!(
                "reference-pack-admin-configuration:sha256:{}",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackOperationKind {
    Build,
    Validate,
    RebuildCompare,
}

impl ReferencePackOperationKind {
    const fn stage(self) -> &'static str {
        match self {
            Self::Build => "reference_pack_build",
            Self::Validate => "reference_pack_validate",
            Self::RebuildCompare => "reference_pack_rebuild_compare",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReferencePackOperationOutput {
    Build {
        request_id: Box<str>,
        execution_profile_id: Box<str>,
        status: ReferencePackBuildStatus,
        pack_id: Box<str>,
        plan_id: Box<str>,
        validation_report_id: Box<str>,
    },
    Validate {
        pack_id: Box<str>,
        validation_report_id: Box<str>,
        candidate_eligible: bool,
        validated_local_eligible: bool,
    },
    RebuildCompare {
        report_id: Box<str>,
        status: RebuildComparisonStatus,
        left_execution_profile_id: Box<str>,
        right_execution_profile_id: Box<str>,
    },
}

impl ReferencePackOperationOutput {
    const fn operation_kind(&self) -> ReferencePackOperationKind {
        match self {
            Self::Build { .. } => ReferencePackOperationKind::Build,
            Self::Validate { .. } => ReferencePackOperationKind::Validate,
            Self::RebuildCompare { .. } => ReferencePackOperationKind::RebuildCompare,
        }
    }

    fn validate(&self) -> bool {
        match self {
            Self::Build {
                request_id,
                execution_profile_id,
                pack_id,
                plan_id,
                validation_report_id,
                ..
            } => [
                request_id.as_ref(),
                execution_profile_id.as_ref(),
                pack_id.as_ref(),
                plan_id.as_ref(),
                validation_report_id.as_ref(),
            ]
            .into_iter()
            .all(valid_text),
            Self::Validate {
                pack_id,
                validation_report_id,
                ..
            } => [pack_id.as_ref(), validation_report_id.as_ref()]
                .into_iter()
                .all(valid_text),
            Self::RebuildCompare {
                report_id,
                left_execution_profile_id,
                right_execution_profile_id,
                ..
            } => [
                report_id.as_ref(),
                left_execution_profile_id.as_ref(),
                right_execution_profile_id.as_ref(),
            ]
            .into_iter()
            .all(valid_text),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackOperationReceipt {
    schema: Box<str>,
    receipt_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    operation_kind: ReferencePackOperationKind,
    output: ReferencePackOperationOutput,
}

impl ReferencePackOperationReceipt {
    fn build(
        configuration_id: &str,
        operation_id: OperationId,
        request_digest: RequestDigest,
        output: ReferencePackOperationOutput,
    ) -> ReferencePackAdminResult<Self> {
        let operation_kind = output.operation_kind();
        let mut receipt = Self {
            schema: REFERENCE_PACK_ADMIN_SCHEMA.into(),
            receipt_id: "pending".into(),
            configuration_id: configuration_id.into(),
            operation_id,
            request_digest,
            operation_kind,
            output,
        };
        receipt.receipt_id = operation_receipt_id(&receipt)?;
        receipt.validate(configuration_id)?;
        Ok(receipt)
    }

    fn validate(&self, configuration_id: &str) -> ReferencePackAdminResult<()> {
        if self.schema.as_ref() != REFERENCE_PACK_ADMIN_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
            || self.operation_kind != self.output.operation_kind()
            || !self.output.validate()
            || self.receipt_id != operation_receipt_id(self)?
        {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "stored reference pack operation receipt is invalid",
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
    pub fn request_digest(&self) -> &RequestDigest {
        &self.request_digest
    }

    #[must_use]
    pub const fn operation_kind(&self) -> ReferencePackOperationKind {
        self.operation_kind
    }

    #[must_use]
    pub fn output(&self) -> &ReferencePackOperationOutput {
        &self.output
    }
}

pub struct ReferencePackOperationExecution<T> {
    receipt: ReferencePackOperationReceipt,
    output: Option<T>,
    replayed: bool,
}

impl<T> ReferencePackOperationExecution<T> {
    #[must_use]
    pub fn receipt(&self) -> &ReferencePackOperationReceipt {
        &self.receipt
    }

    #[must_use]
    pub fn output(&self) -> Option<&T> {
        self.output.as_ref()
    }

    #[must_use]
    pub const fn replayed(&self) -> bool {
        self.replayed
    }

    #[must_use]
    pub fn into_output(self) -> Option<T> {
        self.output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackRecoveryState {
    Cancelled,
    Failed,
    OutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePackResultObservation {
    Absent,
    Present,
    NotChecked,
}

struct RecoveryActions {
    safe: Box<[Box<str>]>,
    prohibited: Box<[Box<str>]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackRecoveryRecord {
    schema: Box<str>,
    recovery_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    operation_kind: ReferencePackOperationKind,
    recovery_state: ReferencePackRecoveryState,
    failed_stage: Box<str>,
    error_code: Box<str>,
    result_observation: ReferencePackResultObservation,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_receipt_id: Option<Box<str>>,
    safe_actions: Box<[Box<str>]>,
    prohibited_actions: Box<[Box<str>]>,
}

impl ReferencePackRecoveryRecord {
    #[allow(clippy::too_many_arguments)]
    fn build(
        configuration_id: &str,
        operation_id: OperationId,
        request_digest: RequestDigest,
        operation_kind: ReferencePackOperationKind,
        recovery_state: ReferencePackRecoveryState,
        failed_stage: impl Into<Box<str>>,
        error_code: impl Into<Box<str>>,
        result_observation: ReferencePackResultObservation,
        observed_receipt_id: Option<&str>,
    ) -> ReferencePackAdminResult<Self> {
        let actions = match recovery_state {
            ReferencePackRecoveryState::Cancelled => RecoveryActions {
                safe: vec![
                    "inspect_exact_request_and_inputs".into(),
                    "start_a_new_operation_id_if_reexecution_is_desired".into(),
                ]
                .into_boxed_slice(),
                prohibited: vec![
                    "claim_pack_completion".into(),
                    "reuse_cancelled_operation_id_for_a_different_request".into(),
                ]
                .into_boxed_slice(),
            },
            ReferencePackRecoveryState::Failed => RecoveryActions {
                safe: vec![
                    "inspect_exact_failure_and_input_binding".into(),
                    "start_a_new_operation_id_after_correcting_the_request".into(),
                ]
                .into_boxed_slice(),
                prohibited: vec![
                    "claim_pack_completion".into(),
                    "rewrite_the_existing_operation_binding".into(),
                ]
                .into_boxed_slice(),
            },
            ReferencePackRecoveryState::OutcomeUnknown => RecoveryActions {
                safe: vec![
                    "reconcile_the_durable_result_catalog".into(),
                    "return_an_observed_receipt_without_reexecuting_effects".into(),
                ]
                .into_boxed_slice(),
                prohibited: vec![
                    "blindly_retry_unknown_external_effects".into(),
                    "claim_pack_completion_without_a_valid_receipt".into(),
                ]
                .into_boxed_slice(),
            },
        };
        let mut record = Self {
            schema: REFERENCE_PACK_ADMIN_SCHEMA.into(),
            recovery_id: "pending".into(),
            configuration_id: configuration_id.into(),
            operation_id,
            request_digest,
            operation_kind,
            recovery_state,
            failed_stage: failed_stage.into(),
            error_code: error_code.into(),
            result_observation,
            observed_receipt_id: observed_receipt_id.map(Into::into),
            safe_actions: actions.safe,
            prohibited_actions: actions.prohibited,
        };
        record.recovery_id = recovery_record_id(&record)?;
        record.validate(configuration_id)?;
        Ok(record)
    }

    fn validate(&self, configuration_id: &str) -> ReferencePackAdminResult<()> {
        if self.schema.as_ref() != REFERENCE_PACK_ADMIN_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
            || !valid_text(&self.failed_stage)
            || !valid_text(&self.error_code)
            || self
                .observed_receipt_id
                .as_deref()
                .is_some_and(|value| !valid_text(value))
            || self.safe_actions.is_empty()
            || self.prohibited_actions.is_empty()
            || !strictly_sorted_unique_or_fixed(&self.safe_actions)
            || !strictly_sorted_unique_or_fixed(&self.prohibited_actions)
            || self.recovery_id != recovery_record_id(self)?
        {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::RecoveryInvalid,
                "stored reference pack recovery record is invalid",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn recovery_id(&self) -> &str {
        &self.recovery_id
    }

    #[must_use]
    pub const fn recovery_state(&self) -> ReferencePackRecoveryState {
        self.recovery_state
    }

    #[must_use]
    pub fn request_digest(&self) -> &RequestDigest {
        &self.request_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePackOperationReconciliation {
    schema: &'static str,
    operation_id: OperationId,
    request_digest: RequestDigest,
    state: OperationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt: Option<ReferencePackOperationReceipt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recovery: Option<ReferencePackRecoveryRecord>,
    next_action: &'static str,
}

impl ReferencePackOperationReconciliation {
    #[must_use]
    pub const fn state(&self) -> OperationState {
        self.state
    }

    #[must_use]
    pub fn receipt(&self) -> Option<&ReferencePackOperationReceipt> {
        self.receipt.as_ref()
    }

    #[must_use]
    pub fn recovery(&self) -> Option<&ReferencePackRecoveryRecord> {
        self.recovery.as_ref()
    }

    #[must_use]
    pub const fn next_action(&self) -> &'static str {
        self.next_action
    }
}

pub struct ReferencePackAdminService {
    configuration: ReferencePackAdminConfiguration,
    store: Store,
}

impl ReferencePackAdminService {
    pub fn open(
        path: impl AsRef<Path>,
        configuration: ReferencePackAdminConfiguration,
    ) -> ReferencePackAdminResult<Self> {
        let store = Store::open(path, configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    pub fn open_in_memory(
        configuration: ReferencePackAdminConfiguration,
    ) -> ReferencePackAdminResult<Self> {
        let store = Store::open_in_memory(configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &ReferencePackAdminConfiguration {
        &self.configuration
    }

    pub fn build(
        &mut self,
        operation_id: OperationId,
        request: &ReferencePackBuildRequest,
        input: &LocalProjectInput,
        stop: &AtomicBool,
    ) -> ReferencePackAdminResult<ReferencePackOperationExecution<ReferencePackBuildOutcome>> {
        let request_digest = build_request_digest(
            self.configuration.configuration_id(),
            ReferencePackOperationKind::Build,
            request,
            &native_input_binding(input)?,
        )?;
        self.execute(
            operation_id,
            request_digest,
            ReferencePackOperationKind::Build,
            || ReferencePackService::reference_pack_build(request, input, stop),
            |outcome| ReferencePackOperationOutput::Build {
                request_id: outcome.request_id().into(),
                execution_profile_id: outcome.execution_profile_id().into(),
                status: outcome.status(),
                pack_id: outcome.plan().pack_id().into(),
                plan_id: outcome.plan().plan_id().into(),
                validation_report_id: outcome.validation_report().report_id().into(),
            },
        )
    }

    pub fn validate(
        &mut self,
        operation_id: OperationId,
        request: &ReferencePackValidationRequest,
        image: &ReferencePackImage,
        stop: &AtomicBool,
    ) -> ReferencePackAdminResult<ReferencePackOperationExecution<ReferencePackValidationReport>>
    {
        let request_digest = validation_request_digest(
            self.configuration.configuration_id(),
            ReferencePackOperationKind::Validate,
            request,
            image,
        )?;
        self.execute(
            operation_id,
            request_digest,
            ReferencePackOperationKind::Validate,
            || ReferencePackService::reference_pack_validate(request, image, stop),
            |report| ReferencePackOperationOutput::Validate {
                pack_id: report.pack_id().into(),
                validation_report_id: report.report_id().into(),
                candidate_eligible: report.candidate_eligible(),
                validated_local_eligible: report.validated_local_eligible(),
            },
        )
    }

    pub fn rebuild_compare(
        &mut self,
        operation_id: OperationId,
        request: &ReferencePackRebuildComparisonRequest,
        input: &LocalProjectInput,
        stop: &AtomicBool,
    ) -> ReferencePackAdminResult<
        ReferencePackOperationExecution<ReferencePackRebuildComparisonReport>,
    > {
        let request_digest = build_request_digest(
            self.configuration.configuration_id(),
            ReferencePackOperationKind::RebuildCompare,
            request,
            &native_input_binding(input)?,
        )?;
        self.execute(
            operation_id,
            request_digest,
            ReferencePackOperationKind::RebuildCompare,
            || ReferencePackService::reference_pack_rebuild_compare(request, input, stop),
            |report| ReferencePackOperationOutput::RebuildCompare {
                report_id: report.report_id().into(),
                status: report.status(),
                left_execution_profile_id: report.left_execution_profile_id().into(),
                right_execution_profile_id: report.right_execution_profile_id().into(),
            },
        )
    }

    pub fn reconcile(
        &mut self,
        operation_id: &OperationId,
        expected_kind: ReferencePackOperationKind,
    ) -> ReferencePackAdminResult<Option<ReferencePackOperationReconciliation>> {
        let Some(mut operation) = self.store.operation(operation_id)? else {
            return Ok(None);
        };
        let mut receipt = self.read_catalog_receipt(operation_id, operation.request_digest())?;
        let mut recovery = self.read_catalog_recovery(operation_id, operation.request_digest())?;

        if operation.state() == OperationState::Prepared {
            if let Some(value) = &receipt {
                operation = self.store.complete_operation(
                    operation_id,
                    operation.request_digest(),
                    &catalog_object_id(&self.store, RESULT_CATALOG, operation_id)?.ok_or_else(
                        || {
                            ReferencePackAdminError::new(
                                ReferencePackAdminErrorCode::ResultInvalid,
                                "reference pack result catalog entry disappeared during reconciliation",
                            )
                        },
                    )?,
                )?;
                if value.operation_kind() != expected_kind {
                    return Err(ReferencePackAdminError::new(
                        ReferencePackAdminErrorCode::ResultInvalid,
                        "reference pack operation kind differs from the reconciled result",
                    ));
                }
            } else if let Some(value) = &recovery {
                operation = self.transition_recovery(operation, value)?;
            }
        }

        if operation.state() == OperationState::OutcomeUnknown && recovery.is_none() {
            let observation = if receipt.is_some() {
                ReferencePackResultObservation::Present
            } else {
                ReferencePackResultObservation::Absent
            };
            let record = ReferencePackRecoveryRecord::build(
                self.configuration.configuration_id(),
                operation_id.clone(),
                operation.request_digest().clone(),
                expected_kind,
                ReferencePackRecoveryState::OutcomeUnknown,
                "durable_result_reconciliation",
                "outcome_unknown",
                observation,
                receipt
                    .as_ref()
                    .map(ReferencePackOperationReceipt::receipt_id),
            )?;
            self.persist_recovery(&record)?;
            recovery = Some(record);
        }

        if receipt
            .as_ref()
            .is_some_and(|value| value.operation_kind() != expected_kind)
            || recovery
                .as_ref()
                .is_some_and(|value| value.operation_kind != expected_kind)
        {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "reference pack operation kind differs from reconciliation request",
            ));
        }

        let next_action = match operation.state() {
            OperationState::Completed => "return_durable_receipt",
            OperationState::Prepared => "resume_same_pure_operation_with_exact_request",
            OperationState::NoEffect => "start_new_operation_if_reexecution_is_desired",
            OperationState::Failed => "correct_request_then_start_new_operation",
            OperationState::OutcomeUnknown if receipt.is_some() => {
                "return_observed_receipt_without_reexecution"
            }
            OperationState::OutcomeUnknown => "operator_review_before_any_retry",
        };
        Ok(Some(ReferencePackOperationReconciliation {
            schema: REFERENCE_PACK_ADMIN_SCHEMA,
            operation_id: operation.operation_id().clone(),
            request_digest: operation.request_digest().clone(),
            state: operation.state(),
            receipt: receipt.take(),
            recovery,
            next_action,
        }))
    }

    fn execute<T, Run, Summarize>(
        &mut self,
        operation_id: OperationId,
        request_digest: RequestDigest,
        kind: ReferencePackOperationKind,
        run: Run,
        summarize: Summarize,
    ) -> ReferencePackAdminResult<ReferencePackOperationExecution<T>>
    where
        Run: FnOnce() -> Result<T, ReferencePackError>,
        Summarize: FnOnce(&T) -> ReferencePackOperationOutput,
    {
        match self
            .store
            .begin_operation(operation_id.clone(), request_digest.clone())?
        {
            OperationBegin::Started(_) => {}
            OperationBegin::Replay(record) => match record.state() {
                OperationState::Completed => {
                    let receipt = self.completed_receipt(&record, kind)?;
                    return Ok(ReferencePackOperationExecution {
                        receipt,
                        output: None,
                        replayed: true,
                    });
                }
                OperationState::Prepared => {
                    if let Some(receipt) =
                        self.read_catalog_receipt(&operation_id, &request_digest)?
                    {
                        if receipt.operation_kind() != kind {
                            return Err(ReferencePackAdminError::new(
                                ReferencePackAdminErrorCode::ResultInvalid,
                                "prepared reference pack result has another operation kind",
                            ));
                        }
                        let object_id =
                            catalog_object_id(&self.store, RESULT_CATALOG, &operation_id)?
                                .ok_or_else(|| {
                                    ReferencePackAdminError::new(
                                        ReferencePackAdminErrorCode::ResultInvalid,
                                        "prepared reference pack result catalog entry is missing",
                                    )
                                })?;
                        self.store.complete_operation(
                            &operation_id,
                            &request_digest,
                            &object_id,
                        )?;
                        return Ok(ReferencePackOperationExecution {
                            receipt,
                            output: None,
                            replayed: true,
                        });
                    }
                    if let Some(recovery) =
                        self.read_catalog_recovery(&operation_id, &request_digest)?
                    {
                        self.transition_recovery(record, &recovery)?;
                        return Err(terminal_recovery_error(&recovery));
                    }
                    // All Reference Pack owner operations in this module are pure:
                    // there is no destination or external effect to repeat. A prepared
                    // record without a durable result may therefore resume exactly.
                }
                OperationState::OutcomeUnknown => {
                    return Err(ReferencePackAdminError::new(
                        ReferencePackAdminErrorCode::OutcomeUnknown,
                        "reference pack operation requires explicit reconciliation",
                    ));
                }
                OperationState::NoEffect | OperationState::Failed => {
                    return Err(ReferencePackAdminError::new(
                        ReferencePackAdminErrorCode::OperationIncomplete,
                        "reference pack operation is terminal without a successful receipt",
                    ));
                }
            },
        }

        let output = match run() {
            Ok(output) => output,
            Err(source) => {
                self.record_pack_failure(
                    operation_id,
                    request_digest,
                    kind,
                    source.code(),
                    source.message(),
                )?;
                return Err(source.into());
            }
        };
        let receipt = ReferencePackOperationReceipt::build(
            self.configuration.configuration_id(),
            operation_id.clone(),
            request_digest.clone(),
            summarize(&output),
        )?;
        self.persist_result(&receipt)?;
        let result_object_id = catalog_object_id(&self.store, RESULT_CATALOG, &operation_id)?
            .ok_or_else(|| {
                ReferencePackAdminError::new(
                    ReferencePackAdminErrorCode::ResultInvalid,
                    "reference pack result receipt was not visible after commit",
                )
            })?;
        if let Err(source) =
            self.store
                .complete_operation(&operation_id, &request_digest, &result_object_id)
        {
            if self.store.operation(&operation_id)?.is_some_and(|record| {
                record.state() == OperationState::Completed
                    && record.result_object_id() == Some(&result_object_id)
            }) {
                return Ok(ReferencePackOperationExecution {
                    receipt,
                    output: Some(output),
                    replayed: false,
                });
            }
            let recovery = ReferencePackRecoveryRecord::build(
                self.configuration.configuration_id(),
                operation_id.clone(),
                request_digest.clone(),
                kind,
                ReferencePackRecoveryState::OutcomeUnknown,
                "operation_completion",
                store_error_code(source.code()),
                ReferencePackResultObservation::Present,
                Some(receipt.receipt_id()),
            )?;
            let _ = self.persist_recovery(&recovery);
            let _ = self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest);
            return Err(ReferencePackAdminError::lower(
                ReferencePackAdminErrorCode::OutcomeUnknown,
                store_error_code(source.code()),
                "reference pack result was committed but operation completion is uncertain",
            ));
        }
        Ok(ReferencePackOperationExecution {
            receipt,
            output: Some(output),
            replayed: false,
        })
    }

    fn record_pack_failure(
        &mut self,
        operation_id: OperationId,
        request_digest: RequestDigest,
        kind: ReferencePackOperationKind,
        code: ReferencePackErrorCode,
        message: &str,
    ) -> ReferencePackAdminResult<()> {
        let recovery_state = if code == ReferencePackErrorCode::Cancelled {
            ReferencePackRecoveryState::Cancelled
        } else {
            ReferencePackRecoveryState::Failed
        };
        let record = ReferencePackRecoveryRecord::build(
            self.configuration.configuration_id(),
            operation_id.clone(),
            request_digest.clone(),
            kind,
            recovery_state,
            kind.stage(),
            pack_error_code(code),
            ReferencePackResultObservation::Absent,
            None,
        )?;
        if let Err(source) = self.persist_recovery(&record) {
            let _ = self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest);
            return Err(ReferencePackAdminError::lower(
                ReferencePackAdminErrorCode::OutcomeUnknown,
                source
                    .lower_code()
                    .unwrap_or("recovery_record_persistence_failed"),
                "reference pack operation failed and its recovery record is uncertain",
            ));
        }
        let transition = match recovery_state {
            ReferencePackRecoveryState::Cancelled => {
                self.store.record_no_effect(&operation_id, &request_digest)
            }
            ReferencePackRecoveryState::Failed => {
                self.store.record_failed(&operation_id, &request_digest)
            }
            ReferencePackRecoveryState::OutcomeUnknown => self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest),
        };
        if let Err(source) = transition {
            let _ = self
                .store
                .mark_outcome_unknown(&operation_id, &request_digest);
            return Err(ReferencePackAdminError::lower(
                ReferencePackAdminErrorCode::OutcomeUnknown,
                store_error_code(source.code()),
                format!(
                    "reference pack operation failed: {message}; terminal journal update is uncertain"
                ),
            ));
        }
        Ok(())
    }

    fn persist_result(
        &mut self,
        receipt: &ReferencePackOperationReceipt,
    ) -> ReferencePackAdminResult<()> {
        let pending = PendingObject::from_json(
            RESULT_KIND,
            RESULT_SCHEMA_VERSION,
            receipt,
            self.configuration.store_configuration().limits(),
        )?;
        self.persist_catalog_object(
            RESULT_CATALOG,
            receipt.operation_id(),
            pending,
            ReferencePackAdminErrorCode::ResultInvalid,
        )
    }

    fn persist_recovery(
        &mut self,
        record: &ReferencePackRecoveryRecord,
    ) -> ReferencePackAdminResult<()> {
        let pending = PendingObject::from_json(
            RECOVERY_KIND,
            RECOVERY_SCHEMA_VERSION,
            record,
            self.configuration.store_configuration().limits(),
        )?;
        self.persist_catalog_object(
            RECOVERY_CATALOG,
            &record.operation_id,
            pending,
            ReferencePackAdminErrorCode::RecoveryInvalid,
        )
    }

    fn persist_catalog_object(
        &mut self,
        catalog: &str,
        operation_id: &OperationId,
        pending: PendingObject,
        invalid_code: ReferencePackAdminErrorCode,
    ) -> ReferencePackAdminResult<()> {
        let target = pending.object_id().clone();
        if let Some(existing) = catalog_object_id(&self.store, catalog, operation_id)? {
            if existing == target {
                return Ok(());
            }
            return Err(ReferencePackAdminError::new(
                invalid_code,
                "reference pack operation catalog is already bound to different content",
            ));
        }
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        batch.add_catalog_mutation(CatalogMutation::set(
            CatalogName::new(catalog)?,
            CatalogPath::new(operation_id.as_str())?,
            CatalogExpectation::Absent,
            target.clone(),
        ))?;
        match self.store.commit(batch) {
            Ok(_) => {}
            Err(source) if source.code() == StoreErrorCode::CatalogConflict => {
                if catalog_object_id(&self.store, catalog, operation_id)?.as_ref() != Some(&target)
                {
                    return Err(source.into());
                }
            }
            Err(source) if source.code() == StoreErrorCode::OutcomeUnknown => {
                if catalog_object_id(&self.store, catalog, operation_id)?.as_ref() != Some(&target)
                {
                    return Err(ReferencePackAdminError::lower(
                        ReferencePackAdminErrorCode::OutcomeUnknown,
                        store_error_code(source.code()),
                        "reference pack operation object commit outcome is unknown",
                    ));
                }
            }
            Err(source) => return Err(source.into()),
        }
        let persisted = self.store.object(&target)?.ok_or_else(|| {
            ReferencePackAdminError::new(
                invalid_code,
                "reference pack operation object is missing after commit",
            )
        })?;
        if persisted.object_id() != &target {
            return Err(ReferencePackAdminError::new(
                invalid_code,
                "reference pack operation object failed exact read-back",
            ));
        }
        Ok(())
    }

    fn completed_receipt(
        &self,
        record: &OperationRecord,
        expected_kind: ReferencePackOperationKind,
    ) -> ReferencePackAdminResult<ReferencePackOperationReceipt> {
        let object_id = record.result_object_id().ok_or_else(|| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "completed reference pack operation is missing its result object",
            )
        })?;
        let receipt = self.read_receipt(object_id, record.request_digest())?;
        if receipt.operation_kind() != expected_kind {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "completed reference pack operation has another result kind",
            ));
        }
        Ok(receipt)
    }

    fn read_catalog_receipt(
        &self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
    ) -> ReferencePackAdminResult<Option<ReferencePackOperationReceipt>> {
        let Some(object_id) = catalog_object_id(&self.store, RESULT_CATALOG, operation_id)? else {
            return Ok(None);
        };
        Ok(Some(self.read_receipt(&object_id, request_digest)?))
    }

    fn read_receipt(
        &self,
        object_id: &ObjectId,
        request_digest: &RequestDigest,
    ) -> ReferencePackAdminResult<ReferencePackOperationReceipt> {
        let record = self.store.object(object_id)?.ok_or_else(|| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "stored reference pack operation receipt is missing",
            )
        })?;
        if record.kind() != RESULT_KIND || record.schema_version() != RESULT_SCHEMA_VERSION {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "stored reference pack operation receipt has an incompatible type",
            ));
        }
        let receipt: ReferencePackOperationReceipt = record.decode().map_err(|_| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "stored reference pack operation receipt cannot be decoded",
            )
        })?;
        receipt.validate(self.configuration.configuration_id())?;
        if receipt.request_digest() != request_digest {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::ResultInvalid,
                "stored reference pack operation receipt belongs to another request",
            ));
        }
        Ok(receipt)
    }

    fn read_catalog_recovery(
        &self,
        operation_id: &OperationId,
        request_digest: &RequestDigest,
    ) -> ReferencePackAdminResult<Option<ReferencePackRecoveryRecord>> {
        let Some(object_id) = catalog_object_id(&self.store, RECOVERY_CATALOG, operation_id)?
        else {
            return Ok(None);
        };
        let record = self.store.object(&object_id)?.ok_or_else(|| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::RecoveryInvalid,
                "stored reference pack recovery record is missing",
            )
        })?;
        if record.kind() != RECOVERY_KIND || record.schema_version() != RECOVERY_SCHEMA_VERSION {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::RecoveryInvalid,
                "stored reference pack recovery record has an incompatible type",
            ));
        }
        let recovery: ReferencePackRecoveryRecord = record.decode().map_err(|_| {
            ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::RecoveryInvalid,
                "stored reference pack recovery record cannot be decoded",
            )
        })?;
        recovery.validate(self.configuration.configuration_id())?;
        if recovery.request_digest() != request_digest {
            return Err(ReferencePackAdminError::new(
                ReferencePackAdminErrorCode::RecoveryInvalid,
                "stored reference pack recovery record belongs to another request",
            ));
        }
        Ok(Some(recovery))
    }

    fn transition_recovery(
        &mut self,
        operation: OperationRecord,
        recovery: &ReferencePackRecoveryRecord,
    ) -> ReferencePackAdminResult<OperationRecord> {
        let result = match recovery.recovery_state() {
            ReferencePackRecoveryState::Cancelled => self
                .store
                .record_no_effect(operation.operation_id(), operation.request_digest()),
            ReferencePackRecoveryState::Failed => self
                .store
                .record_failed(operation.operation_id(), operation.request_digest()),
            ReferencePackRecoveryState::OutcomeUnknown => self
                .store
                .mark_outcome_unknown(operation.operation_id(), operation.request_digest()),
        }?;
        Ok(result)
    }
}

#[derive(Serialize)]
struct NativeInputBinding<'a> {
    receipt: &'a crate::local::NativeInputReceipt,
    reference_view_digest: &'a str,
    annotation_artifact_id: &'a str,
    source_map_id: &'a str,
    loss_report_id: &'a str,
    compatibility_evidence_id: Option<&'a str>,
    distribution_manifest_id: Option<&'a str>,
}

fn native_input_binding(
    input: &LocalProjectInput,
) -> ReferencePackAdminResult<NativeInputBinding<'_>> {
    let receipt = input.native_input_receipt().ok_or_else(|| {
        ReferencePackAdminError::new(
            ReferencePackAdminErrorCode::InvalidRequest,
            "durable Reference Pack operations require manifested native input",
        )
    })?;
    let artifact = input.native_annotation_artifact().ok_or_else(|| {
        ReferencePackAdminError::new(
            ReferencePackAdminErrorCode::InvalidRequest,
            "durable Reference Pack operations require an annotation artifact",
        )
    })?;
    let sidecars = input.native_annotation_sidecars().ok_or_else(|| {
        ReferencePackAdminError::new(
            ReferencePackAdminErrorCode::InvalidRequest,
            "durable Reference Pack operations require annotation sidecars",
        )
    })?;
    Ok(NativeInputBinding {
        receipt,
        reference_view_digest: input.reference_view().self_digest(),
        annotation_artifact_id: artifact.artifact_id(),
        source_map_id: sidecars.source_map().source_map_id(),
        loss_report_id: sidecars.loss_report().report_id(),
        compatibility_evidence_id: input
            .native_annotation_compatibility()
            .map(|evidence| evidence.evidence_id()),
        distribution_manifest_id: input
            .native_distribution_evidence()
            .map(|evidence| evidence.manifest_id()),
    })
}

fn build_request_digest<Request: Serialize + ?Sized>(
    configuration_id: &str,
    operation_kind: ReferencePackOperationKind,
    request: &Request,
    input: &NativeInputBinding<'_>,
) -> ReferencePackAdminResult<RequestDigest> {
    #[derive(Serialize)]
    struct Identity<'a, 'b, Request: Serialize + ?Sized> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_kind: ReferencePackOperationKind,
        request: &'a Request,
        input: &'a NativeInputBinding<'b>,
    }
    request_digest(&Identity {
        schema: "wow-service/reference-pack-operation-request/1",
        configuration_id,
        operation_kind,
        request,
        input,
    })
}

fn validation_request_digest(
    configuration_id: &str,
    operation_kind: ReferencePackOperationKind,
    request: &ReferencePackValidationRequest,
    image: &ReferencePackImage,
) -> ReferencePackAdminResult<RequestDigest> {
    #[derive(Serialize)]
    struct FileIdentity {
        path: Box<str>,
        byte_length: u64,
        sha256: Box<str>,
    }
    let mut files = image
        .files()
        .iter()
        .map(|file| FileIdentity {
            path: file.path().into(),
            byte_length: file.bytes().len() as u64,
            sha256: format!("sha256:{}", hex(&Sha256::digest(file.bytes()))).into(),
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.path.cmp(&right.path));
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_kind: ReferencePackOperationKind,
        request: &'a ReferencePackValidationRequest,
        files: &'a [FileIdentity],
    }
    request_digest(&Identity {
        schema: "wow-service/reference-pack-validation-operation-request/1",
        configuration_id,
        operation_kind,
        request,
        files: &files,
    })
}

fn request_digest<T: Serialize + ?Sized>(value: &T) -> ReferencePackAdminResult<RequestDigest> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        ReferencePackAdminError::new(
            ReferencePackAdminErrorCode::InvalidRequest,
            "reference pack durable request cannot be canonicalized",
        )
    })?;
    Ok(RequestDigest::new(format!(
        "sha256:{}",
        hex(&Sha256::digest(bytes))
    ))?)
}

fn operation_receipt_id(
    receipt: &ReferencePackOperationReceipt,
) -> ReferencePackAdminResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        operation_kind: ReferencePackOperationKind,
        output: &'a ReferencePackOperationOutput,
    }
    content_id(
        "reference-pack-operation-result",
        &Identity {
            schema: REFERENCE_PACK_ADMIN_SCHEMA,
            configuration_id: &receipt.configuration_id,
            operation_id: &receipt.operation_id,
            request_digest: &receipt.request_digest,
            operation_kind: receipt.operation_kind,
            output: &receipt.output,
        },
    )
}

fn recovery_record_id(record: &ReferencePackRecoveryRecord) -> ReferencePackAdminResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        operation_kind: ReferencePackOperationKind,
        recovery_state: ReferencePackRecoveryState,
        failed_stage: &'a str,
        error_code: &'a str,
        result_observation: ReferencePackResultObservation,
        observed_receipt_id: Option<&'a str>,
        safe_actions: &'a [Box<str>],
        prohibited_actions: &'a [Box<str>],
    }
    content_id(
        "reference-pack-recovery-record",
        &Identity {
            schema: REFERENCE_PACK_ADMIN_SCHEMA,
            configuration_id: &record.configuration_id,
            operation_id: &record.operation_id,
            request_digest: &record.request_digest,
            operation_kind: record.operation_kind,
            recovery_state: record.recovery_state,
            failed_stage: &record.failed_stage,
            error_code: &record.error_code,
            result_observation: record.result_observation,
            observed_receipt_id: record.observed_receipt_id.as_deref(),
            safe_actions: &record.safe_actions,
            prohibited_actions: &record.prohibited_actions,
        },
    )
}

fn content_id<T: Serialize + ?Sized>(
    prefix: &str,
    value: &T,
) -> ReferencePackAdminResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        ReferencePackAdminError::new(
            ReferencePackAdminErrorCode::ResultInvalid,
            "reference pack durable identity cannot be canonicalized",
        )
    })?;
    Ok(format!("{prefix}:sha256:{}", hex(&Sha256::digest(bytes))).into())
}

fn catalog_object_id(
    store: &Store,
    catalog: &str,
    operation_id: &OperationId,
) -> ReferencePackAdminResult<Option<ObjectId>> {
    Ok(store
        .catalog_entry(
            &CatalogName::new(catalog)?,
            &CatalogPath::new(operation_id.as_str())?,
        )?
        .map(|entry| entry.object_id().clone()))
}

fn terminal_recovery_error(record: &ReferencePackRecoveryRecord) -> ReferencePackAdminError {
    let code = match record.recovery_state() {
        ReferencePackRecoveryState::Cancelled | ReferencePackRecoveryState::Failed => {
            ReferencePackAdminErrorCode::OperationIncomplete
        }
        ReferencePackRecoveryState::OutcomeUnknown => ReferencePackAdminErrorCode::OutcomeUnknown,
    };
    ReferencePackAdminError::lower(
        code,
        record.error_code.clone(),
        "reference pack operation has a durable recovery record",
    )
}

fn valid_text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 1024
        && !value
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
}

fn strictly_sorted_unique_or_fixed(values: &[Box<str>]) -> bool {
    values.iter().all(|value| valid_text(value))
        && values
            .windows(2)
            .all(|pair| pair[0].as_ref() != pair[1].as_ref())
}

fn pack_error_code(code: ReferencePackErrorCode) -> &'static str {
    match code {
        ReferencePackErrorCode::InvalidRequest => "invalid_request",
        ReferencePackErrorCode::UnsupportedLayout => "unsupported_layout",
        ReferencePackErrorCode::SourceInputUnavailable => "source_input_unavailable",
        ReferencePackErrorCode::IdentityMismatch => "identity_mismatch",
        ReferencePackErrorCode::MemberInvalid => "member_invalid",
        ReferencePackErrorCode::BudgetExceeded => "budget_exceeded",
        ReferencePackErrorCode::Cancelled => "cancelled",
        ReferencePackErrorCode::ManifestInvalid => "manifest_invalid",
        ReferencePackErrorCode::ValidationFailed => "validation_failed",
        ReferencePackErrorCode::SerializationFailed => "serialization_failed",
    }
}

fn store_error_code(code: StoreErrorCode) -> &'static str {
    match code {
        StoreErrorCode::ConfigurationInvalid => "configuration_invalid",
        StoreErrorCode::IdentifierInvalid => "identifier_invalid",
        StoreErrorCode::JsonInvalid => "json_invalid",
        StoreErrorCode::ObjectTooLarge => "object_too_large",
        StoreErrorCode::BatchTooLarge => "batch_too_large",
        StoreErrorCode::ObjectMissing => "object_missing",
        StoreErrorCode::ObjectConflict => "object_conflict",
        StoreErrorCode::CatalogConflict => "catalog_conflict",
        StoreErrorCode::OperationConflict => "operation_conflict",
        StoreErrorCode::OperationStateInvalid => "operation_state_invalid",
        StoreErrorCode::LeaseConflict => "lease_conflict",
        StoreErrorCode::LeaseInvalid => "lease_invalid",
        StoreErrorCode::IntegrityViolation => "integrity_violation",
        StoreErrorCode::BudgetExceeded => "budget_exceeded",
        StoreErrorCode::DatabaseUnavailable => "database_unavailable",
        StoreErrorCode::Cancelled => "cancelled",
        StoreErrorCode::WriterBusy => "writer_busy",
        StoreErrorCode::CurrentConflict => "current_conflict",
        StoreErrorCode::GenerationMissing => "generation_missing",
        StoreErrorCode::OutcomeUnknown => "outcome_unknown",
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
