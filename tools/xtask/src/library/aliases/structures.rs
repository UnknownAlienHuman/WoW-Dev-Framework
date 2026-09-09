//! Validate structure wire outcomes and their exact emitted declarations.
//! This is structural read-back, not another Lua parser or a subtype checker.
use super::{Result, list, lower_terms, span_key, text};
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
    let structure_profile = matches!(
        schema,
        Some(
            "wow-native-alias-projection/5"
                | "wow-native-alias-projection/6"
                | "wow-native-alias-projection/7"
        )
    );
    let field_maps = matches!(
        schema,
        Some("wow-native-alias-projection/6" | "wow-native-alias-projection/7")
    );
    if !structure_profile {
        if report.get("structure_outcomes").is_some()
            || report.get("unresolved_structure_fields").is_some()
        {
            return Err("unexpected structure projection fields".into());
        }
        return Ok(false);
    }
    let mut entries = Vec::new();
    for (&path, &resource) in sources {
        if resource.get("structures").is_some() {
            for structure in list(resource, "structures")? {
                entries.push((path, resource, structure));
            }
        }
    }
    if entries.is_empty() {
        if report.get("structure_outcomes").is_some()
            || report.get("unresolved_structure_fields").is_some()
        {
            return Err("unexpected structure projection fields".into());
        }
        if matches!(
            schema,
            Some("wow-native-alias-projection/5" | "wow-native-alias-projection/6")
        ) {
            return Err("missing external structure outcomes".into());
        }
        return Ok(false);
    }
    let outcomes = list(report, "structure_outcomes")?;
    if outcomes.len() != entries.len() {
        return Err("missing external structure outcomes".into());
    }
    let mut blocked = false;
    let mut spans = BTreeSet::new();
    let mut emitted_fields = BTreeSet::new();
    for (ordinal, ((path, resource, structure), outcome)) in
        entries.into_iter().zip(outcomes).enumerate()
    {
        let name = text(structure, "name")?;
        let span = span_key(&structure["span"])?;
        let raw = text(resource, "text")?;
        let header = structure["header_supported"]
            .as_bool()
            .ok_or("structure header status")?;
        let syntax_error = structure["syntax_error"]
            .as_bool()
            .ok_or("structure syntax status")?;
        if span.0 == span.1
            || raw.get(span.0..span.1).is_none()
            || !spans.insert((path, span))
            || outcome["ordinal"] != json!(ordinal)
            || outcome["name"] != name
        {
            return Err("structure source/outcome mismatch".into());
        }
        let fields = list(structure, "fields")?;
        let mut previous = span.0;
        for field in fields {
            let field_span = span_key(&field["span"])?;
            if field_span.0 >= field_span.1
                || field_span.0 <= span.0
                || field_span.0 < previous
                || field_span.1 > span.1
                || raw.get(field_span.0..field_span.1).is_none()
            {
                return Err("invalid external structure field span".into());
            }
            previous = field_span.1;
        }
        let status = text(outcome, "status")?;
        if status == "emitted" {
            if !header || syntax_error || !emitted.insert(name) {
                return Err("invalid emitted external structure".into());
            }
            let mut expected = format!("---@class {name}");
            let mut names = BTreeSet::new();
            for field in fields {
                let name = text(field, "name")?;
                if !names.insert(name) {
                    return Err("duplicate emitted structure field".into());
                }
                let ty = &field["field_type"];
                let array = ty["array"].as_bool().ok_or("structure array flag")?;
                let nullable = ty["nullable"].as_bool().ok_or("structure nullable flag")?;
                let mut lowered = lower_terms(list(ty, "terms")?)?;
                if lowered.contains('|') && (array || nullable) {
                    lowered = format!("({lowered})");
                }
                if array {
                    lowered.push_str("[]");
                }
                if nullable {
                    lowered.push('?');
                }
                let fragment = format!("---@field {name} {lowered}");
                let key = (path, span_key(&field["span"])?);
                if field_maps && mapped.remove(&key) != Some(fragment.as_str()) {
                    return Err("missing or changed external structure field mapping".into());
                }
                expected.push('\n');
                expected.push_str(&fragment);
                emitted_fields.insert(key);
            }
            if mapped.remove(&(path, span)) != Some(expected.as_str()) {
                return Err("missing or changed external structure declaration".into());
            }
        } else {
            if !matches!(
                status,
                "invalid_structure_name"
                    | "duplicate_structure"
                    | "source_name_conflict"
                    | "structure_syntax_error"
                    | "unsupported_structure"
            ) || mapped.contains_key(&(path, span))
                || (status == "structure_syntax_error" && !syntax_error)
            {
                return Err("invalid blocked external structure".into());
            }
            require_issue(library, status, resource, &structure["span"])?;
            blocked = true;
        }
    }
    let mut unresolved = BTreeSet::new();
    if report.get("unresolved_structure_fields").is_some() {
        let fields = list(report, "unresolved_structure_fields")?;
        if fields.is_empty() || fields.len() > 65_536 {
            return Err("invalid unresolved structure inventory".into());
        }
        for field in fields {
            let path = text(field, "path")?;
            let key = (path, span_key(&field["span"])?);
            let resource = sources.get(path).ok_or("unknown structure resource")?;
            if field["scope"] != "annotation_alias_catalog"
                || field["sha256"] != resource["sha256"]
                || !emitted_fields.contains(&key)
                || !unresolved.insert(key)
            {
                return Err("invalid unresolved structure field link".into());
            }
            require_issue(
                library,
                "unresolved_structure_field_type",
                resource,
                &field["span"],
            )?;
        }
        blocked = true;
    }
    for issue in list(library, "issues")? {
        if issue["code"] == "unresolved_structure_field_type" {
            let source = &issue["source"];
            let path = text(source, "path")?;
            let resource = sources
                .get(path)
                .ok_or("unknown structure issue resource")?;
            if source["scope"] != "annotation_alias_catalog"
                || source["sha256"] != resource["sha256"]
                || !unresolved.contains(&(path, span_key(&source["span"])?))
            {
                return Err("unreported unresolved structure field".into());
            }
        }
    }
    Ok(blocked)
}

