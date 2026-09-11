use std::fmt;
use std::io::{self, Write};

use serde::Serialize;
use wow_service::{ServiceError, ServiceErrorCode, ServiceSemanticStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AppErrorCode {
    Usage,
    InputIo,
    InputLimit,
    InputInvalid,
    Service,
    OutputIo,
}

#[derive(Debug)]
pub(crate) struct AppError {
    code: AppErrorCode,
    message: Box<str>,
    service_code: Option<ServiceErrorCode>,
    operation_id: Option<Box<str>>,
}

impl AppError {
    pub(crate) fn usage(message: impl Into<Box<str>>) -> Self {
        Self {
            code: AppErrorCode::Usage,
            message: message.into(),
            service_code: None,
            operation_id: None,
        }
    }

    pub(crate) fn input_io(message: impl Into<Box<str>>) -> Self {
        Self {
            code: AppErrorCode::InputIo,
            message: message.into(),
            service_code: None,
            operation_id: None,
        }
    }

    pub(crate) fn input_limit(message: impl Into<Box<str>>) -> Self {
        Self {
            code: AppErrorCode::InputLimit,
            message: message.into(),
            service_code: None,
            operation_id: None,
        }
    }

    pub(crate) fn input_invalid(message: impl Into<Box<str>>) -> Self {
        Self {
            code: AppErrorCode::InputInvalid,
            message: message.into(),
            service_code: None,
            operation_id: None,
        }
    }

    pub(crate) fn output_io(error: io::Error) -> Self {
        Self {
            code: AppErrorCode::OutputIo,
            message: format!("failed to write CLI output: {error}").into(),
            service_code: None,
            operation_id: None,
        }
    }

    #[must_use]
    pub(crate) const fn exit_code(&self) -> i32 {
        match self.code {
            AppErrorCode::Usage => 64,
            AppErrorCode::InputInvalid | AppErrorCode::InputLimit => 65,
            AppErrorCode::InputIo => 66,
            AppErrorCode::Service => match self.service_code {
                Some(ServiceErrorCode::Cancelled) => 130,
                Some(
                    ServiceErrorCode::ExactGenerationUnavailable
                    | ServiceErrorCode::CurrentGenerationUnavailable
                    | ServiceErrorCode::ComponentUnavailable
                    | ServiceErrorCode::OperationNotImplementedForMilestone,
                ) => 69,
                Some(
                    ServiceErrorCode::InvalidConfiguration
                    | ServiceErrorCode::InvalidRequest
                    | ServiceErrorCode::InvalidContext
                    | ServiceErrorCode::IdentityMismatch
                    | ServiceErrorCode::OperationConflict
                    | ServiceErrorCode::OperationBusy
                    | ServiceErrorCode::BudgetExceeded
                    | ServiceErrorCode::CanonicalizationFailed
                    | ServiceErrorCode::InternalContractViolation,
                )
                | None => 70,
            },
            AppErrorCode::OutputIo => 74,
        }
    }

    pub(crate) fn write_json(&self, mut writer: impl Write) -> Result<(), io::Error> {
        #[derive(Serialize)]
        struct ErrorEnvelope<'a> {
            schema: &'static str,
            code: AppErrorCode,
            message: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            service_code: Option<ServiceErrorCode>,
            #[serde(skip_serializing_if = "Option::is_none")]
            operation_id: Option<&'a str>,
        }
        serde_json::to_writer(
            &mut writer,
            &ErrorEnvelope {
                schema: "wow-app/error/1",
                code: self.code,
                message: &self.message,
                service_code: self.service_code,
                operation_id: self.operation_id.as_deref(),
            },
        )?;
        writer.write_all(b"\n")
    }
}

impl From<ServiceError> for AppError {
    fn from(error: ServiceError) -> Self {
        Self {
            code: AppErrorCode::Service,
            message: error.message().into(),
            service_code: Some(error.code()),
            operation_id: error.operation_id().map(Into::into),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppError {}

pub(crate) const fn check_exit_code(status: ServiceSemanticStatus) -> i32 {
    match status {
        ServiceSemanticStatus::Clean => 0,
        ServiceSemanticStatus::Findings => 1,
        ServiceSemanticStatus::Partial => 2,
        ServiceSemanticStatus::Failed => 70,
        ServiceSemanticStatus::Cancelled => 130,
    }
}

pub(crate) fn write_json<T: Serialize>(
    mut writer: impl Write,
    value: &T,
) -> Result<(), AppError> {
    serde_json::to_writer(&mut writer, value).map_err(AppError::output_io)?;
    writer.write_all(b"\n").map_err(AppError::output_io)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_status_exit_codes_are_stable() {
        assert_eq!(check_exit_code(ServiceSemanticStatus::Clean), 0);
        assert_eq!(check_exit_code(ServiceSemanticStatus::Findings), 1);
        assert_eq!(check_exit_code(ServiceSemanticStatus::Partial), 2);
        assert_eq!(check_exit_code(ServiceSemanticStatus::Failed), 70);
        assert_eq!(check_exit_code(ServiceSemanticStatus::Cancelled), 130);
    }
}
