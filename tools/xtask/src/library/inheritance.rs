//! Bind applied widget-parent corrections to retained source observations.
//! Structural verification only; this neither imports Ketho nor certifies review.
use super::{Result, list, manifest, text};
use serde_json::Value;

pub(super) fn verify(
    library: &Value,
    set: &Value,
    record: &Value,
    application: &Value,
) -> Result<()> {
    let projection = &record["target"]["projection"];
    if projection["kind"] != "widget_base" {
        return Ok(());
    }
    if set["schema"] != "wow-native-corrections/2" {
        return Err("widget base requires the explicit inheritance correction profile".into());
    }
    if application["status"] != "applied" {
        return Ok(());
    }
    if set["revision"] != library["revision"]
        || record["before"]["kind"] != "absent"
        || record["after"]["kind"] != "text"
    {
        return Err("invalid applied widget base correction".into());
    }
    let target = &record["target"];
    let child = registration(
        library,
        text(target, "path")?,
        &target["registration"],
        &record["expected_source_sha256"],
        &record["expected_raw_sha256"],
    )?;
    let parent = registration(
        library,
        text(projection, "parent_path")?,
        &projection["parent_registration"],
        &projection["expected_parent_source_sha256"],
        &projection["expected_parent_raw_sha256"],
    )?;
    if property(child, "Type")? != "ScriptObject"
        || property(parent, "Type")? != "ScriptObject"
        || property(parent, "Name")? != text(&record["after"], "value")?
        || property(child, "Name")? == property(parent, "Name")?
    {
        return Err("widget base does not join distinct source ScriptObjects".into());
    }
    Ok(())
}

fn registration<'a>(
    library: &'a Value,
    path: &str,
    ordinal: &Value,
    expected_source: &Value,
    expected_raw: &Value,
) -> Result<&'a Value> {
    let ordinal = usize::try_from(ordinal.as_u64().ok_or("invalid widget registration")?)?;
    let mut sources = list(library, "sources")?
        .iter()
        .filter(|s| s["path"] == path);
    let source = sources.next().ok_or("widget source is not retained")?;
    if sources.next().is_some() || &source["sha256"] != expected_source {
        return Err("widget source guard mismatch".into());
    }
    let registration = list(source, "registrations")?
        .get(ordinal)
        .ok_or("widget registration is not retained")?;
    let raw = &registration["value"];
    let hash = format!("sha256:{}", manifest::digest(&serde_json::to_vec(raw)?));
    if registration["ordinal"].as_u64() != Some(ordinal as u64) || expected_raw != &hash {
        return Err("widget raw observation guard mismatch".into());
    }
    Ok(raw)
}

fn property<'a>(raw: &'a Value, name: &str) -> Result<&'a str> {
    let fields = raw["kind"]["Table"]
        .as_array()
        .ok_or("invalid widget source table")?;
    let mut matches = fields.iter().filter(|field| field["key"]["Name"] == name);
    let field = matches.next().ok_or("missing widget source property")?;
    if matches.next().is_some() {
        return Err("duplicate widget source property".into());
    }
    field["value"]["kind"]["String"]
        .as_str()
        .ok_or_else(|| "widget property is not a string".into())
}
