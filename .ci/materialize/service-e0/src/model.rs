use std::sync::Arc;

use serde::Serialize;
use wow_rules::{RuleEvaluationInput, RuleReport};

use crate::identity::validate_identity;
use crate::{ServiceResult, registry::RULES_EVALUATE_ID, registry::RULES_EVALUATE_VERSION};

pub(crate) const REQUEST_SCHEMA: &str = "wow-service/request/1";
pub(crate) const RESPONSE_SCHEMA: &str = "wow-service/response/1";
pub(crate) const SNAPSHOT_SCHEMA: &str = "wow-service/snapshot/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationSelector {
    operation_id: Box<str>,
    version: u32,
}

impl OperationSelector {
    pub fn new(operation_id: impl Into<Box<str>>, version: u32) -> ServiceResult<Self> {
        let operation_id = operation_id.into();
        validate_identity(&operation_id)?;
        Ok(Self {
            operation_id,
            version,
        })
    }

    #[must_use]
    pub fn evaluate_rules() -> Self {
        Self {
            operation_id: RULES_EVALUATE_ID.into(),
            version: RULES_EVALUATE_VERSION,
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceRequest {
    schema: &'static str,
    request_id: Box<str>,
    expected_service_snapshot_id: Box<str>,
    operation: OperationSelector,
}

impl ServiceRequest {
    pub fn new(
        request_id: impl Into<Box<str>>,
        expected_service_snapshot_id: impl Into<Box<str>>,
        operation: OperationSelector,
    ) -> ServiceResult<Self> {
        let request_id = request_id.into();
        let expected_service_snapshot_id = expected_service_snapshot_id.into();
        validate_identity(&request_id)?;
        validate_identity(&expected_service_snapshot_id)?;
        Ok(Self {
            schema: REQUEST_SCHEMA,
            request_id,
            expected_service_snapshot_id,
            operation,
        })
    }

    #[must_use]
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    #[must_use]
    pub fn expected_service_snapshot_id(&self) -> &str {
        &self.expected_service_snapshot_id
    }

    #[must_use]
    pub const fn operation(&self) -> &OperationSelector {
        &self.operation
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceResponseStatus {
    Completed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceFailureCode {
    UnknownOperation,
    UnsupportedOperationVersion,
    StaleServiceSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceFailure {
    pub code: ServiceFailureCode,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceResponse {
    schema: &'static str,
    response_id: Box<str>,
    request_id: Box<str>,
    operation: OperationSelector,
    service_snapshot_id: Box<str>,
    project_snapshot_id: Box<str>,
    project_generation: Box<str>,
    reference_view_id: Box<str>,
    target_profile: Box<str>,
    status: ServiceResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<RuleReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<ServiceFailure>,
}

impl ServiceResponse {
    pub(crate) fn completed(
        response_id: Box<str>,
        request: &ServiceRequest,
        snapshot: &ServiceSnapshot,
    ) -> Self {
        Self {
            schema: RESPONSE_SCHEMA,
            response_id,
            request_id: request.request_id.clone(),
            operation: request.operation.clone(),
            service_snapshot_id: snapshot.snapshot_id.clone(),
            project_snapshot_id: snapshot.input.project_snapshot_id.clone(),
            project_generation: snapshot.input.project_generation.clone(),
            reference_view_id: snapshot.input.reference_view_id.clone(),
            target_profile: snapshot.input.target_profile.clone(),
            status: ServiceResponseStatus::Completed,
            result: Some((*snapshot.report).clone()),
            failure: None,
        }
    }

    pub(crate) fn rejected(
        response_id: Box<str>,
        request: &ServiceRequest,
        snapshot: &ServiceSnapshot,
        failure: ServiceFailure,
    ) -> Self {
        Self {
            schema: RESPONSE_SCHEMA,
            response_id,
            request_id: request.request_id.clone(),
            operation: request.operation.clone(),
            service_snapshot_id: snapshot.snapshot_id.clone(),
            project_snapshot_id: snapshot.input.project_snapshot_id.clone(),
            project_generation: snapshot.input.project_generation.clone(),
            reference_view_id: snapshot.input.reference_view_id.clone(),
            target_profile: snapshot.input.target_profile.clone(),
            status: ServiceResponseStatus::Rejected,
            result: None,
            failure: Some(failure),
        }
    }

    #[must_use]
    pub fn response_id(&self) -> &str {
        &self.response_id
    }

    #[must_use]
    pub const fn status(&self) -> ServiceResponseStatus {
        self.status
    }

    #[must_use]
    pub fn service_snapshot_id(&self) -> &str {
        &self.service_snapshot_id
    }

    #[must_use]
    pub fn result(&self) -> Option<&RuleReport> {
        self.result.as_ref()
    }

    #[must_use]
    pub const fn failure(&self) -> Option<&ServiceFailure> {
        self.failure.as_ref()
    }
}

#[derive(Debug)]
pub struct ServiceSnapshot {
    pub(crate) schema: &'static str,
    pub(crate) snapshot_id: Box<str>,
    pub(crate) input: Arc<RuleEvaluationInput>,
    pub(crate) report: Arc<RuleReport>,
}

impl ServiceSnapshot {
    #[must_use]
    pub const fn schema(&self) -> &str {
        self.schema
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.input.project_snapshot_id
    }

    #[must_use]
    pub fn project_generation(&self) -> &str {
        &self.input.project_generation
    }

    #[must_use]
    pub fn reference_view_id(&self) -> &str {
        &self.input.reference_view_id
    }

    #[must_use]
    pub fn target_profile(&self) -> &str {
        &self.input.target_profile
    }

    #[must_use]
    pub fn rule_report(&self) -> &RuleReport {
        &self.report
    }
}

#[derive(Debug, Clone)]
pub struct ServiceReadView {
    pub(crate) snapshot: Arc<ServiceSnapshot>,
}

impl ServiceReadView {
    #[must_use]
    pub fn snapshot(&self) -> &ServiceSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        self.snapshot.snapshot_id()
    }

    #[must_use]
    pub fn is_same_snapshot(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.snapshot, &other.snapshot)
    }
}
