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
    pub document_uri_profile: Option<&'static str>,
    pub max_header_bytes: usize,
    pub max_frame_bytes: usize,
    pub max_output_bytes: usize,
    pub max_json_depth: usize,
    pub max_active_requests: usize,
    pub max_queued_requests: usize,
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
    pub implementation_pin: &'static str,
    pub profile: LspTransportProfile,
    pub profile_digest: Box<str>,
}
fn object(properties: Value, required: &[&str]) -> Value {
    json!({"type":"object","properties":properties,"required":required,"additionalProperties":false})
}
impl LspRegistry {
    pub(super) fn build(
        local_definition: bool,
        uri_policy: OverlayUriPolicy,
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
                    if client_uris && operation == "frontend_session_open" {
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
                error_schema: error,
                effect_class: "pure_read",
            });
        }
        operations.sort_by_key(|entry| entry.wire_method);
        let digest = canonical_digest(
            "frontend-lsp-registry:sha256:",
            &(&operations, wow_emmy::EMMYLUA_REVISION, LSP_PROFILE_ID),
        )?;
        let profile = LspTransportProfile {
            schema: if client_uris {
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
            document_uri_profile: if client_uris {
                Some(uri_policy.id())
            } else {
                None
            },
            max_header_bytes: LSP_MAX_HEADER_BYTES,
            max_frame_bytes: LSP_MAX_FRAME_BYTES,
            max_output_bytes: LSP_MAX_OUTPUT_BYTES,
            max_json_depth: LSP_MAX_JSON_DEPTH,
            max_active_requests: 1,
            max_queued_requests: 0,
            max_diagnostics: LSP_MAX_DIAGNOSTICS,
            max_retained_report_bytes: LSP_MAX_RETAINED_REPORT_BYTES,
        };
        let profile_digest =
            canonical_digest("frontend-lsp-profile:sha256:", &(&profile, &digest))?;
        Ok(Self {
            schema: "wow-service/lsp-operation-registry/1",
            registry_id: super::super::FRONTEND_REGISTRY_ID,
            profile_id: LSP_PROFILE_ID,
            digest,
            operations,
            implementation_pin: wow_emmy::EMMYLUA_REVISION,
            profile,
            profile_digest,
        })
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
