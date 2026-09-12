use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecognizerErrorCode {
    IdentifierInvalid,
    LimitsInvalid,
    BudgetExceeded,
    Cancelled,
    RegistryEmpty,
    RegistryDuplicate,
    RegistryIncomplete,
    RegistryIdentityMismatch,
    AdapterBindingInvalid,
    AdapterBindingDuplicate,
    AdapterBindingMissing,
    AdapterBindingUnknown,
    AdapterFactMismatch,
    AdapterIdentityMismatch,
    PackTooLarge,
    PackSyntaxInvalid,
    PackNonCanonical,
    PackSchemaUnsupported,
    PackInvalid,
    PackBudgetInvalid,
    PackRuleDuplicate,
    PackClauseInvalid,
    PackNegativeCoverageMissing,
    PackOutputInvalid,
    PackIdentityMismatch,
    ObservationDuplicate,
    ObservationInvalid,
    ObservationSnapshotMismatch,
    ObservationEndpointMissing,
    CoverageDuplicate,
    CoverageInvalid,
    AssertionDuplicate,
    AssertionInvalid,
    ReportIdentityMismatch,
    GraphProjectionFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecognizerError {
    code: RecognizerErrorCode,
    message: Box<str>,
}

impl RecognizerError {
    pub(crate) fn new(code: RecognizerErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> RecognizerErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RecognizerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for RecognizerError {}

pub type RecognizerResult<T> = Result<T, RecognizerError>;
