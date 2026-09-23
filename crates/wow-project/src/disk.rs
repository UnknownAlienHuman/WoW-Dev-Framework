//! Read-only acquisition of explicitly listed local inputs. No directory scan.
use std::collections::BTreeSet;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde::Deserialize;
use wow_core::{ContentDigest, NormalizedSourcePath, SourceContent};

use crate::{
    ProjectError, ProjectErrorCode, ProjectFileRole, ProjectInputFile, ProjectLanguageKind,
    ProjectPhase, ProjectResult,
};

pub const DISK_CONFIGURATION_MAX_BYTES: usize = 32 * 1024 * 1024;
pub const DISK_ARTIFACT_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const DISK_SOURCE_MAX_BYTES: usize = 1024 * 1024;
pub const DISK_INVENTORY_MAX_BYTES: usize = 16 * 1024 * 1024;
pub const DISK_INVENTORY_MAX_FILES: usize = 1024;

/// A logical file path, optionally bound to predeclared exact content.
/// Construction/deserialization is followed by admission before any file open.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectDiskFile {
    path: String,
    #[serde(default)]
    content_digest: Option<ContentDigest<SourceContent>>,
    #[serde(default)]
    byte_length: Option<u64>,
}

impl ProjectDiskFile {
    /// Select a file whose exact identity will be computed during acquisition.
    #[must_use]
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content_digest: None,
            byte_length: None,
        }
    }

    /// Require the read bytes to match a predeclared source-artifact identity.
    #[must_use]
    pub fn with_identity(mut self, digest: ContentDigest<SourceContent>, length: u64) -> Self {
        self.content_digest = Some(digest);
        self.byte_length = Some(length);
        self
    }

    fn validate(&self) -> ProjectResult<()> {
        validate_path(&self.path)?;
        if self.content_digest.is_some() != self.byte_length.is_some() {
            return Err(failure(
                ProjectErrorCode::InvalidInputInventory,
                "declared content digest and length must be supplied together",
            ));
        }
        Ok(())
    }

    fn verify(&self, bytes: &[u8]) -> ProjectResult<()> {
        if self
            .byte_length
            .is_some_and(|length| length != bytes.len() as u64)
        {
            return Err(failure(
                ProjectErrorCode::FileLengthMismatch,
                "local file differs from its declared byte length",
            ));
        }
        if self
            .content_digest
            .is_some_and(|digest| digest != crate::identity::source_digest(bytes))
        {
            return Err(failure(
                ProjectErrorCode::FileDigestMismatch,
                "local file differs from its declared digest",
            ));
        }
        Ok(())
    }
}

/// A registered local input directory, retained by handle, not by its host name.
/// Only explicit JSON artifacts and Lua inventories can be read through this port.
/// No host path, raw handle, write, enumeration or process capability is exposed.
pub struct ProjectInputDirectory {
    directory: Dir,
}

impl ProjectInputDirectory {
    /// Register the directory explicitly selected by the caller. This is the only
    /// ambient filesystem operation. Descendant lookup never follows symlinks.
    pub fn open(root: &Path) -> ProjectResult<Self> {
        if root.as_os_str().is_empty() {
            return Err(failure(
                ProjectErrorCode::InvalidFilePath,
                "input directory is empty",
            ));
        }
        let directory =
            Dir::open_ambient_dir(root, cap_std::ambient_authority()).map_err(|_| {
                failure(
                    ProjectErrorCode::SourceReadFailed,
                    "input directory cannot be opened",
                )
            })?;
        Ok(Self { directory })
    }

    /// Read the explicitly selected configuration, never an inferred filename.
    pub fn read_configuration(&self, name: &str, stop: &AtomicBool) -> ProjectResult<Vec<u8>> {
        if name.contains('/') {
            return Err(failure(
                ProjectErrorCode::InvalidFilePath,
                "configuration must name one file",
            ));
        }
        let selected = ProjectDiskFile::new(name);
        self.read(&selected, DISK_CONFIGURATION_MAX_BYTES, stop)
    }

