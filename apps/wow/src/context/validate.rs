//! Private explicit-file transport for one `wow context validate` operation.

use std::{
    ffi::{OsStr, OsString},
    io::{Read, Write},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_MAP_RESULT_MAX_BYTES, ContextMapStatus, ContextValidateRequest,
        ContextValidateResult, ValidationState, execute_context_validate,
    },
};

const MAX_ARGS: usize = 4096;
const MAX_ARG_BYTES: usize = 1024 * 1024;
const MAX_VALUE_BYTES: usize = 32768;
const PROFILE: &str = "wow-context/primary-structural-pack/1";

pub(super) enum Input {
    File(PathBuf),
    Stdin,
}

pub(super) struct Request {
    input: Input,
    service: ContextValidateRequest,
}

/// Returns `Some` only for the exact command prefix. The parser accepts no
/// source/root/store/platform/renderer flags, host config, or output artifact.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Request>, &'static str> {
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context"))
        || values.get(1).map(OsString::as_os_str) != Some(OsStr::new("validate"))
    {
        return Ok(None);
    }
    let total = values
        .iter()
        .try_fold(0usize, |sum, value| sum.checked_add(value.len()))
        .ok_or("context validate argument size overflow")?;
    if values.len() > MAX_ARGS || total > MAX_ARG_BYTES {
        return Err("context validate argument limit exceeded");
    }

    let mut input = None;
    let mut artifact_kind = false;
    let mut media_type = false;
    let mut profile = false;
    let mut closure = false;
    let mut output = false;
    let mut max_input_bytes = None;
    let mut max_metadata_bytes = None;
    let mut max_context_work = None;
    let mut max_depth = None;
    let mut max_tokens = None;
    let mut args = values[2..].iter();
    while let Some(option) = args.next() {
        let option = option.to_str().ok_or("invalid context validate option")?;
        let value = args.next().ok_or("missing context validate option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid context validate option length");
        }
        match option {
            "--input" => {
                if input.is_some() {
                    return Err("duplicate context validate input");
                }
                input = Some(if value == OsStr::new("-") {
                    Input::Stdin
                } else {
                    Input::File(PathBuf::from(value))
                });
            }
            "--artifact-kind" => {
                if artifact_kind || value != OsStr::new("semantic-pack") {
                    return Err("context validate supports one semantic-pack input");
                }
                artifact_kind = true;
            }
            "--media-type" => {
                if media_type || value != OsStr::new("application/json") {
                    return Err("context validate requires application/json");
                }
                media_type = true;
            }
            "--validation-profile-id" => {
                if profile || value != OsStr::new(PROFILE) {
                    return Err("context validate requires the exact Primary structural profile");
                }
                profile = true;
            }
            "--origin-closure" => {
                if closure || value != OsStr::new("structural") {
                    return Err("context validate supports StructuralOnly closure");
                }
                closure = true;
            }
            "--output" => {
                if output || value != OsStr::new("envelope-json") {
                    return Err("context validate supports envelope-json only");
                }
                output = true;
            }
            "--max-input-bytes" => set_number(&mut max_input_bytes, value)?,
            "--max-metadata-bytes" => set_number(&mut max_metadata_bytes, value)?,
            "--max-context-work" => set_number(&mut max_context_work, value)?,
            "--max-depth" => set_number(&mut max_depth, value)?,
            "--max-tokens" => set_number(&mut max_tokens, value)?,
            _ => return Err("unknown context validate option"),
        }
    }
    if !artifact_kind || !media_type || !profile || !closure {
        return Err("context validate requires artifact kind, media type, profile and closure");
    }
    let max_input_bytes = max_input_bytes.ok_or("context validate requires --max-input-bytes")?;
    let max_metadata_bytes =
        max_metadata_bytes.ok_or("context validate requires --max-metadata-bytes")?;
    let max_context_work =
        max_context_work.ok_or("context validate requires --max-context-work")?;
    let max_depth = max_depth.ok_or("context validate requires --max-depth")?;
    let max_tokens = max_tokens.ok_or("context validate requires --max-tokens")?;
    let service = ContextValidateRequest::primary_semantic_pack_structural(
        max_input_bytes,
        max_metadata_bytes,
        max_context_work,
        max_depth,
        max_tokens,
    )
    .map_err(|_| "invalid bounded context validation limits")?;
    Ok(Some(Request {
        input: input.ok_or("context validate requires --input <PATH|->")?,
        service,
    }))
}

pub(super) fn run(request: &Request, stop: &AtomicBool) -> u8 {
    let limit = match usize::try_from(request.service.max_input_bytes()) {
        Ok(limit) => limit,
        Err(_) => {
            return input_failure("context validation input limit is not representable", stop);
        }
    };
    let bytes = match read_input(&request.input, limit, stop) {
        Ok(bytes) => bytes,
        Err(_) => return input_failure("cannot read bounded context artifact", stop),
    };
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    let result = match execute_context_validate(&request.service, &bytes, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(error) => {
            crate::diagnostic("context validation result could not be finalized");
            return match error.code() {
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed => 4,
                _ => 3,
            };
        }
    };
    emit(&result, stop)
}

