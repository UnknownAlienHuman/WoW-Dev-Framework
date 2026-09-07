//! Alias projection versions bind resources, not an inferred shared source file.
use super::{Result, list, manifest, text};
use serde_json::{Value, json};

pub(super) fn read(report: &Value) -> Result<Vec<&Value>> {
    if report["authority"] != "external_annotation_overlay" {
        return Err("invalid external alias authority".into());
    }
    let primary = &report["source"];
    let multiple = report["schema"] == "wow-native-alias-projection/3";
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
        if !matches!(
            (report["schema"].as_str(), primary["schema"].as_str()),
            (
                Some("wow-native-alias-projection/1"),
                Some("wow-native-alias-resource/1")
            ) | (
                Some("wow-native-alias-projection/2"),
                Some("wow-native-alias-resource/2")
            )
        ) {
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
    for resource in &resources {
        let extended = match resource["schema"].as_str() {
            Some("wow-native-alias-resource/1") => false,
            Some("wow-native-alias-resource/2") => true,
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
        if aliases.is_empty()
            || extended
                != aliases
                    .iter()
                    .any(|alias| alias.get("string_values").is_some())
        {
            return Err("alias schema does not describe its declarations".into());
        }
        bytes = bytes.checked_add(raw.len()).ok_or("alias byte limit")?;
        count = count
            .checked_add(aliases.len())
            .ok_or("alias count limit")?;
        if bytes > 2 * 1024 * 1024 || count > 4096 {
            return Err("alias resource aggregate limit".into());
        }
    }
    Ok(resources)
}
