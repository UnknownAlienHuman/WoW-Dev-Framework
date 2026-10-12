//! Closed combined-map explicit-file transport; semantics remain service-owned.
use super::{Arguments, Command, MAX_VALUE_BYTES, set};
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES, CONTEXT_COMBINED_REQUEST_MAX_BYTES,
        ContextCombinedHostInput, ContextCombinedRequest, execute_context_combined,
    },
};

pub(super) struct Request {
    request_input: PathBuf,
}

/// Run after aggregate argv bounds. Only option positions select Combined.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Arguments>, &'static str> {
    let Some(options) = values.get(2..) else {
        return Ok(None);
    };
    let selected = options.chunks(2).any(|pair| {
        pair.first()
            .is_some_and(|flag| flag == "--combined-request-input")
            || (pair.first().is_some_and(|flag| flag == "--platform")
                && pair.get(1).is_some_and(|value| value == "combined"))
    });
    if !selected {
        return Ok(None);
    }
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context"))
        || values.get(1).map(OsString::as_os_str) != Some(OsStr::new("map"))
    {
        return Err("combined platform supports context map only");
    }
    let mut host_config = None;
    let mut request_input = None;
    let mut platform = None;
    let mut output = None;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        let option = option
            .to_str()
            .ok_or("invalid combined context option encoding")?;
        if !matches!(
            option,
            "--native-host-config" | "--platform" | "--combined-request-input" | "--output"
        ) {
            return Err("combined context rejects primary, native or unknown options");
        }
        let value = options
            .next()
            .ok_or("missing combined context option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid combined context option length");
        }
        match option {
            "--native-host-config" => {
                set(
                    &mut host_config,
                    PathBuf::from(value.as_os_str()),
                    Command::Map,
                )?;
            }
            "--combined-request-input" => {
                set(
                    &mut request_input,
                    PathBuf::from(value.as_os_str()),
                    Command::Map,
                )?;
            }
            "--platform" => {
                if value != "combined" {
                    return Err("combined context rejects mixed platform selection");
                }
                set(&mut platform, (), Command::Map)?;
            }
            "--output" => {
                if value != "envelope-json" {
                    return Err("combined context map output must be envelope-json");
                }
                set(&mut output, (), Command::Map)?;
            }
            _ => return Err("unknown combined context option"),
        }
    }
    platform.ok_or("combined request input requires --platform combined")?;
    Ok(Some(Arguments {
        host_config: host_config.ok_or("combined context requires explicit native host input")?,
        request: super::Request::Combined(Request {
            request_input: request_input
                .ok_or("combined context requires --combined-request-input")?,
        }),
    }))
}

pub(super) fn run(host_path: &Path, arguments: &Request, stop: &AtomicBool) -> u8 {
    let request_bytes = match crate::graph::read_file(
        &arguments.request_input,
        CONTEXT_COMBINED_REQUEST_MAX_BYTES,
        stop,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return super::input_failure("cannot read bounded combined request input", stop),
    };
    let request = match ContextCombinedRequest::from_json_slice(&request_bytes, stop) {
        Ok(request) => request,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid combined context request", stop),
    };
    drop(request_bytes);
    let host_bytes =
        match crate::graph::read_file(host_path, CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES, stop) {
            Ok(bytes) => bytes,
            Err(_) => return super::input_failure("cannot read bounded combined host input", stop),
        };
    let host = match ContextCombinedHostInput::from_json_slice(&host_bytes, stop) {
        Ok(host) => host,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid combined context host configuration", stop),
    };
    drop(host_bytes);
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    // One operation; service holds both native scopes and owns checked closure.
    let result = match execute_context_combined(&host, &request, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("combined context operation could not finalize a result");
            return 4;
        }
    };
    super::emit_json_result(&result, stop)
}
