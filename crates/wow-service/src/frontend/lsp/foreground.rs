//! Service-owned tickets for the explicitly selected foreground LSP profile.
//! Transport framing, progress writes, and worker ownership remain in the app.
use super::*;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, MutexGuard, atomic::AtomicBool},
};

pub const LSP_OWNED_PROFILE_ID: &str = "lsp-3.18-stdio-owned-v1";
pub const LSP_FOREGROUND_MAX_QUEUED: usize = 8;
pub const LSP_FOREGROUND_MAX_METADATA: usize = 16;
pub const LSP_FOREGROUND_MAX_REQUEST_BYTES: usize = 64 * 1024;
pub const LSP_FOREGROUND_MAX_QUEUE_BYTES: usize =
    LSP_FOREGROUND_MAX_QUEUED * LSP_FOREGROUND_MAX_REQUEST_BYTES;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LspForegroundExecutionProfile {
    pub max_queued_messages: usize,
    pub max_queued_message_bytes: usize,
    pub max_outbound_bytes: usize,
    pub max_progress_events: usize,
    pub max_progress_bytes: usize,
    pub partial_frame_idle_ms: u64,
    pub partial_frame_absolute_ms: u64,
    pub output_idle_ms: u64,
    pub output_absolute_ms: u64,
    pub request_idle_ms: u64,
    pub request_absolute_ms: u64,
    pub owner_preemption: bool,
    pub durable_result_replay: bool,
}
pub const LSP_FOREGROUND_EXECUTION: LspForegroundExecutionProfile = LspForegroundExecutionProfile {
    max_queued_messages: 8,
    max_queued_message_bytes: 8 * 1024 * 1024,
    max_outbound_bytes: 8 * 1024 * 1024,
    max_progress_events: 16,
    max_progress_bytes: 32 * 1024,
    partial_frame_idle_ms: 5000,
    partial_frame_absolute_ms: 30000,
    output_idle_ms: 5000,
    output_absolute_ms: 30000,
    request_idle_ms: 30000,
    request_absolute_ms: 30000,
    owner_preemption: false,
    durable_result_replay: false,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LspProgressToken {
    String(String),
    Number(i32),
}

#[derive(Debug, Clone, Serialize)]
pub enum LspReadRequest {
    Diagnostics {
        uri: String,
        previous_result_id: Option<String>,
    },
    Definition(LspDefinitionRequest),
    References(LspReferencesRequest),
    DocumentSymbols(LspDocumentSymbolsRequest),
}

impl LspReadRequest {
    fn uri(&self) -> &str {
        match self {
            Self::Diagnostics { uri, .. } => uri,
            Self::Definition(request) => &request.uri,
            Self::References(request) => &request.uri,
            Self::DocumentSymbols(request) => &request.uri,
        }
    }

    fn operation(&self) -> LspForegroundOperation {
        match self {
            Self::Diagnostics { .. } => LspForegroundOperation::Diagnostics,
            Self::Definition(_) => LspForegroundOperation::Definition,
            Self::References(_) => LspForegroundOperation::References,
            Self::DocumentSymbols(_) => LspForegroundOperation::DocumentSymbols,
        }
    }

    fn wire_method(&self) -> &'static str {
        match self {
            Self::Diagnostics { .. } => "textDocument/diagnostic",
            Self::Definition(_) => "textDocument/definition",
            Self::References(_) => "textDocument/references",
            Self::DocumentSymbols(_) => "textDocument/documentSymbol",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundOperation {
    Diagnostics,
    Definition,
    References,
    DocumentSymbols,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundTicketState {
    Queued,
    Dispatched,
    Completed,
    Failed,
    FailedBeforeOwner,
    RefusedNotReady,
    RefusedSessionClosed,
    CancelledBeforeDispatch,
    OwnerAcknowledged,
    CompletedBeforeCancel,
    ConnectionLostAbandoned,
    ProcessStopped,
    DroppedBeforeDispatch,
    DroppedBeforeOwner,
    OwnerOutcomeUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundCancellationState {
    NotRequested,
    Requested,
    ForwardedToOwner,
    OwnerAcknowledged,
    CompletedBeforeCancel,
    CancelledBeforeDispatch,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspForegroundCancellationObservation {
    pub target: LspRequestId,
    pub state: LspForegroundCancellationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundDeliveryState {
    Prepared,
    Sent,
    LostBefore,
    LostAfterPossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundStopReason {
    ProcessStop,
    ConnectionLost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LspForegroundCancelOrigin {
    ClientRequest,
    RequestDeadline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspReadRefusal {
    NotReady,
    SessionClosed,
}

/// Privacy-safe binding; it contains identity and digest metadata, never a URI,
/// source body, project handle, or owner handle.
#[derive(Debug, Clone, Serialize)]
pub struct LspForegroundOverlayBinding {
    pub document_version: u64,
    pub snapshot_id: Box<str>,
    pub content_digest: Box<str>,
    pub digest: Box<str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LspForegroundTicket {
    pub schema: &'static str,
    pub ticket_id: Box<str>,
    pub wire_id: LspRequestId,
    pub service_request_id: Box<str>,
    pub nonce: u64,
    pub session_id: Box<str>,
    pub profile_id: &'static str,
    pub registry_digest: Box<str>,
    pub request_digest: Box<str>,
    pub operation: LspForegroundOperation,
    pub progress_token: Option<LspProgressToken>,
    pub progress_events_coalesced: u32,
    pub overlay: Option<LspForegroundOverlayBinding>,
    pub state: LspForegroundTicketState,
    pub cancellation: LspForegroundCancellationState,
    pub cancel_origin: Option<LspForegroundCancelOrigin>,
    pub stop_reason: Option<LspForegroundStopReason>,
    pub delivery: LspForegroundDeliveryState,
    pub result_digest: Option<Box<str>>,
    pub digest: Box<str>,
}

#[derive(Debug, Serialize)]
pub enum LspReadResult {
    Diagnostics(LspDiagnosticReport),
    Definition(LspDefinitionResult),
    References(LspReferencesResult),
    DocumentSymbols(LspDocumentSymbolsResult),
}

#[derive(Debug)]
pub struct LspForegroundResult {
    pub ticket: LspForegroundTicket,
    pub result: Result<LspReadResult, ServiceError>,
}

pub enum LspReadDispatch {
    Ready(Box<LspDispatchedRead>),
    Finished(Box<LspForegroundResult>),
}

#[derive(Clone)]
struct SessionBinding {
    session_id: Box<str>,
    profile_id: &'static str,
    registry_digest: Box<str>,
    definition_enabled: bool,
    references_enabled: bool,
    document_symbols_enabled: bool,
}

struct TicketRecord {
    ticket: LspForegroundTicket,
    stop: Arc<AtomicBool>,
    request_bytes: usize,
}

pub(super) struct LspForegroundState {
    session: Option<SessionBinding>,
    stopped: Option<LspForegroundStopReason>,
    next_nonce: u64,
    queue: VecDeque<u64>,
    queued_bytes: usize,
    active: Option<u64>,
    records: VecDeque<TicketRecord>,
}

impl LspForegroundState {
    pub(super) fn new() -> Self {
        Self {
            session: None,
            stopped: None,
            next_nonce: 1,
            queue: VecDeque::new(),
            queued_bytes: 0,
            active: None,
            records: VecDeque::new(),
        }
    }

    pub(super) fn bind_session(&mut self, descriptor: &LspSessionDescriptor) {
        self.session = Some(SessionBinding {
            session_id: descriptor.session_id.clone(),
            profile_id: descriptor.profile_id,
            registry_digest: descriptor.registry_digest.clone(),
            definition_enabled: descriptor.local_definition_enabled,
            references_enabled: descriptor.local_references_enabled == Some(true),
            document_symbols_enabled: descriptor.document_symbol_disclosure
                == Some(LSP_DOCUMENT_SYMBOL_DISCLOSURE)
                && descriptor.document_symbols_client.is_some(),
        });
    }
}

pub(super) fn ensure_session_can_close(core: &Mutex<LspForegroundState>) -> ServiceResult<()> {
    let state = lock(core)?;
    if state.active.is_some()
        || !state.queue.is_empty()
        || state
            .records
            .iter()
            .any(|record| record.ticket.delivery == LspForegroundDeliveryState::Prepared)
    {
        return Err(ServiceError::new(
            ServiceErrorCode::OperationBusy,
            "foreground LSP reads and deliveries must drain before session close",
        ));
    }
    Ok(())
}

pub(super) fn ensure_no_active_owner_call(core: &Mutex<LspForegroundState>) -> ServiceResult<()> {
    if lock(core)?.active.is_some() {
        return Err(ServiceError::new(
            ServiceErrorCode::OperationBusy,
            "foreground LSP owner call is active",
        ));
    }
    Ok(())
}

pub(super) fn close_session(core: &Mutex<LspForegroundState>) -> ServiceResult<()> {
    lock(core)?.session = None;
    Ok(())
}

pub struct LspForegroundControl {
    pub(super) core: Arc<Mutex<LspForegroundState>>,
    pub(super) document_symbols_enabled: bool,
}

impl LspForegroundControl {
    /// Immutable host capability for App framing before session initialization.
    /// This is not session admission or source authorization from client data.
    pub fn document_symbols_enabled(&self) -> bool {
        self.document_symbols_enabled
    }
    pub fn prepare(
        &self,
        wire_id: LspRequestId,
        request: LspReadRequest,
        progress_token: Option<LspProgressToken>,
    ) -> ServiceResult<LspPreparedRead> {
        wire_id.validate()?;
        if request.uri().is_empty() || request.uri().len() > 65_536 {
            return Err(invalid_frontend_request(
                "foreground LSP URI exceeds profile bounds",
            ));
        }
        if matches!(&request, LspReadRequest::Diagnostics { previous_result_id: Some(id), .. } if id.len() > 256)
        {
            return Err(invalid_frontend_request("invalid diagnostic result ID"));
        }
        if matches!(&progress_token, Some(LspProgressToken::String(value)) if value.len() > 128 || value.chars().any(char::is_control))
        {
            return Err(invalid_frontend_request(
                "invalid LSP work-done progress token",
            ));
        }
        let serialized = serde_json::to_vec(&request).map_err(|_| {
            invalid_frontend_request("foreground LSP request serialization refused")
        })?;
        let request_bytes = serialized.len();
        if request_bytes > LSP_FOREGROUND_MAX_REQUEST_BYTES {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "foreground LSP request exceeds profile byte limit",
            ));
        }
        let request_digest = canonical_digest("lsp-foreground-request:sha256:", &request)?;
        let mut state = lock(&self.core)?;
        let session = state.session.as_ref().cloned().ok_or_else(|| {
            ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "foreground LSP session is not open",
            )
        })?;
        if state.stopped.is_some() || session.profile_id != LSP_OWNED_PROFILE_ID {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "foreground LSP admission is closed",
            ));
        }
        match request.operation() {
            LspForegroundOperation::Definition if !session.definition_enabled => {
                return Err(ServiceError::new(
                    ServiceErrorCode::ComponentUnavailable,
                    "local definition is disabled by the session profile",
                ));
            }
            LspForegroundOperation::References if !session.references_enabled => {
                return Err(ServiceError::new(
                    ServiceErrorCode::ComponentUnavailable,
                    "local references are disabled by the session profile",
                ));
            }
            LspForegroundOperation::DocumentSymbols if !session.document_symbols_enabled => {
                return Err(ServiceError::new(
                    ServiceErrorCode::ComponentUnavailable,
                    "document outline disclosure is disabled by the session profile",
                ));
            }
            _ => {}
        }
        if state.records.iter().any(|record| {
            record.ticket.wire_id == wire_id
                && record.ticket.delivery != LspForegroundDeliveryState::Sent
        }) {
            return Err(ServiceError::new(
                ServiceErrorCode::OperationConflict,
                "duplicate outstanding LSP wire request ID",
            ));
        }
        if state.queue.len() >= LSP_FOREGROUND_MAX_QUEUED
            || state
                .queued_bytes
                .checked_add(request_bytes)
                .is_none_or(|bytes| bytes > LSP_FOREGROUND_MAX_QUEUE_BYTES)
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "foreground LSP queue limit exceeded",
            ));
        }
        prune_metadata(&mut state);
        if state.records.len() >= LSP_FOREGROUND_MAX_METADATA {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "foreground LSP metadata journal is full",
            ));
        }
        let nonce = state.next_nonce;
        if nonce > 9_007_199_254_740_991 {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "foreground LSP nonce exceeds the JSON integer profile",
            ));
        }
        let ticket_id = canonical_digest(
            "lsp-foreground-ticket:sha256:",
            &(
                &session.session_id,
                &session.registry_digest,
                nonce,
                &wire_id,
                &request_digest,
                &progress_token,
            ),
        )?;
        let service_request_id = canonical_digest(
            "lsp-service-request:sha256:",
            &(
                &session.session_id,
                &session.registry_digest,
                nonce,
                &request_digest,
            ),
        )?;
        state.next_nonce = state.next_nonce.checked_add(1).ok_or_else(|| {
            ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "foreground LSP nonce exhausted",
            )
        })?;
        let ticket = LspForegroundTicket {
            schema: if session.document_symbols_enabled {
                "wow-service/lsp-foreground-ticket/2"
            } else {
                "wow-service/lsp-foreground-ticket/1"
            },
            ticket_id,
            wire_id,
            service_request_id,
            nonce,
            session_id: session.session_id.clone(),
            profile_id: session.profile_id,
            registry_digest: session.registry_digest.clone(),
            request_digest,
            operation: request.operation(),
            progress_token,
            progress_events_coalesced: 0,
            overlay: None,
            state: LspForegroundTicketState::Queued,
            cancellation: LspForegroundCancellationState::NotRequested,
            cancel_origin: None,
            stop_reason: None,
            delivery: LspForegroundDeliveryState::Prepared,
            result_digest: None,
            digest: "".into(),
        };
        let ticket = ticket_snapshot(&ticket)?;
        state.queued_bytes += request_bytes;
        state.queue.push_back(nonce);
        state.records.push_back(TicketRecord {
            ticket: ticket.clone(),
            stop: Arc::new(AtomicBool::new(false)),
            request_bytes,
        });
        Ok(LspPreparedRead {
            core: self.core.clone(),
            ticket,
            request: Some(request),
            consumed: false,
        })
    }

    pub fn cancel(
        &self,
        wire_id: LspRequestId,
    ) -> ServiceResult<LspForegroundCancellationObservation> {
        let state = self.cancel_from(wire_id.clone(), LspForegroundCancelOrigin::ClientRequest)?;
        Ok(LspForegroundCancellationObservation {
            target: wire_id,
            state,
        })
    }

    pub fn request_timeout(
        &self,
        wire_id: LspRequestId,
    ) -> ServiceResult<LspForegroundCancellationState> {
        self.cancel_from(wire_id, LspForegroundCancelOrigin::RequestDeadline)
    }

    fn cancel_from(
        &self,
        wire_id: LspRequestId,
        origin: LspForegroundCancelOrigin,
    ) -> ServiceResult<LspForegroundCancellationState> {
        wire_id.validate()?;
        let mut state = lock(&self.core)?;
        let Some(index) = state.records.iter().rposition(|record| {
            record.ticket.wire_id == wire_id
                && record.ticket.delivery == LspForegroundDeliveryState::Prepared
                && matches!(
                    record.ticket.state,
                    LspForegroundTicketState::Queued | LspForegroundTicketState::Dispatched
                )
        }) else {
            return Ok(
                if state.records.iter().any(|record| {
                    record.ticket.wire_id == wire_id && record.ticket.result_digest.is_some()
                }) {
                    LspForegroundCancellationState::CompletedBeforeCancel
                } else {
                    LspForegroundCancellationState::NotRequested
                },
            );
        };
        let record = &mut state.records[index];
        if record.ticket.cancel_origin.is_none() {
            record.ticket.cancel_origin = Some(origin);
        }
        record.ticket.cancellation = if record.ticket.state == LspForegroundTicketState::Dispatched
        {
            LspForegroundCancellationState::ForwardedToOwner
        } else {
            LspForegroundCancellationState::Requested
        };
        record
            .stop
            .store(true, std::sync::atomic::Ordering::Release);
        Ok(record.ticket.cancellation)
    }

    pub fn stop(&self, reason: LspForegroundStopReason) -> ServiceResult<()> {
        let mut state = lock(&self.core)?;
        state.stopped = Some(match (state.stopped, reason) {
            (_, LspForegroundStopReason::ConnectionLost) => LspForegroundStopReason::ConnectionLost,
            (Some(existing), _) => existing,
            (None, requested) => requested,
        });
        let applied = state.stopped;
        for record in &mut state.records {
            if record.ticket.delivery == LspForegroundDeliveryState::Prepared {
                record.ticket.stop_reason = applied;
                record
                    .stop
                    .store(true, std::sync::atomic::Ordering::Release);
                if reason == LspForegroundStopReason::ConnectionLost {
                    record.ticket.delivery = LspForegroundDeliveryState::LostBefore;
                }
            }
        }
        Ok(())
    }

    pub fn record_delivery(
        &self,
        ticket_id: &str,
        delivery: LspForegroundDeliveryState,
    ) -> ServiceResult<()> {
        if delivery == LspForegroundDeliveryState::Prepared {
            return Err(invalid_frontend_request("delivery state must be final"));
        }
        let mut state = lock(&self.core)?;
        {
            let record = state
                .records
                .iter_mut()
                .find(|record| record.ticket.ticket_id.as_ref() == ticket_id)
                .ok_or_else(|| invalid_frontend_request("foreground LSP ticket is not retained"))?;
            if record.ticket.result_digest.is_none()
                || record.ticket.delivery != LspForegroundDeliveryState::Prepared
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::InvalidContext,
                    "foreground LSP result is not awaiting delivery",
                ));
            }
            record.ticket.delivery = delivery;
        }
        prune_metadata(&mut state);
        Ok(())
    }

    pub fn ticket_metadata(&self, ticket_id: &str) -> Option<LspForegroundTicket> {
        if ticket_id.len() > 128 {
            return None;
        }
        lock(&self.core)
            .ok()?
            .records
            .iter()
            .find(|record| record.ticket.ticket_id.as_ref() == ticket_id)
            .and_then(|record| ticket_snapshot(&record.ticket).ok())
    }

    pub fn record_progress_coalesced(&self, ticket_id: &str) -> ServiceResult<()> {
        let mut state = lock(&self.core)?;
        let record = state
            .records
            .iter_mut()
            .find(|record| record.ticket.ticket_id.as_ref() == ticket_id)
            .ok_or_else(missing_ticket)?;
        record.ticket.progress_events_coalesced = record
            .ticket
            .progress_events_coalesced
            .checked_add(1)
            .filter(|count| *count <= 2)
            .ok_or_else(|| {
                invalid_frontend_request("foreground progress accounting exceeded begin/end")
            })?;
        Ok(())
    }
}

