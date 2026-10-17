use super::*;

pub(super) fn operation(
    uri: &Value,
    range: &Value,
    span: &Value,
    backend: &Value,
    binding: &Value,
    error: &Value,
) -> ServiceResult<LspOperationDescriptor> {
    let id = json!({"type":"string","maxLength":128});
    let title = json!({"type":"string","maxLength":1024,"x-maxUtf8Bytes":1024});
    let detail = json!({"type":["string","null"],"maxLength":2048,"x-maxUtf8Bytes":2048});
    let parent =
        json!({"type":["integer","null"],"minimum":0,"maximum":LSP_MAX_DOCUMENT_SYMBOLS-1});
    let native_symbol = object(
        json!({"name":title,"original_name":{"type":["string","null"],"maxLength":1024},
            "detail":detail,"kind":{"enum":["file","module","namespace","class","function",
                "variable","string","number","boolean","object","field","struct"]},
            "range":span,"selection_range":span,"parent_index":parent}),
        &["name", "kind", "range", "selection_range"],
    );
    let limits = object(
        json!({"max_file_bytes":{"type":"integer","minimum":1,"maximum":LSP_MAX_FRAME_BYTES/2},
            "max_work":{"type":"integer","minimum":1,"maximum":1024*1024*1024},
            "max_nodes":{"const":131072},"max_symbols":{"const":LSP_MAX_DOCUMENT_SYMBOLS},
            "max_depth":{"const":LSP_MAX_DOCUMENT_SYMBOL_DEPTH},"max_ast_depth":{"const":256},
            "max_name_bytes":{"const":1024},"max_detail_bytes":{"const":2048},
            "max_report_bytes":{"const":LSP_MAX_DOCUMENT_SYMBOL_RESULT_BYTES}}),
        &[
            "max_file_bytes",
            "max_work",
            "max_nodes",
            "max_symbols",
            "max_depth",
            "max_ast_depth",
            "max_name_bytes",
            "max_detail_bytes",
            "max_report_bytes",
        ],
    );
    let original = object(
        json!({"schema":{"const":"wow-emmy/document-symbols/1"},
            "profile":{"const":wow_emmy::document_symbols::DOCUMENT_SYMBOLS_PROFILE},
            "backend":backend,"workspace_snapshot_id":id,
            "path":{"type":"string","maxLength":32768},"content_sha256":id,"limits":limits,
            "state":{"enum":["ready","source_parse_failed","not_evaluated"]},
            "reason":{"type":["string","null"],"maxLength":128},
            "parse_error_count":{"type":"integer","minimum":0},
            "symbols":{"type":"array","maxItems":LSP_MAX_DOCUMENT_SYMBOLS,"items":native_symbol},
            "work_used":{"type":"integer","minimum":0,"maximum":1024*1024*1024},
            "digest":{"type":"string","pattern":"^emmy-document-symbols:sha256:[0-9a-f]{64}$"}}),
        &[
            "schema",
            "profile",
            "backend",
            "workspace_snapshot_id",
            "path",
            "content_sha256",
            "limits",
            "state",
            "parse_error_count",
            "symbols",
            "work_used",
            "digest",
        ],
    );
    let projected = object(
        json!({"name":title,"detail":detail,"kind":{"enum":[1,2,3,5,8,12,13,15,16,17,19,23]},
            "range":range,"selection_range":range,"parent_index":parent}),
        &["name", "kind", "range", "selection_range", "parent_index"],
    );
    let request = object(json!({"uri":uri}), &["uri"]);
    let result = object(
        json!({"schema":{"const":"wow-service/lsp-document-symbols-result/1"},"uri":uri,
            "hierarchical":{"type":"boolean"},"symbols":{"type":"array",
                "maxItems":LSP_MAX_DOCUMENT_SYMBOLS,"items":projected},
            "binding":binding,"original":original}),
        &[
            "schema",
            "uri",
            "hierarchical",
            "symbols",
            "binding",
            "original",
        ],
    );
    let mut descriptor = LspOperationDescriptor {
        wire_method: "textDocument/documentSymbol",
        operation_id: "frontend_document_symbols",
        request_schema_id: "wow-service/lsp/frontend_document_symbols/request/1".into(),
        request_schema_digest: canonical_digest("sha256:", &request)?,
        request_schema: request,
        result_schema_id: "wow-service/lsp/frontend_document_symbols/result/1".into(),
        result_schema_digest: canonical_digest("sha256:", &result)?,
        result_schema: result,
        error_schema_digest: canonical_digest("sha256:", error)?,
        error_schema_id: "wow-service/lsp-operation-error/1",
        error_schema: error.clone(),
        effect_class: "pure_read",
        wire_projection: None,
    };
    descriptor.wire_projection = Some(classic_projection(&descriptor)?);
    Ok(descriptor)
}

