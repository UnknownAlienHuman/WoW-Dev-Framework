//! Exact local binding, use, operation, guard, and dominance facts from the
//! pinned EmmyLua semantic model.
//!
//! These facts describe source structure and analyzer-observable declaration
//! links only. They do not classify a value as a WoW Secret, prove a guard is
//! sufficient for a live client, or authorize an operation.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use emmylua_code_analysis::{
    EmmyLuaAnalysis, Emmyrc, EmmyrcLuaVersion, LuaSemanticDeclId, SemanticDeclLevel, SemanticModel,
    WorkspaceFolder, file_path_to_uri,
};
use emmylua_parser::{
    BinaryOperator, LuaAst, LuaAstNode, LuaAstToken, LuaExpr, LuaIfStat, LuaLocalName, LuaNameExpr,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{SourceSpan, canonical_json_bytes};

use crate::references::{EmmyMemberCallFact, EmmyMemberReferenceFact, analyze_member_calls};
use crate::syntax::{EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE};
use crate::{LuaWorkspaceFile, LuaWorkspaceSnapshot};

const REPORT_SCHEMA: &str = "wow-emmy/local-flow-facts/1";
const MAX_FACTS: usize = 65_536;
const MAX_NAME_BYTES: usize = 1_024;

/// Stable local-flow analysis failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyLocalFlowErrorCode {
    IncompatibleBackend,
    DuplicateWorkspaceSnapshot,
    InvalidMainWorkspace,
    AnalyzerFileRegistrationFailed,
    SemanticModelUnavailable,
    LibraryHealthFailed,
    MemberCallAnalysisFailed,
    CoordinateConversionFailed,
    FactBudgetExceeded,
    CanonicalizationFailed,
}

/// One bounded local-flow analysis failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmyLocalFlowError {
    code: EmmyLocalFlowErrorCode,
    message: Box<str>,
    path: Option<Box<str>>,
}

impl EmmyLocalFlowError {
    fn new(code: EmmyLocalFlowErrorCode, message: impl Into<Box<str>>, path: Option<&str>) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.map(Into::into),
        }
    }

    /// Stable failure class.
    #[must_use]
    pub const fn code(&self) -> EmmyLocalFlowErrorCode {
        self.code
    }

    /// Safe explanation without host paths or source bodies.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Affected logical path, when one exists.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for EmmyLocalFlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path() {
            Some(path) => write!(formatter, "{} ({path})", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for EmmyLocalFlowError {}

/// Result type for local-flow analysis.
pub type EmmyLocalFlowResult<T> = Result<T, EmmyLocalFlowError>;

/// Per-file fact capability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyLocalFlowFileStatus {
    Complete,
    FailedParse,
}

/// Supported operation kinds in the first bounded local-flow slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyOperationKind {
    Concatenation,
}

/// Supported guard shapes in the first bounded local-flow slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyGuardKind {
    AccessSingle,
}

/// Supported control-flow relations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyControlFlowRelationKind {
    Dominates,
}

/// One exact local declaration and its optional direct-member initializer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyLocalBindingFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    name: Box<str>,
    declaration_span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    initializer_call_fact_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initializer_reference_fact_id: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initializer_receiver: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    initializer_member: Option<Box<str>>,
}

impl EmmyLocalBindingFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }

    #[must_use]
    pub fn initializer_call_fact_id(&self) -> Option<&str> {
        self.initializer_call_fact_id.as_deref()
    }

    #[must_use]
    pub fn initializer_reference_fact_id(&self) -> Option<&str> {
        self.initializer_reference_fact_id.as_deref()
    }

    #[must_use]
    pub fn initializer_receiver(&self) -> Option<&str> {
        self.initializer_receiver.as_deref()
    }

    #[must_use]
    pub fn initializer_member(&self) -> Option<&str> {
        self.initializer_member.as_deref()
    }
}

/// One exact use of a local binding by a supported operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyLocalUseFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    binding_fact_id: Box<str>,
    operation_fact_id: Box<str>,
    name: Box<str>,
    use_span: SourceSpan,
}

impl EmmyLocalUseFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub fn binding_fact_id(&self) -> &str {
        &self.binding_fact_id
    }

    #[must_use]
    pub fn operation_fact_id(&self) -> &str {
        &self.operation_fact_id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn use_span(&self) -> SourceSpan {
        self.use_span
    }
}

