use super::*;
use wow_emmy::navigation::{
    LocalDefinitionErrorCode, LocalDefinitionLimits, LocalDefinitionReport, LocalDefinitionState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDefinitionRequest {
    pub uri: String,
    pub position: LspPosition,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspDefinitionLocation {
    pub uri: Box<str>,
    pub range: LspRange,
    pub origin_range: LspRange,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspDefinitionResult {
    pub schema: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LspDefinitionLocation>,
    pub binding: LspOverlayBinding,
    pub original: LocalDefinitionReport,
}
impl LspFrontend {
    pub fn frontend_document_definition(
        &mut self,
        request: LspDefinitionRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LspDefinitionResult> {
        if !self.definition_enabled() {
            return Err(ServiceError::new(
                ServiceErrorCode::ComponentUnavailable,
                "local definition disabled by explicit privacy profile",
            ));
        }
        self.registry.require("textDocument/definition")?;
        checkpoint(stop)?;
        let backend = self.host.backend.clone();
        let limits = LocalDefinitionLimits {
            max_file_bytes: self.host.input.limits.max_document_bytes,
            max_work: self.host.input.limits.max_work,
            max_name_bytes: 256,
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
            .ok_or_else(|| invalid_frontend_request("definition offset unavailable"))?;
        let input = session
            .workspace
            .analyzer_input(&session.scope, lease, backend.clone(), stop)
            .map_err(owner)?;
        let original =
            wow_emmy::navigation::local_definition(input.workspace(), offset, limits, stop)
                .map_err(|error| {
                    ServiceError::new(
                        match error.code() {
                            LocalDefinitionErrorCode::Cancelled => ServiceErrorCode::Cancelled,
                            LocalDefinitionErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
                            LocalDefinitionErrorCode::InvalidPosition => {
                                ServiceErrorCode::InvalidRequest
                            }
                            _ => ServiceErrorCode::ComponentUnavailable,
                        },
                        "native local definition refused",
                    )
                })?;
        let input_id = input.workspace().snapshot_id().to_owned();
        drop(input);
        if original.workspace_snapshot_id.as_ref() != input_id
            || original.backend != backend
            || original.path != snapshot.relative_path
            || original.content_sha256.as_ref() != snapshot.content_digest.to_string()
            || original.query_byte_offset != offset
            || original.negative_authority
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "local definition identity mismatch",
            ));
        }
        let location = if original.state == LocalDefinitionState::ResolvedLocal {
            let origin = original
                .origin_span
                .ok_or_else(|| invalid_frontend_request("native origin unavailable"))?;
            let target = &original
                .target
                .as_ref()
                .ok_or_else(|| invalid_frontend_request("native local target unavailable"))?
                .span;
            let offsets = [
                origin.byte_start(),
                origin.byte_end(),
                target.byte_start(),
                target.byte_end(),
            ]
            .into_iter()
            .map(|n| {
                n.and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| invalid_frontend_request("definition byte span unavailable"))
            })
            .collect::<ServiceResult<Vec<_>>>()?;
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
            if positions.len() != 4 {
                return Err(ServiceError::new(
                    ServiceErrorCode::InternalContractViolation,
                    "definition projection incomplete",
                ));
            }
            Some(LspDefinitionLocation {
                uri: request.uri.into(),
                origin_range: LspRange {
                    start: positions[0],
                    end: positions[1],
                },
                range: LspRange {
                    start: positions[2],
                    end: positions[3],
                },
            })
        } else {
            if original.target.is_some() {
                return Err(ServiceError::new(
                    ServiceErrorCode::InternalContractViolation,
                    "nonlocal result contains target",
                ));
            }
            None
        };
        let binding = LspOverlayBinding {
            schema: "wow-service/lsp-overlay-binding/1",
            id: canonical_digest(
                "lsp-definition-binding:sha256:",
                &(&session.descriptor, &snapshot, &save, &original),
            )?,
            session_id: session.descriptor.session_id.clone(),
            registry_digest: session.descriptor.registry_digest.clone(),
            snapshot,
            analyzer_backend: backend,
            analyzer_workspace_snapshot_id: input_id.into(),
            analyzer_analysis_id: original.analysis_id.clone(),
            position_encoding: session.descriptor.position_encoding,
            coverage: "overlay_local_lexical_definition_no_published_base",
            resource_closure: "operation_leases_closed_session_retained",
            save,
            not_evaluated: session.descriptor.not_evaluated,
        };
        checkpoint(stop)?;
        Ok(LspDefinitionResult {
            schema: "wow-service/lsp-local-definition-result/1",
            location,
            binding,
            original,
        })
    }
}
