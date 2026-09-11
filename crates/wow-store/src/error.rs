use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreErrorCode {
    ConfigurationInvalid,
    IdentifierInvalid,
    JsonInvalid,
    ObjectTooLarge,
    BatchTooLarge,
    ObjectMissing,
    ObjectConflict,
    CatalogConflict,
    OperationConflict,
    OperationStateInvalid,
    LeaseConflict,
    LeaseInvalid,
    IntegrityViolation,
    BudgetExceeded,
    DatabaseUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    code: StoreErrorCode,
    message: Box<str>,
}

impl StoreError {
    pub(crate) fn new(code: StoreErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> StoreErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn database(_source: rusqlite::Error) -> Self {
        Self::new(
            StoreErrorCode::DatabaseUnavailable,
            "durable store operation failed",
        )
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StoreError {}

pub type StoreResult<T> = Result<T, StoreError>;