fn require_issue(library: &Value, code: &str, resource: &Value, span: &Value) -> Result<()> {
    if !list(library, "issues")?.iter().any(|issue| {
        issue["code"] == code
            && issue["source"]["scope"] == "annotation_alias_catalog"
            && issue["source"]["path"] == resource["path"]
            && issue["source"]["sha256"] == resource["sha256"]
            && issue["source"]["span"] == *span
    }) {
        return Err("missing external structure issue".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(unknown: bool) -> Value {
        let ty = if unknown { "Missing" } else { "number" };
        let raw = format!("---@class Record\n---@field id {ty}?\n");
        let field_start = raw.find("---@field").unwrap_or(raw.len());
        let hash = format!("sha256:{}", crate::manifest::digest(raw.as_bytes()));
        let span = json!({"start":0,"end":raw.len()-1});
        let field_span = json!({"start":field_start,"end":raw.len()-1});
        let link = json!({"scope":"annotation_alias_catalog","path":"Types.lua","sha256":hash,"span":span});
        let mut field_link = link.clone();
        field_link["span"] = field_span.clone();
        let mut value = json!({"schema":"wow-native-annotation-library/5","issues":[],
            "aliases":{"schema":"wow-native-alias-projection/5","authority":"external_annotation_overlay",
                "source":{"schema":"wow-native-alias-resource/4","revision":"a".repeat(40),
                    "path":"Types.lua","sha256":hash,"source_bytes":raw.len(),"text":raw,"aliases":[],
                    "structures":[{"name":"Record","span":span,"header_supported":true,"syntax_error":false,
                        "fields":[{"name":"id","span":field_span,"field_type":{"terms":[ty],"array":false,"nullable":true}}]}]},
                "outcomes":[],"structure_outcomes":[{"ordinal":0,"name":"Record","status":"emitted"}]},
            "files":[{"text":raw.trim_end(),"mappings":[{"granularity":"declaration","generated":span,"source":link}]}]});
        if unknown {
            value["aliases"]["unresolved_structure_fields"] = json!([field_link]);
            value["issues"] =
                json!([{"code":"unresolved_structure_field_type","source":field_link}]);
        }
        value
    }

    #[test]
    fn structure_wire_requires_matching_profiles_fields_outcomes_and_generated_bytes() -> Result<()>
    {
        let value = fixture(false);
        assert!(!super::super::verify(&value)?.blocked);
        for (pointer, replacement) in [
            ("/aliases/schema", json!("wow-native-alias-projection/4")),
            (
                "/aliases/source/schema",
                json!("wow-native-alias-resource/3"),
            ),
            ("/aliases/source/structures/0/fields", json!([])),
            (
                "/aliases/source/structures/0/fields/0/field_type/nullable",
                json!(false),
            ),
            (
                "/aliases/source/structures/0/fields/0/span/end",
                json!(usize::MAX),
            ),
            (
                "/aliases/source/structures/0/header_supported",
                json!(false),
            ),
            ("/aliases/structure_outcomes", json!([])),
            ("/files/0/text", json!("---@class Other")),
            ("/files/0/mappings", json!([])),
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(pointer).ok_or("mutation target")? = replacement;
            assert!(super::super::verify(&changed).is_err(), "{pointer}");
        }
        Ok(())
    }

    #[test]
    fn unresolved_fields_require_both_report_and_matching_issue() -> Result<()> {
        let value = fixture(true);
        assert!(super::super::verify(&value)?.blocked);
        let mut missing = value.clone();
        missing["issues"] = json!([]);
        assert!(super::super::verify(&missing).is_err());
        let mut missing = value.clone();
        missing["aliases"]
            .as_object_mut()
            .ok_or("report")?
            .remove("unresolved_structure_fields");
        assert!(super::super::verify(&missing).is_err());
        Ok(())
    }

    #[test]
    fn member_profile_requires_exact_field_bytes_and_keeps_legacy_reports_readable() -> Result<()> {
        let mut value = fixture(false);
        assert!(!super::super::verify(&value)?.blocked);
        value["aliases"]["schema"] = json!("wow-native-alias-projection/6");
        assert!(super::super::verify(&value).is_err());
        let span = value["aliases"]["source"]["structures"][0]["fields"][0]["span"].clone();
        let mut link = value["files"][0]["mappings"][0]["source"].clone();
        link["span"] = span.clone();
        value["files"][0]["mappings"]
            .as_array_mut()
            .ok_or("maps")?
            .push(json!({"granularity":"field","generated":span,"source":link}));
        assert!(!super::super::verify(&value)?.blocked);
        for pointer in [
            "/files/0/mappings/1/generated/start",
            "/files/0/mappings/1/source/span/start",
        ] {
            let mut changed = value.clone();
            *changed.pointer_mut(pointer).ok_or("target")? = json!(0);
            assert!(super::super::verify(&changed).is_err());
        }
        let mut changed = value.clone();
        changed["aliases"]["schema"] = json!("wow-native-alias-projection/5");
        assert!(super::super::verify(&changed).is_err());
        changed["files"][0]["mappings"]
            .as_array_mut()
            .ok_or("maps")?
            .pop();
        assert!(!super::super::verify(&changed)?.blocked);
        Ok(())
    }

    #[test]
    fn namespace_only_profile_does_not_require_structure_outcomes() -> Result<()> {
        let raw = "C_Missing = {}\n";
        let span = json!({"start":0,"end":raw.len()-1});
        let hash = format!("sha256:{}", crate::manifest::digest(raw.as_bytes()));
        let link = json!({"scope":"annotation_alias_catalog","path":"Namespace.lua",
            "sha256":hash,"span":span});
        let value = json!({
            "schema":"wow-native-annotation-library/5",
            "issues":[],
            "aliases":{
                "schema":"wow-native-alias-projection/7",
                "authority":"external_annotation_overlay",
                "source":{"schema":"wow-native-alias-resource/5","revision":"a".repeat(40),
                    "path":"Namespace.lua","sha256":hash,"source_bytes":raw.len(),"text":raw,
                    "aliases":[],"namespaces":[{"name":"C_Missing","span":span}]},
                "outcomes":[],
                "namespace_outcomes":[{"ordinal":0,"name":"C_Missing","status":"emitted"}]
            },
            "files":[{"text":"C_Missing = {}","mappings":[{
                "granularity":"declaration","generated":span,"source":link
            }]}]
        });
        assert!(!super::super::verify(&value)?.blocked);
        Ok(())
    }
}
