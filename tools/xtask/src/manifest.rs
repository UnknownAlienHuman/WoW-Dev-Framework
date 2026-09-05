//! Exact local Git inventory. Reads raw object bytes, never export-filtered
//! archives or worktree files. No Lua, XML or addon behavior is evaluated here.
use crate::{Result, git};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
const MAX_FILES: usize = 200_000;
const MAX_FILE: usize = 32 * 1024 * 1024;
const MAX_TOTAL: usize = 256 * 1024 * 1024;
const MAX_MANIFEST: usize = 64 * 1024 * 1024;
const EXTENSIONS: [&str; 4] = [".lua", ".toc", ".xml", ".xsd"];
pub fn validate_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 4096
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("noncanonical repository-relative path".into());
    }
    Ok(())
}
pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
fn class(path: &str) -> &'static str {
    if path == "version.txt" {
        return "version";
    }
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".lua") {
        if path
            .split('/')
            .any(|part| part == "Blizzard_APIDocumentationGenerated")
        {
            "generated_api"
        } else {
            "lua"
        }
    } else if lower.ends_with(".toc") {
        "toc"
    } else if lower.ends_with(".xml") {
        "xml"
    } else {
        "schema"
    }
}
pub fn build(root: &Path, selector: &str, label: &str) -> Result<Value> {
    if label.is_empty() || label.chars().any(char::is_control) {
        return Err("invalid selector label".into());
    }
    let revision = git::resolve(root, selector)?;
    let format = git::text(root, &["rev-parse", "--show-object-format"])?;
    let oid_len = match format.as_str() {
        "sha1" => 40,
        "sha256" => 64,
        _ => return Err("unsupported Git object format".into()),
    };
    if revision.len() != oid_len {
        return Err("commit/object format mismatch".into());
    }
    let listing = git::run(
        root,
        &["ls-tree", "-rlz", "--full-tree", &revision],
        None,
        MAX_MANIFEST,
    )?;
    let mut selected = Vec::new();
    let mut tracked = 0usize;
    let mut total = 0usize;
    for raw in listing.split(|b| *b == 0).filter(|v| !v.is_empty()) {
        tracked += 1;
        if tracked > MAX_FILES {
            return Err("source file-count limit exceeded".into());
        }
        let entry = std::str::from_utf8(raw)?;
        let (metadata, path) = entry.split_once('\t').ok_or("invalid Git tree record")?;
        validate_path(path)?;
        let values = metadata.split_whitespace().collect::<Vec<_>>();
        if values.len() != 4 || !matches!(values[0], "100644" | "100755") || values[1] != "blob" {
            return Err("nonregular source entry rejected".into());
        }
        let hash = values[2];
        if !git::oid(hash) || hash.len() != oid_len {
            return Err("invalid Git blob identifier".into());
        }
        let size: usize = values[3].parse()?;
        if path != "version.txt"
            && !EXTENSIONS
                .iter()
                .any(|ext| path.to_ascii_lowercase().ends_with(ext))
        {
            continue;
        }
        total = total.checked_add(size).ok_or("source byte limit")?;
        if size > MAX_FILE || total > MAX_TOTAL {
            return Err("source byte limit exceeded".into());
        }
        selected.push((path.to_owned(), hash.to_owned(), size));
    }
    selected.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut version = None;
    let mut files = Vec::new();
    let mut kinds = BTreeMap::<&str, usize>::new();
    for (path, hash, size) in &selected {
        let bytes = git::run(root, &["cat-file", "blob", hash], None, *size)?;
        if bytes.len() != *size {
            return Err("blob length does not match tree".into());
        }
        // Git computes the repository's object hash (SHA-1 or SHA-256). Keep
        // crypto in the existing Git implementation, not a bespoke hash routine.
        let actual = git::run(
            root,
            &["hash-object", "--no-filters", "--stdin"],
            Some(bytes.clone()),
            128,
        )?;
        if std::str::from_utf8(&actual)?.trim() != hash {
            return Err("Git blob digest mismatch".into());
        }
        if path == "version.txt" {
            let text = std::str::from_utf8(&bytes)?.trim();
            if text.is_empty() || text.chars().any(char::is_control) {
                return Err("invalid source version".into());
            }
            version = Some(text.to_owned());
        }
        let kind = class(path);
        *kinds.entry(kind).or_default() += 1;
        files.push(json!({"path":path,"kind":kind,"bytes":size,"git_blob_algorithm":format,"git_blob_id":hash,"content_sha256":digest(&bytes)}));
    }
    let mut coverage = json!({"tracked_files":tracked,"included_files":files.len(),"excluded_files":tracked-files.len(),"included_bytes":total});
    for (kind, count) in kinds {
        coverage[format!("kind_{kind}")] = json!(count);
    }
    let mut value = json!({"schema_version":1,
        "source":{"source_id":"blizzard-ui","selector":label,"revision":revision,"git_object_format":format,"version":version.ok_or("version.txt is absent from the selected revision")?,"acquisition":"local_git_object_database"},
        "selection":{"extensions":EXTENSIONS,"version_path":"version.txt","non_regular_entries":"reject","working_tree":"ignored"},
        "coverage":coverage,"files":files});
    value["manifest_sha256"] = json!(digest(&serde_json::to_vec(&value)?));
    Ok(value)
}
/// New-only publication. Failed writes remove only the file this call created;
/// an existing destination is never overwritten. Not a crash-durable store port.
pub fn write_new(path: &Path, value: &Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    let result = file.write_all(&bytes).and_then(|_| file.sync_all());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(path);
    }
    result?;
    Ok(())
}
pub fn verify(file: &Path, root: &Path, current_ref: Option<&str>) -> Result<u8> {
    let mut bytes = Vec::new();
    fs::File::open(file)?
        .take(MAX_MANIFEST as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_MANIFEST {
        return Err("manifest exceeds byte limit".into());
    }
    let stored: Value = serde_json::from_slice(&bytes)?;
    let revision = stored["source"]["revision"]
        .as_str()
        .ok_or("missing revision")?;
    if !git::oid(revision) {
        return Err("manifest revision must be exact".into());
    }
    let label = stored["source"]["selector"]
        .as_str()
        .ok_or("missing selector")?;
    let rebuilt = build(root, revision, label)?;
    if stored != rebuilt {
        return Err("manifest differs from independently rebuilt exact inventory".into());
    }
    let stale = if let Some(selector) = current_ref {
        git::resolve(root, selector)? != revision
    } else {
        false
    };
    println!(
        "{}",
        json!({"status":if stale {"stale"} else {"verified"},"revision":revision,"network_freshness":"not_verified","manifest_sha256":stored["manifest_sha256"]})
    );
    Ok(if stale { 3 } else { 0 })
}
