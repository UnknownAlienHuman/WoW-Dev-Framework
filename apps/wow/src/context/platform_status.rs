//! Native platform status transport. Status semantics remain Service-owned.
use super::{Arguments, MAX_VALUE_BYTES};
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_MAP_RESULT_MAX_BYTES, CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES,
        CONTEXT_PLATFORM_STATUS_REQUEST_MAX_BYTES, ContextPlatformHostInput,
        ContextPlatformStatusRequest, execute_context_platform_status,
    },
};

pub(super) struct Request {
    status_input: PathBuf,
}

/// Route only when an option position selects native status transport. A Primary
/// path whose value happens to equal a native flag cannot switch parsers.
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
            .is_some_and(|flag| flag == "--platform-status-request-input")
            || (pair.first().is_some_and(|flag| flag == "--platform")
                && pair.get(1).is_some_and(|value| value == "native"))
    });
    if !selected {
        return Ok(None);
    }

    let mut host_config = None;
    let mut status_input = None;
    let mut platform_native = false;
    let mut output_json = false;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        let option = option
            .to_str()
            .ok_or("invalid native status option encoding")?;
        if !matches!(
            option,
            "--native-host-config" | "--platform" | "--platform-status-request-input" | "--output"
        ) {
            return Err("native platform status rejects primary or unknown options");
        }
        let value = options.next().ok_or("missing native status option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid native status option length");
        }
        match option {
            "--native-host-config" => {
                set_unique(&mut host_config, PathBuf::from(value.as_os_str()))?;
            }
            "--platform-status-request-input" => {
                set_unique(&mut status_input, PathBuf::from(value.as_os_str()))?;
            }
            "--platform" => {
                if value != "native" || platform_native {
                    return Err("native platform status rejects mixed or duplicate platform");
                }
                platform_native = true;
            }
            "--output" => {
                if value != "envelope-json" || output_json {
                    return Err("native platform status supports envelope-json only");
                }
                output_json = true;
            }
            _ => return Err("unknown native status option"),
        }
    }
    if !platform_native {
        return Err("platform status request requires --platform native");
    }

    Ok(Some(Arguments {
        source_root: None,
        host_config: host_config.ok_or("native platform status requires explicit host input")?,
        request: super::Request::PlatformStatus(Request {
            status_input: status_input
                .ok_or("native platform status requires --platform-status-request-input")?,
        }),
    }))
}

fn set_unique<T>(slot: &mut Option<T>, value: T) -> Result<(), &'static str> {
    if slot.is_some() {
        return Err("duplicate native platform status option");
    }
    *slot = Some(value);
    Ok(())
}

pub(super) fn run(host_path: &Path, arguments: &Request, stop: &AtomicBool) -> u8 {
    let request_bytes = match crate::graph::read_file(
        &arguments.status_input,
        CONTEXT_PLATFORM_STATUS_REQUEST_MAX_BYTES,
        stop,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return super::input_failure("cannot read bounded platform status request", stop),
    };
    let request = match ContextPlatformStatusRequest::from_json_slice(&request_bytes, stop) {
        Ok(request) => request,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid platform status request", stop),
    };
    drop(request_bytes);

    let host_bytes =
        match crate::graph::read_file(host_path, CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES, stop) {
            Ok(bytes) => bytes,
            Err(_) => return super::input_failure("cannot read bounded platform host input", stop),
        };
    let host = match ContextPlatformHostInput::from_json_slice(&host_bytes, stop) {
        Ok(host) => host,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid platform context host configuration", stop),
    };
    drop(host_bytes);
    if stop.load(Ordering::Acquire) {
        return 130;
    }

    // One explicit metadata operation. The adapter never invokes platform Map.
    let result = match execute_context_platform_status(&host, &request, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("platform status operation could not finalize a result");
            return 4;
        }
    };
    if result.canonical_bytes().is_empty()
        || result.canonical_bytes().len() > CONTEXT_MAP_RESULT_MAX_BYTES
    {
        crate::diagnostic("platform status result byte limit or encoding failure");
        return 4;
    }
    super::emit_json_result(&result, stop)
}
