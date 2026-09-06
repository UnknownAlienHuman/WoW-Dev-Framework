//! Consumers retain their own diagnostic codes. Never treat an empty report as
//! proof that negative cases were checked or that the artifact was understood.
use super::{Result, fixture::NEGATIVES, process::Consumer};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
pub fn normalize(kind: Consumer, bytes: &[u8], input: &Path) -> Result<Vec<Value>> {
    // Compare identities in the same canonical representation; CLI paths may differ.
    let input = input.canonicalize()?;
    let raw: Value = serde_json::from_slice(bytes)?;
    let entries: Vec<(&str, &Value)> = match kind {
        Consumer::Emmy => raw
            .as_array()
            .ok_or("expected Emmy file array")?
            .iter()
            .map(|v| {
                Ok((
                    v["file"].as_str().ok_or("missing Emmy file")?,
                    &v["diagnostics"],
                ))
            })
            .collect::<Result<_>>()?,
        Consumer::LuaLs => raw
            .as_object()
            .ok_or("expected LuaLS URI map")?
            .iter()
            .map(|(k, v)| (k.as_str(), v))
            .collect(),
    };
    let mut records = Vec::new();
    let mut seen = BTreeSet::new();
    for (file, diagnostics) in entries {
        let path = source_path(kind, file)?;
        let path = path.canonicalize()?;
        let relative = path
            .strip_prefix(&input)
            .map_err(|_| "diagnostic outside probe input")?;
        let name = relative
            .to_str()
            .ok_or("invalid diagnostic filename")?
            .replace('\\', "/");
        if !seen.insert(name.clone()) {
            return Err("duplicate diagnostic file".into());
        }
        let source = fs::read_to_string(&path)?;
        for d in diagnostics
            .as_array()
            .ok_or("diagnostics must be an array")?
        {
            if records.len() >= 4096 {
                return Err("diagnostic count exceeded".into());
            }
            let code = d["code"].as_str().ok_or("missing diagnostic code")?;
            let severity = d["severity"]
                .as_u64()
                .ok_or("missing diagnostic severity")?;
            if !(1..=4).contains(&severity) || code.len() > 128 {
                return Err("invalid diagnostic class".into());
            }
            let range = &d["range"];
            let start = position(&range["start"], &source)?;
            let end = position(&range["end"], &source)?;
            if start > end {
                return Err("reversed diagnostic span".into());
            }
            records.push(json!({"file":name,"code":code,"severity":severity,"range":range}));
        }
    }
    records.sort_by_key(Value::to_string);
    Ok(records)
}
fn source_path(kind: Consumer, file: &str) -> Result<PathBuf> {
    if matches!(kind, Consumer::Emmy) {
        return Ok(PathBuf::from(file));
    }
    let encoded = file
        .strip_prefix("file://")
        .ok_or("non-file diagnostic URI")?;
    if !encoded.starts_with('/') {
        return Err("diagnostic URI authority not allowed".into());
    }
    let mut bytes = Vec::new();
    let mut input = encoded.bytes();
    while let Some(b) = input.next() {
        if b == b'%' {
            let a = char::from(input.next().ok_or("bad URI escape")?)
                .to_digit(16)
                .ok_or("bad URI escape")?;
            let b = char::from(input.next().ok_or("bad URI escape")?)
                .to_digit(16)
                .ok_or("bad URI escape")?;
            bytes.push((a * 16 + b) as u8);
        } else {
            bytes.push(b);
        }
    }
    let path = String::from_utf8(bytes)?;
    #[cfg(windows)]
    let path = if path.as_bytes().get(2) == Some(&b':') {
        path[1..].to_string()
    } else {
        path
    };
    Ok(PathBuf::from(path))
}
fn position(value: &Value, source: &str) -> Result<(u64, u64)> {
    let line = value["line"].as_u64().ok_or("invalid diagnostic line")?;
    let column = value["character"]
        .as_u64()
        .ok_or("invalid diagnostic column")?;
    let text = source
        .split('\n')
        .nth(usize::try_from(line)?)
        .ok_or("line outside file")?
        .trim_end_matches('\r');
    if column > text.encode_utf16().count() as u64 {
        return Err("column outside source line".into());
    }
    Ok((line, column))
}
pub fn assert_behavior(records: &[Value], status: i32) -> Result<()> {
    if status != 1 {
        return Err("negative consumer run must report diagnostics with exit 1".into());
    }
    if records.iter().any(|d| {
        d["file"] == "positive.lua"
            || d["file"]
                .as_str()
                .is_some_and(|f| f.starts_with("api-") || f.starts_with("values-"))
    }) {
        return Err("valid generated library/positive fixture has diagnostics".into());
    }
    for (name, _, line) in NEGATIVES {
        let expected = match *name {
            "missing.lua" => "undefined-field",
            "global.lua" => "undefined-global",
            "return.lua" | "multiple.lua" | "structure.lua" | "array.lua" => "assign-type-mismatch",
            _ => "param-type-mismatch",
        };
        if !records.iter().any(|d| {
            d["file"] == *name
                && d["code"] == expected
                && d["range"]["start"]["line"] == *line
                && d["severity"].as_u64().is_some_and(|s| s <= 2)
        }) {
            return Err(format!("missing required {expected} at {name}:{}", line + 1).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_both_root_and_diagnostic_before_containment() -> Result<()> {
        let workspace = super::super::Workspace::new()?;
        let input = workspace.path.join("input");
        super::super::fixture::prepare(&input)?;
        let raw = serde_json::to_vec(&json!([{
            "file": input.join("argument.lua"),
            "diagnostics": [{"code":"param-type-mismatch","severity":2,
                "range":{"start":{"line":0,"character":0},
                    "end":{"line":0,"character":1}}}]
        }]))?;
        for root in [input.clone(), input.join("..").join("input")] {
            let records = normalize(Consumer::Emmy, &raw, &root)?;
            assert_eq!(records.len(), 1);
            assert_eq!(records[0]["file"], "argument.lua");
        }
        Ok(())
    }
}