/// One supported source operation with exact local operands.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyOperationFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    kind: EmmyOperationKind,
    operation_span: SourceSpan,
    operand_binding_fact_ids: Vec<Box<str>>,
    operand_names: Vec<Box<str>>,
}

impl EmmyOperationFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn kind(&self) -> EmmyOperationKind {
        self.kind
    }

    #[must_use]
    pub const fn operation_span(&self) -> SourceSpan {
        self.operation_span
    }

    pub fn operand_binding_fact_ids(&self) -> impl Iterator<Item = &str> {
        self.operand_binding_fact_ids
            .iter()
            .map(|value| value.as_ref())
    }

    pub fn operand_names(&self) -> impl Iterator<Item = &str> {
        self.operand_names.iter().map(|value| value.as_ref())
    }
}

/// One recognized access guard over one exact local binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyGuardFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    kind: EmmyGuardKind,
    callee: &'static str,
    guarded_binding_fact_id: Box<str>,
    guarded_name: Box<str>,
    guard_span: SourceSpan,
    guarded_block_span: SourceSpan,
}

impl EmmyGuardFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub const fn kind(&self) -> EmmyGuardKind {
        self.kind
    }

    #[must_use]
    pub fn guarded_binding_fact_id(&self) -> &str {
        &self.guarded_binding_fact_id
    }

    #[must_use]
    pub fn guarded_name(&self) -> &str {
        &self.guarded_name
    }

    #[must_use]
    pub const fn guard_span(&self) -> SourceSpan {
        self.guard_span
    }

    #[must_use]
    pub const fn guarded_block_span(&self) -> SourceSpan {
        self.guarded_block_span
    }
}

/// One exact relation between a recognized guard and a supported operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyControlFlowFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    relation: EmmyControlFlowRelationKind,
    guard_fact_id: Box<str>,
    operation_fact_id: Box<str>,
    binding_fact_id: Box<str>,
}

impl EmmyControlFlowFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub const fn relation(&self) -> EmmyControlFlowRelationKind {
        self.relation
    }

    #[must_use]
    pub fn guard_fact_id(&self) -> &str {
        &self.guard_fact_id
    }

    #[must_use]
    pub fn operation_fact_id(&self) -> &str {
        &self.operation_fact_id
    }

    #[must_use]
    pub fn binding_fact_id(&self) -> &str {
        &self.binding_fact_id
    }
}

/// Coverage and fact counts for one Main file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyLocalFlowFileReport {
    path: Box<str>,
    content_sha256: Box<str>,
    status: EmmyLocalFlowFileStatus,
    parse_error_count: u64,
    binding_count: u64,
    use_count: u64,
    operation_count: u64,
    guard_count: u64,
    control_flow_count: u64,
}

impl EmmyLocalFlowFileReport {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn status(&self) -> EmmyLocalFlowFileStatus {
        self.status
    }

    #[must_use]
    pub const fn parse_error_count(&self) -> u64 {
        self.parse_error_count
    }

    #[must_use]
    pub const fn binding_count(&self) -> u64 {
        self.binding_count
    }

    #[must_use]
    pub const fn use_count(&self) -> u64 {
        self.use_count
    }

    #[must_use]
    pub const fn operation_count(&self) -> u64 {
        self.operation_count
    }

    #[must_use]
    pub const fn guard_count(&self) -> u64 {
        self.guard_count
    }

    #[must_use]
    pub const fn control_flow_count(&self) -> u64 {
        self.control_flow_count
    }
}

/// Deterministic local-flow fact report for one Main snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyLocalFlowReport {
    schema: &'static str,
    analysis_id: Box<str>,
    upstream_revision: &'static str,
    upstream_tree: &'static str,
    upstream_crate_version: &'static str,
    main_snapshot_id: Box<str>,
    library_snapshot_ids: Vec<Box<str>>,
    member_call_analysis_id: Box<str>,
    files: Vec<EmmyLocalFlowFileReport>,
    bindings: Vec<EmmyLocalBindingFact>,
    uses: Vec<EmmyLocalUseFact>,
    operations: Vec<EmmyOperationFact>,
    guards: Vec<EmmyGuardFact>,
    control_flow: Vec<EmmyControlFlowFact>,
}

