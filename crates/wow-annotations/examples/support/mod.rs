//! Development driver for the documented native Ketho source-to-annotations lane.
//! Reads an exact local Git revision and its selected APIDocumentation TOC; never
//! executes source Lua or an external interpreter. This is not the public service-owned `wow` CLI.
use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_reference::native::{NativeError, ingest_document, source_digest};

const LIMIT: usize = 1024 * 1024;
const TOTAL_LIMIT: usize = 64 * LIMIT;
const USAGE: &str = "native_library <git-checkout> <revision-or-ref> <generated-api.toc> <environment> <new-output-directory> [--corrections <reviewed-pack.json>] [--alias-catalog <git-checkout> <revision-or-ref> <alias-resource.lua>]";

#[derive(Serialize)]
struct Failure {
    path: String,
    sha256: String,
    error: NativeError,
}

mod io;
use io::{git, validate_path, write_new};
#[cfg(test)]
mod tests;

pub fn run(
    args: Vec<OsString>,
    bridge: Option<&dyn wow_render_contract::LiteralBridge>,
) -> Result<bool, Box<dyn std::error::Error>> {
    if args.len() == 1 && args[0] == "--help" {
        println!("{USAGE}");
        return Ok(false);
    }
    if args.len() < 5 {
        return Err(USAGE.into());
    }
    let mut correction_path = None;
    let mut alias_input = None;
    let mut next = 5;
    while next < args.len() {
        if args[next] == "--corrections" && correction_path.is_none() && next + 1 < args.len() {
            correction_path = Some(Path::new(&args[next + 1]));
            next += 2;
        } else if args[next] == "--alias-catalog" && alias_input.is_none() && next + 3 < args.len()
        {
            alias_input = Some((&args[next + 1], &args[next + 2], &args[next + 3]));
            next += 4;
        } else {
            return Err(USAGE.into());
        }
    }
    let root = Path::new(&args[0]);
    let selector = args[1].to_str().ok_or("ref is not UTF-8")?;
    if selector.is_empty() || selector.starts_with('-') || selector.chars().any(char::is_control) {
        return Err("invalid source ref".into());
    }
    let resolved = git(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{selector}^{{commit}}"),
        ],
        128,
    )?;
    let revision = std::str::from_utf8(&resolved)?.trim();
    if !matches!(revision.len(), 40 | 64) || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid resolved revision".into());
    }
    let toc_path = args[2].to_str().ok_or("TOC path is not UTF-8")?;
    validate_path(toc_path)?;
    let environment = args[3].to_str().ok_or("environment is not UTF-8")?;
    if environment.is_empty() || environment.chars().any(char::is_control) {
        return Err("invalid source environment".into());
    }
    let toc_bytes = git(
        root,
        &["cat-file", "blob", &format!("{revision}:{toc_path}")],
        LIMIT,
    )?;
    let toc = std::str::from_utf8(&toc_bytes)?.trim_start_matches('\u{feff}');
    let parent = toc_path.rsplit_once('/').map_or("", |v| v.0);
    let mut paths = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for line in toc.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.replace('\\', "/");
        validate_path(&name)?;
        if !name.ends_with(".lua") {
            return Err("selected TOC requires an unsupported load-entry form".into());
        }
        let path = if parent.is_empty() {
            name
        } else {
            format!("{parent}/{name}")
        };
        if !seen.insert(path.clone()) {
            return Err("selected TOC contains duplicate input paths".into());
        }
        paths.push(path);
        if paths.len() > 4096 {
            return Err("selected TOC exceeds file limit".into());
        }
    }
    if paths.is_empty() {
        return Err("selected TOC has no Lua inputs".into());
    }
    let cancelled = AtomicBool::new(false);
    let mut documents = Vec::new();
    let mut failures = Vec::new();
    let mut total = 0usize;
    for path in &paths {
        // Git object access ignores dirty files, filters, export-ignore and substitution.
        let bytes = git(
            root,
            &["cat-file", "blob", &format!("{revision}:{path}")],
            LIMIT,
        )?;
        total = total.checked_add(bytes.len()).ok_or("source byte limit")?;
        if total > TOTAL_LIMIT {
            return Err("source corpus exceeds byte limit".into());
        }
        let sha256 = source_digest(&bytes);
        let source = std::str::from_utf8(&bytes)?;
        match ingest_document(revision, path, source, &sha256, &cancelled) {
            Ok(document) => documents.push(document),
            Err(error) => failures.push(Failure {
                path: path.clone(),
                sha256,
                error,
            }),
        }
    }
    if documents.is_empty() {
        return Err("no source registrations could be admitted".into());
    }
    let corrections = if let Some(path) = correction_path {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > 2 * LIMIT as u64
        {
            return Err("correction pack must be a bounded regular file".into());
        }
        let mut bytes = Vec::new();
        fs::File::open(path)?
            .take((2 * LIMIT + 1) as u64)
            .read_to_end(&mut bytes)?;
        Some(wow_reference::native_corrections::ValidatedCorrections::from_json(&bytes)?)
    } else {
        None
    };
    let alias_catalog = if let Some((checkout, selector, path)) = alias_input {
        let alias_root = Path::new(checkout);
        let selector = selector.to_str().ok_or("alias ref is not UTF-8")?;
        let path = path.to_str().ok_or("alias path is not UTF-8")?;
        validate_path(path)?;
        if selector.is_empty()
            || selector.starts_with('-')
            || selector.chars().any(char::is_control)
        {
            return Err("invalid alias source ref".into());
        }
        let resolved = git(
            alias_root,
            &[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{selector}^{{commit}}"),
            ],
            128,
        )?;
        let alias_revision = std::str::from_utf8(&resolved)?.trim();
        let entry = git(
            alias_root,
            &["ls-tree", "-z", alias_revision, "--", path],
            8192,
        )?;
        let entry = std::str::from_utf8(&entry)?;
        if !(entry.starts_with("100644 blob ") || entry.starts_with("100755 blob "))
            || !entry.ends_with(&format!("\t{path}\0"))
            || entry.matches('\0').count() != 1
        {
            return Err("alias resource must be an exact regular Git blob".into());
        }
        let bytes = git(
            alias_root,
            &["cat-file", "blob", &format!("{alias_revision}:{path}")],
            256 * 1024,
        )?;
        Some(wow_reference::native_aliases::ingest_alias_catalog(
            alias_revision,
            path,
            std::str::from_utf8(&bytes)?,
            &source_digest(&bytes),
            &cancelled,
        )?)
    } else {
        None
    };
    let library = wow_annotations::native::project_with_literal_bridge(
        &documents,
        environment,
        corrections.as_ref(),
        alias_catalog.as_ref(),
        bridge,
        &cancelled,
    )?;
    let partial = !failures.is_empty() || library.projection == "partial";
    let report = serde_json::json!({
        "schema": "wow-native-source-build/1", "revision": revision, "selector": selector,
        "freshness": "not_network_verified", "environment": environment,
        "toc": {"path":toc_path,"sha256":source_digest(&toc_bytes)}, "source_order":paths,
        "candidate_files":paths.len(), "admitted_files":documents.len(), "input_failures":failures,
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority":false, "library":library
    });
    // Refuse an existing output directory. Write only renderer-owned names under
    // a newly created private directory; a failed write is never complete output.
    // Publication/crash-recovery and multi-process root locking are higher owners.
    let report_bytes = serde_json::to_vec_pretty(&report)?;
    if report_bytes.len() > 512 * LIMIT {
        return Err("report exceeds byte limit".into());
    }
    let destination = Path::new(&args[4]);
    fs::create_dir(destination)?;
    for file in &library.files {
        write_new(&destination.join(&file.path), file.text.as_bytes())?;
    }
    write_new(&destination.join("source-report.json"), &report_bytes)?;
    println!(
        "{}",
        serde_json::json!({"revision":revision,"candidate_files":paths.len(),"admitted_files":documents.len(),"input_failures":failures.len(),"annotation_files":library.files.len(),"projection_issues":library.issues.len(),"status":if partial {"partial"} else {"projected_with_sidecars"},"negative_authority":false})
    );
    Ok(partial)
}
