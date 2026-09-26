//! Native artifact transport only. The service selects/validates the configured
//! project and returns exact bytes; this module neither infers facts nor writes files.
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use wow_service::local::native_artifact::{
    NATIVE_ARTIFACT_MAX_BYTES, NativeExportKind, NativeExportRequest, execute_native_export,
};
use wow_service::{LocalProjectInput, ServiceErrorCode};

const HELP: &str = "wow native report|artifact --config <project.json> --project <ProjectId> [--max-bytes <bytes>] [--expect-sha256 <sha256:...>]\n\nEmit exact native report or reusable input-artifact bytes on stdout. No newline is added. Exit 0 means output completed, not source, producer or consumer acceptance. No source/report file is written; use an explicit shell redirection to retain output. A failed or cancelled stream must be discarded.\n";

struct Arguments {
    config: PathBuf,
    request: NativeExportRequest,
}

pub fn run(values: Vec<OsString>) -> u8 {
    if values.len() == 2 && (values[1] == "--help" || values[1] == "-h") {
        let mut stdout = std::io::stdout().lock();
        return match stdout.write_all(HELP.as_bytes()).and_then(|()| stdout.flush()) {
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
    let input = match LocalProjectInput::from_config_path(&args.config, &stop) {
        Ok(input) => input,
        Err(error) => {
            super::diagnostic(error.message());
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                64
            };
        }
    };
    let artifact = match execute_native_export(input, &args.request, &stop) {
        Ok(artifact) => artifact,
        Err(error) => {
            super::diagnostic(error.message());
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                3
            };
        }
    };
    let mut stdout = std::io::stdout().lock();
    for chunk in artifact.bytes().chunks(64 * 1024) {
        if stop.load(Ordering::Acquire) {
            return 130;
        }
        if let Err(error) = stdout.write_all(chunk) {
            if error.kind() != std::io::ErrorKind::BrokenPipe {
                super::diagnostic("native artifact output failed");
            }
            return 4;
        }
    }
    if let Err(error) = stdout.flush() {
        if error.kind() != std::io::ErrorKind::BrokenPipe {
            super::diagnostic("native artifact flush failed");
        }
        return 4;
    }
    if stop.load(Ordering::Acquire) {
        130
    } else {
        0
    }
}

fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    let total = values
        .iter()
        .try_fold(0usize, |size, value| size.checked_add(value.len()))
        .ok_or("argument size overflow")?;
    if values.len() > 4096 || total > 1024 * 1024 {
        return Err("argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(OsStr::new("native")) {
        return Err("expected native command");
    }
    let kind = match values.next().as_deref() {
        Some(value) if value == "report" => NativeExportKind::Report,
        Some(value) if value == "artifact" => NativeExportKind::Artifact,
        _ => return Err("expected native report or native artifact"),
    };
    let (mut config, mut project, mut limit, mut expected) = (None, None, None, None);
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid native option encoding")?;
        if !matches!(
            option,
            "--config" | "--project" | "--max-bytes" | "--expect-sha256"
        ) {
            return Err("unknown native option");
        }
        let value = values.next().ok_or("missing native option value")?;
        if value.is_empty() || value.len() > 32768 {
            return Err("invalid option length");
        }
        if option == "--config" {
            if config.replace(PathBuf::from(value)).is_some() {
                return Err("duplicate --config");
            }
            continue;
        }
        let value = value
            .into_string()
            .map_err(|_| "invalid native option value encoding")?;
        if value.len() > 4096 || value.chars().any(char::is_control) {
            return Err("invalid native option value");
        }
        let slot = match option {
            "--project" => &mut project,
            "--max-bytes" => &mut limit,
            "--expect-sha256" => &mut expected,
            _ => return Err("unknown native option"),
        };
        if slot.replace(value).is_some() {
            return Err("duplicate native option");
        }
    }
    let limit = match limit {
        Some(value) if value.bytes().all(|b| b.is_ascii_digit()) => value
            .parse::<usize>()
            .map_err(|_| "invalid native byte limit")?,
        Some(_) => return Err("invalid native byte limit"),
        None => NATIVE_ARTIFACT_MAX_BYTES,
    };
    let request = NativeExportRequest::new(
        project.ok_or("native export requires --project")?,
        kind,
        limit,
        expected,
    )
    .map_err(|_| "invalid native export project, digest or budget")?;
    Ok(Arguments {
        config: config.ok_or("native export requires --config")?,
        request,
    })
}