impl EmmyLocalFlowReport {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }

    #[must_use]
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }

    #[must_use]
    pub fn member_call_analysis_id(&self) -> &str {
        &self.member_call_analysis_id
    }

    #[must_use]
    pub fn files(&self) -> &[EmmyLocalFlowFileReport] {
        &self.files
    }

    #[must_use]
    pub fn bindings(&self) -> &[EmmyLocalBindingFact] {
        &self.bindings
    }

    #[must_use]
    pub fn uses(&self) -> &[EmmyLocalUseFact] {
        &self.uses
    }

    #[must_use]
    pub fn operations(&self) -> &[EmmyOperationFact] {
        &self.operations
    }

    #[must_use]
    pub fn guards(&self) -> &[EmmyGuardFact] {
        &self.guards
    }

    #[must_use]
    pub fn control_flow(&self) -> &[EmmyControlFlowFact] {
        &self.control_flow
    }
}

#[derive(Clone)]
struct BindingInternal {
    semantic_decl: LuaSemanticDeclId,
    fact: EmmyLocalBindingFact,
}

#[derive(Clone)]
struct OperationInternal {
    fact: EmmyOperationFact,
    uses: Vec<EmmyLocalUseFact>,
}

/// Extracts the first bounded local-flow fact surface from exact Main and
/// Library snapshots.
pub fn analyze_local_flow(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
) -> EmmyLocalFlowResult<EmmyLocalFlowReport> {
    validate_compiled_backend(main)?;
    if matches!(main.universe(), crate::LuaWorkspaceUniverse::BlizzardUi) {
        return Err(EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::InvalidMainWorkspace,
            "a Blizzard UI snapshot cannot be used as the Main project workspace",
            None,
        ));
    }

    let member_report = analyze_member_calls(main, libraries).map_err(|error| {
        EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::MemberCallAnalysisFailed,
            format!("direct-member prerequisite failed: {}", error.message()),
            error.path(),
        )
    })?;

    let mut ordered_libraries = libraries.to_vec();
    ordered_libraries.sort_by(|left, right| {
        left.snapshot_id()
            .as_bytes()
            .cmp(right.snapshot_id().as_bytes())
    });
    let mut identities = BTreeSet::new();
    identities.insert(main.snapshot_id());
    for library in &ordered_libraries {
        validate_compiled_backend(library)?;
        if library.backend() != main.backend() {
            return Err(EmmyLocalFlowError::new(
                EmmyLocalFlowErrorCode::IncompatibleBackend,
                "Main and Library snapshots do not share one exact analyzer identity",
                None,
            ));
        }
        if !identities.insert(library.snapshot_id()) {
            return Err(EmmyLocalFlowError::new(
                EmmyLocalFlowErrorCode::DuplicateWorkspaceSnapshot,
                "the same workspace snapshot was supplied in more than one role",
                None,
            ));
        }
    }

    let main_root = virtual_root("main", main.snapshot_id());
    let library_roots = ordered_libraries
        .iter()
        .enumerate()
        .map(|(ordinal, snapshot)| {
            (
                *snapshot,
                virtual_root(&format!("library-{ordinal}"), snapshot.snapshot_id()),
            )
        })
        .collect::<Vec<_>>();
    let analysis = build_analysis(main, &main_root, &library_roots)?;

    for (snapshot, root) in &library_roots {
        for file in snapshot.files() {
            let model = semantic_model(&analysis, root, file)?;
            if model
                .get_file_parse_error()
                .is_some_and(|errors| !errors.is_empty())
            {
                return Err(EmmyLocalFlowError::new(
                    EmmyLocalFlowErrorCode::LibraryHealthFailed,
                    "a Library file has parse errors; local-flow facts are unavailable",
                    Some(file.path()),
                ));
            }
        }
    }

    let mut files = Vec::with_capacity(main.files().len());
    let mut bindings = Vec::new();
    let mut uses = Vec::new();
    let mut operations = Vec::new();
    let mut guards = Vec::new();
    let mut control_flow = Vec::new();

    for file in main.files() {
        let model = semantic_model(&analysis, &main_root, file)?;
        let parse_errors = model.get_file_parse_error().unwrap_or_default();
        if !parse_errors.is_empty() {
            files.push(EmmyLocalFlowFileReport {
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                status: EmmyLocalFlowFileStatus::FailedParse,
                parse_error_count: to_u64(parse_errors.len(), file.path())?,
                binding_count: 0,
                use_count: 0,
                operation_count: 0,
                guard_count: 0,
                control_flow_count: 0,
            });
            continue;
        }

        let binding_start = bindings.len();
        let use_start = uses.len();
        let operation_start = operations.len();
        let guard_start = guards.len();
        let control_start = control_flow.len();

        let root = model.get_root().clone();
        let mut file_bindings = collect_bindings(
            main,
            file,
            &model,
            &root,
            member_report.calls(),
            member_report.references(),
        )?;
        let semantic_bindings = file_bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| (binding.semantic_decl.clone(), index))
            .collect::<HashMap<_, _>>();

        let mut file_operations = collect_operations(
            main,
            file,
            &model,
            &root,
            &file_bindings,
            &semantic_bindings,
        )?;
        let (mut file_guards, mut file_control) = collect_guards(
            main,
            file,
            &model,
            &root,
            &file_bindings,
            &semantic_bindings,
            &file_operations,
        )?;

        bindings.extend(file_bindings.drain(..).map(|binding| binding.fact));
        for operation in file_operations.drain(..) {
            uses.extend(operation.uses);
            operations.push(operation.fact);
        }
        guards.append(&mut file_guards);
        control_flow.append(&mut file_control);

        files.push(EmmyLocalFlowFileReport {
            path: file.path().into(),
            content_sha256: file.content_sha256().into(),
            status: EmmyLocalFlowFileStatus::Complete,
            parse_error_count: 0,
            binding_count: to_u64(bindings.len() - binding_start, file.path())?,
            use_count: to_u64(uses.len() - use_start, file.path())?,
            operation_count: to_u64(operations.len() - operation_start, file.path())?,
            guard_count: to_u64(guards.len() - guard_start, file.path())?,
            control_flow_count: to_u64(control_flow.len() - control_start, file.path())?,
        });
    }

    bindings.sort();
    uses.sort();
    operations.sort();
    guards.sort();
    control_flow.sort();

    let library_snapshot_ids = ordered_libraries
        .iter()
        .map(|snapshot| snapshot.snapshot_id().into())
        .collect::<Vec<Box<str>>>();
    #[derive(Serialize)]
    struct ReportIdentity<'a> {
        schema: &'static str,
        upstream_revision: &'static str,
        upstream_tree: &'static str,
        upstream_crate_version: &'static str,
        main_snapshot_id: &'a str,
        library_snapshot_ids: &'a [Box<str>],
        member_call_analysis_id: &'a str,
        files: &'a [EmmyLocalFlowFileReport],
        bindings: &'a [EmmyLocalBindingFact],
        uses: &'a [EmmyLocalUseFact],
        operations: &'a [EmmyOperationFact],
        guards: &'a [EmmyGuardFact],
        control_flow: &'a [EmmyControlFlowFact],
    }
    let identity = ReportIdentity {
        schema: REPORT_SCHEMA,
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        main_snapshot_id: main.snapshot_id(),
        library_snapshot_ids: &library_snapshot_ids,
        member_call_analysis_id: member_report.analysis_id(),
        files: &files,
        bindings: &bindings,
        uses: &uses,
        operations: &operations,
        guards: &guards,
        control_flow: &control_flow,
    };
    let analysis_id = canonical_id("emmy-local-flow:sha256:", &identity)?;

    Ok(EmmyLocalFlowReport {
        schema: REPORT_SCHEMA,
        analysis_id: analysis_id.into_boxed_str(),
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        main_snapshot_id: main.snapshot_id().into(),
        library_snapshot_ids,
        member_call_analysis_id: member_report.analysis_id().into(),
        files,
        bindings,
        uses,
        operations,
        guards,
        control_flow,
    })
}

