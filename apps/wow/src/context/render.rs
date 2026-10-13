//! Bounded explicit-file transport for one Primary semantic-pack render operation.

use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_MAP_RESULT_MAX_BYTES, ContextMapStatus, ContextRenderFormat, ContextRenderRequest,
        ContextRenderResult, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID, ValidationState, execute_context_render,
    },
};

use super::validate::{Input, read_input};

const VALIDATION_PROFILE: &str = "wow-context/primary-structural-pack/1";

pub(super) struct Request {
    input: Input,
    service: ContextRenderRequest,
}

/// Parses only the bounded Primary Structural L0 render route. Unknown flags,
/// including host/source/store selectors, are refused before reading input.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Request>, &'static str> {
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context"))
        || values.get(1).map(OsString::as_os_str) != Some(OsStr::new("render"))
    {
        return Ok(None);
    }
    let total = values
        .iter()
        .try_fold(0usize, |sum, value| sum.checked_add(value.len()))
        .ok_or("context render argument size overflow")?;
    if values.len() > super::MAX_ARGUMENTS || total > super::MAX_ARGUMENT_BYTES {
        return Err("context render argument limit exceeded");
    }

    let mut input = None;
    let mut artifact_kind = false;
    let mut media_type = false;
    let mut validation_profile = false;
    let mut closure = false;
    let mut renderer_format = None;
    let mut output = false;
    let mut max_input_bytes = None;
    let mut max_metadata_bytes = None;
    let mut max_context_work = None;
    let mut max_depth = None;
    let mut max_tokens = None;
    let mut args = values[2..].iter();
    while let Some(option) = args.next() {
        let option = option.to_str().ok_or("invalid context render option")?;
        let value = args.next().ok_or("missing context render option value")?;
        if value.is_empty() || value.len() > super::MAX_VALUE_BYTES {
            return Err("invalid context render option length");
        }
        match option {
            "--input" => {
                if input.is_some() {
                    return Err("duplicate context render input");
                }
                input = Some(if value == OsStr::new("-") {
                    Input::Stdin
                } else {
                    Input::File(PathBuf::from(value))
                });
            }
            "--artifact-kind" => {
                if artifact_kind || value != OsStr::new("semantic-pack") {
                    return Err("context render supports one semantic-pack input");
                }
                artifact_kind = true;
            }
            "--media-type" => {
                if media_type || value != OsStr::new("application/json") {
                    return Err("context render requires application/json");
                }
                media_type = true;
            }
            "--validation-profile-id" => {
                if validation_profile || value != OsStr::new(VALIDATION_PROFILE) {
                    return Err("context render requires the exact Primary structural profile");
                }
                validation_profile = true;
            }
            "--origin-closure" => {
                if closure || value != OsStr::new("structural") {
                    return Err("context render supports StructuralOnly closure");
                }
                closure = true;
            }
            "--renderer-profile-id" => {
                if renderer_format.is_some() {
                    return Err("duplicate context render renderer profile");
                }
                renderer_format = Some(match value.to_str() {
                    Some(PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID) => ContextRenderFormat::Json,
                    Some(PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID) => {
                        ContextRenderFormat::Markdown
                    }
                    _ => return Err("unsupported exact Primary renderer profile"),
                });
            }
            "--output" => {
                if output || value != OsStr::new("envelope-json") {
                    return Err("context render supports envelope-json only");
                }
                output = true;
            }
            "--max-input-bytes" => set_number(&mut max_input_bytes, value)?,
            "--max-metadata-bytes" => set_number(&mut max_metadata_bytes, value)?,
            "--max-context-work" => set_number(&mut max_context_work, value)?,
            "--max-depth" => set_number(&mut max_depth, value)?,
            "--max-tokens" => set_number(&mut max_tokens, value)?,
            _ => return Err("unknown context render option"),
        }
    }
    if !artifact_kind || !media_type || !validation_profile || !closure {
        return Err(
            "context render requires artifact kind, media type, validation profile and closure",
        );
    }
    let format = renderer_format.ok_or("context render requires --renderer-profile-id")?;
    let max_input_bytes = max_input_bytes.ok_or("context render requires --max-input-bytes")?;
    let max_metadata_bytes =
        max_metadata_bytes.ok_or("context render requires --max-metadata-bytes")?;
    let max_context_work = max_context_work.ok_or("context render requires --max-context-work")?;
    let max_depth = max_depth.ok_or("context render requires --max-depth")?;
    let max_tokens = max_tokens.ok_or("context render requires --max-tokens")?;
    let service = ContextRenderRequest::primary_semantic_pack_structural(
        format,
        max_input_bytes,
        max_metadata_bytes,
        max_context_work,
        max_depth,
        max_tokens,
    )
    .map_err(|_| "invalid bounded context render limits")?;
    Ok(Some(Request {
        input: input.ok_or("context render requires --input <PATH|->")?,
        service,
    }))
}

pub(super) fn run(request: &Request, stop: &AtomicBool) -> u8 {
    let limit = match usize::try_from(request.service.max_input_bytes()) {
        Ok(limit) => limit,
        Err(_) => return input_failure("context render input limit is not representable", stop),
    };
    let bytes = match read_input(&request.input, limit, stop) {
        Ok(bytes) => bytes,
        Err(_) => return input_failure("cannot read bounded context artifact", stop),
    };
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    let result = match execute_context_render(&request.service, &bytes, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(error) => {
            crate::diagnostic("context render result could not be finalized");
            return match error.code() {
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed => 4,
                _ => 3,
            };
        }
    };
    emit(&result, stop)
}

fn set_number<T>(slot: &mut Option<T>, value: &OsStr) -> Result<(), &'static str>
where
    T: TryFrom<u64>,
{
    if slot.is_some() {
        return Err("duplicate context render limit");
    }
    let text = value.to_str().ok_or("invalid context render limit")?;
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("context render limits must be unsigned decimal integers");
    }
    let parsed = text
        .parse::<u64>()
        .map_err(|_| "context render limit overflow")?;
    if parsed == 0 {
        return Err("context render limits must be positive");
    }
    let converted = T::try_from(parsed).map_err(|_| "context render limit overflow")?;
    *slot = Some(converted);
    Ok(())
}

fn emit(result: &ContextRenderResult, stop: &AtomicBool) -> u8 {
    let bytes = result.canonical_bytes();
    if bytes.is_empty() || bytes.len() > CONTEXT_MAP_RESULT_MAX_BYTES {
        crate::diagnostic("context render envelope byte limit or encoding failure");
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
    // Match the validator's cancellation contract: before the first byte, leave
    // the result unpublished; after output starts, finish the whole envelope.
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
            match stdout.write(&chunk[offset..]) {
                Ok(0) => {
                    crate::diagnostic("context render output failed");
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
                    crate::diagnostic("context render output failed");
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
        crate::diagnostic("context render output failed");
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
