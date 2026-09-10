use std::fmt;

use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId};

/// Stable project transaction phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectPhase {
    Configuration,
    Inventory,
    Generation,
    Analyzer,
    Registry,
    Snapshot,
    Publication,
    Update,
    View,
}

/// Stable `wow-project` failure class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectErrorCode {
    InvalidConfiguration,
    FixtureReleaseMasquerade,
    FloatingIdentity,
    InvalidAnalyzerBinding,
    InvalidCapabilityPolicy,
    InvalidBudgetPolicy,
    InvalidInputInventory,
    MissingDeclaredFile,
    UndeclaredFile,
    InvalidFileId,
    InvalidFilePath,
    FileCaseCollision,
    InvalidFileRole,
    InvalidFileLanguage,
    InvalidEncoding,
    FileDigestMismatch,
    FileLengthMismatch,
    SourceRegistryInvalid,
    SourceHandleInvalid,
    GenerationDerivationFailed,
    ExpectedGenerationMismatch,
    ExpectedSnapshotDigestMismatch,
    GenerationCollision,
    UpdateRequestInvalid,
    ConflictingOperations,
    AddExistingFile,
    UpdateTargetMissing,
    ExpectedFileDigestMismatch,
    UpdateBudgetExceeded,
    AnalyzerFailed,
    AnalyzerSnapshotMismatch,
    AnalyzerManifestMismatch,
    MandatoryCapabilityUnavailable,
    SnapshotInvalid,
    SnapshotDigestMismatch,
    PublicationAborted,
    AlreadyPublished,
    NoPublishedSnapshot,
    FileNotPresent,
    DeferredCapability,
    CanonicalizationFailed,
}

/// One bounded project failure. It never embeds source text or host paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectError {
    code: ProjectErrorCode,
    phase: ProjectPhase,
    message: Box<str>,
    file_id: Option<Box<str>>,
    relative_path: Option<Box<str>>,
    candidate_generation: Option<ProjectGenerationId>,
    current_generation: Option<ProjectGenerationId>,
    current_snapshot_digest: Option<ContentDigest<CanonicalResult>>,
}

impl ProjectError {
    #[must_use]
    pub(crate) fn new(
        code: ProjectErrorCode,
        phase: ProjectPhase,
        message: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            phase,
            message: message.into(),
            file_id: None,
            relative_path: None,
            candidate_generation: None,
            current_generation: None,
            current_snapshot_digest: None,
        }
    }

    #[must_use]
    pub(crate) fn with_file_id(mut self, file_id: impl Into<Box<str>>) -> Self {
        self.file_id = Some(file_id.into());
        self
    }

    #[must_use]
    pub(crate) fn with_relative_path(mut self, path: impl Into<Box<str>>) -> Self {
        self.relative_path = Some(path.into());
        self
    }

    #[must_use]
    pub(crate) const fn with_candidate_generation(
        mut self,
        generation: ProjectGenerationId,
    ) -> Self {
        self.candidate_generation = Some(generation);
        self
    }

    #[must_use]
    pub(crate) const fn with_current(
        mut self,
        generation: ProjectGenerationId,
        snapshot_digest: ContentDigest<CanonicalResult>,
    ) -> Self {
        self.current_generation = Some(generation);
        self.current_snapshot_digest = Some(snapshot_digest);
        self
    }

    /// Stable failure code.
    #[must_use]
    pub const fn code(&self) -> ProjectErrorCode {
        self.code
    }

    /// Transaction phase in which the failure was detected.
    #[must_use]
    pub const fn phase(&self) -> ProjectPhase {
        self.phase
    }

    /// Safe human-readable explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Exact logical file ID, when applicable.
    #[must_use]
    pub fn file_id(&self) -> Option<&str> {
        self.file_id.as_deref()
    }

    /// Exact logical relative path, when applicable.
    #[must_use]
    pub fn relative_path(&self) -> Option<&str> {
        self.relative_path.as_deref()
    }

    /// Candidate generation when derivation safely completed before failure.
    #[must_use]
    pub const fn candidate_generation(&self) -> Option<ProjectGenerationId> {
        self.candidate_generation
    }

    /// Current published generation retained after failure.
    #[must_use]
    pub const fn current_generation(&self) -> Option<ProjectGenerationId> {
        self.current_generation
    }

    /// Current immutable snapshot digest retained after failure.
    #[must_use]
    pub const fn current_snapshot_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.current_snapshot_digest
    }
}

impl fmt::Display for ProjectError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(file_id) = self.file_id() {
            write!(formatter, " [{file_id}]")?;
        }
        if let Some(path) = self.relative_path() {
            write!(formatter, " ({path})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ProjectError {}

/// Result type for all project operations.
pub type ProjectResult<T> = Result<T, ProjectError>;
