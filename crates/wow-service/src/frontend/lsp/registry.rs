use super::*;
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize)]
pub struct LspOperationDescriptor {
    pub wire_method: &'static str,
    pub operation_id: &'static str,
    pub request_schema_id: Box<str>,
    pub request_schema_digest: Box<str>,
    pub request_schema: Value,
    pub result_schema_id: Box<str>,
    pub result_schema_digest: Box<str>,
    pub result_schema: Value,
    pub error_schema_digest: Box<str>,
    pub error_schema_id: &'static str,
    pub error_schema: Value,
    pub effect_class: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wire_projection: Option<LspWireProjection>,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspWireProjection {
    pub schema: &'static str,
    pub json_schema_dialect: &'static str,
    pub request_schema: Value,
    pub result_schema: Value,
    pub error_data_schema: Value,
    pub digest: Box<str>,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspTransportProfile {
    pub schema: &'static str,
    pub protocol: &'static str,
    pub profile_id: &'static str,
    pub transport: &'static str,
    pub synchronization: &'static str,
    pub cancellation: &'static str,
    pub privacy: &'static str,
    pub local_definition_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_references_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_local_references: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_uri_profile: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_rebinding_policy: Option<&'static str>,
    pub max_header_bytes: usize,
    pub max_frame_bytes: usize,
    pub max_output_bytes: usize,
    pub max_json_depth: usize,
    pub max_active_requests: usize,
    pub max_queued_requests: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_queued_request_bytes: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_metadata_journal_entries: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreground_execution: Option<LspForegroundExecutionProfile>,
    pub max_diagnostics: usize,
    pub max_retained_report_bytes: usize,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspRegistry {
    pub schema: &'static str,
    pub registry_id: &'static str,
    pub profile_id: &'static str,
    pub digest: Box<str>,
    pub operations: Vec<LspOperationDescriptor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_rebinding_policy: Option<&'static str>,
    pub implementation_pin: &'static str,
    pub profile: LspTransportProfile,
    pub profile_digest: Box<str>,
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
fn foreground_ticket_schema() -> Value {
    let string = json!({"type":"string","maxLength":128});
    let token = json!({"oneOf":[{"type":"integer","minimum":i32::MIN,"maximum":i32::MAX},
        {"type":"string","maxLength":128,"x-maxUtf8Bytes":128,"pattern":"^[^\\u0000-\\u001F\\u007F-\\u009F]*$"}]});
    let overlay = object(
        json!({"document_version":{"type":"integer","minimum":0,"maximum":i32::MAX},
        "snapshot_id":string,"content_digest":string,"digest":string}),
        &[
            "document_version",
            "snapshot_id",
            "content_digest",
            "digest",
        ],
    );
    object(
        json!({
            "schema":{"const":"wow-service/lsp-foreground-ticket/1"},
            "ticket_id":string,"wire_id":token,"service_request_id":string,
            "nonce":{"type":"integer","minimum":1,"maximum":9_007_199_254_740_991_u64},
            "session_id":string,"profile_id":{"const":LSP_OWNED_PROFILE_ID},
            "registry_digest":string,"request_digest":string,
            "operation":{"enum":["diagnostics","definition","references"]},
            "progress_token":{"oneOf":[{"type":"null"},token]},
            "progress_events_coalesced":{"type":"integer","minimum":0,"maximum":2},
            "overlay":{"oneOf":[{"type":"null"},overlay]},
            "state":{"enum":["queued","dispatched","completed","failed","failed_before_owner",
                "refused_not_ready","refused_session_closed","cancelled_before_dispatch","owner_acknowledged",
                "completed_before_cancel","connection_lost_abandoned","process_stopped","dropped_before_dispatch",
                "dropped_before_owner","owner_outcome_unknown"]},
            "cancellation":{"enum":["not_requested","requested","forwarded_to_owner","owner_acknowledged",
                "completed_before_cancel","cancelled_before_dispatch"]},
            "cancel_origin":{"enum":[null,"client_request","request_deadline"]},
            "stop_reason":{"enum":[null,"process_stop","connection_lost"]},
            "delivery":{"enum":["prepared","sent","lost_before","lost_after_possible"]},
            "result_digest":{"type":["string","null"],"maxLength":128},"digest":string,
        }),
        &[
            "schema",
            "ticket_id",
            "wire_id",
            "service_request_id",
            "nonce",
            "session_id",
            "profile_id",
            "registry_digest",
            "request_digest",
            "operation",
            "progress_token",
            "progress_events_coalesced",
            "overlay",
            "state",
            "cancellation",
            "cancel_origin",
            "stop_reason",
            "delivery",
            "result_digest",
            "digest",
        ],
    )
}
fn owned_wire_projection(operation: &LspOperationDescriptor) -> ServiceResult<LspWireProjection> {
    let ticket = foreground_ticket_schema();
    let token = json!({"oneOf":[{"type":"integer","minimum":i32::MIN,"maximum":i32::MAX},
        {"type":"string","maxLength":128,"x-maxUtf8Bytes":128,"pattern":"^[^\\u0000-\\u001F\\u007F-\\u009F]*$"}]});
    let uri = operation.request_schema["properties"]["uri"].clone();
    if uri.is_null() {
        return Err(invalid_frontend_request(
            "owned read URI schema unavailable",
        ));
    }
    let mut properties = json!({"textDocument":object(json!({"uri":uri}), &["uri"]),
        "workDoneToken":token,"partialResultToken":token});
    let mut required = vec!["textDocument"];
    if operation.wire_method == "textDocument/diagnostic" {
        properties["identifier"] = json!({"const":"wow.private-syntax"});
        properties["previousResultId"] = json!({"type":"string","maxLength":256});
    } else {
        properties["position"] = operation.request_schema["properties"]["position"].clone();
        required.push("position");
        if operation.wire_method == "textDocument/references" {
            properties["context"] = object(
                json!({"includeDeclaration":{"type":"boolean"}}),
                &["includeDeclaration"],
            );
            required.push("context");
        }
    }
    let request_schema = object(properties, &required);
    let result_schema = match operation.wire_method {
        "textDocument/diagnostic" => {
            let mut schema = operation.result_schema.clone();
            let variants = schema["oneOf"]
                .as_array_mut()
                .ok_or_else(|| invalid_frontend_request("owned diagnostic variants unavailable"))?;
            for variant in variants {
                variant["properties"]["wow_ticket"] = ticket.clone();
                variant["required"]
                    .as_array_mut()
                    .ok_or_else(|| invalid_frontend_request("owned diagnostic fields unavailable"))?
                    .push(json!("wow_ticket"));
            }
            schema
        }
        "textDocument/definition" => {
            let range =
                operation.result_schema["properties"]["location"]["properties"]["range"].clone();
            if range.is_null() {
                return Err(invalid_frontend_request(
                    "owned definition range unavailable",
                ));
            }
            let item = object(
                json!({"uri":uri,"range":range,"wow":operation.result_schema,"wow_ticket":ticket}),
                &["uri", "range", "wow", "wow_ticket"],
            );
            json!({"type":"array","minItems":1,"maxItems":1,"items":item})
        }
        "textDocument/references" => {
            let range =
                operation.result_schema["properties"]["locations"]["items"]["properties"]["range"]
                    .clone();
            if range.is_null() {
                return Err(invalid_frontend_request(
                    "owned reference range unavailable",
                ));
            }
            let first = object(
                json!({"uri":uri,"range":range,"wow_binding":{"type":"string"},
                "wow":operation.result_schema,"wow_ticket":ticket}),
                &["uri", "range", "wow_binding", "wow", "wow_ticket"],
            );
            let rest = object(
                json!({"uri":uri,"range":range,"wow_binding":{"type":"string"}}),
                &["uri", "range", "wow_binding"],
            );
            json!({"type":"array","minItems":1,"maxItems":LSP_MAX_LOCAL_REFERENCES,
                "prefixItems":[first],"items":rest})
        }
        _ => {
            return Err(invalid_frontend_request(
                "owned wire projection is not a registered read",
            ));
        }
    };
    let envelope = object(
        json!({"ticket":ticket,
        "owner_result":{"oneOf":[operation.result_schema,operation.error_schema]}}),
        &["ticket", "owner_result"],
    );
    let error_data_schema = json!({"oneOf":[operation.error_schema,envelope]});
    let digest = canonical_digest(
        "lsp-wire-projection:sha256:",
        &(&request_schema, &result_schema, &error_data_schema),
    )?;
    Ok(LspWireProjection {
        schema: "wow-service/lsp-owned-wire-projection/1",
        json_schema_dialect: "https://json-schema.org/draft/2020-12/schema",
        request_schema,
        result_schema,
        error_data_schema,
        digest,
    })
}
fn references_descriptor(
    uri: &Value,
    position: &Value,
    range: &Value,
    span: &Value,
    backend: &Value,
    binding: &Value,
    error: &Value,
) -> ServiceResult<LspOperationDescriptor> {
    let string = json!({"type":"string"});
    let limits = object(
        json!({
            "max_file_bytes":{"type":"integer"},"max_work":{"type":"integer"},
            "max_name_bytes":{"type":"integer"},"max_references":{"type":"integer"},
        }),
        &[
            "max_file_bytes",
            "max_work",
            "max_name_bytes",
            "max_references",
        ],
    );
    let original = object(
        json!({
            "schema":{"const":"wow-emmy/local-references-result/1"},
            "profile":{"const":"same_file_lexical_local_references_no_trace/1"},
            "analysis_id":string,"workspace_snapshot_id":string,"backend":backend,
            "path":string,"content_sha256":string,"query_byte_offset":{"type":"integer","minimum":0},
            "include_declaration":{"type":"boolean"},"limits":limits,
            "state":{"enum":["resolved_local","not_evaluated","source_parse_failed"]},
        "reason":{"enum":["unsupported_token","declaration_not_observed","outside_local_scope","target_unavailable","reference_index_unavailable"]},
        "origin_span":span,"declaration_span":span,
            "references":{"type":"array","maxItems":LSP_MAX_LOCAL_REFERENCES,"items":span},
        "negative_authority":{"const":false},
        "coverage":{"const":"same_file_native_local_reference_index_observed_subset"},
        }),
        &[
            "schema",
            "profile",
            "analysis_id",
            "workspace_snapshot_id",
            "backend",
            "path",
            "content_sha256",
            "query_byte_offset",
            "include_declaration",
            "limits",
            "state",
            "references",
            "negative_authority",
            "coverage",
        ],
    );
    let request = object(
        json!({"uri":uri,"position":position,"include_declaration":{"type":"boolean"}}),
        &["uri", "position", "include_declaration"],
    );
    let location = object(json!({"uri":uri,"range":range}), &["uri", "range"]);
    let result = object(
        json!({"schema":{"const":"wow-service/lsp-local-references-result/1"},
        "locations":{"type":"array","maxItems":LSP_MAX_LOCAL_REFERENCES,"items":location},
        "binding":binding,"original":original}),
        &["schema", "locations", "binding", "original"],
    );
    Ok(LspOperationDescriptor {
        wire_method: "textDocument/references",
        operation_id: "frontend_document_references",
        request_schema_id: "wow-service/lsp/frontend_document_references/request/1".into(),
        request_schema_digest: canonical_digest("sha256:", &request)?,
        request_schema: request,
        result_schema_id: "wow-service/lsp/frontend_document_references/result/1".into(),
        result_schema_digest: canonical_digest("sha256:", &result)?,
        result_schema: result,
        error_schema_id: "wow-service/lsp-operation-error/1",
        error_schema_digest: canonical_digest("sha256:", error)?,
        error_schema: error.clone(),
        effect_class: "pure_read",
        wire_projection: None,
    })
}
impl LspRegistry {
    pub(super) fn build(
        local_definition: bool,
        uri_policy: OverlayUriPolicy,
        local_references: bool,
        workspace_rebind_enabled: bool,
    ) -> ServiceResult<Self> {
        let client_uris = uri_policy != OverlayUriPolicy::StrictCanonical;
        let uri = json!({"type":"string","maxLength":65536});
        let version = json!({"type":"integer","minimum":0,"maximum":i32::MAX});
        let string = json!({"type":"string"});
        let position = object(
            json!({"line":{"type":"integer","minimum":0,"maximum":u32::MAX},
            "character":{"type":"integer","minimum":0,"maximum":u32::MAX}}),
            &["line", "character"],
        );
        let range = object(json!({"start":position,"end":position}), &["start", "end"]);
        let edit = object(json!({"range":range,"text":string}), &["text"]);
        let limits = object(
            json!({"max_documents":{"type":"integer"},"max_document_bytes":{"type":"integer"},
            "max_private_bytes":{"type":"integer"},"max_live_snapshots":{"type":"integer"},"max_leases":{"type":"integer"},
            "max_lines":{"type":"integer"},"max_edits":{"type":"integer"},"max_work":{"type":"integer"}}),
            &[
                "max_documents",
                "max_document_bytes",
                "max_private_bytes",
                "max_live_snapshots",
                "max_leases",
                "max_lines",
                "max_edits",
                "max_work",
            ],
        );
        let workspace = object(
            json!({"schema":{"const":"wow-project/private-workspace/1"},"registration_id":string,
            "session_scope":{"type":"integer"},"workspace_id":string,"state":string,"path_profile":string,
            "base_state":{"const":"no_published_base"},"privacy":{"const":"session_private_memory_only"},"limits":limits}),
            &[
                "schema",
                "registration_id",
                "session_scope",
                "workspace_id",
                "state",
                "path_profile",
                "base_state",
                "privacy",
                "limits",
            ],
        );
        let snapshot = object(
            json!({"schema":{"const":"wow-project/private-document-overlay/1"},"snapshot_id":string,
            "registration_id":string,"document_id":string,"relative_path":string,"client_version":version,
            "content_digest":string,"byte_length":{"type":"integer"},"line_count":{"type":"integer"},
            "base_state":{"const":"no_published_base"},"synchronization_profile":string}),
            &[
                "schema",
                "snapshot_id",
                "registration_id",
                "document_id",
                "relative_path",
                "client_version",
                "content_digest",
                "byte_length",
                "line_count",
                "base_state",
                "synchronization_profile",
            ],
        );
        let client = object(
            json!({"name":string,"version":string}),
            &["name", "version"],
        );
        let not_evaluated = json!({"type":"array","items":{"type":"string"}});
        let mut session = object(
            json!({"schema":{"const":"wow-service/lsp-private-session/1"},"session_id":string,
            "profile_id":{"const":LSP_PROFILE_ID},"registry_digest":string,"profile_digest":string,"local_definition_enabled":{"type":"boolean"},"client":client,
            "position_encoding":{"enum":["utf-8","utf-16"]},"workspace":workspace,"coverage":string,
            "cancellation":string,"not_evaluated":not_evaluated}),
            &[
                "schema",
                "session_id",
                "profile_id",
                "registry_digest",
                "profile_digest",
                "local_definition_enabled",
                "client",
                "position_encoding",
                "workspace",
                "coverage",
                "cancellation",
                "not_evaluated",
            ],
        );
        if client_uris {
            session["properties"]["schema"] = json!({"const":"wow-service/lsp-private-session/2"});
            session["properties"]["document_uri_profile"] = json!({"const":uri_policy.id()});
            session["required"]
                .as_array_mut()
                .ok_or_else(|| invalid_frontend_request("LSP session schema invalid"))?
                .push(json!("document_uri_profile"));
        }
        if local_references {
            session["properties"]["schema"] = json!({"const":"wow-service/lsp-private-session/3"});
            session["properties"]["local_references_enabled"] = json!({"const":true});
            session["required"]
                .as_array_mut()
                .ok_or_else(|| invalid_frontend_request("LSP session schema invalid"))?
                .push(json!("local_references_enabled"));
        }
        if workspace_rebind_enabled {
            session["properties"]["schema"] = json!({"const":"wow-service/lsp-private-session/4"});
            session["properties"]["workspace_folders"] = object(
                json!({"supported":{"const":true},"change_notifications":{"const":true}}),
                &["supported", "change_notifications"],
            );
            session["required"]
                .as_array_mut()
                .ok_or_else(|| invalid_frontend_request("LSP session schema invalid"))?
                .push(json!("workspace_folders"));
        }
        let save = object(
            json!({"snapshot_id":string,"client_version":version,
            "state":{"enum":["content_acknowledged_only","disk_reconciled","conflict_with_disk"]}}),
            &["snapshot_id", "client_version", "state"],
        );
        let backend = object(
            json!({"crate_name":string,"crate_version":{"type":["string","null"]},
            "revision":string,"tree":string,"surface_sha256":string,"compatibility_report_sha256":string}),
            &[
                "crate_name",
                "crate_version",
                "revision",
                "tree",
                "surface_sha256",
                "compatibility_report_sha256",
            ],
        );
        let binding = object(
            json!({"schema":{"const":"wow-service/lsp-overlay-binding/1"},"id":string,
            "session_id":string,"registry_digest":string,"snapshot":snapshot,"position_encoding":{"enum":["utf-8","utf-16"]},
            "coverage":string,"analyzer_backend":backend,"analyzer_workspace_snapshot_id":string,"analyzer_analysis_id":string,
            "resource_closure":{"const":"operation_leases_closed_session_retained"},"save":save,"not_evaluated":not_evaluated}),
            &[
                "schema",
                "id",
                "session_id",
                "registry_digest",
                "snapshot",
                "analyzer_backend",
                "analyzer_workspace_snapshot_id",
                "analyzer_analysis_id",
                "position_encoding",
                "coverage",
                "resource_closure",
                "not_evaluated",
            ],
        );
        // This consumer admits only byte-range spans and preserves all fields.
        let span = object(
            json!({"kind":{"const":"byte_range"},
            "byte_start":{"type":"integer","minimum":0,"maximum":9_007_199_254_740_991_u64},
            "byte_end":{"type":"integer","minimum":0,"maximum":9_007_199_254_740_991_u64}}),
            &["kind", "byte_start", "byte_end"],
        );
        let severity = json!({"enum":["error","warning","information","hint","unknown"]});
        let original = object(
            json!({"category":string,"upstream_code":string,
            "kind":{"enum":["lua_syntax","documentation_syntax","assignment_type_mismatch"]},
            "upstream_severity":severity,"normalized_severity":severity,"classification":{"const":"accepted"},
            "rollout":{"const":"advisory"},"path":string,"content_sha256":string,"span":span}),
            &[
                "category",
                "upstream_code",
                "kind",
                "upstream_severity",
                "normalized_severity",
                "classification",
                "rollout",
                "path",
                "content_sha256",
                "span",
            ],
        );
        let data = object(
            json!({"binding_id":string,"original":original}),
            &["binding_id", "original"],
        );
        let diagnostic = object(
            json!({"range":range,"severity":{"type":"integer","minimum":1,"maximum":4},
            "code":string,"source":{"const":"wow-emmy/private-syntax"},"message":string,"data":data}),
            &["range", "severity", "code", "source", "message", "data"],
        );
        let full = object(
            json!({"kind":{"const":"full"},"resultId":string,
            "items":{"type":"array","maxItems":2048,"items":diagnostic},"wow":binding}),
            &["kind", "resultId", "items", "wow"],
        );
        let unchanged = object(
            json!({"kind":{"const":"unchanged"},"resultId":string,"wow":binding}),
            &["kind", "resultId", "wow"],
        );
        let report = json!({"oneOf":[full,unchanged]});
        let apply = object(
            json!({"state":{"const":"apply"},"version":version,
            "changes":{"type":"array","minItems":1,"maxItems":4096,"items":edit}}),
            &["state", "version", "changes"],
        );
        let rejected = object(
            json!({"state":{"const":"rejected"},"reason":{"enum":["invalid_version","unsupported_change_shape"]}}),
            &["state", "reason"],
        );
        let change_request = object(
            json!({"uri":uri,"change":{"oneOf":[apply,rejected]}}),
            &["uri", "change"],
        );
        let id = json!({"oneOf":[{"type":"integer","minimum":i32::MIN,"maximum":i32::MAX},{"type":"string","maxLength":128}]});
        let error = object(
            json!({"code":string,"message":string,"operation_id":{"type":["string","null"]}}),
            &["code", "message"],
        );
        let request_uri = object(json!({"uri":uri}), &["uri"]);
        let empty_result = json!({"type":"null"});
        let definitions = [
            (
                "initialize",
                "frontend_session_open",
                object(
                    json!({"client_name":string,"client_version":string,
                "position_encodings":{"type":"array","maxItems":8,"items":{"type":"string","maxLength":32}},
                "pull_diagnostics":{"const":true},"workspace_uris":{"type":"array","maxItems":1,"items":uri}}),
                    &[
                        "client_name",
                        "client_version",
                        "position_encodings",
                        "pull_diagnostics",
                        "workspace_uris",
                    ],
                ),
                session,
                "session_lifecycle",
            ),
            (
                "shutdown",
                "frontend_session_close",
                object(json!({}), &[]),
                empty_result.clone(),
                "session_lifecycle",
            ),
            (
                "textDocument/didOpen",
                "frontend_document_open",
                object(
                    json!({"uri":uri,"language_id":{"const":"lua"},"version":version,"content":string}),
                    &["uri", "language_id", "version", "content"],
                ),
                snapshot.clone(),
                "session_local_mutation",
            ),
            (
                "textDocument/didChange",
                "frontend_document_change",
                change_request,
                snapshot,
                "session_local_mutation",
            ),
            (
                "textDocument/didSave",
                "frontend_document_save",
                object(json!({"uri":uri,"content":string}), &["uri"]),
                save,
                "session_local_mutation",
            ),
            (
                "textDocument/didClose",
                "frontend_document_close",
                request_uri,
                empty_result,
                "session_local_mutation",
            ),
            (
                "textDocument/diagnostic",
                "frontend_document_diagnostics",
                object(
                    json!({"uri":uri,"previous_result_id":{"type":"string","maxLength":256},"ticket":id}),
                    &["uri", "ticket"],
                ),
                report,
                "pure_read",
            ),
            (
                "$/cancelRequest",
                "frontend_operation_cancel",
                id.clone(),
                object(
                    json!({"target":id,
                "state":{"enum":["completed_before_cancel","not_observed_no_cancellation_claim"]}}),
                    &["target", "state"],
                ),
                "session_lifecycle",
            ),
        ];
        let mut operations = Vec::new();
        for (wire, operation, request, result, effect) in definitions {
            operations.push(LspOperationDescriptor {
                wire_method: wire,
                operation_id: operation,
                request_schema_id: format!("wow-service/lsp/{operation}/request/1").into(),
                request_schema_digest: canonical_digest("sha256:", &request)?,
                request_schema: request,
                result_schema_id: format!(
                    "wow-service/lsp/{operation}/result/{}",
                    if workspace_rebind_enabled && operation == "frontend_session_open" {
                        4
                    } else if local_references && operation == "frontend_session_open" {
                        3
                    } else if client_uris && operation == "frontend_session_open" {
                        2
                    } else {
                        1
                    }
                )
                .into(),
                result_schema_digest: canonical_digest("sha256:", &result)?,
                result_schema: result,
                error_schema_digest: canonical_digest("sha256:", &error)?,
                error_schema_id: "wow-service/lsp-operation-error/1",
                error_schema: error.clone(),
                effect_class: effect,
                wire_projection: None,
            });
        }
        if local_definition {
            let target = object(
                json!({"name":{"type":"string","maxLength":256},
                "kind":{"enum":["local_binding","parameter"]},"span":span}),
                &["name", "kind", "span"],
            );
            let limits = object(
                json!({"max_file_bytes":{"type":"integer"},"max_work":{"type":"integer"},"max_name_bytes":{"type":"integer"}}),
                &["max_file_bytes", "max_work", "max_name_bytes"],
            );
            let original = object(
                json!({"schema":{"const":"wow-emmy/local-definition-result/1"},
                "profile":{"const":"same_file_lexical_local_no_trace/1"},"analysis_id":string,"workspace_snapshot_id":string,"backend":backend,
                "path":string,"content_sha256":string,"query_byte_offset":{"type":"integer","minimum":0},"limits":limits,
                "state":{"enum":["resolved_local","not_evaluated","source_parse_failed"]},
                "reason":{"enum":["unsupported_token","declaration_not_observed","outside_local_scope","target_unavailable"]},
                "origin_span":span,"target":target,"negative_authority":{"const":false},"coverage":{"const":"one_exact_main_file_lexical_locals_only"}}),
                &[
                    "schema",
                    "profile",
                    "analysis_id",
                    "workspace_snapshot_id",
                    "backend",
                    "path",
                    "content_sha256",
                    "query_byte_offset",
                    "limits",
                    "state",
                    "negative_authority",
                    "coverage",
                ],
            );
            let request = object(json!({"uri":uri,"position":position}), &["uri", "position"]);
            let location = object(
                json!({"uri":uri,"range":range,"origin_range":range}),
                &["uri", "range", "origin_range"],
            );
            let result = object(
                json!({"schema":{"const":"wow-service/lsp-local-definition-result/1"},"location":location,"binding":binding,"original":original}),
                &["schema", "binding", "original"],
            );
            operations.push(LspOperationDescriptor {
                wire_method: "textDocument/definition",
                operation_id: "frontend_document_definition",
                request_schema_id: "wow-service/lsp/frontend_document_definition/request/1".into(),
                request_schema_digest: canonical_digest("sha256:", &request)?,
                request_schema: request,
                result_schema_id: "wow-service/lsp/frontend_document_definition/result/1".into(),
                result_schema_digest: canonical_digest("sha256:", &result)?,
                result_schema: result,
                error_schema_id: "wow-service/lsp-operation-error/1",
                error_schema_digest: canonical_digest("sha256:", &error)?,
                error_schema: error.clone(),
                effect_class: "pure_read",
                wire_projection: None,
            });
        }
        if local_references {
            operations.push(references_descriptor(
                &uri, &position, &range, &span, &backend, &binding, &error,
            )?);
        }
        if workspace_rebind_enabled {
            let folder_uri = json!({"type":"string","minLength":1,"maxLength":65536});
            let request = object(
                json!({"removed_uri":folder_uri,"added_uri":folder_uri}),
                &["removed_uri", "added_uri"],
            );
            let result = object(
                json!({
                    "schema":{"const":"wow-service/lsp-workspace-rebind-result/1"},
                    "previous_registration_id":{"type":"string","minLength":1,"maxLength":512},
                    "workspace":workspace.clone(),
                    "workspace_digest":{"type":"string","pattern":"^lsp-workspace-descriptor:sha256:[0-9a-f]{64}$"},
                    "transition_digest":{"type":"string","pattern":"^lsp-workspace-transition:sha256:[0-9a-f]{64}$"}
                }),
                &[
                    "schema",
                    "previous_registration_id",
                    "workspace",
                    "workspace_digest",
                    "transition_digest",
                ],
            );
            let rebind_error = object(
                json!({
                    "code":{"enum":[
                        "invalid_request","invalid_context","identity_mismatch","operation_busy",
                        "component_unavailable","budget_exceeded","cancelled",
                        "canonicalization_failed","internal_contract_violation"
                    ]},
                    "message":{"type":"string","maxLength":1024},
                    "operation_id":{"type":["string","null"],"maxLength":512}
                }),
                &["code", "message"],
            );
            operations.push(LspOperationDescriptor {
                wire_method: "workspace/didChangeWorkspaceFolders",
                operation_id: "frontend_workspace_folders_change",
                request_schema_id: "wow-service/lsp/frontend_workspace_folders_change/request/1"
                    .into(),
                request_schema_digest: canonical_digest("sha256:", &request)?,
                request_schema: request,
                result_schema_id: "wow-service/lsp/frontend_workspace_folders_change/result/1"
                    .into(),
                result_schema_digest: canonical_digest("sha256:", &result)?,
                result_schema: result,
                error_schema_digest: canonical_digest("sha256:", &rebind_error)?,
                error_schema_id: "wow-service/lsp-workspace-rebind-error/1",
                error_schema: rebind_error,
                effect_class: "session_local_mutation",
                wire_projection: None,
            });
        }
        operations.sort_by_key(|entry| entry.wire_method);
        let digest = if workspace_rebind_enabled {
            canonical_digest(
                "frontend-lsp-registry:sha256:",
                &(
                    &operations,
                    wow_emmy::EMMYLUA_REVISION,
                    LSP_PROFILE_ID,
                    LSP_WORKSPACE_REBIND_POLICY,
                ),
            )?
        } else {
            canonical_digest(
                "frontend-lsp-registry:sha256:",
                &(&operations, wow_emmy::EMMYLUA_REVISION, LSP_PROFILE_ID),
            )?
        };
        let profile = LspTransportProfile {
            schema: if workspace_rebind_enabled {
                "wow-service/lsp-transport-profile/4"
            } else if local_references {
                "wow-service/lsp-transport-profile/3"
            } else if client_uris {
                "wow-service/lsp-transport-profile/2"
            } else {
                "wow-service/lsp-transport-profile/1"
            },
            protocol: "3.18",
            profile_id: LSP_PROFILE_ID,
            transport: "stdio",
            synchronization: "sequential_intermediate_exact_next_version",
            cancellation: "synchronous_observations_process_stop_owner_checkpoints",
            privacy: if local_definition {
                "session_private_metadata_diagnostics_and_same_document_locations"
            } else {
                "session_private_metadata_and_diagnostics"
            },
            local_definition_enabled: local_definition,
            local_references_enabled: local_references.then_some(true),
            max_local_references: local_references.then_some(LSP_MAX_LOCAL_REFERENCES),
            document_uri_profile: if client_uris {
                Some(uri_policy.id())
            } else {
                None
            },
            workspace_rebinding_policy: workspace_rebind_enabled
                .then_some(LSP_WORKSPACE_REBIND_POLICY),
            max_header_bytes: LSP_MAX_HEADER_BYTES,
            max_frame_bytes: LSP_MAX_FRAME_BYTES,
            max_output_bytes: LSP_MAX_OUTPUT_BYTES,
            max_json_depth: LSP_MAX_JSON_DEPTH,
            max_active_requests: 1,
            max_queued_requests: 0,
            max_queued_request_bytes: None,
            max_metadata_journal_entries: None,
            foreground_execution: None,
            max_diagnostics: LSP_MAX_DIAGNOSTICS,
            max_retained_report_bytes: LSP_MAX_RETAINED_REPORT_BYTES,
        };
        let profile_digest =
            canonical_digest("frontend-lsp-profile:sha256:", &(&profile, &digest))?;
        Ok(Self {
            schema: if workspace_rebind_enabled {
                "wow-service/lsp-operation-registry/2"
            } else {
                "wow-service/lsp-operation-registry/1"
            },
            registry_id: super::super::FRONTEND_REGISTRY_ID,
            profile_id: LSP_PROFILE_ID,
            digest,
            operations,
            workspace_rebinding_policy: workspace_rebind_enabled
                .then_some(LSP_WORKSPACE_REBIND_POLICY),
            implementation_pin: wow_emmy::EMMYLUA_REVISION,
            profile,
            profile_digest,
        })
    }

    pub(super) fn build_owned(
        local_definition: bool,
        uri_policy: OverlayUriPolicy,
        local_references: bool,
        workspace_rebind_enabled: bool,
    ) -> ServiceResult<Self> {
        let mut registry = Self::build(
            local_definition,
            uri_policy,
            local_references,
            workspace_rebind_enabled,
        )?;
        registry.registry_id = "wow-service/e7-a/lsp-owned-foreground-registry/1";
        registry.profile_id = LSP_OWNED_PROFILE_ID;
        registry.profile.schema = if workspace_rebind_enabled {
            "wow-service/lsp-transport-profile/owned/2"
        } else {
            "wow-service/lsp-transport-profile/owned/1"
        };
        registry.profile.profile_id = LSP_OWNED_PROFILE_ID;
        registry.profile.cancellation = "per_ticket_cooperative_owner_checkpoints";
        registry.profile.max_queued_requests = LSP_FOREGROUND_MAX_QUEUED;
        registry.profile.max_queued_request_bytes = Some(LSP_FOREGROUND_MAX_QUEUE_BYTES);
        registry.profile.max_metadata_journal_entries = Some(LSP_FOREGROUND_MAX_METADATA);
        registry.profile.foreground_execution = Some(LSP_FOREGROUND_EXECUTION);

        for operation in &mut registry.operations {
            if matches!(
                operation.wire_method,
                "textDocument/diagnostic" | "textDocument/definition" | "textDocument/references"
            ) {
                operation.wire_projection = Some(owned_wire_projection(operation)?);
            }
            if operation.operation_id == "frontend_session_open" {
                operation.result_schema["properties"]["schema"] = json!({"const": if workspace_rebind_enabled {
                    "wow-service/lsp-private-foreground-session/2"
                } else {
                    "wow-service/lsp-private-foreground-session/1"
                }});
                operation.result_schema["properties"]["profile_id"] =
                    json!({"const":LSP_OWNED_PROFILE_ID});
                operation.result_schema_id = if workspace_rebind_enabled {
                    "wow-service/lsp/frontend_session_open/result/owned-2".into()
                } else {
                    "wow-service/lsp/frontend_session_open/result/owned-1".into()
                };
                operation.result_schema_digest =
                    canonical_digest("sha256:", &operation.result_schema)?;
            } else if operation.operation_id == "frontend_operation_cancel" {
                operation.result_schema = object(
                    json!({"target":{"oneOf":[{"type":"integer"},{"type":"string","maxLength":128}]},
                    "state":{"enum":["not_requested","requested","forwarded_to_owner","owner_acknowledged",
                    "completed_before_cancel","cancelled_before_dispatch"]}}),
                    &["target", "state"],
                );
                operation.result_schema_id =
                    "wow-service/lsp/frontend_operation_cancel/result/owned-1".into();
                operation.result_schema_digest =
                    canonical_digest("sha256:", &operation.result_schema)?;
            }
        }
        registry.digest = if workspace_rebind_enabled {
            canonical_digest(
                "frontend-lsp-registry:sha256:",
                &(
                    &registry.operations,
                    wow_emmy::EMMYLUA_REVISION,
                    LSP_OWNED_PROFILE_ID,
                    LSP_WORKSPACE_REBIND_POLICY,
                ),
            )?
        } else {
            canonical_digest(
                "frontend-lsp-registry:sha256:",
                &(
                    &registry.operations,
                    wow_emmy::EMMYLUA_REVISION,
                    LSP_OWNED_PROFILE_ID,
                ),
            )?
        };
        registry.profile_digest = canonical_digest(
            "frontend-lsp-profile:sha256:",
            &(&registry.profile, &registry.digest),
        )?;
        Ok(registry)
    }
    pub(super) fn require(&self, method: &str) -> ServiceResult<()> {
        if self
            .operations
            .iter()
            .any(|entry| entry.wire_method == method)
        {
            Ok(())
        } else {
            Err(invalid_frontend_request("LSP operation is not registered"))
        }
    }
}
