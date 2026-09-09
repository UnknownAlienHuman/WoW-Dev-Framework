//! Alias projection versions bind resources, not an inferred shared source file.
use super::{Result, list, manifest, text};
use serde_json::{Value, json};

pub(super) fn read(report: &Value) -> Result<Vec<&Value>> {
    if report["authority"] != "external_annotation_overlay" {
        return Err("invalid external alias authority".into());
    }
    let primary = &report["source"];
    let structure_profile = report["schema"] == "wow-native-alias-projection/5";
    let open_profile = report["schema"] == "wow-native-alias-projection/4" || structure_profile;
    let multiple = report["schema"] == "wow-native-alias-projection/3"
        || (open_profile && report.get("additional_sources").is_some());
    let mut resources = vec![primary];
    if multiple {
        let additional = list(report, "additional_sources")?;
        if additional.is_empty() || additional.len() >= 32 {
            return Err("invalid alias resource set size".into());
        }
        resources.extend(additional);
    } else {
        if report.get("additional_sources").is_some() {
            return Err("unexpected multi-resource alias field".into());
        }
        if !open_profile
            && !matches!(
                (report["schema"].as_str(), primary["schema"].as_str()),
                (
                    Some("wow-native-alias-projection/1"),
                    Some("wow-native-alias-resource/1")
                ) | (
                    Some("wow-native-alias-projection/2"),
                    Some("wow-native-alias-resource/2")
                )
            )
        {
            return Err("invalid external alias schema pair".into());
        }
    }
    let revision = text(primary, "revision")?;
    if !crate::git::oid(revision) {
        return Err("invalid external alias revision".into());
    }
    let mut previous = None;
    let mut bytes = 0usize;
    let mut count = 0usize;
    let mut has_open_resource = false;
    let mut has_structures = false;
    let mut field_count = 0usize;
    for resource in &resources {
        let profile = match resource["schema"].as_str() {
            Some("wow-native-alias-resource/1") => 1,
            Some("wow-native-alias-resource/2") => 2,
            Some("wow-native-alias-resource/3") if open_profile => 3,
            Some("wow-native-alias-resource/4") if structure_profile => 4,
            _ => return Err("unsupported alias resource schema".into()),
        };
        let path = text(resource, "path")?;
        manifest::validate_path(path)?;
        if text(resource, "revision")? != revision
            || previous.is_some_and(|previous| previous >= path)
        {
            return Err("mixed, duplicate or unordered alias resource identities".into());
        }
        previous = Some(path);
        let raw = text(resource, "text")?;
        if raw.len() > 256 * 1024
            || resource["source_bytes"] != json!(raw.len())
            || resource["sha256"] != format!("sha256:{}", manifest::digest(raw.as_bytes()))
        {
            return Err("alias resource digest/length mismatch".into());
        }
        let aliases = list(resource, "aliases")?;
        let has_literals = aliases
            .iter()
            .any(|alias| alias.get("string_values").is_some());
        let has_base = aliases
            .iter()
            .any(|alias| alias.get("string_base").is_some());
        has_open_resource |= has_base;
        let structures = if profile == 4 {
            let structures = list(resource, "structures")?;
            if structures.is_empty() {
                return Err("empty external structure resource".into());
            }
            has_structures = true;
            let previous_fields = field_count;
            for structure in structures {
                field_count = field_count
                    .checked_add(list(structure, "fields")?.len())
                    .ok_or("structure field limit")?;
            }
            if field_count > 65_536 || field_count - previous_fields > 4096 {
                return Err("structure field limit".into());
            }
            structures.len()
        } else {
            if resource.get("structures").is_some() {
                return Err("unexpected external structure field".into());
            }
            0
        };
        if (aliases.is_empty() && structures == 0)
            || (profile < 4 && ((profile >= 2) != has_literals || (profile == 3) != has_base))
        {
            return Err("alias schema does not describe its declarations".into());
        }
        for alias in aliases {
            if let Some(base) = alias.get("string_base")
                && (base != "string"
                    || alias.get("terms") != Some(&Value::Null)
                    || alias
                        .get("string_values")
                        .and_then(Value::as_array)
                        .is_none())
            {
                return Err("invalid explicit open string declaration".into());
            }
        }
        bytes = bytes.checked_add(raw.len()).ok_or("alias byte limit")?;
        count = count
            .checked_add(aliases.len() + structures)
            .ok_or("alias count limit")?;
        if bytes > 2 * 1024 * 1024 || count > 4096 {
            return Err("alias resource aggregate limit".into());
        }
    }
    if structure_profile != has_structures
        || (!structure_profile && open_profile != has_open_resource)
    {
        return Err("alias projection profile does not match its open resources".into());
    }
    Ok(resources)
}
