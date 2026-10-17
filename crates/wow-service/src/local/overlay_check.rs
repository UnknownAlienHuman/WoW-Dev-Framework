use super::{
    cancelled, disk_input::acquisition_error, input::invalid, native_input::report_bytes_with_limit,
};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
use wow_core::{ContentDigest, SourceContent};
use wow_project::{
    disk::{ProjectDiskFile, ProjectInputDirectory},
    overlay::*,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    workspace_root: PathBuf,
    workspace_id: String,
    privacy: String,
    analyzer_report: ProjectDiskFile,
    limits: OverlayLimits,
    document: Document,
    changes: Vec<Change>,
    output_encoding: OverlayEncoding,
    reconcile_disk: bool,
    max_output_bytes: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    relative_path: String,
    client_version: u64,
    content: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    expected_version: u64,
    expected_content_digest: ContentDigest<SourceContent>,
    client_version: u64,
    encoding: OverlayEncoding,
    edits: Vec<OverlayEdit>,
}

pub struct OverlayCheckResult {
    bytes: Vec<u8>,
    diagnostics: usize,
}
impl OverlayCheckResult {
    pub fn canonical_bytes(&self, stop: &AtomicBool) -> ServiceResult<Vec<u8>> {
        cancelled(stop)?;
        let bytes = self.bytes.clone();
        cancelled(stop)?;
        Ok(bytes)
    }
    pub const fn diagnostic_count(&self) -> usize {
        self.diagnostics
    }
}
fn owner(error: OverlayError) -> ServiceError {
    ServiceError::new(
        match error.code() {
            OverlayErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            OverlayErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
            OverlayErrorCode::Busy => ServiceErrorCode::OperationBusy,
            OverlayErrorCode::Analyzer => ServiceErrorCode::ComponentUnavailable,
            OverlayErrorCode::ResynchronizationRequired => ServiceErrorCode::IdentityMismatch,
            _ => ServiceErrorCode::InvalidRequest,
        },
        error.to_string(),
    )
}
pub fn check_private_overlay(
    request_path: &Path,
    stop: &AtomicBool,
) -> ServiceResult<OverlayCheckResult> {
    cancelled(stop)?;
    let parent = request_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let name = request_path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| name.ends_with(".json"))
        .ok_or_else(|| invalid("private overlay requires an explicit JSON request"))?;
    let directory = ProjectInputDirectory::open(parent).map_err(acquisition_error)?;
    let bytes = directory
        .read_configuration(name, stop)
        .map_err(acquisition_error)?;
    let request: Request =
        serde_json::from_slice(&bytes).map_err(|_| invalid("invalid private overlay request"))?;
    if request.schema != "wow-service/private-overlay-check/1"
        || request.privacy != "session_private_metadata_and_diagnostics"
        || request.changes.len() > 4096
        || request.max_output_bytes == 0
        || request.max_output_bytes > 16 * 1024 * 1024
    {
        return Err(invalid("private overlay profile/bounds are invalid"));
    }
    request.limits.validate().map_err(owner)?;
    let report = directory
        .read_json_artifact(&request.analyzer_report, stop)
        .map_err(acquisition_error)?;
    let backend = wow_emmy::backend_identity_from_report(&report)
        .map_err(|_| invalid("private overlay analyzer report rejected"))?;
    let scope = OverlaySessionScope::begin().map_err(owner)?;
    let mut workspace = OverlayWorkspace::register(
        &scope,
        &request.workspace_root,
        &request.workspace_id,
        request.limits,
        stop,
    )
    .map_err(owner)?;
    let result = run(&mut workspace, &scope, request, backend, stop);
    let closed = workspace.close(&scope).map_err(owner);
    let (draft, workspace) = match (result, closed) {
        (Ok(draft), Ok(workspace)) => (draft, workspace),
        (Err(error), Ok(_)) => return Err(error),
        (_, Err(_)) => {
            return Err(ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "private overlay workspace ownership did not close",
            ));
        }
    };
    cancelled(stop)?;
    #[derive(Serialize)]
    struct ResultBody<'a> {
        schema: &'static str,
        operation: &'static str,
        resource_closure: &'static str,
        workspace: &'a OverlayWorkspaceDescriptor,
        snapshot: &'a OverlaySnapshotDescriptor,
        transitions: &'a [OverlaySnapshotDescriptor],
        save: &'a OverlaySaveObservation,
        syntax: &'a wow_emmy::EmmySyntaxReport,
        positions: &'a [Option<OverlayRange>],
        output_encoding: OverlayEncoding,
        base_state: &'static str,
        not_evaluated: &'static [&'static str],
    }
    let value = ResultBody {
        schema: "wow-service/private-overlay-check-result/1",
        operation: "private_overlay_check",
        resource_closure: "closed",
        workspace: &workspace,
        snapshot: &draft.snapshot,
        transitions: &draft.transitions,
        save: &draft.save,
        syntax: &draft.syntax,
        positions: &draft.positions,
        output_encoding: draft.output_encoding,
        base_state: "no_published_base",
        not_evaluated: &[
            "published_project_dependencies",
            "reference_platform_facts",
            "semantic_graph",
            "diagnostic_rules",
            "source_publication",
            "runtime",
        ],
    };
    let bytes = report_bytes_with_limit(&value, draft.max_output_bytes, stop)?;
    Ok(OverlayCheckResult {
        bytes,
        diagnostics: draft.syntax.diagnostics().len(),
    })
}
struct Draft {
    snapshot: OverlaySnapshotDescriptor,
    transitions: Vec<OverlaySnapshotDescriptor>,
    save: OverlaySaveObservation,
    syntax: wow_emmy::EmmySyntaxReport,
    positions: Vec<Option<OverlayRange>>,
    output_encoding: OverlayEncoding,
    max_output_bytes: usize,
}
fn run(
    workspace: &mut OverlayWorkspace,
    scope: &OverlaySessionScope,
    request: Request,
    backend: wow_emmy::EmmyBackendIdentity,
    stop: &AtomicBool,
) -> ServiceResult<Draft> {
    let uri = workspace
        .document_uri(scope, &request.document.relative_path)
        .map_err(owner)?;
    let mut current = workspace
        .open_document(
            scope,
            OverlayOpenRequest {
                relative_path: &request.document.relative_path,
                uri: &uri,
                client_version: request.document.client_version,
                content: request.document.content,
            },
            stop,
        )
        .map_err(owner)?;
    let mut transitions = Vec::new();
    for change in &request.changes {
        cancelled(stop)?;
        if change.expected_version != current.client_version
            || change.expected_content_digest != current.content_digest
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "private overlay change requires exact prior version/content",
            ));
        }
        current = workspace
            .change_document(
                scope,
                OverlayChangeRequest {
                    document_id: &current.document_id,
                    expected_snapshot: &current.snapshot_id,
                    expected_content: change.expected_content_digest,
                    client_version: change.client_version,
                    encoding: change.encoding,
                    edits: &change.edits,
                },
                stop,
            )
            .map_err(owner)?;
        transitions.push(current.clone());
    }
    let lease = workspace
        .get(scope, &current.document_id, &current.snapshot_id, stop)
        .map_err(owner)?;
    let save = if request.reconcile_disk {
        workspace
            .reconcile_disk(scope, &lease, stop)
            .map_err(owner)?
    } else {
        workspace.save_acknowledge(scope, &lease).map_err(owner)?
    };
    let input = workspace
        .analyzer_input(scope, lease, backend, stop)
        .map_err(owner)?;
    let syntax = wow_emmy::analyze_syntax_with_stop(input.workspace(), stop).map_err(|_| {
        if stop.load(std::sync::atomic::Ordering::Acquire) {
            ServiceError::new(
                ServiceErrorCode::Cancelled,
                "private overlay analysis cancelled",
            )
        } else {
            ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "private overlay analyzer refused input",
            )
        }
    })?;
    drop(input);
    let lease = workspace
        .get(scope, &current.document_id, &current.snapshot_id, stop)
        .map_err(owner)?;
    let mut offsets = Vec::new();
    for diagnostic in syntax.diagnostics() {
        let span = diagnostic.span();
        if let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) {
            offsets
                .push(usize::try_from(start).map_err(|_| invalid("diagnostic offset overflow"))?);
            offsets.push(usize::try_from(end).map_err(|_| invalid("diagnostic offset overflow"))?);
        }
    }
    let projected = workspace
        .positions(scope, &lease, &offsets, request.output_encoding, stop)
        .map_err(owner)?;
    let mut projected = projected.into_iter();
    let mut positions = Vec::new();
    for diagnostic in syntax.diagnostics() {
        let span = diagnostic.span();
        positions.push(
            if span.byte_start().is_some() && span.byte_end().is_some() {
                Some(OverlayRange {
                    start: projected
                        .next()
                        .ok_or_else(|| invalid("diagnostic position closure incomplete"))?,
                    end: projected
                        .next()
                        .ok_or_else(|| invalid("diagnostic position closure incomplete"))?,
                })
            } else {
                None
            },
        );
    }
    drop(lease);
    workspace
        .close_document(scope, &current.document_id)
        .map_err(owner)?;
    Ok(Draft {
        snapshot: current,
        transitions,
        save,
        syntax,
        positions,
        output_encoding: request.output_encoding,
        max_output_bytes: request.max_output_bytes,
    })
}
