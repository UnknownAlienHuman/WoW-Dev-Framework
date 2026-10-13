//! Combined context-status transport. Selection and status semantics remain Service-owned.
use super::{Arguments, MAX_VALUE_BYTES};
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES, CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES,
        ContextCombinedHostInput, ContextCombinedStatusRequest, execute_context_combined_status,
    },
};

pub(super) struct Request {
    status_input: PathBuf,
}

/// Route only `wow context status`, selected by a combined option in option position.
/// A path/value equal to one of these flags cannot change the selected parser.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Arguments>, &'static str> {
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context"))
        || values.get(1).map(OsString::as_os_str) != Some(OsStr::new("status"))
    {
        return Ok(None);
    }
    let Some(options) = values.get(2..) else {
        return Ok(None);
    };
    let selected = options.chunks(2).any(|pair| {
        pair.first()
            .is_some_and(|flag| flag == "--combined-status-request-input")
            || (pair.first().is_some_and(|flag| flag == "--platform")
                && pair.get(1).is_some_and(|value| value == "combined"))
    });
    if !selected {
        return Ok(None);
    }

    let mut host_config = None;
    let mut status_input = None;
    let mut platform_combined = false;
    let mut output_json = false;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        let option = option
            .to_str()
            .ok_or("invalid combined status option encoding")?;
        if !matches!(
            option,
            "--native-host-config" | "--platform" | "--combined-status-request-input" | "--output"
        ) {
            return Err("combined status rejects mixed or unknown options");
        }
        let value = options
            .next()
            .ok_or("missing combined status option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid combined status option length");
        }
        match option {
            "--native-host-config" => {
                set_unique(&mut host_config, PathBuf::from(value.as_os_str()))?;
            }
            "--combined-status-request-input" => {
                set_unique(&mut status_input, PathBuf::from(value.as_os_str()))?;
            }
            "--platform" => {
                if value != "combined" || platform_combined {
                    return Err("combined status rejects mixed or duplicate platform selection");
                }
                platform_combined = true;
            }
            "--output" => {
                if value != "envelope-json" || output_json {
                    return Err("combined status supports envelope-json only");
                }
                output_json = true;
            }
            _ => return Err("unknown combined status option"),
        }
    }
    if !platform_combined {
        return Err("combined status request requires --platform combined");
    }

    Ok(Some(Arguments {
        host_config: host_config.ok_or("combined status requires explicit native host input")?,
        request: super::Request::CombinedStatus(Request {
            status_input: status_input
                .ok_or("combined status requires --combined-status-request-input")?,
        }),
        source_root: None,
    }))
}

fn set_unique<T>(slot: &mut Option<T>, value: T) -> Result<(), &'static str> {
    if slot.is_some() {
        return Err("duplicate combined status option");
    }
    *slot = Some(value);
    Ok(())
}

pub(super) fn run(host_path: &Path, arguments: &Request, stop: &AtomicBool) -> u8 {
    let request_bytes = match crate::graph::read_file(
        &arguments.status_input,
        CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES,
        stop,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return super::input_failure("cannot read bounded combined status request", stop),
    };
    let request = match ContextCombinedStatusRequest::from_json_slice(&request_bytes, stop) {
        Ok(request) => request,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid combined status request", stop),
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

    // One explicit composed-status operation; Service owns acquisition and closure.
    let result = match execute_context_combined_status(&host, &request, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("combined status operation could not finalize a result");
            return 4;
        }
    };
    super::emit_json_result(&result, stop)
}
