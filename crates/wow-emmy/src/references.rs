//! Exact direct-member call facts from the pinned EmmyLua semantic model.
//!
//! This operation analyzes Main files against explicitly supplied Library
//! snapshots. It emits analyzer-observable resolution facts only; it never
//! converts an unresolved member into a World of Warcraft availability claim.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use emmylua_code_analysis::{
    EmmyLuaAnalysis, Emmyrc, EmmyrcLuaVersion, LuaType, SemanticModel, WorkspaceFolder,
    file_path_to_uri,
};
use emmylua_parser::{LuaAst, LuaAstNode, LuaExpr, LuaIndexExpr, LuaIndexKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{SourceSpan, canonical_json_bytes};

use crate::syntax::{EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE};
use crate::{LuaWorkspaceFile, LuaWorkspaceSnapshot};

const REPORT_SCHEMA: &str = "wow-emmy/member-call-facts/1";
const REFERENCE_CAPABILITY: &str = "emmy.fact.references";
const CALL_CAPABILITY: &str = "emmy.fact.calls";
const MAX_FACTS: usize = 65_536;
const MAX_RECEIVER_BYTES: usize = 4_096;
const MAX_MEMBER_BYTES: usize = 1_024;

/// Stable direct-member analysis failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyMemberCallErrorCode {
    IncompatibleBackend,
    DuplicateWorkspaceSnapshot,
    InvalidMainWorkspace,
    AnalyzerFileRegistrationFailed,
    SemanticModelUnavailable,
    LibraryHealthFailed,
    CoordinateConversionFailed,
    FactBudgetExceeded,
    CanonicalizationFailed,
}

/// One bounded direct-member analysis failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmyMemberCallError {
    code: EmmyMemberCallErrorCode,
    message: Box<str>,
    path: Option<Box<str>>,
}

impl EmmyMemberCallError {
    fn new(
        code: EmmyMemberCallErrorCode,
        message: impl Into<Box<str>>,
        path: Option<&str>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.map(Into::into),
        }
    }

    /// Stable failure class.
    #[must_use]
    pub const fn code(&self) -> EmmyMemberCallErrorCode {
        self.code
    }

    /// Safe explanation without local host paths or source bodies.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Affected logical snapshot path, when one exists.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for EmmyMemberCallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path() {
            Some(path) => write!(formatter, "{} ({path})", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for EmmyMemberCallError {}

/// Result type for direct-member call analysis.
pub type EmmyMemberCallResult<T> = Result<T, EmmyMemberCallError>;

/// Analyzer-observable resolution state for one static member reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyReferenceResolution {
    /// EmmyLua linked the member expression to a declaration or exact type.
    Resolved,
    /// The receiver is known, but the static member is not declared.
    Unresolved,
    /// The receiver is dynamic or insufficiently typed; no absence is claimed.
    Possible,
}

/// Per-file fact capability state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyFactFileStatus {
    Complete,
    FailedParse,
}

/// Exact static member reference used as the callee of one direct call.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyMemberReferenceFact {
    fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    receiver: Box<str>,
    member: Box<str>,
    receiver_span: SourceSpan,
    member_span: SourceSpan,
    reference_span: SourceSpan,
    resolution: EmmyReferenceResolution,
}

impl EmmyMemberReferenceFact {
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
    pub fn receiver(&self) -> &str {
        &self.receiver
    }

    #[must_use]
    pub fn member(&self) -> &str {
        &self.member
    }

    #[must_use]
    pub const fn receiver_span(&self) -> SourceSpan {
        self.receiver_span
    }

    #[must_use]
    pub const fn member_span(&self) -> SourceSpan {
        self.member_span
    }

    #[must_use]
    pub const fn reference_span(&self) -> SourceSpan {
        self.reference_span
    }

    #[must_use]
    pub const fn resolution(&self) -> EmmyReferenceResolution {
        self.resolution
    }
}

/// Exact call fact linked to one [`EmmyMemberReferenceFact`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyMemberCallFact {
    fact_id: Box<str>,
    reference_fact_id: Box<str>,
    path: Box<str>,
    content_sha256: Box<str>,
    callee_span: SourceSpan,
    call_span: SourceSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    argument_count: Option<u64>,
    colon_call: bool,
}

