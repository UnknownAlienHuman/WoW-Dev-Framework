use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use serde_json::json;
use wow_reference::ui_topology::{
    UiTopologyDocumentLookup, UiTopologyIndex, import_ui_topology_draft,
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
        "document" if arguments.len() == 3 => {
            let index = load_index(Path::new(&arguments[1]))?;
            let path = arguments[2]
                .to_str()
                .ok_or_else(|| "document path is not valid UTF-8".to_owned())?;
            print_document(&index, path)
        }
        "outgoing" if arguments.len() == 3 => {
            let index = load_index(Path::new(&arguments[1]))?;
            let path = arguments[2]
                .to_str()
                .ok_or_else(|| "source path is not valid UTF-8".to_owned())?;
            print_json(&json!({
                "status": "ok",
                "source": path,
                "edges": index.outgoing(path),
                "negative_authority": index.coverage().negative_authority(),
            }))
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
        "  wow-reference-topology verify <topology.json>",
        "  wow-reference-topology materialize <topology.json> <index.json>",
        "  wow-reference-topology document <topology.json> <Interface/path.toc|xml>",
        "  wow-reference-topology outgoing <topology.json> <Interface/path.toc|xml>",
    ]
    .join("\n")
}

fn load_index(path: &Path) -> Result<UiTopologyIndex, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    import_ui_topology_draft(&bytes).map_err(|error| {
        format!(
            "topology import failed ({:?}): {}",
            error.code(),
            error.message()
        )
    })
}

#[derive(Serialize)]
struct Summary<'a> {
    status: &'static str,
    index_id: &'a str,
    topology_sha256: &'a str,
    revision: &'a str,
    selector: Option<&'a str>,
    version: Option<&'a str>,
    coverage: &'static str,
    negative_authority: bool,
    descriptors: usize,
    xml_documents: usize,
    edges: usize,
    issues: usize,
    cycles: usize,
}

fn summary(index: &UiTopologyIndex) -> Summary<'_> {
    Summary {
        status: "ok",
        index_id: index.index_id(),
        topology_sha256: index.topology_sha256(),
        revision: index.provenance().revision(),
        selector: index.provenance().selector(),
        version: index.provenance().version(),
        coverage: match index.coverage().status() {
            wow_reference::ui_topology::UiTopologyCoverageStatus::Complete => "complete",
            wow_reference::ui_topology::UiTopologyCoverageStatus::Partial => "partial",
        },
        negative_authority: index.coverage().negative_authority(),
        descriptors: index.descriptors().len(),
        xml_documents: index.xml_documents().len(),
        edges: index.edges().len(),
        issues: index.issues().len(),
        cycles: index.include_cycles().len(),
    }
}

fn print_document(index: &UiTopologyIndex, path: &str) -> Result<(), String> {
    let result = match index.lookup_document(path) {
        UiTopologyDocumentLookup::Toc(descriptor) => json!({
            "status": "found",
            "kind": "toc",
            "document": descriptor,
        }),
        UiTopologyDocumentLookup::Xml(document) => json!({
            "status": "found",
            "kind": "xml",
            "document": document,
        }),
        UiTopologyDocumentLookup::AbsentAuthoritative => json!({
            "status": "absent_authoritative",
            "path": path,
        }),
        UiTopologyDocumentLookup::NotAuthoritative => json!({
            "status": "not_authoritative",
            "path": path,
        }),
        UiTopologyDocumentLookup::OutOfScope => json!({
            "status": "out_of_scope",
            "path": path,
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
    if path.exists() {
        return Err(format!(
            "refusing to replace existing output {}",
            path.display()
        ));
    }
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
        drop(file);
        fs::rename(&temporary, path).map_err(|error| {
            format!(
                "cannot atomically publish {} as {}: {error}",
                temporary.display(),
                path.display()
            )
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}
