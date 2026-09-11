use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceErrorCode {
    InvalidIdentity,
    RuleInputRejected,
    SnapshotBuildFailed,
    PublicationConflict,
    LockPoisoned,
    CanonicalizationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceError {
    code: ServiceErrorCode,
    message: Box<str>,
}

impl ServiceError {
    pub(crate) fn new(code: ServiceErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
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
}

impl fmt::Display for ServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ServiceError {}

pub type ServiceResult<T> = Result<T, ServiceError>;
