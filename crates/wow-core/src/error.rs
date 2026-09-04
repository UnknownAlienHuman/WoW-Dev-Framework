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

/// Closed scalar kind for safe structured error arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorArgumentKind {
    Text,
    Integer,
    Boolean,
    Identifier,
    Path,
    Digest,
}

/// Safe structured argument attached to an error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorArgument {
    name: Box<str>,
    kind: ErrorArgumentKind,
    value: Box<str>,
}

impl ErrorArgument {
    /// Creates a bounded text argument.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new_typed(name, ErrorArgumentKind::Text, value)
    }

    /// Creates a bounded argument with an explicit scalar kind.
    pub fn new_typed(
        name: impl Into<String>,
        kind: ErrorArgumentKind,
        value: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into().into_boxed_str(),
            kind,
            value: value.into().into_boxed_str(),
        }
    }

    /// Argument name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Declared scalar kind.
    #[must_use]
    pub const fn kind(&self) -> ErrorArgumentKind {
        self.kind
    }

    /// Canonical safe value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    fn validate(&self) -> CoreResult<()> {
        validate_error_argument_name(self.name())?;
        validate_error_argument_value(self.kind(), self.value())
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
    #[serde(default, skip_serializing_if = "slice_is_empty")]
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

    /// Adds a bounded text reason argument.
    #[must_use]
    pub fn with_argument(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_typed_argument(name, ErrorArgumentKind::Text, value)
    }

    /// Adds a bounded explicitly typed reason argument.
    #[must_use]
    pub fn with_typed_argument(
        mut self,
        name: impl Into<String>,
        kind: ErrorArgumentKind,
        value: impl Into<String>,
    ) -> Self {
        let mut arguments = self.reason_arguments.into_vec();
        arguments.push(ErrorArgument::new_typed(name, kind, value));
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

    /// Validates the stable, non-secret public error boundary.
    pub fn validate(&self) -> CoreResult<()> {
        validate_safe_error_text(&self.operation_id, "error.operation_id", 256)?;
        if let Some(field_path) = &self.field_path {
            validate_safe_error_text(field_path, "error.field_path", 512)?;
        }
        if let Some(subject_kind) = &self.subject_kind {
            validate_safe_error_text(subject_kind, "error.subject_kind", 128)?;
        }
        if let Some(subject_id) = &self.subject_id {
            validate_safe_error_text(subject_id, "error.subject_id", 4_096)?;
        }

        if self.reason_arguments.len() > 64 {
            return Err(validation_error(
                "validate_operation_error",
                CoreErrorCode::BudgetExceeded,
                "error.reason_arguments",
            ));
        }
        for argument in &self.reason_arguments {
            argument.validate()?;
        }
        for pair in self.reason_arguments.windows(2) {
            if pair[0].name() >= pair[1].name() {
                return Err(validation_error(
                    "validate_operation_error",
                    CoreErrorCode::InvalidMessageArgument,
                    "error.reason_arguments",
                ));
            }
        }
        for pair in self.cause_codes.windows(2) {
            if pair[0] >= pair[1] {
                return Err(validation_error(
                    "validate_operation_error",
                    CoreErrorCode::ContractViolation,
                    "error.cause_codes",
                ));
            }
        }
        Ok(())
    }
}

fn validate_error_argument_name(name: &str) -> CoreResult<()> {
    let valid = (1..=63).contains(&name.len())
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        });
    if !valid || is_sensitive_error_key(name) {
        return Err(validation_error(
            "validate_operation_error",
            CoreErrorCode::ContractViolation,
            "error.reason_arguments.name",
        ));
    }
    Ok(())
}

fn validate_error_argument_value(kind: ErrorArgumentKind, value: &str) -> CoreResult<()> {
    validate_safe_error_text(value, "error.reason_arguments.value", 4_096)?;
    let canonical = match kind {
        ErrorArgumentKind::Text => true,
        ErrorArgumentKind::Integer => {
            value == "0"
                || (value
                    .as_bytes()
                    .first()
                    .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value.parse::<u64>().is_ok())
        }
        ErrorArgumentKind::Boolean => matches!(value, "true" | "false"),
        ErrorArgumentKind::Identifier => !value.chars().any(char::is_whitespace),
        ErrorArgumentKind::Path => {
            !value.starts_with('/')
                && value.as_bytes().first().is_none_or(|byte| *byte != 92)
                && !value.as_bytes().get(1).is_some_and(|byte| *byte == b':')
                && !value
                    .split(|character| character == '/' || character == char::from(92_u8))
                    .any(|part| part == "..")
        }
        ErrorArgumentKind::Digest => {
            value.len() == 71
                && value.starts_with("sha256:")
                && value[7..]
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        }
    };
    if !canonical || contains_sensitive_error_value(value) {
        return Err(validation_error(
            "validate_operation_error",
            CoreErrorCode::ContractViolation,
            "error.reason_arguments.value",
        ));
    }
    Ok(())
}

fn validate_safe_error_text(value: &str, field: &'static str, max_bytes: usize) -> CoreResult<()> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.chars().any(char::is_control)
        || contains_sensitive_error_value(value)
    {
        return Err(validation_error(
            "validate_operation_error",
            CoreErrorCode::ContractViolation,
            field,
        ));
    }
    Ok(())
}

fn is_sensitive_error_key(name: &str) -> bool {
    const SENSITIVE: &[&str] = &[
        "access_token",
        "api_key",
        "authorization",
        "bearer",
        "client_secret",
        "cookie",
        "credential",
        "password",
        "passwd",
        "private_key",
        "refresh_token",
        "session_token",
        "signing_key",
    ];
    SENSITIVE.iter().any(|candidate| {
        name == *candidate
            || name
                .strip_suffix(candidate)
                .is_some_and(|prefix| prefix.ends_with('_'))
    })
}

fn contains_sensitive_error_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.starts_with("bearer ")
        || lower.starts_with("basic ")
        || lower.starts_with("ghp_")
        || lower.starts_with("github_pat_")
        || lower.starts_with("sk-")
        || lower.contains("-----begin private key-----")
}

fn slice_is_empty<T>(values: &[T]) -> bool {
    values.is_empty()
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
