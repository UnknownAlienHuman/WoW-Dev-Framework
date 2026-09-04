from __future__ import annotations

from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return source.replace(old, new, 1)


def move_trailing_items_before_tests(relative: str, suffix_marker: str) -> None:
    path = Path(relative)
    source = path.read_text(encoding="utf-8")
    test_marker = "#[cfg(test)]\nmod tests {"
    test_start = source.find(test_marker)
    if test_start < 0:
        raise SystemExit(f"test module marker not found in {relative}")
    suffix_start = source.find(suffix_marker, test_start)
    if suffix_start < 0:
        raise SystemExit(f"compatibility suffix marker not found in {relative}")

    prefix = source[:test_start].rstrip()
    tests = source[test_start:suffix_start].rstrip()
    suffix = source[suffix_start:].rstrip()
    path.write_text(f"{prefix}\n\n{suffix}\n\n{tests}\n", encoding="utf-8")


coverage = Path("crates/wow-core/src/coverage.rs")
source = coverage.read_text(encoding="utf-8")
for old, new, label in (
    (
        "use crate::evidence::{ConflictAffectedRef, ConflictRecord};",
        "use crate::evidence::ConflictRecord;",
        "coverage unused import",
    ),
    (
        "let mut applicable_status = None;",
        "let mut applicable_status: Option<CoverageStatus> = None;",
        "coverage inferred option",
    ),
    (
        "NotEvaluated(NotEvaluatedRecord),",
        "NotEvaluated(Box<NotEvaluatedRecord>),",
        "coverage large enum variant",
    ),
    (
        "Ok(CapabilityAvailability::NotEvaluated(\n        NotEvaluatedRecord::new(",
        "Ok(CapabilityAvailability::NotEvaluated(Box::new(\n        NotEvaluatedRecord::new(",
        "coverage boxed constructor open",
    ),
    (
        "        )?,\n    ))\n}\n\n/// Outcome of a conservative negative-authority evaluation.",
        "        )?,\n    )))\n}\n\n/// Outcome of a conservative negative-authority evaluation.",
        "coverage boxed constructor close",
    ),
):
    source = replace_once(source, old, new, label)
coverage.write_text(source, encoding="utf-8")

move_trailing_items_before_tests(
    "crates/wow-core/src/digest.rs",
    "/// Parses one field-purpose-specific SHA-256 digest.",
)
move_trailing_items_before_tests(
    "crates/wow-core/src/generation.rs",
    "/// Returns the validated identity of one complete generation context.",
)
move_trailing_items_before_tests(
    "crates/wow-core/src/ids.rs",
    "/// Parses a profile identifier while preserving canonicalization information.",
)
move_trailing_items_before_tests(
    "crates/wow-core/src/profile.rs",
    "/// Operation wrapper for validating a structured profile identity.",
)
move_trailing_items_before_tests(
    "crates/wow-core/src/source.rs",
    "/// Normalizes one source path while retaining canonicalization information.",
)

error_path = Path("crates/wow-core/src/error.rs")
source = error_path.read_text(encoding="utf-8")
old_argument_block = '''/// Safe structured argument attached to an error.
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
'''
new_argument_block = '''/// Closed scalar kind for safe structured error arguments.
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
'''
source = replace_once(source, old_argument_block, new_argument_block, "typed error argument")
source = replace_once(
    source,
    "    reason_arguments: Box<[ErrorArgument]>,\n    retry_class: RetryClass,\n    cause_codes: Box<[CoreErrorCode]>,",
    "    reason_arguments: Box<[ErrorArgument]>,\n    retry_class: RetryClass,\n    #[serde(default, skip_serializing_if = \"slice_is_empty\")]\n    cause_codes: Box<[CoreErrorCode]>,",
    "optional cause codes",
)
source = replace_once(
    source,
    '''    /// Adds a bounded structured reason argument.
    #[must_use]
    pub fn with_argument(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let mut arguments = self.reason_arguments.into_vec();
        arguments.push(ErrorArgument::new(name, value));
        arguments.sort();
        arguments.dedup();
        self.reason_arguments = arguments.into_boxed_slice();
        self
    }
''',
    '''    /// Adds a bounded text reason argument.
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
''',
    "typed with_argument",
)
validate_method_marker = '''    /// Retry classification.
    #[must_use]
    pub const fn retry_class(&self) -> RetryClass {
        self.retry_class
    }
'''
validate_method_replacement = validate_method_marker + '''
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
'''
source = replace_once(source, validate_method_marker, validate_method_replacement, "CoreError validation")
helpers_marker = "impl fmt::Display for CoreError {"
helpers = '''fn validate_error_argument_name(name: &str) -> CoreResult<()> {
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
                || (value.as_bytes().first().is_some_and(|byte| matches!(byte, b'1'..=b'9'))
                    && value.bytes().all(|byte| byte.is_ascii_digit())
                    && value.parse::<u64>().is_ok())
        }
        ErrorArgumentKind::Boolean => matches!(value, "true" | "false"),
        ErrorArgumentKind::Identifier => !value.chars().any(char::is_whitespace),
        ErrorArgumentKind::Path => {
            !value.starts_with('/')
                && !value.starts_with('\\')
                && !value.as_bytes().get(1).is_some_and(|byte| *byte == b':')
                && !value.replace('\\', "/").split('/').any(|part| part == "..")
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

'''
source = replace_once(source, helpers_marker, helpers + helpers_marker, "error validation helpers")
error_path.write_text(source, encoding="utf-8")

