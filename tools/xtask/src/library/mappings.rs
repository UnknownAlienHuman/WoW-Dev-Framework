//! Final-byte mapping layout and exact raw-member joins; no Lua parsing.
use super::{Result, list, text};
use serde_json::Value;
use std::collections::BTreeMap;

const PROFILE: &str = "wow-native-field-maps/1";
const MAX_MAPPINGS: usize = 1_048_576;
type Key<'a> = (&'a str, usize, usize);

#[derive(Default)]
pub(super) struct Tables<'a> {
    native: BTreeMap<Key<'a>, &'a Value>,
    external: BTreeMap<Key<'a>, Vec<Member>>,
}

pub(super) fn profile(library: &Value) -> Result<bool> {
    match library.get("source_map_profile") {
        None => Ok(false),
        Some(Value::String(profile)) if profile == PROFILE => Ok(true),
        _ => Err("unsupported native source map profile".into()),
    }
}

pub(super) fn source_tables<'a>(
    sources: &BTreeMap<&'a str, &'a Value>,
    detailed: bool,
) -> Result<Tables<'a>> {
    let mut tables = Tables::default();
    if !detailed {
        return Ok(tables);
    }
    let mut visited = 0usize;
    for (&path, &source) in sources {
        let mut pending = list(source, "registrations")?
            .iter()
            .map(|registration| &registration["value"])
            .collect::<Vec<_>>();
        while let Some(raw) = pending.pop() {
            visited += 1;
            if visited > 4 * MAX_MAPPINGS {
                return Err("raw source map input limit".into());
            }
            let Some(fields) = raw["kind"].get("Table") else {
                continue;
            };
            let (start, end) = span(&raw["span"])?;
            if let Some(previous) = tables.native.insert((path, start, end), raw)
                && previous != raw
            {
                return Err("conflicting raw tables at one source identity".into());
            }
            if tables.native.len() > MAX_MAPPINGS {
                return Err("raw source table limit".into());
            }
            for field in fields.as_array().ok_or("invalid raw source table")? {
                pending.push(&field["value"]);
            }
        }
    }
    Ok(tables)
}

/// Called only after external catalog outcomes, spans and emitted bytes pass
/// their owner verifier. The two maps remain separate even for equal paths.
pub(super) fn include_catalogs<'a>(
    tables: &mut Tables<'a>,
    library: &Value,
    sources: &BTreeMap<&'a str, &'a Value>,
    detailed: bool,
) -> Result<()> {
    if !matches!(
        library["aliases"]["schema"].as_str(),
        Some("wow-native-alias-projection/6" | "wow-native-alias-projection/7")
    ) {
        return Ok(());
    }
    let has_structures = sources
        .values()
        .any(|resource| resource.get("structures").is_some());
    if !has_structures {
        return Ok(());
    }
    if !detailed {
        return Err("catalog field mappings require the native field-map profile".into());
    }
    let mut count = 0usize;
    for (&path, &resource) in sources {
        if resource.get("structures").is_none() {
            continue;
        }
        for structure in list(resource, "structures")? {
            let (start, end) = span(&structure["span"])?;
            let mut expected = Vec::new();
            for field in list(structure, "fields")? {
                count = count.checked_add(1).ok_or("catalog member limit")?;
                if count > MAX_MAPPINGS {
                    return Err("catalog member limit".into());
                }
                expected.push(("field", span(&field["span"])?));
            }
            if tables
                .external
                .insert((path, start, end), expected)
                .is_some()
            {
                return Err("duplicate external class source identity".into());
            }
        }
    }
    Ok(())
}

struct Declaration<'a> {
    source: &'a Value,
    start: usize,
    end: usize,
    expected: Vec<Member>,
    consumed: usize,
    member_end: usize,
}

