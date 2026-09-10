//! Exact inert globals projected from external `CreateColor` assignments.
use super::{Result, list, span_key, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Key<'a> = (&'a str, (usize, usize));

pub(super) fn verify<'a>(
    library: &'a Value,
    sources: &BTreeMap<&'a str, &'a Value>,
    mapped: &mut BTreeMap<Key<'a>, &'a str>,
    emitted: &mut BTreeSet<&'a str>,
) -> Result<bool> {
    let report = &library["aliases"];
    if report["schema"] != "wow-native-alias-projection/9" {
        if report.get("global_color_outcomes").is_some()
            || report.get("unresolved_global_color_types").is_some()
        {
            return Err("unexpected global color projection field".into());
        }
        return Ok(false);
    }
    let mut entries = Vec::new();
    for (&path, &resource) in sources {
        if resource.get("global_colors").is_some() {
            for color in list(resource, "global_colors")? {
                entries.push((path, resource, color));
            }
        }
    }
    let outcomes = list(report, "global_color_outcomes")?;
    if entries.is_empty() || entries.len() != outcomes.len() {
        return Err("missing external global color outcomes".into());
    }
    let mut blocked = false;
    let mut observed = BTreeSet::new();
    let mut emitted_colors = BTreeSet::new();
    for (ordinal, ((path, resource, color), outcome)) in
        entries.into_iter().zip(outcomes).enumerate()
    {
        let name = text(color, "name")?;
        let span = span_key(&color["span"])?;
        let raw = text(resource, "text")?;
        let fragment = raw
            .get(span.0..span.1)
            .ok_or("invalid external global color source range")?;
        if span.0 == span.1
            || !observed.insert((path, span))
            || outcome["ordinal"] != json!(ordinal)
            || outcome["name"] != name
        {
            return Err("global color source/outcome mismatch".into());
        }
        verify_components(color, fragment, name)?;
        let status = text(outcome, "status")?;
        let valid = valid_identifier(name);
        if status == "emitted" {
            if !valid || !emitted.insert(name) {
                return Err("invalid emitted global color".into());
            }
            let expected = format!("---@type colorRGBA\n{name} = nil");
            if mapped.remove(&(path, span)) != Some(expected.as_str()) {
                return Err("missing or changed global color declaration".into());
            }
            emitted_colors.insert((path, span));
        } else {
            if !matches!(
                status,
                "invalid_global_color_name" | "duplicate_global_color" | "source_name_conflict"
            ) || (status == "invalid_global_color_name" && valid)
                || mapped.contains_key(&(path, span))
            {
                return Err("invalid blocked global color".into());
            }
            require_issue(library, status, resource, &color["span"])?;
            blocked = true;
        }
    }
    let mut unresolved = BTreeSet::new();
    if report.get("unresolved_global_color_types").is_some() {
        let types = list(report, "unresolved_global_color_types")?;
        if types.is_empty() || types.len() > 65_536 {
            return Err("invalid unresolved global color inventory".into());
        }
        for link in types {
            let path = text(link, "path")?;
            let key = (path, span_key(&link["span"])?);
            let resource = sources.get(path).ok_or("unknown global color resource")?;
            if link["scope"] != "annotation_alias_catalog"
                || link["sha256"] != resource["sha256"]
                || !emitted_colors.contains(&key)
                || !unresolved.insert(key)
            {
                return Err("invalid unresolved global color link".into());
            }
            require_issue(
                library,
                "unresolved_global_color_type",
                resource,
                &link["span"],
            )?;
        }
        blocked = true;
    }
    for issue in list(library, "issues")? {
        if issue["code"] == "unresolved_global_color_type" {
            let source = &issue["source"];
            let path = text(source, "path")?;
            let resource = sources
                .get(path)
                .ok_or("unknown global color issue resource")?;
            if source["scope"] != "annotation_alias_catalog"
                || source["sha256"] != resource["sha256"]
                || !unresolved.contains(&(path, span_key(&source["span"])?))
            {
                return Err("unreported unresolved global color type".into());
            }
        }
    }
    Ok(blocked)
}

