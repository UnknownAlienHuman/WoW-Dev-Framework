//! Artifact consistency for the explicit literal bridge sidecar, not module trust.
use crate::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
fn object_keys(value: &Value, keys: &[&str]) -> Result<()> {
    let object = value.as_object().ok_or("expected literal report object")?;
    if object.len() != keys.len() || !keys.iter().all(|key| object.contains_key(*key)) {
        return Err("unknown or missing literal report field".into());
    }
    Ok(())
}
fn string(value: &Value) -> Result<&str> {
    value
        .as_str()
        .ok_or_else(|| "expected literal report string".into())
}
fn digest(value: &Value) -> Result<&str> {
    let text = string(value)?;
    let hash = text
        .strip_prefix("sha256:")
        .ok_or("invalid literal digest")?;
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid literal digest".into());
    }
    Ok(text)
}
pub fn verify(library: &Value, expected_module: Option<&str>) -> Result<()> {
    if library["schema"] != "wow-native-annotation-library/6" {
        if library.get("literal_execution").is_some() || expected_module.is_some() {
            return Err("missing required or unexpected literal execution report".into());
        }
        return Ok(());
    }
    let trace = &library["literal_execution"];
    object_keys(trace, &["schema", "module", "calls", "artifacts"])?;
    if trace["schema"] != "wow-literal-execution/1" {
        return Err("unsupported literal execution report".into());
    }
    object_keys(&trace["module"], &["sha256", "epoch"])?;
    let module = digest(&trace["module"]["sha256"])?;
    if expected_module.is_some_and(|expected| expected != module) {
        return Err("literal module differs from caller selection".into());
    }
    trace["module"]["epoch"]
        .as_u64()
        .ok_or("invalid selection epoch")?;
    let calls = trace["calls"].as_array().ok_or("missing literal calls")?;
    if calls.len() > 8192 {
        return Err("literal call limit".into());
    }
    let mut last = BTreeMap::new();
    for (ordinal, call) in calls.iter().enumerate() {
        object_keys(call, &["ordinal", "operation", "request_sha256", "result"])?;
        if call["ordinal"].as_u64() != Some(ordinal as u64) {
            return Err("invalid literal call order".into());
        }
        let operation = string(&call["operation"])?;
        if !matches!(operation, "enums" | "events" | "cvars") {
            return Err("unknown literal operation".into());
        }
        digest(&call["request_sha256"])?;
        let result = call["result"].as_object().ok_or("invalid literal result")?;
        if result.len() != 1 {
            return Err("ambiguous literal result".into());
        }
        if let Some(hash) = result.get("Ok") {
            digest(hash)?;
        } else if let Some(error) = result.get("Err") {
            if !matches!(
                string(error)?,
                "InvalidIdentifier"
                    | "UnsafeDocumentation"
                    | "DuplicateName"
                    | "InputLimit"
                    | "OutputLimit"
                    | "UnsupportedLiteral"
            ) {
                return Err("fatal bridge error cannot have a published artifact".into());
            }
            if library["projection"] != "partial" {
                return Err("literal domain failure hidden by clean projection".into());
            }
        } else {
            return Err("invalid literal result tag".into());
        }
        last.insert(operation, ordinal);
    }
    let files = library["files"]
        .as_array()
        .ok_or("missing annotation files")?;
    let mut literals = BTreeMap::new();
    for file in files {
        let path = string(&file["path"])?;
        let operation = if path.starts_with("values-") {
            "enums"
        } else if path.starts_with("events-") {
            "events"
        } else {
            continue;
        };
        if literals
            .insert(path, (operation, digest(&file["sha256"])?))
            .is_some()
        {
            return Err("duplicate literal file".into());
        }
    }
    let artifacts = trace["artifacts"]
        .as_array()
        .ok_or("missing literal artifact bindings")?;
    let mut seen = BTreeSet::new();
    for artifact in artifacts {
        object_keys(artifact, &["path", "call_ordinal"])?;
        let path = string(&artifact["path"])?;
        if !seen.insert(path) {
            return Err("duplicate literal artifact binding".into());
        }
        let (operation, hash) = literals
            .remove(path)
            .ok_or("binding outside literal inventory")?;
        let ordinal = usize::try_from(
            artifact["call_ordinal"]
                .as_u64()
                .ok_or("invalid literal ordinal")?,
        )?;
        let call = calls.get(ordinal).ok_or("literal ordinal outside trace")?;
        if last.get(operation) != Some(&ordinal)
            || call["operation"] != operation
            || call["result"]["Ok"] != hash
        {
            return Err("literal output is not the final selected-module result".into());
        }
    }
    if !literals.is_empty() {
        return Err("unbound literal output".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests;
