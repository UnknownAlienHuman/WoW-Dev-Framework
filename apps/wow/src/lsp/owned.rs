//! One App I/O controller and one scoped, joined semantic writer.
use super::*;
use super::{
    framing::FrameDecoder,
    io::{Incoming, OwnedStdio, READ_CHUNK_BYTES, WRITE_CHUNK_BYTES},
};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{
        Arc,
        mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError},
    },
    time::{Duration, Instant},
};
use wow_service::frontend::lsp::{
    LSP_FOREGROUND_EXECUTION, LspForegroundControl, LspForegroundDeliveryState,
    LspForegroundResult, LspForegroundStopReason, LspForegroundTicket, LspPreparedRead,
    LspProgressToken, LspReadRequest, LspReadResult,
};
#[cfg(test)]
mod conformance;
mod worker;
use worker::{Command, Event};

struct Queued {
    command: Command,
    bytes: usize,
}
struct Outgoing {
    bytes: Vec<u8>,
    position: usize,
    chunk: usize,
    ticket: Option<Box<str>>,
    prepared: Instant,
    progressed: Instant,
    is_progress: bool,
}
struct Pending {
    wire_id: LspRequestId,
    prepared: Instant,
    progressed: Instant,
    timed_out: bool,
}
struct Driver {
    control: Arc<LspForegroundControl>,
    input: FrameDecoder,
    commands: VecDeque<Queued>,
    command_bytes: usize,
    output: VecDeque<Outgoing>,
    output_bytes: usize,
    progress: VecDeque<Outgoing>,
    progress_bytes: usize,
    omitted_progress: BTreeSet<Box<str>>,
    current: Option<Outgoing>,
    pending: BTreeMap<Box<str>, Pending>,
    partial_since: Option<Instant>,
    last_input: Instant,
    shutdown_fence: bool,
    shutdown_seen: bool,
    stopping: Option<u8>,
    output_lost: bool,
    worker_closing: bool,
}

