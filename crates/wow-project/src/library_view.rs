//! Borrowed, byte-verified views into one admitted analyzer Library snapshot.
//!
//! Membership here proves only that bytes belong to the exact producer snapshot
//! already named by a project analyzer binding. It grants no Reference or runtime
//! authority and performs no source parsing or execution.

use std::sync::atomic::AtomicBool;

use wow_core::{ProfileKind, ProjectGenerationId};
use wow_emmy::{LuaWorkspaceSnapshot, LuaWorkspaceUniverse};

use crate::disk::{DISK_INVENTORY_MAX_BYTES, DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult, ProjectView};

const LIBRARY_PATH_MAX_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileDescriptor {
    path: Box<str>,
    sha256: Box<str>,
    byte_length: u64,
}

/// Exact Project and analyzer-Library binding, qualified from an admitted
/// `LuaWorkspaceSnapshot`. Fields cannot be forged or deserialized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLibraryBinding {
    files: Vec<FileDescriptor>,
    library_snapshot_id: Box<str>,
    project_snapshot_id: Box<str>,
    project_generation: ProjectGenerationId,
    metadata_bytes: usize,
}

impl ProjectLibraryBinding {
    /// Qualifies the one exact Library snapshot already recorded by `project`.
    ///
    /// The selected profile determines the only accepted workspace universe.
    /// No source bodies are copied or retained.
    pub fn qualify(
        project: &ProjectView,
        snapshot: &LuaWorkspaceSnapshot,
        max_metadata_bytes: usize,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        if max_metadata_bytes == 0 || max_metadata_bytes > DISK_INVENTORY_MAX_BYTES {
            return Err(library_error(
                ProjectErrorCode::SourceBudgetExceeded,
                "Library metadata reservation is outside the native inventory limit",
            ));
        }

        let project_snapshot = project.snapshot();
        let configuration = project_snapshot.configuration();
        let expected_universe = match configuration.selected_profile().profile_kind() {
            ProfileKind::Fixture => LuaWorkspaceUniverse::Fixture,
            ProfileKind::Release => LuaWorkspaceUniverse::BlizzardUi,
        };
        if snapshot.backend() != configuration.analyzer_binding().backend()
            || snapshot.universe() != expected_universe
        {
            return Err(library_error(
                ProjectErrorCode::InvalidAnalyzerBinding,
                "Library snapshot backend or universe differs from the configured Project profile",
            ));
        }

        if !project_snapshot
            .analyzer_binding()
            .library_snapshot_ids()
            .eq(std::iter::once(snapshot.snapshot_id()))
        {
            return Err(library_error(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                "Library snapshot is not the exact Library admitted by this Project snapshot",
            ));
        }

        let source_files = snapshot.files();
        if source_files.is_empty() || source_files.len() > DISK_INVENTORY_MAX_FILES {
            return Err(library_error(
                ProjectErrorCode::SourceBudgetExceeded,
                "Library descriptor count exceeds the native source limit",
            ));
        }

        let mut metadata_bytes = source_files
            .len()
            .checked_mul(std::mem::size_of::<FileDescriptor>())
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .and_then(|bytes| bytes.checked_add(snapshot.snapshot_id().len()))
            .and_then(|bytes| bytes.checked_add(project_snapshot.snapshot_id().len()))
            .ok_or_else(|| {
                library_error(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "Library descriptor reservation exceeds the native source limit",
                )
            })?;
        let mut source_bytes = 0u64;
        for file in source_files {
            crate::analyzer::checkpoint(stop)?;
            let path_bytes = file.path().len();
            let byte_length = file.byte_len();
            if path_bytes == 0
                || path_bytes > LIBRARY_PATH_MAX_BYTES
                || file.content_sha256().len() != 71
                || !file.content_sha256().starts_with("sha256:")
                || file.byte_len() > DISK_SOURCE_MAX_BYTES as u64
                || u64::try_from(file.text().len()).ok() != Some(byte_length)
            {
                return Err(library_error(
                    ProjectErrorCode::AnalyzerSnapshotMismatch,
                    "Library snapshot contains invalid bounded file metadata",
                ));
            }
            metadata_bytes = metadata_bytes
                .checked_add(path_bytes)
                .and_then(|total| total.checked_add(file.content_sha256().len()))
                .ok_or_else(|| {
                    library_error(
                        ProjectErrorCode::SourceBudgetExceeded,
                        "Library descriptor metadata exceeds the native source limit",
                    )
                })?;
            source_bytes = source_bytes.checked_add(byte_length).ok_or_else(|| {
                library_error(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "Library source length exceeds the native source limit",
                )
            })?;
            if metadata_bytes > max_metadata_bytes || source_bytes > DISK_INVENTORY_MAX_BYTES as u64
            {
                return Err(library_error(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "Library descriptors exceed the native source limit",
                ));
            }
        }

        // Allocate and clone only after the complete scope and cancellation state
        // have been checked.
        crate::analyzer::checkpoint(stop)?;
        let mut files = Vec::with_capacity(source_files.len());
        for file in source_files {
            crate::analyzer::checkpoint(stop)?;
            files.push(FileDescriptor {
                path: file.path().into(),
                sha256: file.content_sha256().into(),
                byte_length: file.byte_len(),
            });
        }
        crate::analyzer::checkpoint(stop)?;

        Ok(Self {
            files,
            library_snapshot_id: snapshot.snapshot_id().into(),
            project_snapshot_id: project_snapshot.snapshot_id().into(),
            project_generation: project_snapshot.project_generation(),
            metadata_bytes,
        })
    }

