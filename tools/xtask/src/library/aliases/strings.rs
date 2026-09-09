//! Closed literal wire validation; no resource parsing or dependency resolution.
use crate::Result;
use serde_json::Value;
use std::collections::BTreeSet;

const MAX_VALUES: usize = 512;

pub(super) fn lower(values: &Value) -> Result<String> {
    let values = values.as_array().ok_or("invalid literal alias terms")?;
    if values.is_empty() || values.len() > MAX_VALUES {
        return Err("literal alias count limit".into());
    }
    let mut seen = BTreeSet::new();
    let mut terms = Vec::new();
    for value in values {
        let value = value.as_str().ok_or("non-string literal alias value")?;
        if value.len() > 128
            || !seen.insert(value)
            || !value
                .bytes()
                .all(|b| (b' '..=b'~').contains(&b) && b != b'"' && b != b'\\')
        {
            return Err("unsupported literal alias value".into());
        }
        terms.push(format!("\"{value}\""));
    }
    Ok(terms.join("|"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn literal_values_cannot_become_names_or_directives() -> Result<()> {
        assert_eq!(
            lower(&json!(["A|B", "---@meta", ""]))?,
            "\"A|B\"|\"---@meta\"|\"\""
        );
        for values in [
            json!([]),
            json!(["A", "A"]),
            json!([42]),
            json!(["a\nb"]),
            json!(["a\\b"]),
            json!(["a\"b"]),
            json!(["é"]),
            json!(["x".repeat(129)]),
            Value::Null,
        ] {
            assert!(lower(&values).is_err());
        }
        Ok(())
    }

    #[test]
    fn bounded_large_completion_sets_are_verified_without_an_unbounded_wire() -> Result<()> {
        let values = Value::Array(
            (0..MAX_VALUES)
                .map(|index| Value::String(format!("SYNTHETIC_{index}")))
                .collect(),
        );
        assert_eq!(lower(&values)?.matches('|').count(), MAX_VALUES - 1);
        let mut over = values.as_array().ok_or("values")?.clone();
        over.push(Value::String("OVER_BUDGET".into()));
        assert!(lower(&Value::Array(over)).is_err());
        Ok(())
    }
}
