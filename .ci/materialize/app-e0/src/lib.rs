#![forbid(unsafe_code)]

//! Bounded one-shot command line entry point over the versioned E0-F service.

use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;
use wow_core::canonical_json_bytes;
use wow_rules::RuleEvaluationInput;
use wow_service::{
    OperationRegistry, OperationSelector, ServiceHost, ServiceRequest, ServiceResponseStatus,
};

pub const EXIT_SUCCESS: u8 = 0;
pub const EXIT_DIAGNOSTICS: u8 = 1;
pub const EXIT_INPUT_ERROR: u8 = 2;
pub const EXIT_OPERATION_REJECTED: u8 = 3;
pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = "Usage:\n  wow operations\n  wow rules-evaluate --input <PATH|->\n  wow --help\n  wow --version\n";

#[derive(Debug)]
enum Command {
    Operations,
    RulesEvaluate { input: Input },
    Help,
    Version,
}

#[derive(Debug)]
enum Input {
    Stdin,
    File(PathBuf),
}

#[derive(Debug)]
struct CliError(Box<str>);

impl CliError {
    fn new(message: impl Into<Box<str>>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Executes exactly one bounded command and returns a stable process exit code.
///
/// `rules-evaluate` accepts one strict JSON `RuleEvaluationInput`, constructs one
/// immutable service snapshot, executes `rules.evaluate@1`, and writes one
/// canonical JSON response. It performs no discovery, network access, or mutation.
pub fn run<I, R, W, E>(args: I, mut stdin: R, mut stdout: W, mut stderr: E) -> u8
where
    I: IntoIterator<Item = OsString>,
    R: Read,
    W: Write,
    E: Write,
{
    match execute(args, &mut stdin, &mut stdout) {
        Ok(code) => code,
        Err(error) => {
            let _ = writeln!(stderr, "wow: {error}");
            EXIT_INPUT_ERROR
        }
    }
}

fn execute<I, R, W>(args: I, stdin: &mut R, stdout: &mut W) -> Result<u8, CliError>
where
    I: IntoIterator<Item = OsString>,
    R: Read,
    W: Write,
{
    match parse(args)? {
        Command::Help => {
            stdout.write_all(USAGE.as_bytes()).map_err(write_error)?;
            Ok(EXIT_SUCCESS)
        }
        Command::Version => {
            writeln!(stdout, "wow {VERSION}").map_err(write_error)?;
            Ok(EXIT_SUCCESS)
        }
        Command::Operations => {
            #[derive(Serialize)]
            struct Operations<'a> {
                schema: &'static str,
                operations: &'a [wow_service::OperationDescriptor],
            }
            write_canonical(
                &Operations {
                    schema: "wow-cli/operations/1",
                    operations: OperationRegistry::descriptors(),
                },
                stdout,
            )?;
            Ok(EXIT_SUCCESS)
        }
        Command::RulesEvaluate { input } => {
            let bytes = read_input(input, stdin)?;
            let rule_input: RuleEvaluationInput = serde_json::from_slice(&bytes)
                .map_err(|error| CliError::new(format!("invalid rule input JSON: {error}")))?;
            let host = ServiceHost::new(rule_input)
                .map_err(|error| CliError::new(format!("service snapshot rejected: {error}")))?;
            let view = host
                .read()
                .map_err(|error| CliError::new(format!("service snapshot unavailable: {error}")))?;
            let request = ServiceRequest::new(
                "wow-cli:rules.evaluate@1",
                view.snapshot_id(),
                OperationSelector::evaluate_rules(),
            )
            .map_err(|error| CliError::new(format!("service request rejected: {error}")))?;
            let response = host
                .execute(&request)
                .map_err(|error| CliError::new(format!("service execution failed: {error}")))?;
            let code = match response.status() {
                ServiceResponseStatus::Rejected => EXIT_OPERATION_REJECTED,
                ServiceResponseStatus::Completed => {
                    if response
                        .result()
                        .is_some_and(|report| !report.diagnostics().is_empty())
                    {
                        EXIT_DIAGNOSTICS
                    } else {
                        EXIT_SUCCESS
                    }
                }
            };
            write_canonical(&response, stdout)?;
            Ok(code)
        }
    }
}

fn parse<I>(args: I) -> Result<Command, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let values = args
        .into_iter()
        .map(|value| {
            value
                .into_string()
                .map_err(|_| CliError::new("command arguments must be valid UTF-8"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    match values.as_slice() {
        [] => Ok(Command::Help),
        [value] if value == "--help" || value == "-h" => Ok(Command::Help),
        [value] if value == "--version" || value == "-V" => Ok(Command::Version),
        [value] if value == "operations" => Ok(Command::Operations),
        [command, flag, input] if command == "rules-evaluate" && flag == "--input" => {
            if input == "-" {
                Ok(Command::RulesEvaluate {
                    input: Input::Stdin,
                })
            } else {
                validate_input_path(input)?;
                Ok(Command::RulesEvaluate {
                    input: Input::File(PathBuf::from(input)),
                })
            }
        }
        _ => Err(CliError::new(format!("invalid invocation\n{USAGE}"))),
    }
}

fn validate_input_path(value: &str) -> Result<(), CliError> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(CliError::new(
            "input path is empty or contains control characters",
        ));
    }
    Ok(())
}

fn read_input<R: Read>(input: Input, stdin: &mut R) -> Result<Vec<u8>, CliError> {
    match input {
        Input::Stdin => read_bounded(stdin, "stdin"),
        Input::File(path) => {
            let mut file = File::open(&path).map_err(|error| {
                CliError::new(format!(
                    "cannot open input {}: {error}",
                    display_path(&path)
                ))
            })?;
            read_bounded(&mut file, &display_path(&path))
        }
    }
}

fn read_bounded(reader: &mut impl Read, source: &str) -> Result<Vec<u8>, CliError> {
    let limit = u64::try_from(MAX_INPUT_BYTES + 1)
        .map_err(|_| CliError::new("input limit cannot be represented"))?;
    let mut bytes = Vec::new();
    reader
        .take(limit)
        .read_to_end(&mut bytes)
        .map_err(|error| CliError::new(format!("cannot read {source}: {error}")))?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err(CliError::new(format!(
            "input exceeds {MAX_INPUT_BYTES} bytes"
        )));
    }
    if bytes.is_empty() {
        return Err(CliError::new("input is empty"));
    }
    Ok(bytes)
}

fn write_canonical(value: &impl Serialize, output: &mut impl Write) -> Result<(), CliError> {
    let bytes = canonical_json_bytes(value)
        .map_err(|error| CliError::new(format!("cannot canonicalize output: {error}")))?;
    output.write_all(&bytes).map_err(write_error)?;
    output.write_all(b"\n").map_err(write_error)
}

fn write_error(error: io::Error) -> CliError {
    CliError::new(format!("cannot write output: {error}"))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
