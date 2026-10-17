//! Standard locations from one exact Service-owned local-reference operation.
use super::*;
use wow_service::frontend::lsp::{LspPosition, LspReferencesRequest};
impl Server {
    pub(super) fn references<W: Write>(
        &mut self,
        id: Option<LspRequestId>,
        params: &Map<String, Value>,
        output: &mut W,
        stop: &AtomicBool,
    ) -> Result<Control, ()> {
        let Some(id) = id else {
            return notification_error(output, "references requires a request ID");
        };
        if !self.service.references_enabled() {
            return response_error(
                output,
                Some(id),
                -32601,
                "method is not supported by the selected profile",
            );
        }
        let request = (|| {
            only(
                params,
                &[
                    "textDocument",
                    "position",
                    "context",
                    "workDoneToken",
                    "partialResultToken",
                ],
            )?;
            for name in ["workDoneToken", "partialResultToken"] {
                if let Some(token) = params.get(name) {
                    let valid = match token {
                        Value::String(text) => {
                            text.len() <= 128 && !text.chars().any(char::is_control)
                        }
                        Value::Number(number) => number
                            .as_i64()
                            .is_some_and(|value| i32::try_from(value).is_ok()),
                        _ => false,
                    };
                    if !valid {
                        return Err(());
                    }
                }
            }
            let document = params
                .get("textDocument")
                .and_then(Value::as_object)
                .ok_or(())?;
            only(document, &["uri"])?;
            let uri = string(document, "uri", 65536)?;
            let position: LspPosition =
                serde_json::from_value(params.get("position").cloned().ok_or(())?)
                    .map_err(|_| ())?;
            let context = params.get("context").and_then(Value::as_object).ok_or(())?;
            only(context, &["includeDeclaration"])?;
            let include_declaration = context
                .get("includeDeclaration")
                .and_then(Value::as_bool)
                .ok_or(())?;
            Ok::<_, ()>(LspReferencesRequest {
                uri,
                position,
                include_declaration,
            })
        })();
        let request = match request {
            Ok(request) => request,
            Err(()) => {
                return response_error(output, Some(id), -32602, "invalid references parameters");
            }
        };
        match self.service.frontend_document_references(request, stop) {
            Ok(result) => {
                if result.locations.is_empty() {
                    send(
                        output,
                        json!({"jsonrpc":"2.0","id":id,"error":{"code":-32001,
                        "message":"no local reference locations observed under the selected scope","data":result}}),
                    )?;
                } else {
                    let mut locations = result.locations.iter().map(|location| json!({
                        "uri":location.uri,"range":location.range,"wow_binding":result.binding.id,
                    })).collect::<Vec<_>>();
                    // Carry the exact bounded result once, rather than once per location.
                    locations[0]["wow"] = serde_json::to_value(&result).map_err(|_| ())?;
                    send(output, json!({"jsonrpc":"2.0","id":id,"result":locations}))?;
                }
                Ok(Control::Continue)
            }
            Err(error) => domain_error(output, Some(id), error),
        }
    }
}
