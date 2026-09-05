//! Canonical JSON for source-producer interchange, not the E0 core-ID profile.
//!
//! Source documents contain null, signed values, and decimal literals. Keep
//! those values here rather than relaxing `wow-core`'s stricter ID contract.
//! `serde_json/arbitrary_precision` retains number lexemes supplied by source producers;
//! this avoids silently changing decimal/exponent spelling during digest checks.

use serde::Serialize;
use serde_json::{Map, Value};

/// Serialize source wire data with sorted keys, UTF-8, and no extra whitespace.
/// Numeric lexemes are retained; this is not a cross-producer number normalizer.
pub fn canonical_json_bytes<T: Serialize + ?Sized>(value: &T) -> serde_json::Result<Vec<u8>> {
    fn ordered(value: Value) -> Value {
        match value {
            Value::Array(items) => Value::Array(items.into_iter().map(ordered).collect()),
            Value::Object(object) => {
                let mut entries = object.into_iter().collect::<Vec<_>>();
                entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
                let mut sorted = Map::new();
                for (key, value) in entries {
                    sorted.insert(key, ordered(value));
                }
                Value::Object(sorted)
            }
            other => other,
        }
    }
    serde_json::to_vec(&ordered(serde_json::to_value(value)?))
}

#[cfg(test)]
mod tests {
    use super::canonical_json_bytes;
    use serde_json::Value;

    #[test]
    fn preserves_wire_number_lexemes_and_source_nulls() -> serde_json::Result<()> {
        let value: Value = serde_json::from_str(
            r#"{"z":null,"a":[-2,1e-07,1.0,123456789012345678901234567890]}"#,
        )?;
        assert_eq!(
            canonical_json_bytes(&value)?,
            br#"{"a":[-2,1e-07,1.0,123456789012345678901234567890],"z":null}"#
        );
        assert!(wow_core::canonical_json_bytes(&value).is_err());
        Ok(())
    }
}
