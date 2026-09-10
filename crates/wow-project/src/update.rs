use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

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
    Vec<LuaWorkspaceSnapshot>,
);

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
    target_libraries: Vec<LuaWorkspaceSnapshot>,
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
            target_libraries: Vec::new(),
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

    #[must_use]
    pub fn with_target_libraries(mut self, libraries: Vec<LuaWorkspaceSnapshot>) -> Self {
        self.target_libraries = libraries;
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

    #[must_use]
    pub fn target_libraries(&self) -> &[LuaWorkspaceSnapshot] {
        &self.target_libraries
    }

    pub(crate) fn into_parts(self) -> ProjectUpdateParts {
        (
            self.expected_current_project_generation,
            self.expected_current_snapshot_digest,
            self.target_configuration,
            self.file_operations,
            self.target_libraries,
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
) -> ProjectResult<Vec<ProjectInputFile>> {
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
    let mut files = current_files
        .iter()
        .cloned()
        .map(|file| (file.file_id().clone(), file))
        .collect::<BTreeMap<_, _>>();
    for operation in operations {
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
