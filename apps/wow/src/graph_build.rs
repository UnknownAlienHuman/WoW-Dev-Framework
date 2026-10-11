//! Thin transport for the service-owned source graph build operation.
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use wow_service::graph::{
    FunctionFingerprintSelection, GRAPH_REQUEST_MAX_BYTES, GraphBuildRequest, GraphReadStatus,
    SourceGraphDiagnostics, execute_graph_build_diagnosed, source_graph_diagnostics_bytes,
};
use wow_service::{LocalProjectInput, ServiceErrorCode};

const HELP: &str = "wow graph build --config <project.json> --project <ProjectId> [--generation current|<ProjectGenerationId>] [--format json|snapshot|text] [--diagnostics] [--fingerprints <selection.json>]\n\nBuild the bounded first-party source/file-load graph using the same materialization path as wow check. JSON includes provenance; snapshot emits the bare artifact for graph subgraph/axis/explain/path. Partial source graphs exit 2. No files or ProjectStore pointers are written. --diagnostics emits one bounded source metadata sidecar on stderr, outside graph identities. --fingerprints selects an explicit service-owned function fingerprint query in JSON output; snapshot remains the bare graph artifact.\n";

#[derive(Clone, Copy)]
enum Format {
    Json,
    Snapshot,
    Text,
}
struct Arguments {
    config: PathBuf,
    fingerprints: Option<PathBuf>,
    request: GraphBuildRequest,
    format: Format,
    diagnostics: bool,
}

