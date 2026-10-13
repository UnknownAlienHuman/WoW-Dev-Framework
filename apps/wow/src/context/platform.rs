//! Native platform map/inspect transport. Selection and authority are service-owned.
use super::{Arguments, Command, MAX_VALUE_BYTES, set};
use std::{
    ffi::{OsStr, OsString},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_MAP_RESULT_MAX_BYTES, CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES,
        CONTEXT_PLATFORM_REQUEST_MAX_BYTES, ContextMapStatus, ContextPlatformHostInput,
        ContextPlatformOperation, ContextPlatformRequest, ContextPlatformResult,
        execute_context_platform,
    },
};

#[derive(Clone, Copy)]
enum Output {
    EnvelopeJson,
    Text,
}

pub(super) struct Request {
    command: Command,
    request_input: PathBuf,
    output: Output,
}

/// Call after the parent's existing aggregate argv preflight, before consuming values.
/// Option positions only: a Primary path/value named like a flag must not select Native.
pub(super) fn parse(values: &[OsString]) -> Result<Option<Arguments>, &'static str> {
    let Some(options) = values.get(2..) else {
        return Ok(None);
    };
    let selected = options.chunks(2).any(|pair| {
        pair.first()
            .is_some_and(|flag| flag == "--platform-request-input")
            || (pair.first().is_some_and(|flag| flag == "--platform")
                && pair.get(1).is_some_and(|value| value == "native"))
    });
    if !selected {
        return Ok(None);
    }
    if values.first().map(OsString::as_os_str) != Some(OsStr::new("context")) {
        return Err("expected context map or inspect");
    }
    let command = match values.get(1).and_then(|value| value.to_str()) {
        Some("map") => Command::Map,
        Some("inspect") => Command::Inspect,
        _ => return Err("native platform supports context map or inspect only"),
    };
    let mut host_config = None;
    let mut request_input = None;
    let mut platform = None;
    let mut output = None;
    let mut options = options.iter();
    while let Some(option) = options.next() {
        let option = option
            .to_str()
            .ok_or("invalid native context option encoding")?;
        if !matches!(
            option,
            "--native-host-config" | "--platform" | "--platform-request-input" | "--output"
        ) {
            return Err("native platform context rejects primary or unknown options");
        }
        let value = options
            .next()
            .ok_or("missing native context option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid native context option length");
        }
        match option {
            "--native-host-config" => {
                set(&mut host_config, PathBuf::from(value.as_os_str()), command)?;
            }
            "--platform-request-input" => {
                set(
                    &mut request_input,
                    PathBuf::from(value.as_os_str()),
                    command,
                )?;
            }
            "--platform" => {
                if value != "native" {
                    return Err("native platform context rejects mixed platform selection");
                }
                set(&mut platform, (), command)?;
            }
            "--output" => {
                let value = value
                    .to_str()
                    .ok_or("invalid native context value encoding")?;
                let selected = match value {
                    "envelope-json" => Output::EnvelopeJson,
                    "text" => Output::Text,
                    // E3-C forbids artifact mode for map/inspect.
                    _ => {
                        return Err(
                            "native context map/inspect output must be envelope-json or text",
                        );
                    }
                };
                set(&mut output, selected, command)?;
            }
            _ => return Err("unknown native context option"),
        }
    }
    platform.ok_or("platform request input requires --platform native")?;
    Ok(Some(Arguments {
        source_root: None,
        host_config: host_config.ok_or("native context requires explicit native host input")?,
        request: super::Request::Platform(Request {
            command,
            request_input: request_input
                .ok_or("native context requires --platform-request-input")?,
            output: output.unwrap_or(Output::EnvelopeJson),
        }),
    }))
}

fn operation_matches(command: Command, operation: ContextPlatformOperation) -> bool {
    matches!(
        (command, operation),
        (Command::Map, ContextPlatformOperation::Map)
            | (Command::Inspect, ContextPlatformOperation::Inspect)
    )
}

pub(super) fn run(host_path: &Path, arguments: &Request, stop: &AtomicBool) -> u8 {
    let request_bytes = match crate::graph::read_file(
        &arguments.request_input,
        CONTEXT_PLATFORM_REQUEST_MAX_BYTES,
        stop,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return super::input_failure("cannot read bounded platform request input", stop),
    };
    let request = match ContextPlatformRequest::from_json_slice(&request_bytes, stop) {
        Ok(request) => request,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return super::input_failure("invalid platform context request", stop),
    };
    drop(request_bytes);
    if !operation_matches(arguments.command, request.operation()) {
        return super::input_failure("platform request operation conflicts with command", stop);
    }
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
    // One semantic operation: the service owns acquisition, exact guards and checked closure.
    let result = match execute_context_platform(&host, &request, stop) {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("platform context operation could not finalize a result");
            return 4;
        }
    };
    if result.canonical_bytes().is_empty()
        || result.canonical_bytes().len() > CONTEXT_MAP_RESULT_MAX_BYTES
    {
        crate::diagnostic("platform context result byte limit or encoding failure");
        return 4;
    }
    let exit = match result.status() {
        ContextMapStatus::Complete => 0,
        ContextMapStatus::Partial | ContextMapStatus::Truncated => 2,
        ContextMapStatus::Cancelled => 130,
        ContextMapStatus::Failed => match result.failure_code() {
            Some(
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed,
            ) => 4,
            _ => 3,
        },
    };
    // Preflight the entire text before stdout; canonical bytes remain borrowed and exact.
    let text = match arguments.output {
        Output::EnvelopeJson => None,
        Output::Text => match render_text(arguments.command, &result) {
            Ok(bytes) => Some(bytes),
            Err(()) => {
                crate::diagnostic("platform context text output limit or encoding failure");
                return if stop.load(Ordering::Acquire) { 130 } else { 4 };
            }
        },
    };
    let mut output = std::io::stdout().lock();
    let written = match text {
        Some(bytes) => output.write_all(&bytes),
        None => output
            .write_all(result.canonical_bytes())
            .and_then(|()| output.write_all(b"\n")),
    };
    if written.and_then(|()| output.flush()).is_err() {
        crate::diagnostic("platform context output failed");
        return if stop.load(Ordering::Acquire) { 130 } else { 4 };
    }
    exit
}