    /// Exact admitted analyzer Library snapshot identity.
    #[must_use]
    pub fn library_snapshot_id(&self) -> &str {
        &self.library_snapshot_id
    }

    /// Exact immutable Project snapshot identity used during qualification.
    #[must_use]
    pub fn project_snapshot_id(&self) -> &str {
        &self.project_snapshot_id
    }

    /// Project generation used during qualification.
    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.project_generation
    }

    /// Retained descriptor and identity reservation; excludes source bodies.
    #[must_use]
    pub const fn metadata_bytes(&self) -> usize {
        self.metadata_bytes
    }

    /// Verifies exact caller-supplied file bytes and returns a borrowed view.
    pub fn file_view<'a>(
        &'a self,
        path: &str,
        text: &'a str,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectLibraryFileView<'a>> {
        crate::analyzer::checkpoint(stop)?;
        let descriptor = self
            .files
            .binary_search_by(|file| file.path.as_bytes().cmp(path.as_bytes()))
            .ok()
            .and_then(|index| self.files.get(index))
            .ok_or_else(|| {
                library_error(
                    ProjectErrorCode::FileNotPresent,
                    "file is not a member of the admitted Library snapshot",
                )
            })?;

        let byte_length = u64::try_from(text.len()).map_err(|_| {
            library_error(
                ProjectErrorCode::FileLengthMismatch,
                "supplied Library file length differs from its admitted descriptor",
            )
        })?;
        if byte_length != descriptor.byte_length || text.len() > DISK_SOURCE_MAX_BYTES {
            return Err(library_error(
                ProjectErrorCode::FileLengthMismatch,
                "supplied Library file length differs from its admitted descriptor",
            ));
        }
        crate::analyzer::checkpoint(stop)?;
        if crate::identity::source_digest(text.as_bytes()).to_string() != descriptor.sha256.as_ref()
        {
            return Err(library_error(
                ProjectErrorCode::FileDigestMismatch,
                "supplied Library file bytes differ from their admitted descriptor",
            ));
        }
        crate::analyzer::checkpoint(stop)?;

        Ok(ProjectLibraryFileView {
            binding: self,
            descriptor,
            text,
        })
    }
}

/// Immutable borrowed access to one full-file-verified Library member.
#[derive(Debug, Clone, Copy)]
pub struct ProjectLibraryFileView<'a> {
    binding: &'a ProjectLibraryBinding,
    descriptor: &'a FileDescriptor,
    text: &'a str,
}

impl<'a> ProjectLibraryFileView<'a> {
    /// Exact case-preserving Library path.
    #[must_use]
    pub fn path(self) -> &'a str {
        &self.descriptor.path
    }

    /// SHA-256 of the exact verified UTF-8 bytes.
    #[must_use]
    pub fn sha256(self) -> &'a str {
        &self.descriptor.sha256
    }

    /// Exact verified file byte length.
    #[must_use]
    pub const fn byte_length(self) -> u64 {
        self.descriptor.byte_length
    }

    /// Exact admitted analyzer Library snapshot identity.
    #[must_use]
    pub fn library_snapshot_id(self) -> &'a str {
        self.binding.library_snapshot_id()
    }

    /// Exact Project snapshot identity used during qualification.
    #[must_use]
    pub fn project_snapshot_id(self) -> &'a str {
        self.binding.project_snapshot_id()
    }

    /// Project generation used during qualification.
    #[must_use]
    pub const fn project_generation(self) -> ProjectGenerationId {
        self.binding.project_generation()
    }

    /// Binding that admitted this member.
    #[must_use]
    pub const fn binding(self) -> &'a ProjectLibraryBinding {
        self.binding
    }

    /// Returns one exact byte-offset slice when both offsets are UTF-8 boundaries.
    pub fn slice(self, start: usize, end: usize, stop: &AtomicBool) -> ProjectResult<&'a str> {
        crate::analyzer::checkpoint(stop)?;
        let slice = self.text.get(start..end).ok_or_else(|| {
            library_error(
                ProjectErrorCode::FileLengthMismatch,
                "Library slice is outside the file or splits a UTF-8 code point",
            )
        })?;
        crate::analyzer::checkpoint(stop)?;
        Ok(slice)
    }
}

fn library_error(code: ProjectErrorCode, message: &'static str) -> ProjectError {
    ProjectError::new(code, ProjectPhase::View, message)
}