pub fn run(values: Vec<OsString>) -> u8 {
    if values.len() == 3 && (values[2] == "--help" || values[2] == "-h") {
        return match std::io::stdout().lock().write_all(HELP.as_bytes()) {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    let args = match parse(values) {
        Ok(args) => args,
        Err(e) => return super::usage(e),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let fingerprints = match args.fingerprints.as_deref() {
        Some(path) => {
            let bytes = match super::graph::read_file(path, GRAPH_REQUEST_MAX_BYTES, &stop) {
                Ok(bytes) => bytes,
                Err(message) => {
                    super::diagnostic(message);
                    return if stop.load(Ordering::Acquire) {
                        130
                    } else {
                        64
                    };
                }
            };
            let selection =
                match FunctionFingerprintSelection::from_json_slice_cancellable(&bytes, &stop) {
                    Ok(selection) => selection,
                    Err(error) => {
                        super::diagnostic(error.message());
                        return if stop.load(Ordering::Acquire)
                            || error.code() == ServiceErrorCode::Cancelled
                        {
                            130
                        } else {
                            64
                        };
                    }
                };
            if stop.load(Ordering::Acquire) {
                return 130;
            }
            Some(selection)
        }
        None => None,
    };
    let input = match LocalProjectInput::from_config_path(&args.config, &stop) {
        Ok(input) => input,
        Err(e) => {
            super::diagnostic(e.message());
            return if e.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                64
            };
        }
    };
    let request = args.request.with_input_profile(&input);
    let request = match fingerprints {
        Some(selection) => match request.with_function_fingerprints(selection) {
            Ok(request) => request,
            Err(error) => {
                super::diagnostic(error.message());
                return if stop.load(Ordering::Acquire)
                    || error.code() == ServiceErrorCode::Cancelled
                {
                    130
                } else {
                    64
                };
            }
        },
        None => request,
    };
    let mut diagnostics = if args.diagnostics {
        SourceGraphDiagnostics::enabled()
    } else {
        SourceGraphDiagnostics::default()
    };
    let result =
        execute_graph_build_diagnosed(input, &request, &stop, &mut diagnostics).and_then(|r| {
            if stop.load(Ordering::Acquire) {
                r.into_cancelled()
            } else {
                Ok(r)
            }
        });
    if args.diagnostics {
        emit_diagnostics(&diagnostics);
    }
    let result = match result {
        Ok(result) => result,
        Err(_) => {
            super::diagnostic("graph build result encoding failed");
            return 4;
        }
    };
    let exit = match result.status() {
        GraphReadStatus::Complete => 0,
        GraphReadStatus::Partial | GraphReadStatus::Truncated | GraphReadStatus::NotEvaluated => 2,
        GraphReadStatus::Failed => 3,
        GraphReadStatus::Cancelled => 130,
    };
    if let Some(message) = result.failure_message() {
        super::diagnostic(message);
    }
    let bytes = match args.format {
        Format::Snapshot => match result.snapshot_bytes() {
            Ok(Some(bytes)) => bytes,
            Ok(None) => {
                super::diagnostic(
                    "graph build did not produce a snapshot; no artifact was emitted",
                );
                return exit;
            }
            Err(_) => {
                super::diagnostic("graph snapshot encoding failed");
                return 4;
            }
        },
        Format::Json | Format::Text => match result.canonical_bytes() {
            Ok(mut bytes) => {
                if matches!(args.format, Format::Text) {
                    let header = format!("graph build: {:?}\n", result.status());
                    bytes.splice(0..0, header.bytes());
                }
                bytes.push(b'\n');
                bytes
            }
            Err(_) => {
                super::diagnostic("graph build result encoding failed");
                return 4;
            }
        },
    };
    // Do not emit a completed graph after a cancellation during serialization.
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    let mut stdout = std::io::stdout().lock();
    if let Err(e) = stdout.write_all(&bytes).and_then(|()| stdout.flush()) {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            super::diagnostic("graph build output failed");
        }
        return 4;
    }
    exit
}

// Diagnostic encoding/output is best effort and never changes owner status or retries.
fn emit_diagnostics(diagnostics: &SourceGraphDiagnostics) {
    let mut bytes = match source_graph_diagnostics_bytes(diagnostics) {
        Ok(bytes) => bytes,
        Err(_) => {
            super::diagnostic("graph diagnostics encoding failed");
            return;
        }
    };
    bytes.push(b'\n');
    let output = {
        let mut stderr = std::io::stderr().lock();
        stderr.write_all(&bytes).and_then(|()| stderr.flush())
    };
    if let Err(error) = output
        && error.kind() != std::io::ErrorKind::BrokenPipe
    {
        super::diagnostic("graph diagnostics output failed");
    }
}

fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    let total = values
        .iter()
        .try_fold(0usize, |n, v| n.checked_add(v.len()))
        .ok_or("argument size overflow")?;
    if values.len() > 4096 || total > 1024 * 1024 {
        return Err("argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(std::ffi::OsStr::new("graph"))
        || values.next().as_deref() != Some(std::ffi::OsStr::new("build"))
    {
        return Err("expected graph build");
    }
    let (mut config, mut project, mut generation, mut format) = (None, None, None, None);
    let mut fingerprints = None;
    let mut diagnostics = false;
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid option encoding")?;
        if option == "--diagnostics" {
            if diagnostics {
                return Err("duplicate --diagnostics");
            }
            diagnostics = true;
            continue;
        }
        if !matches!(
            option,
            "--config" | "--project" | "--generation" | "--format" | "--fingerprints"
        ) {
            return Err("unknown graph build option");
        }
        let value = values.next().ok_or("missing graph build option value")?;
        if value.is_empty() || value.len() > 32768 {
            return Err("invalid option length");
        }
        if option == "--fingerprints" {
            if value.to_str().is_some_and(|value| value.starts_with("--")) {
                return Err("missing graph build option value");
            }
            if fingerprints.replace(PathBuf::from(value)).is_some() {
                return Err("duplicate --fingerprints");
            }
            continue;
        }
        if option == "--config" {
            if config.replace(PathBuf::from(value)).is_some() {
                return Err("duplicate --config");
            }
            continue;
        }
        let value = value
            .into_string()
            .map_err(|_| "invalid option value encoding")?;
        if value.len() > 4096 || value.chars().any(char::is_control) {
            return Err("invalid option value");
        }
        match option {
            "--project" => {
                if project.replace(value).is_some() {
                    return Err("duplicate --project");
                }
            }
            "--generation" => {
                if generation.replace(value).is_some() {
                    return Err("duplicate --generation");
                }
            }
            "--format" => {
                if format.replace(value).is_some() {
                    return Err("duplicate --format");
                }
            }
            _ => return Err("unknown graph build option"),
        }
    }
    let format = match format.as_deref().unwrap_or("json") {
        "json" => Format::Json,
        "snapshot" => Format::Snapshot,
        "text" => Format::Text,
        _ => return Err("unknown graph build format"),
    };
    let request = GraphBuildRequest::new(
        project.ok_or("graph build requires --project")?,
        generation.unwrap_or_else(|| "current".into()),
    )
    .map_err(|_| "invalid graph build project or generation")?;
    Ok(Arguments {
        config: config.ok_or("graph build requires --config")?,
        fingerprints,
        request,
        format,
        diagnostics,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_arguments() -> Vec<OsString> {
        [
            "graph",
            "build",
            "--config",
            "project.json",
            "--project",
            "test-project",
        ]
        .into_iter()
        .map(OsString::from)
        .collect()
    }

    #[test]
    fn diagnostics_and_fingerprints_are_optional_flags() -> Result<(), &'static str> {
        let ordinary = parse(build_arguments())?;
        assert!(!ordinary.diagnostics);
        assert!(ordinary.fingerprints.is_none());
        let mut values = build_arguments();
        values.push("--diagnostics".into());
        assert!(parse(values)?.diagnostics);
        let mut values = build_arguments();
        values.extend(["--fingerprints", "selection.json", "--diagnostics"].map(OsString::from));
        let selected = parse(values)?;
        assert_eq!(selected.fingerprints, Some(PathBuf::from("selection.json")));
        assert!(selected.diagnostics);
        Ok(())
    }

    #[test]
    fn duplicate_or_missing_graph_build_options_are_refused() {
        let mut values = build_arguments();
        values.push("--diagnostics".into());
        values.push("--diagnostics".into());
        assert_eq!(parse(values).err(), Some("duplicate --diagnostics"));

        let mut values = build_arguments();
        values.extend(
            ["--fingerprints", "one.json", "--fingerprints", "two.json"].map(OsString::from),
        );
        assert_eq!(parse(values).err(), Some("duplicate --fingerprints"));

        let mut values = build_arguments();
        values.push("--fingerprints".into());
        assert_eq!(
            parse(values).err(),
            Some("missing graph build option value")
        );

        let mut values = build_arguments();
        values.extend(["--fingerprints", "--diagnostics"].map(OsString::from));
        assert_eq!(
            parse(values).err(),
            Some("missing graph build option value")
        );
    }
}