    /// Read a bounded, digest-pinned profile, ReferenceView or analyzer report.
    /// The respective semantic owner still validates its decoded contents.
    pub fn read_json_artifact(
        &self,
        selected: &ProjectDiskFile,
        stop: &AtomicBool,
    ) -> ProjectResult<Vec<u8>> {
        selected.validate()?;
        if !selected.path.ends_with(".json") || selected.content_digest.is_none() {
            return Err(failure(
                ProjectErrorCode::InvalidInputInventory,
                "JSON artifacts require a path, exact digest and byte length",
            ));
        }
        self.read(selected, DISK_ARTIFACT_MAX_BYTES, stop)
    }

    /// Copy one explicitly listed Main or Library inventory into immutable owner
    /// inputs. Paths and case collisions are admitted before reading any source.
    /// Source digests are computed from retained UTF-8 bytes, never file metadata.
    pub fn read_lua_inventory(
        &self,
        root: &str,
        files: &[ProjectDiskFile],
        role: ProjectFileRole,
        stop: &AtomicBool,
    ) -> ProjectResult<Vec<ProjectInputFile>> {
        checkpoint(stop)?;
        if files.is_empty() || files.len() > DISK_INVENTORY_MAX_FILES {
            return Err(failure(
                ProjectErrorCode::InvalidInputInventory,
                "invalid source inventory size",
            ));
        }
        let mut seen = BTreeSet::new();
        for file in files {
            file.validate()?;
            if !file.path.ends_with(".lua") {
                return Err(failure(
                    ProjectErrorCode::InvalidFileLanguage,
                    "source inventory accepts Lua files only",
                ));
            }
            if !seen.insert(file.path.to_lowercase()) {
                return Err(failure(
                    ProjectErrorCode::FileCaseCollision,
                    "source paths collide ignoring case",
                ));
            }
        }
        let directory = Self {
            directory: descend(&self.directory, root)?,
        };
        let mut total = 0usize;
        let mut output = Vec::with_capacity(files.len());
        for file in files {
            checkpoint(stop)?;
            let remaining = DISK_INVENTORY_MAX_BYTES - total;
            let bytes = directory.read(file, remaining.min(DISK_SOURCE_MAX_BYTES), stop)?;
            total += bytes.len();
            let text = String::from_utf8(bytes).map_err(|_| {
                failure(
                    ProjectErrorCode::InvalidEncoding,
                    "source file must contain UTF-8",
                )
            })?;
            output.push(
                ProjectInputFile::declared(
                    file.path.clone(),
                    text,
                    ProjectLanguageKind::Lua,
                    role,
                    None::<String>,
                )
                .map_err(|_| {
                    failure(
                        ProjectErrorCode::InvalidInputInventory,
                        "source content was rejected",
                    )
                })?,
            );
        }
        checkpoint(stop)?;
        Ok(output)
    }

