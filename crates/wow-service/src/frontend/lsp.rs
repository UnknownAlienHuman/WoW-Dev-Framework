//! Selected synchronous private-overlay LSP use cases. No transport framing.
use super::{FrontendClientIdentity, invalid_frontend_request};
use crate::{ServiceError, ServiceErrorCode, ServiceResult, identity::canonical_digest};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use wow_project::{
    disk::{ProjectDiskFile, ProjectInputDirectory},
    overlay::*,
};

mod diagnostics;
mod registry;
pub use diagnostics::{LspDiagnostic, LspDiagnosticReport, LspOverlayBinding};
pub use registry::{LspOperationDescriptor, LspRegistry, LspTransportProfile};
pub use wow_project::overlay::{OverlayEdit as LspContentChange, OverlayRange as LspRange};

pub const LSP_PROFILE_ID: &str = "lsp-3.18-stdio-v1";
pub const LSP_MAX_HEADER_BYTES: usize = 8192;
pub const LSP_MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;
pub const LSP_MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;
pub const LSP_MAX_JSON_DEPTH: usize = 32;
pub const LSP_MAX_DIAGNOSTICS: usize = 2048;
pub const LSP_MAX_RETAINED_REPORT_BYTES: usize = LSP_MAX_OUTPUT_BYTES / 2;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HostInput {
    schema: String,
    workspace_root: PathBuf,
    workspace_id: String,
    privacy: String,
    analyzer_report: ProjectDiskFile,
    limits: OverlayLimits,
    reconcile_on_save: bool,
}
/// Explicit host configuration; no directory registration or source scan on load.
pub struct LspHost {
    input: HostInput,
    backend: wow_emmy::EmmyBackendIdentity,
}
impl LspHost {
    pub fn load(path: &Path, stop: &AtomicBool) -> ServiceResult<Self> {
        checkpoint(stop)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| n.ends_with(".json"))
            .ok_or_else(|| invalid_frontend_request("LSP requires an explicit JSON host"))?;
        let directory = ProjectInputDirectory::open(parent)
            .map_err(|_| invalid_frontend_request("LSP host directory refused"))?;
        let bytes = directory
            .read_configuration(name, stop)
            .map_err(|_| acquisition(stop))?;
        let input: HostInput = serde_json::from_slice(&bytes)
            .map_err(|_| invalid_frontend_request("invalid LSP host"))?;
        if input.schema != "wow-service/lsp-private-overlay-host/1"
            || input.privacy != "session_private_metadata_and_diagnostics"
            || input.limits.max_document_bytes > LSP_MAX_FRAME_BYTES / 2
        {
            return Err(invalid_frontend_request("unsupported LSP host profile"));
        }
        input.limits.validate().map_err(owner)?;
        let report = directory
            .read_json_artifact(&input.analyzer_report, stop)
            .map_err(|_| acquisition(stop))?;
        let backend = wow_emmy::backend_identity_from_report(&report)
            .map_err(|_| invalid_frontend_request("LSP analyzer identity refused"))?;
        checkpoint(stop)?;
        Ok(Self { input, backend })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspSessionOpenRequest {
    pub client_name: String,
    pub client_version: String,
    pub position_encodings: Vec<String>,
    pub pull_diagnostics: bool,
    pub workspace_uris: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspSessionDescriptor {
    pub schema: &'static str,
    pub session_id: Box<str>,
    pub profile_id: &'static str,
    pub registry_digest: Box<str>,
    pub profile_digest: Box<str>,
    pub client: FrontendClientIdentity,
    pub position_encoding: &'static str,
    pub workspace: OverlayWorkspaceDescriptor,
    pub coverage: &'static str,
    pub cancellation: &'static str,
    pub not_evaluated: &'static [&'static str],
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDocumentOpenRequest {
    pub uri: String,
    pub language_id: String,
    pub version: i64,
    pub content: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDocumentChangeRequest {
    pub uri: String,
    pub change: LspDocumentChange,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum LspDocumentChange {
    Apply {
        version: i64,
        changes: Vec<LspContentChange>,
    },
    Rejected {
        reason: LspSynchronizationRefusal,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LspSynchronizationRefusal {
    InvalidVersion,
    UnsupportedChangeShape,
}
struct AppliedChange {
    pub version: i64,
    pub changes: Vec<LspContentChange>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDocumentSaveRequest {
    pub uri: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
struct Head {
    snapshot: OverlaySnapshotDescriptor,
    save: Option<OverlaySaveObservation>,
    resynchronization_required: bool,
}
struct Session {
    scope: OverlaySessionScope,
    workspace: OverlayWorkspace,
    descriptor: LspSessionDescriptor,
    encoding: OverlayEncoding,
    heads: BTreeMap<Box<str>, Head>,
    diagnostic: Option<LspDiagnosticReport>,
    last_completed: Option<LspRequestId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspRequestId {
    Number(i32),
    Text(String),
}
impl LspRequestId {
    fn validate(&self) -> ServiceResult<()> {
        if matches!(self, Self::Text(value) if value.len() > 128 || value.chars().any(char::is_control))
        {
            Err(invalid_frontend_request("invalid LSP request ID"))
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Clone, Serialize)]
pub struct LspCancellationObservation {
    pub target: LspRequestId,
    pub state: &'static str,
}
/// One connection-scoped, synchronous frontend. IDs never confer access to
/// another instance; only this object owns its private Project scope token.
pub struct LspFrontend {
    host: LspHost,
    registry: LspRegistry,
    session: Option<Session>,
    opened: bool,
}
impl LspFrontend {
    pub fn new(host: LspHost) -> ServiceResult<Self> {
        Ok(Self {
            host,
            registry: LspRegistry::build()?,
            session: None,
            opened: false,
        })
    }
    pub fn registry(&self) -> &LspRegistry {
        &self.registry
    }
    pub fn frontend_session_open(
        &mut self,
        request: LspSessionOpenRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LspSessionDescriptor> {
        self.registry.require("initialize")?;
        checkpoint(stop)?;
        if self.opened
            || !request.pull_diagnostics
            || request.position_encodings.len() > 8
            || request.workspace_uris.len() > 1
        {
            return Err(invalid_frontend_request(
                "LSP session profile/initialize order refused",
            ));
        }
        let client = FrontendClientIdentity::new(request.client_name, request.client_version)?;
        if request
            .position_encodings
            .iter()
            .any(|value| value.len() > 32)
        {
            return Err(invalid_frontend_request(
                "LSP position negotiation exceeds bounds",
            ));
        }
        let (encoding, name) = if request
            .position_encodings
            .iter()
            .any(|value| value == "utf-8")
        {
            (OverlayEncoding::Utf8, "utf-8")
        } else if request.position_encodings.is_empty()
            || request
                .position_encodings
                .iter()
                .any(|value| value == "utf-16")
        {
            (OverlayEncoding::Utf16, "utf-16")
        } else {
            return Err(invalid_frontend_request(
                "unsupported LSP position encoding",
            ));
        };
        let scope = OverlaySessionScope::begin().map_err(owner)?;
        let mut workspace = OverlayWorkspace::register(
            &scope,
            &self.host.input.workspace_root,
            &self.host.input.workspace_id,
            self.host.input.limits,
            stop,
        )
        .map_err(owner)?;
        for uri in &request.workspace_uris {
            if let Err(error) = workspace.validate_workspace_uri(&scope, uri) {
                workspace.close(&scope).map_err(owner)?;
                return Err(owner(error));
            }
        }
        let descriptor = LspSessionDescriptor {
            schema: "wow-service/lsp-private-session/1",
            session_id: format!("lsp-session:{}", workspace.descriptor().session_scope).into(),
            profile_id: LSP_PROFILE_ID,
            registry_digest: self.registry.digest.clone(),
            profile_digest: self.registry.profile_digest.clone(),
            client,
            position_encoding: name,
            workspace: workspace.descriptor().clone(),
            coverage: "overlay_local_selected_syntax_no_published_base",
            cancellation: "synchronous_process_stop_owner_checkpoints",
            not_evaluated: &[
                "published_dependencies",
                "reference",
                "graph",
                "rules",
                "wire_inflight_cancellation",
                "progress",
                "reconnect",
                "runtime",
            ],
        };
        self.session = Some(Session {
            scope,
            workspace,
            descriptor: descriptor.clone(),
            encoding,
            heads: BTreeMap::new(),
            diagnostic: None,
            last_completed: None,
        });
        self.opened = true;
        Ok(descriptor)
    }
    pub fn frontend_document_open(
        &mut self,
        request: LspDocumentOpenRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<OverlaySnapshotDescriptor> {
        self.registry.require("textDocument/didOpen")?;
        checkpoint(stop)?;
        if request.language_id != "lua" || !(0..=i64::from(i32::MAX)).contains(&request.version) {
            return Err(invalid_frontend_request(
                "unsupported LSP document language/version",
            ));
        }
        let session = self.active()?;
        let path = session
            .workspace
            .relative_path_for_uri(&session.scope, &request.uri)
            .map_err(owner)?;
        if session
            .heads
            .get(&path)
            .is_some_and(|head| head.resynchronization_required)
        {
            return Err(resynchronization());
        }
        let opened = session
            .workspace
            .open_document(
                &session.scope,
                OverlayOpenRequest {
                    relative_path: &path,
                    uri: &request.uri,
                    client_version: request.version as u64,
                    content: request.content,
                },
                stop,
            )
            .map_err(owner);
        let snapshot = match opened {
            Ok(snapshot) => snapshot,
            Err(error) => {
                if let Some(head) = session.heads.get_mut(&path) {
                    head.resynchronization_required = true;
                }
                session.diagnostic = None;
                return Err(error);
            }
        };
        session.heads.entry(path).or_insert_with(|| Head {
            snapshot: snapshot.clone(),
            save: None,
            resynchronization_required: false,
        });
        Ok(snapshot)
    }
    pub fn frontend_document_change(
        &mut self,
        request: LspDocumentChangeRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<OverlaySnapshotDescriptor> {
        self.registry.require("textDocument/didChange")?;
        checkpoint(stop)?;
        let session = self.active()?;
        let path = session
            .workspace
            .relative_path_for_uri(&session.scope, &request.uri)
            .map_err(owner)?;
        let request = match request.change {
            LspDocumentChange::Apply { version, changes } => AppliedChange { version, changes },
            LspDocumentChange::Rejected { .. } => {
                session
                    .heads
                    .get_mut(&path)
                    .ok_or_else(missing)?
                    .resynchronization_required = true;
                session.diagnostic = None;
                return Err(resynchronization());
            }
        };
        if !(0..=i64::from(i32::MAX)).contains(&request.version) {
            session
                .heads
                .get_mut(&path)
                .ok_or_else(missing)?
                .resynchronization_required = true;
            session.diagnostic = None;
            return Err(resynchronization());
        }
        let head = session.heads.get(&path).ok_or_else(missing)?;
        if head.resynchronization_required {
            return Err(resynchronization());
        }
        let prior = &head.snapshot;
        let result = session
            .workspace
            .change_document(
                &session.scope,
                OverlayChangeRequest {
                    document_id: &prior.document_id,
                    expected_snapshot: &prior.snapshot_id,
                    expected_content: prior.content_digest,
                    client_version: request.version as u64,
                    encoding: session.encoding,
                    edits: &request.changes,
                },
                stop,
            )
            .map_err(owner);
        let snapshot = match result {
            Ok(snapshot) => snapshot,
            Err(error) => {
                session
                    .heads
                    .get_mut(&path)
                    .ok_or_else(missing)?
                    .resynchronization_required = true;
                session.diagnostic = None;
                return Err(error);
            }
        };
        session.heads.insert(
            path,
            Head {
                snapshot: snapshot.clone(),
                save: None,
                resynchronization_required: false,
            },
        );
        session.diagnostic = None;
        Ok(snapshot)
    }
    pub fn frontend_document_save(
        &mut self,
        request: LspDocumentSaveRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<OverlaySaveObservation> {
        self.registry.require("textDocument/didSave")?;
        checkpoint(stop)?;
        let reconcile = self.host.input.reconcile_on_save;
        let session = self.active()?;
        let path = session
            .workspace
            .relative_path_for_uri(&session.scope, &request.uri)
            .map_err(owner)?;
        let head = session.heads.get(&path).ok_or_else(missing)?;
        if head.resynchronization_required {
            return Err(resynchronization());
        }
        if let Some(content) = request.content {
            use sha2::{Digest, Sha256};
            if content.len() != head.snapshot.byte_length
                || wow_core::ContentDigest::<wow_core::SourceContent>::from_bytes(
                    Sha256::digest(content.as_bytes()).into(),
                ) != head.snapshot.content_digest
            {
                session
                    .heads
                    .get_mut(&path)
                    .ok_or_else(missing)?
                    .resynchronization_required = true;
                session.diagnostic = None;
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "LSP save content differs from exact overlay",
                ));
            }
        }
        let snapshot = head.snapshot.clone();
        // A new save attempt supersedes the prior disk observation even when
        // acquisition fails. It must not replay an old DiskReconciled result.
        session.heads.get_mut(&path).ok_or_else(missing)?.save = None;
        session.diagnostic = None;
        let lease = session
            .workspace
            .get(
                &session.scope,
                &snapshot.document_id,
                &snapshot.snapshot_id,
                stop,
            )
            .map_err(owner)?;
        let observation = if reconcile {
            session
                .workspace
                .reconcile_disk(&session.scope, &lease, stop)
        } else {
            session.workspace.save_acknowledge(&session.scope, &lease)
        }
        .map_err(owner)?;
        drop(lease);
        session.heads.get_mut(&path).ok_or_else(missing)?.save = Some(observation.clone());
        session.diagnostic = None;
        Ok(observation)
    }
    pub fn frontend_document_close(&mut self, uri: &str, stop: &AtomicBool) -> ServiceResult<()> {
        self.registry.require("textDocument/didClose")?;
        checkpoint(stop)?;
        let session = self.active()?;
        let path = session
            .workspace
            .relative_path_for_uri(&session.scope, uri)
            .map_err(owner)?;
        let head = session.heads.get(&path).ok_or_else(missing)?;
        session
            .workspace
            .close_document(&session.scope, &head.snapshot.document_id)
            .map_err(owner)?;
        session.heads.remove(&path);
        session.diagnostic = None;
        Ok(())
    }
    pub fn frontend_session_close(&mut self) -> ServiceResult<()> {
        self.registry.require("shutdown")?;
        if let Some(session) = self.session.as_mut() {
            session.workspace.close(&session.scope).map_err(owner)?;
        }
        self.session.take();
        Ok(())
    }
    pub fn frontend_operation_cancel(
        &mut self,
        target: LspRequestId,
    ) -> ServiceResult<LspCancellationObservation> {
        self.registry.require("$/cancelRequest")?;
        target.validate()?;
        let session = self.active()?;
        let state = if session.last_completed.as_ref() == Some(&target) {
            "completed_before_cancel"
        } else {
            "not_observed_no_cancellation_claim"
        };
        Ok(LspCancellationObservation { target, state })
    }
    fn active(&mut self) -> ServiceResult<&mut Session> {
        self.session.as_mut().ok_or_else(|| {
            ServiceError::new(ServiceErrorCode::InvalidContext, "LSP session is not open")
        })
    }
}
fn checkpoint(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(ServiceError::new(
            ServiceErrorCode::Cancelled,
            "LSP operation cancelled",
        ))
    } else {
        Ok(())
    }
}
fn acquisition(stop: &AtomicBool) -> ServiceError {
    if stop.load(Ordering::Acquire) {
        ServiceError::new(
            ServiceErrorCode::Cancelled,
            "LSP host acquisition cancelled",
        )
    } else {
        invalid_frontend_request("LSP host artifact acquisition refused")
    }
}
fn missing() -> ServiceError {
    invalid_frontend_request("LSP document is not registered")
}
fn resynchronization() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::IdentityMismatch,
        "LSP document requires didClose/didOpen resynchronization",
    )
}
fn owner(error: OverlayError) -> ServiceError {
    ServiceError::new(
        match error.code() {
            OverlayErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            OverlayErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
            OverlayErrorCode::Busy => ServiceErrorCode::OperationBusy,
            OverlayErrorCode::Closed | OverlayErrorCode::ScopeMismatch => {
                ServiceErrorCode::InvalidContext
            }
            OverlayErrorCode::ResynchronizationRequired => ServiceErrorCode::IdentityMismatch,
            OverlayErrorCode::Analyzer => ServiceErrorCode::ComponentUnavailable,
            _ => ServiceErrorCode::InvalidRequest,
        },
        error.to_string(),
    )
}
