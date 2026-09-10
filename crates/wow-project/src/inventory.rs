use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, NormalizedSourcePath, SourceContent};
use wow_emmy::{LuaWorkspaceFileInput, LuaWorkspaceSnapshot};

use crate::identity::{canonical_digest, source_digest};
use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileId, ProjectPhase,
    ProjectResult,
};

/// E0 project language class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLanguageKind {
    Lua,
    Other,
}

/// E0 project source role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectFileRole {
    FirstPartyMain,
    Library,
}

/// Canonical public file-manifest entry. Source bytes are intentionally absent.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectFileManifestEntry {
    file_id: ProjectFileId,
    relative_path: NormalizedSourcePath,
    language_kind: ProjectLanguageKind,
    role: ProjectFileRole,
    content_digest: ContentDigest<SourceContent>,
    byte_length: u64,
}

impl ProjectFileManifestEntry {
    #[must_use]
    pub const fn file_id(&self) -> &ProjectFileId {
        &self.file_id
    }

    #[must_use]
    pub const fn relative_path(&self) -> &NormalizedSourcePath {
        &self.relative_path
    }

    #[must_use]
    pub const fn language_kind(&self) -> ProjectLanguageKind {
        self.language_kind
    }

    #[must_use]
    pub const fn role(&self) -> ProjectFileRole {
        self.role
    }

    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.content_digest
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }
}

/// Explicit caller-supplied first-party file. It never opens a filesystem path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInputFile {
    manifest: ProjectFileManifestEntry,
    text: Box<str>,
    source_fixture_ref: Option<Box<str>>,
}

impl ProjectInputFile {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> ProjectResult<Self> {
        Self::declared(
            path,
            text,
            ProjectLanguageKind::Lua,
            ProjectFileRole::FirstPartyMain,
            None::<String>,
        )
    }

    pub fn declared(
        path: impl Into<String>,
        text: impl Into<String>,
        language_kind: ProjectLanguageKind,
        role: ProjectFileRole,
        source_fixture_ref: Option<impl Into<String>>,
    ) -> ProjectResult<Self> {
        let path = path.into();
        let parsed = NormalizedSourcePath::parse(&path).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::InvalidFilePath,
                ProjectPhase::Inventory,
                "project input path is not a safe relative path",
            )
            .with_relative_path(path.as_str())
        })?;
        if !parsed.was_canonical() || parsed.value().as_str() != path {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidFilePath,
                ProjectPhase::Inventory,
                "project input path is not canonical",
            )
            .with_relative_path(path));
        }
        let relative_path = parsed.into_value();
        let file_id = ProjectFileId::from_path(&relative_path)?;
        let text = text.into();
        if text.contains('\0') {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidEncoding,
                ProjectPhase::Inventory,
                "project source contains a NUL character",
            )
            .with_file_id(file_id.as_str())
            .with_relative_path(relative_path.as_str()));
        }
        let byte_length = u64::try_from(text.len()).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::FileLengthMismatch,
                ProjectPhase::Inventory,
                "project source byte length exceeds u64",
            )
            .with_file_id(file_id.as_str())
        })?;
        let content_digest = source_digest(text.as_bytes());
        let source_fixture_ref = source_fixture_ref
            .map(Into::into)
            .map(validate_fixture_ref)
            .transpose()?;
        Ok(Self {
            manifest: ProjectFileManifestEntry {
                file_id,
                relative_path,
                language_kind,
                role,
                content_digest,
                byte_length,
            },
            text: text.into_boxed_str(),
            source_fixture_ref,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn declared_with_identity(
        path: impl Into<String>,
        bytes: Vec<u8>,
        language_kind: ProjectLanguageKind,
        role: ProjectFileRole,
        expected_digest: ContentDigest<SourceContent>,
        expected_byte_length: u64,
        source_fixture_ref: Option<impl Into<String>>,
    ) -> ProjectResult<Self> {
        let text = String::from_utf8(bytes).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::InvalidEncoding,
                ProjectPhase::Inventory,
                "project source is not valid UTF-8",
            )
        })?;
        let value = Self::declared(path, text, language_kind, role, source_fixture_ref)?;
        if value.manifest.content_digest != expected_digest {
            return Err(ProjectError::new(
                ProjectErrorCode::FileDigestMismatch,
                ProjectPhase::Inventory,
                "declared project source digest does not match exact bytes",
            )
            .with_file_id(value.file_id().as_str())
            .with_relative_path(value.relative_path().as_str()));
        }
        if value.manifest.byte_length != expected_byte_length {
            return Err(ProjectError::new(
                ProjectErrorCode::FileLengthMismatch,
                ProjectPhase::Inventory,
                "declared project source length does not match exact bytes",
            )
            .with_file_id(value.file_id().as_str())
            .with_relative_path(value.relative_path().as_str()));
        }
        Ok(value)
    }

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
    pub const fn language_kind(&self) -> ProjectLanguageKind {
        self.manifest.language_kind()
    }

    #[must_use]
    pub const fn role(&self) -> ProjectFileRole {
        self.manifest.role()
    }

    #[must_use]
    pub fn source_fixture_ref(&self) -> Option<&str> {
        self.source_fixture_ref.as_deref()
    }

    pub(crate) fn workspace_input(&self) -> LuaWorkspaceFileInput {
        LuaWorkspaceFileInput::new(self.relative_path().as_str(), self.text.as_ref())
    }
}

