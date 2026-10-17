//! Bounded native outline observations for one exact private Lua snapshot.
//! The native algorithm is adapted from the pinned EmmyLua LS outline donor.
//! Copyright (c) 2024-2026 CppCXY, MIT: DOCUMENT_SYMBOLS_DONOR_LICENSE.
use crate::{EmmyBackendIdentity, LuaWorkspaceSnapshot, LuaWorkspaceUniverse};
use emmylua_parser::LuaAstNode;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_core::SourceSpan;

mod builder;
mod comment;
mod expr;
mod stats;
#[cfg(test)]
mod tests;
mod walk;

pub const DOCUMENT_SYMBOLS_PROFILE: &str = "native-document-outline/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentSymbolsLimits {
    pub max_file_bytes: usize,
    pub max_work: usize,
    pub max_nodes: usize,
    pub max_symbols: usize,
    pub max_depth: usize,
    pub max_ast_depth: usize,
    pub max_name_bytes: usize,
    pub max_detail_bytes: usize,
    pub max_report_bytes: usize,
}
impl DocumentSymbolsLimits {
    fn validate(self) -> DocumentSymbolsResult<()> {
        if self.max_file_bytes == 0
            || self.max_file_bytes > 16 * 1024 * 1024
            || self.max_work == 0
            || self.max_work > 1024 * 1024 * 1024
            || self.max_nodes == 0
            || self.max_nodes > 1_048_576
            || self.max_symbols == 0
            || self.max_symbols > 2048
            || self.max_depth == 0
            || self.max_depth > 64
            || self.max_ast_depth == 0
            || self.max_ast_depth > 256
            || self.max_name_bytes == 0
            || self.max_name_bytes > 4096
            || self.max_detail_bytes == 0
            || self.max_detail_bytes > 8192
            || self.max_report_bytes == 0
            || self.max_report_bytes > 4 * 1024 * 1024
        {
            return Err(error(DocumentSymbolsErrorCode::Limit));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSymbolKind {
    File,
    Module,
    Namespace,
    Class,
    Function,
    Variable,
    String,
    Number,
    Boolean,
    Object,
    Field,
    Struct,
}
// Concrete donor spelling stays private; public records use the normalized enum.
impl DocumentSymbolKind {
    const FILE: Self = Self::File;
    const MODULE: Self = Self::Module;
    const NAMESPACE: Self = Self::Namespace;
    const FUNCTION: Self = Self::Function;
    const FIELD: Self = Self::Field;
    const STRUCT: Self = Self::Struct;
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentSymbol {
    pub name: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_name: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<Box<str>>,
    pub kind: DocumentSymbolKind,
    pub range: SourceSpan,
    pub selection_range: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_index: Option<usize>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentSymbolsState {
    Ready,
    SourceParseFailed,
    NotEvaluated,
}
#[derive(Debug, Clone, Serialize)]
pub struct DocumentSymbolsReport {
    pub schema: &'static str,
    pub profile: &'static str,
    pub backend: EmmyBackendIdentity,
    pub workspace_snapshot_id: Box<str>,
    pub path: Box<str>,
    pub content_sha256: Box<str>,
    pub limits: DocumentSymbolsLimits,
    pub state: DocumentSymbolsState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
    pub parse_error_count: usize,
    pub symbols: Vec<DocumentSymbol>,
    pub work_used: usize,
    pub digest: Box<str>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSymbolsErrorCode {
    InvalidWorkspace,
    Limit,
    Cancelled,
    Analyzer,
    InvalidSpan,
    Canonicalization,
}
#[derive(Debug, Clone)]
pub struct DocumentSymbolsError {
    code: DocumentSymbolsErrorCode,
}
impl DocumentSymbolsError {
    pub const fn code(&self) -> DocumentSymbolsErrorCode {
        self.code
    }
}
impl fmt::Display for DocumentSymbolsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native outline refused ({:?})", self.code)
    }
}
impl std::error::Error for DocumentSymbolsError {}
pub type DocumentSymbolsResult<T> = Result<T, DocumentSymbolsError>;
fn error(code: DocumentSymbolsErrorCode) -> DocumentSymbolsError {
    DocumentSymbolsError { code }
}

struct Budget<'a> {
    limits: DocumentSymbolsLimits,
    stop: &'a AtomicBool,
    work: usize,
    nodes: usize,
    retained: usize,
    failure: Option<DocumentSymbolsErrorCode>,
}
impl<'a> Budget<'a> {
    fn charge(&mut self, amount: usize) -> DocumentSymbolsResult<()> {
        if let Some(code) = self.failure {
            return Err(error(code));
        }
        if self.stop.load(Ordering::Acquire) {
            self.failure = Some(DocumentSymbolsErrorCode::Cancelled);
            return Err(error(DocumentSymbolsErrorCode::Cancelled));
        }
        self.work = self
            .work
            .checked_add(amount)
            .filter(|work| *work <= self.limits.max_work)
            .ok_or_else(|| error(DocumentSymbolsErrorCode::Limit))?;
        Ok(())
    }
    fn node(&mut self) -> DocumentSymbolsResult<()> {
        self.charge(1)?;
        self.nodes = self
            .nodes
            .checked_add(1)
            .filter(|nodes| *nodes <= self.limits.max_nodes)
            .ok_or_else(|| error(DocumentSymbolsErrorCode::Limit))?;
        Ok(())
    }
}

pub fn document_symbols(
    snapshot: &LuaWorkspaceSnapshot,
    limits: DocumentSymbolsLimits,
    stop: &AtomicBool,
) -> DocumentSymbolsResult<DocumentSymbolsReport> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        collect(snapshot, limits, stop)
    }))
    .unwrap_or_else(|_| Err(error(DocumentSymbolsErrorCode::Analyzer)))
}
fn collect(
    snapshot: &LuaWorkspaceSnapshot,
    limits: DocumentSymbolsLimits,
    stop: &AtomicBool,
) -> DocumentSymbolsResult<DocumentSymbolsReport> {
    limits.validate()?;
    if snapshot.files().len() != 1
        || !matches!(
            snapshot.universe(),
            LuaWorkspaceUniverse::Project | LuaWorkspaceUniverse::Fixture
        )
    {
        return Err(error(DocumentSymbolsErrorCode::InvalidWorkspace));
    }
    let file = &snapshot.files()[0];
    if file.text().len() > limits.max_file_bytes {
        return Err(error(DocumentSymbolsErrorCode::Limit));
    }
    let mut budget = Budget {
        limits,
        stop,
        work: 0,
        nodes: 0,
        retained: 0,
        failure: None,
    };
    budget.charge(
        file.text()
            .len()
            .checked_mul(4)
            .ok_or_else(|| error(DocumentSymbolsErrorCode::Limit))?,
    )?;
    let root_uri = crate::syntax::analyzer::virtual_root(snapshot.snapshot_id());
    let analysis = crate::syntax::analyzer::build_registered(snapshot, &root_uri)
        .map_err(|_| error(DocumentSymbolsErrorCode::Analyzer))?;
    budget.charge(1)?;
    let model = crate::references::semantic_model(&analysis, &root_uri, file)
        .map_err(|_| error(DocumentSymbolsErrorCode::Analyzer))?;
    let tree = model
        .get_db()
        .get_vfs()
        .get_syntax_tree(&model.get_file_id())
        .ok_or_else(|| error(DocumentSymbolsErrorCode::Analyzer))?;
    let mut report = DocumentSymbolsReport {
        schema: "wow-emmy/document-symbols/1",
        profile: DOCUMENT_SYMBOLS_PROFILE,
        backend: snapshot.backend().clone(),
        workspace_snapshot_id: snapshot.snapshot_id().into(),
        path: file.path().into(),
        content_sha256: file.content_sha256().into(),
        limits,
        state: DocumentSymbolsState::NotEvaluated,
        reason: None,
        parse_error_count: tree.get_errors().len(),
        symbols: Vec::new(),
        work_used: 0,
        digest: "".into(),
    };
    if report.parse_error_count != 0 {
        report.state = DocumentSymbolsState::SourceParseFailed;
        report.reason = Some("source_parse_failed");
        report.work_used = budget.work;
    } else if let Some(declarations) = model
        .get_db()
        .get_decl_index()
        .get_decl_tree(&model.get_file_id())
    {
        let root = model.get_root();
        let mut depth = 0usize;
        for event in root.syntax().preorder() {
            budget.node()?;
            match event {
                rowan::WalkEvent::Enter(_) => {
                    depth = depth
                        .checked_add(1)
                        .ok_or_else(|| error(DocumentSymbolsErrorCode::Limit))?;
                    if depth > limits.max_ast_depth {
                        return Err(error(DocumentSymbolsErrorCode::Limit));
                    }
                }
                rowan::WalkEvent::Leave(_) => {
                    depth = depth.saturating_sub(1);
                }
            }
        }
        let mut builder = builder::DocumentSymbolBuilder::new(
            model.get_db(),
            declarations,
            model.get_file_id(),
            file,
            budget,
        );
        let root_id = builder.add_node_symbol(
            root.syntax().clone(),
            builder::LuaSymbol::new("".into(), None, DocumentSymbolKind::FILE, root.get_range()),
            None,
        );
        let complete = walk::process_chunk(&mut builder, root, root_id).is_some();
        let (symbols, work) = builder.finish(root_id, complete)?;
        report.work_used = work;
        if complete {
            report.state = DocumentSymbolsState::Ready;
            report.symbols = symbols;
        } else {
            report.reason = Some("native_outline_prerequisite_unavailable");
        }
    } else {
        report.reason = Some("native_declaration_tree_unavailable");
        report.work_used = budget.work;
    }
    if stop.load(Ordering::Acquire) {
        return Err(error(DocumentSymbolsErrorCode::Cancelled));
    }
    let mut identity = serde_json::to_value(&report)
        .map_err(|_| error(DocumentSymbolsErrorCode::Canonicalization))?;
    identity
        .as_object_mut()
        .ok_or_else(|| error(DocumentSymbolsErrorCode::Canonicalization))?
        .remove("digest");
    let bytes = wow_core::canonical_json_bytes(&identity)
        .map_err(|_| error(DocumentSymbolsErrorCode::Canonicalization))?;
    if bytes.len() > limits.max_report_bytes {
        return Err(error(DocumentSymbolsErrorCode::Limit));
    }
    report.digest = format!("emmy-document-symbols:sha256:{:x}", Sha256::digest(&bytes)).into();
    if serde_json::to_vec(&report)
        .map_err(|_| error(DocumentSymbolsErrorCode::Canonicalization))?
        .len()
        > limits.max_report_bytes
    {
        return Err(error(DocumentSymbolsErrorCode::Limit));
    }
    if stop.load(Ordering::Acquire) {
        return Err(error(DocumentSymbolsErrorCode::Cancelled));
    }
    Ok(report)
}
