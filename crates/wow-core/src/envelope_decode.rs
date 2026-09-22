//! Bounded raw-byte admission for the two concrete E0 envelopes.
//!
//! This preflight is not a replacement JSON parser. It retains member names
//! before Serde can discard duplicates, bounds work before typed allocation,
//! and rejects scalar spellings that a feature-unified decoder could normalize.
//! Serde remains responsible for JSON grammar and the concrete E0 schema.

use std::collections::BTreeSet;

use crate::{
    CoreError, CoreErrorCode, CoreResult, E0CheckResultEnvelope, E0OperationErrorEnvelope,
    ErrorCategory, RetryClass,
};

const OPERATION: &str = "decode_result_envelope";
const MAX_INPUT_BYTES: usize = 64 * 1024 * 1024;
const MAX_NESTING_DEPTH: usize = 64;
const MAX_TOKENS: usize = 1_000_000;
const MAX_STRING_BYTES: usize = 1024 * 1024;

/// Caller-owned limits for decoding E0 JSON, independent of an input's budget.
///
/// Limits must be selected before reading untrusted input. No default or wire
/// deserializer is provided: a document must not choose its own admission policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct E0DecodeLimits {
    max_input_bytes: usize,
    max_nesting_depth: usize,
    max_tokens: usize,
    max_string_bytes: usize,
}

impl E0DecodeLimits {
    /// Creates explicit nonzero limits within the implementation's ceilings.
    ///
    /// Ceilings: 64 MiB input, 64 nested containers, 1,000,000 tokens, and
    /// 1 MiB per raw string interior. Tokens count opening containers, object
    /// keys and scalar values; punctuation and closing delimiters do not count.
    /// String bytes count the original UTF-8 spelling, including escape bytes
    /// but excluding the surrounding quotes, not the decoded string length.
    pub fn new(
        max_input_bytes: usize,
        max_nesting_depth: usize,
        max_tokens: usize,
        max_string_bytes: usize,
    ) -> CoreResult<Self> {
        for (field, value, ceiling) in [
            ("limits.max_input_bytes", max_input_bytes, MAX_INPUT_BYTES),
            (
                "limits.max_nesting_depth",
                max_nesting_depth,
                MAX_NESTING_DEPTH,
            ),
            ("limits.max_tokens", max_tokens, MAX_TOKENS),
            (
                "limits.max_string_bytes",
                max_string_bytes,
                MAX_STRING_BYTES,
            ),
        ] {
            if value == 0 || value > ceiling {
                return Err(error(CoreErrorCode::BudgetInvalid, field));
            }
        }
        Ok(Self {
            max_input_bytes,
            max_nesting_depth,
            max_tokens,
            max_string_bytes,
        })
    }
}

impl E0CheckResultEnvelope {
    /// Decodes bounded, duplicate-free E0 JSON and validates the complete result.
    ///
    /// Accepts whitespace and object-key order, but never repairs collection
    /// order, identities, digests or counts. Raw null, negative/float numbers and
    /// duplicate decoded keys reject before typed deserialization. Serde errors
    /// are deliberately not echoed; semantic validation preserves CoreError codes.
    pub fn from_json_slice(input: &[u8], limits: E0DecodeLimits) -> CoreResult<Self> {
        preflight(input, limits)?;
        let envelope: Self = serde_json::from_slice(input).map_err(|_| invalid_json())?;
        envelope.validate()?;
        Ok(envelope)
    }
}

impl E0OperationErrorEnvelope {
    /// Decodes bounded E0 error JSON and validates schema, safe error and digest.
    ///
    /// Uses the same raw-input policy as the check envelope. The existing Serde
    /// implementation remains a structural decoder, not this admission boundary.
    pub fn from_json_slice(input: &[u8], limits: E0DecodeLimits) -> CoreResult<Self> {
        preflight(input, limits)?;
        let envelope: Self = serde_json::from_slice(input).map_err(|_| invalid_json())?;
        envelope.validate()?;
        Ok(envelope)
    }
}

enum Frame {
    Object(BTreeSet<String>),
    Array,
}

