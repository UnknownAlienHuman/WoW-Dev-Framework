//! Mechanical LSP layouts of explicit Service-owned outline records.
use super::*;
use wow_service::frontend::lsp::{
    LSP_MAX_DOCUMENT_SYMBOLS, LspDocumentSymbolClientCapabilities, LspDocumentSymbolsRequest,
    LspDocumentSymbolsResult, LspProgressToken,
};

impl Server {
    pub(super) fn document_symbols<W: Write>(
        &mut self,
        id: Option<LspRequestId>,
        params: &Map<String, Value>,
        output: &mut W,
        stop: &AtomicBool,
    ) -> Result<Control, ()> {
        let Some(id) = id else {
            return notification_error(output, "document symbols require a request ID");
        };
        let request = match parse_request(params) {
            Ok(request) => request,
            Err(()) => {
                return response_error(
                    output,
                    Some(id),
                    -32602,
                    "invalid document symbol parameters",
                );
            }
        };
        match self.service.frontend_document_symbols(request, stop) {
            Ok(result) => {
                send(output, result_response(&id, &result)?)?;
                Ok(Control::Continue)
            }
            Err(error) => domain_error(output, Some(id), error),
        }
    }
}

pub(super) fn client_capabilities(
    capabilities: &Map<String, Value>,
) -> Result<LspDocumentSymbolClientCapabilities, ()> {
    let document = capabilities
        .get("textDocument")
        .and_then(Value::as_object)
        .ok_or(())?;
    let Some(symbol) = document.get("documentSymbol") else {
        return Ok(LspDocumentSymbolClientCapabilities {
            hierarchical: false,
            extended_kinds: false,
        });
    };
    let symbol = symbol.as_object().ok_or(())?;
    let hierarchical = symbol
        .get("hierarchicalDocumentSymbolSupport")
        .map(|value| value.as_bool().ok_or(()))
        .transpose()?
        .unwrap_or(false);
    let extended_kinds = if let Some(kind) = symbol.get("symbolKind") {
        let kind = kind.as_object().ok_or(())?;
        if let Some(values) = kind.get("valueSet") {
            let values = values
                .as_array()
                .filter(|values| values.len() <= 26)
                .ok_or(())?;
            if values
                .iter()
                .any(|value| value.as_u64().is_none_or(|kind| !(1..=26).contains(&kind)))
            {
                return Err(());
            }
            true
        } else {
            false
        }
    } else {
        false
    };
    Ok(LspDocumentSymbolClientCapabilities {
        hierarchical,
        extended_kinds,
    })
}

pub(super) fn parse_request(params: &Map<String, Value>) -> Result<LspDocumentSymbolsRequest, ()> {
    only(
        params,
        &["textDocument", "workDoneToken", "partialResultToken"],
    )?;
    let document = params
        .get("textDocument")
        .and_then(Value::as_object)
        .ok_or(())?;
    only(document, &["uri"])?;
    for field in ["workDoneToken", "partialResultToken"] {
        if let Some(value) = params.get(field) {
            let token: LspProgressToken = serde_json::from_value(value.clone()).map_err(|_| ())?;
            if let LspProgressToken::String(value) = token
                && (value.len() > 128 || value.chars().any(char::is_control))
            {
                return Err(());
            }
        }
    }
    Ok(LspDocumentSymbolsRequest {
        uri: string(document, "uri", 65536)?,
    })
}

pub(super) fn result_response(
    id: &LspRequestId,
    result: &LspDocumentSymbolsResult,
) -> Result<Value, ()> {
    if !result.projection_available() {
        return Ok(json!({"jsonrpc":"2.0","id":id,"error":{"code":-32001,
            "message":"native document outline not evaluated","data":result}}));
    }
    let count = result.symbols.len();
    if count > LSP_MAX_DOCUMENT_SYMBOLS {
        return Err(());
    }
    let mut values = Vec::with_capacity(count);
    let mut children = Vec::with_capacity(count);
    children.resize_with(count, Vec::<usize>::new);
    let mut roots = Vec::new();
    for (index, symbol) in result.symbols.iter().enumerate() {
        if let Some(parent) = symbol.parent_index {
            if parent >= index {
                return Err(());
            }
            children[parent].push(index);
        } else {
            roots.push(index);
        }
        let mut value = if result.hierarchical {
            json!({"name":symbol.name,"kind":symbol.kind,"range":symbol.range,
                "selectionRange":symbol.selection_range,"wow_binding":result.binding.id,
                "wow_symbol_index":index})
        } else {
            json!({"name":symbol.name,"kind":symbol.kind,
                "location":{"uri":result.uri,"range":symbol.range},
                "wow_binding":result.binding.id,"wow_symbol_index":index})
        };
        if result.hierarchical {
            if let Some(detail) = &symbol.detail {
                value["detail"] = json!(detail);
            }
        } else if let Some(parent) = symbol.parent_index {
            // Exact explicit owner parent label; containerName is a UI field.
            value["containerName"] = json!(result.symbols[parent].name);
        }
        values.push(Some(value));
    }
    let mut output = Vec::with_capacity(if result.hierarchical {
        roots.len()
    } else {
        count
    });
    if result.hierarchical {
        // Flat owner preorder makes children available in reverse order. This
        // conversion follows explicit parent indices and does not infer a tree.
        for index in (0..count).rev() {
            let mut nested = Vec::with_capacity(children[index].len());
            for child in &children[index] {
                nested.push(values[*child].take().ok_or(())?);
            }
            if !nested.is_empty() {
                values[index].as_mut().ok_or(())?["children"] = Value::Array(nested);
            }
        }
        for root in roots {
            output.push(values[root].take().ok_or(())?);
        }
    } else {
        for value in values {
            output.push(value.ok_or(())?);
        }
    }
    if let Some(first) = output.first_mut() {
        first["wow"] = serde_json::to_value(result).map_err(|_| ())?;
        Ok(json!({"jsonrpc":"2.0","id":id,"result":output}))
    } else {
        // A genuinely qualified empty outline must retain the same exact
        // owner/binding evidence despite having no first symbol for extension.
        Ok(json!({"jsonrpc":"2.0","id":id,"result":[],"wow":result}))
    }
}
