//! Source-level global-slot accesses from the existing semantic session.
//! This module knows nothing about WoW, TOC declarations or graph identities.
mod aliases;
mod path;
use crate::references::{
    EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallResult, ast_span, canonical_id,
};
use crate::{LuaWorkspaceFile, bindings::SymbolTarget};
pub(crate) use aliases::AliasIndex;
use emmylua_code_analysis::{FileId, LuaSemanticDeclId, SemanticDeclLevel, SemanticModel};
use emmylua_parser::{LuaAstNode, LuaNameExpr};
pub use path::GlobalAccessKey;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use wow_core::SourceSpan;

pub const GLOBAL_ACCESS_PROFILE: &str = "wow-emmy/global-source-access/2";
pub(crate) const MAX_ACCESSES: usize = 65_536;
const MAX_PATH_DEPTH: usize = 64;
const MAX_ALIAS_DEPTH: usize = 16;
const MAX_TEXT_BYTES: usize = 8 * 1024 * 1024;
const MAX_ACCESS_WORK: usize = 2_000_000;

/// One cumulative budget for all files, alias indexing and path expansion in a
/// report. It is not reset for each use or each alias in a long chain.
#[derive(Default)]
pub(crate) struct AccessBudget {
    work: usize,
    text: usize,
}
impl AccessBudget {
    fn step(&mut self) -> EmmyMemberCallResult<()> {
        self.work = self.work.checked_add(1).ok_or_else(budget)?;
        if self.work > MAX_ACCESS_WORK {
            return Err(budget());
        }
        Ok(())
    }
    fn text(&mut self, amount: usize) -> EmmyMemberCallResult<()> {
        self.text = self.text.checked_add(amount).ok_or_else(budget)?;
        if self.text > MAX_TEXT_BYTES {
            return Err(budget());
        }
        Ok(())
    }
}

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

/// A syntactic alias is never a proven runtime reference to the current global
/// value. Reassignment blocks even a Possible association, rather than silently
/// treating the initializer as the binding's value at every use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AliasAccessBlocker {
    ReassignedBinding,
    LocalBindingWrite,
}

/// One exact local declaration and initializer, in root-to-use order. Every hop
/// is in the access's captured source file. The enclosing statement is the
/// source-evidence anchor; the smaller spans remain available for inspection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceAliasHop {
    pub name: String,
    pub declaration_span: SourceSpan,
    pub initializer_span: SourceSpan,
    pub statement_span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_reassignment: Option<SourceSpan>,
}
impl SourceAliasHop {
    fn valid(&self) -> bool {
        !self.name.is_empty()
            && self.name.len() <= 1024
            && contains(self.statement_span, self.declaration_span)
            && contains(self.statement_span, self.initializer_span)
            && self.declaration_span.byte_end() <= self.initializer_span.byte_start()
            && self.first_reassignment.is_none_or(nonempty)
    }
}
fn nonempty(span: SourceSpan) -> bool {
    matches!((span.byte_start(), span.byte_end()), (Some(start), Some(end)) if start < end)
}
fn contains(outer: SourceSpan, inner: SourceSpan) -> bool {
    matches!((outer.byte_start(), outer.byte_end(), inner.byte_start(), inner.byte_end()),
        (Some(a), Some(b), Some(c), Some(d)) if a <= c && c < d && d <= b)
}

