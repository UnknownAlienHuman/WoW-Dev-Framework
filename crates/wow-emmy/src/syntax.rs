//! Exact-snapshot accepted diagnostics through the pinned upstream analyzer.
//!
//! Upstream database, URI, LSP range, and diagnostic types remain private.
//! Public spans are exact, end-exclusive UTF-8 byte ranges over snapshot text.

mod analyzer;
mod coordinates;

use std::fmt;

use serde::Serialize;
use wow_core::SourceSpan;

use crate::{LuaWorkspaceSnapshot, LuaWorkspaceUniverse};

pub const EMMYLUA_REVISION: &str = "aaaca68425d9362876228649b0b8d92f07654daa";
pub const EMMYLUA_TREE: &str = "9175c01384e650b9a5bd64da69c36f47dbeaaf67";
pub const EMMYLUA_CODE_ANALYSIS_VERSION: &str = "0.25.1";
pub(crate) const REPORT_SCHEMA: &str = "wow-emmy/diagnostics/2";
pub(crate) const FRAMEWORK_CATEGORY: &str = "emmy.generic.fixture_error";
pub(crate) const CAPABILITY_ID: &str = "emmy.file.diagnostics";
pub(crate) const MAX_DIAGNOSTICS: usize = 65_536;
pub(crate) const MAX_DIAGNOSTICS_PER_FILE: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmySyntaxErrorCode {
    IncompatibleBackend,
    AnalyzerFileRegistrationFailed,
    DiagnosticsUnavailable,
    DiagnosticBudgetExceeded,
    CoordinateConversionFailed,
    CanonicalizationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmySyntaxError {
    code: EmmySyntaxErrorCode,
    message: Box<str>,
    path: Option<Box<str>>,
}

impl EmmySyntaxError {
    pub(crate) fn new(
        code: EmmySyntaxErrorCode,
        message: impl Into<Box<str>>,
        path: Option<&str>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.map(Into::into),
        }
    }

    #[must_use]
    pub const fn code(&self) -> EmmySyntaxErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for EmmySyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.path() {
            Some(path) => write!(formatter, "{} ({path})", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for EmmySyntaxError {}

pub type EmmySyntaxResult<T> = Result<T, EmmySyntaxError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmySyntaxDiagnosticKind {
    LuaSyntax,
    DocumentationSyntax,
    AssignmentTypeMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyDiagnosticClassification {
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyDiagnosticRollout {
    Advisory,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmySyntaxDiagnostic {
    pub(crate) category: &'static str,
    pub(crate) upstream_code: Box<str>,
    pub(crate) kind: EmmySyntaxDiagnosticKind,
    pub(crate) upstream_severity: EmmyDiagnosticSeverity,
    pub(crate) normalized_severity: EmmyDiagnosticSeverity,
    pub(crate) classification: EmmyDiagnosticClassification,
    pub(crate) rollout: EmmyDiagnosticRollout,
    pub(crate) path: Box<str>,
    pub(crate) content_sha256: Box<str>,
    pub(crate) span: SourceSpan,
}

impl EmmySyntaxDiagnostic {
    #[must_use]
    pub const fn category(&self) -> &str {
        self.category
    }
    #[must_use]
    pub fn upstream_code(&self) -> &str {
        &self.upstream_code
    }
    #[must_use]
    pub const fn kind(&self) -> EmmySyntaxDiagnosticKind {
        self.kind
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
    pub const fn span(&self) -> SourceSpan {
        self.span
    }
    #[must_use]
    pub const fn upstream_severity(&self) -> EmmyDiagnosticSeverity {
        self.upstream_severity
    }
    #[must_use]
    pub const fn normalized_severity(&self) -> EmmyDiagnosticSeverity {
        self.normalized_severity
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmySyntaxFileReport {
    pub(crate) capability_id: &'static str,
    pub(crate) path: Box<str>,
    pub(crate) content_sha256: Box<str>,
    pub(crate) status: &'static str,
    pub(crate) diagnostic_count: u64,
}

impl EmmySyntaxFileReport {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub const fn diagnostic_count(&self) -> u64 {
        self.diagnostic_count
    }
    #[must_use]
    pub const fn status(&self) -> &str {
        self.status
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EmmySyntaxReport {
    pub(crate) schema: &'static str,
    pub(crate) analysis_id: Box<str>,
    pub(crate) upstream_revision: &'static str,
    pub(crate) upstream_tree: &'static str,
    pub(crate) upstream_crate_version: &'static str,
    pub(crate) upstream_column_encoding: &'static str,
    pub(crate) workspace_snapshot_id: Box<str>,
    pub(crate) universe: LuaWorkspaceUniverse,
    pub(crate) files: Vec<EmmySyntaxFileReport>,
    pub(crate) diagnostics: Vec<EmmySyntaxDiagnostic>,
}

impl EmmySyntaxReport {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn workspace_snapshot_id(&self) -> &str {
        &self.workspace_snapshot_id
    }
    #[must_use]
    pub fn files(&self) -> &[EmmySyntaxFileReport] {
        &self.files
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[EmmySyntaxDiagnostic] {
        &self.diagnostics
    }
}

/// Builds one analyzer instance from exact supplied bytes and returns only the
/// accepted Lua syntax, documentation syntax, and frozen E0 generic families. No source file is read or executed.
pub fn analyze_syntax(snapshot: &LuaWorkspaceSnapshot) -> EmmySyntaxResult<EmmySyntaxReport> {
    analyzer::analyze(snapshot)
}