pub struct LspPreparedRead {
    core: Arc<Mutex<LspForegroundState>>,
    ticket: LspForegroundTicket,
    request: Option<LspReadRequest>,
    consumed: bool,
}

impl LspPreparedRead {
    pub fn ticket(&self) -> &LspForegroundTicket {
        &self.ticket
    }
    pub fn ticket_id(&self) -> &str {
        &self.ticket.ticket_id
    }
}

impl Drop for LspPreparedRead {
    fn drop(&mut self) {
        if self.consumed {
            return;
        }
        let Ok(mut state) = lock(&self.core) else {
            return;
        };
        if !remove_queued(&mut state, self.ticket.nonce) {
            return;
        }
        let bytes = if let Some(record) = record_mut(&mut state, self.ticket.nonce) {
            record.ticket.state = LspForegroundTicketState::DroppedBeforeDispatch;
            Some(record.request_bytes)
        } else {
            None
        };
        if let Some(bytes) = bytes {
            state.queued_bytes = state.queued_bytes.saturating_sub(bytes);
        }
    }
}

pub struct LspDispatchedRead {
    core: Arc<Mutex<LspForegroundState>>,
    ticket: LspForegroundTicket,
    request: Option<LspReadRequest>,
    stop: Arc<AtomicBool>,
    started: bool,
    finished: bool,
}