envelope = Path("crates/wow-core/src/envelope.rs")
source = envelope.read_text(encoding="utf-8")
source = replace_once(
    source,
    '''    ) -> CoreResult<Self> {
        let zero = ContentDigest::<CanonicalResult>::from_bytes([0; 32]);
''',
    '''    ) -> CoreResult<Self> {
        error.validate()?;
        let zero = ContentDigest::<CanonicalResult>::from_bytes([0; 32]);
''',
    "error envelope finalize validation",
)
source = replace_once(
    source,
    '''        if error_envelope_digest(self)? != self.canonical_digest {
''',
    '''        self.error.validate()?;
        if error_envelope_digest(self)? != self.canonical_digest {
''',
    "error envelope read validation",
)
envelope.write_text(source, encoding="utf-8")

lib = Path("crates/wow-core/src/lib.rs")
source = lib.read_text(encoding="utf-8")
source = replace_once(
    source,
    "pub use error::{CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass};",
    "pub use error::{\n    CoreError, CoreErrorCode, CoreResult, ErrorArgument, ErrorArgumentKind, ErrorCategory,\n    RetryClass,\n};",
    "error argument re-export",
)
lib.write_text(source, encoding="utf-8")

semantics = Path("crates/wow-core/tests/e0_result_semantics.rs")
source = semantics.read_text(encoding="utf-8")
for old, new, label in (
    (
        'let error = validate_message_arguments(&[zeta, alpha]).expect_err("order must be rejected");',
        'let error = require_error(\n        validate_message_arguments(&[zeta, alpha]),\n        "order must be rejected",\n    )?;',
        "message order error",
    ),
    (
        'let float = MessageArgument::new("count", MessageArgumentKind::Integer, "1.5", true)\n        .expect_err("floating-point message argument must be rejected");',
        'let float = require_error(\n        MessageArgument::new("count", MessageArgumentKind::Integer, "1.5", true),\n        "floating-point message argument must be rejected",\n    )?;',
        "float message error",
    ),
    (
        'let error = validate_budget(&invalid).expect_err("zero limit must be rejected");',
        'let error = require_error(validate_budget(&invalid), "zero limit must be rejected")?;',
        "zero budget error",
    ),
    (
        'let error = accumulate_budget_usage(left, right).expect_err("usage overflow must fail");',
        'let error = require_error(\n        accumulate_budget_usage(left, right),\n        "usage overflow must fail",\n    )?;',
        "budget overflow error",
    ),
    (
        '    )\n    .expect_err("known and unknown omission counts are mutually exclusive");',
        '    ),\n        "known and unknown omission counts are mutually exclusive",\n    )?;',
        "truncation error close",
    ),
    (
        'let invalid = TruncationEntry::new(\n',
        'let invalid = require_error(\n        TruncationEntry::new(\n',
        "truncation error open",
    ),
    (
        'let future = validate_schema_version(&schema, &"0.3.0".parse()?, &schema, &supported)\n        .expect_err("future minor version must be rejected");',
        'let future = require_error(\n        validate_schema_version(&schema, &"0.3.0".parse()?, &schema, &supported),\n        "future minor version must be rejected",\n    )?;',
        "future schema error",
    ),
    (
        'let error = envelope\n        .validate()\n        .expect_err("tampered canonical digest must fail");',
        'let error = require_error(\n        envelope.validate(),\n        "tampered canonical digest must fail",\n    )?;',
        "digest tamper error",
    ),
    (
        'let error = envelope\n        .validate()\n        .expect_err("complete must not hide NotEvaluated");',
        'let error = require_error(\n        envelope.validate(),\n        "complete must not hide NotEvaluated",\n    )?;',
        "status error",
    ),
):
    source = replace_once(source, old, new, label)
helper_marker = "const fn valid_limits() -> BudgetLimits {"
helper = '''fn require_error<T>(
    result: Result<T, wow_core::CoreError>,
    message: &'static str,
) -> Result<wow_core::CoreError, Box<dyn Error>> {
    match result {
        Ok(_) => Err(std::io::Error::other(message).into()),
        Err(error) => Ok(error),
    }
}

'''
source = replace_once(source, helper_marker, helper + helper_marker, "test error helper")
semantics.write_text(source, encoding="utf-8")

examples = Path("crates/wow-core/tests/e0_examples.rs")
source = examples.read_text(encoding="utf-8")
function_marker = "fn validate_result(source: &str)"
function_start = source.find(function_marker)
if function_start < 0:
    raise SystemExit("validate_result function was not found")
start_marker = "    let bytes = envelope.canonical_bytes()?;\n"
end_marker = "    let reparsed: E0CheckResultEnvelope"
start = source.find(start_marker, function_start)
if start < 0:
    raise SystemExit("validate_result canonical bytes marker was not found")
start += len(start_marker)
end = source.find(end_marker, start)
if end < 0:
    raise SystemExit("validate_result reparsing marker was not found")
source = source[:start] + "    assert_eq!(bytes, envelope.canonical_bytes()?);\n" + source[end:]
examples.write_text(source, encoding="utf-8")

authority = Path("crates/wow-core/tests/e0_authority_security.rs")
source = authority.read_text(encoding="utf-8")
old_path_assertion = '''    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::PathEscape);
    Ok(())
}

#[test]
fn error_reason_argument_cannot_be_a_credential_payload'''
new_path_assertion = '''    let error = match serde_json::from_value::<E0CheckResultEnvelope>(value) {
        Ok(_) => {
            return Err(std::io::Error::other(
                "path escape unexpectedly deserialized into a typed envelope",
            )
            .into());
        }
        Err(error) => error,
    };
    assert!(error.to_string().contains("PathEscape"));
    Ok(())
}

#[test]
fn error_reason_argument_cannot_be_a_credential_payload'''
source = replace_once(source, old_path_assertion, new_path_assertion, "path escape early rejection")
authority.write_text(source, encoding="utf-8")
