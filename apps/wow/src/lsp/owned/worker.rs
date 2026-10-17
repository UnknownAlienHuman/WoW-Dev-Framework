//! The single semantic writer. Pipe handles never enter this module.
use super::*;
use wow_service::frontend::lsp::{LspReadDispatch, LspReadRefusal};

pub(super) enum Command {
    Protocol(Message),
    Read(Box<LspPreparedRead>),
    #[cfg(test)]
    PanicForConformance,
}
pub(super) struct WorkerExit {
    pub code: u8,
    pub session_closed: bool,
}
pub(super) enum Event {
    ReadResult(Box<LspForegroundResult>),
    Frame {
        bytes: Vec<u8>,
        ticket: Option<Box<str>>,
        shutdown: bool,
        shutdown_rejected: bool,
    },
    Progress {
        ticket: Box<LspForegroundTicket>,
        begin: bool,
    },
    Closing {
        code: u8,
        shutdown: bool,
    },
}

pub(super) fn run(
    mut server: Server,
    control: Arc<LspForegroundControl>,
    commands: Receiver<Command>,
    events: SyncSender<Event>,
    stop: Arc<AtomicBool>,
) -> WorkerExit {
    let body = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serve(&mut server, &commands, &events, &stop)
    }));
    let mut code = body.unwrap_or(4);
    // Hand the controller a closing event before waiting for its held tokens to
    // retire. On parent unwind, channel drops provide the same rendezvous.
    let notified = events.send(Event::Closing {
        code,
        shutdown: server.shutdown,
    });
    if notified.is_err() {
        // The controller has unwound. Its Driver drop guard has already
        // classified any partly written final frame before channel teardown.
        let _ = control.stop(LspForegroundStopReason::ConnectionLost);
    }
    while let Ok(command) = commands.recv() {
        drop(command);
    }
    // A parent unwind/channel loss can end the loop without an explicit exit.
    // Retire unsent read receipts before the session's mandatory close gate.
    if !server.shutdown {
        let _ = control.stop(LspForegroundStopReason::ConnectionLost);
    }
    let session_closed = server.service.frontend_session_close().is_ok();
    if !session_closed {
        code = 4;
    }
    WorkerExit {
        code,
        session_closed,
    }
}
fn serve(
    server: &mut Server,
    commands: &Receiver<Command>,
    events: &SyncSender<Event>,
    stop: &AtomicBool,
) -> u8 {
    while let Ok(command) = commands.recv() {
        match command {
            #[cfg(test)]
            Command::PanicForConformance => {
                std::panic::resume_unwind(Box::new("owned worker conformance panic"))
            }
            Command::Protocol(message) => {
                let shutdown_request = message.method == "shutdown";
                let mut bytes = Vec::new();
                let control = match server.handle(message, &mut bytes, stop) {
                    Ok(control) => control,
                    Err(()) => return 4,
                };
                if events
                    .send(Event::Frame {
                        bytes,
                        ticket: None,
                        shutdown: server.shutdown,
                        shutdown_rejected: shutdown_request && !server.shutdown,
                    })
                    .is_err()
                {
                    return 4;
                }
                if let Control::Exit(code) = control {
                    return code;
                }
            }
            Command::Read(prepared) => {
                let mut progress = None;
                let result = if !server.ready || server.shutdown {
                    server.service.frontend_refuse_read(
                        *prepared,
                        if server.shutdown {
                            LspReadRefusal::SessionClosed
                        } else {
                            LspReadRefusal::NotReady
                        },
                    )
                } else {
                    match server.service.frontend_dispatch_read(*prepared) {
                        Ok(LspReadDispatch::Finished(result)) => Ok(*result),
                        Ok(LspReadDispatch::Ready(dispatched)) => {
                            if dispatched.ticket().progress_token.is_some() {
                                let ticket = dispatched.ticket().clone();
                                if events
                                    .send(Event::Progress {
                                        ticket: Box::new(ticket.clone()),
                                        begin: true,
                                    })
                                    .is_err()
                                {
                                    return 4;
                                }
                                progress = Some(ticket);
                            }
                            server.service.frontend_finish_read(*dispatched)
                        }
                        Err(error) => Err(error),
                    }
                };
                let result = match result {
                    Ok(result) => result,
                    Err(_) => return 4,
                };
                if events.send(Event::ReadResult(Box::new(result))).is_err() {
                    return 4;
                }
                if let Some(ticket) = progress
                    && events
                        .send(Event::Progress {
                            ticket: Box::new(ticket),
                            begin: false,
                        })
                        .is_err()
                {
                    return 4;
                }
            }
        }
    }
    0
}
pub(super) fn response(result: LspForegroundResult) -> Result<Vec<u8>, ()> {
    let LspForegroundResult { ticket, result } = result;
    let id = &ticket.wire_id;
    let mut value = match result {
        Ok(LspReadResult::Diagnostics(report)) => json!({"jsonrpc":"2.0","id":id,"result":report}),
        Ok(LspReadResult::Definition(report)) => definition::result_response(id, &report),
        Ok(LspReadResult::References(report)) => references::result_response(id, &report)?,
        Ok(LspReadResult::DocumentSymbols(report)) => {
            document_symbols::result_response(id, &report)?
        }
        Err(error) => domain_error_value(id, &error),
    };
    if value.get("error").is_some() {
        let data = value["error"]["data"].take();
        value["error"]["data"] = json!({"ticket":ticket,"owner_result":data});
    } else if let Some(locations) = value.get_mut("result").and_then(Value::as_array_mut) {
        if let Some(first) = locations.first_mut() {
            first["wow_ticket"] = serde_json::to_value(&ticket).map_err(|_| ())?;
        } else {
            value["wow_ticket"] = serde_json::to_value(&ticket).map_err(|_| ())?;
        }
    } else {
        value["result"]["wow_ticket"] = serde_json::to_value(&ticket).map_err(|_| ())?;
    }
    encode(value)
}
