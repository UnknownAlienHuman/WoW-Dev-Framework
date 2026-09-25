//! Source-level global-slot accesses from the existing semantic session.
//! This module knows nothing about WoW, TOC declarations or graph identities.
use crate::references::{
    EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallResult, ast_span, canonical_id,
};
use crate::{LuaWorkspaceFile, bindings::SymbolTarget};
use emmylua_code_analysis::{FileId, LuaSemanticDeclId, SemanticDeclLevel, SemanticModel};
use emmylua_parser::{
    LuaAssignStat, LuaAstNode, LuaFuncStat, LuaIndexExpr, LuaIndexKey, LuaNameExpr,
};
use serde::Serialize;
use std::collections::HashMap;
use wow_core::SourceSpan;

pub const GLOBAL_ACCESS_PROFILE: &str = "wow-emmy/global-source-access/1";
pub(crate) const MAX_ACCESSES: usize = 65_536;
const MAX_PATH_DEPTH: usize = 64;
const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalAccessKind {
    Read,
    Write,
    UnsupportedAssignment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlobalAccessResolution {
    MainGlobal,
    LibraryGlobal,
    Unresolved,
}

/// One maximal, contiguous index chain rooted at an unshadowed global name.
/// `keys` stops at the first unsupported/dynamic key. An incomplete chain is
/// never presented as an exact access to that prefix's storage slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceGlobalAccess {
    fact_id: String,
    path: String,
    content_digest: String,
    function_id: String,
    root_name: String,
    root_span: SourceSpan,
    span: SourceSpan,
    keys: Vec<String>,
    path_complete: bool,
    kind: GlobalAccessKind,
    resolution: GlobalAccessResolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    declaration: Option<SymbolTarget>,
}
impl SourceGlobalAccess {
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub fn function_id(&self) -> &str {
        &self.function_id
    }
    pub fn root_name(&self) -> &str {
        &self.root_name
    }
    pub const fn root_span(&self) -> SourceSpan {
        self.root_span
    }
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
    pub fn keys(&self) -> &[String] {
        &self.keys
    }
    pub const fn path_complete(&self) -> bool {
        self.path_complete
    }
    pub const fn kind(&self) -> GlobalAccessKind {
        self.kind
    }
    pub const fn resolution(&self) -> GlobalAccessResolution {
        self.resolution
    }
    pub fn declaration(&self) -> Option<&SymbolTarget> {
        self.declaration.as_ref()
    }
    fn identity(&self, main: &str) -> EmmyMemberCallResult<String> {
        canonical_id(
            "emmy-global-access:sha256:",
            &(
                GLOBAL_ACCESS_PROFILE,
                main,
                (&self.path, &self.content_digest, &self.function_id),
                (&self.root_name, self.root_span, self.span),
                (
                    &self.keys,
                    self.path_complete,
                    self.kind,
                    self.resolution,
                    self.declaration.as_slice(),
                ),
            ),
        )
    }
    pub(crate) fn validate(&self, main: &str) -> EmmyMemberCallResult<()> {
        if self.identity(main)? != self.fact_id
            || self.keys.len() > MAX_PATH_DEPTH
            || self.root_name.len() > 1024
            || self.root_name.is_empty()
            || self.keys.iter().any(|k| k.len() > 1024)
            || (self.resolution == GlobalAccessResolution::Unresolved) != self.declaration.is_none()
        {
            return Err(invalid());
        }
        let (Some(root_start), Some(root_end), Some(start), Some(end)) = (
            self.root_span.byte_start(),
            self.root_span.byte_end(),
            self.span.byte_start(),
            self.span.byte_end(),
        ) else {
            return Err(invalid());
        };
        if start > root_start || root_end > end || root_start >= root_end {
            return Err(invalid());
        }
        if let Some(target) = &self.declaration
            && (self.resolution == GlobalAccessResolution::MainGlobal
                && (target.role != "main" || target.workspace_id != main)
                || self.resolution == GlobalAccessResolution::LibraryGlobal
                    && target.role != "library")
        {
            return Err(invalid());
        }
        Ok(())
    }
}

pub(crate) struct AccessSource<'a> {
    pub workspace_id: &'a str,
    pub file: &'a LuaWorkspaceFile,
    pub is_main: bool,
}

