#![forbid(unsafe_code)]
mod args;
mod graph;
mod graph_build;
mod graph_store;
mod native;
mod output;

use std::io::Write;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_service::{LocalOperationResult, LocalProjectInput, ServiceErrorCode, execute_local};

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
    if arguments.first().is_some_and(|value| value == "native") {
        return native::run(arguments);
    }
    if arguments.first().is_some_and(|value| value == "graph") {
        return graph::run(arguments);
    }
    let arguments = match args::parse(arguments) {
        Ok(value) => value,
        Err(message) => return usage(message),
    };
    let Some(stop) = cancellation_flag() else {
        return 64;
    };
    let result = match LocalProjectInput::from_config_path(&arguments.config, &stop) {
        Ok(input) => execute_local(input, &arguments.command, &stop),
        Err(error) if error.code() == ServiceErrorCode::Cancelled => {
            LocalOperationResult::cancelled(&arguments.command)
        }
        Err(error)
            if matches!(
                error.code(),
                ServiceErrorCode::ProjectTargetExcluded | ServiceErrorCode::ProjectTargetUnresolved
            ) =>
        {
            LocalOperationResult::input_failure(&arguments.command, &error)
        }
        Err(error) => {
            diagnostic(error.message());
            return 64;
        }
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

fn cancellation_flag() -> Option<Arc<AtomicBool>> {
    let stop = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&stop);
    if ctrlc::set_handler(move || signal.store(true, Ordering::Release)).is_err() {
        diagnostic("could not initialize cancellation handling");
        None
    } else {
        Some(stop)
    }
}
