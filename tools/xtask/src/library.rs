//! Verify final native-driver output bytes. This checks artifact consistency,
//! not whether annotation semantics match a client or language server.
use crate::{Result, manifest};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;
fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("missing {key}").into())
}
fn list<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    value[key]
        .as_array()
        .ok_or_else(|| format!("missing {key}").into())
}
fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("invalid artifact file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("artifact byte bound exceeded".into());
    }
    Ok(bytes)
}
pub fn verify(root: &Path, require_input_complete: bool) -> Result<u8> {
    let report: Value =
        serde_json::from_slice(&read(&root.join("source-report.json"), 512 * 1024 * 1024)?)?;
    let library = &report["library"];
    if text(&report, "schema")? != "wow-native-source-build/1"
        || !matches!(
            text(library, "schema")?,
            "wow-native-annotation-library/3" | "wow-native-annotation-library/4"
        )
    {
        return Err("unsupported native report schema".into());
    }
    let revision = text(&report, "revision")?;
    if !crate::git::oid(revision)
        || text(library, "revision")? != revision
        || report["negative_authority"] != false
        || library["negative_authority"] != false
    {
        return Err("native report identity/authority mismatch".into());
    }
    let order = list(&report, "source_order")?;
    let sources = list(library, "sources")?;
    let failures = list(&report, "input_failures")?;
    let issues = list(library, "issues")?;
    let files = list(library, "files")?;
    if sources.is_empty()
        || files.is_empty()
        || order.len() != sources.len() + failures.len()
        || report["candidate_files"] != json!(order.len())
        || report["admitted_files"] != json!(sources.len())
    {
        return Err("inconsistent native source counts".into());
    }
    let mut selected = BTreeSet::new();
    for path in order {
        let path = path.as_str().ok_or("invalid source path")?;
        manifest::validate_path(path)?;
        if !selected.insert(path) {
            return Err("duplicate source path".into());
        }
    }
    let mut source_map = BTreeMap::new();
    for source in sources {
        let path = text(source, "path")?;
        let source_hash = text(source, "sha256")?;
        let digest = source_hash
            .strip_prefix("sha256:")
            .ok_or("invalid source digest")?;
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err("invalid source digest".into());
        }
        if !selected.remove(path)
            || text(source, "revision")? != revision
            || source_map.insert(path, source).is_some()
        {
            return Err("source identity mismatch".into());
        }
    }
    for failure in failures {
        if !selected.remove(text(failure, "path")?) {
            return Err("invalid failure inventory".into());
        }
    }
    if !selected.is_empty() {
        return Err("unaccounted source input".into());
    }
    if require_input_complete && !failures.is_empty() {
        return Err("native source admission is incomplete".into());
    }
    let correction_blockers = verify_corrections(library)?;
    let partial = !failures.is_empty() || !issues.is_empty() || correction_blockers;
    if text(&report, "status")?
        != if partial {
            "partial"
        } else {
            "projected_with_sidecars"
        }
        || text(library, "projection")?
            != if issues.is_empty() && !correction_blockers {
                "projected_with_sidecars"
            } else {
                "partial"
            }
    {
        return Err("native report status mismatch".into());
    }
    let mut expected = BTreeSet::from(["source-report.json"]);
    let mut total = 0usize;
    for file in files {
        let path = text(file, "path")?;
        manifest::validate_path(path)?;
        if path.contains('/') || !path.ends_with(".lua") || !expected.insert(path) {
            return Err("invalid generated file path".into());
        }
        let bytes = read(&root.join(path), 8 * 1024 * 1024)?;
        total = total.checked_add(bytes.len()).ok_or("library limit")?;
        if total > 64 * 1024 * 1024
            || bytes != text(file, "text")?.as_bytes()
            || text(file, "sha256")? != format!("sha256:{}", manifest::digest(&bytes))
        {
            return Err("generated file bytes/digest mismatch".into());
        }
        let generated = std::str::from_utf8(&bytes)?;
        for mapping in list(file, "mappings")? {
            let start = mapping["generated"]["start"]
                .as_u64()
                .ok_or("invalid generated range")? as usize;
            let end = mapping["generated"]["end"]
                .as_u64()
                .ok_or("invalid generated range")? as usize;
            if generated.get(start..end).is_none() {
                return Err("invalid UTF-8 generated range".into());
            }
            let source = &mapping["source"];
            let original = source_map
                .get(text(source, "path")?)
                .ok_or("mapping points outside corpus")?;
            let start = source["span"]["start"]
                .as_u64()
                .ok_or("invalid source span")?;
            let end = source["span"]["end"]
                .as_u64()
                .ok_or("invalid source span")?;
            if start > end
                || end
                    > original["source_bytes"]
                        .as_u64()
                        .ok_or("invalid source length")?
                || source["sha256"] != original["sha256"]
            {
                return Err("source map identity/range mismatch".into());
            }
        }
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("non-UTF-8 artifact path")?;
        if !expected.remove(name) {
            return Err("unlisted artifact file".into());
        }
    }
    if !expected.is_empty() {
        return Err("missing artifact file".into());
    }
    println!(
        "{}",
        json!({"status":if partial {"partial"} else {"verified_artifact"},"revision":revision,"admitted_files":sources.len(),"input_failures":failures.len(),"projection_issues":issues.len(),"annotation_files":files.len(),"negative_authority":false,"semantic_consumer_compatibility":"not_evaluated"})
    );
    Ok(if partial { 3 } else { 0 })
}

// Consistency checks only: do not reproduce correction matching here or pretend
// a self-consistent report proves external review/source truth.
fn verify_corrections(library: &Value) -> Result<bool> {
    if library["schema"] == "wow-native-annotation-library/3" {
        if library.get("corrections").is_some() {
            return Err("unexpected correction report in v3".into());
        }
        return Ok(false);
    }
    let report = &library["corrections"];
    if report["schema"] != "wow-native-correction-applications/1" {
        return Err("missing correction report".into());
    }
    let set = &report["corrections"]["set"];
    let hash = format!("sha256:{}", manifest::digest(&serde_json::to_vec(set)?));
    if report["corrections"]["id"] != hash {
        return Err("correction set digest mismatch".into());
    }
    let records = list(set, "records")?;
    let applications = list(report, "applications")?;
    if records.len() != applications.len() {
        return Err("missing correction outcomes".into());
    }
    let mut blocked = false;
    for (record, application) in records.iter().zip(applications) {
        if record["id"] != application["correction_id"] || record["target"] != application["target"]
        {
            return Err("correction target/outcome mismatch".into());
        }
        match text(application, "status")? {
            "applied" => {
                if application["after"] != record["after"]
                    || application["before"] != record["before"]
                    || application["observed_source_sha256"] != record["expected_source_sha256"]
                    || application["observed_raw_sha256"] != record["expected_raw_sha256"]
                {
                    return Err("inconsistent applied correction".into());
                }
            }
            "expired" | "rejected" | "conflict" | "not_applicable" => {
                if !application["after"].is_null() {
                    return Err("blocked correction has replacement".into());
                }
                blocked |= application["status"] != "not_applicable";
            }
            _ => return Err("unknown correction outcome".into()),
        }
    }
    Ok(blocked)
}
