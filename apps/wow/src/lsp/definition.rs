//! Standard location projection of one exact Service-owned lexical query.
use super::*;
use wow_service::frontend::lsp::{LspDefinitionRequest, LspDefinitionResult, LspPosition};
impl Server {
    pub(super) fn definition<W: Write>(
        &mut self,
        id: Option<LspRequestId>,
        params: &Map<String, Value>,
        output: &mut W,
        stop: &AtomicBool,
    ) -> Result<Control, ()> {
        let Some(id) = id else {
            return notification_error(output, "definition requires a request ID");
        };
        let request = parse_request(params);
        let request = match request {
            Ok(request) => request,
            Err(()) => {
                return response_error(output, Some(id), -32602, "invalid definition parameters");
            }
        };
        match self.service.frontend_document_definition(request, stop) {
            Ok(result) => {
                send(output, result_response(&id, &result))?;
                Ok(Control::Continue)
            }
            Err(error) => domain_error(output, Some(id), error),
        }
    }
}

pub(super) fn result_response(id: &LspRequestId, result: &LspDefinitionResult) -> Value {
    if let Some(location) = &result.location {
        json!({"jsonrpc":"2.0","id":id,"result":[{"uri":location.uri,"range":location.range,"wow":result}]})
    } else {
        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32001,"message":"local definition not evaluated for this position","data":result}})
    }
}

pub(super) fn parse_request(params: &Map<String, Value>) -> Result<LspDefinitionRequest, ()> {
    only(
        params,
        &[
            "textDocument",
            "position",
            "workDoneToken",
            "partialResultToken",
        ],
    )?;
    let document = params
        .get("textDocument")
        .and_then(Value::as_object)
        .ok_or(())?;
    only(document, &["uri"])?;
    let uri = string(document, "uri", 65536)?;
    let position: LspPosition =
        serde_json::from_value(params.get("position").cloned().ok_or(())?).map_err(|_| ())?;
    Ok::<_, ()>(LspDefinitionRequest { uri, position })
}
