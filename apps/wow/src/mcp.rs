//! The first fixed, read-only MCP stdio profile.

use std::ffi::OsString;
use std::fmt;
use std::io::{self, BufRead, BufReader, Write};

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Number, Value, json};
use wow_service::frontend::{
    FrontendCapabilitySet, FrontendClientIdentity, FrontendService, FrontendSessionId,
    FrontendSessionOpenRequest, MCP_MAX_FRAME_BYTES, MCP_MAX_JSON_DEPTH, MCP_MAX_OUTPUT_BYTES,
    MCP_PROFILE_RESOURCE_URI, MCP_PROTOCOL_REVISION, MCP_STATUS_TOOL_NAME, MCP_STDIO_PROFILE_ID,
};
use wow_service::{ServiceError, ServiceErrorCode};

pub mod transport_capabilities;

const SERVER_NAME: &str = "wow-dev-framework";
const INITIALIZE_METHOD: &str = "initialize";
const INITIALIZED_NOTIFICATION: &str = "notifications/initialized";
const MAX_METHOD_BYTES: usize = 128;
const MAX_REQUEST_ID_BYTES: usize = 256;
const STDERR_MESSAGE: &str = "wow: MCP stdio request failed";

pub fn run(arguments: Vec<OsString>) -> u8 {
    if arguments.len() != 5
        || arguments[0] != "mcp"
        || arguments[1] != "--transport"
        || arguments[2] != "stdio"
        || arguments[3] != "--profile"
        || arguments[4] != MCP_STDIO_PROFILE_ID
    {
        diagnostic("invalid MCP command; use the exact stdio transport and profile");
        return 64;
    }

    let service = match FrontendService::new() {
        Ok(service) => service,
        Err(_) => {
            diagnostic("frontend registry initialization failed");
            return 64;
        }
    };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();
    run_stdio(
        &service,
        BufReader::new(stdin.lock()),
        &mut stdout.lock(),
        &mut stderr.lock(),
    )
}

fn run_stdio<R: BufRead, W: Write, E: Write>(
    service: &FrontendService,
    mut reader: R,
    stdout: &mut W,
    stderr: &mut E,
) -> u8 {
    let mut server = McpServer {
        service,
        phase: SessionPhase::AwaitInitialize,
    };
    let mut frame = Vec::with_capacity(4096);
    let mut exit_code = 0;

    loop {
        frame.clear();
        match read_frame(&mut reader, &mut frame) {
            Ok(false) => break,
            Ok(true) => match server.handle_frame(&frame, stdout) {
                Ok(Control::Continue) => {}
                Ok(Control::Close(code)) => {
                    exit_code = code;
                    break;
                }
                Err(()) => {
                    diagnostic_to(stderr, STDERR_MESSAGE);
                    exit_code = 4;
                    break;
                }
            },
            Err(FrameError::TooLarge) => {
                let response =
                    error_response(Value::Null, -32700, "input frame exceeds profile limit");
                if write_message(stdout, &response).is_err() {
                    diagnostic_to(stderr, "wow: MCP stdout write failed");
                    exit_code = 4;
                } else {
                    exit_code = 2;
                }
                break;
            }
            Err(FrameError::Truncated) => {
                let response = error_response(Value::Null, -32700, "unterminated input frame");
                if write_message(stdout, &response).is_err() {
                    diagnostic_to(stderr, "wow: MCP stdout write failed");
                    exit_code = 4;
                } else {
                    exit_code = 2;
                }
                break;
            }
            Err(FrameError::Io) => {
                diagnostic_to(stderr, "wow: MCP stdin read failed");
                exit_code = 3;
                break;
            }
        }
    }

    if let Some(session_id) = server.session_id()
        && server.service.frontend_session_close(session_id).is_err()
    {
        diagnostic_to(stderr, "wow: MCP frontend session close failed");
        exit_code = 4;
    }
    if matches!(server.phase, SessionPhase::AwaitInitialized { .. }) && exit_code == 0 {
        diagnostic_to(stderr, "wow: MCP input ended before initialized admission");
        exit_code = 2;
    }
    exit_code
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SessionPhase {
    AwaitInitialize,
    AwaitInitialized { session_id: FrontendSessionId },
    Ready { session_id: FrontendSessionId },
}

impl SessionPhase {
    const fn session_id(self) -> Option<FrontendSessionId> {
        match self {
            Self::AwaitInitialize => None,
            Self::AwaitInitialized { session_id } | Self::Ready { session_id } => Some(session_id),
        }
    }
}

struct McpServer<'a> {
    service: &'a FrontendService,
    phase: SessionPhase,
}

