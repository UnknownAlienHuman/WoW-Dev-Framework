use super::{ProjectDiskFile, ProjectInputDirectory, checkpoint, failure, read_error, validate_path};
use crate::{ProjectErrorCode, ProjectFileRole, ProjectInputFile, ProjectResult};
use cap_fs_ext::DirExt;
use cap_std::fs::Dir;
use std::sync::atomic::AtomicBool;

const MAX_ENTRIES: usize = 16_384;
const MAX_DEPTH: usize = 32;

impl ProjectInputDirectory {
    /// Read all `.lua` files below the selected Library root, sorted by relative path.
    /// Ordinary non-Lua files (including generator reports) are ignored. Links and
    /// special files reject; acquisition never executes source or reads a report.
    pub fn read_lua_library(
        &self,
        root: &str,
        stop: &AtomicBool,
    ) -> ProjectResult<Vec<ProjectInputFile>> {
        checkpoint(stop)?;
        let directory = self.subdirectory(root)?;
        let mut files = Vec::new();
        let mut entries = 0;
        collect(&directory.directory, "", 0, &mut entries, &mut files, stop)?;
        files.sort_by(|left, right| left.path().cmp(right.path()));
        // Reuse case-collision, byte-budget, UTF-8 and no-follow file admission.
        directory.read_lua_inventory(".", &files, ProjectFileRole::Library, stop)
    }
}

fn collect(
    directory: &Dir,
    prefix: &str,
    depth: usize,
    entries: &mut usize,
    files: &mut Vec<ProjectDiskFile>,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    checkpoint(stop)?;
    for entry in directory.entries().map_err(|_| read_error())? {
        checkpoint(stop)?;
        if *entries == MAX_ENTRIES {
            return Err(failure(
                ProjectErrorCode::SourceBudgetExceeded,
                "Library directory exceeds the entry budget",
            ));
        }
        *entries += 1;
        let entry = entry.map_err(|_| read_error())?;
        let kind = entry.file_type().map_err(|_| read_error())?;
        if kind.is_symlink() || (!kind.is_dir() && !kind.is_file()) {
            return Err(failure(
                ProjectErrorCode::SourceReadFailed,
                "Library entries must be ordinary files or directories, not links",
            ));
        }
        let name = entry.file_name();
        if kind.is_file()
            && std::path::Path::new(&name)
                .extension()
                .is_none_or(|ext| ext != "lua")
        {
            continue;
        }
        let name = name.to_str().ok_or_else(|| {
            failure(
                ProjectErrorCode::InvalidFilePath,
                "Library path must be UTF-8",
            )
        })?;
        let path = format!("{prefix}{name}");
        validate_path(&path)?;
        if kind.is_dir() {
            if depth == MAX_DEPTH {
                return Err(failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "Library directory exceeds the nesting budget",
                ));
            }
            let child = directory.open_dir_nofollow(name).map_err(|_| read_error())?;
            collect(&child, &format!("{path}/"), depth + 1, entries, files, stop)?;
        } else {
            if files.len() == wow_core::library_limits::MAX_FILES {
                return Err(failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "Library directory exceeds the Lua file budget",
                ));
            }
            files.push(ProjectDiskFile::new(path));
        }
    }
    checkpoint(stop)
}