fn collect_bindings(
    main: &LuaWorkspaceSnapshot,
    file: &LuaWorkspaceFile,
    model: &SemanticModel<'_>,
    root: &emmylua_parser::LuaChunk,
    calls: &[EmmyMemberCallFact],
    references: &[EmmyMemberReferenceFact],
) -> EmmyLocalFlowResult<Vec<BindingInternal>> {
    let mut result = Vec::new();
    for node in root.descendants::<LuaAst>() {
        let LuaAst::LuaLocalStat(stat) = node else {
            continue;
        };
        let names = stat.get_local_name_list().collect::<Vec<LuaLocalName>>();
        let values = stat.get_value_exprs().collect::<Vec<LuaExpr>>();
        for (index, name) in names.into_iter().enumerate() {
            if result.len() >= MAX_FACTS {
                return Err(fact_budget(file));
            }
            let Some(token) = name.get_name_token() else {
                continue;
            };
            let spelling = token.get_name_text();
            if spelling.is_empty() || spelling.len() > MAX_NAME_BYTES {
                return Err(fact_budget(file));
            }
            let Some(semantic_decl) =
                model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
            else {
                continue;
            };
            let declaration_span = ast_span(file, token.syntax().text_range())?;
            let initializer = values
                .get(index)
                .and_then(|value| direct_member_initializer(file, value, calls, references))
                .transpose()?;
            let (
                initializer_call_fact_id,
                initializer_reference_fact_id,
                initializer_receiver,
                initializer_member,
            ) = initializer
                .map(|(call, reference)| {
                    (
                        Some(Box::<str>::from(call.fact_id())),
                        Some(Box::<str>::from(reference.fact_id())),
                        Some(Box::<str>::from(reference.receiver())),
                        Some(Box::<str>::from(reference.member())),
                    )
                })
                .unwrap_or((None, None, None, None));
            #[derive(Serialize)]
            struct Identity<'a> {
                schema: &'static str,
                snapshot_id: &'a str,
                path: &'a str,
                content_sha256: &'a str,
                name: &'a str,
                declaration_span: SourceSpan,
                #[serde(skip_serializing_if = "Option::is_none")]
                initializer_call_fact_id: Option<&'a str>,
            }
            let fact_id = canonical_id(
                "emmy-local-binding:sha256:",
                &Identity {
                    schema: "wow-emmy/local-binding-fact/1",
                    snapshot_id: main.snapshot_id(),
                    path: file.path(),
                    content_sha256: file.content_sha256(),
                    name: spelling,
                    declaration_span,
                    initializer_call_fact_id: initializer_call_fact_id.as_deref(),
                },
            )?;
            result.push(BindingInternal {
                semantic_decl,
                fact: EmmyLocalBindingFact {
                    fact_id: fact_id.into_boxed_str(),
                    path: file.path().into(),
                    content_sha256: file.content_sha256().into(),
                    name: spelling.into(),
                    declaration_span,
                    initializer_call_fact_id,
                    initializer_reference_fact_id,
                    initializer_receiver,
                    initializer_member,
                },
            });
        }
    }
    result.sort_by(|left, right| left.fact.cmp(&right.fact));
    Ok(result)
}