pub(super) fn verify(
    file: &Value,
    detailed: bool,
    tables: &Tables<'_>,
    count: &mut usize,
) -> Result<()> {
    let mappings = list(file, "mappings")?;
    *count = count
        .checked_add(mappings.len())
        .ok_or("source map limit")?;
    if *count > MAX_MAPPINGS || (detailed && mappings.is_empty()) {
        return Err("source map inventory limit or empty mapped file".into());
    }
    let generated = text(file, "text")?;
    let mut declarations: Vec<Declaration<'_>> = Vec::new();
    let mut current = 0;
    let mut members_started = false;
    let mut literal = false;
    for mapping in mappings {
        let kind = mapping["granularity"].as_str();
        if !detailed {
            // Retain the older report contract, including the first member-map
            // checkpoint that predated an explicit completeness profile.
            continue;
        }
        let (start, end) = span(&mapping["generated"])?;
        if generated.get(start..end).is_none() {
            return Err("mapping range is not on final UTF-8 boundaries".into());
        }
        match kind {
            Some("declaration") => {
                if members_started
                    || literal
                    || declarations.last().is_some_and(|parent| start < parent.end)
                {
                    return Err("unordered or overlapping declaration maps".into());
                }
                let source = &mapping["source"];
                let (first, last) = span(&source["span"])?;
                let key = (text(source, "path")?, first, last);
                let expected = match source.get("scope") {
                    None => {
                        let raw = tables
                            .native
                            .get(&key)
                            .ok_or("declaration has no exact raw table")?;
                        members(raw)?
                    }
                    Some(scope) if scope == "annotation_alias_catalog" => {
                        tables.external.get(&key).cloned().unwrap_or_default()
                    }
                    _ => return Err("unknown declaration source scope".into()),
                };
                declarations.push(Declaration {
                    source,
                    start,
                    end,
                    expected,
                    consumed: 0,
                    member_end: start,
                });
            }
            Some("literal_file") => {
                if start != 0
                    || end != generated.len()
                    || !declarations.is_empty()
                    || members_started
                {
                    return Err("invalid whole literal-file mapping".into());
                }
                literal = true;
            }
            Some(role @ ("parameter" | "return" | "field")) => {
                members_started = true;
                while declarations
                    .get(current)
                    .is_some_and(|parent| start >= parent.end)
                {
                    current += 1;
                }
                let owner = declarations
                    .get_mut(current)
                    .ok_or("member has no declaration")?;
                let source = &mapping["source"];
                let original = owner.source;
                let observed = (role, span(&source["span"])?);
                // Immutable local tables can be outside the callable's byte
                // span. The exact raw descriptor and ordinal bind membership.
                if owner.expected.get(owner.consumed) != Some(&observed)
                    || start < owner.start
                    || end > owner.end
                    || start < owner.member_end
                    || source.get("scope") != original.get("scope")
                    || source["path"] != original["path"]
                    || source["sha256"] != original["sha256"]
                {
                    return Err("member map does not match its ordered source field".into());
                }
                owner.member_end = end;
                owner.consumed += 1;
            }
            _ => return Err("unknown mapping granularity for native field maps".into()),
        }
    }
    if declarations
        .iter()
        .any(|parent| parent.consumed != parent.expected.len())
    {
        return Err("missing declaration member maps".into());
    }
    Ok(())
}

type Member = (&'static str, (usize, usize));

fn members(raw: &Value) -> Result<Vec<Member>> {
    let fields = raw["kind"]["Table"]
        .as_array()
        .ok_or("invalid raw declaration")?;
    let mut properties = BTreeMap::new();
    for field in fields {
        let name = field["key"]["Name"]
            .as_str()
            .ok_or("invalid declaration key")?;
        if properties.insert(name, &field["value"]).is_some() {
            return Err("duplicate raw declaration key".into());
        }
    }
    let kind = properties
        .get("Type")
        .map(|value| {
            value["kind"]["String"]
                .as_str()
                .ok_or("invalid declaration type")
        })
        .transpose()?;
    let collections: &[(&'static str, &'static str)] = match kind {
        Some("Structure") => &[("Fields", "field")],
        Some("ScriptObject" | "System") => &[],
        Some("Function" | "CallbackType") | None => {
            &[("Arguments", "parameter"), ("Returns", "return")]
        }
        _ => return Err("unsupported raw declaration for member maps".into()),
    };
    let mut result = Vec::new();
    for &(key, role) in collections {
        let Some(collection) = properties.get(key) else {
            continue;
        };
        let values = collection["kind"]["Table"]
            .as_array()
            .ok_or("invalid raw member list")?;
        let mut ordered = BTreeMap::new();
        for entry in values {
            let index = entry["key"]["Index"]
                .as_u64()
                .ok_or("invalid member ordinal")?;
            if ordered.insert(index, &entry["value"]).is_some() {
                return Err("duplicate member ordinal".into());
            }
        }
        for (position, (&index, value)) in ordered.iter().enumerate() {
            if index != (position + 1) as u64
                || result.len() >= MAX_MAPPINGS
                || value["kind"]["Table"].as_array().is_none()
            {
                return Err("invalid source member inventory".into());
            }
            result.push((role, span(&value["span"])?));
        }
    }
    Ok(result)
}

fn span(value: &Value) -> Result<(usize, usize)> {
    let start = usize::try_from(value["start"].as_u64().ok_or("missing mapping start")?)?;
    let end = usize::try_from(value["end"].as_u64().ok_or("missing mapping end")?)?;
    if start >= end {
        return Err("empty or reversed mapping span".into());
    }
    Ok((start, end))
}

#[cfg(test)]
mod tests;