/// Returns None for an analyzer-resolved local/parameter/implicit-self. Name
/// spelling alone cannot convert a shadowing binding into a global access.
pub(crate) fn collect_one(
    model: &SemanticModel<'_>,
    main: &str,
    file: &LuaWorkspaceFile,
    name: &LuaNameExpr,
    sources: &HashMap<FileId, AccessSource<'_>>,
    text_bytes: &mut usize,
) -> EmmyMemberCallResult<Option<SourceGlobalAccess>> {
    let root_name = name.get_name_text().ok_or_else(invalid)?.to_string();
    if root_name.len() > 1024 {
        return Err(budget());
    }
    let (resolution, declaration) =
        match model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default()) {
            Some(LuaSemanticDeclId::LuaDecl(id)) => {
                let declaration = model
                    .get_db()
                    .get_decl_index()
                    .get_decl(&id)
                    .ok_or_else(invalid)?;
                if !declaration.is_global() {
                    return Ok(None);
                }
                if declaration.get_name() != root_name {
                    return Err(invalid());
                }
                let Some(source) = sources.get(&id.file_id) else {
                    return Ok(None);
                };
                let target = SymbolTarget {
                    workspace_id: source.workspace_id.into(),
                    role: if source.is_main { "main" } else { "library" },
                    path: source.file.path().into(),
                    content_digest: source.file.content_sha256().into(),
                    span: ast_span(source.file, declaration.get_range())?,
                };
                (
                    if source.is_main {
                        GlobalAccessResolution::MainGlobal
                    } else {
                        GlobalAccessResolution::LibraryGlobal
                    },
                    Some(target),
                )
            }
            Some(_) | None => (GlobalAccessResolution::Unresolved, None),
        };
    let mut current = name.syntax().clone();
    let mut keys = Vec::new();
    let mut complete = true;
    let mut depth = 0;
    while let Some(index) = current.parent().and_then(LuaIndexExpr::cast) {
        if index
            .get_prefix_expr()
            .is_none_or(|prefix| prefix.syntax() != &current)
        {
            break;
        }
        depth += 1;
        if depth > MAX_PATH_DEPTH {
            return Err(budget());
        }
        if complete {
            let key = match index.get_index_key() {
                Some(LuaIndexKey::Name(token)) => Some(token.get_name_text().to_string()),
                Some(LuaIndexKey::String(token)) => Some(token.get_value()),
                _ => None,
            };
            if let Some(key) = key {
                if key.len() > 1024 {
                    return Err(budget());
                }
                keys.push(key);
            } else {
                complete = false;
            }
        }
        current = index.syntax().clone();
    }
    let mut kind = GlobalAccessKind::Read;
    if let Some(parent) = current.parent() {
        if let Some(assignment) = LuaAssignStat::cast(parent.clone()) {
            let (vars, _) = assignment.get_var_and_expr_list();
            if vars.iter().any(|var| var.syntax() == &current) {
                kind = if assignment.get_assign_op().is_some_and(|op| {
                    use emmylua_parser::LuaAstToken;
                    op.syntax().text() == "="
                }) {
                    GlobalAccessKind::Write
                } else {
                    GlobalAccessKind::UnsupportedAssignment
                };
            }
        } else if let Some(statement) = LuaFuncStat::cast(parent)
            && statement
                .get_func_name()
                .is_some_and(|var| var.syntax() == &current)
        {
            kind = GlobalAccessKind::Write;
        }
    }
    let mut access = SourceGlobalAccess {
        fact_id: String::new(),
        path: file.path().into(),
        content_digest: file.content_sha256().into(),
        function_id: crate::function_calls::caller(main, file, name.syntax())?,
        root_name,
        root_span: ast_span(file, name.get_range())?,
        span: ast_span(file, current.text_range())?,
        keys,
        path_complete: complete,
        kind,
        resolution,
        declaration,
    };
    let charge = access.path.len()
        + access.root_name.len()
        + access.keys.iter().map(String::len).sum::<usize>()
        + access
            .declaration
            .as_ref()
            .map_or(0, |d| d.path.len() + d.workspace_id.len())
        + 512;
    *text_bytes = text_bytes.checked_add(charge).ok_or_else(budget)?;
    if *text_bytes > MAX_TEXT_BYTES {
        return Err(budget());
    }
    access.fact_id = access.identity(main)?;
    Ok(Some(access))
}
fn invalid() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::SemanticModelUnavailable,
        "global access facts disagree with the captured semantic session",
        None,
    )
}
fn budget() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::FactBudgetExceeded,
        "global access facts exceed their bounded source profile",
        None,
    )
}
