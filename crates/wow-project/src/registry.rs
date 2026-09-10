use std::collections::BTreeMap;

use serde::Serialize;
use wow_core::{
    CanonicalResult, ContentDigest, EntityKey, NormalizedSourcePath, ProjectGenerationId,
    ReferenceGenerationId, SourceContent, SourceHandle, SourceHandleBuilder, SourceOriginKind,
    SourceSpan,
};

use crate::identity::{canonical_digest, canonical_id};
use crate::{
    ProjectAnalyzerBinding, ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileId,
    ProjectFileManifestEntry, ProjectInputInventory, ProjectKind, ProjectPhase, ProjectResult,
    ProjectSourceOriginId, ProjectWorkspaceId,
};

/// Project-owned source-origin class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSourceOriginKind {
    FixtureProject,
    RepositoryProject,
}

/// Immutable project source-origin declaration bound to one generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceOrigin {
    origin_id: ProjectSourceOriginId,
    project_id: crate::ProjectId,
    workspace_id: ProjectWorkspaceId,
    origin_kind: ProjectSourceOriginKind,
    logical_root: NormalizedSourcePath,
    revision_identity: Box<str>,
    project_generation: ProjectGenerationId,
}

impl ProjectSourceOrigin {
    #[must_use]
    pub const fn origin_id(&self) -> &ProjectSourceOriginId {
        &self.origin_id
    }

    #[must_use]
    pub const fn project_id(&self) -> &crate::ProjectId {
        &self.project_id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &ProjectWorkspaceId {
        &self.workspace_id
    }

    #[must_use]
    pub const fn origin_kind(&self) -> ProjectSourceOriginKind {
        self.origin_kind
    }

    #[must_use]
    pub const fn logical_root(&self) -> &NormalizedSourcePath {
        &self.logical_root
    }

    #[must_use]
    pub fn revision_identity(&self) -> &str {
        &self.revision_identity
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }
}

/// Published project file/source record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileRecord {
    manifest: ProjectFileManifestEntry,
    source_origin_id: ProjectSourceOriginId,
    workspace_id: ProjectWorkspaceId,
    project_generation: ProjectGenerationId,
    analyzer_file_id: Box<str>,
    source_handle_base: SourceHandle,
}

impl ProjectFileRecord {
    #[must_use]
    pub const fn manifest(&self) -> &ProjectFileManifestEntry {
        &self.manifest
    }

    #[must_use]
    pub const fn file_id(&self) -> &ProjectFileId {
        self.manifest.file_id()
    }

    #[must_use]
    pub const fn relative_path(&self) -> &NormalizedSourcePath {
        self.manifest.relative_path()
    }

    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.manifest.content_digest()
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.manifest.byte_length()
    }

    #[must_use]
    pub const fn source_origin_id(&self) -> &ProjectSourceOriginId {
        &self.source_origin_id
    }

    #[must_use]
    pub const fn workspace_id(&self) -> &ProjectWorkspaceId {
        &self.workspace_id
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    #[must_use]
    pub fn analyzer_file_id(&self) -> &str {
        &self.analyzer_file_id
    }

    #[must_use]
    pub const fn source_handle_base(&self) -> &SourceHandle {
        &self.source_handle_base
    }
}

/// Immutable source registry for one published project generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectSourceRegistry {
    registry_id: Box<str>,
    canonical_digest: ContentDigest<CanonicalResult>,
    project_generation: ProjectGenerationId,
    reference_generation: ReferenceGenerationId,
    source_origin: ProjectSourceOrigin,
    file_records: Vec<ProjectFileRecord>,
}

