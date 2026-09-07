//! Structural alias artifact validation, not another Lua parser or semantic probe.
use super::{Result, list, manifest, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
mod resources;
mod strings;

#[derive(Default)]
pub(super) struct CheckedAliases<'a> {
    pub blocked: bool,
    pub sources: BTreeMap<&'a str, &'a Value>,
}

pub(super) fn verify(library: &Value) -> Result<CheckedAliases<'_>> {
    if library["schema"] == "wow-native-annotation-library/6" && library.get("aliases").is_none() {
        return Ok(CheckedAliases::default());
    }
    if !matches!(
        library["schema"].as_str(),
        Some("wow-native-annotation-library/5" | "wow-native-annotation-library/6")
    ) {
        if library.get("aliases").is_some() {
            return Err("unexpected alias report".into());
        }
        return Ok(CheckedAliases::default());
    }
    let report = &library["aliases"];
    let resources = resources::read(report)?;
    let mut entries = Vec::new();
    let mut source_map = BTreeMap::new();
    for resource in resources {
        source_map.insert(text(resource, "path")?, resource);
        for alias in list(resource, "aliases")? {
            entries.push((resource, alias));
        }
    }
    let outcomes = list(report, "outcomes")?;
    if entries.len() != outcomes.len() {
        return Err("missing alias outcomes".into());
    }
    let mut mapped = BTreeMap::new();
    for file in list(library, "files")? {
        for mapping in list(file, "mappings")? {
            if mapping["source"]["scope"] == "annotation_alias_catalog" {
                let link = &mapping["source"];
                let path = text(link, "path")?;
                let resource = source_map.get(path).ok_or("unknown mapped alias resource")?;
                let span = span_key(&link["span"])?;
                if link["sha256"] != resource["sha256"]
                    || span.0 == span.1
                    || text(resource, "text")?.get(span.0..span.1).is_none()
                {
                    return Err("alias mapping source identity/range mismatch".into());
                }
                let key = (path, span);
                let (start, end) = span_key(&mapping["generated"])?;
                let fragment = text(file, "text")?
                    .get(start..end)
                    .ok_or("bad alias output range")?;
                if mapped.insert(key, fragment).is_some() {
                    return Err("duplicate alias mapping".into());
                }
            }
        }
    }
    let mut blocked = false;
    let mut emitted = BTreeSet::new();
    let mut observed_spans = BTreeSet::new();
    for (index, ((resource, alias), outcome)) in entries.iter().zip(outcomes).enumerate() {
        let raw = text(resource, "text")?;
        let span = span_key(&alias["span"])?;
        let key = (text(resource, "path")?, span);
        if raw.get(span.0..span.1).is_none()
            || span.0 == span.1
            || !observed_spans.insert(key)
            || outcome["ordinal"] != json!(index)
            || outcome["name"] != alias["name"]
        {
            return Err("alias source/outcome mismatch".into());
        }
        let literal = alias.get("string_values").map(strings::lower).transpose()?;
        if literal.is_some() && alias.get("terms") != Some(&Value::Null) {
            return Err("literal and named alias terms overlap".into());
        }
        let syntax_error = alias["syntax_error"]
            .as_bool()
            .ok_or("missing alias syntax status")?;
        let status = text(outcome, "status")?;
        if (status == "emitted" && syntax_error)
            || (status == "alias_syntax_error" && !syntax_error)
        {
            return Err("inconsistent alias syntax outcome".into());
        }
        if status == "emitted" {
            let name = text(alias, "name")?;
            if !emitted.insert(name) {
                return Err("invalid emitted alias".into());
            }
            let lowered = if let Some(literal) = literal {
                literal
            } else {
                lower_terms(list(alias, "terms")?)?
            };
            let expected = format!("---@alias {name} {lowered}");
            if mapped.remove(&key) != Some(expected.as_str()) {
                return Err("missing or changed emitted alias declaration".into());
            }
        } else {
            if !matches!(
                status,
                "invalid_alias_name"
                    | "alias_syntax_error"
                    | "duplicate_alias"
                    | "source_name_conflict"
                    | "unsupported_alias_type"
                    | "unresolved_alias_target"
                    | "unresolved_or_cyclic_alias_dependency"
            ) {
                return Err("unknown alias outcome".into());
            }
            blocked = true;
            if mapped.contains_key(&key) {
                return Err("blocked alias has generated output".into());
            }
            if !list(library, "issues")?.iter().any(|issue| {
                issue["code"] == status
                    && issue["source"]["scope"] == "annotation_alias_catalog"
                    && issue["source"]["path"] == resource["path"]
                    && issue["source"]["sha256"] == resource["sha256"]
                    && issue["source"]["span"] == alias["span"]
            }) {
                return Err("missing blocked alias issue".into());
            }
        }
    }
    if !mapped.is_empty() {
        return Err("unaccounted alias output".into());
    }
    Ok(CheckedAliases {
        blocked,
        sources: source_map,
    })
}

fn lower_terms(terms: &[Value]) -> Result<String> {
    if terms.is_empty() || terms.len() > 16 {
        return Err("invalid emitted alias".into());
    }
    Ok(terms
        .iter()
        .map(|term| {
            let term = term.as_str().ok_or("invalid alias term")?;
            if term.is_empty()
                || term
                    .chars()
                    .any(|c| c.is_control() || c.is_whitespace() || c == '|')
            {
                return Err("invalid alias term");
            }
            Ok(match term {
                "bool" => "boolean",
                "cstring" => "string",
                "luaIndex" => "number",
                other => other,
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("|"))
}

fn span_key(span: &Value) -> Result<(usize, usize)> {
    let start = usize::try_from(span["start"].as_u64().ok_or("invalid alias span")?)?;
    let end = usize::try_from(span["end"].as_u64().ok_or("invalid alias span")?)?;
    if start > end {
        return Err("reversed alias span".into());
    }
    Ok((start, end))
}
