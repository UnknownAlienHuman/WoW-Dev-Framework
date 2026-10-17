//! Thin transport for the original native Reference callable comparison.
use crate::args::Format;
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode, ServiceResult,
    source_reference_compare::{SourceReferenceComparisonResult, compare_local_source_references},
};

const HELP: &str = "wow source reference-compare --before-config <before.json> --after-config <after.json> --request <request.json> [--format json|text]\n\nCompare two pinned source reference configurations against an explicit request. Partial coverage exits 2; cancellation exits 130; invalid input exits 4 or 64.\n";

pub fn run(values: Vec<OsString>) -> u8 {
    if values.len() == 3
        && values[1] == "reference-compare"
        && (values[2] == "--help" || values[2] == "-h")
    {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let (before_config, after_config, request_file, format) = match parse(values) {
        Ok(arguments) => arguments,
        Err(message) => return super::usage(message),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let result = match execute(&before_config, &after_config, &request_file, &stop) {
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
    let exit = if result.is_partial() { 2 } else { 0 };
    let mut bytes = match result.canonical_bytes(&stop) {
        Ok(bytes) => bytes,
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
        bytes.splice(0..0, b"Source reference comparison\n".iter().copied());
    }
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        return 4;
    }
    exit
}

fn execute(
    before_config: &Path,
    after_config: &Path,
    request_file: &Path,
    stop: &AtomicBool,
) -> ServiceResult<SourceReferenceComparisonResult> {
    compare_local_source_references(before_config, after_config, request_file, stop)
}

fn parse(values: Vec<OsString>) -> Result<(PathBuf, PathBuf, PathBuf, Format), &'static str> {
    let size = values
        .iter()
        .try_fold(0usize, |size, value| size.checked_add(value.len()))
        .ok_or("source reference-compare argument size overflow")?;
    if values.len() > 10 || size > 16 * 1024 {
        return Err("source reference-compare argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().is_none_or(|value| value != "source") {
        return Err("expected source command");
    }
    if values
        .next()
        .is_none_or(|value| value != "reference-compare")
    {
        return Err("expected source reference-compare");
    }
    let mut before_config = None;
    let mut after_config = None;
    let mut request_file = None;
    let mut format = None;
    while let Some(flag) = values.next() {
        let value = values
            .next()
            .ok_or("missing source reference-compare option value")?;
        match flag
            .to_str()
            .ok_or("invalid source reference-compare option encoding")?
        {
            "--before-config" if before_config.is_none() && !value.is_empty() => {
                before_config = Some(PathBuf::from(value))
            }
            "--after-config" if after_config.is_none() && !value.is_empty() => {
                after_config = Some(PathBuf::from(value))
            }
            "--request" if request_file.is_none() && !value.is_empty() => {
                request_file = Some(PathBuf::from(value))
            }
            "--format" if format.is_none() => {
                format = Some(match value.to_str() {
                    Some("json") => Format::Json,
                    Some("text") => Format::Text,
                    _ => return Err("source reference-compare format must be json or text"),
                })
            }
            _ => return Err("unknown or repeated source reference-compare option"),
        }
    }
    Ok((
        before_config.ok_or("--before-config is required")?,
        after_config.ok_or("--after-config is required")?,
        request_file.ok_or("--request is required")?,
        format.unwrap_or(Format::Json),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_requires_all_three_paths() -> Result<(), &'static str> {
        let values = vec![
            OsString::from("source"),
            OsString::from("reference-compare"),
        ];
        let err = parse(values).err().ok_or("missing paths must fail")?;
        assert!(err.contains("--before-config"));
        Ok(())
    }

    #[test]
    fn parse_rejects_repeated_option() -> Result<(), &'static str> {
        let values = vec![
            OsString::from("source"),
            OsString::from("reference-compare"),
            OsString::from("--before-config"),
            OsString::from("a.json"),
            OsString::from("--before-config"),
            OsString::from("b.json"),
        ];
        let err = parse(values).err().ok_or("duplicate paths must fail")?;
        assert!(err.contains("repeated") || err.contains("unknown"));
        Ok(())
    }

    #[test]
    fn parse_rejects_unknown_option() -> Result<(), &'static str> {
        let values = vec![
            OsString::from("source"),
            OsString::from("reference-compare"),
            OsString::from("--unknown"),
            OsString::from("value"),
        ];
        let err = parse(values).err().ok_or("unknown flags must fail")?;
        assert!(err.contains("unknown") || err.contains("repeated"));
        Ok(())
    }

    #[test]
    fn parse_rejects_missing_value() -> Result<(), &'static str> {
        let values = vec![
            OsString::from("source"),
            OsString::from("reference-compare"),
            OsString::from("--before-config"),
        ];
        let err = parse(values).err().ok_or("missing value must fail")?;
        assert!(err.contains("missing") || err.contains("required"));
        Ok(())
    }

    #[test]
    fn parse_accepts_valid_arguments() -> Result<(), &'static str> {
        let values = vec![
            OsString::from("source"),
            OsString::from("reference-compare"),
            OsString::from("--before-config"),
            OsString::from("before.json"),
            OsString::from("--after-config"),
            OsString::from("after.json"),
            OsString::from("--request"),
            OsString::from("request.json"),
            OsString::from("--format"),
            OsString::from("text"),
        ];
        let result = parse(values)?;
        assert_eq!(result.0, PathBuf::from("before.json"));
        assert_eq!(result.1, PathBuf::from("after.json"));
        assert_eq!(result.2, PathBuf::from("request.json"));
        assert!(matches!(result.3, Format::Text));
        Ok(())
    }
}