fn direct_member_initializer<'a>(
    file: &LuaWorkspaceFile,
    value: &LuaExpr,
    calls: &'a [EmmyMemberCallFact],
    references: &'a [EmmyMemberReferenceFact],
) -> Option<EmmyLocalFlowResult<(&'a EmmyMemberCallFact, &'a EmmyMemberReferenceFact)>> {
    let LuaExpr::CallExpr(call_expr) = value else {
        return None;
    };
    let call_span = match ast_span(file, call_expr.syntax().text_range()) {
        Ok(span) => span,
        Err(error) => return Some(Err(error)),
    };
    let call = calls
        .iter()
        .find(|fact| fact.path() == file.path() && fact.call_span() == call_span)?;
    let reference = references
        .iter()
        .find(|fact| fact.fact_id() == call.reference_fact_id())?;
    Some(Ok((call, reference)))
}

fn collect_operations(
    main: &LuaWorkspaceSnapshot,
    file: &LuaWorkspaceFile,
    model: &SemanticModel<'_>,
    root: &emmylua_parser::LuaChunk,
    bindings: &[BindingInternal],
    semantic_bindings: &HashMap<LuaSemanticDeclId, usize>,
) -> EmmyLocalFlowResult<Vec<OperationInternal>> {
    let mut result = Vec::new();
    for node in root.descendants::<LuaAst>() {
        let LuaAst::LuaBinaryExpr(binary) = node else {
            continue;
        };
        if binary
            .get_op_token()
            .is_none_or(|token| token.get_op() != BinaryOperator::OpConcat)
        {
            continue;
        }
        if result.len() >= MAX_FACTS {
            return Err(fact_budget(file));
        }
        let operation_span = ast_span(file, binary.syntax().text_range())?;
        let mut operand_occurrences = Vec::new();
        for node in binary.syntax().descendants() {
            let Some(name) = LuaNameExpr::cast(node) else {
                continue;
            };
            let Some(decl) =
                model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
            else {
                continue;
            };
            let Some(binding_index) = semantic_bindings.get(&decl).copied() else {
                continue;
            };
            let use_span = ast_span(file, name.syntax().text_range())?;
            operand_occurrences.push((binding_index, use_span));
        }
        operand_occurrences.sort_by(|left, right| {
            left.1
                .byte_start()
                .cmp(&right.1.byte_start())
                .then(left.0.cmp(&right.0))
        });
        if operand_occurrences.is_empty() {
            continue;
        }
        let mut operand_binding_fact_ids = operand_occurrences
            .iter()
            .map(|(index, _)| Box::<str>::from(bindings[*index].fact.fact_id()))
            .collect::<Vec<_>>();
        operand_binding_fact_ids.sort();
        operand_binding_fact_ids.dedup();
        let mut operand_names = operand_occurrences
            .iter()
            .map(|(index, _)| Box::<str>::from(bindings[*index].fact.name()))
            .collect::<Vec<_>>();
        operand_names.sort();
        operand_names.dedup();
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            snapshot_id: &'a str,
            path: &'a str,
            content_sha256: &'a str,
            kind: EmmyOperationKind,
            operation_span: SourceSpan,
            operand_binding_fact_ids: &'a [Box<str>],
        }
        let operation_id = canonical_id(
            "emmy-operation:sha256:",
            &Identity {
                schema: "wow-emmy/operation-fact/1",
                snapshot_id: main.snapshot_id(),
                path: file.path(),
                content_sha256: file.content_sha256(),
                kind: EmmyOperationKind::Concatenation,
                operation_span,
                operand_binding_fact_ids: &operand_binding_fact_ids,
            },
        )?;
        let mut operation_uses = Vec::new();
        for (binding_index, use_span) in operand_occurrences {
            let binding = &bindings[binding_index].fact;
            #[derive(Serialize)]
            struct UseIdentity<'a> {
                schema: &'static str,
                operation_fact_id: &'a str,
                binding_fact_id: &'a str,
                use_span: SourceSpan,
            }
            let use_id = canonical_id(
                "emmy-local-use:sha256:",
                &UseIdentity {
                    schema: "wow-emmy/local-use-fact/1",
                    operation_fact_id: &operation_id,
                    binding_fact_id: binding.fact_id(),
                    use_span,
                },
            )?;
            operation_uses.push(EmmyLocalUseFact {
                fact_id: use_id.into_boxed_str(),
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                binding_fact_id: binding.fact_id().into(),
                operation_fact_id: operation_id.as_str().into(),
                name: binding.name().into(),
                use_span,
            });
        }
        operation_uses.sort();
        result.push(OperationInternal {
            fact: EmmyOperationFact {
                fact_id: operation_id.into_boxed_str(),
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                kind: EmmyOperationKind::Concatenation,
                operation_span,
                operand_binding_fact_ids,
                operand_names,
            },
            uses: operation_uses,
        });
    }
    result.sort_by(|left, right| left.fact.cmp(&right.fact));
    Ok(result)
}

