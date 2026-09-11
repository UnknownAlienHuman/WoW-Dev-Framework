#![forbid(unsafe_code)]

mod cli;
mod input;
mod output;

use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::sync::atomic::AtomicBool;

use cli::{Arguments, Command, USAGE};
use output::{AppError, check_exit_code, write_json};
use wow_service::{CheckRequest, Service, StatusRequest};

fn main() {
    let exit_code = match run(env::args_os()) {
        Ok(exit_code) => exit_code,
        Err(error) => {
            let exit_code = error.exit_code();
            let stderr = io::stderr();
            let mut writer = stderr.lock();
            if error.write_json(&mut writer).is_err() {
                let _ = writeln!(writer, "wow: {error}");
            }
            exit_code
        }
    };
    std::process::exit(exit_code);
}

fn run<I>(arguments: I) -> Result<i32, AppError>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments = Arguments::parse(arguments)?;
    match arguments.command {
        Command::Help => {
            let stdout = io::stdout();
            writeln!(stdout.lock(), "{USAGE}").map_err(AppError::output_io)?;
            Ok(0)
        }
        Command::Version => {
            let stdout = io::stdout();
            writeln!(stdout.lock(), "wow {}", env!("CARGO_PKG_VERSION"))
                .map_err(AppError::output_io)?;
            Ok(0)
        }
        Command::Status => {
            let input = arguments
                .input
                .ok_or_else(|| AppError::usage("status input path disappeared"))?;
            let operation_id = arguments
                .operation_id
                .ok_or_else(|| AppError::usage("status operation ID disappeared"))?;
            let prepared = input::load(&input)?;
            let service = Service::new(prepared.configuration, prepared.backend)
                .map_err(AppError::from)?;
            let result = service
                .status(&StatusRequest::new(operation_id), &AtomicBool::new(false))
                .map_err(AppError::from)?;
            write_json(io::stdout().lock(), &*result)?;
            Ok(0)
        }
        Command::Check { selector, scope } => {
            let input = arguments
                .input
                .ok_or_else(|| AppError::usage("check input path disappeared"))?;
            let operation_id = arguments
                .operation_id
                .ok_or_else(|| AppError::usage("check operation ID disappeared"))?;
            let prepared = input::load(&input)?;
            let service = Service::new(prepared.configuration, prepared.backend)
                .map_err(AppError::from)?;
            let result = service
                .check(
                    &CheckRequest::new(operation_id, selector, scope),
                    &AtomicBool::new(false),
                )
                .map_err(AppError::from)?;
            let exit_code = check_exit_code(result.semantic_status());
            write_json(io::stdout().lock(), &*result)?;
            Ok(exit_code)
        }
    }
}
