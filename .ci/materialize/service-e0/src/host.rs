use std::sync::{Arc, RwLock};

use serde::Serialize;
use wow_rules::{RuleEvaluationInput, evaluate};

use crate::identity::{canonical_id, validate_identity};
use crate::model::SNAPSHOT_SCHEMA;
use crate::registry::{OperationRegistry, RULES_EVALUATE_ID};
use crate::{
    ServiceError, ServiceErrorCode, ServiceFailure, ServiceFailureCode, ServiceReadView,
    ServiceRequest, ServiceResponse, ServiceResponseStatus, ServiceResult, ServiceSnapshot,
};

#[derive(Debug)]
pub struct ServiceHost {
    current: RwLock<Arc<ServiceSnapshot>>,
}

impl ServiceHost {
    pub fn new(input: RuleEvaluationInput) -> ServiceResult<Self> {
        Ok(Self {
            current: RwLock::new(build_snapshot(input)?),
        })
    }

    /// Captures one immutable snapshot. The returned view cannot change when a
    /// later publication succeeds.
    pub fn read(&self) -> ServiceResult<ServiceReadView> {
        let snapshot = self
            .current
            .read()
            .map_err(|_| lock_error())?
            .clone();
        Ok(ServiceReadView { snapshot })
    }

    /// Builds and validates the complete candidate before taking the publication
    /// lock. A stale expected snapshot cannot move the current read view.
    pub fn publish(
        &self,
        expected_current_snapshot_id: &str,
        input: RuleEvaluationInput,
    ) -> ServiceResult<ServiceReadView> {
        validate_identity(expected_current_snapshot_id)?;
        let candidate = build_snapshot(input)?;
        let mut current = self.current.write().map_err(|_| lock_error())?;
        if current.snapshot_id() != expected_current_snapshot_id {
            return Err(ServiceError::new(
                ServiceErrorCode::PublicationConflict,
                "expected service snapshot is no longer current",
            ));
        }
        if current.snapshot_id() == candidate.snapshot_id() {
            return Ok(ServiceReadView {
                snapshot: current.clone(),
            });
        }
        *current = candidate.clone();
        Ok(ServiceReadView {
            snapshot: candidate,
        })
    }

    /// Clones the current Arc exactly once and executes against that immutable
    /// snapshot. No operation can observe a mixed project/Reference generation.
    pub fn execute(&self, request: &ServiceRequest) -> ServiceResult<ServiceResponse> {
        let view = self.read()?;
        execute_on(&view, request)
    }
}

fn build_snapshot(input: RuleEvaluationInput) -> ServiceResult<Arc<ServiceSnapshot>> {
    let report = evaluate(&input).map_err(|error| {
        ServiceError::new(
            ServiceErrorCode::RuleInputRejected,
            format!("rule input rejected: {:?}", error.code()),
        )
    })?;
    #[derive(Serialize)]
    struct SnapshotIdentity<'a> {
        schema: &'static str,
        project_snapshot_id: &'a str,
        project_generation: &'a str,
        reference_view_id: &'a str,
        target_profile: &'a str,
        rule_input_id: &'a str,
        rule_report_id: &'a str,
    }
    let identity = SnapshotIdentity {
        schema: SNAPSHOT_SCHEMA,
        project_snapshot_id: &input.project_snapshot_id,
        project_generation: &input.project_generation,
        reference_view_id: &input.reference_view_id,
        target_profile: &input.target_profile,
        rule_input_id: &report.input_id,
        rule_report_id: report.report_id(),
    };
    let snapshot_id = canonical_id("service-snapshot:sha256:", &identity)?;
    Ok(Arc::new(ServiceSnapshot {
        schema: SNAPSHOT_SCHEMA,
        snapshot_id,
        input: Arc::new(input),
        report: Arc::new(report),
    }))
}

fn execute_on(view: &ServiceReadView, request: &ServiceRequest) -> ServiceResult<ServiceResponse> {
    let snapshot = view.snapshot();
    let failure = if request.expected_service_snapshot_id() != snapshot.snapshot_id() {
        Some(ServiceFailure {
            code: ServiceFailureCode::StaleServiceSnapshot,
            reason: "expected_service_snapshot_is_not_current",
        })
    } else if OperationRegistry::resolve(
        request.operation().operation_id(),
        request.operation().version(),
    )
    .is_some()
    {
        None
    } else if OperationRegistry::contains_id(request.operation().operation_id()) {
        Some(ServiceFailure {
            code: ServiceFailureCode::UnsupportedOperationVersion,
            reason: "operation_version_is_not_registered",
        })
    } else {
        Some(ServiceFailure {
            code: ServiceFailureCode::UnknownOperation,
            reason: "operation_id_is_not_registered",
        })
    };
    let status = if failure.is_some() {
        ServiceResponseStatus::Rejected
    } else {
        ServiceResponseStatus::Completed
    };
    #[derive(Serialize)]
    struct ResponseIdentity<'a> {
        request_id: &'a str,
        operation_id: &'a str,
        operation_version: u32,
        service_snapshot_id: &'a str,
        status: ServiceResponseStatus,
        #[serde(skip_serializing_if = "Option::is_none")]
        result_report_id: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        failure: Option<&'a ServiceFailure>,
    }
    let identity = ResponseIdentity {
        request_id: request.request_id(),
        operation_id: request.operation().operation_id(),
        operation_version: request.operation().version(),
        service_snapshot_id: snapshot.snapshot_id(),
        status,
        result_report_id: failure.is_none().then(|| snapshot.rule_report().report_id()),
        failure: failure.as_ref(),
    };
    let response_id = canonical_id("service-response:sha256:", &identity)?;
    Ok(match failure {
        Some(failure) => ServiceResponse::rejected(response_id, request, snapshot, failure),
        None => {
            debug_assert_eq!(request.operation().operation_id(), RULES_EVALUATE_ID);
            ServiceResponse::completed(response_id, request, snapshot)
        }
    })
}

fn lock_error() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::LockPoisoned,
        "service publication lock is poisoned",
    )
}
