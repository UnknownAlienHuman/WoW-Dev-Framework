//! Same-file lexical local references from the accepted native reference index.
//! This reports observed byte spans only; it never claims complete absence.

use crate::{EmmyBackendIdentity, LuaWorkspaceFile, LuaWorkspaceSnapshot, LuaWorkspaceUniverse};
use emmylua_code_analysis::{LuaSemanticDeclId, SemanticDeclLevel};
use emmylua_parser::{LuaAstNode, LuaTokenKind};
use rowan::{TextRange, TextSize, TokenAtOffset};
use serde::Serialize;
use std::{
    collections::BTreeSet,
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_core::SourceSpan;

const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
const MAX_WORK: usize = 1024 * 1024 * 1024;
const MAX_NAME_BYTES: usize = 4096;
const MAX_REFERENCES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalReferencesLimits {
    pub max_file_bytes: usize,
    pub max_work: usize,
    pub max_name_bytes: usize,
    pub max_references: usize,
}

impl LocalReferencesLimits {
    fn validate(self) -> Result<(), LocalReferencesError> {
        if self.max_file_bytes == 0
            || self.max_file_bytes > MAX_FILE_BYTES
            || self.max_work == 0
            || self.max_work > MAX_WORK
            || self.max_name_bytes == 0
            || self.max_name_bytes > MAX_NAME_BYTES
            || self.max_references > MAX_REFERENCES
        {
            return Err(error(LocalReferencesErrorCode::Limit));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalReferencesState {
    ResolvedLocal,
    NotEvaluated,
    SourceParseFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalReferencesReason {
    UnsupportedToken,
    DeclarationNotObserved,
    OutsideLocalScope,
    TargetUnavailable,
    ReferenceIndexUnavailable,
}

#[derive(Debug, Clone, Serialize)]
pub struct LocalReferencesReport {
    pub schema: &'static str,
    pub profile: &'static str,
    pub analysis_id: Box<str>,
    pub workspace_snapshot_id: Box<str>,
    pub backend: EmmyBackendIdentity,
    pub path: Box<str>,
    pub content_sha256: Box<str>,
    pub query_byte_offset: usize,
    pub include_declaration: bool,
    pub limits: LocalReferencesLimits,
    pub state: LocalReferencesState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<LocalReferencesReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_span: Option<SourceSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration_span: Option<SourceSpan>,
    pub references: Vec<SourceSpan>,
    pub negative_authority: bool,
    pub coverage: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalReferencesErrorCode {
    InvalidWorkspace,
    InvalidPosition,
    Limit,
    Cancelled,
    Analyzer,
    Canonicalization,
}

#[derive(Debug, Clone)]
pub struct LocalReferencesError {
    code: LocalReferencesErrorCode,
}

impl LocalReferencesError {
    pub const fn code(&self) -> LocalReferencesErrorCode {
        self.code
    }
}

impl fmt::Display for LocalReferencesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "local references refused ({:?})", self.code)
    }
}

impl std::error::Error for LocalReferencesError {}

pub type LocalReferencesResult<T> = Result<T, LocalReferencesError>;

fn error(code: LocalReferencesErrorCode) -> LocalReferencesError {
    LocalReferencesError { code }
}

fn checkpoint(stop: &AtomicBool) -> LocalReferencesResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(error(LocalReferencesErrorCode::Cancelled))
    } else {
        Ok(())
    }
}

fn charge_work(work: &mut usize, amount: usize, limit: usize) -> LocalReferencesResult<()> {
    *work = work
        .checked_add(amount)
        .filter(|charged| *charged <= limit)
        .ok_or_else(|| error(LocalReferencesErrorCode::Limit))?;
    Ok(())
}

/// Resolve one identifier to its same-file lexical local and native reference spans.
///
/// The native model and reference index are built only from this exact snapshot.
/// Reference and output budgets refuse the whole request rather than returning a
/// truncated list. The returned report never grants negative authority.
pub fn local_references(
    input: &LuaWorkspaceSnapshot,
    byte_offset: usize,
    include_declaration: bool,
    limits: LocalReferencesLimits,
    stop: &AtomicBool,
) -> LocalReferencesResult<LocalReferencesReport> {
    limits.validate()?;
    checkpoint(stop)?;
    if input.files().len() != 1
        || !matches!(
            input.universe(),
            LuaWorkspaceUniverse::Project | LuaWorkspaceUniverse::Fixture
        )
    {
        return Err(error(LocalReferencesErrorCode::InvalidWorkspace));
    }

    let file = &input.files()[0];
    let text = file.text();
    if text.len() > limits.max_file_bytes {
        return Err(error(LocalReferencesErrorCode::Limit));
    }
    let mut work = text
        .len()
        .checked_mul(4)
        .filter(|charged| *charged <= limits.max_work)
        .ok_or_else(|| error(LocalReferencesErrorCode::Limit))?;
    if byte_offset > text.len() || !text.is_char_boundary(byte_offset) {
        return Err(error(LocalReferencesErrorCode::InvalidPosition));
    }

    let mut report = LocalReferencesReport {
        schema: "wow-emmy/local-references-result/1",
        profile: "same_file_lexical_local_references_no_trace/1",
        analysis_id: "".into(),
        workspace_snapshot_id: input.snapshot_id().into(),
        backend: input.backend().clone(),
        path: file.path().into(),
        content_sha256: file.content_sha256().into(),
        query_byte_offset: byte_offset,
        include_declaration,
        limits,
        state: LocalReferencesState::NotEvaluated,
        reason: None,
        origin_span: None,
        declaration_span: None,
        references: Vec::new(),
        negative_authority: false,
        coverage: "same_file_native_local_reference_index_observed_subset",
    };

    // Keep all upstream analysis state inside this scope so it is dropped before
    // the owned result is canonicalized or returned.
    {
        let root = crate::syntax::analyzer::virtual_root(input.snapshot_id());
        let analysis = crate::syntax::analyzer::build_registered(input, &root)
            .map_err(|_| error(LocalReferencesErrorCode::Analyzer))?;
        checkpoint(stop)?;
        let model = crate::references::semantic_model(&analysis, &root, file)
            .map_err(|_| error(LocalReferencesErrorCode::Analyzer))?;
        let file_id = model.get_file_id();
        let tree = model
            .get_db()
            .get_vfs()
            .get_syntax_tree(&file_id)
            .ok_or_else(|| error(LocalReferencesErrorCode::Analyzer))?;

        if !tree.get_errors().is_empty() {
            report.state = LocalReferencesState::SourceParseFailed;
        } else {
            let offset = TextSize::from(
                u32::try_from(byte_offset)
                    .map_err(|_| error(LocalReferencesErrorCode::InvalidPosition))?,
            );
            let ast = model.get_root();
            let token = match ast.syntax().token_at_offset(offset) {
                TokenAtOffset::Single(token) => Some(token),
                TokenAtOffset::Between(left, right) => {
                    Some(if left.kind() == LuaTokenKind::TkName.into() {
                        left
                    } else {
                        right
                    })
                }
                TokenAtOffset::None => None,
            };

            if let Some(token) = token.filter(|token| token.kind() == LuaTokenKind::TkName.into()) {
                if token.text().len() > limits.max_name_bytes {
                    return Err(error(LocalReferencesErrorCode::Limit));
                }
                report.origin_span =
                    Some(native_span(file, token.text_range(), Some(token.text()))?);

                match model.find_decl(token.into(), SemanticDeclLevel::NoTrace) {
                    Some(LuaSemanticDeclId::LuaDecl(decl_id)) => {
                        if let Some(declaration) =
                            model.get_db().get_decl_index().get_decl(&decl_id)
                        {
                            if declaration.get_file_id() != file_id || !declaration.is_local() {
                                report.reason = Some(LocalReferencesReason::OutsideLocalScope);
                            } else {
                                let name = declaration.get_name();
                                if name.len() > limits.max_name_bytes {
                                    return Err(error(LocalReferencesErrorCode::Limit));
                                }
                                let declaration_span =
                                    native_span(file, declaration.get_range(), Some(name))?;
                                report.declaration_span = Some(declaration_span);

                                let reference_index = model.get_db().get_reference_index();
                                if reference_index.get_local_reference(&file_id).is_none() {
                                    report.declaration_span = None;
                                    report.reason =
                                        Some(LocalReferencesReason::ReferenceIndexUnavailable);
                                } else {
                                    let mut spans = BTreeSet::new();
                                    let declaration_slots = if include_declaration { 1 } else { 0 };
                                    if let Some(native_references) =
                                        reference_index.get_decl_references(&file_id, &decl_id)
                                    {
                                        for reference in &native_references.cells {
                                            checkpoint(stop)?;
                                            charge_work(
                                                &mut work,
                                                name.len().saturating_add(1),
                                                limits.max_work,
                                            )?;
                                            let span =
                                                native_span(file, reference.range, Some(name))?;
                                            if span == declaration_span || spans.contains(&span) {
                                                continue;
                                            }
                                            let projected = spans
                                                .len()
                                                .checked_add(1)
                                                .and_then(|count| {
                                                    count.checked_add(declaration_slots)
                                                })
                                                .ok_or_else(|| {
                                                    error(LocalReferencesErrorCode::Limit)
                                                })?;
                                            if projected > limits.max_references {
                                                return Err(error(LocalReferencesErrorCode::Limit));
                                            }
                                            spans.insert(span);
                                        }
                                    }

                                    let final_count = spans
                                        .len()
                                        .checked_add(declaration_slots)
                                        .ok_or_else(|| error(LocalReferencesErrorCode::Limit))?;
                                    if final_count > limits.max_references {
                                        return Err(error(LocalReferencesErrorCode::Limit));
                                    }
                                    if include_declaration {
                                        spans.insert(declaration_span);
                                    }
                                    report.references = spans.into_iter().collect();
                                    report.state = LocalReferencesState::ResolvedLocal;
                                }
                            }
                        } else {
                            report.reason = Some(LocalReferencesReason::TargetUnavailable);
                        }
                    }
                    Some(_) => {
                        report.reason = Some(LocalReferencesReason::OutsideLocalScope);
                    }
                    None => {
                        report.reason = Some(LocalReferencesReason::DeclarationNotObserved);
                    }
                }
            } else {
                report.reason = Some(LocalReferencesReason::UnsupportedToken);
            }
        }
    }

    finish_report(report, work, stop)
}

fn finish_report(
    mut report: LocalReferencesReport,
    work: usize,
    stop: &AtomicBool,
) -> LocalReferencesResult<LocalReferencesReport> {
    checkpoint(stop)?;
    let output_work = report
        .references
        .len()
        .checked_mul(96)
        .and_then(|size| size.checked_add(report.path.len()))
        .and_then(|size| size.checked_add(report.content_sha256.len()))
        .and_then(|size| size.checked_add(256))
        .ok_or_else(|| error(LocalReferencesErrorCode::Limit))?;
    let mut charged_work = work;
    charge_work(&mut charged_work, output_work, report.limits.max_work)?;
    let mut identity = serde_json::to_value(&report)
        .map_err(|_| error(LocalReferencesErrorCode::Canonicalization))?;
    identity
        .as_object_mut()
        .ok_or_else(|| error(LocalReferencesErrorCode::Canonicalization))?
        .remove("analysis_id");
    report.analysis_id = crate::references::canonical_id("local-references:sha256:", &identity)
        .map_err(|_| error(LocalReferencesErrorCode::Canonicalization))?
        .into();
    checkpoint(stop)?;
    Ok(report)
}

fn native_span(
    file: &LuaWorkspaceFile,
    range: TextRange,
    expected_text: Option<&str>,
) -> LocalReferencesResult<SourceSpan> {
    let span = crate::references::ast_span(file, range)
        .map_err(|_| error(LocalReferencesErrorCode::Analyzer))?;
    let start = usize::try_from(
        span.byte_start()
            .ok_or_else(|| error(LocalReferencesErrorCode::Analyzer))?,
    )
    .map_err(|_| error(LocalReferencesErrorCode::Analyzer))?;
    let end = usize::try_from(
        span.byte_end()
            .ok_or_else(|| error(LocalReferencesErrorCode::Analyzer))?,
    )
    .map_err(|_| error(LocalReferencesErrorCode::Analyzer))?;
    if start >= end {
        return Err(error(LocalReferencesErrorCode::Analyzer));
    }
    let source_text = file
        .text()
        .get(start..end)
        .ok_or_else(|| error(LocalReferencesErrorCode::Analyzer))?;
    if expected_text.is_some_and(|expected| source_text != expected) {
        return Err(error(LocalReferencesErrorCode::Analyzer));
    }
    Ok(span)
}

#[cfg(test)]
mod tests;