/// Validated closed project input inventory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInputInventory {
    declared_file_ids: Vec<ProjectFileId>,
    files: Vec<ProjectInputFile>,
    manifest_digest: ContentDigest<CanonicalResult>,
    total_source_bytes: u64,
}

impl ProjectInputInventory {
    pub fn build(
        configuration: &ProjectConfiguration,
        declared_paths: Vec<String>,
        mut files: Vec<ProjectInputFile>,
    ) -> ProjectResult<Self> {
        configuration.validate()?;
        let budget = configuration.budget_policy();
        if files.is_empty() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidInputInventory,
                ProjectPhase::Inventory,
                "project input inventory is empty",
            ));
        }
        let file_count = u64::try_from(files.len()).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::UpdateBudgetExceeded,
                ProjectPhase::Inventory,
                "project file count exceeds u64",
            )
        })?;
        if file_count > budget.max_files() {
            return Err(ProjectError::new(
                ProjectErrorCode::UpdateBudgetExceeded,
                ProjectPhase::Inventory,
                "project input inventory exceeds the file-count budget",
            ));
        }

        let mut declarations = BTreeMap::<ProjectFileId, NormalizedSourcePath>::new();
        let mut folded_declarations = BTreeMap::<String, String>::new();
        for candidate in declared_paths {
            let parsed = NormalizedSourcePath::parse(&candidate).map_err(|_| {
                ProjectError::new(
                    ProjectErrorCode::InvalidFilePath,
                    ProjectPhase::Inventory,
                    "declared project path is invalid",
                )
                .with_relative_path(candidate.as_str())
            })?;
            if !parsed.was_canonical() || parsed.value().as_str() != candidate {
                return Err(ProjectError::new(
                    ProjectErrorCode::InvalidFilePath,
                    ProjectPhase::Inventory,
                    "declared project path is not canonical",
                )
                .with_relative_path(candidate));
            }
            let path = parsed.into_value();
            let file_id = ProjectFileId::from_path(&path)?;
            if declarations.insert(file_id.clone(), path.clone()).is_some() {
                return Err(ProjectError::new(
                    ProjectErrorCode::InvalidInputInventory,
                    ProjectPhase::Inventory,
                    "project input declaration contains a duplicate file",
                )
                .with_file_id(file_id.as_str()));
            }
            let folded = path.as_str().to_ascii_lowercase();
            if let Some(existing) = folded_declarations.insert(folded, path.as_str().to_owned()) {
                return Err(ProjectError::new(
                    ProjectErrorCode::FileCaseCollision,
                    ProjectPhase::Inventory,
                    format!("declared paths collide under case folding with {existing:?}"),
                )
                .with_relative_path(path.as_str()));
            }
        }
        if declarations.is_empty() {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidInputInventory,
                ProjectPhase::Inventory,
                "project input declaration is empty",
            ));
        }

        files.sort_by(|left, right| left.manifest.cmp(&right.manifest));
        let mut observed_ids = BTreeSet::new();
        let mut observed_paths = BTreeSet::new();
        let mut folded_paths = BTreeMap::<String, String>::new();
        let mut total_source_bytes = 0_u64;
        for file in &files {
            validate_e0_file(configuration, file)?;
            if !declarations.contains_key(file.file_id()) {
                return Err(ProjectError::new(
                    ProjectErrorCode::UndeclaredFile,
                    ProjectPhase::Inventory,
                    "project input contains an undeclared file",
                )
                .with_file_id(file.file_id().as_str())
                .with_relative_path(file.relative_path().as_str()));
            }
            if !observed_ids.insert(file.file_id().clone())
                || !observed_paths.insert(file.relative_path().clone())
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::InvalidInputInventory,
                    ProjectPhase::Inventory,
                    "project input contains duplicate file identity",
                )
                .with_file_id(file.file_id().as_str()));
            }
            let folded = file.relative_path().as_str().to_ascii_lowercase();
            if let Some(existing) =
                folded_paths.insert(folded, file.relative_path().as_str().to_owned())
            {
                return Err(ProjectError::new(
                    ProjectErrorCode::FileCaseCollision,
                    ProjectPhase::Inventory,
                    format!("project paths collide under case folding with {existing:?}"),
                )
                .with_relative_path(file.relative_path().as_str()));
            }
            if file.byte_length() > budget.max_single_file_bytes() {
                return Err(ProjectError::new(
                    ProjectErrorCode::UpdateBudgetExceeded,
                    ProjectPhase::Inventory,
                    "project source exceeds the per-file byte budget",
                )
                .with_file_id(file.file_id().as_str()));
            }
            total_source_bytes = total_source_bytes
                .checked_add(file.byte_length())
                .ok_or_else(|| {
                    ProjectError::new(
                        ProjectErrorCode::UpdateBudgetExceeded,
                        ProjectPhase::Inventory,
                        "project source byte count overflowed",
                    )
                })?;
            if total_source_bytes > budget.max_total_source_bytes() {
                return Err(ProjectError::new(
                    ProjectErrorCode::UpdateBudgetExceeded,
                    ProjectPhase::Inventory,
                    "project input exceeds the aggregate source byte budget",
                ));
            }
        }
        for (file_id, path) in &declarations {
            if !observed_ids.contains(file_id) {
                return Err(ProjectError::new(
                    ProjectErrorCode::MissingDeclaredFile,
                    ProjectPhase::Inventory,
                    "declared project file is missing from supplied inputs",
                )
                .with_file_id(file_id.as_str())
                .with_relative_path(path.as_str()));
            }
        }

        let declared_file_ids = declarations.into_keys().collect::<Vec<_>>();
        let manifest = files
            .iter()
            .map(|file| file.manifest.clone())
            .collect::<Vec<_>>();
        #[derive(Serialize)]
        struct Identity<'a> {
            schema_version: u64,
            project_id: &'a crate::ProjectId,
            workspace_id: &'a crate::ProjectWorkspaceId,
            source_origin_id: &'a crate::ProjectSourceOriginId,
            declared_file_ids: &'a [ProjectFileId],
            files: &'a [ProjectFileManifestEntry],
            total_source_bytes: u64,
        }
        let manifest_digest = canonical_digest(
            "wow-project/input-inventory/e0-d/1",
            &Identity {
                schema_version: 1,
                project_id: configuration.project_id(),
                workspace_id: configuration.workspace_id(),
                source_origin_id: configuration.source_origin_id(),
                declared_file_ids: &declared_file_ids,
                files: &manifest,
                total_source_bytes,
            },
            ProjectPhase::Inventory,
        )?;
        Ok(Self {
            declared_file_ids,
            files,
            manifest_digest,
            total_source_bytes,
        })
    }

    #[must_use]
    pub fn declared_file_ids(&self) -> &[ProjectFileId] {
        &self.declared_file_ids
    }

    #[must_use]
    pub fn files(&self) -> &[ProjectInputFile] {
        &self.files
    }

    #[must_use]
    pub fn manifest_entries(&self) -> Vec<ProjectFileManifestEntry> {
        self.files
            .iter()
            .map(|file| file.manifest.clone())
            .collect()
    }

    #[must_use]
    pub const fn manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.manifest_digest
    }

    #[must_use]
    pub const fn total_source_bytes(&self) -> u64 {
        self.total_source_bytes
    }

    #[must_use]
    pub fn file_by_id(&self, file_id: &ProjectFileId) -> Option<&ProjectInputFile> {
        self.files.iter().find(|file| file.file_id() == file_id)
    }

    #[must_use]
    pub fn file_by_path(&self, path: &NormalizedSourcePath) -> Option<&ProjectInputFile> {
        self.files.iter().find(|file| file.relative_path() == path)
    }
}