impl McpServer<'_> {
    fn session_id(&self) -> Option<FrontendSessionId> {
        self.phase.session_id()
    }

    fn handle_frame<W: Write>(&mut self, frame: &[u8], output: &mut W) -> Result<Control, ()> {
        let message = match parse_message(frame) {
            Ok(message) => message,
            Err(error) => {
                let response = error_response(error.id, error.code, error.message);
                write_message(output, &response).map_err(|_| ())?;
                return Ok(Control::Continue);
            }
        };
        self.handle_message(message, output)
    }

    fn handle_message<W: Write>(
        &mut self,
        message: RpcMessage,
        output: &mut W,
    ) -> Result<Control, ()> {
        if message.method == "ping" {
            if message.id.is_none() {
                return Ok(Control::Close(2));
            }
            if self.phase == SessionPhase::AwaitInitialize {
                return self.invalid_request(message.id, output);
            }
            if !message.params.is_empty() {
                return self.invalid_params(message.id, output);
            }
            write_result(output, message.id, json!({})).map_err(|_| ())?;
            return Ok(Control::Continue);
        }

        if message.method == INITIALIZE_METHOD {
            return self.initialize(message, output);
        }

        if message.method == INITIALIZED_NOTIFICATION {
            return self.initialized(message, output);
        }

        if message.id.is_none() {
            diagnostic("wow: unsupported MCP notification");
            return Ok(Control::Close(2));
        }

        match message.method.as_str() {
            "tools/list" => self.tools_list(message, output),
            "tools/call" => self.tools_call(message, output),
            "resources/list" => self.resources_list(message, output),
            "resources/read" => self.resources_read(message, output),
            _ => {
                let response = error_response(
                    message.id.unwrap_or(Value::Null),
                    -32601,
                    "method is not implemented by this profile",
                );
                write_message(output, &response).map_err(|_| ())?;
                Ok(Control::Continue)
            }
        }
    }

    fn initialize<W: Write>(&mut self, message: RpcMessage, output: &mut W) -> Result<Control, ()> {
        let Some(id) = message.id else {
            return Ok(Control::Close(2));
        };
        if self.phase != SessionPhase::AwaitInitialize {
            return self.invalid_request(Some(id), output);
        }
        let params = match initialize_params(&message.params) {
            Ok(params) => params,
            Err(()) => return self.invalid_params(Some(id), output),
        };
        if params.protocol_version != MCP_PROTOCOL_REVISION {
            let response =
                error_response(id, -32602, "only protocol revision 2025-11-25 is supported");
            write_message(output, &response).map_err(|_| ())?;
            return Ok(Control::Continue);
        }
        let client = match FrontendClientIdentity::new(params.client_name, params.client_version) {
            Ok(client) => client,
            Err(_) => return self.invalid_params(Some(id), output),
        };
        let opened = match self
            .service
            .frontend_session_open(FrontendSessionOpenRequest {
                protocol_revision: MCP_PROTOCOL_REVISION.into(),
                profile_id: MCP_STDIO_PROFILE_ID.into(),
                transport: "stdio".into(),
                client,
            }) {
            Ok(opened) => opened,
            Err(error) => return self.service_error(Some(id), error, output),
        };
        self.phase = SessionPhase::AwaitInitialized {
            session_id: opened.session_id,
        };
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": MCP_PROTOCOL_REVISION,
                "capabilities": {
                    "tools": {"listChanged": false},
                    "resources": {}
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "title": "WoW Dev Framework",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "instructions": "This stdio profile exposes only the fixed wow.status tool and exact profile manifest resource. Status reports frontend session and registry state; it does not inspect a project or run analysis."
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn initialized<W: Write>(
        &mut self,
        message: RpcMessage,
        output: &mut W,
    ) -> Result<Control, ()> {
        if message.id.is_some() || !message.params.is_empty() {
            if message.id.is_some() {
                return self.invalid_request(message.id, output);
            }
            diagnostic("wow: invalid initialized notification");
            return Ok(Control::Close(2));
        }
        let SessionPhase::AwaitInitialized { session_id } = self.phase else {
            diagnostic("wow: initialized notification outside the negotiated phase");
            return Ok(Control::Close(2));
        };
        if self.service.frontend_session_get(session_id).is_err() {
            diagnostic("wow: frontend session admission failed");
            return Ok(Control::Close(3));
        }
        self.phase = SessionPhase::Ready { session_id };
        Ok(Control::Continue)
    }

    fn tools_list<W: Write>(&self, message: RpcMessage, output: &mut W) -> Result<Control, ()> {
        let Some(id) = message.id else {
            return Ok(Control::Close(2));
        };
        if !self.require_ready(Some(&id), output)? {
            return Ok(Control::Continue);
        }
        if !message.params.is_empty() {
            return self.invalid_params(Some(id), output);
        }
        let capabilities = self.service.frontend_capabilities();
        let Some(tool) = selected_status_tool(capabilities) else {
            return self.internal_error(Some(id), output);
        };
        let input_schema = match serde_json::from_str::<Value>(&tool.input_schema_json) {
            Ok(value) => value,
            Err(_) => return self.internal_error(Some(id), output),
        };
        let output_schema = match serde_json::from_str::<Value>(&tool.output_schema_json) {
            Ok(value) => value,
            Err(_) => return self.internal_error(Some(id), output),
        };
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [{
                    "name": tool.name,
                    "title": "WoW frontend status",
                    "description": tool.description,
                    "inputSchema": input_schema,
                    "outputSchema": output_schema,
                    "annotations": {
                        "readOnlyHint": tool.read_only_hint,
                        "destructiveHint": tool.destructive_hint,
                        "idempotentHint": tool.idempotent_hint,
                        "openWorldHint": tool.open_world_hint
                    }
                }]
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn tools_call<W: Write>(&self, message: RpcMessage, output: &mut W) -> Result<Control, ()> {
        let Some(id) = message.id else {
            return Ok(Control::Close(2));
        };
        let Some(session_id) = self.ready_or_reject(Some(&id), output)? else {
            return Ok(Control::Continue);
        };
        let params = match exact_object(&message.params, &["name", "arguments"]) {
            Ok(params) => params,
            Err(()) => return self.invalid_params(Some(id), output),
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return self.invalid_params(Some(id), output);
        };
        if name != MCP_STATUS_TOOL_NAME {
            return self.invalid_params(Some(id), output);
        }
        if let Some(arguments) = params.get("arguments")
            && (!arguments.is_object() || !arguments.as_object().is_some_and(Map::is_empty))
        {
            return self.tool_arguments_error(id, output);
        }
        let status = match self.service.frontend_status(session_id) {
            Ok(status) => status,
            Err(error) => return self.service_error(Some(id), error, output),
        };
        let structured = serde_json::to_value(&status).map_err(|_| ())?;
        let text = serde_json::to_string(&status).map_err(|_| ())?;
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{"type": "text", "text": text}],
                "structuredContent": structured,
                "isError": false
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn resources_list<W: Write>(&self, message: RpcMessage, output: &mut W) -> Result<Control, ()> {
        let Some(id) = message.id else {
            return Ok(Control::Close(2));
        };
        if !self.require_ready(Some(&id), output)? {
            return Ok(Control::Continue);
        }
        if !message.params.is_empty() {
            return self.invalid_params(Some(id), output);
        }
        let capabilities = self.service.frontend_capabilities();
        let Some(resource) = selected_profile_resource(capabilities) else {
            return self.internal_error(Some(id), output);
        };
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resources": [{
                    "uri": resource.uri,
                    "name": resource.name,
                    "description": format!("Exact manifest for profile {}; digest {}", MCP_STDIO_PROFILE_ID, capabilities.profile_digest),
                    "mimeType": resource.mime_type
                }]
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn resources_read<W: Write>(&self, message: RpcMessage, output: &mut W) -> Result<Control, ()> {
        let Some(id) = message.id else {
            return Ok(Control::Close(2));
        };
        if !self.require_ready(Some(&id), output)? {
            return Ok(Control::Continue);
        }
        let params = match exact_object(&message.params, &["uri"]) {
            Ok(params) => params,
            Err(()) => return self.invalid_params(Some(id), output),
        };
        if params.get("uri").and_then(Value::as_str) != Some(MCP_PROFILE_RESOURCE_URI) {
            return self.invalid_params(Some(id), output);
        }
        let capabilities = self.service.frontend_capabilities();
        let Some(resource) = selected_profile_resource(capabilities) else {
            return self.internal_error(Some(id), output);
        };
        let manifest = match serde_json::to_string(capabilities) {
            Ok(manifest) => manifest,
            Err(_) => return self.internal_error(Some(id), output),
        };
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "contents": [{
                    "uri": resource.uri,
                    "mimeType": resource.mime_type,
                    "text": manifest
                }]
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn require_ready<W: Write>(&self, id: Option<&Value>, output: &mut W) -> Result<bool, ()> {
        if matches!(self.phase, SessionPhase::Ready { .. }) {
            Ok(true)
        } else {
            self.invalid_request(id.cloned(), output)?;
            Ok(false)
        }
    }

    fn ready_or_reject<W: Write>(
        &self,
        id: Option<&Value>,
        output: &mut W,
    ) -> Result<Option<FrontendSessionId>, ()> {
        let SessionPhase::Ready { session_id } = self.phase else {
            self.invalid_request(id.cloned(), output)?;
            return Ok(None);
        };
        Ok(Some(session_id))
    }

    fn invalid_request<W: Write>(&self, id: Option<Value>, output: &mut W) -> Result<Control, ()> {
        let response = error_response(
            id.unwrap_or(Value::Null),
            -32600,
            "request is not admitted in the current MCP lifecycle phase",
        );
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn invalid_params<W: Write>(&self, id: Option<Value>, output: &mut W) -> Result<Control, ()> {
        let response = error_response(
            id.unwrap_or(Value::Null),
            -32602,
            "parameters do not match the exact implemented schema",
        );
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn tool_arguments_error<W: Write>(&self, id: Value, output: &mut W) -> Result<Control, ()> {
        let response = json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [{
                    "type": "text",
                    "text": "arguments do not match the exact input schema for wow.status"
                }],
                "isError": true
            }
        });
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn internal_error<W: Write>(&self, id: Option<Value>, output: &mut W) -> Result<Control, ()> {
        let response = error_response(
            id.unwrap_or(Value::Null),
            -32603,
            "frontend registry does not match the fixed transport route",
        );
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }

    fn service_error<W: Write>(
        &self,
        id: Option<Value>,
        error: ServiceError,
        output: &mut W,
    ) -> Result<Control, ()> {
        let code = if error.code() == ServiceErrorCode::InvalidRequest {
            -32600
        } else {
            -32000
        };
        let response = error_response(id.unwrap_or(Value::Null), code, error.message());
        write_message(output, &response).map_err(|_| ())?;
        Ok(Control::Continue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Continue,
    Close(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrameError {
    TooLarge,
    Truncated,
    Io,
}

fn read_frame<R: BufRead>(reader: &mut R, frame: &mut Vec<u8>) -> Result<bool, FrameError> {
    loop {
        let available = reader.fill_buf().map_err(|_| FrameError::Io)?;
        if available.is_empty() {
            return if frame.is_empty() {
                Ok(false)
            } else {
                Err(FrameError::Truncated)
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.unwrap_or(available.len());
        if frame
            .len()
            .checked_add(count)
            .is_none_or(|size| size > MCP_MAX_FRAME_BYTES)
        {
            return Err(FrameError::TooLarge);
        }
        frame.extend_from_slice(&available[..count]);
        reader.consume(count + usize::from(newline.is_some()));
        if newline.is_some() {
            return Ok(true);
        }
    }
}

struct StrictJson(Value);

impl<'de> Deserialize<'de> for StrictJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        StrictValueSeed { depth: 0 }
            .deserialize(deserializer)
            .map(Self)
    }
}

#[derive(Clone, Copy)]
struct StrictValueSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for StrictValueSeed {
    type Value = Value;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        if self.depth > MCP_MAX_JSON_DEPTH {
            return Err(de::Error::custom("JSON nesting exceeds profile limit"));
        }
        deserializer.deserialize_any(StrictValueVisitor { depth: self.depth })
    }
}

struct StrictValueVisitor {
    depth: usize,
}

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded JSON value with unique object keys")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("invalid JSON number"))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(Value::String(value.to_owned()))
    }

    fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
        Ok(Value::String(value))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element_seed(StrictValueSeed {
            depth: self.depth + 1,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            let value = object.next_value_seed(StrictValueSeed {
                depth: self.depth + 1,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

struct RpcMessage {
    id: Option<Value>,
    method: String,
    params: Map<String, Value>,
}

struct RpcParseError {
    id: Value,
    code: i32,
    message: &'static str,
}

fn parse_message(frame: &[u8]) -> Result<RpcMessage, RpcParseError> {
    let StrictJson(value) =
        serde_json::from_slice::<StrictJson>(frame).map_err(|_| RpcParseError {
            id: Value::Null,
            code: -32700,
            message: "invalid JSON message",
        })?;
    let Some(object) = value.as_object() else {
        return Err(RpcParseError {
            id: Value::Null,
            code: -32600,
            message: "JSON-RPC message must be an object",
        });
    };
    let id = object
        .get("id")
        .filter(|value| valid_request_id(value))
        .cloned();
    if object
        .keys()
        .any(|key| !matches!(key.as_str(), "jsonrpc" | "id" | "method" | "params"))
    {
        return Err(RpcParseError {
            id: id.unwrap_or(Value::Null),
            code: -32600,
            message: "JSON-RPC message contains an unsupported field",
        });
    }
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(RpcParseError {
            id: id.unwrap_or(Value::Null),
            code: -32600,
            message: "JSON-RPC version must be 2.0",
        });
    }
    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return Err(RpcParseError {
            id: id.unwrap_or(Value::Null),
            code: -32600,
            message: "JSON-RPC method must be a string",
        });
    };
    if method.is_empty() || method.len() > MAX_METHOD_BYTES || method.chars().any(char::is_control)
    {
        return Err(RpcParseError {
            id: id.unwrap_or(Value::Null),
            code: -32600,
            message: "JSON-RPC method is outside the supported bound",
        });
    }
    if object.contains_key("id") && id.is_none() {
        return Err(RpcParseError {
            id: Value::Null,
            code: -32600,
            message: "JSON-RPC request id must be a string or integer",
        });
    }
    let params = match object.get("params") {
        None => Map::new(),
        Some(Value::Object(params)) => params.clone(),
        Some(_) => {
            return Err(RpcParseError {
                id: id.unwrap_or(Value::Null),
                code: -32600,
                message: "JSON-RPC params must be an object for this profile",
            });
        }
    };
    Ok(RpcMessage {
        id,
        method: method.to_owned(),
        params,
    })
}

fn valid_request_id(value: &Value) -> bool {
    match value {
        Value::String(value) => {
            value.len() <= MAX_REQUEST_ID_BYTES && !value.chars().any(char::is_control)
        }
        Value::Number(value) => value.is_i64() || value.is_u64(),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => false,
    }
}

struct InitializeParams {
    protocol_version: String,
    client_name: String,
    client_version: String,
}

fn initialize_params(params: &Map<String, Value>) -> Result<InitializeParams, ()> {
    if params.keys().any(|key| {
        !matches!(
            key.as_str(),
            "protocolVersion" | "capabilities" | "clientInfo" | "_meta"
        )
    }) {
        return Err(());
    }
    if params
        .get("_meta")
        .is_some_and(|meta| !meta.as_object().is_some_and(Map::is_empty))
    {
        return Err(());
    }
    let protocol_version = bounded_string(params.get("protocolVersion"), 64)?;
    let capabilities = params
        .get("capabilities")
        .and_then(Value::as_object)
        .ok_or(())?;
    if capabilities.keys().any(|key| {
        !matches!(
            key.as_str(),
            "roots" | "sampling" | "elicitation" | "tasks" | "experimental"
        )
    }) || capabilities.values().any(|value| !value.is_object())
    {
        return Err(());
    }
    let client_info = params
        .get("clientInfo")
        .and_then(Value::as_object)
        .ok_or(())?;
    if client_info.keys().any(|key| {
        !matches!(
            key.as_str(),
            "name" | "version" | "title" | "description" | "icons" | "websiteUrl"
        )
    }) {
        return Err(());
    }
    for field in ["title", "description", "websiteUrl"] {
        if let Some(value) = client_info.get(field) {
            let _ = bounded_string(Some(value), 512)?;
        }
    }
    if let Some(icons) = client_info.get("icons")
        && (!icons.as_array().is_some_and(|icons| icons.len() <= 16)
            || icons
                .as_array()
                .is_some_and(|icons| icons.iter().any(|icon| !valid_client_icon(icon))))
    {
        return Err(());
    }
    Ok(InitializeParams {
        protocol_version,
        client_name: bounded_string(client_info.get("name"), 128)?,
        client_version: bounded_string(client_info.get("version"), 64)?,
    })
}

fn valid_client_icon(icon: &Value) -> bool {
    let Some(icon) = icon.as_object() else {
        return false;
    };
    if icon
        .keys()
        .any(|key| !matches!(key.as_str(), "src" | "mimeType" | "sizes"))
    {
        return false;
    }
    for field in ["src", "mimeType"] {
        if icon
            .get(field)
            .is_some_and(|value| bounded_string(Some(value), 512).is_err())
        {
            return false;
        }
    }
    icon.get("sizes").is_none_or(|sizes| {
        sizes.as_array().is_some_and(|sizes| {
            sizes.len() <= 16
                && sizes
                    .iter()
                    .all(|size| bounded_string(Some(size), 32).is_ok())
        })
    })
}

fn bounded_string(value: Option<&Value>, max_bytes: usize) -> Result<String, ()> {
    let value = value.and_then(Value::as_str).ok_or(())?;
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        return Err(());
    }
    Ok(value.to_owned())
}

fn exact_object<'a>(
    params: &'a Map<String, Value>,
    allowed: &[&str],
) -> Result<&'a Map<String, Value>, ()> {
    if params.keys().any(|key| !allowed.contains(&key.as_str())) {
        Err(())
    } else {
        Ok(params)
    }
}

fn selected_status_tool(
    capabilities: &FrontendCapabilitySet,
) -> Option<&wow_service::frontend::FrontendMcpToolDescriptor> {
    (capabilities.tools.len() == 1)
        .then(|| capabilities.tools.first())
        .flatten()
        .filter(|tool| {
            tool.name.as_ref() == MCP_STATUS_TOOL_NAME
                && tool.service_operation.as_ref() == "frontend_status"
                && tool.read_only_hint
                && !tool.destructive_hint
                && !tool.open_world_hint
        })
}

fn selected_profile_resource(
    capabilities: &FrontendCapabilitySet,
) -> Option<&wow_service::frontend::FrontendMcpResourceDescriptor> {
    (capabilities.resources.len() == 1)
        .then(|| capabilities.resources.first())
        .flatten()
        .filter(|resource| resource.uri.as_ref() == MCP_PROFILE_RESOURCE_URI)
}

fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}})
}

