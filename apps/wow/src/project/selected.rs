//! Explicit transport only: one service operation per selected command.
use crate::args::Format;
use std::{
    io::Write,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceError, ServiceErrorCode,
    graph::{
        FunctionFingerprintSelection, GRAPH_REQUEST_MAX_BYTES, GraphBuildRequest,
        XmlFingerprintSelection,
    },
    live_project::{
        LIVE_SKELETON_REQUEST_MAX_BYTES, PLATFORM_PROJECT_SELECTION_MAX_BYTES,
        PlatformProjectSelection, execute_live_skeleton_read, read_selected_live_project,
    },
};

pub(super) fn load_selection(
    path: &Path,
    stop: &AtomicBool,
) -> Result<PlatformProjectSelection, &'static str> {
    let bytes = crate::graph::read_file(path, PLATFORM_PROJECT_SELECTION_MAX_BYTES, stop)?;
    PlatformProjectSelection::from_json_slice(&bytes, stop)
        .map_err(|_| "invalid or refused platform namespace/catalog selection")
}

pub(super) fn build_queries(
    mut request: GraphBuildRequest,
    functions: Option<&Path>,
    xml: Option<&Path>,
    stop: &AtomicBool,
) -> Result<GraphBuildRequest, &'static str> {
    if let Some(path) = functions {
        let bytes = crate::graph::read_file(path, GRAPH_REQUEST_MAX_BYTES, stop)?;
        let selection = FunctionFingerprintSelection::from_json_slice_cancellable(&bytes, stop)
            .map_err(|_| "invalid or refused function query selection")?;
        request = request
            .with_function_fingerprints(selection)
            .map_err(|_| "refused function query request")?;
    }
    if let Some(path) = xml {
        let bytes = crate::graph::read_file(path, GRAPH_REQUEST_MAX_BYTES, stop)?;
        let selection = XmlFingerprintSelection::from_json_slice_cancellable(&bytes, stop)
            .map_err(|_| "invalid or refused XML query selection")?;
        request = request
            .with_xml_fingerprints(selection)
            .map_err(|_| "refused XML query request")?;
    }
    if stop.load(Ordering::Acquire) {
        return Err("project query input cancelled");
    }
    Ok(request)
}

pub(super) fn read(
    root: &Path,
    selection: &PlatformProjectSelection,
    generation: &str,
    format: Format,
    stop: &AtomicBool,
) -> u8 {
    match read_selected_live_project(root, selection, generation, stop) {
        Ok(result) => emit(result.canonical_bytes(), result.exit_code(), format, stop),
        Err(error) => service_failure(error),
    }
}

pub(super) fn skeleton(
    root: &Path,
    selection: &PlatformProjectSelection,
    generation: &str,
    request: &Path,
    format: Format,
    stop: &AtomicBool,
) -> u8 {
    let request = match crate::graph::read_file(request, LIVE_SKELETON_REQUEST_MAX_BYTES, stop) {
        Ok(bytes) => bytes,
        Err(error) => return input_failure(error, stop),
    };
    match execute_live_skeleton_read(root, selection, generation, &request, stop) {
        Ok(result) => emit(result.canonical_bytes(), result.exit_code(), format, stop),
        Err(error) => service_failure(error),
    }
}

pub(super) fn input_failure(message: &'static str, stop: &AtomicBool) -> u8 {
    if stop.load(Ordering::Acquire) {
        130
    } else {
        crate::usage(message)
    }
}
fn service_failure(error: ServiceError) -> u8 {
    crate::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
    if error.code() == ServiceErrorCode::Cancelled {
        130
    } else {
        4
    }
}
fn emit(bytes: &[u8], exit: u8, format: Format, stop: &AtomicBool) -> u8 {
    // Do not write any result bytes for a cancelled read. Streaming transport
    // loss/late cancellation cannot retract bytes already written to stdout.
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    let mut out = std::io::stdout().lock();
    let prefix: &[u8] = if format == Format::Text {
        b"Selected native Skeleton/project read (partial structural coverage)\n"
    } else {
        b""
    };
    if out
        .write_all(prefix)
        .and_then(|()| out.write_all(bytes))
        .and_then(|()| out.write_all(b"\n"))
        .and_then(|()| out.flush())
        .is_err()
    {
        crate::diagnostic("selected project read output failed");
        return 4;
    }
    exit
}