fn preflight(input: &[u8], limits: E0DecodeLimits) -> CoreResult<()> {
    if input.len() > limits.max_input_bytes {
        return Err(error(CoreErrorCode::BudgetExceeded, "input.bytes"));
    }
    // Validate without allocation before converting any token to a String.
    let text = std::str::from_utf8(input).map_err(|_| invalid_json())?;
    let mut frames = Vec::new();
    let mut cursor = 0;
    let mut tokens = 0;
    while let Some(&byte) = input.get(cursor) {
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' | b',' | b':' => cursor += 1,
            b'{' | b'[' => {
                count_token(&mut tokens, limits)?;
                if frames.len() == limits.max_nesting_depth {
                    return Err(error(CoreErrorCode::BudgetExceeded, "input.nesting_depth"));
                }
                frames.push(if byte == b'{' {
                    Frame::Object(BTreeSet::new())
                } else {
                    Frame::Array
                });
                cursor += 1;
            }
            b'}' | b']' => {
                if !matches!(
                    (byte, frames.pop()),
                    (b'}', Some(Frame::Object(_))) | (b']', Some(Frame::Array))
                ) {
                    return Err(invalid_json());
                }
                cursor += 1;
            }
            b'"' => {
                count_token(&mut tokens, limits)?;
                let start = cursor;
                cursor = string_end(input, cursor, limits.max_string_bytes)?;
                // Decode only one bounded token. All strings, not just keys,
                // must have valid escapes and surrogate pairs. The scan never
                // treats quoted brackets, commas or colons as structure.
                let value: String =
                    serde_json::from_str(&text[start..cursor]).map_err(|_| invalid_json())?;
                let mut next = cursor;
                while input.get(next).is_some_and(|byte| json_space(*byte)) {
                    next += 1;
                }
                if input.get(next) == Some(&b':') {
                    let Some(Frame::Object(keys)) = frames.last_mut() else {
                        return Err(invalid_json());
                    };
                    if !keys.insert(value) {
                        return Err(error(CoreErrorCode::DuplicateField, "input.object_key"));
                    }
                }
            }
            _ => {
                count_token(&mut tokens, limits)?;
                let start = cursor;
                while input.get(cursor).is_some_and(|byte| !delimiter(*byte)) {
                    cursor += 1;
                }
                let token = &text[start..cursor];
                match token {
                    "true" | "false" => {}
                    "null" => {
                        return Err(error(
                            CoreErrorCode::CanonicalizationFailure,
                            "input.scalar",
                        ));
                    }
                    _ if byte.is_ascii_digit() || matches!(byte, b'-' | b'+') => {
                        // Compare exact unsigned decimal spelling before any
                        // serde_json feature can turn lexical -0 into integer 0.
                        let value = token.parse::<u64>().map_err(|_| invalid_number())?;
                        if value.to_string() != token {
                            return Err(invalid_number());
                        }
                    }
                    _ => return Err(invalid_json()),
                }
            }
        }
    }
    if !frames.is_empty() {
        return Err(invalid_json());
    }
    Ok(())
}

fn string_end(input: &[u8], start: usize, limit: usize) -> CoreResult<usize> {
    let mut cursor = start + 1;
    loop {
        if cursor - start - 1 > limit {
            return Err(error(CoreErrorCode::BudgetExceeded, "input.string_bytes"));
        }
        match input.get(cursor) {
            Some(b'"') => return Ok(cursor + 1),
            Some(b'\\') => cursor += 2,
            Some(_) => cursor += 1,
            None => return Err(invalid_json()),
        }
    }
}

fn count_token(tokens: &mut usize, limits: E0DecodeLimits) -> CoreResult<()> {
    // The validated ceiling is far below usize::MAX on supported targets.
    *tokens += 1;
    if *tokens > limits.max_tokens {
        return Err(error(CoreErrorCode::BudgetExceeded, "input.tokens"));
    }
    Ok(())
}

fn json_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}

fn delimiter(byte: u8) -> bool {
    json_space(byte) || matches!(byte, b'{' | b'}' | b'[' | b']' | b',' | b':' | b'"')
}

fn invalid_number() -> CoreError {
    error(CoreErrorCode::CanonicalizationFailure, "input.number")
}

fn invalid_json() -> CoreError {
    error(CoreErrorCode::ContractViolation, "input")
}

fn error(code: CoreErrorCode, field: &'static str) -> CoreError {
    CoreError::new(
        code,
        if matches!(
            code,
            CoreErrorCode::BudgetInvalid | CoreErrorCode::BudgetExceeded
        ) {
            ErrorCategory::Budget
        } else {
            ErrorCategory::Validation
        },
        OPERATION,
        RetryClass::AfterInputChange,
    )
    .at_field(field)
}