fn collect_guards(
    main: &LuaWorkspaceSnapshot,
    file: &LuaWorkspaceFile,
    model: &SemanticModel<'_>,
    root: &emmylua_parser::LuaChunk,
    bindings: &[BindingInternal],
    semantic_bindings: &HashMap<LuaSemanticDeclId, usize>,
    operations: &[OperationInternal],
) -> EmmyLocalFlowResult<(Vec<EmmyGuardFact>, Vec<EmmyControlFlowFact>)> {
    let mut guards = Vec::new();
    let mut control_flow = Vec::new();
    for node in root.descendants::<LuaAst>() {
        let LuaAst::LuaIfStat(if_stat) = node else {
            continue;
        };
        let Some((binding_index, guard_span, block_span)) =
            access_guard(file, model, &if_stat, semantic_bindings)?
        else {
            continue;
        };
        if guards.len() >= MAX_FACTS {
            return Err(fact_budget(file));
        }
        let binding = &bindings[binding_index].fact;
        #[derive(Serialize)]
        struct GuardIdentity<'a> {
            schema: &'static str,
            snapshot_id: &'a str,
            path: &'a str,
            content_sha256: &'a str,
            binding_fact_id: &'a str,
            guard_span: SourceSpan,
            block_span: SourceSpan,
        }
        let guard_id = canonical_id(
            "emmy-guard:sha256:",
            &GuardIdentity {
                schema: "wow-emmy/access-single-guard-fact/1",
                snapshot_id: main.snapshot_id(),
                path: file.path(),
                content_sha256: file.content_sha256(),
                binding_fact_id: binding.fact_id(),
                guard_span,
                block_span,
            },
        )?;
        let guard = EmmyGuardFact {
            fact_id: guard_id.clone().into_boxed_str(),
            path: file.path().into(),
            content_sha256: file.content_sha256().into(),
            kind: EmmyGuardKind::AccessSingle,
            callee: "canaccessvalue",
            guarded_binding_fact_id: binding.fact_id().into(),
            guarded_name: binding.name().into(),
            guard_span,
            guarded_block_span: block_span,
        };
        for operation in operations {
            if !span_contains(block_span, operation.fact.operation_span())
                || !operation
                    .fact
                    .operand_binding_fact_ids()
                    .any(|id| id == binding.fact_id())
            {
                continue;
            }
            if control_flow.len() >= MAX_FACTS {
                return Err(fact_budget(file));
            }
            #[derive(Serialize)]
            struct RelationIdentity<'a> {
                schema: &'static str,
                guard_fact_id: &'a str,
                operation_fact_id: &'a str,
                binding_fact_id: &'a str,
                relation: EmmyControlFlowRelationKind,
            }
            let fact_id = canonical_id(
                "emmy-control-flow:sha256:",
                &RelationIdentity {
                    schema: "wow-emmy/control-flow-fact/1",
                    guard_fact_id: &guard_id,
                    operation_fact_id: operation.fact.fact_id(),
                    binding_fact_id: binding.fact_id(),
                    relation: EmmyControlFlowRelationKind::Dominates,
                },
            )?;
            control_flow.push(EmmyControlFlowFact {
                fact_id: fact_id.into_boxed_str(),
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                relation: EmmyControlFlowRelationKind::Dominates,
                guard_fact_id: guard_id.as_str().into(),
                operation_fact_id: operation.fact.fact_id().into(),
                binding_fact_id: binding.fact_id().into(),
            });
        }
        guards.push(guard);
    }
    guards.sort();
    control_flow.sort();
    Ok((guards, control_flow))
}

