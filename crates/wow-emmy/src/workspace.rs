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

/// Exact analyzer build identity used by one analysis operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct EmmyBackendIdentity {
    crate_name: Box<str>,
    crate_version: Option<Box<str>>,
    revision: Box<str>,
    tree: Box<str>,
    surface_sha256: Box<str>,
    compatibility_report_sha256: Box<str>,
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
        let crate_name = crate_name.into();
        let crate_version = crate_version.map(Into::into);
        let revision = revision.into();
        let tree = tree.into();
        let surface_sha256 = surface_sha256.into();
        let compatibility_report_sha256 = compatibility_report_sha256.into();
        if crate_name != ANALYZER_CRATE {
            return Err(workspace_error(
                EmmyWorkspaceErrorCode::InvalidBackendIdentity,
                "unexpected analyzer crate identity",
                None::<String>,
            ));
        }
        if crate_version.as_deref().is_some_and(invalid_scalar) {
            return Err(workspace_error(
                EmmyWorkspaceErrorCode::InvalidBackendIdentity,
                "analyzer crate version is invalid",
                None::<String>,
            ));
        }
        validate_object_id(&revision, None, "analyzer revision")?;
        validate_object_id(&tree, Some(revision.len()), "analyzer tree")?;
        validate_sha256(&surface_sha256, "analyzer surface digest")?;
        validate_sha256(
            &compatibility_report_sha256,
            "analyzer compatibility report digest",
        )?;
        Ok(Self {
            crate_name: crate_name.into_boxed_str(),
            crate_version: crate_version.map(String::into_boxed_str),
            revision: revision.into_boxed_str(),
            tree: tree.into_boxed_str(),
            surface_sha256: surface_sha256.into_boxed_str(),
            compatibility_report_sha256: compatibility_report_sha256.into_boxed_str(),
        })
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

    /// Deterministic public-surface digest from the compatibility report.
    #[must_use]
    pub fn surface_sha256(&self) -> &str {
        &self.surface_sha256
    }

    /// Self-digest of the compatibility report that authorized this identity.
    #[must_use]
    pub fn compatibility_report_sha256(&self) -> &str {
        &self.compatibility_report_sha256
    }
}

/// Semantic universe represented by one analyzer workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LuaWorkspaceUniverse {
    Project,
    BlizzardUi,
    Fixture,
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
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    /// Creates an input from exact bytes after UTF-8 validation.
    pub fn from_bytes(
        path: impl Into<String>,
        content: Vec<u8>,
    ) -> EmmyWorkspaceResult<Self> {
        let path = path.into();
        let text = String::from_utf8(content).map_err(|_| {
            workspace_error(
                EmmyWorkspaceErrorCode::InvalidEncoding,
                "Lua source is not valid UTF-8",
                Some(path.clone()),
            )
        })?;
        Ok(Self { path, text })
    }
}

/// One validated immutable Lua source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaWorkspaceFile {
    path: Box<str>,
    text: Box<str>,
    content_sha256: Box<str>,
    byte_len: u64,
}

impl LuaWorkspaceFile {
    /// Canonical case-preserving source path.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Exact UTF-8 source text. Newline and BOM bytes are not normalized.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// SHA-256 identity of exact source bytes.
    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    /// Exact UTF-8 byte length.
    #[must_use]
    pub const fn byte_len(&self) -> u64 {
        self.byte_len
    }
}

/// Immutable explicit Lua workspace for one analyzer identity and one universe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LuaWorkspaceSnapshot {
    snapshot_id: Box<str>,
    backend: EmmyBackendIdentity,
    universe: LuaWorkspaceUniverse,
    files: Vec<LuaWorkspaceFile>,
    total_bytes: u64,
}