/// One maximal index/parenthesis chain. `keys` stops at an unsupported key and
/// never presents that prefix as an exact slot when `path_complete` is false.
/// `root_span` is the physical name at the use (possibly an alias); `root_name`
/// and `declaration` identify the originating global. Alias hops bridge them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceGlobalAccess {
    fact_id: String,
    path: String,
    content_digest: String,
    function_id: String,
    root_name: String,
    root_span: SourceSpan,
    span: SourceSpan,
    keys: Vec<GlobalAccessKey>,
    path_complete: bool,
    kind: GlobalAccessKind,
    resolution: GlobalAccessResolution,
    aliases: Vec<SourceAliasHop>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alias_blocker: Option<AliasAccessBlocker>,
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
    pub fn keys(&self) -> &[GlobalAccessKey] {
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
    pub fn aliases(&self) -> &[SourceAliasHop] {
        &self.aliases
    }
    pub fn is_alias(&self) -> bool {
        !self.aliases.is_empty()
    }
    pub const fn alias_blocker(&self) -> Option<AliasAccessBlocker> {
        self.alias_blocker
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
                (&self.aliases, self.alias_blocker.as_slice()),
            ),
        )
    }
    pub(crate) fn validate(&self, main: &str) -> EmmyMemberCallResult<()> {
        if self.identity(main)? != self.fact_id
            || self.keys.len() > MAX_PATH_DEPTH
            || self.root_name.len() > 1024
            || self.root_name.is_empty()
            || self.keys.iter().any(|key| !key.valid())
            || self.aliases.len() > MAX_ALIAS_DEPTH
            || self.aliases.iter().any(|hop| !hop.valid())
            || !contains(self.span, self.root_span)
            || (self.resolution == GlobalAccessResolution::Unresolved) != self.declaration.is_none()
            || (!self.is_alias() && self.alias_blocker.is_some())
        {
            return Err(invalid());
        }
        let reassigned = self
            .aliases
            .iter()
            .any(|hop| hop.first_reassignment.is_some());
        match self.alias_blocker {
            Some(AliasAccessBlocker::ReassignedBinding) if !reassigned => return Err(invalid()),
            Some(AliasAccessBlocker::LocalBindingWrite) if self.kind == GlobalAccessKind::Read => {
                return Err(invalid());
            }
            None if reassigned => return Err(invalid()),
            _ => {}
        }
        let mut declarations = HashSet::new();
        for hop in &self.aliases {
            if !declarations.insert(hop.declaration_span) {
                return Err(invalid());
            }
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

/// Follow only exact lexical declaration IDs and rooted-path initializers. No
/// name-based alias table, constant propagation, call execution or heap model.
pub(crate) fn collect_one(
    model: &SemanticModel<'_>,
    main: &str,
    file: &LuaWorkspaceFile,
    name: &LuaNameExpr,
    sources: &HashMap<FileId, AccessSource<'_>>,
    aliases: &AliasIndex,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<Option<SourceGlobalAccess>> {
    let (current, use_path) = path::maximal(name, limits)?;
    let kind = path::access_kind(&current);
    let local_binding_write =
        use_path.keys.is_empty() && use_path.complete && kind != GlobalAccessKind::Read;
    let mut root = use_path.root;
    let mut keys = use_path.keys;
    let mut complete = use_path.complete;
    let mut hops = Vec::new();
    let mut visited = HashSet::new();
    let mut alias_blocker = None;
    let (root_name, resolution, declaration) = loop {
        limits.step()?;
        let spelling = root.get_name_text().ok_or_else(invalid)?;
        if spelling.is_empty() || spelling.len() > 1024 {
            return Err(budget());
        }
        let found = model.find_decl(root.syntax().clone().into(), SemanticDeclLevel::default());
        let Some(LuaSemanticDeclId::LuaDecl(id)) = found else {
            break (spelling, GlobalAccessResolution::Unresolved, None);
        };
        let declaration = model
            .get_db()
            .get_decl_index()
            .get_decl(&id)
            .ok_or_else(invalid)?;
        if declaration.is_global() {
            if declaration.get_name() != spelling {
                return Err(invalid());
            }
            let Some(source) = sources.get(&id.file_id) else {
                break (spelling, GlobalAccessResolution::Unresolved, None);
            };
            let target = SymbolTarget {
                workspace_id: source.workspace_id.into(),
                role: if source.is_main { "main" } else { "library" },
                path: source.file.path().into(),
                content_digest: source.file.content_sha256().into(),
                span: ast_span(source.file, declaration.get_range())?,
            };
            break (
                spelling,
                if source.is_main {
                    GlobalAccessResolution::MainGlobal
                } else {
                    GlobalAccessResolution::LibraryGlobal
                },
                Some(target),
            );
        }
        let Some(candidate) = aliases.get(&id) else {
            return Ok(None);
        };
        if !visited.insert(candidate.hop.declaration_span) {
            return Err(invalid());
        }
        if hops.len() >= MAX_ALIAS_DEPTH {
            return Err(budget());
        }
        limits.text(
            candidate.hop.name.len()
                + candidate
                    .path
                    .keys
                    .iter()
                    .map(GlobalAccessKey::text_bytes)
                    .sum::<usize>()
                + 256,
        )?;
        let mut hop = candidate.hop.clone();
        hop.first_reassignment = aliases.first_write(&id);
        if hop.first_reassignment.is_some() {
            alias_blocker = Some(AliasAccessBlocker::ReassignedBinding);
        }
        hops.push(hop);
        if candidate.path.complete {
            if candidate.path.keys.len() + keys.len() > MAX_PATH_DEPTH {
                return Err(budget());
            }
            let mut prefix = candidate.path.keys.clone();
            prefix.append(&mut keys);
            keys = prefix;
        } else {
            keys = candidate.path.keys.clone();
            complete = false;
        }
        root = candidate.path.root.clone();
    };
    // A local `alias = value` rebinds the cell, not the originating state slot.
    if !hops.is_empty() && local_binding_write {
        alias_blocker = Some(AliasAccessBlocker::LocalBindingWrite);
    }
    hops.reverse();
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
        aliases: hops,
        alias_blocker,
        declaration,
    };
    limits.text(
        access.path.len()
            + access.root_name.len()
            + access
                .keys
                .iter()
                .map(GlobalAccessKey::text_bytes)
                .sum::<usize>()
            + access
                .declaration
                .as_ref()
                .map_or(0, |d| d.path.len() + d.workspace_id.len())
            + 512,
    )?;
    access.fact_id = access.identity(main)?;
    access.validate(main)?;
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
        "global access and alias projection exceeds its bounded profile",
        None,
    )
}