fn access_guard(
    file: &LuaWorkspaceFile,
    model: &SemanticModel<'_>,
    if_stat: &LuaIfStat,
    semantic_bindings: &HashMap<LuaSemanticDeclId, usize>,
) -> EmmyLocalFlowResult<Option<(usize, SourceSpan, SourceSpan)>> {
    let Some(LuaExpr::CallExpr(call)) = if_stat.get_condition_expr() else {
        return Ok(None);
    };
    let Some(LuaExpr::NameExpr(callee)) = call.get_prefix_expr() else {
        return Ok(None);
    };
    if callee.get_name_text().as_deref() != Some("canaccessvalue") {
        return Ok(None);
    }
    let Some(args) = call.get_args_list() else {
        return Ok(None);
    };
    let mut args = args.get_args();
    let Some(LuaExpr::NameExpr(argument)) = args.next() else {
        return Ok(None);
    };
    if args.next().is_some() {
        return Ok(None);
    }
    let Some(decl) = model.find_decl(
        argument.syntax().clone().into(),
        SemanticDeclLevel::default(),
    ) else {
        return Ok(None);
    };
    let Some(binding_index) = semantic_bindings.get(&decl).copied() else {
        return Ok(None);
    };
    let Some(block) = if_stat.get_block() else {
        return Ok(None);
    };
    Ok(Some((
        binding_index,
        ast_span(file, call.syntax().text_range())?,
        ast_span(file, block.syntax().text_range())?,
    )))
}

fn build_analysis(
    main: &LuaWorkspaceSnapshot,
    main_root: &Path,
    library_roots: &[(&LuaWorkspaceSnapshot, PathBuf)],
) -> EmmyLocalFlowResult<EmmyLuaAnalysis> {
    let mut analysis = EmmyLuaAnalysis::new();
    let mut configuration = Emmyrc::default();
    configuration.runtime.version = EmmyrcLuaVersion::Lua51;
    configuration.diagnostics.enable = false;
    analysis.update_config(Arc::new(configuration));
    analysis.add_main_workspace(main_root.to_path_buf());
    for (_, root) in library_roots {
        analysis.add_library_workspace(&WorkspaceFolder::new(root.clone(), true));
    }
    let mut exact_files = Vec::new();
    for (snapshot, root) in library_roots {
        exact_files.extend(
            snapshot
                .files()
                .iter()
                .map(|file| (root.join(file.path()), Some(file.text().to_owned()))),
        );
    }
    exact_files.extend(
        main.files()
            .iter()
            .map(|file| (main_root.join(file.path()), Some(file.text().to_owned()))),
    );
    let expected_files = exact_files.len();
    if analysis.update_files_by_path(exact_files).len() != expected_files {
        return Err(EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::AnalyzerFileRegistrationFailed,
            "upstream did not register every supplied Main and Library file",
            None,
        ));
    }
    Ok(analysis)
}

