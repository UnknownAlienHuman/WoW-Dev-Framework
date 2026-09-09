//! Exact empty namespace bindings from external resources.
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
    let schema = report["schema"].as_str();
    if !matches!(
        schema,
        Some("wow-native-alias-projection/7" | "wow-native-alias-projection/8")
    ) {
        if report.get("namespace_outcomes").is_some() {
            return Err("unexpected namespace projection field".into());
        }
        return Ok(false);
    }
    let mut entries = Vec::new();
    for (&path, &resource) in sources {
        if resource.get("namespaces").is_some() {
            for namespace in list(resource, "namespaces")? {
                entries.push((path, resource, namespace));
            }
        }
    }
    if entries.is_empty() {
        if report.get("namespace_outcomes").is_some() {
            return Err("unexpected namespace projection field".into());
        }
        if schema == Some("wow-native-alias-projection/7") {
            return Err("missing external namespace outcomes".into());
        }
        return Ok(false);
    }
    let outcomes = list(report, "namespace_outcomes")?;
    if outcomes.len() != entries.len() {
        return Err("missing external namespace outcomes".into());
    }
    let mut blocked = false;
    let mut spans = BTreeSet::new();
    for (ordinal, ((path, resource, namespace), outcome)) in
        entries.into_iter().zip(outcomes).enumerate()
    {
        let name = text(namespace, "name")?;
        let span = span_key(&namespace["span"])?;
        let raw = text(resource, "text")?;
        if span.0 == span.1
            || raw.get(span.0..span.1).is_none()
            || !spans.insert((path, span))
            || outcome["ordinal"] != json!(ordinal)
            || outcome["name"] != name
        {
            return Err("namespace source/outcome mismatch".into());
        }
        let valid = valid_name(name);
        let status = text(outcome, "status")?;
        if status == "emitted" {
            if !valid || !emitted.insert(name) {
                return Err("invalid emitted namespace".into());
            }
            let expected = format!("{name} = {{}}");
            if mapped.remove(&(path, span)) != Some(expected.as_str()) {
                return Err("missing or changed namespace declaration".into());
            }
        } else {
            if !matches!(
                status,
                "invalid_namespace_name" | "duplicate_namespace" | "source_name_conflict"
            ) || (status == "invalid_namespace_name" && valid)
                || mapped.contains_key(&(path, span))
            {
                return Err("invalid blocked namespace".into());
            }
            require_issue(library, status, resource, &namespace["span"])?;
            blocked = true;
        }
    }
    Ok(blocked)
}

fn valid_name(name: &str) -> bool {
    name.len() <= 1024
        && name.starts_with("C_")
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn require_issue(library: &Value, code: &str, resource: &Value, span: &Value) -> Result<()> {
    if !list(library, "issues")?.iter().any(|issue| {
        issue["code"] == code
            && issue["source"]["scope"] == "annotation_alias_catalog"
            && issue["source"]["path"] == resource["path"]
            && issue["source"]["sha256"] == resource["sha256"]
            && issue["source"]["span"] == *span
    }) {
        return Err("missing external namespace issue".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Value {
        let raw = "C_Missing = {}\n";
        let span = json!({"start":0,"end":raw.len()-1});
        let hash = format!("sha256:{}", crate::manifest::digest(raw.as_bytes()));
        let link = json!({"scope":"annotation_alias_catalog","path":"Namespace.lua",
            "sha256":hash,"span":span});
        json!({
            "schema":"wow-native-annotation-library/5",
            "issues":[],
            "aliases":{
                "schema":"wow-native-alias-projection/7",
                "authority":"external_annotation_overlay",
                "source":{
                    "schema":"wow-native-alias-resource/5",
                    "revision":"a".repeat(40),
                    "path":"Namespace.lua",
                    "sha256":hash,
                    "source_bytes":raw.len(),
                    "text":raw,
                    "aliases":[],
                    "namespaces":[{"name":"C_Missing","span":span}]
                },
                "outcomes":[],
                "namespace_outcomes":[{"ordinal":0,"name":"C_Missing","status":"emitted"}]
            },
            "files":[{"text":"C_Missing = {}","mappings":[{
                "granularity":"declaration","generated":span,"source":link
            }]}]
        })
    }

    #[test]
    fn namespace_wire_requires_exact_resource_outcome_and_generated_bytes() -> Result<()> {
        let value = fixture();
        assert!(!super::super::verify(&value)?.blocked);
        for (pointer, replacement) in [
            ("/aliases/schema", json!("wow-native-alias-projection/6")),
            (
                "/aliases/source/schema",
                json!("wow-native-alias-resource/4"),
            ),
            ("/aliases/source/namespaces", json!([])),
            ("/aliases/namespace_outcomes", json!([])),
            ("/files/0/text", json!("C_Other = {}")),
            ("/files/0/mappings", json!([])),
            ("/aliases/source/namespaces/0/span/end", json!(usize::MAX)),
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(pointer).ok_or("mutation target")? = replacement;
            assert!(super::super::verify(&changed).is_err(), "{pointer}");
        }
        Ok(())
    }

    #[test]
    fn blocked_namespace_requires_matching_issue_and_no_generated_map() -> Result<()> {
        let mut value = fixture();
        value["aliases"]["namespace_outcomes"][0]["status"] = json!("source_name_conflict");
        let source = value["files"][0]["mappings"][0]["source"].clone();
        value["issues"] = json!([{"code":"source_name_conflict","source":source}]);
        value["files"][0]["mappings"] = json!([]);
        assert!(super::super::verify(&value)?.blocked);
        value["issues"] = json!([]);
        assert!(super::super::verify(&value).is_err());
        Ok(())
    }
}