pub(super) fn read_input(
    input: &Input,
    limit: usize,
    stop: &AtomicBool,
) -> Result<Vec<u8>, &'static str> {
    match input {
        Input::File(path) => crate::graph::read_file(path, limit, stop),
        Input::Stdin => {
            let stdin = std::io::stdin();
            let mut reader = stdin.lock();
            let mut bytes = Vec::with_capacity(limit.min(64 * 1024));
            let mut chunk = [0u8; 8192];
            loop {
                if stop.load(Ordering::Acquire) {
                    return Err("context artifact input cancelled");
                }
                let available = chunk.len().min(limit - bytes.len() + 1);
                let count = match reader.read(&mut chunk[..available]) {
                    Ok(count) => count,
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => return Err("cannot read context artifact input"),
                };
                if count == 0 {
                    return Ok(bytes);
                }
                if bytes.len() + count > limit {
                    return Err("context artifact input byte limit");
                }
                bytes.extend_from_slice(&chunk[..count]);
            }
        }
    }
}

fn set_number<T>(slot: &mut Option<T>, value: &OsStr) -> Result<(), &'static str>
where
    T: TryFrom<u64>,
{
    if slot.is_some() {
        return Err("duplicate context validation limit");
    }
    let text = value.to_str().ok_or("invalid context validation limit")?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("context validation limits must be unsigned decimal integers");
    }
    let parsed = text
        .parse::<u64>()
        .map_err(|_| "context validation limit overflow")?;
    let converted = T::try_from(parsed).map_err(|_| "context validation limit overflow")?;
    *slot = Some(converted);
    Ok(())
}

fn emit(result: &ContextValidateResult, stop: &AtomicBool) -> u8 {
    let bytes = result.canonical_bytes();
    if bytes.is_empty() || bytes.len() > CONTEXT_MAP_RESULT_MAX_BYTES {
        crate::diagnostic("context validation envelope byte limit or encoding failure");
        return 4;
    }
    let exit = match result.operation_status() {
        ContextMapStatus::Cancelled => 130,
        ContextMapStatus::Failed => match result.failure_code() {
            Some(
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed,
            ) => 4,
            Some(_) => 3,
            None => 4,
        },
        ContextMapStatus::Partial | ContextMapStatus::Truncated => 2,
        ContextMapStatus::Complete => match result.validation_state() {
            Some(ValidationState::Valid) => 0,
            Some(ValidationState::Invalid) => 1,
            Some(ValidationState::NotEvaluated) | None => 2,
        },
    };
    // A completed result remains unpublished until this check passes. Once the
    // first byte is written, finish the envelope while observing cancellation
    // between bounded writes; this avoids publishing a deliberately partial JSON
    // result and still preserves 130 if cancellation arrives during delivery.
    let mut stdout = std::io::stdout().lock();
    let cancellation_envelope = result.operation_status() == ContextMapStatus::Cancelled;
    if stop.load(Ordering::Acquire) && !cancellation_envelope {
        return 130;
    }
    let mut cancellation_seen = stop.load(Ordering::Acquire);
    let mut first_write = true;
    for chunk in bytes.chunks(8192).chain(std::iter::once(&b"\n"[..])) {
        let mut offset = 0;
        while offset < chunk.len() {
            let stopped = stop.load(Ordering::Acquire);
            cancellation_seen |= stopped;
            if first_write && stopped && !cancellation_envelope {
                return 130;
            }
            let write = stdout.write(&chunk[offset..]);
            match write {
                Ok(0) => {
                    crate::diagnostic("context validation output failed");
                    return if cancellation_seen || stop.load(Ordering::Acquire) {
                        130
                    } else {
                        4
                    };
                }
                Ok(written) => {
                    first_write = false;
                    offset += written;
                    cancellation_seen |= stop.load(Ordering::Acquire);
                }
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => {
                    crate::diagnostic("context validation output failed");
                    return if cancellation_seen || stop.load(Ordering::Acquire) {
                        130
                    } else {
                        4
                    };
                }
            }
        }
    }
    if stdout.flush().is_err() {
        crate::diagnostic("context validation output failed");
        return if cancellation_seen || stop.load(Ordering::Acquire) {
            130
        } else {
            4
        };
    }
    cancellation_seen |= stop.load(Ordering::Acquire);
    if cancellation_seen { 130 } else { exit }
}

fn input_failure(message: &'static str, stop: &AtomicBool) -> u8 {
    if stop.load(Ordering::Acquire) {
        130
    } else {
        crate::usage(message)
    }
}