/// Closed initial publication input: configuration, declared Main inventory,
/// and explicit normalized Library snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectInputBundle {
    configuration: ProjectConfiguration,
    inventory: ProjectInputInventory,
    libraries: Vec<LuaWorkspaceSnapshot>,
}

impl ProjectInputBundle {
    pub fn new(
        configuration: ProjectConfiguration,
        declared_paths: Vec<String>,
        files: Vec<ProjectInputFile>,
        libraries: Vec<LuaWorkspaceSnapshot>,
    ) -> ProjectResult<Self> {
        let inventory = ProjectInputInventory::build(&configuration, declared_paths, files)?;
        Ok(Self {
            configuration,
            inventory,
            libraries,
        })
    }

    pub fn closed(
        configuration: ProjectConfiguration,
        files: Vec<ProjectInputFile>,
        libraries: Vec<LuaWorkspaceSnapshot>,
    ) -> ProjectResult<Self> {
        let declared_paths = files
            .iter()
            .map(|file| file.relative_path().as_str().to_owned())
            .collect();
        Self::new(configuration, declared_paths, files, libraries)
    }

    #[must_use]
    pub const fn configuration(&self) -> &ProjectConfiguration {
        &self.configuration
    }

    #[must_use]
    pub const fn inventory(&self) -> &ProjectInputInventory {
        &self.inventory
    }

