//! Opt-in registered native metadata query; shares the existing MCP framing.
use super::*;
use std::{path::Path, sync::Arc};
use wow_service::{
    context::{CONTEXT_MAP_HOST_INPUT_MAX_BYTES, ContextMapHostInput, ContextMapStatus},
    frontend::{FrontendContextInspectRequest, MCP_CONTEXT_MAX_OUTPUT_BYTES},
    search::LocalMetadataPermission,
};

pub(super) fn enabled(service: &FrontendService) -> bool {
    service.frontend_capabilities().profile.profile_id.as_ref() == MCP_CONTEXT_PROFILE_ID
}

pub(super) fn run(arguments: Vec<OsString>) -> u8 {
    if arguments.len() != 8
        || arguments[0] != "mcp"
        || arguments[1] != "--transport"
        || arguments[2] != "stdio"
        || arguments[3] != "--profile"
        || arguments[4] != MCP_CONTEXT_PROFILE_ID
        || arguments[5] != "--context-host"
        || arguments[7] != "--allow-local-metadata-only"
    {
        diagnostic("invalid registered Context MCP command");
        return 64;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let signal_stop = stop.clone();
    if ctrlc::set_handler(move || signal_stop.store(true, Ordering::Release)).is_err() {
        diagnostic("cannot install native Context cancellation handler");
        return 64;
    }
    let path = Path::new(&arguments[6]);
    if !path.is_absolute() || !std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file()) {
        diagnostic("context host must be one explicit absolute regular file");
        return 64;
    }
    let host = crate::graph::read_file(path, CONTEXT_MAP_HOST_INPUT_MAX_BYTES, &stop)
        .map_err(|_| ())
        .and_then(|bytes| ContextMapHostInput::from_json_slice(&bytes, &stop).map_err(|_| ()));
    let service = match host.and_then(|host| {
        FrontendService::new_with_context_registration(
            host,
            LocalMetadataPermission::authorize_local_metadata_only(),
            &stop,
        )
        .map_err(|_| ())
    }) {
        Ok(service) => service,
        Err(()) => {
            diagnostic("invalid or refused registered native Context host");
            return 64;
        }
    };
    run_stdio_with_stop(
        &service,
        BufReader::new(io::stdin().lock()),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
        &stop,
    )
}

pub(super) fn instructions(service: &FrontendService) -> &'static str {
    if enabled(service) {
        "This profile inspects native metadata for the operator-configured project registration, exact retained generation and explicit root. Read the exact profile resource for registration, schemas and limits. Source bodies and wire progress/cancellation are unavailable."
    } else {
        "This stdio profile exposes only the fixed wow.status tool and exact profile manifest resource. Status reports frontend session and registry state; it does not inspect a project or run analysis."
    }
}

pub(super) fn listed_tools(capabilities: &FrontendCapabilitySet) -> Result<Vec<Value>, ()> {
    if selected_status_tool(capabilities).is_none() {
        return Err(());
    }
    capabilities.tools.iter().map(|tool| {
        let title = match tool.name.as_ref() {
            MCP_STATUS_TOOL_NAME => "WoW frontend status",
            MCP_CONTEXT_TOOL_NAME if capabilities.profile.profile_id.as_ref() == MCP_CONTEXT_PROFILE_ID => "WoW native Context inspection",
            _ => return Err(()),
        };
        Ok(json!({"name":tool.name,"title":title,"description":tool.description,
            "inputSchema":serde_json::from_str::<Value>(&tool.input_schema_json).map_err(|_|())?,
            "outputSchema":serde_json::from_str::<Value>(&tool.output_schema_json).map_err(|_|())?,
            "annotations":{"readOnlyHint":tool.read_only_hint,"destructiveHint":tool.destructive_hint,
                "idempotentHint":tool.idempotent_hint,"openWorldHint":tool.open_world_hint}}))
    }).collect()
}

impl McpServer<'_> {
    pub(super) fn context_inspect_call<W: Write>(
        &self,
        id: Value,
        arguments: Option<&Value>,
        output: &mut W,
    ) -> Result<Control, ()> {
        let Some(session_id) = self.session_id() else {
            return self.invalid_request(Some(id), output);
        };
        let Some(arguments) = arguments.filter(|value| value.is_object()) else {
            return self.tool_arguments_error(id, output);
        };
        let bytes = serde_json::to_vec(arguments).map_err(|_| ())?;
        let request = match FrontendContextInspectRequest::from_json_slice(&bytes, self.stop) {
            Ok(request) => request,
            Err(_) => return self.tool_arguments_error(id, output),
        };
        // One registered Service operation, no owner logic or retry in the app.
        let result = match self
            .service
            .frontend_context_inspect(session_id, &request, self.stop)
        {
            Ok(result) => result,
            Err(error) => {
                let text = serde_json::to_string(&json!({"code":format!("{:?}",error.code()),
                    "message":"registered native Context inspection failed","operationId":null}))
                .map_err(|_| ())?;
                write_message(
                    output,
                    &json!({"jsonrpc":"2.0","id":id,"result":{
                    "content":[{"type":"text","text":text}],"isError":true}}),
                )
                .map_err(|_| ())?;
                return Ok(Control::Continue);
            }
        };
        let structured: Value = serde_json::from_slice(result.canonical_bytes()).map_err(|_| ())?;
        let text = std::str::from_utf8(result.canonical_bytes()).map_err(|_| ())?;
        let is_error = result.failure_code().is_some()
            || matches!(
                result.status(),
                ContextMapStatus::Failed | ContextMapStatus::Cancelled
            );
        write_message_with_limit(output, &json!({"jsonrpc":"2.0","id":id,"result":{
            "content":[{"type":"text","text":text}],"structuredContent":structured,"isError":is_error}}),
            MCP_CONTEXT_MAX_OUTPUT_BYTES).map_err(|_|())?;
        Ok(Control::Continue)
    }
}