    fn read(
        &self,
        selected: &ProjectDiskFile,
        limit: usize,
        stop: &AtomicBool,
    ) -> ProjectResult<Vec<u8>> {
        checkpoint(stop)?;
        selected.validate()?;
        if selected
            .byte_length
            .is_some_and(|length| length > limit as u64)
        {
            return Err(failure(
                ProjectErrorCode::SourceBudgetExceeded,
                "declared file exceeds the read budget",
            ));
        }
        let (parent, filename) = selected
            .path
            .rsplit_once('/')
            .unwrap_or((".", &selected.path));
        let directory = descend(&self.directory, parent)?;
        // Reject stable special files before open; the actual handle is checked below.
        let entry = directory.symlink_metadata(filename).map_err(|_| {
            failure(
                ProjectErrorCode::MissingDeclaredFile,
                "declared input file is unavailable",
            )
        })?;
        if !entry.is_file() || entry.file_type().is_symlink() {
            return Err(failure(
                ProjectErrorCode::SourceReadFailed,
                "input must be a regular file, not a symlink",
            ));
        }
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        // A racing replacement with a FIFO must not block open on Unix.
        #[cfg(unix)]
        {
            use cap_fs_ext::OpenOptionsSyncExt;
            options.nonblock(true);
        }
        let mut file = directory.open_with(filename, &options).map_err(|_| {
            failure(
                ProjectErrorCode::SourceReadFailed,
                "declared input file cannot be opened",
            )
        })?;
        let before = file.metadata().map_err(|_| read_error())?;
        if !before.is_file() {
            return Err(failure(
                ProjectErrorCode::SourceReadFailed,
                "opened input is not a regular file",
            ));
        }
        if before.len() > limit as u64 {
            return Err(failure(
                ProjectErrorCode::SourceBudgetExceeded,
                "input file exceeds the read budget",
            ));
        }
        let modified = before.modified().map_err(|_| read_error())?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(before.len() as usize)
            .map_err(|_| {
                failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "input buffer cannot be allocated",
                )
            })?;
        let mut chunk = [0u8; 16 * 1024];
        loop {
            checkpoint(stop)?;
            let count = chunk.len().min(limit + 1 - bytes.len());
            let count = match file.read(&mut chunk[..count]) {
                Ok(count) => count,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(_) => return Err(read_error()),
            };
            if count == 0 {
                break;
            }
            if bytes.len() + count > limit {
                return Err(failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "input file exceeds the read budget",
                ));
            }
            bytes.try_reserve_exact(count).map_err(|_| {
                failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "input buffer cannot be allocated",
                )
            })?;
            bytes.extend_from_slice(&chunk[..count]);
        }
        let after = file.metadata().map_err(|_| read_error())?;
        if before.len() != bytes.len() as u64
            || before.len() != after.len()
            || modified != after.modified().map_err(|_| read_error())?
        {
            return Err(failure(
                ProjectErrorCode::SourceChangedDuringRead,
                "input file changed during acquisition",
            ));
        }
        selected.verify(&bytes)?;
        checkpoint(stop)?;
        Ok(bytes)
    }
}

fn descend(root: &Dir, path: &str) -> ProjectResult<Dir> {
    if path != "." {
        validate_path(path)?;
    }
    let mut directory = root.try_clone().map_err(|_| read_error())?;
    if path != "." {
        // Each open retains its parent handle. No check-then-open absolute path.
        for component in path.split('/') {
            directory = directory.open_dir_nofollow(component).map_err(|_| {
                failure(
                    ProjectErrorCode::SourceReadFailed,
                    "input subdirectory is unavailable or linked",
                )
            })?;
        }
    }
    Ok(directory)
}

fn validate_path(path: &str) -> ProjectResult<()> {
    let admitted = path.len() <= 4096
        && path.parse::<NormalizedSourcePath>().is_ok()
        && path.split('/').all(|part| {
            !part.ends_with(['.', ' '])
                && !part
                    .chars()
                    .any(|c| matches!(c, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
                && !is_device_component(part)
        });
    if !admitted {
        return Err(failure(
            ProjectErrorCode::InvalidFilePath,
            "input path is not a portable canonical relative path",
        ));
    }
    Ok(())
}

fn is_device_component(part: &str) -> bool {
    let stem = part
        .split('.')
        .next()
        .unwrap_or("")
        .trim_end_matches(' ')
        .to_ascii_uppercase();
    if matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
    ) {
        return true;
    }
    stem.strip_prefix("COM")
        .or_else(|| stem.strip_prefix("LPT"))
        .is_some_and(|suffix| {
            matches!(
                suffix,
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            )
        })
}

fn checkpoint(stop: &AtomicBool) -> ProjectResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(
            ProjectErrorCode::SourceReadCancelled,
            "input acquisition cancelled",
        ))
    } else {
        Ok(())
    }
}
fn read_error() -> ProjectError {
    failure(
        ProjectErrorCode::SourceReadFailed,
        "input acquisition failed",
    )
}
fn failure(code: ProjectErrorCode, message: &'static str) -> ProjectError {
    ProjectError::new(code, ProjectPhase::Inventory, message)
}