fn write_result<W: Write>(output: &mut W, id: Option<Value>, result: Value) -> io::Result<()> {
    let Some(id) = id else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "response id is required",
        ));
    };
    write_message(output, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

fn write_message<W: Write>(output: &mut W, message: &Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(message).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "MCP response serialization failed",
        )
    })?;
    if bytes.len() > MCP_MAX_OUTPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "MCP response exceeds profile limit",
        ));
    }
    output.write_all(&bytes)?;
    output.write_all(b"\n")?;
    output.flush()
}

fn diagnostic(message: &str) {
    diagnostic_to(&mut io::stderr().lock(), message);
}

fn diagnostic_to<W: Write>(stderr: &mut W, message: &str) {
    let _ = writeln!(stderr, "{message}");
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn initialize(id: u64) -> String {
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":{id},\"method\":\"initialize\",\"params\":{{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{{}},\"clientInfo\":{{\"name\":\"test-client\",\"version\":\"1\"}}}}}}"
        )
    }

    fn response_lines(output: &[u8]) -> Result<Vec<Value>, serde_json::Error> {
        output
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .map(serde_json::from_slice)
            .collect()
    }

    #[test]
    fn malformed_and_preinitialized_requests_do_not_reach_tool_dispatch()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut input = String::from("{bad json}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/list\"}\n");
        input.push_str(&initialize(1));
        input.push('\n');
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/list\"}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":8,\"method\":\"tools/list\"}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"wow.search\",\"arguments\":{}}}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"wow.status\",\"arguments\":{\"extra\":true}}}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":5,\"method\":\"tools/call\",\"params\":{\"name\":\"wow.status\",\"arguments\":{}}}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":6,\"method\":\"resources/list\"}\n");
        input.push_str("{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"resources/read\",\"params\":{\"uri\":\"wow://profiles/mcp-2025-11-25-stdio-v1\"}}\n");

        let service = FrontendService::new()?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run_stdio(
            &service,
            Cursor::new(input.into_bytes()),
            &mut stdout,
            &mut stderr,
        );
        assert_eq!(exit, 0);
        let responses = response_lines(&stdout)?;
        assert_eq!(responses.len(), 10);
        assert_eq!(responses[0]["error"]["code"], -32700);
        assert_eq!(responses[1]["error"]["code"], -32600);
        assert_eq!(
            responses[2]["result"]["protocolVersion"],
            MCP_PROTOCOL_REVISION
        );
        assert_eq!(responses[3]["error"]["code"], -32600);
        assert_eq!(
            responses[4]["result"]["tools"].as_array().map(Vec::len),
            Some(1)
        );
        assert_eq!(
            responses[4]["result"]["tools"][0]["name"],
            MCP_STATUS_TOOL_NAME
        );
        assert_eq!(responses[5]["error"]["code"], -32602);
        assert_eq!(responses[6]["result"]["isError"], true);
        assert!(responses[6].get("error").is_none());
        assert_eq!(responses[7]["result"]["structuredContent"]["state"], "open");
        assert_eq!(
            responses[8]["result"]["resources"][0]["uri"],
            MCP_PROFILE_RESOURCE_URI
        );
        assert_eq!(
            responses[9]["result"]["contents"][0]["uri"],
            MCP_PROFILE_RESOURCE_URI
        );
        let manifest_text = responses[9]["result"]["contents"][0]["text"]
            .as_str()
            .ok_or_else(|| io::Error::other("profile resource text is missing"))?;
        let manifest: Value = serde_json::from_str(manifest_text)?;
        assert_eq!(manifest["profile"]["profileId"], MCP_STDIO_PROFILE_ID);
        assert_eq!(manifest["tools"].as_array().map(Vec::len), Some(1));
        let session_id = responses[7]["result"]["structuredContent"]["sessionId"]
            .as_u64()
            .ok_or_else(|| io::Error::other("frontend session id is missing"))?;
        let session_id = FrontendSessionId::new(session_id)
            .ok_or_else(|| io::Error::other("frontend session id is invalid"))?;
        assert!(service.frontend_session_get(session_id).is_err());
        assert!(stderr.is_empty());
        Ok(())
    }

    #[test]
    fn request_ids_are_strings_or_integer_numbers_only() -> Result<(), Box<dyn std::error::Error>> {
        for request in [
            br#"{"jsonrpc":"2.0","id":"client-request","method":"tools/list"}"#.as_slice(),
            br#"{"jsonrpc":"2.0","id":0,"method":"tools/list"}"#.as_slice(),
        ] {
            assert!(parse_message(request).is_ok());
        }
        for request in [
            br#"{"jsonrpc":"2.0","id":null,"method":"tools/list"}"#.as_slice(),
            br#"{"jsonrpc":"2.0","id":1.25,"method":"tools/list"}"#.as_slice(),
        ] {
            let error = parse_message(request)
                .err()
                .ok_or_else(|| io::Error::other("invalid request id was admitted"))?;
            assert_eq!(error.code, -32600);
        }
        Ok(())
    }

    #[test]
    fn invalid_known_tool_arguments_are_tool_errors_before_service_dispatch()
    -> Result<(), Box<dyn std::error::Error>> {
        let service = FrontendService::new()?;
        let opened = service.frontend_session_open(FrontendSessionOpenRequest {
            protocol_revision: MCP_PROTOCOL_REVISION.into(),
            profile_id: MCP_STDIO_PROFILE_ID.into(),
            transport: "stdio".into(),
            client: FrontendClientIdentity::new("test-client", "1")?,
        })?;
        let session_id = opened.session_id;
        service.frontend_session_close(session_id)?;
        assert!(service.frontend_status(session_id).is_err());

        let mut server = McpServer {
            service: &service,
            phase: SessionPhase::Ready { session_id },
        };
        let frames = [
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"wow.status","arguments":{"extra":true}}}"#,
            r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"arguments":{}}}"#,
            r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"wow.search","arguments":{}}}"#,
            r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"wow.status","arguments":{}}}"#,
        ];
        let mut output = Vec::new();
        for frame in frames {
            assert!(matches!(
                server.handle_frame(frame.as_bytes(), &mut output),
                Ok(Control::Continue)
            ));
        }

        let responses = response_lines(&output)?;
        assert_eq!(responses.len(), 4);
        assert_eq!(responses[0]["id"], 1);
        assert_eq!(responses[0]["result"]["isError"], true);
        assert!(responses[0].get("error").is_none());
        assert_eq!(responses[1]["error"]["code"], -32602);
        assert_eq!(responses[2]["error"]["code"], -32602);
        assert_eq!(
            responses[3]["error"]["message"],
            "frontend session is not open"
        );
        Ok(())
    }

    #[test]
    fn duplicate_keys_depth_and_frame_limits_fail_before_dispatch()
    -> Result<(), Box<dyn std::error::Error>> {
        let duplicate = parse_message(br#"{"jsonrpc":"2.0","id":1,"id":2,"method":"tools/list"}"#);
        assert!(duplicate.is_err());

        let nested = format!(
            "{}0{}",
            "[".repeat(MCP_MAX_JSON_DEPTH + 1),
            "]".repeat(MCP_MAX_JSON_DEPTH + 1)
        );
        assert!(parse_message(nested.as_bytes()).is_err());

        let service = FrontendService::new()?;
        let oversized = vec![b'a'; MCP_MAX_FRAME_BYTES + 1];
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let exit = run_stdio(&service, Cursor::new(oversized), &mut stdout, &mut stderr);
        assert_eq!(exit, 2);
        assert_eq!(response_lines(&stdout)?[0]["error"]["code"], -32700);
        assert!(stderr.is_empty());
        Ok(())
    }
}
