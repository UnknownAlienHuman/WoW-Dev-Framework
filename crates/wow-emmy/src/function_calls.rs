//! Function scopes and call targets from an existing semantic session.
//! A target is an analyzer-observed concrete closure, not a runtime dispatch claim.
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use emmylua_code_analysis::{EmmyLuaAnalysis, LuaSignatureId, LuaType};
use emmylua_parser::{LuaAst, LuaAstNode, LuaClosureExpr};
use serde::Serialize;
use wow_core::SourceSpan;

use crate::bindings::{SymbolTarget, checkpoint};
use crate::references::{
    EmmyMemberCallError, EmmyMemberCallErrorCode, EmmyMemberCallResult, ast_span, canonical_id,
    semantic_model,
};
use crate::{EmmyBackendIdentity, LuaWorkspaceFile, LuaWorkspaceSnapshot};

pub const FUNCTION_CALL_PROFILE: &str = "wow-emmy/function-call-facts/2";
// Existing callable/call occurrence keys keep their original recipe. The new
// named-target sidecar changes report identity, not the meaning of an old key.
const OCCURRENCE_PROFILE: &str = "wow-emmy/function-call-facts/1";
const MAX_FUNCTIONS: usize = 65_536;
const MAX_CALLS: usize = 65_536;
const MAX_AST_VISITS: usize = 2_000_000;
const MAX_SCOPE_DEPTH: usize = 256;
const MAX_REPORT_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionKind {
    Chunk,
    Closure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionFact {
    fact_id: String,
    path: String,
    content_digest: String,
    kind: SourceFunctionKind,
    span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_function_id: Option<String>,
}
impl SourceFunctionFact {
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub const fn kind(&self) -> SourceFunctionKind {
        self.kind
    }
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
    pub fn parent_function_id(&self) -> Option<&str> {
        self.parent_function_id.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceCallTarget {
    MainFunction {
        function_id: String,
    },
    LibraryFunction {
        target: SymbolTarget,
    },
    /// A signature need not identify an executable closure (for example a doc type).
    SignatureNotCaptured,
    /// Any/unknown/error: no fabricated endpoint and no negative claim.
    Unresolved,
    /// Union, callable object, documented function or another non-single-signature type.
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallFact {
    fact_id: String,
    path: String,
    content_digest: String,
    caller_function_id: String,
    call_span: SourceSpan,
    callee_span: SourceSpan,
    colon_call: bool,
    target: SourceCallTarget,
}
impl SourceCallFact {
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn content_digest(&self) -> &str {
        &self.content_digest
    }
    pub fn caller_function_id(&self) -> &str {
        &self.caller_function_id
    }
    pub const fn call_span(&self) -> SourceSpan {
        self.call_span
    }
    pub const fn callee_span(&self) -> SourceSpan {
        self.callee_span
    }
    pub fn target(&self) -> &SourceCallTarget {
        &self.target
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionFile {
    pub path: String,
    pub content_digest: String,
    pub parse_error_count: usize,
    pub function_count: usize,
    pub call_count: usize,
}

/// Immutable independently identified sidecar; legacy member-call bytes stay unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FunctionCallReport {
    profile: &'static str,
    backend: EmmyBackendIdentity,
    main_snapshot_id: String,
    library_snapshot_ids: Vec<String>,
    files: Vec<SourceFunctionFile>,
    functions: Vec<SourceFunctionFact>,
    calls: Vec<SourceCallFact>,
    /// Only unique full-path lookups with a concrete signature enter this map.
    /// Missing names are not proven non-callable; inspect the symbol report.
    named_targets: BTreeMap<String, SourceCallTarget>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbol_lookup_analysis_id: Option<String>,
    analysis_id: String,
}
impl FunctionCallReport {
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }
    pub fn library_snapshot_ids(&self) -> &[String] {
        &self.library_snapshot_ids
    }
    pub fn files(&self) -> &[SourceFunctionFile] {
        &self.files
    }
    pub fn functions(&self) -> &[SourceFunctionFact] {
        &self.functions
    }
    pub fn calls(&self) -> &[SourceCallFact] {
        &self.calls
    }
    pub fn named_targets(&self) -> &BTreeMap<String, SourceCallTarget> {
        &self.named_targets
    }
    pub fn symbol_lookup_analysis_id(&self) -> Option<&str> {
        self.symbol_lookup_analysis_id.as_deref()
    }
    pub fn source_health_complete(&self) -> bool {
        self.files.iter().all(|file| file.parse_error_count == 0)
    }
    fn identity(&self) -> EmmyMemberCallResult<String> {
        canonical_id(
            "emmy-function-calls:sha256:",
            &(
                self.profile,
                &self.backend,
                &self.main_snapshot_id,
                &self.library_snapshot_ids,
                &self.files,
                &self.functions,
                &self.calls,
                &self.named_targets,
                &self.symbol_lookup_analysis_id,
            ),
        )
    }
    pub fn validate(&self) -> EmmyMemberCallResult<()> {
        if self.profile != FUNCTION_CALL_PROFILE
            || self.identity()? != self.analysis_id
            || self.functions.len() > MAX_FUNCTIONS
            || self.calls.len() > MAX_CALLS
            || self
                .functions
                .windows(2)
                .any(|w| w[0].fact_id >= w[1].fact_id)
            || self.calls.windows(2).any(|w| w[0].fact_id >= w[1].fact_id)
            || self.files.windows(2).any(|w| w[0].path >= w[1].path)
        {
            return Err(invalid());
        }
        let functions = self
            .functions
            .iter()
            .map(|f| (f.fact_id(), f))
            .collect::<BTreeMap<_, _>>();
        let files = self
            .files
            .iter()
            .map(|f| (f.path.as_str(), f))
            .collect::<BTreeMap<_, _>>();
        for function in &self.functions {
            let file = files.get(function.path()).ok_or_else(invalid)?;
            if file.content_digest != function.content_digest || file.parse_error_count != 0 {
                return Err(invalid());
            }
            if let Some(parent) = function.parent_function_id() {
                let parent = functions.get(parent).ok_or_else(invalid)?;
                if parent.path != function.path || parent.fact_id == function.fact_id {
                    return Err(invalid());
                }
            }
        }
        for call in &self.calls {
            let caller = functions
                .get(call.caller_function_id())
                .ok_or_else(invalid)?;
            if caller.path != call.path || caller.content_digest != call.content_digest {
                return Err(invalid());
            }
            if let SourceCallTarget::MainFunction { function_id } = &call.target
                && functions
                    .get(function_id.as_str())
                    .is_none_or(|f| f.kind != SourceFunctionKind::Closure)
            {
                return Err(invalid());
            }
        }
        if self.named_targets.len() > 4096
            || (!self.named_targets.is_empty() && self.symbol_lookup_analysis_id.is_none())
        {
            return Err(invalid());
        }
        for (query, target) in &self.named_targets {
            if !crate::bindings::supported_path(query) {
                return Err(invalid());
            }
            match target {
                SourceCallTarget::MainFunction { function_id } => {
                    if functions
                        .get(function_id.as_str())
                        .is_none_or(|f| f.kind != SourceFunctionKind::Closure)
                    {
                        return Err(invalid());
                    }
                }
                SourceCallTarget::LibraryFunction { target } => {
                    if target.role != "library"
                        || !self.library_snapshot_ids.contains(&target.workspace_id)
                    {
                        return Err(invalid());
                    }
                }
                SourceCallTarget::SignatureNotCaptured => {}
                _ => return Err(invalid()),
            }
        }
        Ok(())
    }
}

fn invalid() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::SemanticModelUnavailable,
        "function-call facts disagree with the captured semantic session",
        None,
    )
}
fn budget() -> EmmyMemberCallError {
    EmmyMemberCallError::new(
        EmmyMemberCallErrorCode::FactBudgetExceeded,
        "function-call facts exceed their bounded profile",
        None,
    )
}
fn visit(count: &mut usize) -> EmmyMemberCallResult<()> {
    *count = count.checked_add(1).ok_or_else(budget)?;
    if *count > MAX_AST_VISITS {
        return Err(budget());
    }
    Ok(())
}
fn function_id(
    workspace: &str,
    file: &LuaWorkspaceFile,
    kind: SourceFunctionKind,
    span: SourceSpan,
) -> EmmyMemberCallResult<String> {
    canonical_id(
        "emmy-function:sha256:",
        &(
            OCCURRENCE_PROFILE,
            workspace,
            file.path(),
            file.content_sha256(),
            kind,
            span,
        ),
    )
}
fn caller(
    workspace: &str,
    file: &LuaWorkspaceFile,
    syntax: &emmylua_parser::LuaSyntaxNode,
) -> EmmyMemberCallResult<String> {
    // Ancestors are bounded independently; no recursive walk or source parsing.
    for (depth, ancestor) in syntax.ancestors().skip(1).enumerate() {
        if depth >= MAX_SCOPE_DEPTH {
            return Err(budget());
        }
        if let Some(closure) = LuaClosureExpr::cast(ancestor) {
            return function_id(
                workspace,
                file,
                SourceFunctionKind::Closure,
                ast_span(file, closure.get_range())?,
            );
        }
    }
    function_id(
        workspace,
        file,
        SourceFunctionKind::Chunk,
        SourceSpan::whole_file(),
    )
}

enum CapturedFunction {
    Main(String),
    Library(SymbolTarget),
}

pub(crate) fn collect(
    analysis: &EmmyLuaAnalysis,
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    libraries: &[(&LuaWorkspaceSnapshot, PathBuf)],
    callable_signatures: &BTreeMap<String, LuaSignatureId>,
    symbol_lookup_analysis_id: Option<&str>,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<FunctionCallReport> {
    let mut signatures = HashMap::new();
    let mut functions = Vec::new();
    let mut files = BTreeMap::new();
    let mut visits = 0usize;
    // Capture actual closure positions first, so forward/cross-file calls never
    // depend on traversal order. Library closures remain foreign targets only.
    for (workspace, root, is_main) in std::iter::once((main, main_root, true))
        .chain(libraries.iter().map(|(w, r)| (*w, r.as_path(), false)))
    {
        for file in workspace.files() {
            checkpoint(stop)?;
            let model = semantic_model(analysis, root, file)?;
            let errors = model
                .get_file_parse_error()
                .map_or(0, |errors| errors.len());
            if is_main {
                files.insert(
                    file.path().to_owned(),
                    SourceFunctionFile {
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        parse_error_count: errors,
                        function_count: 0,
                        call_count: 0,
                    },
                );
            }
            if errors != 0 {
                continue;
            }
            if is_main {
                if functions.len() >= MAX_FUNCTIONS {
                    return Err(budget());
                }
                functions.push(SourceFunctionFact {
                    fact_id: function_id(
                        workspace.snapshot_id(),
                        file,
                        SourceFunctionKind::Chunk,
                        SourceSpan::whole_file(),
                    )?,
                    path: file.path().into(),
                    content_digest: file.content_sha256().into(),
                    kind: SourceFunctionKind::Chunk,
                    span: SourceSpan::whole_file(),
                    parent_function_id: None,
                });
                files
                    .get_mut(file.path())
                    .ok_or_else(invalid)?
                    .function_count += 1;
            }
            for ast in model.get_root().descendants::<LuaAst>() {
                checkpoint(stop)?;
                visit(&mut visits)?;
                let LuaAst::LuaClosureExpr(closure) = ast else {
                    continue;
                };
                if signatures.len() >= MAX_FUNCTIONS || functions.len() >= MAX_FUNCTIONS {
                    return Err(budget());
                }
                let span = ast_span(file, closure.get_range())?;
                let signature = LuaSignatureId::from_closure(model.get_file_id(), &closure);
                let captured = if is_main {
                    let id = function_id(
                        workspace.snapshot_id(),
                        file,
                        SourceFunctionKind::Closure,
                        span,
                    )?;
                    functions.push(SourceFunctionFact {
                        fact_id: id.clone(),
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        kind: SourceFunctionKind::Closure,
                        span,
                        parent_function_id: Some(caller(
                            workspace.snapshot_id(),
                            file,
                            closure.syntax(),
                        )?),
                    });
                    files
                        .get_mut(file.path())
                        .ok_or_else(invalid)?
                        .function_count += 1;
                    CapturedFunction::Main(id)
                } else {
                    CapturedFunction::Library(SymbolTarget {
                        workspace_id: workspace.snapshot_id().into(),
                        role: "library",
                        path: file.path().into(),
                        content_digest: file.content_sha256().into(),
                        span,
                    })
                };
                if signatures.insert(signature, captured).is_some() {
                    return Err(invalid());
                }
            }
        }
    }
    let mut calls = Vec::new();
    for file in main.files() {
        checkpoint(stop)?;
        if files
            .get(file.path())
            .ok_or_else(invalid)?
            .parse_error_count
            != 0
        {
            continue;
        }
        let model = semantic_model(analysis, main_root, file)?;
        for ast in model.get_root().descendants::<LuaAst>() {
            checkpoint(stop)?;
            visit(&mut visits)?;
            let LuaAst::LuaCallExpr(call) = ast else {
                continue;
            };
            if calls.len() >= MAX_CALLS {
                return Err(budget());
            }
            let prefix = call.get_prefix_expr().ok_or_else(invalid)?;
            let call_span = ast_span(file, call.get_range())?;
            let callee_span = ast_span(file, prefix.syntax().text_range())?;
            let target = match model.infer_expr(prefix) {
                Ok(LuaType::Signature(id)) => match signatures.get(&id) {
                    Some(CapturedFunction::Main(id)) => SourceCallTarget::MainFunction {
                        function_id: id.clone(),
                    },
                    Some(CapturedFunction::Library(target)) => SourceCallTarget::LibraryFunction {
                        target: target.clone(),
                    },
                    None => SourceCallTarget::SignatureNotCaptured,
                },
                Ok(LuaType::Unknown | LuaType::Any) | Err(_) => SourceCallTarget::Unresolved,
                _ => SourceCallTarget::Indeterminate,
            };
            let caller_function_id = caller(main.snapshot_id(), file, call.syntax())?;
            let colon_call = call.is_colon_call();
            let fact_id = canonical_id(
                "emmy-source-call:sha256:",
                &(
                    OCCURRENCE_PROFILE,
                    main.snapshot_id(),
                    file.path(),
                    file.content_sha256(),
                    &caller_function_id,
                    call_span,
                    callee_span,
                    colon_call,
                    &target,
                ),
            )?;
            calls.push(SourceCallFact {
                fact_id,
                path: file.path().into(),
                content_digest: file.content_sha256().into(),
                caller_function_id,
                call_span,
                callee_span,
                colon_call,
                target,
            });
            files.get_mut(file.path()).ok_or_else(invalid)?.call_count += 1;
        }
    }
    functions.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    calls.sort_by(|a, b| a.fact_id.cmp(&b.fact_id));
    let library_snapshot_ids = libraries
        .iter()
        .map(|(w, _)| w.snapshot_id().to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut named_targets = BTreeMap::new();
    for (query, signature) in callable_signatures {
        checkpoint(stop)?;
        let target = match signatures.get(signature) {
            Some(CapturedFunction::Main(id)) => SourceCallTarget::MainFunction {
                function_id: id.clone(),
            },
            Some(CapturedFunction::Library(target)) => SourceCallTarget::LibraryFunction {
                target: target.clone(),
            },
            None => SourceCallTarget::SignatureNotCaptured,
        };
        named_targets.insert(query.clone(), target);
    }
    let mut report = FunctionCallReport {
        profile: FUNCTION_CALL_PROFILE,
        backend: main.backend().clone(),
        main_snapshot_id: main.snapshot_id().into(),
        library_snapshot_ids,
        files: files.into_values().collect(),
        functions,
        calls,
        named_targets,
        symbol_lookup_analysis_id: symbol_lookup_analysis_id.map(str::to_owned),
        analysis_id: String::new(),
    };
    struct Count(usize);
    impl std::io::Write for Count {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > MAX_REPORT_BYTES.saturating_sub(self.0) {
                return Err(std::io::Error::other("function-call report limit"));
            }
            self.0 += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Count(128), &report).map_err(|_| budget())?;
    checkpoint(stop)?;
    report.analysis_id = report.identity()?;
    report.validate()?;
    checkpoint(stop)?;
    Ok(report)
}
