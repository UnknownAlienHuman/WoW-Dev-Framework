//! Thin source census transport: one service operation and faithful output.
use crate::args::Format;
use std::{ffi::OsString, io::Write, path::PathBuf, sync::atomic::Ordering};
use wow_service::{
    ServiceErrorCode,
    source_census::{CensusCoverage, census_local_source},
};

const HELP: &str = "wow source census --config <source-census.json> [--format json|text]\n\nMeasures one explicit admitted local manifest with native TOC/XML owners. Partial input or parser refusals exit 2; no source acquisition or graph/publication is performed. See apps/wow/SOURCE_CENSUS.md.\n";

pub fn run(values: Vec<OsString>) -> u8 {
    if (values.len() == 2 || (values.len() == 3 && values[1] == "census"))
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
    let (config, format) = match parse(values) {
        Ok(arguments) => arguments,
        Err(message) => return super::usage(message),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let result = match census_local_source(&config, &stop) {
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
    let exit = match result.census().coverage() {
        CensusCoverage::DeclaredMembersMeasured => 0,
        CensusCoverage::Partial => 2,
    };
    let mut bytes = match result.canonical_bytes() {
        Ok(bytes) => bytes,
        Err(error) => {
            super::diagnostic(error.message());
            return 4;
        }
    };
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    if format == Format::Text {
        bytes.splice(
            0..0,
            b"Manifest-bound lexical source census\n".iter().copied(),
        );
    }
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        return 4;
    }
    exit
}

fn parse(values: Vec<OsString>) -> Result<(PathBuf, Format), &'static str> {
    let size = values
        .iter()
        .try_fold(0usize, |size, value| size.checked_add(value.len()))
        .ok_or("source argument size overflow")?;
    if values.len() > 6 || size > 16 * 1024 {
        return Err("source argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().is_none_or(|value| value != "source")
        || values.next().is_none_or(|value| value != "census")
    {
        return Err("expected source census");
    }
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
        config.ok_or("--config is required")?,
        format.unwrap_or(Format::Json),
    ))
}
