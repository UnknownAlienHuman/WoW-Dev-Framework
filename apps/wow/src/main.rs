#![forbid(unsafe_code)]
mod args;
mod output;

use std::fs::File;
use std::io::{Read, Write};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_service::{LOCAL_INPUT_MAX_BYTES, LocalOperationResult, LocalProjectInput, execute_local};

fn main() -> ExitCode {
    ExitCode::from(run())
}

fn run() -> u8 {
    let arguments: Vec<_> = std::env::args_os().skip(1).take(4097).collect();
    if arguments.len() == 1 && (arguments[0] == "--help" || arguments[0] == "-h") {
        return match std::io::stdout().lock().write_all(args::HELP.as_bytes()) {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    if arguments.len() == 1 && arguments[0] == "--version" {
        let text = concat!("wow ", env!("CARGO_PKG_VERSION"), "\n");
        return match std::io::stdout().lock().write_all(text.as_bytes()) {
            Ok(()) => 0,
            Err(_) => 4,
        };
    }
    let arguments = match args::parse(arguments) {
        Ok(value) => value,
        Err(message) => return usage(message),
    };
    let stop = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&stop);
    if ctrlc::set_handler(move || signal.store(true, Ordering::Release)).is_err() {
        return usage("could not initialize cancellation handling");
    }
    // Reject ordinary special files before open; the opened handle is checked again.
    if !std::fs::symlink_metadata(&arguments.config).is_ok_and(|metadata| metadata.is_file()) {
        return usage("configuration must be a regular file, not a symlink");
    }
    let file = match File::open(&arguments.config) {
        Ok(file) => file,
        Err(_) => return usage("could not open explicit configuration"),
    };
    if !file.metadata().is_ok_and(|metadata| metadata.is_file()) {
        return usage("configuration must be a regular file");
    }
    let mut bytes = Vec::new();
    // Read at most limit+1, regardless of metadata or file growth during acquisition.
    if file
        .take((LOCAL_INPUT_MAX_BYTES as u64) + 1)
        .read_to_end(&mut bytes)
        .is_err()
    {
        return usage("configuration read failed");
    }
    if bytes.len() > LOCAL_INPUT_MAX_BYTES {
        return usage("configuration byte limit exceeded");
    }
    let result = if stop.load(Ordering::Acquire) {
        LocalOperationResult::cancelled(&arguments.command)
    } else {
        let input = match LocalProjectInput::from_json_slice(&bytes) {
            Ok(input) => input,
            Err(_) => return usage("configuration or exact input artifacts were rejected"),
        };
        drop(bytes);
        execute_local(input, &arguments.command, &stop)
    };
    // Cancellation may replace an unpublished result, never bytes already being written.
    let result = if stop.load(Ordering::Acquire) {
        LocalOperationResult::cancelled(&arguments.command)
    } else {
        result
    };
    let exit = output::exit_code(&result);
    let bytes = match output::render(&result, arguments.format, arguments.capabilities) {
        Ok(bytes) => bytes,
        Err(()) => {
            diagnostic("result encoding failed");
            return 4;
        }
    };
    let mut stdout = std::io::stdout().lock();
    if let Err(error) = stdout.write_all(&bytes).and_then(|()| stdout.flush()) {
        if error.kind() != std::io::ErrorKind::BrokenPipe {
            diagnostic("result output failed");
        }
        return 4;
    }
    exit
}

fn usage(message: &'static str) -> u8 {
    diagnostic(message);
    64
}
fn diagnostic(message: &str) {
    let _ = writeln!(std::io::stderr().lock(), "wow: {message}");
}
