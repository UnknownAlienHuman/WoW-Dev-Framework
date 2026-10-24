use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

const SNAPSHOT_SCHEMA_VERSION: u64 = 1;
const ANALYZER_CRATE: &str = "emmylua_code_analysis";

/// Stable explicit-workspace failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EmmyWorkspaceErrorCode {
    InvalidBackendIdentity,
    InvalidPath,
    InvalidEncoding,
    DuplicatePath,
    CaseCollision,
    FileLimitExceeded,
    FileSizeLimitExceeded,
    TotalSizeLimitExceeded,
    CanonicalizationFailed,
}

/// One bounded workspace-construction failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmmyWorkspaceError {
    code: EmmyWorkspaceErrorCode,
    message: Box<str>,
    path: Option<Box<str>>,
}

impl EmmyWorkspaceError {
    fn new(
        code: EmmyWorkspaceErrorCode,
        message: impl Into<Box<str>>,
        path: Option<impl Into<Box<str>>>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            path: path.map(Into::into),
        }
    }

    /// Stable error class.
    #[must_use]
    pub const fn code(&self) -> EmmyWorkspaceErrorCode {
        self.code
    }

    /// Safe explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Affected canonical candidate path, when one exists.
    #[must_use]
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

impl fmt::Display for EmmyWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = self.path() {
            write!(formatter, "{} ({path})", self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for EmmyWorkspaceError {}

/// Result type for explicit workspace construction.
pub type EmmyWorkspaceResult<T> = Result<T, EmmyWorkspaceError>;

/// Evidence source bound to an exact analyzer build identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmmyBackendIdentitySource {
    CompatibilityReport,
    CompiledPin,
}

/// Borrowed compatibility-report evidence attached to a report-backed identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmmyCompatibilityReportEvidence<'a> {
    surface_sha256: &'a str,
    compatibility_report_sha256: &'a str,
}

/// Exact analyzer build identity used by one analysis operation.
///
/// Report-backed identities keep their historical serialized shape. Compiled
/// identities carry an explicit source marker and omit report digests instead
/// of fabricating compatibility evidence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EmmyBackendIdentity {
    crate_name: Box<str>,
    crate_version: Option<Box<str>>,
    revision: Box<str>,
    tree: Box<str>,
    #[serde(skip_serializing_if = "is_compatibility_report_source")]
    source: EmmyBackendIdentitySource,
    #[serde(skip_serializing_if = "Option::is_none")]
    surface_sha256: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    compatibility_report_sha256: Option<Box<str>>,
}

struct ValidatedBackendParts {
    crate_name: Box<str>,
    crate_version: Option<Box<str>>,
    revision: Box<str>,
    tree: Box<str>,
}

impl<'a> EmmyCompatibilityReportEvidence<'a> {
    /// Deterministic public-surface digest from the compatibility report.
    #[must_use]
    pub const fn surface_sha256(self) -> &'a str {
        self.surface_sha256
    }

    /// Self-digest of the compatibility report that authorized this identity.
    #[must_use]
    pub const fn compatibility_report_sha256(self) -> &'a str {
        self.compatibility_report_sha256
    }
}

impl EmmyBackendIdentity {
    /// Creates one exact backend identity from a verified compatibility report.
    pub fn new(
        crate_name: impl Into<String>,
        crate_version: Option<impl Into<String>>,
        revision: impl Into<String>,
        tree: impl Into<String>,
        surface_sha256: impl Into<String>,
        compatibility_report_sha256: impl Into<String>,
    ) -> EmmyWorkspaceResult<Self> {
        let parts = validate_backend_parts(
            crate_name.into(),
            crate_version.map(Into::into),
            revision.into(),
            tree.into(),
        )?;
        let surface_sha256 = surface_sha256.into();
        let compatibility_report_sha256 = compatibility_report_sha256.into();
        validate_sha256(&surface_sha256, "analyzer surface digest")?;
        validate_sha256(
            &compatibility_report_sha256,
            "analyzer compatibility report digest",
        )?;
        Ok(Self {
            crate_name: parts.crate_name,
            crate_version: parts.crate_version,
            revision: parts.revision,
            tree: parts.tree,
            source: EmmyBackendIdentitySource::CompatibilityReport,
            surface_sha256: Some(surface_sha256.into_boxed_str()),
            compatibility_report_sha256: Some(compatibility_report_sha256.into_boxed_str()),
        })
    }

    /// Creates one exact identity from the compiled crate/version/revision/tree
    /// without claiming that a compatibility report authorized it.
    pub fn compiled(
        crate_name: impl Into<String>,
        crate_version: Option<impl Into<String>>,
        revision: impl Into<String>,
        tree: impl Into<String>,
    ) -> EmmyWorkspaceResult<Self> {
        let parts = validate_backend_parts(
            crate_name.into(),
            crate_version.map(Into::into),
            revision.into(),
            tree.into(),
        )?;
        Ok(Self {
            crate_name: parts.crate_name,
            crate_version: parts.crate_version,
            revision: parts.revision,
            tree: parts.tree,
            source: EmmyBackendIdentitySource::CompiledPin,
            surface_sha256: None,
            compatibility_report_sha256: None,
        })
    }

