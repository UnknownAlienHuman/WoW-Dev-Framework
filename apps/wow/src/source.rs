//! Thin source transport: one service operation and faithful bounded output.
use crate::args::Format;
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode, ServiceResult,
    source_census::{CensusCoverage, census_local_source},
    source_reference::read_local_source_reference,
};

const HELP: &str = "wow source census --config <source-census.json> [--format json|text]\nwow source reference --config <source-reference.json> [--format json|text]\n\nCensus measures one explicit local manifest with native TOC/XML owners and optional package loads. Reference projects pinned generated API into an owner-derived profile, generation and callable view. Partial coverage exits 2; acquisition/identity/budget failures exit 4. Legacy surfaces; replacement: docs/IMPLEMENTATION.md.\n";

#[derive(Clone, Copy)]
enum Operation {
    Census,
    Reference,
}

pub fn run(values: Vec<OsString>) -> u8 {
    if (values.len() == 2
        || (values.len() == 3 && (values[1] == "census" || values[1] == "reference")))
        && values
            .last()
            .is_some_and(|value| value == "--help" || value == "-h")
    {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let (operation, config, format) = match parse(values) {
        Ok(arguments) => arguments,
        Err(message) => return super::usage(message),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let (mut bytes, exit, label) = match execute(operation, &config, &stop) {
        Ok(result) => result,
        Err(error) => {
            super::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
            return match error.code() {
                ServiceErrorCode::Cancelled => 130,
                ServiceErrorCode::InvalidRequest | ServiceErrorCode::InvalidConfiguration => 64,
                _ => 4,
            };
        }
    };
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    if format == Format::Text {
        bytes.splice(0..0, label.iter().copied());
    }
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        return 4;
    }
    exit
}

fn execute(
    operation: Operation,
    config: &Path,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<u8>, u8, &'static [u8])> {
    match operation {
        Operation::Census => {
            let result = census_local_source(config, stop)?;
            let exit = match result.coverage() {
                CensusCoverage::DeclaredMembersMeasured => 0,
                CensusCoverage::Partial => 2,
            };
            let label: &[u8] = if result.package_loads().is_some() {
                b"Manifest-bound source/package load census\n"
            } else {
                b"Manifest-bound lexical source census\n"
            };
            Ok((result.canonical_bytes()?, exit, label))
        }
        Operation::Reference => {
            let result = read_local_source_reference(config, stop)?;
            let exit = if result.is_partial() { 2 } else { 0 };
            Ok((
                result.canonical_bytes(stop)?,
                exit,
                b"Pinned native callable reference\n",
            ))
        }
    }
}

fn parse(values: Vec<OsString>) -> Result<(Operation, PathBuf, Format), &'static str> {
    let size = values
        .iter()
        .try_fold(0usize, |size, value| size.checked_add(value.len()))
        .ok_or("source argument size overflow")?;
    if values.len() > 6 || size > 16 * 1024 {
        return Err("source argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().is_none_or(|value| value != "source") {
        return Err("expected source command");
    }
    let operation = match values
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
    {
        Some("census") => Operation::Census,
        Some("reference") => Operation::Reference,
        _ => return Err("expected source census or reference"),
    };
    let mut config = None;
    let mut format = None;
    while let Some(flag) = values.next() {
        let value = values.next().ok_or("missing source option value")?;
        match flag.to_str().ok_or("invalid source option encoding")? {
            "--config" if config.is_none() && !value.is_empty() => {
                config = Some(PathBuf::from(value))
            }
            "--format" if format.is_none() => {
                format = Some(match value.to_str() {
                    Some("json") => Format::Json,
                    Some("text") => Format::Text,
                    _ => return Err("source format must be json or text"),
                })
            }
            _ => return Err("unknown or repeated source option"),
        }
    }
    Ok((
        operation,
        config.ok_or("--config is required")?,
        format.unwrap_or(Format::Json),
    ))
}
