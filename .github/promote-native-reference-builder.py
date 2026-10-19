from pathlib import Path

LIB = r'''#![forbid(unsafe_code)]

//! Native Blizzard API source-to-EmmyLua library generator.
//!
//! The application reads exact Git objects, projects them through `wow-reference`
//! and `wow-annotations`, and writes ordinary generated files. It owns no pack,
//! publication journal, installer, migration or compatibility lifecycle.

mod catalogs;
mod io;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_reference::native::{NativeError, ingest_document, source_digest};

use io::{git, validate_path, write_new};

const LIMIT: usize = 1024 * 1024;
const TOTAL_LIMIT: usize = 64 * LIMIT;
const USAGE: &str = "wow-reference-builder generate <git-checkout> <revision-or-ref> <generated-api.toc> <environment> <new-output-directory> [--corrections <reviewed-pack.json>] [--alias-catalog <git-checkout> <revision-or-ref> <alias-resource.lua>]...";

#[derive(Serialize)]
struct Failure {
    path: String,
    sha256: String,
    error: NativeError,
}

struct GenerateArgs {
    root: OsString,
    selector: OsString,
    toc: OsString,
    environment: OsString,
    output: OsString,
    corrections: Option<OsString>,
    aliases: Vec<(OsString, OsString, OsString)>,
}

pub fn run(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    run_with_stop(arguments, stdout, stderr, &AtomicBool::new(false))
}

pub fn run_with_stop(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    stop: &AtomicBool,
) -> i32 {
    let arguments = arguments.into_iter().map(OsString::from).collect::<Vec<_>>();
    if arguments.as_slice() == [OsStr::new("--help")]
        || arguments.as_slice() == [OsStr::new("generate"), OsStr::new("--help")]
    {
        let _ = writeln!(stdout, "{USAGE}");
        return 0;
    }
    let request = match parse(arguments) {
        Ok(request) => request,
        Err(message) => {
            let _ = writeln!(stderr, "{message}\nusage: {USAGE}");
            return 2;
        }
    };
    match generate(request, stdout, stop) {
        Ok(partial) => i32::from(partial) * 3,
        Err(error) => {
            let _ = writeln!(stderr, "generation_failed: {error}");
            if stop.load(Ordering::Acquire) { 6 } else { 5 }
        }
    }
}

fn parse(arguments: Vec<OsString>) -> Result<GenerateArgs, &'static str> {
    if arguments.len() < 6 || arguments[0] != "generate" {
        return Err("expected the generate command and five positional arguments");
    }
    let mut corrections = None;
    let mut aliases = Vec::new();
    let mut next = 6;
    while next < arguments.len() {
        if arguments[next] == "--corrections"
            && corrections.is_none()
            && next + 1 < arguments.len()
        {
            corrections = Some(arguments[next + 1].clone());
            next += 2;
        } else if arguments[next] == "--alias-catalog" && next + 3 < arguments.len() {
            if aliases.len() >= wow_annotations::aliases::MAX_CATALOG_FILES {
                return Err("alias catalog file limit exceeded");
            }
            aliases.push((
                arguments[next + 1].clone(),
                arguments[next + 2].clone(),
                arguments[next + 3].clone(),
            ));
            next += 4;
        } else {
            return Err("invalid generate option");
        }
    }
    Ok(GenerateArgs {
        root: arguments[1].clone(),
        selector: arguments[2].clone(),
        toc: arguments[3].clone(),
        environment: arguments[4].clone(),
        output: arguments[5].clone(),
        corrections,
        aliases,
    })
}

fn generate(
    request: GenerateArgs,
    stdout: &mut dyn Write,
    stop: &AtomicBool,
) -> Result<bool, Box<dyn std::error::Error>> {
    checkpoint(stop)?;
    let root = Path::new(&request.root);
    let selector = request.selector.to_str().ok_or("ref is not UTF-8")?;
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
    if !matches!(revision.len(), 40 | 64)
        || !revision.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err("invalid resolved revision".into());
    }

    let toc_path = request.toc.to_str().ok_or("TOC path is not UTF-8")?;
    validate_path(toc_path)?;
    let environment = request
        .environment
        .to_str()
        .ok_or("environment is not UTF-8")?;
    if environment.is_empty() || environment.chars().any(char::is_control) {
        return Err("invalid source environment".into());
    }

    let toc_bytes = git(
        root,
        &["cat-file", "blob", &format!("{revision}:{toc_path}")],
        LIMIT,
    )?;
    let toc = std::str::from_utf8(&toc_bytes)?.trim_start_matches('\u{feff}');
    let parent = toc_path.rsplit_once('/').map_or("", |value| value.0);
    let mut paths = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for line in toc.lines().map(str::trim) {
        checkpoint(stop)?;
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

    let mut documents = Vec::new();
    let mut failures = Vec::new();
    let mut total = 0usize;
    for path in &paths {
        checkpoint(stop)?;
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
        match ingest_document(
            revision,
            path,
            std::str::from_utf8(&bytes)?,
            &sha256,
            stop,
        ) {
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

    let corrections = read_corrections(request.corrections.as_deref())?;
    let alias_refs = request
        .aliases
        .iter()
        .map(|(root, revision, path)| (root.as_os_str(), revision.as_os_str(), path.as_os_str()))
        .collect::<Vec<_>>();
    let alias_catalogs = catalogs::read(&alias_refs, stop)?;
    let selected_catalogs = alias_catalogs.iter().collect::<Vec<_>>();
    let library = wow_annotations::native::project_with_alias_catalogs(
        &documents,
        environment,
        corrections.as_ref(),
        &selected_catalogs,
        stop,
    )?;
    let partial = !failures.is_empty() || library.projection == "partial";
    let report = serde_json::json!({
        "schema": "wow-native-source-build/1",
        "revision": revision,
        "selector": selector,
        "freshness": "not_network_verified",
        "environment": environment,
        "toc": {"path": toc_path, "sha256": source_digest(&toc_bytes)},
        "source_order": &paths,
        "candidate_files": paths.len(),
        "admitted_files": documents.len(),
        "input_failures": &failures,
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority": false,
        "library": &library,
    });
    let report_bytes = serde_json::to_vec_pretty(&report)?;
    if report_bytes.len() > 512 * LIMIT {
        return Err("report exceeds byte limit".into());
    }

    checkpoint(stop)?;
    let destination = Path::new(&request.output);
    fs::create_dir(destination)?;
    for file in &library.files {
        checkpoint(stop)?;
        write_new(&destination.join(&file.path), file.text.as_bytes())?;
    }
    write_new(&destination.join("source-report.json"), &report_bytes)?;

    let summary = serde_json::json!({
        "revision": revision,
        "candidate_files": paths.len(),
        "admitted_files": documents.len(),
        "input_failures": failures.len(),
        "annotation_files": library.files.len(),
        "projection_issues": library.issues.len(),
        "status": if partial {"partial"} else {"projected_with_sidecars"},
        "negative_authority": false,
    });
    serde_json::to_writer(&mut *stdout, &summary)?;
    stdout.write_all(b"\n")?;
    Ok(partial)
}

fn read_corrections(
    path: Option<&OsStr>,
) -> Result<Option<wow_reference::native_corrections::ValidatedCorrections>, Box<dyn std::error::Error>> {
    let Some(path) = path else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    let metadata = fs::symlink_metadata(&path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 2 * LIMIT as u64 {
        return Err("correction pack must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take((2 * LIMIT + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(Some(
        wow_reference::native_corrections::ValidatedCorrections::from_json(&bytes)?,
    ))
}

fn checkpoint(stop: &AtomicBool) -> Result<(), Box<dyn std::error::Error>> {
    if stop.load(Ordering::Acquire) {
        Err("generation cancelled".into())
    } else {
        Ok(())
    }
}
'''

