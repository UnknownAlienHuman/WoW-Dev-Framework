//! Exact lexical local declarations from the accepted native semantic model.
//! No global/member/call-origin fallback, source loading or transport positions.
use crate::{EmmyBackendIdentity, LuaWorkspaceSnapshot, LuaWorkspaceUniverse};
use emmylua_code_analysis::{LuaSemanticDeclId, SemanticDeclLevel};
use emmylua_parser::{LuaAstNode, LuaTokenKind};
use rowan::{TextSize, TokenAtOffset};
use serde::Serialize;
use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_core::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalDefinitionLimits {
    pub max_file_bytes: usize,
    pub max_work: usize,
    pub max_name_bytes: usize,
}
impl LocalDefinitionLimits {
    fn validate(self) -> Result<(), LocalDefinitionError> {
        if self.max_file_bytes == 0
            || self.max_file_bytes > 16 * 1024 * 1024
            || self.max_work == 0
            || self.max_work > 1024 * 1024 * 1024
            || self.max_name_bytes == 0
            || self.max_name_bytes > 4096
        {
            return Err(error(LocalDefinitionErrorCode::Limit));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalDefinitionState {
    ResolvedLocal,
    NotEvaluated,
    SourceParseFailed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalDefinitionReason {
    UnsupportedToken,
    DeclarationNotObserved,
    OutsideLocalScope,
    TargetUnavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalDeclarationKind {
    LocalBinding,
    Parameter,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalDeclarationTarget {
    pub name: Box<str>,
    pub kind: LocalDeclarationKind,
    pub span: SourceSpan,
}
#[derive(Debug, Clone, Serialize)]
pub struct LocalDefinitionReport {
    pub schema: &'static str,
    pub profile: &'static str,
    pub analysis_id: Box<str>,
    pub workspace_snapshot_id: Box<str>,
    pub backend: EmmyBackendIdentity,
    pub path: Box<str>,
    pub content_sha256: Box<str>,
    pub query_byte_offset: usize,
    pub limits: LocalDefinitionLimits,
    pub state: LocalDefinitionState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<LocalDefinitionReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin_span: Option<SourceSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<LocalDeclarationTarget>,
    pub negative_authority: bool,
    pub coverage: &'static str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalDefinitionErrorCode {
    InvalidWorkspace,
    InvalidPosition,
    Limit,
    Cancelled,
    Analyzer,
    Canonicalization,
}
#[derive(Debug, Clone)]
pub struct LocalDefinitionError {
    code: LocalDefinitionErrorCode,
}
impl LocalDefinitionError {
    pub const fn code(&self) -> LocalDefinitionErrorCode {
        self.code
    }
}
impl fmt::Display for LocalDefinitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "local definition refused ({:?})", self.code)
    }
}
impl std::error::Error for LocalDefinitionError {}
fn error(code: LocalDefinitionErrorCode) -> LocalDefinitionError {
    LocalDefinitionError { code }
}
fn checkpoint(stop: &AtomicBool) -> Result<(), LocalDefinitionError> {
    if stop.load(Ordering::Acquire) {
        Err(error(LocalDefinitionErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
/// One single-file Main input, exact original UTF8 offset and bounded policy.
/// Returned declarations are analyzer observations, never runtime or absence proof.
pub fn local_definition(
    input: &LuaWorkspaceSnapshot,
    byte_offset: usize,
    limits: LocalDefinitionLimits,
    stop: &AtomicBool,
) -> Result<LocalDefinitionReport, LocalDefinitionError> {
    limits.validate()?;
    checkpoint(stop)?;
    if input.files().len() != 1
        || !matches!(
            input.universe(),
            LuaWorkspaceUniverse::Project | LuaWorkspaceUniverse::Fixture
        )
    {
        return Err(error(LocalDefinitionErrorCode::InvalidWorkspace));
    }
    let file = &input.files()[0];
    if file.text().len() > limits.max_file_bytes
        || file
            .text()
            .len()
            .checked_mul(4)
            .is_none_or(|n| n > limits.max_work)
    {
        return Err(error(LocalDefinitionErrorCode::Limit));
    }
    if byte_offset > file.text().len() || !file.text().is_char_boundary(byte_offset) {
        return Err(error(LocalDefinitionErrorCode::InvalidPosition));
    }
    let root = crate::syntax::analyzer::virtual_root(input.snapshot_id());
    let analysis = crate::syntax::analyzer::build_registered(input, &root)
        .map_err(|_| error(LocalDefinitionErrorCode::Analyzer))?;
    checkpoint(stop)?;
    let model = crate::references::semantic_model(&analysis, &root, file)
        .map_err(|_| error(LocalDefinitionErrorCode::Analyzer))?;
    let mut report = LocalDefinitionReport {
        schema: "wow-emmy/local-definition-result/1",
        profile: "same_file_lexical_local_no_trace/1",
        analysis_id: "".into(),
        workspace_snapshot_id: input.snapshot_id().into(),
        backend: input.backend().clone(),
        path: file.path().into(),
        content_sha256: file.content_sha256().into(),
        query_byte_offset: byte_offset,
        limits,
        state: LocalDefinitionState::NotEvaluated,
        reason: None,
        origin_span: None,
        target: None,
        negative_authority: false,
        coverage: "one_exact_main_file_lexical_locals_only",
    };
    let tree = model
        .get_db()
        .get_vfs()
        .get_syntax_tree(&model.get_file_id())
        .ok_or_else(|| error(LocalDefinitionErrorCode::Analyzer))?;
    if !tree.get_errors().is_empty() {
        report.state = LocalDefinitionState::SourceParseFailed;
    } else {
        let offset = TextSize::from(
            u32::try_from(byte_offset)
                .map_err(|_| error(LocalDefinitionErrorCode::InvalidPosition))?,
        );
        let ast = model.get_root();
        // Same-boundary left identifier follows the accepted donor's explicit
        // cursor convention. No nearest token or spelling search is performed.
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
            let range = token.text_range();
            let start = u32::from(range.start()) as usize;
            let end = u32::from(range.end()) as usize;
            report.origin_span = Some(span(file.text(), start, end)?);
            match model.find_decl(token.into(), SemanticDeclLevel::NoTrace) {
                Some(LuaSemanticDeclId::LuaDecl(id)) => {
                    if let Some(decl) = model.get_db().get_decl_index().get_decl(&id) {
                        if decl.get_file_id() != model.get_file_id() || !decl.is_local() {
                            report.reason = Some(LocalDefinitionReason::OutsideLocalScope);
                        } else if decl.get_name().len() > limits.max_name_bytes {
                            return Err(error(LocalDefinitionErrorCode::Limit));
                        } else {
                            let range = decl.get_range();
                            let start = u32::from(range.start()) as usize;
                            let end = u32::from(range.end()) as usize;
                            let target_span = span(file.text(), start, end)?;
                            if file.text().get(start..end) != Some(decl.get_name()) {
                                report.reason = Some(LocalDefinitionReason::TargetUnavailable);
                            } else {
                                report.state = LocalDefinitionState::ResolvedLocal;
                                report.target = Some(LocalDeclarationTarget {
                                    name: decl.get_name().into(),
                                    kind: if decl.is_param() {
                                        LocalDeclarationKind::Parameter
                                    } else {
                                        LocalDeclarationKind::LocalBinding
                                    },
                                    span: target_span,
                                });
                            }
                        }
                    } else {
                        report.reason = Some(LocalDefinitionReason::TargetUnavailable);
                    }
                }
                Some(_) => report.reason = Some(LocalDefinitionReason::OutsideLocalScope),
                None => report.reason = Some(LocalDefinitionReason::DeclarationNotObserved),
            }
        } else {
            report.reason = Some(LocalDefinitionReason::UnsupportedToken);
        }
    }
    checkpoint(stop)?;
    let mut identity = serde_json::to_value(&report)
        .map_err(|_| error(LocalDefinitionErrorCode::Canonicalization))?;
    identity
        .as_object_mut()
        .ok_or_else(|| error(LocalDefinitionErrorCode::Canonicalization))?
        .remove("analysis_id");
    report.analysis_id = crate::references::canonical_id("local-definition:sha256:", &identity)
        .map_err(|_| error(LocalDefinitionErrorCode::Canonicalization))?
        .into();
    checkpoint(stop)?;
    Ok(report)
}
fn span(text: &str, start: usize, end: usize) -> Result<SourceSpan, LocalDefinitionError> {
    if start >= end || text.get(start..end).is_none() {
        return Err(error(LocalDefinitionErrorCode::Analyzer));
    }
    SourceSpan::byte_range(start as u64, end as u64)
        .map_err(|_| error(LocalDefinitionErrorCode::Analyzer))
}

#[cfg(test)]
mod tests;
