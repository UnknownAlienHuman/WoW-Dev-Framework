use std::fmt;

use serde::Serialize;

/// Stable service failure families. Message text is explanatory, not identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceErrorCode {
    InvalidConfiguration,
    InvalidRequest,
    InvalidContext,
    IdentityMismatch,
    ExactGenerationUnavailable,
    CurrentGenerationUnavailable,
    ComponentUnavailable,
    OperationConflict,
    OperationBusy,
    OperationNotImplementedForMilestone,
    BudgetExceeded,
    Cancelled,
    CanonicalizationFailed,
    InternalContractViolation,
}

/// Compact typed error suitable for application projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceError {
    code: ServiceErrorCode,
    message: Box<str>,
    operation_id: Option<Box<str>>,
}

impl ServiceError {
    pub(crate) fn new(code: ServiceErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
            operation_id: None,
        }
    }

    pub(crate) fn for_operation(
        code: ServiceErrorCode,
        message: impl Into<Box<str>>,
        operation_id: &str,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            operation_id: Some(operation_id.into()),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ServiceErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn operation_id(&self) -> Option<&str> {
        self.operation_id.as_deref()
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(operation_id) = self.operation_id() {
            write!(formatter, "{} (operation {operation_id})", self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for ServiceError {}

pub type ServiceResult<T> = Result<T, ServiceError>;