CARGO = '''[package]
name = "wow-reference-builder"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
publish = false

[dependencies]
ctrlc = "3.5"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
wow-annotations = { path = "../../crates/wow-annotations" }
wow-reference = { path = "../../crates/wow-reference" }

[lints]
workspace = true
'''

README = '''# `wow-reference-builder`

Supported native Blizzard API annotation generator. It reads exact objects from
a local `wow-ui-source` Git revision, ingests APIDocumentation through
`wow-reference`, projects Ketho-compatible EmmyLua files through
`wow-annotations`, and writes an ordinary new directory.

```text
wow-reference-builder generate <checkout> <ref> <GeneratedApi.toc> \
  <environment> <new-output-directory> \
  [--corrections <reviewed.json>] \
  [--alias-catalog <checkout> <ref> <resource.lua>]...
```

The destination must not exist. Output is generated `.lua` files plus
`source-report.json`; exit `3` means useful partial output, `0` means complete
projection, `2` is usage, `5` is generation failure, and `6` is cancellation.
The command reads committed Git blobs and never executes donor Lua.

There is no pack, signing, publication journal, validation/rebuild-compare
lifecycle, installer, migration path or compatibility command. Generated output
is derived and may be rebuilt from source, corrections and alias catalogs.
'''

AGENTS = '''# `wow-reference-builder`

Follow root `AGENTS.md`, `docs/ARCHITECTURE.md`, and
`docs/IMPLEMENTATION.md#native-annotation-generation`.

This app owns only command parsing, bounded local Git reads and writing a new
output directory. `wow-reference` owns source facts; `wow-annotations` owns
projection and rendering. Keep the single `generate` workflow direct.

Do not add pack manifests, operation journals, signatures, promotion states,
backup/restore, installers, compatibility commands or another parser. Preserve
explicit revision/TOC/environment selection, non-execution of donor Lua,
cancellation, finite input/output bounds, partial-result reporting and new-only
output.
'''