impl EmmyMemberCallFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        &self.fact_id
    }

    #[must_use]
    pub fn reference_fact_id(&self) -> &str {
        &self.reference_fact_id
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
    pub const fn callee_span(&self) -> SourceSpan {
        self.callee_span
    }

    #[must_use]
    pub const fn call_span(&self) -> SourceSpan {
        self.call_span
    }

    #[must_use]
    pub const fn argument_count(&self) -> Option<u64> {
        self.argument_count
    }

    #[must_use]
    pub const fn is_colon_call(&self) -> bool {
        self.colon_call
    }
}

/// Coverage and output counts for one Main file.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyMemberCallFileReport {
    reference_capability: &'static str,
    call_capability: &'static str,
    path: Box<str>,
    content_sha256: Box<str>,
    status: EmmyFactFileStatus,
    parse_error_count: u64,
    reference_count: u64,
    call_count: u64,
}

impl EmmyMemberCallFileReport {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn status(&self) -> EmmyFactFileStatus {
        self.status
    }

    #[must_use]
    pub const fn parse_error_count(&self) -> u64 {
        self.parse_error_count
    }

    #[must_use]
    pub const fn reference_count(&self) -> u64 {
        self.reference_count
    }

    #[must_use]
    pub const fn call_count(&self) -> u64 {
        self.call_count
    }
}

/// Deterministic direct-member fact report for one Main snapshot and zero or
/// more explicit Library snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmyMemberCallReport {
    schema: &'static str,
    analysis_id: Box<str>,
    upstream_revision: &'static str,
    upstream_tree: &'static str,
    upstream_crate_version: &'static str,
    main_snapshot_id: Box<str>,
    library_snapshot_ids: Vec<Box<str>>,
    files: Vec<EmmyMemberCallFileReport>,
    references: Vec<EmmyMemberReferenceFact>,
    calls: Vec<EmmyMemberCallFact>,
}

impl EmmyMemberCallReport {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }

    #[must_use]
    pub fn main_snapshot_id(&self) -> &str {
        &self.main_snapshot_id
    }

    pub fn library_snapshot_ids(&self) -> impl Iterator<Item = &str> {
        self.library_snapshot_ids.iter().map(|value| value.as_ref())
    }

    #[must_use]
    pub fn files(&self) -> &[EmmyMemberCallFileReport] {
        &self.files
    }

    #[must_use]
    pub fn references(&self) -> &[EmmyMemberReferenceFact] {
        &self.references
    }

    #[must_use]
    pub fn calls(&self) -> &[EmmyMemberCallFact] {
        &self.calls
    }
}