impl LspDispatchedRead {
    pub fn ticket(&self) -> &LspForegroundTicket {
        &self.ticket
    }
}

impl Drop for LspDispatchedRead {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        let Ok(mut state) = lock(&self.core) else {
            return;
        };
        if state.active == Some(self.ticket.nonce) {
            state.active = None;
        }
        if let Some(record) = record_mut(&mut state, self.ticket.nonce) {
            record.ticket.state = if self.started {
                LspForegroundTicketState::OwnerOutcomeUnknown
            } else {
                LspForegroundTicketState::DroppedBeforeOwner
            };
        }
    }
}

impl LspFrontend {
    pub fn frontend_refuse_read(
        &mut self,
        mut prepared: LspPreparedRead,
        refusal: LspReadRefusal,
    ) -> ServiceResult<LspForegroundResult> {
        if !self.uses_foreground(&prepared.core) {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "prepared LSP read belongs to another frontend",
            ));
        }
        let mut state = lock(&prepared.core)?;
        let Some(index) = state.records.iter().position(|record| {
            record.ticket.nonce == prepared.ticket.nonce
                && record.ticket.ticket_id == prepared.ticket.ticket_id
                && record.ticket.state == LspForegroundTicketState::Queued
        }) else {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "prepared LSP ticket is not queued",
            ));
        };
        remove_queued(&mut state, prepared.ticket.nonce);
        let bytes = state.records[index].request_bytes;
        state.queued_bytes = state.queued_bytes.saturating_sub(bytes);
        let record = &mut state.records[index];
        record.ticket.state = match refusal {
            LspReadRefusal::NotReady => LspForegroundTicketState::RefusedNotReady,
            LspReadRefusal::SessionClosed => LspForegroundTicketState::RefusedSessionClosed,
        };
        let code = ServiceErrorCode::InvalidContext;
        let message = match refusal {
            LspReadRefusal::NotReady => "LSP protocol is not ready",
            LspReadRefusal::SessionClosed => "LSP protocol session is closed",
        };
        let result = Err(ServiceError::new(code, message));
        record.ticket.result_digest = Some(result_digest(&result)?);
        let ticket = ticket_snapshot(&record.ticket)?;
        let output = LspForegroundResult { ticket, result };
        prepared.consumed = true;
        prepared.request.take();
        Ok(output)
    }

    pub fn frontend_dispatch_read(
        &mut self,
        mut prepared: LspPreparedRead,
    ) -> ServiceResult<LspReadDispatch> {
        if !self.uses_foreground(&prepared.core) {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "prepared LSP read belongs to another frontend",
            ));
        }
        if let Some(result) = self.finish_queued_stop(&mut prepared)? {
            return Ok(LspReadDispatch::Finished(Box::new(result)));
        }
        let request = prepared
            .request
            .as_ref()
            .ok_or_else(|| invalid_frontend_request("prepared LSP read was consumed"))?;
        let overlay = match self
            .registry
            .require(request.wire_method())
            .and_then(|()| self.foreground_overlay(request))
        {
            Ok(overlay) => overlay,
            Err(error) => {
                return self
                    .finish_prepared_error(&mut prepared, error)
                    .map(|result| LspReadDispatch::Finished(Box::new(result)));
            }
        };
        let (stop, ticket) = {
            let mut state = lock(&prepared.core)?;
            let session_matches = state.session.as_ref().is_some_and(|session| {
                session.session_id == prepared.ticket.session_id
                    && session.profile_id == prepared.ticket.profile_id
                    && session.registry_digest == prepared.ticket.registry_digest
            });
            let Some(record) = state
                .records
                .iter()
                .find(|record| record.ticket.nonce == prepared.ticket.nonce)
            else {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "prepared LSP ticket expired",
                ));
            };
            if record.ticket.ticket_id != prepared.ticket.ticket_id || !session_matches {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "prepared LSP binding changed",
                ));
            }
            if state.active.is_some() || state.queue.front().copied() != Some(prepared.ticket.nonce)
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::OperationBusy,
                    "foreground LSP dispatch is not at queue head",
                ));
            }
            let (stop, canceled_by_request, stop_reason, bytes) = {
                let record = state
                    .records
                    .iter()
                    .find(|record| record.ticket.nonce == prepared.ticket.nonce)
                    .ok_or_else(missing_ticket)?;
                (
                    record.stop.clone(),
                    record.ticket.cancellation == LspForegroundCancellationState::Requested,
                    record.ticket.stop_reason.or(state.stopped),
                    record.request_bytes,
                )
            };
            remove_queued(&mut state, prepared.ticket.nonce);
            state.queued_bytes = state.queued_bytes.saturating_sub(bytes);
            if canceled_by_request || stop.load(std::sync::atomic::Ordering::Acquire) {
                let record =
                    record_mut(&mut state, prepared.ticket.nonce).ok_or_else(missing_ticket)?;
                record.ticket.stop_reason = stop_reason;
                record.ticket.state = if canceled_by_request {
                    LspForegroundTicketState::CancelledBeforeDispatch
                } else if stop_reason == Some(LspForegroundStopReason::ConnectionLost) {
                    LspForegroundTicketState::ConnectionLostAbandoned
                } else {
                    LspForegroundTicketState::ProcessStopped
                };
                if canceled_by_request {
                    record.ticket.cancellation =
                        LspForegroundCancellationState::CancelledBeforeDispatch;
                }
                let result = Err(ServiceError::new(
                    ServiceErrorCode::Cancelled,
                    "foreground LSP read stopped before owner dispatch",
                ));
                record.ticket.result_digest = Some(result_digest(&result)?);
                let output = LspForegroundResult {
                    ticket: ticket_snapshot(&record.ticket)?,
                    result,
                };
                prepared.consumed = true;
                prepared.request.take();
                return Ok(LspReadDispatch::Finished(Box::new(output)));
            }
            let record =
                record_mut(&mut state, prepared.ticket.nonce).ok_or_else(missing_ticket)?;
            record.ticket.overlay = Some(overlay);
            record.ticket.state = LspForegroundTicketState::Dispatched;
            let ticket = ticket_snapshot(&record.ticket)?;
            state.active = Some(prepared.ticket.nonce);
            (stop, ticket)
        };
        prepared.consumed = true;
        let request = prepared.request.take();
        Ok(LspReadDispatch::Ready(Box::new(LspDispatchedRead {
            core: prepared.core.clone(),
            ticket,
            request,
            stop,
            started: false,
            finished: false,
        })))
    }

    fn finish_prepared_error(
        &self,
        prepared: &mut LspPreparedRead,
        error: ServiceError,
    ) -> ServiceResult<LspForegroundResult> {
        let result = Err(error);
        let digest = result_digest(&result)?;
        let mut state = lock(&prepared.core)?;
        if state.active.is_some() || state.queue.front().copied() != Some(prepared.ticket.nonce) {
            return Err(ServiceError::new(
                ServiceErrorCode::OperationBusy,
                "failed foreground LSP read is not at queue head",
            ));
        }
        let record = record_mut(&mut state, prepared.ticket.nonce).ok_or_else(missing_ticket)?;
        if record.ticket.ticket_id != prepared.ticket.ticket_id {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "failed foreground LSP ticket binding changed",
            ));
        }
        record.ticket.state = LspForegroundTicketState::FailedBeforeOwner;
        record.ticket.result_digest = Some(digest);
        let bytes = record.request_bytes;
        let ticket = ticket_snapshot(&record.ticket)?;
        remove_queued(&mut state, prepared.ticket.nonce);
        state.queued_bytes = state.queued_bytes.saturating_sub(bytes);
        prepared.consumed = true;
        prepared.request.take();
        Ok(LspForegroundResult { ticket, result })
    }

    pub fn frontend_finish_read(
        &mut self,
        mut dispatched: LspDispatchedRead,
    ) -> ServiceResult<LspForegroundResult> {
        if !self.uses_foreground(&dispatched.core)
            || lock(&dispatched.core)?.active != Some(dispatched.ticket.nonce)
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "dispatched LSP ticket is not active here",
            ));
        }
        let request = dispatched
            .request
            .take()
            .ok_or_else(|| invalid_frontend_request("dispatched LSP read was consumed"))?;
        let current = match self.foreground_overlay(&request) {
            Ok(binding) => binding,
            Err(error) => return self.record_finished_result(dispatched, Err(error)),
        };
        if dispatched.ticket.overlay.as_ref().is_none_or(|bound| {
            bound.digest != current.digest
                || bound.snapshot_id != current.snapshot_id
                || bound.document_version != current.document_version
                || bound.content_digest != current.content_digest
        }) {
            return self.record_finished_result(
                dispatched,
                Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "LSP overlay changed after dispatch binding",
                )),
            );
        }
        dispatched.started = true;
        let result = match request {
            LspReadRequest::Diagnostics {
                uri,
                previous_result_id,
            } => self
                .frontend_document_diagnostics(
                    &uri,
                    previous_result_id.as_deref(),
                    dispatched.ticket.wire_id.clone(),
                    &dispatched.stop,
                )
                .map(LspReadResult::Diagnostics),
            LspReadRequest::Definition(request) => self
                .frontend_document_definition(request, &dispatched.stop)
                .map(LspReadResult::Definition),
            LspReadRequest::References(request) => self
                .frontend_document_references(request, &dispatched.stop)
                .map(LspReadResult::References),
            LspReadRequest::DocumentSymbols(request) => self
                .frontend_document_symbols(request, &dispatched.stop)
                .map(LspReadResult::DocumentSymbols),
        };
        self.record_finished_result(dispatched, result)
    }

    fn finish_queued_stop(
        &self,
        prepared: &mut LspPreparedRead,
    ) -> ServiceResult<Option<LspForegroundResult>> {
        let mut state = lock(&prepared.core)?;
        if state.active.is_some() || state.queue.front().copied() != Some(prepared.ticket.nonce) {
            return Err(ServiceError::new(
                ServiceErrorCode::OperationBusy,
                "foreground LSP dispatch is not at queue head",
            ));
        }
        let session_matches = state.session.as_ref().is_some_and(|session| {
            session.session_id == prepared.ticket.session_id
                && session.profile_id == prepared.ticket.profile_id
                && session.registry_digest == prepared.ticket.registry_digest
        });
        let Some(index) = state.records.iter().position(|record| {
            record.ticket.nonce == prepared.ticket.nonce
                && record.ticket.ticket_id == prepared.ticket.ticket_id
        }) else {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "prepared LSP ticket expired",
            ));
        };
        if !session_matches {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "prepared LSP binding changed",
            ));
        }
        let record = &state.records[index];
        let requested = matches!(
            record.ticket.cancellation,
            LspForegroundCancellationState::Requested
                | LspForegroundCancellationState::ForwardedToOwner
        );
        let stop_reason = record.ticket.stop_reason.or(state.stopped);
        if !requested && !record.stop.load(std::sync::atomic::Ordering::Acquire) {
            return Ok(None);
        }
        let bytes = record.request_bytes;
        remove_queued(&mut state, prepared.ticket.nonce);
        state.queued_bytes = state.queued_bytes.saturating_sub(bytes);
        let record = &mut state.records[index];
        record.ticket.stop_reason = stop_reason;
        record.ticket.state = if requested {
            LspForegroundTicketState::CancelledBeforeDispatch
        } else if stop_reason == Some(LspForegroundStopReason::ConnectionLost) {
            LspForegroundTicketState::ConnectionLostAbandoned
        } else {
            LspForegroundTicketState::ProcessStopped
        };
        if requested {
            record.ticket.cancellation = LspForegroundCancellationState::CancelledBeforeDispatch;
        }
        let result = Err(ServiceError::new(
            ServiceErrorCode::Cancelled,
            "foreground LSP read stopped before owner dispatch",
        ));
        record.ticket.result_digest = Some(result_digest(&result)?);
        let output = LspForegroundResult {
            ticket: ticket_snapshot(&record.ticket)?,
            result,
        };
        prepared.consumed = true;
        prepared.request.take();
        Ok(Some(output))
    }

    fn record_finished_result(
        &mut self,
        mut dispatched: LspDispatchedRead,
        result: Result<LspReadResult, ServiceError>,
    ) -> ServiceResult<LspForegroundResult> {
        let digest = result_digest(&result)?;
        let mut state = lock(&dispatched.core)?;
        let stopped = state.stopped;
        let record = record_mut(&mut state, dispatched.ticket.nonce).ok_or_else(|| {
            ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "LSP ticket metadata expired during execution",
            )
        })?;
        let requested = matches!(
            record.ticket.cancellation,
            LspForegroundCancellationState::Requested
                | LspForegroundCancellationState::ForwardedToOwner
        );
        let stop_reason = record.ticket.stop_reason.or(stopped);
        record.ticket.stop_reason = stop_reason;
        if requested
            && result
                .as_ref()
                .is_err_and(|error| error.code() == ServiceErrorCode::Cancelled)
        {
            record.ticket.state = LspForegroundTicketState::OwnerAcknowledged;
            record.ticket.cancellation = LspForegroundCancellationState::OwnerAcknowledged;
        } else if requested && result.is_ok() {
            record.ticket.state = LspForegroundTicketState::CompletedBeforeCancel;
            record.ticket.cancellation = LspForegroundCancellationState::CompletedBeforeCancel;
        } else if result
            .as_ref()
            .is_err_and(|error| error.code() == ServiceErrorCode::Cancelled)
            && stop_reason == Some(LspForegroundStopReason::ConnectionLost)
        {
            record.ticket.state = LspForegroundTicketState::ConnectionLostAbandoned;
        } else if result
            .as_ref()
            .is_err_and(|error| error.code() == ServiceErrorCode::Cancelled)
            && stop_reason == Some(LspForegroundStopReason::ProcessStop)
        {
            record.ticket.state = LspForegroundTicketState::ProcessStopped;
        } else if result.is_ok() {
            record.ticket.state = LspForegroundTicketState::Completed;
        } else {
            record.ticket.state = if dispatched.started {
                LspForegroundTicketState::Failed
            } else {
                LspForegroundTicketState::FailedBeforeOwner
            };
        }
        record.ticket.result_digest = Some(digest);
        let ticket = ticket_snapshot(&record.ticket)?;
        state.active = None;
        dispatched.ticket = ticket.clone();
        let output = LspForegroundResult { ticket, result };
        dispatched.finished = true;
        Ok(output)
    }

    fn uses_foreground(&self, core: &Arc<Mutex<LspForegroundState>>) -> bool {
        self.foreground
            .as_ref()
            .is_some_and(|own| Arc::ptr_eq(own, core))
    }

    fn foreground_overlay(
        &self,
        request: &LspReadRequest,
    ) -> ServiceResult<LspForegroundOverlayBinding> {
        if self.registry.profile_id != LSP_OWNED_PROFILE_ID {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "owned LSP profile is not active",
            ));
        }
        let session = self.session.as_ref().ok_or_else(|| {
            ServiceError::new(ServiceErrorCode::InvalidContext, "LSP session is not open")
        })?;
        if session.descriptor.profile_id != LSP_OWNED_PROFILE_ID
            || session.descriptor.registry_digest != self.registry.digest
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "LSP session registry binding changed",
            ));
        }
        let path = session.document_path(request.uri())?;
        let head = session.heads.get(&path).ok_or_else(missing)?;
        if head.resynchronization_required {
            return Err(resynchronization());
        }
        let snapshot = &head.snapshot;
        let digest = canonical_digest(
            "lsp-foreground-overlay-binding:sha256:",
            &(
                &session.descriptor,
                LSP_OWNED_PROFILE_ID,
                &self.registry.digest,
                request,
                &path,
                snapshot,
            ),
        )?;
        Ok(LspForegroundOverlayBinding {
            document_version: snapshot.client_version,
            snapshot_id: snapshot.snapshot_id.clone(),
            content_digest: snapshot.content_digest.to_string().into(),
            digest,
        })
    }
}

