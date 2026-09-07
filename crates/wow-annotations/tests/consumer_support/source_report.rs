//! Narrow fixture receipt checks; not the general artifact verifier or E1-C.
use super::Result;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
use wow_reference::native::source_digest;

pub(super) fn validate(
    report: &Value,
    root: &Path,
    revision: &str,
    sources: &[(&str, &str)],
) -> Result<()> {
    let toc = sources
        .iter()
        .map(|(name, _)| format!("{name}\n"))
        .collect::<String>();
    let order = sources.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    let library = &report["library"];
    if report["schema"] != "wow-native-source-build/1"
        || report["revision"] != revision
        || report["selector"] != "HEAD"
        || report["environment"] != "Mainline"
        || report["freshness"] != "not_network_verified"
        || report["status"] != "projected_with_sidecars"
        || report["negative_authority"] != false
        || report["toc"] != json!({"path":"API.toc","sha256":source_digest(toc.as_bytes())})
        || report["source_order"] != json!(order)
        || report["candidate_files"] != json!(sources.len())
        || report["admitted_files"] != json!(sources.len())
        || report["input_failures"] != json!([])
        || library["schema"] != "wow-native-annotation-library/3"
        || library["revision"] != revision
        || library["projection"] != "projected_with_sidecars"
        || library["negative_authority"] != false
        || library["issues"] != json!([])
    {
        return Err("incomplete or mismatched source build receipt".into());
    }
    let expected: BTreeMap<_, _> = sources.iter().copied().collect();
    let mut seen = BTreeSet::new();
    for source in array(&library["sources"])? {
        let path = source["path"].as_str().ok_or("missing source path")?;
        let text = expected.get(path).ok_or("unexpected source document")?;
        if !seen.insert(path)
            || source["revision"] != revision
            || source["sha256"] != source_digest(text.as_bytes())
            || source["source_bytes"] != json!(text.len())
            || array(&source["registrations"])?.is_empty()
        {
            return Err("source document identity mismatch".into());
        }
    }
    if seen.len() != expected.len() {
        return Err("missing source document".into());
    }
    let files = array(&library["files"])?;
    if files.is_empty() {
        return Err("empty generated library".into());
    }
    let mut names = BTreeSet::from(["source-report.json".to_owned()]);
    for file in files {
        let name = file["path"].as_str().ok_or("missing generated filename")?;
        if !name.ends_with(".lua")
            || name.starts_with('.')
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            || !names.insert(name.to_owned())
        {
            return Err("invalid or duplicate generated filename".into());
        }
        let text = file["text"].as_str().ok_or("missing generated text")?;
        if file["sha256"] != source_digest(text.as_bytes())
            || fs::read(root.join(name))? != text.as_bytes()
        {
            return Err("generated file bytes or digest mismatch".into());
        }
        let mappings = array(&file["mappings"])?;
        if mappings.is_empty() {
            return Err("unmapped generated file".into());
        }
        for mapping in mappings {
            span(&mapping["generated"], text)?;
            let link = &mapping["source"];
            let path = link["path"].as_str().ok_or("missing mapped source")?;
            let source = expected.get(path).ok_or("unknown mapped source")?;
            if link.get("scope").is_some() || link["sha256"] != source_digest(source.as_bytes()) {
                return Err("mapped source identity mismatch".into());
            }
            span(&link["span"], source)?;
        }
    }
    let mut actual = BTreeSet::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err("unexpected source build entry".into());
        }
        actual.insert(
            entry
                .file_name()
                .into_string()
                .map_err(|_| "invalid output filename")?,
        );
    }
    if actual != names {
        return Err("source build output inventory mismatch".into());
    }
    Ok(())
}

fn array(value: &Value) -> Result<&[Value]> {
    value
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| "missing receipt array".into())
}

fn span(value: &Value, text: &str) -> Result<()> {
    let start = usize::try_from(value["start"].as_u64().ok_or("missing span start")?)?;
    let end = usize::try_from(value["end"].as_u64().ok_or("missing span end")?)?;
    if start >= end || text.get(start..end).is_none() {
        return Err("invalid source/generated byte span".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "source_report_tests.rs"]
mod tests;
