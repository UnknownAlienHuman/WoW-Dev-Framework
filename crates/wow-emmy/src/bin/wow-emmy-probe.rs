use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use serde_json::json;
use wow_emmy::compatibility::{EmmyCompatibilityErrorCode, backend_identity_from_report};

#[path = "../compatibility/limits.rs"]
mod compatibility_limits;
#[path = "wow-emmy-probe/host_input.rs"]
mod host_input;

#[path = "wow-emmy-probe/source.rs"]
mod source;

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
        "produce-backend" if arguments.len() == 2 => {
            let bytes = source::produce(Path::new(&arguments[1]))?;
            io::stdout()
                .lock()
                .write_all(&bytes)
                .map_err(|_| "cannot write probe report".to_owned())
        }
        "verify-backend" if arguments.len() == 2 => {
            let path = Path::new(&arguments[1]);
            // Bound host acquisition before the importer is reached. The owner
            // still owns every semantic validation of the bytes.
            let bytes = host_input::read_bounded_artifact(
                path,
                compatibility_limits::MAX_REPORT_BYTES,
            )
            .map_err(|error| match error {
                host_input::ReadError::TooLarge => format!(
                    "backend report import failed ({:?}): EmmyLua compatibility report exceeds the import limit",
                    EmmyCompatibilityErrorCode::InputTooLarge
                ),
                error => format!("cannot read {}: {error}", path.display()),
            })?;
            let identity = backend_identity_from_report(&bytes).map_err(|error| {
                format!(
                    "backend report import failed ({:?}): {}",
                    error.code(),
                    error.message()
                )
            })?;
            let output = json!({
                "status": "compatible",
                "crate_name": identity.crate_name(),
                "crate_version": identity.crate_version(),
                "revision": identity.revision(),
                "tree": identity.tree(),
                "surface_sha256": identity.surface_sha256(),
                "compatibility_report_sha256": identity.compatibility_report_sha256(),
            });
            print_json(&output)
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
        "  wow-emmy-probe produce-backend <exact-compiled-upstream-checkout>",
        "  wow-emmy-probe verify-backend <compatibility-report.json>",
    ]
    .join("\n")
}

fn print_json(value: &serde_json::Value) -> Result<(), String> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    serde_json::to_writer_pretty(&mut handle, value)
        .map_err(|error| format!("cannot serialize output: {error}"))?;
    handle
        .write_all(b"\n")
        .map_err(|error| format!("cannot write output: {error}"))
}