fn verify_components(color: &Value, source: &str, name: &str) -> Result<()> {
    let components = list(color, "components")?;
    if components.len() != 4 {
        return Err("invalid global color component count".into());
    }
    let call = source
        .find("CreateColor")
        .ok_or("missing global color source call")?;
    if !source[..call].contains(name) {
        return Err("global color source name mismatch".into());
    }
    let mut tail = source
        .get(call + "CreateColor".len()..)
        .ok_or("invalid global color source call")?;
    for component in components {
        let component = component.as_str().ok_or("invalid global color component")?;
        if component.is_empty()
            || component.len() > 64
            || !component.bytes().all(|byte| {
                byte.is_ascii_hexdigit()
                    || matches!(byte, b'.' | b'x' | b'X' | b'p' | b'P' | b'+' | b'-')
            })
        {
            return Err("invalid global color component".into());
        }
        let start = tail
            .find(component)
            .ok_or("global color component/source mismatch")?;
        tail = tail
            .get(start + component.len()..)
            .ok_or("invalid global color component range")?;
    }
    Ok(())
}

fn valid_identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 1024 || keyword(name) {
        return false;
    }
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn keyword(name: &str) -> bool {
    matches!(
        name,
        "and"
            | "break"
            | "do"
            | "else"
            | "elseif"
            | "end"
            | "false"
            | "for"
            | "function"
            | "goto"
            | "if"
            | "in"
            | "local"
            | "nil"
            | "not"
            | "or"
            | "repeat"
            | "return"
            | "then"
            | "true"
            | "until"
            | "while"
    )
}

fn require_issue(library: &Value, code: &str, resource: &Value, span: &Value) -> Result<()> {
    if !list(library, "issues")?.iter().any(|issue| {
        issue["code"] == code
            && issue["source"]["scope"] == "annotation_alias_catalog"
            && issue["source"]["path"] == resource["path"]
            && issue["source"]["sha256"] == resource["sha256"]
            && issue["source"]["span"] == *span
    }) {
        return Err("missing external global color issue".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        let raw = "SYNTHETIC_COLOR = CreateColor(0.000, 0.500, 1.000, 1.000)\n";
        let source_span = json!({"start":0,"end":raw.len()-1});
        let output = "---@type colorRGBA\nSYNTHETIC_COLOR = nil";
        let output_span = json!({"start":0,"end":output.len()});
        let hash = format!("sha256:{}", crate::manifest::digest(raw.as_bytes()));
        let source = json!({
            "scope":"annotation_alias_catalog",
            "path":"GlobalColors.lua",
            "sha256":hash,
            "span":source_span
        });
        json!({
            "schema":"wow-native-annotation-library/5",
            "issues":[],
            "aliases":{
                "schema":"wow-native-alias-projection/9",
                "authority":"external_annotation_overlay",
                "source":{
                    "schema":"wow-native-alias-resource/7",
                    "revision":"a".repeat(40),
                    "path":"GlobalColors.lua",
                    "sha256":hash,
                    "source_bytes":raw.len(),
                    "text":raw,
                    "aliases":[],
                    "global_colors":[{
                        "name":"SYNTHETIC_COLOR",
                        "components":["0.000","0.500","1.000","1.000"],
                        "span":source_span
                    }]
                },
                "outcomes":[],
                "global_color_outcomes":[{
                    "ordinal":0,"name":"SYNTHETIC_COLOR","status":"emitted"
                }]
            },
            "files":[{"text":output,"mappings":[{
                "granularity":"declaration","generated":output_span,"source":source
            }]}]
        })
    }

    #[test]
    fn exact_inert_global_bytes_and_resource_evidence_are_required() -> Result<()> {
        let value = fixture();
        assert!(!super::super::verify(&value)?.blocked);
        for (pointer, replacement) in [
            ("/aliases/schema", json!("wow-native-alias-projection/8")),
            (
                "/aliases/source/schema",
                json!("wow-native-alias-resource/6"),
            ),
            ("/aliases/source/global_colors", json!([])),
            ("/aliases/global_color_outcomes", json!([])),
            (
                "/aliases/source/global_colors/0/components/0",
                json!("0.125"),
            ),
            (
                "/files/0/text",
                json!("SYNTHETIC_COLOR = CreateColor(0,0,0,1)"),
            ),
            ("/files/0/mappings", json!([])),
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(pointer).ok_or("mutation target")? = replacement;
            assert!(super::super::verify(&changed).is_err(), "{pointer}");
        }
        Ok(())
    }

    #[test]
    fn unresolved_type_requires_inventory_and_matching_issue() -> Result<()> {
        let mut value = fixture();
        let link = value["files"][0]["mappings"][0]["source"].clone();
        value["aliases"]["unresolved_global_color_types"] = json!([link.clone()]);
        value["issues"] = json!([{
            "code":"unresolved_global_color_type","source":link
        }]);
        assert!(super::super::verify(&value)?.blocked);
        value["issues"] = json!([]);
        assert!(super::super::verify(&value).is_err());
        Ok(())
    }
}