/// Analyzes direct static member calls in Main files against explicitly supplied
/// Library snapshots. Library files participate in semantic resolution but are
/// never emitted as first-party reference or call facts.
pub fn analyze_member_calls(
    main: &LuaWorkspaceSnapshot,
    libraries: &[&LuaWorkspaceSnapshot],
) -> EmmyMemberCallResult<EmmyMemberCallReport> {
    validate_compiled_backend(main)?;
    if matches!(main.universe(), crate::LuaWorkspaceUniverse::BlizzardUi) {
        return Err(EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::InvalidMainWorkspace,
            "a Blizzard UI snapshot cannot be used as the Main project workspace",
            None,
        ));
    }

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
            return Err(EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::IncompatibleBackend,
                "Main and Library snapshots do not share one exact analyzer identity",
                None,
            ));
        }
        if !identities.insert(library.snapshot_id()) {
            return Err(EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::DuplicateWorkspaceSnapshot,
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

    let mut analysis = EmmyLuaAnalysis::new();
    let mut configuration = Emmyrc::default();
    configuration.runtime.version = EmmyrcLuaVersion::Lua51;
    configuration.diagnostics.enable = false;
    analysis.update_config(Arc::new(configuration));
    analysis.add_main_workspace(main_root.clone());
    for (_, root) in &library_roots {
        analysis.add_library_workspace(&WorkspaceFolder::new(root.clone(), true));
    }

    let mut exact_files = Vec::new();
    for (snapshot, root) in &library_roots {
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
        return Err(EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
            "upstream did not register every supplied Main and Library file",
            None,
        ));
    }

    for (snapshot, root) in &library_roots {
        for file in snapshot.files() {
            let model = semantic_model(&analysis, root, file)?;
            if model
                .get_file_parse_error()
                .is_some_and(|errors| !errors.is_empty())
            {
                return Err(EmmyMemberCallError::new(
                    EmmyMemberCallErrorCode::LibraryHealthFailed,
                    "a Library file has parse errors; resolution-dependent facts are unavailable",
                    Some(file.path()),
                ));
            }
        }
    }

    let mut files = Vec::with_capacity(main.files().len());
    let mut references = Vec::new();
    let mut calls = Vec::new();
    for file in main.files() {
        let model = semantic_model(&analysis, &main_root, file)?;
        let parse_errors = model.get_file_parse_error().unwrap_or_default();
        if !parse_errors.is_empty() {
            files.push(EmmyMemberCallFileReport {
                reference_capability: REFERENCE_CAPABILITY,
                call_capability: CALL_CAPABILITY,
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                status: EmmyFactFileStatus::FailedParse,
                parse_error_count: to_u64(parse_errors.len(), file.path())?,
                reference_count: 0,
                call_count: 0,
            });
            continue;
        }

        let reference_start = references.len();
        let call_start = calls.len();
        let root = model.get_root().clone();
        for node in root.descendants::<LuaAst>() {
            let LuaAst::LuaCallExpr(call) = node else {
                continue;
            };
            let Some(LuaExpr::IndexExpr(index)) = call.get_prefix_expr() else {
                continue;
            };
            let Some(key) = index.get_index_key() else {
                continue;
            };
            let member = match &key {
                LuaIndexKey::Name(token) => token.get_name_text().to_string(),
                LuaIndexKey::String(token) => token.get_value(),
                LuaIndexKey::Integer(_) | LuaIndexKey::Expr(_) | LuaIndexKey::Idx(_) => continue,
            };
            let Some(receiver_expr) = index.get_prefix_expr() else {
                continue;
            };
            let receiver = receiver_expr.syntax().text().to_string();
            if receiver.len() > MAX_RECEIVER_BYTES
                || member.is_empty()
                || member.len() > MAX_MEMBER_BYTES
            {
                return Err(EmmyMemberCallError::new(
                    EmmyMemberCallErrorCode::FactBudgetExceeded,
                    "a static member path exceeds the normalized fact budget",
                    Some(file.path()),
                ));
            }
            if references.len() >= MAX_FACTS || calls.len() >= MAX_FACTS {
                return Err(EmmyMemberCallError::new(
                    EmmyMemberCallErrorCode::FactBudgetExceeded,
                    "direct-member fact count exceeds the adapter budget",
                    Some(file.path()),
                ));
            }

            let receiver_span = ast_span(file, receiver_expr.syntax().text_range())?;
            let member_range = key.get_range().ok_or_else(|| {
                EmmyMemberCallError::new(
                    EmmyMemberCallErrorCode::CoordinateConversionFailed,
                    "a static member key has no exact source range",
                    Some(file.path()),
                )
            })?;
            let member_span = ast_span(file, member_range)?;
            let reference_span = ast_span(file, index.syntax().text_range())?;
            let call_span = ast_span(file, call.syntax().text_range())?;
            let resolution = reference_resolution(&model, &index, receiver_expr);
            let reference_fact_id = reference_fact_id(
                main.snapshot_id(),
                file,
                &receiver,
                &member,
                receiver_span,
                member_span,
                reference_span,
                resolution,
            )?;
            references.push(EmmyMemberReferenceFact {
                fact_id: reference_fact_id.clone().into_boxed_str(),
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                receiver: receiver.into_boxed_str(),
                member: member.into_boxed_str(),
                receiver_span,
                member_span,
                reference_span,
                resolution,
            });

            let argument_count = call
                .get_args_count()
                .map(|count| to_u64(count, file.path()))
                .transpose()?;
            let colon_call = call.is_colon_call();
            let call_fact_id = call_fact_id(
                main.snapshot_id(),
                file,
                &reference_fact_id,
                reference_span,
                call_span,
                argument_count,
                colon_call,
            )?;
            calls.push(EmmyMemberCallFact {
                fact_id: call_fact_id.into_boxed_str(),
                reference_fact_id: reference_fact_id.into_boxed_str(),
                path: file.path().into(),
                content_sha256: file.content_sha256().into(),
                callee_span: reference_span,
                call_span,
                argument_count,
                colon_call,
            });
        }

        files.push(EmmyMemberCallFileReport {
            reference_capability: REFERENCE_CAPABILITY,
            call_capability: CALL_CAPABILITY,
            path: file.path().into(),
            content_sha256: file.content_sha256().into(),
            status: EmmyFactFileStatus::Complete,
            parse_error_count: 0,
            reference_count: to_u64(references.len() - reference_start, file.path())?,
            call_count: to_u64(calls.len() - call_start, file.path())?,
        });
    }
    references.sort();
    calls.sort();

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
        files: &'a [EmmyMemberCallFileReport],
        references: &'a [EmmyMemberReferenceFact],
        calls: &'a [EmmyMemberCallFact],
    }
    let identity = ReportIdentity {
        schema: REPORT_SCHEMA,
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        main_snapshot_id: main.snapshot_id(),
        library_snapshot_ids: &library_snapshot_ids,
        files: &files,
        references: &references,
        calls: &calls,
    };
    let analysis_id = canonical_id("emmy-member-calls:sha256:", &identity)?;
    Ok(EmmyMemberCallReport {
        schema: REPORT_SCHEMA,
        analysis_id: analysis_id.into_boxed_str(),
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        upstream_crate_version: EMMYLUA_CODE_ANALYSIS_VERSION,
        main_snapshot_id: main.snapshot_id().into(),
        library_snapshot_ids,
        files,
        references,
        calls,
    })
}

