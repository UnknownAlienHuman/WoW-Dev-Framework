use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Result type used by pure `wow-core` operations.
pub type CoreResult<T> = Result<T, CoreError>;

/// Stable machine-readable error codes defined by the E0-A contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreErrorCode {
    InvalidIdentifier,
    ReservedIdentifierSegment,
    IdentifierTooLong,
    InvalidEntityKey,
    NoncanonicalPercentEncoding,
    InvalidDigest,
    UnsupportedDigestAlgorithm,
    UnsupportedIdentifierFamily,
    DigestPurposeMismatch,
    DigestMismatch,
    InvalidProfileIdentity,
    ProfileKindViolation,
    ProfileMismatch,
    GenerationMismatch,
    DuplicateExternalGenerationScope,
    MergeModeViolation,
    DuplicateSchemaId,
    DuplicateProducerId,
    InvalidSourcePath,
    PathEscape,
    AbsolutePathForbidden,
    UnsupportedNonUtf8Path,
    InvalidSourceSpan,
    SpanStateConflict,
    InvalidSourceHandle,
    MissingSourceHandle,
    EvidenceAuthorityViolation,
    EvidenceContextMismatch,
    DerivedEvidenceMissingInputs,
    EvidenceDerivationCycle,
    MissingEvidenceReference,
    DuplicateEvidenceReference,
    ConflictContextMismatch,
    ConflictScopeEmpty,
    MissingConflictReference,
    DuplicateConflictReference,
    CoverageConflict,
    CoverageRecordMissing,
    CoverageContextMismatch,
    DuplicateCoverageRecord,
    NegativeAuthorityUnavailable,
    InvalidMessageArgument,
    FindingContextMismatch,
    WarningContextMismatch,
    RemediationAuthorityViolation,
    ResultDuplicateId,
    ResultContextViolation,
    ResultStatusViolation,
    ResultReferenceViolation,
    CanonicalizationFailure,
    CanonicalDigestMismatch,
    ContractViolation,
    BudgetInvalid,
    BudgetExceeded,
    UsageOverflow,
    SchemaVersionUnsupported,
    UnknownField,
    DuplicateField,
}

/// Broad error category retained across transports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Validation,
    Mismatch,
    Unsupported,
    Budget,
    Invariant,
}

/// Whether the same operation may be retried and under which condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryClass {
    Never,
    AfterInputChange,
    AfterDependencyRecovery,
}

/// Safe structured argument attached to an error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorArgument {
    name: Box<str>,
    value: Box<str>,
}

impl ErrorArgument {
    /// Creates a bounded structured argument.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into().into_boxed_str(),
            value: value.into().into_boxed_str(),
        }
    }

    /// Argument name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Canonical safe value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Stable, non-secret boundary error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreError {
    code: CoreErrorCode,
    category: ErrorCategory,
    operation_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field_path: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_kind: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject_id: Option<Box<str>>,
    reason_arguments: Box<[ErrorArgument]>,
    retry_class: RetryClass,
    cause_codes: Box<[CoreErrorCode]>,
}

impl CoreError {
    /// Creates a structured error with no unsafe input echo.
    #[must_use]
    pub fn new(
        code: CoreErrorCode,
        category: ErrorCategory,
        operation_id: impl Into<String>,
        retry_class: RetryClass,
    ) -> Self {
        Self {
            code,
            category,
            operation_id: operation_id.into().into_boxed_str(),
            field_path: None,
            subject_kind: None,
            subject_id: None,
            reason_arguments: Box::new([]),
            retry_class,
            cause_codes: Box::new([]),
        }
    }

    /// Adds a safe schema field path.
    #[must_use]
    pub fn at_field(mut self, field_path: impl Into<String>) -> Self {
        self.field_path = Some(field_path.into().into_boxed_str());
        self
    }

    /// Adds a safe subject kind and canonical subject ID.
    #[must_use]
    pub fn with_subject(
        mut self,
        subject_kind: impl Into<String>,
        subject_id: impl Into<String>,
    ) -> Self {
        self.subject_kind = Some(subject_kind.into().into_boxed_str());
        self.subject_id = Some(subject_id.into().into_boxed_str());
        self
    }

    /// Adds a bounded structured reason argument.
    #[must_use]
    pub fn with_argument(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut arguments = self.reason_arguments.into_vec();
        arguments.push(ErrorArgument::new(name, value));
        arguments.sort();
        arguments.dedup();
        self.reason_arguments = arguments.into_boxed_slice();
        self
    }

    /// Adds a nested stable cause code.
    #[must_use]
    pub fn with_cause(mut self, code: CoreErrorCode) -> Self {
        let mut causes = self.cause_codes.into_vec();
        causes.push(code);
        causes.sort_unstable();
        causes.dedup();
        self.cause_codes = causes.into_boxed_slice();
        self
    }

    /// Stable code.
    #[must_use]
    pub const fn code(&self) -> CoreErrorCode {
        self.code
    }

    /// Broad category.
    #[must_use]
    pub const fn category(&self) -> ErrorCategory {
        self.category
    }

    /// Operation that rejected the input or invariant.
    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    /// Safe field path, when one is known.
    #[must_use]
    pub fn field_path(&self) -> Option<&str> {
        self.field_path.as_deref()
    }

    /// Retry classification.
    #[must_use]
    pub const fn retry_class(&self) -> RetryClass {
        self.retry_class
    }
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} in {}{}",
            self.code,
            self.operation_id,
            self.field_path
                .as_deref()
                .map_or_else(String::new, |path| format!(" at {path}"))
        )
    }
}

impl Error for CoreError {}

/// Builds a common invalid-input error.
#[must_use]
pub(crate) fn validation_error(
    operation: &'static str,
    code: CoreErrorCode,
    field: &'static str,
) -> CoreError {
    CoreError::new(
        code,
        ErrorCategory::Validation,
        operation,
        RetryClass::AfterInputChange,
    )
    .at_field(field)
}

/// Builds a common mismatch error.
#[must_use]
pub(crate) fn mismatch_error(
    operation: &'static str,
    code: CoreErrorCode,
    field: &'static str,
) -> CoreError {
    CoreError::new(
        code,
        ErrorCategory::Mismatch,
        operation,
        RetryClass::AfterInputChange,
    )
    .at_field(field)
}

/// Builds a common unsupported-input error.
#[must_use]
pub(crate) fn unsupported_error(
    operation: &'static str,
    code: CoreErrorCode,
    field: &'static str,
) -> CoreError {
    CoreError::new(
        code,
        ErrorCategory::Unsupported,
        operation,
        RetryClass::AfterInputChange,
    )
    .at_field(field)
}