fn ticket_snapshot(ticket: &LspForegroundTicket) -> ServiceResult<LspForegroundTicket> {
    let mut snapshot = ticket.clone();
    let mut value = serde_json::to_value(ticket).map_err(|_| {
        ServiceError::new(
            ServiceErrorCode::CanonicalizationFailed,
            "foreground LSP ticket encoding refused",
        )
    })?;
    value
        .as_object_mut()
        .ok_or_else(missing_ticket)?
        .remove("digest");
    snapshot.digest = canonical_digest("lsp-foreground-state:sha256:", &value)?;
    Ok(snapshot)
}

fn result_digest(result: &Result<LspReadResult, ServiceError>) -> ServiceResult<Box<str>> {
    match result {
        Ok(value) => canonical_digest("lsp-foreground-result:sha256:", value),
        Err(error) => canonical_digest(
            "lsp-foreground-result:sha256:",
            &("error", error.code(), error.message(), error.operation_id()),
        ),
    }
}

fn prune_metadata(state: &mut LspForegroundState) {
    while state.records.len() >= LSP_FOREGROUND_MAX_METADATA {
        let Some(index) = state
            .records
            .iter()
            .position(|record| record.ticket.delivery != LspForegroundDeliveryState::Prepared)
        else {
            break;
        };
        state.records.remove(index);
    }
}

