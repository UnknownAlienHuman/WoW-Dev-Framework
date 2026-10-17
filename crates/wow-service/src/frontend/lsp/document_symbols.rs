use super::*;
use wow_emmy::document_symbols::{
    DocumentSymbolKind, DocumentSymbolsErrorCode, DocumentSymbolsLimits, DocumentSymbolsReport,
    DocumentSymbolsState,
};

pub const LSP_DOCUMENT_SYMBOL_DISCLOSURE: &str = "same_document_outline_v1";
pub const LSP_MAX_DOCUMENT_SYMBOLS: usize = 2048;
pub const LSP_MAX_DOCUMENT_SYMBOL_DEPTH: usize = 12;
pub const LSP_MAX_DOCUMENT_SYMBOL_RESULT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDocumentSymbolClientCapabilities {
    pub hierarchical: bool,
    pub extended_kinds: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LspDocumentSymbolsRequest {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LspDocumentSymbol {
    pub name: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Box<str>>,
    pub kind: u8,
    pub range: LspRange,
    pub selection_range: LspRange,
    pub parent_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LspDocumentSymbolsResult {
    pub schema: &'static str,
    pub uri: Box<str>,
    pub hierarchical: bool,
    pub symbols: Vec<LspDocumentSymbol>,
    pub binding: LspOverlayBinding,
    pub original: DocumentSymbolsReport,
}
impl LspDocumentSymbolsResult {
    pub fn projection_available(&self) -> bool {
        matches!(&self.original.state, DocumentSymbolsState::Ready)
    }
}

impl LspFrontend {
    pub fn frontend_document_symbols(
        &mut self,
        request: LspDocumentSymbolsRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<LspDocumentSymbolsResult> {
        if !self.document_symbols_enabled() {
            return Err(ServiceError::new(
                ServiceErrorCode::ComponentUnavailable,
                "document symbols require explicit source-label disclosure",
            ));
        }
        self.registry.require("textDocument/documentSymbol")?;
        checkpoint(stop)?;
        let backend = self.host.backend.clone();
        let limits = DocumentSymbolsLimits {
            max_file_bytes: self.host.input.limits.max_document_bytes,
            max_work: self.host.input.limits.max_work,
            max_nodes: 131_072,
            max_symbols: LSP_MAX_DOCUMENT_SYMBOLS,
            max_depth: LSP_MAX_DOCUMENT_SYMBOL_DEPTH,
            max_ast_depth: 256,
            max_name_bytes: 1024,
            max_detail_bytes: 2048,
            max_report_bytes: LSP_MAX_DOCUMENT_SYMBOL_RESULT_BYTES,
        };
        let session = self.active()?;
        let client = session.descriptor.document_symbols_client.ok_or_else(|| {
            invalid_frontend_request("document-symbol client projection unavailable")
        })?;
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
        let input = session
            .workspace
            .analyzer_input(&session.scope, lease, backend.clone(), stop)
            .map_err(owner)?;
        let original =
            wow_emmy::document_symbols::document_symbols(input.workspace(), limits, stop).map_err(
                |error| {
                    ServiceError::new(
                        match error.code() {
                            DocumentSymbolsErrorCode::Cancelled => ServiceErrorCode::Cancelled,
                            DocumentSymbolsErrorCode::Limit => ServiceErrorCode::BudgetExceeded,
                            _ => ServiceErrorCode::ComponentUnavailable,
                        },
                        "native document outline refused",
                    )
                },
            )?;
        let input_id = input.workspace().snapshot_id().to_owned();
        drop(input);
        if original.workspace_snapshot_id.as_ref() != input_id
            || original.backend != backend
            || original.path != snapshot.relative_path
            || original.content_sha256.as_ref() != snapshot.content_digest.to_string()
            || original.profile != wow_emmy::document_symbols::DOCUMENT_SYMBOLS_PROFILE
        {
            return Err(ServiceError::new(
                ServiceErrorCode::IdentityMismatch,
                "document outline identity mismatch",
            ));
        }
        if original.symbols.len() > LSP_MAX_DOCUMENT_SYMBOLS {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "document outline exceeds symbol limit",
            ));
        }
        let ready = matches!(&original.state, DocumentSymbolsState::Ready);
        if !ready && !original.symbols.is_empty() {
            return Err(ServiceError::new(
                ServiceErrorCode::InternalContractViolation,
                "unqualified document outline contains symbols",
            ));
        }
        let mut depths = Vec::<usize>::with_capacity(original.symbols.len());
        let mut offsets = Vec::with_capacity(original.symbols.len() * 4);
        for (index, symbol) in original.symbols.iter().enumerate() {
            checkpoint(stop)?;
            let depth = match symbol.parent_index {
                None => 1,
                Some(parent) if parent < index => depths[parent] + 1,
                Some(_) => {
                    return Err(ServiceError::new(
                        ServiceErrorCode::IdentityMismatch,
                        "native outline parent is not an exact prior record",
                    ));
                }
            };
            if depth > LSP_MAX_DOCUMENT_SYMBOL_DEPTH || symbol.name.trim().is_empty() {
                return Err(ServiceError::new(
                    ServiceErrorCode::InternalContractViolation,
                    "native outline depth or title is invalid",
                ));
            }
            depths.push(depth);
            let spans = [
                symbol.range.byte_start(),
                symbol.range.byte_end(),
                symbol.selection_range.byte_start(),
                symbol.selection_range.byte_end(),
            ]
            .map(|value| {
                value
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or_else(|| invalid_frontend_request("outline byte span unavailable"))
            });
            let [start, end, selection_start, selection_end] = spans;
            let (start, end, selection_start, selection_end) =
                (start?, end?, selection_start?, selection_end?);
            if start > selection_start || selection_start > selection_end || selection_end > end {
                return Err(ServiceError::new(
                    ServiceErrorCode::IdentityMismatch,
                    "native outline selection exceeds its exact range",
                ));
            }
            offsets.extend([start, end, selection_start, selection_end]);
        }
        let mut symbols = Vec::with_capacity(original.symbols.len());
        if ready {
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
                    "document outline projection incomplete",
                ));
            }
            for (symbol, positions) in original.symbols.iter().zip(positions.as_chunks::<4>().0) {
                checkpoint(stop)?;
                symbols.push(LspDocumentSymbol {
                    name: symbol.name.clone(),
                    detail: symbol.detail.clone(),
                    kind: wire_kind(&symbol.kind, client.extended_kinds),
                    range: LspRange {
                        start: positions[0],
                        end: positions[1],
                    },
                    selection_range: LspRange {
                        start: positions[2],
                        end: positions[3],
                    },
                    parent_index: symbol.parent_index,
                });
            }
        }
        let binding = LspOverlayBinding {
            schema: "wow-service/lsp-overlay-binding/1",
            id: canonical_digest(
                "lsp-document-symbols-binding:sha256:",
                &(&session.descriptor, &snapshot, &save, &original),
            )?,
            session_id: session.descriptor.session_id.clone(),
            registry_digest: session.descriptor.registry_digest.clone(),
            snapshot,
            analyzer_backend: backend,
            analyzer_workspace_snapshot_id: input_id.into(),
            analyzer_analysis_id: original.digest.clone(),
            position_encoding: session.descriptor.position_encoding,
            coverage: "overlay_local_native_document_outline_no_published_base",
            resource_closure: "operation_leases_closed_session_retained",
            save,
            not_evaluated: session.descriptor.not_evaluated,
        };
        let result = LspDocumentSymbolsResult {
            schema: "wow-service/lsp-document-symbols-result/1",
            uri: request.uri.into(),
            hierarchical: client.hierarchical,
            symbols,
            binding,
            original,
        };
        if serde_json::to_vec(&result)
            .map_err(|_| invalid_frontend_request("outline result encoding refused"))?
            .len()
            > LSP_MAX_DOCUMENT_SYMBOL_RESULT_BYTES
        {
            return Err(ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "document outline result exceeds profile limit",
            ));
        }
        checkpoint(stop)?;
        Ok(result)
    }
}

fn wire_kind(kind: &DocumentSymbolKind, extended: bool) -> u8 {
    let value = match kind {
        DocumentSymbolKind::File => 1,
        DocumentSymbolKind::Module => 2,
        DocumentSymbolKind::Namespace => 3,
        DocumentSymbolKind::Class => 5,
        DocumentSymbolKind::Field => 8,
        DocumentSymbolKind::Function => 12,
        DocumentSymbolKind::Variable => 13,
        DocumentSymbolKind::String => 15,
        DocumentSymbolKind::Number => 16,
        DocumentSymbolKind::Boolean => 17,
        DocumentSymbolKind::Object => 19,
        DocumentSymbolKind::Struct => 23,
    };
    // The exact owner kind remains in `original`. The initial LSP client set
    // ends at Array(18); Variable is its generic UI fallback, not new owner truth.
    if value > 18 && !extended { 13 } else { value }
}