impl LuaWorkspaceSnapshot {
    /// Validates, orders, and content-addresses one explicit source set.
    pub fn build(
        backend: EmmyBackendIdentity,
        universe: LuaWorkspaceUniverse,
        inputs: Vec<LuaWorkspaceFileInput>,
        limits: LuaWorkspaceLimits,
    ) -> EmmyWorkspaceResult<Self> {
        let input_count = u64::try_from(inputs.len()).map_err(|_| {
            workspace_error(
                EmmyWorkspaceErrorCode::FileLimitExceeded,
                "workspace file count exceeds u64",
                None::<String>,
            )
        })?;
        if input_count > limits.max_files {
            return Err(workspace_error(
                EmmyWorkspaceErrorCode::FileLimitExceeded,
                "workspace file count exceeds the configured limit",
                None::<String>,
            ));
        }

        let mut exact_paths = BTreeSet::new();
        let mut folded_paths = BTreeMap::<String, String>::new();
        let mut files = Vec::with_capacity(inputs.len());
        let mut total_bytes = 0_u64;
        for input in inputs {
            let path = validate_path(input.path, limits.max_path_bytes)?;
            if !exact_paths.insert(path.clone()) {
                return Err(workspace_error(
                    EmmyWorkspaceErrorCode::DuplicatePath,
                    "workspace contains the same exact path more than once",
                    Some(path),
                ));
            }
            let folded = path.to_lowercase();
            if let Some(existing) = folded_paths.insert(folded, path.clone()) {
                return Err(workspace_error(
                    EmmyWorkspaceErrorCode::CaseCollision,
                    format!("workspace paths collide under case folding with {existing:?}"),
                    Some(path),
                ));
            }
            if input.text.contains('\0') {
                return Err(workspace_error(
                    EmmyWorkspaceErrorCode::InvalidEncoding,
                    "Lua source contains a NUL character",
                    Some(path),
                ));
            }
            let byte_len = u64::try_from(input.text.len()).map_err(|_| {
                workspace_error(
                    EmmyWorkspaceErrorCode::FileSizeLimitExceeded,
                    "Lua source byte length exceeds u64",
                    Some(path.clone()),
                )
            })?;
            if byte_len > limits.max_file_bytes {
                return Err(workspace_error(
                    EmmyWorkspaceErrorCode::FileSizeLimitExceeded,
                    "Lua source exceeds the configured per-file limit",
                    Some(path),
                ));
            }
            total_bytes = total_bytes.checked_add(byte_len).ok_or_else(|| {
                workspace_error(
                    EmmyWorkspaceErrorCode::TotalSizeLimitExceeded,
                    "workspace byte count overflow",
                    None::<String>,
                )
            })?;
            if total_bytes > limits.max_total_bytes {
                return Err(workspace_error(
                    EmmyWorkspaceErrorCode::TotalSizeLimitExceeded,
                    "workspace exceeds the configured total byte limit",
                    Some(path),
                ));
            }
            let content_sha256 = sha256_id(input.text.as_bytes());
            files.push(LuaWorkspaceFile {
                path: path.into_boxed_str(),
                text: input.text.into_boxed_str(),
                content_sha256: content_sha256.into_boxed_str(),
                byte_len,
            });
        }
        files.sort_by(|left, right| left.path.as_bytes().cmp(right.path.as_bytes()));

        #[derive(Serialize)]
        struct FileProjection<'a> {
            path: &'a str,
            content_sha256: &'a str,
            byte_len: u64,
        }
        #[derive(Serialize)]
        struct SnapshotProjection<'a> {
            schema_version: u64,
            backend: &'a EmmyBackendIdentity,
            universe: LuaWorkspaceUniverse,
            total_bytes: u64,
            files: Vec<FileProjection<'a>>,
        }
        let projection = SnapshotProjection {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            backend: &backend,
            universe,
            total_bytes,
            files: files
                .iter()
                .map(|file| FileProjection {
                    path: file.path(),
                    content_sha256: file.content_sha256(),
                    byte_len: file.byte_len(),
                })
                .collect(),
        };
        let canonical = canonical_json_bytes(&projection).map_err(|source| {
            workspace_error(
                EmmyWorkspaceErrorCode::CanonicalizationFailed,
                format!("workspace identity cannot be canonicalized: {source}"),
                None::<String>,
            )
        })?;
        let snapshot_id = format!("workspace:{}", sha256_id(&canonical));
        Ok(Self {
            snapshot_id: snapshot_id.into_boxed_str(),
            backend,
            universe,
            files,
            total_bytes,
        })
    }

    /// Content-addressed workspace identity.
    #[must_use]
    pub fn snapshot_id(&self) -> &str {
        &self.snapshot_id
    }

    /// Exact analyzer identity bound to this workspace.
    #[must_use]
    pub const fn backend(&self) -> &EmmyBackendIdentity {
        &self.backend
    }

    /// Semantic source universe.
    #[must_use]
    pub const fn universe(&self) -> LuaWorkspaceUniverse {
        self.universe
    }

    /// Sources ordered by UTF-8 path bytes.
    #[must_use]
    pub fn files(&self) -> &[LuaWorkspaceFile] {
        &self.files
    }

    /// Exact aggregate source bytes.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    /// Exact case-sensitive source lookup.
    #[must_use]
    pub fn file(&self, path: &str) -> Option<&LuaWorkspaceFile> {
        self.files
            .binary_search_by(|file| file.path().as_bytes().cmp(path.as_bytes()))
            .ok()
            .map(|index| &self.files[index])
    }
}

