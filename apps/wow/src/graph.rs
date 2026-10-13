//! Explicit graph-artifact transport. All graph semantics belong to the service.
#[cfg(unix)]
use cap_fs_ext::OpenOptionsSyncExt;
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::args::Format;
use wow_service::ServiceErrorCode;
use wow_service::graph::{
    GRAPH_BUNDLE_MAX_BYTES, GRAPH_INPUT_MAX_BYTES, GRAPH_REQUEST_MAX_BYTES, GraphReadOperation,
    GraphReadResult, GraphReadStatus, execute_graph_bundle_read, execute_graph_bundle_source_read,
    execute_graph_read, execute_graph_store_read,
};

pub const HELP: &str = "wow graph build --config <project.json> --project <ProjectId> [--format json|snapshot|text]\nwow graph publish --bundle <build.json> --store-root <private-directory> --operation-id <id> --expected-current absent|<record-id> --allow-partial [--initialize] [--format json|text]\nwow graph reconcile --store-root <private-directory> --operation-id <id> [--format json|text]\nwow graph entity|neighbors|subgraph|axis|explain|path (--snapshot <snapshot.json> | --bundle <build.json> | --store-root <directory> --store-generation current|<generation-id>) --request <query.json> [--format json|text] [--source-root <Main-root> (explain only)]\n\nPublish explicitly saves a validated v7 graph-build bundle as immutable manifested partitions, verifies fresh read-back and switches current only by exact CAS. --initialize authorizes a new private directory, not adoption of an unrelated database. --allow-partial acknowledges retained metadata, not full E2 acceptance. Reconcile observes one operation without repeating it. Store reads pin one exact generation; other reads never select current. Reads do not run source analysis. Source files are opened only by explain with --source-root. See apps/wow/GRAPH_STORE.md.\n";

struct Arguments {
    operation: GraphReadOperation,
    artifact: Artifact,
    request: PathBuf,
    format: Format,
    source_root: Option<PathBuf>,
}

enum Artifact {
    Snapshot(PathBuf),
    Bundle(PathBuf),
    Store { root: PathBuf, generation: String },
}