fn semantic_model<'a>(
    analysis: &'a EmmyLuaAnalysis,
    root: &Path,
    file: &LuaWorkspaceFile,
) -> EmmyMemberCallResult<SemanticModel<'a>> {
    let uri = file_path_to_uri(&root.join(file.path())).ok_or_else(|| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
            "upstream URI construction failed",
            Some(file.path()),
        )
    })?;
    let file_id = analysis.get_file_id(&uri).ok_or_else(|| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::AnalyzerFileRegistrationFailed,
            "upstream did not retain a supplied file",
            Some(file.path()),
        )
    })?;
    analysis
        .compilation
        .get_semantic_model(file_id)
        .ok_or_else(|| {
            EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::SemanticModelUnavailable,
                "upstream semantic model is unavailable for a registered file",
                Some(file.path()),
            )
        })
}

fn reference_resolution(
    model: &SemanticModel<'_>,
    index: &LuaIndexExpr,
    receiver: LuaExpr,
) -> EmmyReferenceResolution {
    let linked = model
        .get_semantic_info(index.syntax().clone().into())
        .is_some_and(|info| info.semantic_decl.is_some());
    let typed = model
        .get_index_decl_type(index.clone())
        .is_some_and(|typ| !matches!(typ, LuaType::Unknown));
    if linked || typed {
        return EmmyReferenceResolution::Resolved;
    }
    let prefix_type = model.infer_expr(receiver).unwrap_or(LuaType::Unknown);
    if matches!(
        prefix_type,
        LuaType::Unknown
            | LuaType::Any
            | LuaType::Table
            | LuaType::Global
            | LuaType::Userdata
            | LuaType::TableConst(_)
            | LuaType::StrTplRef(_)
    ) {
        EmmyReferenceResolution::Possible
    } else {
        EmmyReferenceResolution::Unresolved
    }
}