fn validate_path(candidate: String, max_path_bytes: u64) -> EmmyWorkspaceResult<String> {
    let path_bytes = u64::try_from(candidate.len()).map_err(|_| {
        workspace_error(
            EmmyWorkspaceErrorCode::InvalidPath,
            "Lua path byte length exceeds u64",
            Some(candidate.clone()),
        )
    })?;
    let drive_absolute = candidate
        .as_bytes()
        .get(1)
        .is_some_and(|byte| *byte == b':');
    let extension_is_lua = candidate
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("lua"));
    if path_bytes == 0
        || path_bytes > max_path_bytes
        || candidate.starts_with('/')
        || drive_absolute
        || candidate.contains('\\')
        || candidate.chars().any(char::is_control)
        || candidate
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || !extension_is_lua
    {
        return Err(workspace_error(
            EmmyWorkspaceErrorCode::InvalidPath,
            "Lua path is noncanonical, unsafe, or outside the .lua source scope",
            Some(candidate),
        ));
    }
    Ok(candidate)
}

fn validate_object_id(
    value: &str,
    expected_length: Option<usize>,
    label: &str,
) -> EmmyWorkspaceResult<()> {
    if !matches!(value.len(), 40 | 64)
        || expected_length.is_some_and(|length| length != value.len())
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(workspace_error(
            EmmyWorkspaceErrorCode::InvalidBackendIdentity,
            format!("{label} is not a canonical Git object identifier"),
            None::<String>,
        ));
    }
    Ok(())
}

fn validate_sha256(value: &str, label: &str) -> EmmyWorkspaceResult<()> {
    let Some(digest) = value.strip_prefix("sha256:") else {
        return Err(workspace_error(
            EmmyWorkspaceErrorCode::InvalidBackendIdentity,
            format!("{label} is not a canonical SHA-256 identifier"),
            None::<String>,
        ));
    };
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(workspace_error(
            EmmyWorkspaceErrorCode::InvalidBackendIdentity,
            format!("{label} is not a SHA-256 digest"),
            None::<String>,
        ));
    }
    Ok(())
}

fn invalid_scalar(value: &str) -> bool {
    value.is_empty() || value.chars().any(char::is_control)
}

fn sha256_id(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{digest:x}")
}