fn classic_projection(operation: &LspOperationDescriptor) -> ServiceResult<LspWireProjection> {
    let token = json!({"oneOf":[{"type":"integer","minimum":i32::MIN,"maximum":i32::MAX},
        {"type":"string","maxLength":128,"x-maxUtf8Bytes":128,"pattern":"^[^\\u0000-\\u001F\\u007F-\\u009F]*$"}]});
    let uri = operation.request_schema["properties"]["uri"].clone();
    let request_schema = object(
        json!({"textDocument":object(json!({"uri":uri}), &["uri"]),
            "workDoneToken":token,"partialResultToken":token}),
        &["textDocument"],
    );
    let result_schema = wire_result(operation, None);
    let error_data_schema = json!({"oneOf":[operation.error_schema,operation.result_schema]});
    let digest = canonical_digest(
        "lsp-wire-projection:sha256:",
        &(&request_schema, &result_schema, &error_data_schema),
    )?;
    Ok(LspWireProjection {
        schema: "wow-service/lsp-document-symbols-wire-projection/1",
        json_schema_dialect: "https://json-schema.org/draft/2020-12/schema",
        request_schema,
        result_schema,
        error_data_schema,
        digest,
    })
}

pub(super) fn wire_result(operation: &LspOperationDescriptor, ticket: Option<&Value>) -> Value {
    let position = object(
        json!({"line":{"type":"integer","minimum":0,"maximum":u32::MAX},
            "character":{"type":"integer","minimum":0,"maximum":u32::MAX}}),
        &["line", "character"],
    );
    let range = object(json!({"start":position,"end":position}), &["start", "end"]);
    let name = json!({"type":"string","minLength":1,"maxLength":1024,"pattern":"\\S"});
    let kind = json!({"type":"integer","minimum":1,"maximum":26});
    let uri = operation.request_schema["properties"]["uri"].clone();
    let marker = json!({"type":"string","maxLength":128});
    let index = json!({"type":"integer","minimum":0,"maximum":LSP_MAX_DOCUMENT_SYMBOLS-1});
    let mut hierarchical = object(
        json!({"name":name,"detail":{"type":"string","maxLength":2048},"kind":kind,
            "range":range,"selectionRange":range,"children":{"type":"array",
                "maxItems":LSP_MAX_DOCUMENT_SYMBOLS,"items":{"$ref":"#/$defs/documentSymbol"}},
            "wow_binding":marker,"wow_symbol_index":index,"wow":operation.result_schema}),
        &[
            "name",
            "kind",
            "range",
            "selectionRange",
            "wow_binding",
            "wow_symbol_index",
        ],
    );
    let mut flat = object(
        json!({"name":name,"kind":kind,"location":object(json!({"uri":uri,"range":range}),&["uri","range"]),
            "containerName":{"type":"string","maxLength":1024},"wow_binding":marker,
            "wow_symbol_index":index,"wow":operation.result_schema}),
        &[
            "name",
            "kind",
            "location",
            "wow_binding",
            "wow_symbol_index",
        ],
    );
    if let Some(ticket) = ticket {
        hierarchical["properties"]["wow_ticket"] = ticket.clone();
        flat["properties"]["wow_ticket"] = ticket.clone();
    }
    json!({"$defs":{"documentSymbol":hierarchical},
        "anyOf":[{"type":"array","maxItems":LSP_MAX_DOCUMENT_SYMBOLS,
            "items":{"$ref":"#/$defs/documentSymbol"}},
            {"type":"array","maxItems":LSP_MAX_DOCUMENT_SYMBOLS,"items":flat}],
        "x-maxTotalSymbols":LSP_MAX_DOCUMENT_SYMBOLS,"x-maxOutlineDepth":LSP_MAX_DOCUMENT_SYMBOL_DEPTH,
        "x-emptyResultMetadata":"top-level wow extension preserves exact qualified owner report"})
}
