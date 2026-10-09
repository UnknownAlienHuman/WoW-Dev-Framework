use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent};
use wow_emmy::LuaWorkspaceSnapshot;

use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileId, ProjectInputFile,
    ProjectPhase, ProjectResult, ProjectSnapshot,
};

type ProjectUpdateParts = (
    Option<ProjectGenerationId>,
    Option<ContentDigest<CanonicalResult>>,
    ProjectConfiguration,
    Vec<ProjectFileOperation>,
    ProjectLibraryOperation,
);

/// Exact intent for the analyzer Library inputs of one update transaction.
///
/// `Keep` retains the publisher's current libraries, `Replace` installs the
/// listed snapshots, and `Clear` drops every library. A bare `Vec` request cannot
/// distinguish retaining from dropping, which is why this type exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectLibraryOperation {
    Keep,
    Replace(Vec<LuaWorkspaceSnapshot>),
    Clear,
}

impl ProjectLibraryOperation {
    /// Library snapshots this operation installs, if any.
    #[must_use]
    pub fn snapshots(&self) -> &[LuaWorkspaceSnapshot] {
        match self {
            Self::Replace(snapshots) => snapshots,
            Self::Keep | Self::Clear => &[],
        }
    }
}

/// Explicit E0 project-file operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectFileOperation {
    Add(ProjectInputFile),
    Update {
        file_id: ProjectFileId,
        expected_old_digest: ContentDigest<SourceContent>,
        new_text: Box<str>,
    },
    Remove {
        file_id: ProjectFileId,
        expected_old_digest: ContentDigest<SourceContent>,
    },
}

impl ProjectFileOperation {
    #[must_use]
    pub fn add(file: ProjectInputFile) -> Self {
        Self::Add(file)
    }

    #[must_use]
    pub fn update(
        file_id: ProjectFileId,
        expected_old_digest: ContentDigest<SourceContent>,
        new_text: impl Into<String>,
    ) -> Self {
        Self::Update {
            file_id,
            expected_old_digest,
            new_text: new_text.into().into_boxed_str(),
        }
    }

    #[must_use]
    pub const fn remove(
        file_id: ProjectFileId,
        expected_old_digest: ContentDigest<SourceContent>,
    ) -> Self {
        Self::Remove {
            file_id,
            expected_old_digest,
        }
    }

    #[must_use]
    pub fn file_id(&self) -> &ProjectFileId {
        match self {
            Self::Add(file) => file.file_id(),
            Self::Update { file_id, .. } | Self::Remove { file_id, .. } => file_id,
        }
    }

    fn rank(&self) -> u8 {
        match self {
            Self::Add(_) => 0,
            Self::Update { .. } => 1,
            Self::Remove { .. } => 2,
        }
    }
}

/// One explicit update transaction request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectUpdateRequest {
    expected_current_project_generation: Option<ProjectGenerationId>,
    expected_current_snapshot_digest: Option<ContentDigest<CanonicalResult>>,
    target_configuration: ProjectConfiguration,
    file_operations: Vec<ProjectFileOperation>,
    library_operation: ProjectLibraryOperation,
}

impl ProjectUpdateRequest {
    #[must_use]
    pub fn new(
        target_configuration: ProjectConfiguration,
        file_operations: Vec<ProjectFileOperation>,
    ) -> Self {
        Self {
            expected_current_project_generation: None,
            expected_current_snapshot_digest: None,
            target_configuration,
            file_operations,
            library_operation: ProjectLibraryOperation::Keep,
        }
    }

    #[must_use]
    pub const fn expected_generation(mut self, generation: ProjectGenerationId) -> Self {
        self.expected_current_project_generation = Some(generation);
        self
    }

    #[must_use]
    pub const fn expected_snapshot_digest(
        mut self,
        digest: ContentDigest<CanonicalResult>,
    ) -> Self {
        self.expected_current_snapshot_digest = Some(digest);
        self
    }

    /// Legacy conversion: an empty vector retains current libraries. Use
    /// `with_library_operation` to request an explicit empty replacement/clear.
    #[must_use]
    pub fn with_target_libraries(mut self, libraries: Vec<LuaWorkspaceSnapshot>) -> Self {
        // Legacy compatibility: an empty vector historically meant "unchanged",
        // so it must not silently become a request to clear every library.
        self.library_operation = if libraries.is_empty() {
            ProjectLibraryOperation::Keep
        } else {
            ProjectLibraryOperation::Replace(libraries)
        };
        self
    }

    /// Explicit typed intent. Prefer this over `with_target_libraries`, which
    /// cannot express the difference between keeping and clearing libraries.
    #[must_use]
    pub fn with_library_operation(mut self, operation: ProjectLibraryOperation) -> Self {
        self.library_operation = operation;
        self
    }

    #[must_use]
    pub const fn expected_current_project_generation(&self) -> Option<ProjectGenerationId> {
        self.expected_current_project_generation
    }

    #[must_use]
    pub const fn expected_current_snapshot_digest(&self) -> Option<ContentDigest<CanonicalResult>> {
        self.expected_current_snapshot_digest
    }

    #[must_use]
    pub const fn target_configuration(&self) -> &ProjectConfiguration {
        &self.target_configuration
    }

    #[must_use]
    pub fn file_operations(&self) -> &[ProjectFileOperation] {
        &self.file_operations
    }