fn workspace_error(
    code: EmmyWorkspaceErrorCode,
    message: impl Into<Box<str>>,
    path: Option<impl Into<Box<str>>>,
) -> EmmyWorkspaceError {
    EmmyWorkspaceError::new(code, message, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend() -> EmmyWorkspaceResult<EmmyBackendIdentity> {
        EmmyBackendIdentity::new(
            ANALYZER_CRATE,
            Some("0.21.0"),
            "1111111111111111111111111111111111111111",
            "2222222222222222222222222222222222222222",
            "sha256:3333333333333333333333333333333333333333333333333333333333333333",
            "sha256:4444444444444444444444444444444444444444444444444444444444444444",
        )
    }

    fn limits() -> EmmyWorkspaceResult<LuaWorkspaceLimits> {
        LuaWorkspaceLimits::new(8, 256, 1024, 4096)
    }

    #[test]
    fn snapshot_is_deterministic_and_byte_sensitive() -> EmmyWorkspaceResult<()> {
        let first = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("B.lua", "return 2\r\n"),
                LuaWorkspaceFileInput::new("A.lua", "return 1\n"),
            ],
            limits()?,
        )?;
        let reordered = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("A.lua", "return 1\n"),
                LuaWorkspaceFileInput::new("B.lua", "return 2\r\n"),
            ],
            limits()?,
        )?;
        let changed = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("A.lua", "return 1\r\n"),
                LuaWorkspaceFileInput::new("B.lua", "return 2\r\n"),
            ],
            limits()?,
        )?;
        assert_eq!(first.snapshot_id(), reordered.snapshot_id());
        assert_ne!(first.snapshot_id(), changed.snapshot_id());
        assert_eq!(first.files()[0].path(), "A.lua");
        assert_eq!(first.file("B.lua").map(LuaWorkspaceFile::text), Some("return 2\r\n"));
        Ok(())
    }

    #[test]
    fn universe_and_backend_identity_participate_in_snapshot_identity() -> EmmyWorkspaceResult<()> {
        let files = || vec![LuaWorkspaceFileInput::new("Main.lua", "local value = 1\n")];
        let project = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            files(),
            limits()?,
        )?;
        let blizzard = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::BlizzardUi,
            files(),
            limits()?,
        )?;
        let other_backend = EmmyBackendIdentity::new(
            ANALYZER_CRATE,
            Some("0.22.0"),
            "5555555555555555555555555555555555555555",
            "6666666666666666666666666666666666666666",
            "sha256:7777777777777777777777777777777777777777777777777777777777777777",
            "sha256:8888888888888888888888888888888888888888888888888888888888888888",
        )?;
        let updated = LuaWorkspaceSnapshot::build(
            other_backend,
            LuaWorkspaceUniverse::Project,
            files(),
            limits()?,
        )?;
        assert_ne!(project.snapshot_id(), blizzard.snapshot_id());
        assert_ne!(project.snapshot_id(), updated.snapshot_id());
        Ok(())
    }

    #[test]
    fn duplicate_and_case_colliding_paths_are_rejected() -> EmmyWorkspaceResult<()> {
        let duplicate = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("Main.lua", "return 1"),
                LuaWorkspaceFileInput::new("Main.lua", "return 2"),
            ],
            limits()?,
        )
        .err()
        .ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::DuplicatePath,
                "duplicate path unexpectedly accepted",
                None::<String>,
            )
        })?;
        assert_eq!(duplicate.code(), EmmyWorkspaceErrorCode::DuplicatePath);

        let collision = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("Main.lua", "return 1"),
                LuaWorkspaceFileInput::new("main.lua", "return 2"),
            ],
            limits()?,
        )
        .err()
        .ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::CaseCollision,
                "case collision unexpectedly accepted",
                None::<String>,
            )
        })?;
        assert_eq!(collision.code(), EmmyWorkspaceErrorCode::CaseCollision);
        Ok(())
    }

    #[test]
    fn unsafe_or_non_lua_paths_are_rejected() -> EmmyWorkspaceResult<()> {
        for path in [
            "../Main.lua",
            "/Main.lua",
            "C:/Main.lua",
            "Folder\\Main.lua",
            "Folder//Main.lua",
            "Main.xml",
        ] {
            let result = LuaWorkspaceSnapshot::build(
                backend()?,
                LuaWorkspaceUniverse::Project,
                vec![LuaWorkspaceFileInput::new(path, "return 1")],
                limits()?,
            );
            let error = result.err().ok_or_else(|| {
                workspace_error(
                    EmmyWorkspaceErrorCode::InvalidPath,
                    format!("unsafe path {path:?} unexpectedly accepted"),
                    None::<String>,
                )
            })?;
            assert_eq!(error.code(), EmmyWorkspaceErrorCode::InvalidPath);
        }
        Ok(())
    }

    #[test]
    fn invalid_bytes_and_nul_text_are_rejected() -> EmmyWorkspaceResult<()> {
        let invalid = LuaWorkspaceFileInput::from_bytes("Main.lua", vec![0xff])
            .err()
            .ok_or_else(|| {
                workspace_error(
                    EmmyWorkspaceErrorCode::InvalidEncoding,
                    "invalid UTF-8 unexpectedly accepted",
                    None::<String>,
                )
            })?;
        assert_eq!(invalid.code(), EmmyWorkspaceErrorCode::InvalidEncoding);

        let nul = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![LuaWorkspaceFileInput::new("Main.lua", "local x = '\0'")],
            limits()?,
        )
        .err()
        .ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::InvalidEncoding,
                "NUL source unexpectedly accepted",
                None::<String>,
            )
        })?;
        assert_eq!(nul.code(), EmmyWorkspaceErrorCode::InvalidEncoding);
        Ok(())
    }

    #[test]
    fn configured_file_and_total_limits_are_enforced() -> EmmyWorkspaceResult<()> {
        let tiny = LuaWorkspaceLimits::new(1, 32, 4, 4)?;
        let file = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![LuaWorkspaceFileInput::new("Main.lua", "12345")],
            tiny,
        )
        .err()
        .ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::FileSizeLimitExceeded,
                "file limit unexpectedly ignored",
                None::<String>,
            )
        })?;
        assert_eq!(file.code(), EmmyWorkspaceErrorCode::FileSizeLimitExceeded);

        let total_limits = LuaWorkspaceLimits::new(2, 32, 4, 6)?;
        let total = LuaWorkspaceSnapshot::build(
            backend()?,
            LuaWorkspaceUniverse::Project,
            vec![
                LuaWorkspaceFileInput::new("A.lua", "1234"),
                LuaWorkspaceFileInput::new("B.lua", "5678"),
            ],
            total_limits,
        )
        .err()
        .ok_or_else(|| {
            workspace_error(
                EmmyWorkspaceErrorCode::TotalSizeLimitExceeded,
                "total limit unexpectedly ignored",
                None::<String>,
            )
        })?;
        assert_eq!(total.code(), EmmyWorkspaceErrorCode::TotalSizeLimitExceeded);
        Ok(())
    }
}