def write(path: str, content: str) -> None:
    Path(path).write_text(content)


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if old not in text:
        raise SystemExit(f"missing text in {path}: {old[:80]!r}")
    file.write_text(text.replace(old, new))


write("apps/wow-reference-builder/src/lib.rs", LIB)
write("apps/wow-reference-builder/Cargo.toml", CARGO)
write("apps/wow-reference-builder/README.md", README)
write("apps/wow-reference-builder/AGENTS.md", AGENTS)

# Switch every documented/automated driver invocation to the supported app.
replacements = {
    "cargo +stable run --locked -p wow-annotations --example native_library --":
        "cargo +stable run --locked -p wow-reference-builder -- generate",
    "cargo run --locked -p wow-annotations --example native_library --":
        "cargo run --locked -p wow-reference-builder -- generate",
    "cargo run -p wow-annotations --example native_library --":
        "cargo run -p wow-reference-builder -- generate",
}
for file in Path(".").rglob("*"):
    if not file.is_file() or file.suffix not in {".md", ".yml", ".yaml"}:
        continue
    text = file.read_text()
    updated = text
    for old, new in replacements.items():
        updated = updated.replace(old, new)
    if updated != text:
        file.write_text(updated)

replace(
    "crates/wow-annotations/README.md",
    "The development example is\n`examples/native_library.rs`; #196 will expose that native path through a\nsupported command and remove remaining pack administration.",
    "The supported source-to-directory command is `wow-reference-builder generate`;\nthis crate remains the projection owner and has no filesystem/package lifecycle.",
)
replace(
    "docs/IMPLEMENTATION.md",
    "**Start #196 in `apps/wow-reference-builder/src/lib.rs`.** Promote the reusable\ndriver flow from `wow-annotations/examples/support/mod.rs` into the supported\ncommand. The example is development tooling; `apps/wow/src/native.rs` currently\nexports prepared bytes and is not a source-to-library generation command.",
    "**`apps/wow-reference-builder` now owns the supported `generate` command.** It\nreads an exact local Git revision/TOC and writes ordinary annotation files plus a\nsource report. `apps/wow/src/native.rs` remains the prepared-input path used by\nlocal analysis, not a second generator.",
)
replace(
    "docs/IMPLEMENTATION.md",
    "4. Uncalled `annotation_admin`, `reference_admin` and `reference_pack_admin`\n   layers are removed. Replace the still-live builder `reference_pack` and\n   `reference_pack_materialization` path with ordinary generated files. Native\n   generation must not require a pack registry or signed report.",
    "4. Uncalled admin layers and the builder pack/materialization CLI are removed.\n   Delete their now-unconsumed Service implementation and retained Lua copy.\n   Native generation requires no pack registry or signed report.",
)
replace(
    "docs/REFACTOR_QUEUE.md",
    "| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Uncalled admin journals are removed; replace builder pack/materialization and its exclusive retained Lua copy with generated files. |",
    "| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported native directory generation is live; delete unconsumed Service pack/materialization and its retained Lua copy. |",
)
