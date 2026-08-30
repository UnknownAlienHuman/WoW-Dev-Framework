use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreErrorCode {
    InvalidIdentifier,
    InvalidProfileIdentity,
    GenerationMismatch,
    InvalidSourceHandle,
    DigestMismatch,
    CoverageConflict,
    NegativeAuthorityUnavailable,
    ResultContextViolation,
    BudgetInvalid,
    SchemaVersionUnsupported,
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreError {
    pub code: CoreErrorCode,
    pub operation: &'static str,
    pub detail: String,
}

impl CoreError {
    #[must_use]
    pub fn new(code: CoreErrorCode, operation: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            operation,
            detail: detail.into(),
        }
    }

    #[must_use]
    pub fn invalid_identifier(kind: &'static str, detail: impl Into<String>) -> Self {
        Self::new(
            CoreErrorCode::InvalidIdentifier,
            "parse_identifier",
            format!("{kind}: {}", detail.into()),
        )
    }

    #[must_use]
    pub fn generation_mismatch(
        component: &'static str,
        left: impl Display,
        right: impl Display,
    ) -> Self {
        Self::new(
            CoreErrorCode::GenerationMismatch,
            "merge_generation_context",
            format!("{component} differs: {left} != {right}"),
        )
    }
}

impl Display for CoreError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?} during {}: {}", self.code, self.operation, self.detail)
    }
}

impl Error for CoreError {}

impl From<serde_json::Error> for CoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(
            CoreErrorCode::Serialization,
            "canonical_json",
            error.to_string(),
        )
    }
}