fn semantic_model<'a>(
    analysis: &'a EmmyLuaAnalysis,
    root: &Path,
    file: &LuaWorkspaceFile,
) -> EmmyLocalFlowResult<SemanticModel<'a>> {
    let uri = file_path_to_uri(&root.join(file.path())).ok_or_else(|| {
        EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::AnalyzerFileRegistrationFailed,
            "upstream URI construction failed",
            Some(file.path()),
        )
    })?;
    let file_id = analysis.get_file_id(&uri).ok_or_else(|| {
        EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::AnalyzerFileRegistrationFailed,
            "upstream did not retain a supplied file",
            Some(file.path()),
        )
    })?;
    analysis
        .compilation
        .get_semantic_model(file_id)
        .ok_or_else(|| {
            EmmyLocalFlowError::new(
                EmmyLocalFlowErrorCode::SemanticModelUnavailable,
                "upstream semantic model is unavailable for a registered file",
                Some(file.path()),
            )
        })
}

fn validate_compiled_backend(snapshot: &LuaWorkspaceSnapshot) -> EmmyLocalFlowResult<()> {
    let backend = snapshot.backend();
    if backend.crate_name() != "emmylua_code_analysis"
        || backend.crate_version() != Some(EMMYLUA_CODE_ANALYSIS_VERSION)
        || backend.revision() != EMMYLUA_REVISION
        || backend.tree() != EMMYLUA_TREE
    {
        return Err(EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::IncompatibleBackend,
            "workspace backend does not match the compiled analyzer pin",
            None,
        ));
    }
    Ok(())
}

fn ast_span(file: &LuaWorkspaceFile, range: rowan::TextRange) -> EmmyLocalFlowResult<SourceSpan> {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    if start > end
        || end > file.text().len()
        || !file.text().is_char_boundary(start)
        || !file.text().is_char_boundary(end)
    {
        return Err(EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::CoordinateConversionFailed,
            "upstream AST range cannot bind to exact source bytes",
            Some(file.path()),
        ));
    }
    SourceSpan::byte_range(
        u64::try_from(start).map_err(|_| coordinate_error(file))?,
        u64::try_from(end).map_err(|_| coordinate_error(file))?,
    )
    .map_err(|_| coordinate_error(file))
}

fn span_contains(container: SourceSpan, nested: SourceSpan) -> bool {
    match (
        container.byte_start(),
        container.byte_end(),
        nested.byte_start(),
        nested.byte_end(),
    ) {
        (Some(container_start), Some(container_end), Some(start), Some(end)) => {
            container_start <= start && end <= container_end
        }
        _ => false,
    }
}

fn canonical_id(prefix: &str, value: &impl Serialize) -> EmmyLocalFlowResult<String> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::CanonicalizationFailed,
            "local-flow fact identity cannot be canonicalized",
            None,
        )
    })?;
    Ok(format!("{prefix}{:x}", Sha256::digest(bytes)))
}

fn virtual_root(role: &str, snapshot_id: &str) -> PathBuf {
    let stable = snapshot_id
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                char::from(byte)
            } else {
                '_'
            }
        })
        .collect::<String>();
    std::env::temp_dir()
        .join("wow-emmy-local-flow-v1")
        .join(role)
        .join(stable)
}

fn to_u64(value: usize, path: &str) -> EmmyLocalFlowResult<u64> {
    u64::try_from(value).map_err(|_| {
        EmmyLocalFlowError::new(
            EmmyLocalFlowErrorCode::FactBudgetExceeded,
            "fact or parse-error count exceeds u64",
            Some(path),
        )
    })
}

fn fact_budget(file: &LuaWorkspaceFile) -> EmmyLocalFlowError {
    EmmyLocalFlowError::new(
        EmmyLocalFlowErrorCode::FactBudgetExceeded,
        "local-flow fact count or normalized name exceeds the adapter budget",
        Some(file.path()),
    )
}

fn coordinate_error(file: &LuaWorkspaceFile) -> EmmyLocalFlowError {
    EmmyLocalFlowError::new(
        EmmyLocalFlowErrorCode::CoordinateConversionFailed,
        "source range violates the framework byte-span contract",
        Some(file.path()),
    )
}