impl ProjectSourceRegistry {
    pub(crate) fn build(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        analyzer: &ProjectAnalyzerBinding,
        project_generation: ProjectGenerationId,
    ) -> ProjectResult<Self> {
        if analyzer.project_generation() != project_generation {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::Registry,
                "analyzer binding belongs to a different project generation",
            )
            .with_candidate_generation(project_generation));
        }
        let origin_kind = match configuration.project_kind() {
            ProjectKind::Fixture => ProjectSourceOriginKind::FixtureProject,
            ProjectKind::Repository => ProjectSourceOriginKind::RepositoryProject,
        };
        let source_origin = ProjectSourceOrigin {
            origin_id: configuration.source_origin_id().clone(),
            project_id: configuration.project_id().clone(),
            workspace_id: configuration.workspace_id().clone(),
            origin_kind,
            logical_root: configuration.logical_root().clone(),
            revision_identity: project_generation.canonical().into_boxed_str(),
            project_generation,
        };
        let mut file_records = Vec::with_capacity(inventory.files().len());
        for file in inventory.files() {
            let entity_key =
                EntityKey::new("project.file", file.file_id().as_str()).map_err(|source| {
                    ProjectError::new(
                        ProjectErrorCode::SourceRegistryInvalid,
                        ProjectPhase::Registry,
                        format!("project file entity identity is invalid: {source}"),
                    )
                    .with_file_id(file.file_id().as_str())
                })?;
            let core_origin_kind = match configuration.project_kind() {
                ProjectKind::Fixture => SourceOriginKind::Fixture,
                ProjectKind::Repository => SourceOriginKind::GeneratedArtifact,
            };
            let source_handle_base = SourceHandleBuilder::new(
                core_origin_kind,
                configuration.source_origin_id().as_str(),
                project_generation.canonical(),
                file.relative_path().as_str(),
                SourceSpan::whole_file(),
                file.content_digest(),
            )
            .reference_generation(configuration.reference_generation())
            .project_generation(project_generation)
            .entity_key(entity_key)
            .build()
            .map_err(|source| {
                ProjectError::new(
                    ProjectErrorCode::SourceHandleInvalid,
                    ProjectPhase::Registry,
                    format!("project source handle cannot be built: {source}"),
                )
                .with_file_id(file.file_id().as_str())
                .with_relative_path(file.relative_path().as_str())
            })?;
            #[derive(Serialize)]
            struct AnalyzerFileIdentity<'a> {
                project_generation: ProjectGenerationId,
                main_workspace_id: &'a str,
                file_id: &'a ProjectFileId,
                path: &'a NormalizedSourcePath,
                content_digest: ContentDigest<SourceContent>,
            }
            let analyzer_file_id = canonical_id(
                "project-analyzer-file:sha256:",
                "wow-project/analyzer-file/e0-d/1",
                &AnalyzerFileIdentity {
                    project_generation,
                    main_workspace_id: analyzer.main_workspace().snapshot_id(),
                    file_id: file.file_id(),
                    path: file.relative_path(),
                    content_digest: file.content_digest(),
                },
                ProjectPhase::Registry,
            )?;
            file_records.push(ProjectFileRecord {
                manifest: file.manifest().clone(),
                source_origin_id: configuration.source_origin_id().clone(),
                workspace_id: configuration.workspace_id().clone(),
                project_generation,
                analyzer_file_id,
                source_handle_base,
            });
        }
        file_records.sort_by(|left, right| {
            left.file_id()
                .cmp(right.file_id())
                .then(left.relative_path().cmp(right.relative_path()))
        });
        #[derive(Serialize)]
        struct Identity<'a> {
            schema_version: u64,
            project_id: &'a crate::ProjectId,
            project_generation: ProjectGenerationId,
            reference_generation: ReferenceGenerationId,
            source_origin: &'a ProjectSourceOrigin,
            file_records: &'a [ProjectFileRecord],
        }
        let identity = Identity {
            schema_version: 1,
            project_id: configuration.project_id(),
            project_generation,
            reference_generation: configuration.reference_generation(),
            source_origin: &source_origin,
            file_records: &file_records,
        };
        let canonical_digest = canonical_digest(
            "wow-project/source-registry/e0-d/1",
            &identity,
            ProjectPhase::Registry,
        )?;
        let registry_id = canonical_id(
            "project-source-registry:sha256:",
            "wow-project/source-registry-id/e0-d/1",
            &identity,
            ProjectPhase::Registry,
        )?;
        let registry = Self {
            registry_id,
            canonical_digest,
            project_generation,
            reference_generation: configuration.reference_generation(),
            source_origin,
            file_records,
        };
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> ProjectResult<()> {
        let mut ids = BTreeMap::new();
        let mut paths = BTreeMap::new();
        for record in &self.file_records {
            record.source_handle_base.validate().map_err(|source| {
                ProjectError::new(
                    ProjectErrorCode::SourceHandleInvalid,
                    ProjectPhase::Registry,
                    format!("project source handle failed validation: {source}"),
                )
                .with_file_id(record.file_id().as_str())
            })?;
            if record.project_generation != self.project_generation
                || record.source_origin_id != self.source_origin.origin_id
                || record.workspace_id != self.source_origin.workspace_id
                || record.source_handle_base.project_generation() != Some(self.project_generation)
                || record.source_handle_base.reference_generation()
                    != Some(self.reference_generation)
                || record.source_handle_base.path() != record.relative_path()
                || record.source_handle_base.content_digest() != &record.content_digest()
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::SourceRegistryInvalid,
                    ProjectPhase::Registry,
                    "project file record and source handle identities disagree",
                )
                .with_file_id(record.file_id().as_str()));
            }
            if ids.insert(record.file_id().clone(), ()).is_some()
                || paths.insert(record.relative_path().clone(), ()).is_some()
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::SourceRegistryInvalid,
                    ProjectPhase::Registry,
                    "project source registry contains duplicate file identity",
                )
                .with_file_id(record.file_id().as_str()));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn registry_id(&self) -> &str {
        &self.registry_id
    }

    #[must_use]
    pub const fn canonical_digest(&self) -> ContentDigest<CanonicalResult> {
        self.canonical_digest
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    #[must_use]
    pub const fn source_origin(&self) -> &ProjectSourceOrigin {
        &self.source_origin
    }

    #[must_use]
    pub fn file_records(&self) -> &[ProjectFileRecord] {
        &self.file_records
    }

    #[must_use]
    pub fn file_by_id(&self, file_id: &ProjectFileId) -> Option<&ProjectFileRecord> {
        self.file_records
            .iter()
            .find(|record| record.file_id() == file_id)
    }

    pub fn file_by_path(&self, path: &str) -> ProjectResult<Option<&ProjectFileRecord>> {
        let parsed = NormalizedSourcePath::parse(path).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::InvalidFilePath,
                ProjectPhase::View,
                "project file lookup path is invalid",
            )
            .with_relative_path(path)
        })?;
        if !parsed.was_canonical() || parsed.value().as_str() != path {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidFilePath,
                ProjectPhase::View,
                "project file lookup path is not canonical",
            )
            .with_relative_path(path));
        }
        Ok(self
            .file_records
            .iter()
            .find(|record| record.relative_path() == parsed.value()))
    }

    pub fn validate_source_handle(&self, handle: &SourceHandle) -> ProjectResult<()> {
        handle.validate().map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::SourceHandleInvalid,
                ProjectPhase::Registry,
                format!("source handle is invalid: {source}"),
            )
        })?;
        let record = self
            .file_records
            .iter()
            .find(|record| record.source_handle_base().handle_id() == handle.handle_id())
            .ok_or_else(|| {
                ProjectError::new(
                    ProjectErrorCode::SourceHandleInvalid,
                    ProjectPhase::Registry,
                    "source handle does not belong to this project generation",
                )
            })?;
        if record.source_handle_base() == handle {
            Ok(())
        } else {
            Err(ProjectError::new(
                ProjectErrorCode::SourceHandleInvalid,
                ProjectPhase::Registry,
                "source handle fields differ from the registered file identity",
            )
            .with_file_id(record.file_id().as_str()))
        }
    }

    #[must_use]
    pub fn contains_current_file(
        &self,
        file_id: &ProjectFileId,
        digest: ContentDigest<SourceContent>,
    ) -> bool {
        self.file_by_id(file_id)
            .is_some_and(|record| record.content_digest() == digest)
    }
}