    /// Legacy projection. Keep and Clear both have an empty slice; callers that
    /// need exact intent must use `library_operation`.
    #[must_use]
    pub fn target_libraries(&self) -> &[LuaWorkspaceSnapshot] {
        self.library_operation.snapshots()
    }

    /// Exact library intent, including the retain-versus-drop distinction that
    /// the legacy slice cannot carry.
    #[must_use]
    pub const fn library_operation(&self) -> &ProjectLibraryOperation {
        &self.library_operation
    }

    pub(crate) fn into_parts(self) -> ProjectUpdateParts {
        (
            self.expected_current_project_generation,
            self.expected_current_snapshot_digest,
            self.target_configuration,
            self.file_operations,
            self.library_operation,
        )
    }
}

/// Successful update/publication result.
#[derive(Debug, Clone)]
pub enum ProjectUpdateOutcome {
    Published(Arc<ProjectSnapshot>),
    NoChange(Arc<ProjectSnapshot>),
}

impl ProjectUpdateOutcome {
    #[must_use]
    pub fn snapshot(&self) -> &Arc<ProjectSnapshot> {
        match self {
            Self::Published(snapshot) | Self::NoChange(snapshot) => snapshot,
        }
    }

    #[must_use]
    pub const fn changed(&self) -> bool {
        matches!(self, Self::Published(_))
    }
}

pub(crate) fn apply_file_operations(
    current_files: &[ProjectInputFile],
    target_configuration: &ProjectConfiguration,
    mut operations: Vec<ProjectFileOperation>,
    stop: &AtomicBool,
) -> ProjectResult<Vec<ProjectInputFile>> {
    crate::analyzer::checkpoint(stop)?;
    target_configuration.validate()?;
    let operation_count = u64::try_from(operations.len()).map_err(|_| {
        ProjectError::new(
            ProjectErrorCode::UpdateBudgetExceeded,
            ProjectPhase::Update,
            "project update operation count exceeds u64",
        )
    })?;
    if operation_count > target_configuration.budget_policy().max_update_operations() {
        return Err(ProjectError::new(
            ProjectErrorCode::UpdateBudgetExceeded,
            ProjectPhase::Update,
            "project update exceeds the operation-count budget",
        ));
    }
    let mut targets = BTreeSet::new();
    for operation in &operations {
        crate::analyzer::checkpoint(stop)?;
        if !targets.insert(operation.file_id().clone()) {
            return Err(ProjectError::new(
                ProjectErrorCode::ConflictingOperations,
                ProjectPhase::Update,
                "project update contains multiple operations for one logical file",
            )
            .with_file_id(operation.file_id().as_str()));
        }
    }
    operations.sort_by(|left, right| {
        left.file_id()
            .cmp(right.file_id())
            .then(left.rank().cmp(&right.rank()))
    });
    crate::analyzer::checkpoint(stop)?;
    let mut files = current_files
        .iter()
        .cloned()
        .map(|file| (file.file_id().clone(), file))
        .collect::<BTreeMap<_, _>>();
    crate::analyzer::checkpoint(stop)?;
    for operation in operations {
        crate::analyzer::checkpoint(stop)?;
        match operation {
            ProjectFileOperation::Add(file) => {
                if files.contains_key(file.file_id()) {
                    return Err(ProjectError::new(
                        ProjectErrorCode::AddExistingFile,
                        ProjectPhase::Update,
                        "project update attempted to add an existing file",
                    )
                    .with_file_id(file.file_id().as_str()));
                }
                files.insert(file.file_id().clone(), file);
            }
            ProjectFileOperation::Update {
                file_id,
                expected_old_digest,
                new_text,
            } => {
                let current = files.get(&file_id).ok_or_else(|| {
                    ProjectError::new(
                        ProjectErrorCode::UpdateTargetMissing,
                        ProjectPhase::Update,
                        "project update target does not exist",
                    )
                    .with_file_id(file_id.as_str())
                })?;
                if current.content_digest() != expected_old_digest {
                    return Err(ProjectError::new(
                        ProjectErrorCode::ExpectedFileDigestMismatch,
                        ProjectPhase::Update,
                        "project update expected old digest does not match current content",
                    )
                    .with_file_id(file_id.as_str()));
                }
                let replacement = ProjectInputFile::declared(
                    current.relative_path().as_str(),
                    new_text.as_ref(),
                    current.language_kind(),
                    current.role(),
                    current.source_fixture_ref().map(str::to_owned),
                )?;
                files.insert(file_id, replacement);
            }
            ProjectFileOperation::Remove {
                file_id,
                expected_old_digest,
            } => {
                let current = files.get(&file_id).ok_or_else(|| {
                    ProjectError::new(
                        ProjectErrorCode::UpdateTargetMissing,
                        ProjectPhase::Update,
                        "project remove target does not exist",
                    )
                    .with_file_id(file_id.as_str())
                })?;
                if current.content_digest() != expected_old_digest {
                    return Err(ProjectError::new(
                        ProjectErrorCode::ExpectedFileDigestMismatch,
                        ProjectPhase::Update,
                        "project remove expected old digest does not match current content",
                    )
                    .with_file_id(file_id.as_str()));
                }
                files.remove(&file_id);
            }
        }
    }
    Ok(files.into_values().collect())
}
