use super::*;

#[derive(Debug, Clone, Serialize)]
pub struct LspOverlayBinding {
    pub schema: &'static str,
    pub id: Box<str>,
    pub session_id: Box<str>,
    pub registry_digest: Box<str>,
    pub snapshot: OverlaySnapshotDescriptor,
    pub analyzer_backend: wow_emmy::EmmyBackendIdentity,
    pub analyzer_workspace_snapshot_id: Box<str>,
    pub analyzer_analysis_id: Box<str>,
    pub position_encoding: &'static str,
    pub coverage: &'static str,
    pub resource_closure: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<OverlaySaveObservation>,
    pub not_evaluated: &'static [&'static str],
}
#[derive(Debug, Clone, Serialize)]
pub struct LspDiagnosticData {
    pub binding_id: Box<str>,
    pub original: wow_emmy::EmmySyntaxDiagnostic,
}
#[derive(Debug, Clone, Serialize)]
pub struct LspDiagnostic {
    pub range: LspRange,
    pub severity: u8,
    pub code: Box<str>,
    pub source: &'static str,
    pub message: &'static str,
    pub data: LspDiagnosticData,
}
/// The selected profile carries its exact owner binding even for empty and
/// unchanged standard diagnostic reports, rather than silently losing scope.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspDiagnosticReport {
    pub kind: &'static str,
    pub result_id: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<LspDiagnostic>>,
    pub wow: LspOverlayBinding,
}
impl LspFrontend {
    pub fn frontend_document_diagnostics(
        &mut self,
        uri: &str,
        previous: Option<&str>,
        ticket: LspRequestId,
        stop: &AtomicBool,
    ) -> ServiceResult<LspDiagnosticReport> {
        self.registry.require("textDocument/diagnostic")?;
        ticket.validate()?;
        let result = self.diagnostics(uri, previous, stop);
        if let Some(session) = self.session.as_mut() {
            session.last_completed = Some(ticket);
        }
        result
    }
    fn diagnostics(
        &mut self,
        uri: &str,
        previous: Option<&str>,
        stop: &AtomicBool,
    ) -> ServiceResult<LspDiagnosticReport> {
        checkpoint(stop)?;
        if previous.is_some_and(|id| id.len() > 256) {
            return Err(invalid_frontend_request("invalid diagnostic result ID"));
        }
        let backend = self.host.backend.clone();
        let session = self.active()?;
        let path = session
            .workspace
            .relative_path_for_uri(&session.scope, uri)
            .map_err(owner)?;
        let head = session.heads.get(&path).ok_or_else(missing)?;
        if head.resynchronization_required {
            return Err(resynchronization());
        }
        if let Some(cached) = &session.diagnostic
            && cached.wow.snapshot.snapshot_id == head.snapshot.snapshot_id
        {
            let mut result = cached.clone();
            if previous == Some(result.result_id.as_ref()) {
                result.kind = "unchanged";
                result.items = None;
            }
            return Ok(result);
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
        let input = session
            .workspace
            .analyzer_input(&session.scope, lease, backend.clone(), stop)
            .map_err(owner)?;
        let syntax = wow_emmy::analyze_syntax_with_stop(input.workspace(), stop).map_err(|_| {
            if stop.load(Ordering::Acquire) {
                ServiceError::new(ServiceErrorCode::Cancelled, "LSP analysis cancelled")
            } else {
                ServiceError::new(
                    ServiceErrorCode::ComponentUnavailable,
                    "LSP syntax analysis refused",
                )
            }
        })?;
        drop(input);
        if syntax.files().len() != 1
            || syntax.files()[0].path() != snapshot.relative_path.as_ref()
            || syntax.files()[0].content_sha256() != snapshot.content_digest.to_string()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "LSP diagnostic snapshot/bounds mismatch",
            ));
        }
        if syntax.diagnostics().len() > LSP_MAX_DIAGNOSTICS {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "LSP diagnostic count exceeds profile limit",
            ));
        }
        let mut offsets = Vec::with_capacity(syntax.diagnostics().len() * 2);
        for diagnostic in syntax.diagnostics() {
            if diagnostic.path() != snapshot.relative_path.as_ref()
                || diagnostic.content_sha256() != snapshot.content_digest.to_string()
            {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "LSP diagnostic identity mismatch",
                ));
            }
            let span = diagnostic.span();
            let start = span.byte_start().and_then(|n| usize::try_from(n).ok());
            let end = span.byte_end().and_then(|n| usize::try_from(n).ok());
            let (Some(start), Some(end)) = (start, end) else {
                return Err(ServiceError::new(
                    ServiceErrorCode::ComponentUnavailable,
                    "LSP diagnostic position unavailable",
                ));
            };
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
        let binding = LspOverlayBinding {
            schema: "wow-service/lsp-overlay-binding/1",
            id: canonical_digest(
                "lsp-overlay-binding:sha256:",
                &(
                    &session.descriptor,
                    &snapshot,
                    &save,
                    &backend,
                    syntax.workspace_snapshot_id(),
                    syntax.analysis_id(),
                ),
            )?,
            session_id: session.descriptor.session_id.clone(),
            registry_digest: session.descriptor.registry_digest.clone(),
            snapshot,
            analyzer_backend: backend,
            analyzer_workspace_snapshot_id: syntax.workspace_snapshot_id().into(),
            analyzer_analysis_id: syntax.analysis_id().into(),
            position_encoding: session.descriptor.position_encoding,
            coverage: session.descriptor.coverage,
            resource_closure: "operation_leases_closed_session_retained",
            save,
            not_evaluated: session.descriptor.not_evaluated,
        };
        let mut items = Vec::with_capacity(syntax.diagnostics().len());
        for (diagnostic, positions) in syntax
            .diagnostics()
            .iter()
            .zip(positions.as_chunks::<2>().0)
        {
            use wow_emmy::EmmyDiagnosticSeverity;
            let severity = match diagnostic.normalized_severity() {
                EmmyDiagnosticSeverity::Error => 1,
                EmmyDiagnosticSeverity::Warning => 2,
                EmmyDiagnosticSeverity::Hint => 4,
                EmmyDiagnosticSeverity::Information | EmmyDiagnosticSeverity::Unknown => 3,
            };
            items.push(LspDiagnostic {
                range: LspRange {
                    start: positions[0],
                    end: positions[1],
                },
                severity,
                code: diagnostic.upstream_code().into(),
                source: "wow-emmy/private-syntax",
                message: "Diagnostic from the selected local syntax adapter family",
                data: LspDiagnosticData {
                    binding_id: binding.id.clone(),
                    original: diagnostic.clone(),
                },
            });
        }
        let result = LspDiagnosticReport {
            kind: "full",
            result_id: canonical_digest(
                "lsp-diagnostic:sha256:",
                &(&binding, &items, syntax.analysis_id()),
            )?,
            items: Some(items),
            wow: binding,
        };
        // One metadata-only result is retained, never a source/body journal.
        if serde_json::to_vec(&result)
            .map_err(|_| invalid_frontend_request("LSP result serialization refused"))?
            .len()
            > LSP_MAX_RETAINED_REPORT_BYTES
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "LSP diagnostic result exceeds profile limit",
            ));
        }
        checkpoint(stop)?;
        session.diagnostic = Some(result.clone());
        let mut result = result;
        if previous == Some(result.result_id.as_ref()) {
            result.kind = "unchanged";
            result.items = None;
        }
        Ok(result)
    }
}
