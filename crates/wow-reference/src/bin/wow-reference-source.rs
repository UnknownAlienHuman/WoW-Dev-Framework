use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_reference::generated_api::{
    GeneratedApiCoverageStatus, GeneratedApiIndex, import_generated_api_draft,
};
use wow_reference::ui_topology::{
    UiTopologyCoverageStatus, UiTopologyIndex, import_ui_topology_draft,
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
        "verify" if arguments.len() == 3 => {
            let api = load_api(Path::new(&arguments[1]))?;
            let topology = load_topology(Path::new(&arguments[2]))?;
            let bundle = build_bundle(&api, &topology)?;
            print_json(&bundle)
        }
        "materialize" if arguments.len() == 4 => {
            let api = load_api(Path::new(&arguments[1]))?;
            let topology = load_topology(Path::new(&arguments[2]))?;
            let bundle = build_bundle(&api, &topology)?;
            write_json_atomic(Path::new(&arguments[3]), &bundle)?;
            print_json(&bundle)
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
        "  wow-reference-source verify <api-reference.json> <ui-topology.json>",
        "  wow-reference-source materialize <api-reference.json> <ui-topology.json> <bundle.json>",
    ]
    .join("\n")
}

fn load_api(path: &Path) -> Result<GeneratedApiIndex, String> {
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

fn load_topology(path: &Path) -> Result<UiTopologyIndex, String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    import_ui_topology_draft(&bytes).map_err(|error| {
        format!(
            "UI topology import failed ({:?}): {}",
            error.code(),
            error.message()
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct SourceIdentity {
    manifest_sha256: String,
    source_id: Option<String>,
    selector: Option<String>,
    revision: String,
    version: Option<String>,
}

impl SourceIdentity {
    fn from_api(index: &GeneratedApiIndex) -> Self {
        let provenance = index.provenance();
        Self {
            manifest_sha256: provenance.source_manifest_sha256().to_owned(),
            source_id: provenance.source_id().map(str::to_owned),
            selector: provenance.selector().map(str::to_owned),
            revision: provenance.revision().to_owned(),
            version: provenance.version().map(str::to_owned),
        }
    }

    fn from_topology(index: &UiTopologyIndex) -> Self {
        let provenance = index.provenance();
        Self {
            manifest_sha256: provenance.manifest_sha256().to_owned(),
            source_id: provenance.source_id().map(str::to_owned),
            selector: provenance.selector().map(str::to_owned),
            revision: provenance.revision().to_owned(),
            version: provenance.version().map(str::to_owned),
        }
    }
}

#[derive(Serialize)]
struct ProductIdentity<'a> {
    id: &'a str,
    draft_sha256: &'a str,
    coverage: &'static str,
    negative_authority: bool,
}

#[derive(Serialize)]
struct BundleProjection<'a> {
    schema: &'static str,
    schema_version: u64,
    source: &'a SourceIdentity,
    generated_api: ProductIdentity<'a>,
    ui_topology: ProductIdentity<'a>,
}

fn build_bundle(api: &GeneratedApiIndex, topology: &UiTopologyIndex) -> Result<Value, String> {
    let api_source = SourceIdentity::from_api(api);
    let topology_source = SourceIdentity::from_topology(topology);
    if api_source != topology_source {
        return Err(format!(
            "source products belong to different generations: API={} topology={}",
            api_source.revision, topology_source.revision
        ));
    }
    let projection = BundleProjection {
        schema: "wow-dev-framework/current-source-reference-bundle",
        schema_version: 1,
        source: &api_source,
        generated_api: ProductIdentity {
            id: api.generation_id(),
            draft_sha256: api.draft_sha256(),
            coverage: match api.coverage().status() {
                GeneratedApiCoverageStatus::Complete => "complete",
                GeneratedApiCoverageStatus::Partial => "partial",
            },
            negative_authority: api.coverage().negative_authority(),
        },
        ui_topology: ProductIdentity {
            id: topology.index_id(),
            draft_sha256: topology.topology_sha256(),
            coverage: match topology.coverage().status() {
                UiTopologyCoverageStatus::Complete => "complete",
                UiTopologyCoverageStatus::Partial => "partial",
            },
            negative_authority: topology.coverage().negative_authority(),
        },
    };
    let canonical = canonical_json_bytes(&projection)
        .map_err(|error| format!("cannot canonicalize source bundle: {error}"))?;
    let bundle_sha256 = sha256(&canonical);
    let mut value = serde_json::to_value(projection)
        .map_err(|error| format!("cannot serialize source bundle: {error}"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "source bundle projection is not an object".to_owned())?;
    object.insert("bundle_sha256".to_owned(), Value::String(bundle_sha256));
    Ok(value)
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(revision: &str) -> SourceIdentity {
        SourceIdentity {
            manifest_sha256: "sha256:1111111111111111111111111111111111111111111111111111111111111111"
                .to_owned(),
            source_id: Some("public-source".to_owned()),
            selector: Some("live".to_owned()),
            revision: revision.to_owned(),
            version: Some("99.1.2.34567".to_owned()),
        }
    }

    #[test]
    fn source_identity_is_exact_and_generation_sensitive() {
        assert_eq!(
            identity("2222222222222222222222222222222222222222"),
            identity("2222222222222222222222222222222222222222")
        );
        assert_ne!(
            identity("2222222222222222222222222222222222222222"),
            identity("3333333333333333333333333333333333333333")
        );
    }

    #[test]
    fn source_identity_includes_manifest_and_selector() {
        let first = identity("2222222222222222222222222222222222222222");
        let mut changed = first.clone();
        changed.selector = Some("ptr".to_owned());
        assert_ne!(first, changed);

        let mut changed = first.clone();
        changed.manifest_sha256 =
            "sha256:4444444444444444444444444444444444444444444444444444444444444444"
                .to_owned();
        assert_ne!(first, changed);
    }
}