fn remove_queued(state: &mut LspForegroundState, nonce: u64) -> bool {
    if let Some(index) = state.queue.iter().position(|item| *item == nonce) {
        state.queue.remove(index);
        true
    } else {
        false
    }
}

fn record_mut(state: &mut LspForegroundState, nonce: u64) -> Option<&mut TicketRecord> {
    state
        .records
        .iter_mut()
        .find(|record| record.ticket.nonce == nonce)
}
fn missing_ticket() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InternalContractViolation,
        "foreground LSP ticket record disappeared",
    )
}

pub(super) fn lock(
    mutex: &Mutex<LspForegroundState>,
) -> ServiceResult<MutexGuard<'_, LspForegroundState>> {
    mutex.lock().map_err(|poisoned| {
        // Stop held requests, but never repair or resume uncertain metadata.
        // Keep the mutex poisoned so every further semantic admission fails.
        let mut state = poisoned.into_inner();
        state.stopped = Some(LspForegroundStopReason::ConnectionLost);
        for record in &mut state.records {
            record.stop.store(true, Ordering::Release);
            if record.ticket.delivery == LspForegroundDeliveryState::Prepared {
                record.ticket.delivery = LspForegroundDeliveryState::LostBefore;
                record.ticket.stop_reason = Some(LspForegroundStopReason::ConnectionLost);
            }
        }
        ServiceError::new(
            ServiceErrorCode::InternalContractViolation,
            "foreground LSP metadata is poisoned; session cannot resume",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn control_for(name: &str) -> LspForegroundControl {
        let mut state = LspForegroundState::new();
        state.session = Some(SessionBinding {
            session_id: name.into(),
            profile_id: LSP_OWNED_PROFILE_ID,
            registry_digest: "test-registry".into(),
            definition_enabled: true,
            references_enabled: true,
            document_symbols_enabled: false,
        });
        LspForegroundControl {
            core: Arc::new(Mutex::new(state)),
            document_symbols_enabled: false,
        }
    }
    fn control() -> LspForegroundControl {
        control_for("test-session")
    }
    fn request() -> LspReadRequest {
        LspReadRequest::Diagnostics {
            uri: "file:///workspace/a.lua".into(),
            previous_result_id: None,
        }
    }
    #[test]
    fn duplicate_outstanding_wire_id_is_refused() -> ServiceResult<()> {
        let control = control();
        let _first = control.prepare(LspRequestId::Number(4), request(), None)?;
        assert!(
            control
                .prepare(LspRequestId::Number(4), request(), None)
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn queued_wire_cancel_sets_intent_and_only_its_stop_flag() -> ServiceResult<()> {
        let control = control();
        let _first = control.prepare(LspRequestId::Number(4), request(), None)?;
        let _second = control.prepare(LspRequestId::Number(5), request(), None)?;
        assert_eq!(
            control.cancel(LspRequestId::Number(4))?.state,
            LspForegroundCancellationState::Requested
        );
        let state = lock(&control.core)?;
        assert!(state.records[0].stop.load(Ordering::Acquire));
        assert!(!state.records[1].stop.load(Ordering::Acquire));
        assert_eq!(
            state.records[0].ticket.state,
            LspForegroundTicketState::Queued
        );
        assert_eq!(
            state.records[0].ticket.cancellation,
            LspForegroundCancellationState::Requested
        );
        assert!(state.records[0].ticket.result_digest.is_none());
        Ok(())
    }
    #[test]
    fn sent_wire_id_reuse_gets_a_new_nonce_and_stale_drop_is_scoped() -> ServiceResult<()> {
        let control = control();
        let first = control.prepare(LspRequestId::Number(4), request(), None)?;
        let old_nonce = first.ticket.nonce;
        {
            let mut state = lock(&control.core)?;
            state.queue.clear();
            state.queued_bytes = 0;
            let record = record_mut(&mut state, old_nonce).ok_or_else(missing_ticket)?;
            record.ticket.state = LspForegroundTicketState::Completed;
            record.ticket.result_digest = Some("digest".into());
        }
        control.record_delivery(first.ticket_id(), LspForegroundDeliveryState::Sent)?;
        let second = control.prepare(LspRequestId::Number(4), request(), None)?;
        let new_nonce = second.ticket.nonce;
        assert_ne!(old_nonce, new_nonce);
        let before = lock(&control.core)?.queued_bytes;
        drop(first);
        let state = lock(&control.core)?;
        let current = state
            .records
            .iter()
            .find(|record| record.ticket.nonce == new_nonce)
            .ok_or_else(missing_ticket)?;
        assert_eq!(current.ticket.state, LspForegroundTicketState::Queued);
        assert_eq!(state.queue.front().copied(), Some(new_nonce));
        assert_eq!(state.queued_bytes, before);
        Ok(())
    }
    #[test]
    fn connection_loss_preserves_partial_delivery_and_marks_other_tickets_lost_before()
    -> ServiceResult<()> {
        let control = control();
        let partial = control.prepare(LspRequestId::Number(7), request(), None)?;
        {
            let mut state = lock(&control.core)?;
            let record = record_mut(&mut state, partial.ticket.nonce).ok_or_else(missing_ticket)?;
            record.ticket.state = LspForegroundTicketState::Completed;
            record.ticket.result_digest = Some("result-digest".into());
        }
        control.record_delivery(
            partial.ticket_id(),
            LspForegroundDeliveryState::LostAfterPossible,
        )?;
        let partial_id = partial.ticket_id().to_owned();
        let pending = control.prepare(LspRequestId::Number(8), request(), None)?;
        let pending_id = pending.ticket_id().to_owned();
        control.stop(LspForegroundStopReason::ConnectionLost)?;
        drop(partial);
        let partial_meta = control
            .ticket_metadata(&partial_id)
            .ok_or_else(missing_ticket)?;
        assert_eq!(
            partial_meta.delivery,
            LspForegroundDeliveryState::LostAfterPossible
        );
        assert_eq!(
            partial_meta.cancellation,
            LspForegroundCancellationState::NotRequested
        );
        let pending_meta = control
            .ticket_metadata(&pending_id)
            .ok_or_else(missing_ticket)?;
        assert_eq!(
            pending_meta.delivery,
            LspForegroundDeliveryState::LostBefore
        );
        assert_eq!(
            pending_meta.cancellation,
            LspForegroundCancellationState::NotRequested
        );
        Ok(())
    }
    #[test]
    fn timeout_intent_is_separate_and_session_ticket_ids_do_not_collide() -> ServiceResult<()> {
        let first = control_for("first");
        let second = control_for("second");
        let a = first.prepare(LspRequestId::Number(1), request(), None)?;
        let b = second.prepare(LspRequestId::Number(1), request(), None)?;
        assert_ne!(a.ticket_id(), b.ticket_id());
        assert_eq!(
            first.request_timeout(LspRequestId::Number(1))?,
            LspForegroundCancellationState::Requested
        );
        let _ = first.cancel(LspRequestId::Number(1))?;
        let metadata = first
            .ticket_metadata(a.ticket_id())
            .ok_or_else(missing_ticket)?;
        assert_eq!(
            metadata.cancel_origin,
            Some(LspForegroundCancelOrigin::RequestDeadline)
        );
        assert_eq!(
            metadata.cancellation,
            LspForegroundCancellationState::Requested
        );
        Ok(())
    }
    #[test]
    fn poisoned_metadata_stops_owned_flags_and_never_resumes_admission() -> ServiceResult<()> {
        let control = control();
        let pending = control.prepare(LspRequestId::Number(1), request(), None)?;
        let flag = lock(&control.core)?.records[0].stop.clone();
        let core = Arc::clone(&control.core);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if let Ok(_guard) = core.lock() {
                std::panic::resume_unwind(Box::new("metadata invariant failure"));
            }
        }));
        assert!(panic.is_err());
        let error = control
            .prepare(LspRequestId::Number(2), request(), None)
            .err()
            .ok_or_else(missing_ticket)?;
        assert_eq!(error.code(), ServiceErrorCode::InternalContractViolation);
        assert!(flag.load(Ordering::Acquire));
        assert!(control.ticket_metadata(pending.ticket_id()).is_none());
        Ok(())
    }
}