pub fn run(values: Vec<OsString>) -> u8 {
    if values
        .get(1)
        .is_some_and(|value| value == "publish" || value == "reconcile")
    {
        return super::graph_store::run(values);
    }
    if values.get(1).is_some_and(|value| value == "build") {
        return super::graph_build::run(values);
    }
    if values.len() == 2 && (values[1] == "--help" || values[1] == "-h") {
        return match std::io::stdout().lock().write_all(HELP.as_bytes()) {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    let args = match parse(values) {
        Ok(args) => args,
        Err(message) => return super::usage(message),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let request = match read_file(&args.request, GRAPH_REQUEST_MAX_BYTES, &stop) {
        Ok(bytes) => bytes,
        Err(message) => return read_failure(message, &stop),
    };
    let artifact = match &args.artifact {
        Artifact::Snapshot(path) => read_file(path, GRAPH_INPUT_MAX_BYTES, &stop),
        Artifact::Bundle(path) => read_file(path, GRAPH_BUNDLE_MAX_BYTES, &stop),
        Artifact::Store { .. } => Ok(Vec::new()),
    };
    let artifact = match artifact {
        Ok(bytes) => bytes,
        Err(message) => return read_failure(message, &stop),
    };
    let result = match args.artifact {
        Artifact::Snapshot(_) => execute_graph_read(args.operation, &artifact, &request, &stop),
        Artifact::Bundle(_) => match args.source_root.as_deref() {
            Some(root) => {
                execute_graph_bundle_source_read(args.operation, &artifact, &request, root, &stop)
            }
            None => execute_graph_bundle_read(args.operation, &artifact, &request, &stop),
        },
        Artifact::Store { root, generation } => execute_graph_store_read(
            args.operation,
            &root,
            &generation,
            &request,
            args.source_root.as_deref(),
            &stop,
        ),
    }
    .and_then(|result| {
        if stop.load(Ordering::Acquire) {
            result.into_cancelled()
        } else {
            Ok(result)
        }
    });
    let result = match result {
        Ok(result) => result,
        Err(error) => {
            super::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    let exit = exit_code(&result);
    let bytes = match render(&result, args.format) {
        Ok(bytes) => bytes,
        Err(()) => {
            super::diagnostic("graph output byte limit or encoding failure");
            return 4;
        }
    };
    // One write of a completed result; a broken pipe never reruns the operation.
    let mut stdout = std::io::stdout().lock();
    if let Err(error) = stdout.write_all(&bytes).and_then(|()| stdout.flush()) {
        if error.kind() != std::io::ErrorKind::BrokenPipe {
            super::diagnostic("graph output failed");
        }
        return 4;
    }
    exit
}

fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    let total = values
        .iter()
        .try_fold(0usize, |n, value| n.checked_add(value.len()))
        .ok_or("argument size overflow")?;
    if values.len() > 4096 || total > 1024 * 1024 {
        return Err("argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(std::ffi::OsStr::new("graph")) {
        return Err("missing graph command");
    }
    let operation = match values.next().as_deref().and_then(|v| v.to_str()) {
        Some("subgraph") => GraphReadOperation::Subgraph,
        Some("axis") => GraphReadOperation::Axis,
        Some("explain") => GraphReadOperation::Explain,
        Some("path") => GraphReadOperation::Path,
        Some("entity") => GraphReadOperation::Entity,
        Some("neighbors") => GraphReadOperation::Neighbors,
        _ => return Err("expected graph entity, neighbors, subgraph, axis, explain or path"),
    };
    let (mut artifact, mut request, mut format, mut source_root) = (None, None, None, None);
    let (mut store_root, mut store_generation) = (None, None);
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid option encoding")?;
        if !matches!(
            option,
            "--snapshot"
                | "--bundle"
                | "--request"
                | "--format"
                | "--source-root"
                | "--store-root"
                | "--store-generation"
        ) {
            return Err("unknown graph option");
        }
        let value = values.next().ok_or("missing graph option value")?;
        if value.is_empty() || value.len() > 32768 {
            return Err("invalid graph option length");
        }
        match option {
            "--store-root" => {
                if store_root.replace(PathBuf::from(value)).is_some() {
                    return Err("duplicate --store-root");
                }
            }
            "--store-generation" => {
                let value = value
                    .into_string()
                    .map_err(|_| "invalid generation encoding")?;
                if store_generation.replace(value).is_some() {
                    return Err("duplicate --store-generation");
                }
            }
            "--source-root" => {
                if source_root.replace(PathBuf::from(value)).is_some() {
                    return Err("duplicate --source-root");
                }
            }
            "--snapshot" | "--bundle" => {
                let input = if option == "--bundle" {
                    Artifact::Bundle(PathBuf::from(value))
                } else {
                    Artifact::Snapshot(PathBuf::from(value))
                };
                if artifact.replace(input).is_some() {
                    return Err("provide exactly one --snapshot or --bundle");
                }
            }
            "--request" => {
                if request.replace(PathBuf::from(value)).is_some() {
                    return Err("duplicate --request");
                }
            }
            "--format" => {
                let value = match value.to_str() {
                    Some("json") => Format::Json,
                    Some("text") => Format::Text,
                    _ => return Err("unknown graph output format"),
                };
                if format.replace(value).is_some() {
                    return Err("duplicate --format");
                }
            }
            _ => return Err("unknown graph option"),
        }
    }
    if store_root.is_some() || store_generation.is_some() {
        if artifact.is_some() {
            return Err("store input excludes --snapshot and --bundle");
        }
        artifact = Some(Artifact::Store {
            root: store_root.ok_or("--store-generation requires --store-root")?,
            generation: store_generation
                .ok_or("--store-root requires explicit --store-generation")?,
        });
    }
    let artifact = artifact.ok_or("graph requires --snapshot, --bundle or explicit store input")?;
    if source_root.is_some()
        && (operation != GraphReadOperation::Explain || matches!(&artifact, Artifact::Snapshot(_)))
    {
        return Err("--source-root requires graph explain with bundle or store input");
    }
    Ok(Arguments {
        operation,
        artifact,
        request: request.ok_or("graph requires --request")?,
        format: format.unwrap_or(Format::Json),
        source_root,
    })
}

pub(super) fn read_file(
    path: &Path,
    limit: usize,
    stop: &AtomicBool,
) -> Result<Vec<u8>, &'static str> {
    let checkpoint = || {
        if stop.load(Ordering::Acquire) {
            Err("graph input cancelled")
        } else {
            Ok(())
        }
    };
    checkpoint()?;
    let name = path.file_name().ok_or("graph input must name one file")?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    // Hold the selected parent and reject a final-component link at open. A
    // separate pathname inspection would race a replacement before File::open.
    let directory = Dir::open_ambient_dir(parent, cap_std::ambient_authority())
        .map_err(|_| "cannot open graph input parent")?;
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    // A raced FIFO must fail the opened-handle regular-file check without
    // blocking the open before cancellation can be observed.
    #[cfg(unix)]
    options.nonblock(true);
    let mut file = directory
        .open_with(name, &options)
        .map_err(|_| "cannot open regular graph input")?;
    let metadata = file
        .metadata()
        .map_err(|_| "cannot inspect opened graph input")?;
    if !metadata.file_type().is_file() || metadata.len() > limit as u64 {
        return Err("graph input type or byte limit");
    }
    let mut bytes = Vec::with_capacity((metadata.len() as usize).min(64 * 1024));
    let mut buffer = [0u8; 8192];
    loop {
        checkpoint()?;
        // Read no more than one byte beyond the cap, including files that grow.
        let available = buffer.len().min(limit - bytes.len() + 1);
        let count = match file.read(&mut buffer[..available]) {
            Ok(count) => count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("cannot read graph input"),
        };
        if count == 0 {
            break;
        }
        if bytes.len() + count > limit {
            return Err("graph input byte limit exceeded");
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    checkpoint()?;
    Ok(bytes)
}
fn read_failure(message: &'static str, stop: &AtomicBool) -> u8 {
    if stop.load(Ordering::Acquire) {
        130
    } else {
        super::usage(message)
    }
}
fn exit_code(result: &GraphReadResult) -> u8 {
    match result.status() {
        GraphReadStatus::Complete => 0,
        GraphReadStatus::Partial | GraphReadStatus::NotEvaluated | GraphReadStatus::Truncated => 2,
        GraphReadStatus::Cancelled => 130,
        GraphReadStatus::Failed => match result.failure().map(|f| f.code) {
            Some(
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed,
            ) => 4,
            _ => 3,
        },
    }
}
fn render(result: &GraphReadResult, format: Format) -> Result<Vec<u8>, ()> {
    if format == Format::Json {
        let mut bytes = result.canonical_bytes().map_err(|_| ())?;
        bytes.push(b'\n');
        return Ok(bytes);
    }
    struct Output(Vec<u8>);
    impl Write for Output {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.0.len().saturating_add(bytes.len()) > 32 * 1024 * 1024 {
                return Err(std::io::Error::other("graph text output limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut output = Output(Vec::new());
    writeln!(
        output,
        "graph {:?}: {:?}",
        result.operation(),
        result.status()
    )
    .map_err(|_| ())?;
    // Preserve the entire service receipt, including evidence, partial coverage,
    // all truncation reasons and explanation boundaries. JSON escapes source text.
    serde_json::to_writer_pretty(&mut output, result).map_err(|_| ())?;
    output.write_all(b"\n").map_err(|_| ())?;
    Ok(output.0)
}

#[cfg(test)]
mod read_file_tests {
    use super::read_file;
    use std::{
        fs, io,
        path::PathBuf,
        sync::atomic::{AtomicBool, AtomicU64, Ordering},
    };

    const CONTENT: &[u8] = b"bounded graph input\n";
    static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

    enum CaseOutcome {
        EvaluatedPass,
        #[cfg(not(unix))]
        NotEvaluated(String),
    }

    struct TempTree(PathBuf);

    impl TempTree {
        fn create() -> io::Result<Self> {
            let base = std::env::temp_dir();
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            let path = base.join(format!(
                "wow-cli-graph-read-file-nofollow-{}-{timestamp}-{id}",
                std::process::id(),
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn run_regular_and_leaf_symlink_case() -> Result<CaseOutcome, String> {
        let tree = TempTree::create().map_err(|error| format!("create temp tree: {error}"))?;
        let sentinel = tree.0.join("sentinel.bin");
        fs::write(&sentinel, CONTENT).map_err(|error| format!("write sentinel: {error}"))?;

        let selected_parent = tree.0.join("selected");
        fs::create_dir(&selected_parent)
            .map_err(|error| format!("create selected parent: {error}"))?;
        let link = selected_parent.join("input.bin");

        let stop = AtomicBool::new(false);
        let actual = read_file(&sentinel, CONTENT.len(), &stop)
            .map_err(|error| format!("regular-file read refused: {error}"))?;
        if actual != CONTENT {
            return Err("regular-file read returned different bytes".into());
        }

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&sentinel, &link)
                .map_err(|error| format!("create file symlink: {error:?}"))?;
        }
        #[cfg(windows)]
        {
            if let Err(error) = std::os::windows::fs::symlink_file(&sentinel, &link) {
                if matches!(
                    error.kind(),
                    io::ErrorKind::PermissionDenied | io::ErrorKind::Unsupported
                ) {
                    return Ok(CaseOutcome::NotEvaluated(format!(
                        "Windows could not create the file symlink ({error:?})"
                    )));
                }
                return Err(format!("create file symlink: {error:?}"));
            }
        }
        #[cfg(not(any(unix, windows)))]
        return Ok(CaseOutcome::NotEvaluated(
            "this platform has no configured file-symlink API".into(),
        ));

        if read_file(&link, CONTENT.len(), &stop).is_ok() {
            return Err("read_file followed a final-component symlink".into());
        }

        Ok(CaseOutcome::EvaluatedPass)
    }

    #[test]
    fn read_file_reads_regular_files_and_refuses_leaf_symlinks() -> Result<(), String> {
        match run_regular_and_leaf_symlink_case() {
            Ok(CaseOutcome::EvaluatedPass) => Ok(()),
            #[cfg(not(unix))]
            Ok(CaseOutcome::NotEvaluated(reason)) => {
                eprintln!("NOT_EVALUATED: {reason}");
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}