fn render_text(command: Command, result: &ContextPlatformResult) -> Result<Vec<u8>, ()> {
    struct TextOutput(Vec<u8>);
    impl Write for TextOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let length = self
                .0
                .len()
                .checked_add(bytes.len())
                .ok_or_else(|| io::Error::other("platform context text size overflow"))?;
            if length > CONTEXT_MAP_RESULT_MAX_BYTES {
                return Err(io::Error::other("platform context text output limit"));
            }
            self.0.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let command = match command {
        Command::Map => "map",
        Command::Inspect => "inspect",
        Command::Build => return Err(()),
    };
    let mut output = TextOutput(Vec::new());
    writeln!(output, "context platform {command}: {:?}", result.status()).map_err(|_| ())?;
    writeln!(output, "failure_code: {:?}", result.failure_code()).map_err(|_| ())?;
    writeln!(output, "resource_closure: {}", result.resource_closure()).map_err(|_| ())?;
    writeln!(output, "summary:").map_err(|_| ())?;
    // Preserve all public typed counts, IDs and NotEvaluated classes. A missing
    // summary stays null; it never becomes zero counts or authoritative absence.
    serde_json::to_writer_pretty(&mut output, &result.summary()).map_err(|_| ())?;
    output.write_all(b"\n").map_err(|_| ())?;
    Ok(output.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments(command: &str) -> Vec<OsString> {
        [
            "context",
            command,
            "--platform",
            "native",
            "--native-host-config",
            "native-host.json",
            "--platform-request-input",
            "native-request.json",
        ]
        .into_iter()
        .map(OsString::from)
        .collect()
    }

    #[test]
    fn native_transport_preserves_explicit_paths_and_operation() -> Result<(), &'static str> {
        for (command, operation) in [
            ("map", ContextPlatformOperation::Map),
            ("inspect", ContextPlatformOperation::Inspect),
        ] {
            let parsed = super::super::parse(arguments(command))?;
            assert_eq!(parsed.host_config, PathBuf::from("native-host.json"));
            let super::super::Request::Platform(request) = parsed.request else {
                return Err("native parser selected primary request");
            };
            assert_eq!(request.request_input, PathBuf::from("native-request.json"));
            assert!(matches!(request.output, Output::EnvelopeJson));
            assert!(operation_matches(request.command, operation));
            let mut text = arguments(command);
            text.extend([OsString::from("--output"), OsString::from("text")]);
            let parsed = super::super::parse(text)?;
            assert!(matches!(
                parsed.request,
                super::super::Request::Platform(Request {
                    output: Output::Text,
                    ..
                })
            ));
        }
        assert!(!operation_matches(
            Command::Map,
            ContextPlatformOperation::Inspect
        ));
        assert!(!operation_matches(
            Command::Inspect,
            ContextPlatformOperation::Map
        ));
        assert!(!operation_matches(
            Command::Build,
            ContextPlatformOperation::Map
        ));
        // Option positions only: a Primary host path can equal a Native flag.
        assert!(
            parse(&[
                "context".into(),
                "map".into(),
                "--native-host-config".into(),
                "--platform-request-input".into(),
                "--platform".into(),
                "omitted".into(),
            ])?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn native_transport_refuses_mixed_duplicate_missing_and_artifact_options()
    -> Result<(), &'static str> {
        for command in ["map", "inspect"] {
            for (flag, value) in [
                ("--platform", "native"),
                ("--platform", "omitted"),
                ("--native-host-config", "other.json"),
                ("--platform-request-input", "other.json"),
                ("--project-store", "primary-store"),
                ("--root", "project@YQ"),
                ("--map-profile-id", "primary"),
                ("--level", "l1"),
                ("--output", "artifact"),
            ] {
                let mut values = arguments(command);
                values.extend([OsString::from(flag), OsString::from(value)]);
                assert!(super::super::parse(values).is_err());
            }
            for flag in [
                "--platform",
                "--native-host-config",
                "--platform-request-input",
            ] {
                let mut values = arguments(command);
                let index = values
                    .iter()
                    .position(|value| value == flag)
                    .ok_or("missing fixture option")?;
                values.drain(index..index + 2);
                assert!(super::super::parse(values).is_err());
            }
            let mut missing = arguments(command);
            missing.push("--output".into());
            assert!(super::super::parse(missing).is_err());
            let mut duplicate = arguments(command);
            for _ in 0..2 {
                duplicate.extend([OsString::from("--output"), OsString::from("text")]);
            }
            assert!(super::super::parse(duplicate).is_err());
        }
        assert!(super::super::parse(arguments("build")).is_err());
        Ok(())
    }
}
