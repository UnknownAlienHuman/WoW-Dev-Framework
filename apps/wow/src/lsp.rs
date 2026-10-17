//! LSP framing/lifecycle projection. All document state is Service owned.
use super::json::StrictJson;
use serde_json::{Map, Value, json};
use std::{
    ffi::OsString,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceError, ServiceErrorCode,
    frontend::lsp::{
        LSP_MAX_FRAME_BYTES, LSP_MAX_HEADER_BYTES, LSP_MAX_OUTPUT_BYTES, LSP_OWNED_PROFILE_ID,
        LSP_PROFILE_ID, LspContentChange, LspDocumentChange, LspDocumentChangeRequest,
        LspDocumentOpenRequest, LspDocumentSaveRequest, LspFrontend, LspHost, LspRequestId,
        LspSessionOpenRequest, LspSynchronizationRefusal, LspWorkspaceFoldersChangeRequest,
    },
};
mod definition;
mod document_symbols;
mod framing;
mod io;
mod owned;
mod references;
use framing::{Limits, read_frame, write_frame};

const HELP: &str = "wow lsp --transport stdio --profile <lsp-3.18-stdio-v1|lsp-3.18-stdio-owned-v1> --host <explicit.json>\nPrivate Lua diagnostics and explicitly selected local navigation over one registered workspace. Owned mode adds foreground tickets, cooperative request cancellation and bounded delivery. Published dependencies, Reference, Graph, Rules and runtime are not evaluated. See apps/wow/LSP.md.\n";
pub(super) fn run(args: Vec<OsString>) -> u8 {
    if args.len() == 2 && args[1] == "--help" {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let (host, owned_execution) = match arguments(&args) {
        Ok(path) => path,
        Err(()) => return super::usage("invalid LSP command; use wow lsp --help"),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let host = match LspHost::load(&host, &stop) {
        Ok(host) => host,
        Err(_) => {
            super::diagnostic("wow: LSP host/profile acquisition refused");
            return 64;
        }
    };
    if owned_execution {
        return match LspFrontend::new_foreground(host) {
            Ok((service, control)) => owned::run(service, control, stop),
            Err(_) => {
                super::diagnostic("wow: owned LSP profile refused");
                64
            }
        };
    }
    let host = match LspFrontend::new(host) {
        Ok(host) => host,
        Err(_) => {
            super::diagnostic("wow: LSP host/profile acquisition refused");
            return 64;
        }
    };
    // An upstream panic must not expose private source via its panic payload.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut server = Server {
        service: host,
        initialized: false,
        ready: false,
        shutdown: false,
    };
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        serve(
            &mut server,
            BufReader::new(std::io::stdin().lock()),
            &mut std::io::stdout().lock(),
            &stop,
        )
    }));
    let closed = server.service.frontend_session_close();
    std::panic::set_hook(hook);
    if closed.is_err() {
        super::diagnostic("wow: LSP mandatory session close failed");
        return 4;
    }
    match result {
        Ok(code) => code,
        Err(_) => {
            super::diagnostic("wow: LSP internal failure; session terminated");
            4
        }
    }
}
fn arguments(args: &[OsString]) -> Result<(PathBuf, bool), ()> {
    if args.len() != 7
        || args.iter().map(|v| v.len()).sum::<usize>() > 65536
        || args[0] != "lsp"
        || args[1] != "--transport"
        || args[2] != "stdio"
        || args[3] != "--profile"
        || (args[4] != LSP_PROFILE_ID && args[4] != LSP_OWNED_PROFILE_ID)
        || args[5] != "--host"
        || args[6].is_empty()
    {
        return Err(());
    }
    Ok((PathBuf::from(&args[6]), args[4] == LSP_OWNED_PROFILE_ID))
}
struct Server {
    service: LspFrontend,
    initialized: bool,
    ready: bool,
    shutdown: bool,
}
enum Control {
    Continue,
    Exit(u8),
}
struct Message {
    id: Option<LspRequestId>,
    method: String,
    params: Map<String, Value>,
}
fn serve<R: BufRead, W: Write>(
    server: &mut Server,
    mut input: R,
    output: &mut W,
    stop: &AtomicBool,
) -> u8 {
    let mut frame = Vec::new();
    loop {
        if stop.load(Ordering::Acquire) {
            return 130;
        }
        match read_frame(
            &mut input,
            &mut frame,
            Limits {
                header_bytes: LSP_MAX_HEADER_BYTES,
                body_bytes: LSP_MAX_FRAME_BYTES,
            },
        ) {
            Ok(false) => return if server.shutdown { 0 } else { 1 },
            Err(_) => {
                super::diagnostic("wow: LSP framing refused");
                return 1;
            }
            Ok(true) => {}
        }
        match parse(&frame) {
            Ok(message) => match server.handle(message, output, stop) {
                Ok(Control::Continue) => {}
                Ok(Control::Exit(code)) => return code,
                Err(()) => {
                    super::diagnostic("wow: LSP output failed");
                    return 4;
                }
            },
            Err((code, message)) => {
                if send(
                    output,
                    json!({"jsonrpc":"2.0","id":null,"error":{"code":code,"message":message}}),
                )
                .is_err()
                {
                    return 4;
                }
            }
        }
    }
}
fn parse(frame: &[u8]) -> Result<Message, (i32, &'static str)> {
    let StrictJson(value) = serde_json::from_slice(frame).map_err(|_| (-32700, "invalid JSON"))?;
    let object = value
        .as_object()
        .ok_or((-32600, "request must be an object"))?;
    only(object, &["jsonrpc", "id", "method", "params"])
        .map_err(|_| (-32600, "unsupported request field"))?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err((-32600, "JSON-RPC 2.0 required"));
    }
    let method = string(object, "method", 128).map_err(|_| (-32600, "invalid method"))?;
    if method.is_empty() || method.chars().any(char::is_control) {
        return Err((-32600, "invalid method"));
    }
    let id = match object.get("id") {
        None => None,
        Some(value) => {
            let id: LspRequestId = serde_json::from_value(value.clone())
                .map_err(|_| (-32600, "invalid request ID"))?;
            if matches!(&id,LspRequestId::Text(v) if v.len()>128 || v.chars().any(char::is_control))
            {
                return Err((-32600, "request ID exceeds profile"));
            }
            Some(id)
        }
    };
    let params = match object.get("params") {
        None | Some(Value::Null) => Map::new(),
        Some(Value::Object(value)) => value.clone(),
        _ => return Err((-32602, "object parameters required")),
    };
    Ok(Message { id, method, params })
}
impl Server {
    fn handle<W: Write>(
        &mut self,
        message: Message,
        output: &mut W,
        stop: &AtomicBool,
    ) -> Result<Control, ()> {
        let Message { id, method, params } = message;
        if method == "exit" && id.is_none() {
            return Ok(Control::Exit(if self.shutdown && params.is_empty() {
                0
            } else {
                1
            }));
        }
        if self.shutdown {
            return response_error(output, id, -32600, "server has shut down");
        }
        if method == "initialize" {
            let Some(id) = id else {
                return notification_error(output, "initialize requires a request ID");
            };
            if self.initialized {
                return response_error(output, Some(id), -32600, "already initialized");
            }
            let request = match initialize(&params, self.service.document_symbols_enabled()) {
                Ok(request) => request,
                Err(()) => {
                    return response_error(
                        output,
                        Some(id),
                        -32602,
                        "invalid initialize parameters",
                    );
                }
            };
            return match self.service.frontend_session_open(request, stop) {
                Ok(session) => {
                    self.initialized = true;
                    let local_definition = session.local_definition_enabled;
                    let mut response = json!({"jsonrpc":"2.0","id":id,"result":{"capabilities":{
                        "positionEncoding":session.position_encoding,
                        "textDocumentSync":{"openClose":true,"change":2,"save":{"includeText":false}},
                        "diagnosticProvider":{"identifier":"wow.private-syntax","interFileDependencies":false,"workspaceDiagnostics":false},
                        "experimental":{"wow":{"session":session,"registry":self.service.registry()}}
                    },"serverInfo":{"name":"wow-private-syntax","version":env!("CARGO_PKG_VERSION")}}});
                    if local_definition {
                        response["result"]["capabilities"]["definitionProvider"] = json!(true);
                    }
                    if self.service.references_enabled() {
                        response["result"]["capabilities"]["referencesProvider"] = json!(true);
                    }
                    if self.service.document_symbols_enabled() {
                        response["result"]["capabilities"]["documentSymbolProvider"] =
                            json!({"workDoneProgress":session.profile_id == LSP_OWNED_PROFILE_ID});
                    }
                    if self.service.workspace_rebind_enabled() {
                        response["result"]["capabilities"]["workspace"] = json!({"workspaceFolders":{"supported":true,"changeNotifications":true}});
                    }
                    if session.profile_id == LSP_OWNED_PROFILE_ID {
                        response["result"]["capabilities"]["diagnosticProvider"]["workDoneProgress"] =
                            json!(true);
                        if local_definition {
                            response["result"]["capabilities"]["definitionProvider"] =
                                json!({"workDoneProgress":true});
                        }
                        if self.service.references_enabled() {
                            response["result"]["capabilities"]["referencesProvider"] =
                                json!({"workDoneProgress":true});
                        }
                    }
                    send(output, response)?;
                    Ok(Control::Continue)
                }
                Err(error) => domain_error(output, Some(id), error),
            };
        }
        if !self.initialized {
            return response_error(output, id, -32002, "server is not initialized");
        }
        if method == "initialized" && id.is_none() {
            if !params.is_empty() || self.ready {
                return notification_error(output, "invalid initialized notification");
            }
            self.ready = true;
            return Ok(Control::Continue);
        }
        if !self.ready {
            return response_error(output, id, -32002, "initialized notification required");
        }
        match method.as_str() {
            "textDocument/definition" => self.definition(id, &params, output, stop),
            "textDocument/references" => self.references(id, &params, output, stop),
            "textDocument/documentSymbol" if self.service.document_symbols_enabled() => {
                self.document_symbols(id, &params, output, stop)
            }
            "workspace/didChangeWorkspaceFolders" if self.service.workspace_rebind_enabled() => {
                if id.is_some() {
                    return response_error(
                        output,
                        id,
                        -32600,
                        "workspace synchronization requires a notification",
                    );
                }
                let request = match workspace_folders_change(&params) {
                    Ok(request) => request,
                    Err(()) => {
                        return notification_error(output, "invalid workspace folder change");
                    }
                };
                match self
                    .service
                    .frontend_workspace_folders_change(request, stop)
                {
                    Ok(_) => Ok(Control::Continue),
                    Err(error) if error.code() == ServiceErrorCode::InternalContractViolation => {
                        domain_error(output, None, error)?;
                        Err(())
                    }
                    Err(error) => domain_error(output, None, error),
                }
            }
            "shutdown" => {
                let Some(id) = id else {
                    return notification_error(output, "shutdown requires a request ID");
                };
                if !params.is_empty() {
                    return response_error(
                        output,
                        Some(id),
                        -32602,
                        "shutdown parameters must be empty",
                    );
                }
                match self.service.frontend_session_close() {
                    Ok(()) => {
                        self.shutdown = true;
                        send(output, json!({"jsonrpc":"2.0","id":id,"result":null}))?;
                        Ok(Control::Continue)
                    }
                    Err(error) => domain_error(output, Some(id), error),
                }
            }
            "textDocument/didOpen"
            | "textDocument/didChange"
            | "textDocument/didSave"
            | "textDocument/didClose" => {
                if id.is_some() {
                    return response_error(
                        output,
                        id,
                        -32600,
                        "document synchronization requires notifications",
                    );
                }
                self.synchronize(&method, &params, output, stop)
            }
            "textDocument/diagnostic" => {
                let Some(id) = id else {
                    return notification_error(output, "pull diagnostics requires a request ID");
                };
                let request = diagnostic_request(&params);
                let (uri, previous) = match request {
                    Ok(v) => v,
                    Err(()) => {
                        return response_error(
                            output,
                            Some(id),
                            -32602,
                            "invalid diagnostic parameters",
                        );
                    }
                };
                match self.service.frontend_document_diagnostics(
                    &uri,
                    previous.as_deref(),
                    id.clone(),
                    stop,
                ) {
                    Ok(report) => {
                        send(output, json!({"jsonrpc":"2.0","id":id,"result":report}))?;
                        Ok(Control::Continue)
                    }
                    Err(error) => domain_error(output, Some(id), error),
                }
            }
            "$/cancelRequest" if id.is_none() => {
                if only(&params, &["id"]).is_err() {
                    return notification_error(output, "invalid cancellation parameters");
                }
                let target = match params
                    .get("id")
                    .and_then(|v| serde_json::from_value::<LspRequestId>(v.clone()).ok())
                {
                    Some(v) => v,
                    None => return notification_error(output, "invalid cancellation target"),
                };
                match self.service.frontend_operation_cancel(target) {
                    Ok(_) => Ok(Control::Continue),
                    Err(error) => domain_error(output, None, error),
                }
            }
            _ if method.starts_with("$/") && id.is_none() => Ok(Control::Continue),
            _ => response_error(
                output,
                id,
                -32601,
                "method is not supported by the selected profile",
            ),
        }
    }
    fn synchronize<W: Write>(
        &mut self,
        method: &str,
        params: &Map<String, Value>,
        output: &mut W,
        stop: &AtomicBool,
    ) -> Result<Control, ()> {
        let uri = match document_uri(params) {
            Ok(uri) => uri,
            Err(()) => return notification_error(output, "invalid document identifier"),
        };
        let result = match method {
            "textDocument/didOpen" => (|| {
                only(params, &["textDocument"])?;
                let doc = params
                    .get("textDocument")
                    .and_then(Value::as_object)
                    .ok_or(())?;
                only(doc, &["uri", "languageId", "version", "text"])?;
                let request = LspDocumentOpenRequest {
                    uri,
                    language_id: string(doc, "languageId", 32)?,
                    version: doc.get("version").and_then(Value::as_i64).ok_or(())?,
                    content: string(doc, "text", LSP_MAX_FRAME_BYTES)?,
                };
                Ok::<_, ()>(
                    self.service
                        .frontend_document_open(request, stop)
                        .map(|_| ()),
                )
            })(),
            "textDocument/didChange" => {
                let applied = (|| {
                    only(params, &["textDocument", "contentChanges"])?;
                    let doc = params
                        .get("textDocument")
                        .and_then(Value::as_object)
                        .ok_or(())?;
                    only(doc, &["uri", "version"])?;
                    let version = doc.get("version").and_then(Value::as_i64).ok_or(())?;
                    let changes: Vec<LspContentChange> =
                        serde_json::from_value(params.get("contentChanges").cloned().ok_or(())?)
                            .map_err(|_| ())?;
                    Ok::<_, ()>(LspDocumentChange::Apply { version, changes })
                })();
                let change = applied.unwrap_or(LspDocumentChange::Rejected {
                    reason: LspSynchronizationRefusal::UnsupportedChangeShape,
                });
                Ok(self
                    .service
                    .frontend_document_change(LspDocumentChangeRequest { uri, change }, stop)
                    .map(|_| ()))
            }
            "textDocument/didSave" => (|| {
                only(params, &["textDocument", "text"])?;
                let content = params
                    .get("text")
                    .map(|v| v.as_str().map(str::to_owned).ok_or(()))
                    .transpose()?;
                Ok::<_, ()>(
                    self.service
                        .frontend_document_save(LspDocumentSaveRequest { uri, content }, stop)
                        .map(|_| ()),
                )
            })(),
            "textDocument/didClose" => {
                if only(params, &["textDocument"]).is_err() {
                    Err(())
                } else {
                    Ok(self.service.frontend_document_close(&uri, stop))
                }
            }
            _ => Err(()),
        };
        match result {
            Ok(Ok(())) => Ok(Control::Continue),
            Ok(Err(error)) => domain_error(output, None, error),
            Err(()) => notification_error(output, "invalid document parameters"),
        }
    }
}
fn workspace_folders_change(
    params: &Map<String, Value>,
) -> Result<LspWorkspaceFoldersChangeRequest, ()> {
    only(params, &["event"])?;
    let event = params.get("event").and_then(Value::as_object).ok_or(())?;
    only(event, &["added", "removed"])?;
    let folder_uri = |field: &str| -> Result<String, ()> {
        let folders = event.get(field).and_then(Value::as_array).ok_or(())?;
        if folders.len() != 1 {
            return Err(());
        }
        let folder = folders[0].as_object().ok_or(())?;
        only(folder, &["uri", "name"])?;
        // The client label is required by LSP but is never a selector or path.
        string(folder, "name", 128)?;
        string(folder, "uri", 65536)
    };
    Ok(LspWorkspaceFoldersChangeRequest {
        removed_uri: folder_uri("removed")?,
        added_uri: folder_uri("added")?,
    })
}
fn initialize(
    params: &Map<String, Value>,
    document_symbols_enabled: bool,
) -> Result<LspSessionOpenRequest, ()> {
    only(
        params,
        &[
            "processId",
            "clientInfo",
            "locale",
            "rootPath",
            "rootUri",
            "initializationOptions",
            "capabilities",
            "trace",
            "workspaceFolders",
            "workDoneToken",
        ],
    )?;
    if params
        .get("initializationOptions")
        .is_some_and(|v| !v.is_null() && !v.as_object().is_some_and(Map::is_empty))
    {
        return Err(());
    }
    let capabilities = params
        .get("capabilities")
        .and_then(Value::as_object)
        .ok_or(())?;
    let client = params
        .get("clientInfo")
        .map(|v| v.as_object().ok_or(()))
        .transpose()?;
    let (client_name, client_version) = if let Some(client) = client {
        only(client, &["name", "version"])?;
        (
            string(client, "name", 128)?,
            client
                .get("version")
                .map(|v| {
                    v.as_str()
                        .filter(|s| s.len() <= 64)
                        .map(str::to_owned)
                        .ok_or(())
                })
                .transpose()?
                .unwrap_or_else(|| "unreported".into()),
        )
    } else {
        ("unreported-client".into(), "unreported".into())
    };
    let position_encodings = match capabilities
        .get("general")
        .and_then(Value::as_object)
        .and_then(|v| v.get("positionEncodings"))
    {
        None => Vec::new(),
        Some(v) => v
            .as_array()
            .filter(|v| v.len() <= 8)
            .ok_or(())?
            .iter()
            .map(|v| {
                v.as_str()
                    .filter(|s| s.len() <= 32)
                    .map(str::to_owned)
                    .ok_or(())
            })
            .collect::<Result<_, _>>()?,
    };
    let pull_diagnostics = capabilities
        .get("textDocument")
        .and_then(Value::as_object)
        .and_then(|v| v.get("diagnostic"))
        .is_some_and(Value::is_object);
    let workspace_uris = match params.get("workspaceFolders") {
        Some(Value::Array(folders)) => {
            if folders.len() > 1 {
                return Err(());
            }
            folders
                .iter()
                .map(|folder| {
                    let folder = folder.as_object().ok_or(())?;
                    only(folder, &["uri", "name"])?;
                    string(folder, "uri", 65536)
                })
                .collect::<Result<_, _>>()?
        }
        None | Some(Value::Null) => match params.get("rootUri") {
            None | Some(Value::Null) => Vec::new(),
            Some(Value::String(uri)) if uri.len() <= 65536 => vec![uri.clone()],
            _ => return Err(()),
        },
        _ => return Err(()),
    };
    Ok(LspSessionOpenRequest {
        client_name,
        client_version,
        position_encodings,
        pull_diagnostics,
        workspace_uris,
        document_symbols: if document_symbols_enabled {
            Some(document_symbols::client_capabilities(capabilities)?)
        } else {
            None
        },
    })
}
fn document_uri(params: &Map<String, Value>) -> Result<String, ()> {
    let document = params
        .get("textDocument")
        .and_then(Value::as_object)
        .ok_or(())?;
    string(document, "uri", 65536)
}
fn only(map: &Map<String, Value>, fields: &[&str]) -> Result<(), ()> {
    if map.keys().all(|key| fields.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(())
    }
}
fn string(map: &Map<String, Value>, field: &str, max: usize) -> Result<String, ()> {
    map.get(field)
        .and_then(Value::as_str)
        .filter(|v| v.len() <= max)
        .map(str::to_owned)
        .ok_or(())
}
fn send<W: Write>(output: &mut W, value: Value) -> Result<(), ()> {
    let bytes = super::json::encode(&value, LSP_MAX_OUTPUT_BYTES).map_err(|_| ())?;
    write_frame(output, &bytes, LSP_MAX_OUTPUT_BYTES).map_err(|_| ())
}
fn notification_error<W: Write>(output: &mut W, message: &'static str) -> Result<Control, ()> {
    send(
        output,
        json!({"jsonrpc":"2.0","method":"window/logMessage","params":{"type":1,"message":message}}),
    )?;
    Ok(Control::Continue)
}
fn response_error<W: Write>(
    output: &mut W,
    id: Option<LspRequestId>,
    code: i32,
    message: &'static str,
) -> Result<Control, ()> {
    if let Some(id) = id {
        send(
            output,
            json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}}),
        )?;
        Ok(Control::Continue)
    } else {
        notification_error(output, message)
    }
}
fn domain_error<W: Write>(
    output: &mut W,
    id: Option<LspRequestId>,
    error: ServiceError,
) -> Result<Control, ()> {
    if let Some(id) = id {
        send(output, domain_error_value(&id, &error))?;
        Ok(Control::Continue)
    } else {
        notification_error(
            output,
            if error.code() == ServiceErrorCode::IdentityMismatch {
                "document requires resynchronization with didClose/didOpen"
            } else {
                "document operation refused by the selected profile"
            },
        )
    }
}
fn domain_error_value(id: &LspRequestId, error: &ServiceError) -> Value {
    let code = match error.code() {
        ServiceErrorCode::IdentityMismatch => -32801,
        ServiceErrorCode::Cancelled => -32800,
        ServiceErrorCode::InvalidContext => -32002,
        ServiceErrorCode::InvalidRequest | ServiceErrorCode::InvalidConfiguration => -32602,
        ServiceErrorCode::InternalContractViolation | ServiceErrorCode::CanonicalizationFailed => {
            -32603
        }
        _ => -32001,
    };
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":error.message(),
        "data":{"code":error.code(),"message":error.message()}}})
}

fn diagnostic_request(params: &Map<String, Value>) -> Result<(String, Option<String>), ()> {
    only(
        params,
        &[
            "textDocument",
            "identifier",
            "previousResultId",
            "workDoneToken",
            "partialResultToken",
        ],
    )?;
    let uri = document_uri(params)?;
    if params
        .get("identifier")
        .is_some_and(|v| v.as_str() != Some("wow.private-syntax"))
    {
        return Err(());
    }
    let previous = params
        .get("previousResultId")
        .map(|v| {
            v.as_str()
                .filter(|s| s.len() <= 256)
                .map(str::to_owned)
                .ok_or(())
        })
        .transpose()?;
    Ok::<_, ()>((uri, previous))
}
