//! Alias projection versions bind resources, not an inferred shared source file.
use super::{Result, list, manifest, text};
use serde_json::{Value, json};

pub(super) fn read(report: &Value) -> Result<Vec<&Value>> {
    if report["authority"] != "external_annotation_overlay" {
        return Err("invalid external alias authority".into());
    }
    let primary = &report["source"];
    let schema = report["schema"].as_str();
    let global_color_profile = schema == Some("wow-native-alias-projection/9");
    let function_capable = matches!(
        schema,
        Some("wow-native-alias-projection/8" | "wow-native-alias-projection/9")
    );
    let function_required = schema == Some("wow-native-alias-projection/8");
    let namespace_capable = matches!(
        schema,
        Some(
            "wow-native-alias-projection/7"
                | "wow-native-alias-projection/8"
                | "wow-native-alias-projection/9"
        )
    );
    let namespace_required = schema == Some("wow-native-alias-projection/7");
    let structure_capable = matches!(
        schema,
        Some(
            "wow-native-alias-projection/5"
                | "wow-native-alias-projection/6"
                | "wow-native-alias-projection/7"
                | "wow-native-alias-projection/8"
                | "wow-native-alias-projection/9"
        )
    );
    let structure_required = matches!(
        schema,
        Some("wow-native-alias-projection/5" | "wow-native-alias-projection/6")
    );
    let extended_profile =
        structure_capable || namespace_capable || function_capable || global_color_profile;
    let open_profile = schema == Some("wow-native-alias-projection/4") || extended_profile;
    let multiple = schema == Some("wow-native-alias-projection/3")
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
                (schema, primary["schema"].as_str()),
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
    let mut has_namespaces = false;
    let mut has_function_containers = false;
    let mut has_global_colors = false;
    let mut field_count = 0usize;
    let mut method_count = 0usize;
    for resource in &resources {
        let profile = match resource["schema"].as_str() {
            Some("wow-native-alias-resource/1") => 1,
            Some("wow-native-alias-resource/2") => 2,
            Some("wow-native-alias-resource/3") if open_profile => 3,
            Some("wow-native-alias-resource/4") if structure_capable => 4,
            Some("wow-native-alias-resource/5") if namespace_capable => 5,
            Some("wow-native-alias-resource/6") if function_capable => 6,
            Some("wow-native-alias-resource/7") if global_color_profile => 7,
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
        let namespaces = if profile == 5 {
            let namespaces = list(resource, "namespaces")?;
            if namespaces.is_empty() || !aliases.is_empty() || structures != 0 {
                return Err("invalid external namespace resource".into());
            }
            has_namespaces = true;
            namespaces.len()
        } else {
            if resource.get("namespaces").is_some() {
                return Err("unexpected external namespace field".into());
            }
            0
        };
        let function_items = if profile == 6 {
            let containers = list(resource, "function_containers")?;
            if containers.is_empty() || !aliases.is_empty() || structures != 0 || namespaces != 0 {
                return Err("invalid external function container resource".into());
            }
            has_function_containers = true;
            let previous_methods = method_count;
            for container in containers {
                let methods = list(container, "methods")?;
                if methods.is_empty() {
                    return Err("empty external function container".into());
                }
                method_count = method_count
                    .checked_add(methods.len())
                    .ok_or("function container method limit")?;
                for method in methods {
                    if list(method, "returns")?.len() > 16 {
                        return Err("function container return limit".into());
                    }
                }
            }
            if method_count > 65_536 || method_count - previous_methods > 4096 {
                return Err("function container method limit".into());
            }
            containers.len() + method_count - previous_methods
        } else {
            if resource.get("function_containers").is_some() {
                return Err("unexpected external function container field".into());
            }
            0
        };
        let global_colors = if profile == 7 {
            let colors = list(resource, "global_colors")?;
            if colors.is_empty()
                || !aliases.is_empty()
                || structures != 0
                || namespaces != 0
                || function_items != 0
            {
                return Err("invalid external global color resource".into());
            }
            for color in colors {
                let components = list(color, "components")?;
                if components.len() != 4
                    || components.iter().any(|component| {
                        component
                            .as_str()
                            .is_none_or(|component| component.is_empty() || component.len() > 64)
                    })
                {
                    return Err("invalid external global color components".into());
                }
            }
            has_global_colors = true;
            colors.len()
        } else {
            if resource.get("global_colors").is_some() {
                return Err("unexpected external global color field".into());
            }
            0
        };
        if (aliases.is_empty()
            && structures == 0
            && namespaces == 0
            && function_items == 0
            && global_colors == 0)
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
            .checked_add(
                aliases.len() + structures + namespaces + function_items + global_colors,
            )
            .ok_or("alias count limit")?;
        if bytes > 2 * 1024 * 1024 || count > 4096 {
            return Err("alias resource aggregate limit".into());
        }
    }
    if global_color_profile != has_global_colors
        || (!function_capable && has_function_containers)
        || (function_required && !has_function_containers)
        || (!namespace_capable && has_namespaces)
        || (namespace_required && !has_namespaces)
        || (!structure_capable && has_structures)
        || (structure_required && !has_structures)
        || (!extended_profile && open_profile != has_open_resource)
    {
        return Err("alias projection profile does not match its resources".into());
    }
    Ok(resources)
}