    /// Exact identity of the analyzer compiled into this framework build.
    pub fn compiled_pinned() -> EmmyWorkspaceResult<Self> {
        Self::compiled(
            ANALYZER_CRATE,
            Some(crate::syntax::EMMYLUA_CODE_ANALYSIS_VERSION),
            crate::syntax::EMMYLUA_REVISION,
            crate::syntax::EMMYLUA_TREE,
        )
    }

    /// Analyzer crate name.
    #[must_use]
    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    /// Analyzer crate version, when the upstream manifest declares one directly.
    #[must_use]
    pub fn crate_version(&self) -> Option<&str> {
        self.crate_version.as_deref()
    }

    /// Exact analyzer source revision.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Exact analyzer source tree.
    #[must_use]
    pub fn tree(&self) -> &str {
        &self.tree
    }

    /// Evidence source attached to this exact build identity.
    #[must_use]
    pub const fn source(&self) -> EmmyBackendIdentitySource {
        self.source
    }

    /// Compatibility-report evidence when this identity came from a report.
    #[must_use]
    pub fn compatibility_report_evidence(&self) -> Option<EmmyCompatibilityReportEvidence<'_>> {
        match (
            self.source,
            self.surface_sha256.as_deref(),
            self.compatibility_report_sha256.as_deref(),
        ) {
            (
                EmmyBackendIdentitySource::CompatibilityReport,
                Some(surface_sha256),
                Some(compatibility_report_sha256),
            ) => Some(EmmyCompatibilityReportEvidence {
                surface_sha256,
                compatibility_report_sha256,
            }),
            _ => None,
        }
    }

    /// Requires report evidence for a legacy publication/replay caller.
    ///
    /// Direct compiled-pin callers must not use this boundary. Returning a typed
    /// error keeps accidental cross-use explicit without a sentinel or panic.
    pub fn require_compatibility_report_evidence(
        &self,
    ) -> EmmyWorkspaceResult<EmmyCompatibilityReportEvidence<'_>> {
        self.compatibility_report_evidence().ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::InvalidBackendIdentity,
                "compatibility-report evidence is unavailable for a compiled-pin identity",
                None::<String>,
            )
        })
    }
}

/// Semantic universe represented by one analyzer workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LuaWorkspaceUniverse {
    Project,
    BlizzardUi,
    Fixture,
    /// Platform implementation analyzed as Main, distinct from the legacy Library universe.
    BlizzardUiMain,
}

/// Explicit limits applied before analyzer ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LuaWorkspaceLimits {
    max_files: u64,
    max_path_bytes: u64,
    max_file_bytes: u64,
    max_total_bytes: u64,
}

impl LuaWorkspaceLimits {
    /// Creates one nonzero limit set.
    pub fn new(
        max_files: u64,
        max_path_bytes: u64,
        max_file_bytes: u64,
        max_total_bytes: u64,
    ) -> EmmyWorkspaceResult<Self> {
        if max_files == 0
            || max_path_bytes == 0
            || max_file_bytes == 0
            || max_total_bytes == 0
            || max_file_bytes > max_total_bytes
        {
            return Err(workspace_error(
                EmmyWorkspaceErrorCode::FileLimitExceeded,
                "workspace limits are zero or internally inconsistent",
                None::<String>,
            ));
        }
        Ok(Self {
            max_files,
            max_path_bytes,
            max_file_bytes,
            max_total_bytes,
        })
    }

    /// Maximum file count.
    #[must_use]
    pub const fn max_files(self) -> u64 {
        self.max_files
    }

    /// Maximum UTF-8 path byte length.
    #[must_use]
    pub const fn max_path_bytes(self) -> u64 {
        self.max_path_bytes
    }

    /// Maximum bytes in one source.
    #[must_use]
    pub const fn max_file_bytes(self) -> u64 {
        self.max_file_bytes
    }

    /// Maximum bytes across all sources.
    #[must_use]
    pub const fn max_total_bytes(self) -> u64 {
        self.max_total_bytes
    }
}

/// One caller-supplied Lua source before validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaWorkspaceFileInput {
    path: String,
    text: String,
}

impl LuaWorkspaceFileInput {
    /// Creates an explicit UTF-8 source input.
    #[must_use]
    pub fn new(Path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    /// Creates an input from exact bytes after UTF-8 validation.
    pub fn from_bytes(path: impl Into<String>, content: Vec<u8>) -> EmmyWorkspaceResult<Self> {
        let path = path.into();
        let text = String::from_utf8(content).map_err(|_|  {
            workspace_error(
                EmmyWorkspaceErrorCode::InvalidEncoding,
                "Lua source is not valid UTF-8",
                Some(path.clone()),
            )
        })?;
        Ok(Self { path, text })
    }
}
