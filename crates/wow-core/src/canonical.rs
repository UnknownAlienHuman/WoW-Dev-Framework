use serde::Serialize;
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};

use crate::error::{CoreError, CoreErrorCode, CoreResult, ErrorCategory, RetryClass};

/// E0-A canonical JSON profile identifier.
pub const CANONICALIZATION_VERSION: &str = "wow-core-json/e0-1";

#[derive(Serialize)]
struct DomainMaterial<'a, T: Serialize + ?Sized> {
    domain: &'a str,
    value: &'a T,
}

/// Encodes a value using the strict deterministic E0 JSON subset.
///
/// Object keys are sorted bytewise, insignificant whitespace is omitted, null
/// and floating-point values are rejected, and non-ASCII UTF-8 is preserved.
pub fn canonical_json_bytes<T: Serialize + ?Sized>(value: &T) -> CoreResult<Vec<u8>> {
    let value = serde_json::to_value(value).map_err(|error| canonical_error(error.to_string()))?;
    let canonical = canonicalize_value(value)?;
    serde_json::to_vec(&canonical).map_err(|error| canonical_error(error.to_string()))
}

/// Encodes a value to canonical UTF-8 JSON text.
pub fn canonical_json_string<T: Serialize + ?Sized>(value: &T) -> CoreResult<String> {
    let bytes = canonical_json_bytes(value)?;
    String::from_utf8(bytes).map_err(|error| canonical_error(error.to_string()))
}

/// Hashes exact canonical domain-separated material with SHA-256.
pub fn domain_separated_digest<T: Serialize + ?Sized>(
    domain: &str,
    value: &T,
) -> CoreResult<[u8; 32]> {
    validate_domain(domain)?;
    let bytes = canonical_json_bytes(&DomainMaterial { domain, value })?;
    let digest = Sha256::digest(bytes);
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    Ok(output)
}

fn canonicalize_value(value: Value) -> CoreResult<Value> {
    match value {
        Value::Null => Err(canonical_error("null is outside the E0 canonical subset")),
        Value::Bool(value) => Ok(Value::Bool(value)),
        Value::Number(number) => canonicalize_number(number),
        Value::String(value) => Ok(Value::String(value)),
        Value::Array(values) => values
            .into_iter()
            .map(canonicalize_value)
            .collect::<CoreResult<Vec<_>>>()
            .map(Value::Array),
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));

            let mut canonical = Map::new();
            for (key, value) in entries {
                if key.is_empty() || key.bytes().any(|byte| byte.is_ascii_control()) {
                    return Err(canonical_error("invalid canonical object key"));
                }
                canonical.insert(key, canonicalize_value(value)?);
            }
            Ok(Value::Object(canonical))
        }
    }
}

fn canonicalize_number(number: Number) -> CoreResult<Value> {
    if number.as_u64().is_some() {
        return Ok(Value::Number(number));
    }

    if number.as_i64().is_some_and(|value| value >= 0) {
        return Ok(Value::Number(number));
    }

    Err(canonical_error(
        "floating-point and negative values are outside the E0 canonical subset",
    ))
}

fn validate_domain(domain: &str) -> CoreResult<()> {
    let valid = !domain.is_empty()
        && domain.is_ascii()
        && !domain.bytes().any(|byte| byte.is_ascii_control())
        && !domain.chars().any(char::is_whitespace);

    if valid {
        Ok(())
    } else {
        Err(canonical_error("invalid hash domain"))
    }
}

fn canonical_error(reason: impl Into<String>) -> CoreError {
    CoreError::new(
        CoreErrorCode::CanonicalizationFailure,
        ErrorCategory::Invariant,
        "canonical_json",
        RetryClass::AfterInputChange,
    )
    .with_argument("reason", reason)
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{canonical_json_string, domain_separated_digest};

    #[derive(Serialize)]
    struct OutOfOrder<'a> {
        zebra: &'a str,
        alpha: &'a str,
    }

    #[test]
    fn canonical_keys_are_sorted() {
        let value = OutOfOrder {
            zebra: "z",
            alpha: "a",
        };
        let actual = canonical_json_string(&value);
        assert_eq!(
            actual.as_deref(),
            Ok(r#"{"alpha":"a","zebra":"z"}"#)
        );
    }

    #[test]
    fn empty_object_domain_vector_matches() {
        let value = serde_json::json!({});
        let digest = domain_separated_digest("wow-core/test/e0-1", &value);
        let expected = hex_to_array(
            "a5b546c8b8a738bf5d7483dd556c3018f02a8fad5b1569836e054eb0273730e2",
        );
        assert_eq!(digest, Ok(expected));
    }

    fn hex_to_array(input: &str) -> [u8; 32] {
        let mut output = [0_u8; 32];
        for (index, chunk) in input.as_bytes().chunks_exact(2).enumerate() {
            let high = hex_nibble(chunk[0]);
            let low = hex_nibble(chunk[1]);
            output[index] = (high << 4) | low;
        }
        output
    }

    const fn hex_nibble(byte: u8) -> u8 {
        match byte {
            b'0'..=b'9' => byte - b'0',
            b'a'..=b'f' => byte - b'a' + 10,
            _ => 0,
        }
    }
}
