use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use serde_json::{Value, json};
use wow_reference::generated_api::{
    GeneratedApiFactKind, GeneratedApiIndex, GeneratedApiLookup, import_generated_api_draft,
};

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: Vec<OsString>) -> Result<(), String> {
    let Some(command) = arguments.first().and_then(|value| value.to_str()) else {
        return Err(usage());
    };
    match command {
        "verify" if arguments.len() == 2 => {
            let index = load_index(Path::new(&arguments[1]))?;
            print_json(&summary(&index))
        }
        "materialize" if arguments.len() == 3 => {
            let index = load_index(Path::new(&arguments[1]))?;
            write_json_atomic(Path::new(&arguments[2]), &index)?;
            print_json(&summary(&index))
        }
        "lookup" if arguments.len() == 4 => {
            let index = load_index(Path::new(&arguments[1]))?;
            let kind = parse_kind(&arguments[2])?;
            let qualified_name = arguments[3]
                .to_str()
                .ok_or_else(|| "lookup name is not valid UTF-8".to_owned())?;
            print_lookup(&index, kind, qualified_name)
        }
        "help" | "--help" | "-h" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    [
        "usage:",
        "  wow-reference-api verify <draft.json>",
        "  wow-reference-api materialize <draft.json> <index.json>",
        "  wow-reference-api lookup <draft.json> <kind> <qualified-name>",
        "",
        "kind: function | event | table | enumeration | constant | predicate",
    ]
    .join("\n")
}

fn load_index(path: &Path) -> Result<GeneratedApiIndex, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    import_generated_api_draft(&bytes).map_err(|error| {
        format!(
            "generated API import failed ({:?}): {}",
            error.code(),
            error.message()
        )
    })
}

fn parse_kind(value: &OsString) -> Result<GeneratedApiFactKind, String> {
    match value.to_str() {
        Some("function") => Ok(GeneratedApiFactKind::Function),
        Some("event") => Ok(GeneratedApiFactKind::Event),
        Some("table") => Ok(GeneratedApiFactKind::Table),
        Some("enumeration") => Ok(GeneratedApiFactKind::Enumeration),
        Some("constant") => Ok(GeneratedApiFactKind::Constant),
        Some("predicate") => Ok(GeneratedApiFactKind::Predicate),
        Some(other) => Err(format!("unknown generated API fact kind: {other}")),
        None => Err("generated API fact kind is not valid UTF-8".to_owned()),
    }
}

#[derive(Serialize)]
struct Summary<'a> {
    status: &'static str,
    index_id: &'a str,
    draft_sha256: &'a str,
    revision: &'a str,
    selector: Option<&'a str>,
    version: Option<&'a str>,
    coverage: &'static str,
    negative_authority: bool,
    candidate_files: u64,
    parsed_files: u64,
    failed_files: u64,
    facts: usize,
    conflicts: usize,
}

fn summary(index: &GeneratedApiIndex) -> Summary<'_> {
    let coverage = index.coverage();
    Summary {
        status: "ok",
        index_id: index.index_id(),
        draft_sha256: index.draft_sha256(),
        revision: index.provenance().revision(),
        selector: index.provenance().selector(),
        version: index.provenance().version(),
        coverage: match coverage.status() {
            wow_reference::generated_api::GeneratedApiCoverageStatus::Complete => "complete",
            wow_reference::generated_api::GeneratedApiCoverageStatus::Partial => "partial",
        },
        negative_authority: coverage.negative_authority(),
        candidate_files: coverage.candidate_files(),
        parsed_files: coverage.parsed_files(),
        failed_files: coverage.failed_files(),
        facts: index.facts().len(),
        conflicts: index.conflicts().len(),
    }
}

fn print_lookup(
    index: &GeneratedApiIndex,
    kind: GeneratedApiFactKind,
    qualified_name: &str,
) -> Result<(), String> {
    let result = match index.lookup(kind, qualified_name) {
        GeneratedApiLookup::Found(fact) => json!({
            "status": "found",
            "fact": fact,
        }),
        GeneratedApiLookup::Conflicted(facts) => json!({
            "status": "conflicted",
            "facts": facts,
        }),
        GeneratedApiLookup::AbsentAuthoritative => json!({
            "status": "absent_authoritative",
            "kind": kind,
            "qualified_name": qualified_name,
        }),
        GeneratedApiLookup::NotAuthoritative => json!({
            "status": "not_authoritative",
            "kind": kind,
            "qualified_name": qualified_name,
        }),
    };
    print_json(&result)
}

fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)
        .map_err(|error| format!("cannot serialize output: {error}"))?;
    handle
        .write_all(b"\n")
        .map_err(|error| format!("cannot write output: {error}"))
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|candidate| !candidate.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("output path {} has no file name", path.display()))?;
    let mut temporary_name = file_name.to_os_string();
    temporary_name.push(format!(".{}.tmp", std::process::id()));
    let temporary = parent.join(PathBuf::from(temporary_name));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("cannot create {}: {error}", temporary.display()))?;
        serde_json::to_writer_pretty(&mut file, value)
            .map_err(|error| format!("cannot serialize {}: {error}", temporary.display()))?;
        file.write_all(b"\n")
            .map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
        file.sync_all()
            .map_err(|error| format!("cannot sync {}: {error}", temporary.display()))?;
        fs::rename(&temporary, path).map_err(|error| {
            format!(
                "cannot atomically replace {} with {}: {error}",
                path.display(),
                temporary.display()
            )
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
