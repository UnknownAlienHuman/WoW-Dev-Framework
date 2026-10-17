//! Exact private-overlay local references; all analyzer semantics stay in Emmy.
use super::*;
use wow_emmy::local_references::{
    LocalReferencesErrorCode, LocalReferencesLimits, LocalReferencesReport, LocalReferencesState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspReferencesRequest {
    pub uri: String,
    pub position: LspPosition,
    pub include_declaration: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspReferencesLocation {
    pub uri: Box<str>,
    pub range: LspRange,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspReferencesResult {
    pub schema: &'static str,
    pub locations: Vec<LspReferencesLocation>,
    pub binding: LspOverlayBinding,
    pub original: LocalReferencesReport,
}
impl LspFrontend {
    pub fn frontend_document_references(
        &mut self,
        request: LspReferencesRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LspReferencesResult> {
        if !self.references_enabled() {
            return Err(ServiceError::new(
                ServiceErrorCode::ComponentUnavailable,
                "local references disabled by explicit navigation policy",
            ));
        }
        self.registry.require("textDocument/references")?;
        checkpoint(stop)?;
        let backend = self.host.backend.clone();
        let limits = LocalReferencesLimits {
            max_file_bytes: self.host.input.limits.max_document_bytes,
            max_work: self.host.input.limits.max_work,
            max_name_bytes: 256,
            max_references: LSP_MAX_LOCAL_REFERENCES,
        };
        let session = self.active()?;
        let path = session.document_path(&request.uri)?;
        let head = session.heads.get(&path).ok_or_else(missing)?;
        if head.resynchronization_required {
            return Err(resynchronization());
        }
        let snapshot = head.snapshot.clone();
        let save = head.save.clone();
        let lease = session
            .workspace
            .get(
                &session.scope,
                &snapshot.document_id,
                &snapshot.snapshot_id,
                stop,
            )
            .map_err(owner)?;
        let offsets = session
            .workspace
            .byte_offsets(
                &session.scope,
                &lease,
                &[request.position],
                session.encoding,
                stop,
            )
            .map_err(owner)?;
        let offset = *offsets
            .first()
            .ok_or_else(|| invalid_frontend_request("references offset unavailable"))?;
        let input = session
            .workspace
            .analyzer_input(&session.scope, lease, backend.clone(), stop)
            .map_err(owner)?;
        let original = wow_emmy::local_references::local_references(
            input.workspace(),
            offset,
            request.include_declaration,
            limits,
            stop,
        )
        .map_err(|error| {
            ServiceError::new(
                match error.code() {
                    LocalReferencesErrorCode::Cancelled => ServiceErrorCode::Cancelled,
                    LocalReferencesErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
                    LocalReferencesErrorCode::InvalidPosition => ServiceErrorCode::InvalidRequest,
                    _ => ServiceErrorCode::ComponentUnavailable,
                },
                "native local references refused",
            )
        })?;
        let input_id = input.workspace().snapshot_id().to_owned();
        drop(input);
        if original.schema != "wow-emmy/local-references-result/1"
            || original.profile != "same_file_lexical_local_references_no_trace/1"
            || original.limits != limits
            || original.coverage != "same_file_native_local_reference_index_observed_subset"
            || original.workspace_snapshot_id.as_ref() != input_id
            || original.backend != backend
            || original.path != snapshot.relative_path
            || original.content_sha256.as_ref() != snapshot.content_digest.to_string()
            || original.query_byte_offset != offset
            || original.include_declaration != request.include_declaration
            || original.negative_authority
            || original.references.len() > LSP_MAX_LOCAL_REFERENCES
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "local references identity mismatch",
            ));
        }
        // Charge repeated URI bytes before creating one owned URI per location.
        let uri_bytes = serde_json::to_vec(&request.uri)
            .map_err(|_| invalid_frontend_request("references URI encoding refused"))?
            .len();
        if uri_bytes
            .checked_add(256)
            .and_then(|unit| unit.checked_mul(original.references.len()))
            .is_none_or(|bytes| bytes > LSP_MAX_RETAINED_REPORT_BYTES / 2)
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "local reference locations exceed output budget",
            ));
        }
        let mut locations = Vec::new();
        if original.state == LocalReferencesState::ResolvedLocal {
            if original.origin_span.is_none() || original.declaration_span.is_none() {
                return Err(invalid_frontend_request(
                    "native references declaration unavailable",
                ));
            }
            let mut offsets = Vec::with_capacity(original.references.len() * 2);
            let mut previous = None;
            for span in &original.references {
                checkpoint(stop)?;
                let start = span.byte_start().and_then(|n| usize::try_from(n).ok());
                let end = span.byte_end().and_then(|n| usize::try_from(n).ok());
                let (Some(start), Some(end)) = (start, end) else {
                    return Err(invalid_frontend_request(
                        "native references byte span unavailable",
                    ));
                };
                if start >= end || previous.is_some_and(|prior| prior >= (start, end)) {
                    return Err(invalid_frontend_request(
                        "native references order unavailable",
                    ));
                }
                previous = Some((start, end));
                offsets.extend([start, end]);
            }
            let lease = session
                .workspace
                .get(
                    &session.scope,
                    &snapshot.document_id,
                    &snapshot.snapshot_id,
                    stop,
                )
                .map_err(owner)?;
            let positions = session
                .workspace
                .positions(&session.scope, &lease, &offsets, session.encoding, stop)
                .map_err(owner)?;
            drop(lease);
            if positions.len() != offsets.len() {
                return Err(ServiceError::new(
                    ServiceErrorCode::InternalContractViolation,
                    "references projection incomplete",
                ));
            }
            for pair in positions.as_chunks::<2>().0 {
                locations.push(LspReferencesLocation {
                    uri: request.uri.clone().into(),
                    range: LspRange {
                        start: pair[0],
                        end: pair[1],
                    },
                });
            }
        } else if !original.references.is_empty() || original.declaration_span.is_some() {
            return Err(ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "unsupported references result contains locations",
            ));
        }
        let binding = LspOverlayBinding {
            schema: "wow-service/lsp-overlay-binding/1",
            id: canonical_digest(
                "lsp-references-binding:sha256:",
                &(&session.descriptor, &snapshot, &save, &original),
            )?,
            session_id: session.descriptor.session_id.clone(),
            registry_digest: session.descriptor.registry_digest.clone(),
            snapshot,
            analyzer_backend: backend,
            analyzer_workspace_snapshot_id: input_id.into(),
            analyzer_analysis_id: original.analysis_id.clone(),
            position_encoding: session.descriptor.position_encoding,
            coverage: "overlay_local_lexical_references_no_published_base",
            resource_closure: "operation_leases_closed_session_retained",
            save,
            not_evaluated: session.descriptor.not_evaluated,
        };
        let result = LspReferencesResult {
            schema: "wow-service/lsp-local-references-result/1",
            locations,
            binding,
            original,
        };
        if serde_json::to_vec(&result)
            .map_err(|_| invalid_frontend_request("references result encoding refused"))?
            .len()
            > LSP_MAX_RETAINED_REPORT_BYTES
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "local references result exceeds profile limit",
            ));
        }
        checkpoint(stop)?;
        Ok(result)
    }
}