    #[must_use]
    pub fn libraries(&self) -> &[LuaWorkspaceSnapshot] {
        &self.libraries
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        ProjectConfiguration,
        ProjectInputInventory,
        Vec<LuaWorkspaceSnapshot>,
    ) {
        (self.configuration, self.inventory, self.libraries)
    }
}

fn validate_e0_file(
    configuration: &ProjectConfiguration,
    file: &ProjectInputFile,
) -> ProjectResult<()> {
    if file.language_kind() != ProjectLanguageKind::Lua {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidFileLanguage,
            ProjectPhase::Inventory,
            "E0 project inventory supports Lua sources only",
        )
        .with_file_id(file.file_id().as_str()));
    }
    if file.role() != ProjectFileRole::FirstPartyMain {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidFileRole,
            ProjectPhase::Inventory,
            "Library sources cannot enter the first-party project inventory",
        )
        .with_file_id(file.file_id().as_str()));
    }
    if let Some(fixture_ref) = file.source_fixture_ref() {
        if configuration.project_kind() != crate::ProjectKind::Fixture {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidInputInventory,
                ProjectPhase::Inventory,
                "repository project input cannot claim fixture-source provenance",
            )
            .with_file_id(file.file_id().as_str()));
        }
        let path = file.relative_path().as_str();
        let known_fixture_member = matches!(
            path,
            "main/clean.lua"
                | "main/generic-error.lua"
                | "main/missing-api.lua"
                | "main/secret-local.lua"
        );
        let expected = format!("wow-emmy/workspace-fixture:{path}");
        if !known_fixture_member || fixture_ref != expected {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidInputInventory,
                ProjectPhase::Inventory,
                "source fixture reference does not identify the exact E0-C Main member",
            )
            .with_file_id(file.file_id().as_str())
            .with_relative_path(path));
        }
    }
    if !file
        .relative_path()
        .as_str()
        .rsplit_once('.')
        .is_some_and(|(_, extension)| extension == "lua")
    {
        return Err(ProjectError::new(
            ProjectErrorCode::InvalidFileLanguage,
            ProjectPhase::Inventory,
            "E0 project path must use the canonical .lua extension",
        )
        .with_relative_path(file.relative_path().as_str()));
    }
    Ok(())
}

fn validate_fixture_ref(value: String) -> ProjectResult<Box<str>> {
    let valid = !value.is_empty()
        && value.len() <= 1024
        && value.trim() == value
        && !value.chars().any(char::is_control)
        && !value.contains("://")
        && !value.starts_with('/')
        && !value.starts_with('\\');
    if valid {
        Ok(value.into_boxed_str())
    } else {
        Err(ProjectError::new(
            ProjectErrorCode::InvalidInputInventory,
            ProjectPhase::Inventory,
            "source fixture reference is unsafe or noncanonical",
        ))
    }
}