pub(super) fn run(
    service: LspFrontend,
    control: LspForegroundControl,
    stop: Arc<AtomicBool>,
) -> u8 {
    let mut io = match OwnedStdio::acquire() {
        Ok(io) => io,
        Err(_) => {
            super::super::diagnostic("wow: owned LSP stdio acquisition refused");
            return 64;
        }
    };
    let control = Arc::new(control);
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::scope(|scope| {
            // Creation order matters on unwind: Driver drops queued read tokens
            // before channel senders disappear and the scope joins its worker.
            let (send_command, receive_command) = mpsc::sync_channel(0);
            let (send_event, receive_event) = mpsc::sync_channel(1);
            let worker_control = Arc::clone(&control);
            let worker_stop = Arc::clone(&stop);
            let worker = scope.spawn(move || {
                worker::run(
                    Server {
                        service,
                        initialized: false,
                        ready: false,
                        shutdown: false,
                    },
                    worker_control,
                    receive_command,
                    send_event,
                    worker_stop,
                )
            });
            let mut sender = Some(send_command);
            let mut driver = match Driver::new(Arc::clone(&control)) {
                Ok(driver) => driver,
                Err(()) => {
                    let _ = control.stop(LspForegroundStopReason::ConnectionLost);
                    sender.take();
                    drop(receive_event);
                    let _ = worker.join();
                    return 4;
                }
            };
            let code = driver.serve(&mut io, &control, &stop, &mut sender, &receive_event);
            // Retire every held token before allowing the worker's checked close.
            driver.commands.clear();
            driver.command_bytes = 0;
            if !driver.pending.is_empty() {
                driver.lose_output(&control, code);
            }
            sender.take();
            // A blocked final-event send must observe receiver loss before join.
            drop(receive_event);
            match worker.join() {
                Ok(exit) if !exit.session_closed || exit.code != 0 => {
                    if exit.code == 0 {
                        4
                    } else {
                        exit.code
                    }
                }
                Ok(_) => code,
                Err(_) => 4,
            }
        })
    }));
    let restored = io.close();
    std::panic::set_hook(hook);
    if restored.is_err() {
        super::super::diagnostic("wow: owned LSP required stdio restoration failed");
        return 4;
    }
    match result {
        Ok(code) => code,
        Err(_) => {
            let _ = control.stop(LspForegroundStopReason::ConnectionLost);
            super::super::diagnostic("wow: owned LSP internal failure; session terminated");
            4
        }
    }
}
impl Driver {
    fn new(control: Arc<LspForegroundControl>) -> Result<Self, ()> {
        let now = Instant::now();
        Ok(Self {
            control,
            input: FrameDecoder::new(Limits {
                header_bytes: LSP_MAX_HEADER_BYTES,
                body_bytes: LSP_MAX_FRAME_BYTES,
            })
            .map_err(|_| ())?,
            commands: VecDeque::new(),
            command_bytes: 0,
            output: VecDeque::new(),
            output_bytes: 0,
            progress: VecDeque::new(),
            progress_bytes: 0,
            omitted_progress: BTreeSet::new(),
            current: None,
            pending: BTreeMap::new(),
            partial_since: None,
            last_input: now,
            shutdown_fence: false,
            shutdown_seen: false,
            stopping: None,
            output_lost: false,
            worker_closing: false,
        })
    }
    fn serve(
        &mut self,
        io: &mut OwnedStdio,
        control: &LspForegroundControl,
        stop: &AtomicBool,
        sender: &mut Option<SyncSender<Command>>,
        events: &Receiver<Event>,
    ) -> u8 {
        let mut buffer = [0; READ_CHUNK_BYTES];
        let mut worker_done = false;
        loop {
            let mut progressed = false;
            loop {
                match events.try_recv() {
                    Ok(event) => {
                        progressed = true;
                        if self.event(event, control).is_err() {
                            self.lose_output(control, 4);
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        worker_done = true;
                        break;
                    }
                }
            }
            if stop.load(Ordering::Acquire) && self.stopping.is_none() {
                let _ = control.stop(LspForegroundStopReason::ProcessStop);
                self.stopping = Some(130);
            }
            self.expire(control);
            if self.stopping.is_none() {
                match io.read(&mut buffer) {
                    Ok(Incoming::Pending) => {}
                    Ok(Incoming::Eof) => {
                        if self.input.finish().is_err() || !self.shutdown_seen {
                            self.lose_output(control, 1);
                        } else {
                            self.stopping = Some(0);
                        }
                    }
                    Ok(Incoming::Bytes(count)) => {
                        progressed = true;
                        self.last_input = Instant::now();
                        if self.input.push(&buffer[..count]).is_err() {
                            self.lose_output(control, 1);
                        } else {
                            loop {
                                match self.input.next_frame() {
                                    Ok(Some(frame)) => {
                                        self.partial_since = None;
                                        if self.message(&frame, control).is_err() {
                                            self.lose_output(control, 1);
                                            break;
                                        }
                                    }
                                    Ok(None) => break,
                                    Err(_) => {
                                        self.lose_output(control, 1);
                                        break;
                                    }
                                }
                            }
                            if self.input.finish().is_ok() {
                                self.partial_since = None;
                            } else if self.partial_since.is_none() {
                                self.partial_since = Some(Instant::now());
                            }
                        }
                    }
                    Err(_) => self.lose_output(control, 4),
                }
            }
            if let Some(tx) = sender.as_ref() {
                let wait_shutdown = self.commands.front().is_some_and(|queued|
                    matches!(&queued.command, Command::Protocol(message) if message.method == "shutdown")
                    && !self.pending.is_empty());
                if !wait_shutdown && let Some(queued) = self.commands.pop_front() {
                    match tx.try_send(queued.command) {
                        Ok(()) => {
                            self.command_bytes -= queued.bytes;
                            progressed = true;
                        }
                        Err(TrySendError::Full(command)) => self.commands.push_front(Queued {
                            command,
                            bytes: queued.bytes,
                        }),
                        Err(TrySendError::Disconnected(command)) => {
                            drop(command);
                            self.command_bytes -= queued.bytes;
                            self.lose_output(control, 4);
                        }
                    }
                }
            }
            if !self.output_lost {
                match self.write(io, control) {
                    Ok(written) => progressed |= written,
                    Err(()) => self.lose_output(control, 4),
                }
            }
            let drained = self.commands.is_empty()
                && self.pending.is_empty()
                && self.current.is_none()
                && self.output.is_empty()
                && self.progress.is_empty();
            if self.worker_closing && !self.commands.is_empty() {
                if !self.pending.is_empty() {
                    self.lose_output(control, 4);
                }
                self.commands.clear();
                self.command_bytes = 0;
            }
            if self.output_lost || (self.stopping.is_some() && drained) {
                // The peer is closed or all admitted work and final frames drained.
                self.commands.clear();
                self.command_bytes = 0;
                sender.take();
            }
            if worker_done && (drained || self.output_lost) {
                return self.stopping.unwrap_or(4);
            }
            if worker_done && !self.pending.is_empty() {
                self.lose_output(control, 4);
            }
            if !progressed {
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }
    fn message(&mut self, frame: &[u8], control: &LspForegroundControl) -> Result<(), ()> {
        let message = match parse(frame) {
            Ok(message) => message,
            Err((code, message)) => {
                return self.enqueue(
                    encode(json!({"jsonrpc":"2.0","id":null,
                "error":{"code":code,"message":message}}))?,
                    None,
                    false,
                );
            }
        };
        if message.method == "$/cancelRequest" && message.id.is_none() {
            if only(&message.params, &["id"]).is_err() {
                return self.notice("invalid cancellation parameters");
            }
            let target = message
                .params
                .get("id")
                .and_then(|id| serde_json::from_value(id.clone()).ok());
            return match target {
                Some(id) => control.cancel(id).map(|_| ()).map_err(|_| ()),
                None => self.notice("invalid cancellation target"),
            };
        }
        let request = match message.method.as_str() {
            "textDocument/documentSymbol" if control.document_symbols_enabled() => Some(
                document_symbols::parse_request(&message.params)
                    .map(LspReadRequest::DocumentSymbols),
            ),
            "textDocument/diagnostic" => Some(diagnostic_request(&message.params).map(
                |(uri, previous_result_id)| LspReadRequest::Diagnostics {
                    uri,
                    previous_result_id,
                },
            )),
            "textDocument/definition" => {
                Some(definition::parse_request(&message.params).map(LspReadRequest::Definition))
            }
            "textDocument/references" => {
                Some(references::parse_request(&message.params).map(LspReadRequest::References))
            }
            _ => None,
        };
        if let Some(request) = request {
            let Some(id) = message.id else {
                return self.notice("read operation requires a request ID");
            };
            if self.shutdown_fence {
                return self.error(id, -32600, "shutdown is queued");
            }
            let request = match request {
                Ok(request) => request,
                Err(()) => return self.error(id, -32602, "invalid read parameters"),
            };
            let progress = match progress_token(&message.params) {
                Ok(token) => token,
                Err(()) => return self.error(id, -32602, "invalid progress token"),
            };
            self.admit(frame.len())?;
            let prepared = match control.prepare(id.clone(), request, progress) {
                Ok(prepared) => prepared,
                Err(error) if error.code() == ServiceErrorCode::OperationConflict => {
                    // A duplicate cannot receive an ambiguous second response ID.
                    return Err(());
                }
                Err(error) => return self.service_error(id, error),
            };
            let key: Box<str> = prepared.ticket_id().into();
            let now = Instant::now();
            self.pending.insert(
                key,
                Pending {
                    wire_id: id,
                    prepared: now,
                    progressed: now,
                    timed_out: false,
                },
            );
            self.command_bytes += frame.len();
            self.commands.push_back(Queued {
                command: Command::Read(Box::new(prepared)),
                bytes: frame.len(),
            });
        } else {
            self.admit(frame.len())?;
            if message.method == "shutdown" && message.id.is_some() && message.params.is_empty() {
                self.shutdown_fence = true;
            }
            self.command_bytes += frame.len();
            self.commands.push_back(Queued {
                command: Command::Protocol(message),
                bytes: frame.len(),
            });
        }
        Ok(())
    }
    fn admit(&self, bytes: usize) -> Result<(), ()> {
        if self.commands.len() >= LSP_FOREGROUND_EXECUTION.max_queued_messages
            || self
                .command_bytes
                .checked_add(bytes)
                .is_none_or(|size| size > LSP_FOREGROUND_EXECUTION.max_queued_message_bytes)
        {
            Err(())
        } else {
            Ok(())
        }
    }
    fn event(&mut self, event: Event, control: &LspForegroundControl) -> Result<(), ()> {
        match event {
            Event::ReadResult(mut result) => {
                if self.output_lost {
                    return Ok(());
                }
                result.ticket = control
                    .ticket_metadata(&result.ticket.ticket_id)
                    .ok_or(())?;
                let key = result.ticket.ticket_id.clone();
                self.enqueue(worker::response(*result)?, Some(key), false)
            }
            Event::Frame {
                bytes,
                ticket,
                shutdown,
                shutdown_rejected,
            } => {
                self.shutdown_seen |= shutdown;
                if shutdown_rejected {
                    self.shutdown_fence = false;
                }
                if self.output_lost {
                    return Ok(());
                }
                self.enqueue(bytes, ticket, false)
            }
            Event::Progress { ticket, begin } => {
                if let Some(pending) = self.pending.get_mut(&ticket.ticket_id) {
                    pending.progressed = Instant::now();
                }
                if self.output_lost {
                    return Ok(());
                }
                let Some(token) = &ticket.progress_token else {
                    return Ok(());
                };
                let bytes = encode(json!({"jsonrpc":"2.0","method":"$/progress","params":{
                    "token":token,"value":if begin {json!({"kind":"begin","title":"Local request","cancellable":false,
                        "wow":{"ticket_id":ticket.ticket_id,"sequence":1,"non_authoritative":true}})}
                    else {json!({"kind":"end","wow":{"ticket_id":ticket.ticket_id,"sequence":2,"non_authoritative":true}})}
                }}))?;
                if !begin && self.omitted_progress.remove(&ticket.ticket_id) {
                    control
                        .record_progress_coalesced(&ticket.ticket_id)
                        .map_err(|_| ())?;
                    return Ok(());
                }
                // Reserve both begin and end together. Cosmetic progress never
                // consumes final-result capacity or changes an owner outcome.
                let active_progress =
                    usize::from(self.current.as_ref().is_some_and(|frame| frame.is_progress));
                if begin
                    && (self.progress.len() + active_progress + 2
                        > LSP_FOREGROUND_EXECUTION.max_progress_events
                        || self
                            .progress_bytes
                            .checked_add(bytes.len().saturating_mul(2))
                            .is_none_or(|n| n > LSP_FOREGROUND_EXECUTION.max_progress_bytes))
                {
                    self.omitted_progress.insert(ticket.ticket_id.clone());
                    control
                        .record_progress_coalesced(&ticket.ticket_id)
                        .map_err(|_| ())?;
                    return Ok(());
                }
                self.enqueue(bytes, None, true)
            }
            Event::Closing { code, shutdown } => {
                self.worker_closing = true;
                self.shutdown_seen |= shutdown;
                if code != 0 {
                    self.lose_output(control, code);
                } else if self.stopping.is_none() {
                    self.stopping = Some(0);
                }
                Ok(())
            }
        }
    }
    fn enqueue(
        &mut self,
        bytes: Vec<u8>,
        ticket: Option<Box<str>>,
        progress: bool,
    ) -> Result<(), ()> {
        if bytes.is_empty() {
            return Ok(());
        }
        let (charged, maximum) = if progress {
            (
                self.progress_bytes,
                LSP_FOREGROUND_EXECUTION.max_progress_bytes,
            )
        } else {
            (
                self.output_bytes,
                LSP_FOREGROUND_EXECUTION.max_outbound_bytes,
            )
        };
        if charged.checked_add(bytes.len()).is_none_or(|n| n > maximum) {
            return Err(());
        }
        if progress {
            self.progress_bytes += bytes.len();
        } else {
            self.output_bytes += bytes.len();
        }
        let now = Instant::now();
        let frame = Outgoing {
            bytes,
            position: 0,
            chunk: WRITE_CHUNK_BYTES,
            ticket,
            prepared: now,
            progressed: now,
            is_progress: progress,
        };
        if progress {
            self.progress.push_back(frame);
        } else {
            self.output.push_back(frame);
        }
        Ok(())
    }
    fn write(&mut self, io: &mut OwnedStdio, control: &LspForegroundControl) -> Result<bool, ()> {
        if self.current.is_none() {
            self.current = self
                .output
                .pop_front()
                .or_else(|| self.progress.pop_front());
        }
        let Some(frame) = self.current.as_mut() else {
            return Ok(false);
        };
        if elapsed(frame.prepared, LSP_FOREGROUND_EXECUTION.output_absolute_ms)
            || elapsed(frame.progressed, LSP_FOREGROUND_EXECUTION.output_idle_ms)
        {
            return Err(());
        }
        let end = frame.bytes.len().min(frame.position + frame.chunk);
        let written = io
            .write(&frame.bytes[frame.position..end])
            .map_err(|_| ())?;
        if written == 0 {
            frame.chunk = (frame.chunk / 2).max(1);
            return Ok(false);
        }
        frame.position += written;
        frame.progressed = Instant::now();
        frame.chunk = WRITE_CHUNK_BYTES;
        if frame.position == frame.bytes.len() {
            let frame = self.current.take().ok_or(())?;
            if frame.is_progress {
                self.progress_bytes -= frame.bytes.len();
            } else {
                self.output_bytes -= frame.bytes.len();
            }
            if let Some(key) = frame.ticket {
                control
                    .record_delivery(&key, LspForegroundDeliveryState::Sent)
                    .map_err(|_| ())?;
                self.pending.remove(&key);
            }
        }
        Ok(true)
    }
    fn expire(&mut self, control: &LspForegroundControl) {
        if self.stopping.is_some() {
            return;
        }
        if let Some(started) = self.partial_since
            && (elapsed(started, LSP_FOREGROUND_EXECUTION.partial_frame_absolute_ms)
                || elapsed(
                    self.last_input,
                    LSP_FOREGROUND_EXECUTION.partial_frame_idle_ms,
                ))
        {
            self.lose_output(control, 1);
            return;
        }
        if self.stopping.is_none() {
            for pending in self.pending.values_mut() {
                if !pending.timed_out
                    && (elapsed(
                        pending.prepared,
                        LSP_FOREGROUND_EXECUTION.request_absolute_ms,
                    ) || elapsed(pending.progressed, LSP_FOREGROUND_EXECUTION.request_idle_ms))
                {
                    let _ = control.request_timeout(pending.wire_id.clone());
                    pending.timed_out = true;
                }
            }
        }
    }
    fn lose_output(&mut self, control: &LspForegroundControl, code: u8) {
        if !self.output_lost {
            if let Some(frame) = &self.current
                && frame.position > 0
                && let Some(key) = &frame.ticket
            {
                let _ = control.record_delivery(key, LspForegroundDeliveryState::LostAfterPossible);
            }
            let _ = control.stop(LspForegroundStopReason::ConnectionLost);
            self.commands.clear();
            self.command_bytes = 0;
            self.output.clear();
            self.progress.clear();
            self.current = None;
            self.output_bytes = 0;
            self.progress_bytes = 0;
            self.omitted_progress.clear();
            self.pending.clear();
            self.output_lost = true;
        }
        self.stopping = Some(match (self.stopping, code) {
            (Some(4), _) | (_, 4) => 4,
            (_, 0) => self.stopping.unwrap_or(4),
            (_, code) => code,
        });
    }
    fn notice(&mut self, message: &'static str) -> Result<(), ()> {
        let mut frame = Vec::new();
        notification_error(&mut frame, message)?;
        self.enqueue(frame, None, false)
    }
    fn error(&mut self, id: LspRequestId, code: i32, message: &'static str) -> Result<(), ()> {
        let mut frame = Vec::new();
        response_error(&mut frame, Some(id), code, message)?;
        self.enqueue(frame, None, false)
    }
    fn service_error(&mut self, id: LspRequestId, error: ServiceError) -> Result<(), ()> {
        let mut frame = Vec::new();
        domain_error(&mut frame, Some(id), error)?;
        self.enqueue(frame, None, false)
    }
}
impl Drop for Driver {
    fn drop(&mut self) {
        if !self.pending.is_empty()
            || !self.commands.is_empty()
            || self.current.is_some()
            || !self.output.is_empty()
            || !self.progress.is_empty()
        {
            // This guard runs before senders/receivers disappear on parent
            // unwind. It records the actual current-frame offset first.
            let control = Arc::clone(&self.control);
            self.lose_output(&control, 4);
        }
    }
}
fn elapsed(since: Instant, milliseconds: u64) -> bool {
    since.elapsed() >= Duration::from_millis(milliseconds)
}
fn encode(value: Value) -> Result<Vec<u8>, ()> {
    let mut bytes = Vec::new();
    send(&mut bytes, value)?;
    Ok(bytes)
}
fn progress_token(params: &Map<String, Value>) -> Result<Option<LspProgressToken>, ()> {
    for name in ["workDoneToken", "partialResultToken"] {
        if let Some(value) = params.get(name) {
            let token: LspProgressToken = serde_json::from_value(value.clone()).map_err(|_| ())?;
            if matches!(token,LspProgressToken::String(ref text) if text.len()>128 || text.chars().any(char::is_control))
            {
                return Err(());
            }
        }
    }
    params
        .get("workDoneToken")
        .map(|value| serde_json::from_value(value.clone()).map_err(|_| ()))
        .transpose()
}