#[allow(clippy::too_many_arguments)]
fn reference_fact_id(
    snapshot_id: &str,
    file: &LuaWorkspaceFile,
    receiver: &str,
    member: &str,
    receiver_span: SourceSpan,
    member_span: SourceSpan,
    reference_span: SourceSpan,
    resolution: EmmyReferenceResolution,
) -> EmmyMemberCallResult<String> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        snapshot_id: &'a str,
        path: &'a str,
        content_sha256: &'a str,
        receiver: &'a str,
        member: &'a str,
        receiver_span: SourceSpan,
        member_span: SourceSpan,
        reference_span: SourceSpan,
        resolution: EmmyReferenceResolution,
    }
    canonical_id(
        "emmy-reference:sha256:",
        &Identity {
            schema: "wow-emmy/member-reference-fact/1",
            snapshot_id,
            path: file.path(),
            content_sha256: file.content_sha256(),
            receiver,
            member,
            receiver_span,
            member_span,
            reference_span,
            resolution,
        },
    )
}

#[allow(clippy::too_many_arguments)]
fn call_fact_id(
    snapshot_id: &str,
    file: &LuaWorkspaceFile,
    reference_fact_id: &str,
    callee_span: SourceSpan,
    call_span: SourceSpan,
    argument_count: Option<u64>,
    colon_call: bool,
) -> EmmyMemberCallResult<String> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        snapshot_id: &'a str,
        path: &'a str,
        content_sha256: &'a str,
        reference_fact_id: &'a str,
        callee_span: SourceSpan,
        call_span: SourceSpan,
        #[serde(skip_serializing_if = "Option::is_none")]
        argument_count: Option<u64>,
        colon_call: bool,
    }
    canonical_id(
        "emmy-call:sha256:",
        &Identity {
            schema: "wow-emmy/member-call-fact/1",
            snapshot_id,
            path: file.path(),
            content_sha256: file.content_sha256(),
            reference_fact_id,
            callee_span,
            call_span,
            argument_count,
            colon_call,
        },
    )
}

fn canonical_id(prefix: &str, value: &impl Serialize) -> EmmyMemberCallResult<String> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::CanonicalizationFailed,
            "member-call fact identity cannot be canonicalized",
            None,
        )
    })?;
    Ok(format!("{prefix}{:x}", Sha256::digest(bytes)))
}

fn ast_span(file: &LuaWorkspaceFile, range: rowan::TextRange) -> EmmyMemberCallResult<SourceSpan> {
    let start = usize::from(range.start());
    let end = usize::from(range.end());
    if start > end
        || end > file.text().len()
        || !file.text().is_char_boundary(start)
        || !file.text().is_char_boundary(end)
    {
        return Err(EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::CoordinateConversionFailed,
            "upstream AST range cannot bind to exact source bytes",
            Some(file.path()),
        ));
    }
    SourceSpan::byte_range(
        u64::try_from(start).map_err(|_| {
            EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::CoordinateConversionFailed,
                "source range start exceeds u64",
                Some(file.path()),
            )
        })?,
        u64::try_from(end).map_err(|_| {
            EmmyMemberCallError::new(
                EmmyMemberCallErrorCode::CoordinateConversionFailed,
                "source range end exceeds u64",
                Some(file.path()),
            )
        })?,
    )
    .map_err(|_| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::CoordinateConversionFailed,
            "source range violates the framework byte-span contract",
            Some(file.path()),
        )
    })
}

fn validate_compiled_backend(snapshot: &LuaWorkspaceSnapshot) -> EmmyMemberCallResult<()> {
    let backend = snapshot.backend();
    if backend.crate_name() != "emmylua_code_analysis"
        || backend.crate_version() != Some(EMMYLUA_CODE_ANALYSIS_VERSION)
        || backend.revision() != EMMYLUA_REVISION
        || backend.tree() != EMMYLUA_TREE
    {
        return Err(EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::IncompatibleBackend,
            "workspace backend does not match the compiled analyzer pin",
            None,
        ));
    }
    Ok(())
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
        .join("wow-emmy-member-calls-v1")
        .join(role)
        .join(stable)
}

fn to_u64(value: usize, path: &str) -> EmmyMemberCallResult<u64> {
    u64::try_from(value).map_err(|_| {
        EmmyMemberCallError::new(
            EmmyMemberCallErrorCode::FactBudgetExceeded,
            "fact or parse-error count exceeds u64",
            Some(path),
        )
    })
}
