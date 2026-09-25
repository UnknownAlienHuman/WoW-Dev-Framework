//! Role-neutral acquisition of explicitly pinned Lua data. These bytes are not
//! Main or Library files until a semantic owner explicitly admits that role.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use wow_core::{ContentDigest, SourceContent};

use super::{
    DISK_INVENTORY_MAX_BYTES, DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES, ProjectDiskFile,
    ProjectInputDirectory, checkpoint, failure,
};
use crate::{ProjectErrorCode, ProjectResult};

/// Exact captured Lua bytes for a non-executing data consumer.
/// No filesystem or project/analyzer handle crosses this boundary.
pub struct PinnedLuaSource {
    path: String,
    text: String,
    digest: ContentDigest<SourceContent>,
}

impl PinnedLuaSource {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.digest
    }
}

impl ProjectInputDirectory {
    /// Read only the listed, digest/length-pinned Lua files through the existing
    /// confined no-follow reader. This does not register a Main/Library universe.
    pub fn read_pinned_lua_sources(
        &self,
        root: &str,
        files: &[ProjectDiskFile],
        stop: &AtomicBool,
    ) -> ProjectResult<Vec<PinnedLuaSource>> {
        checkpoint(stop)?;
        if files.is_empty() || files.len() > DISK_INVENTORY_MAX_FILES {
            return Err(failure(
                ProjectErrorCode::InvalidInputInventory,
                "invalid pinned Lua inventory size",
            ));
        }
        let mut seen = BTreeSet::new();
        let mut declared_total = 0u64;
        for file in files {
            checkpoint(stop)?;
            file.validate()?;
            let Some(length) = file.byte_length else {
                return Err(failure(
                    ProjectErrorCode::InvalidInputInventory,
                    "native source requires exact digest and byte length",
                ));
            };
            if !file.path.ends_with(".lua") || file.content_digest.is_none() {
                return Err(failure(
                    ProjectErrorCode::InvalidInputInventory,
                    "native source requires pinned Lua files",
                ));
            }
            if !seen.insert(file.path.to_lowercase()) {
                return Err(failure(
                    ProjectErrorCode::FileCaseCollision,
                    "native source paths collide ignoring case",
                ));
            }
            declared_total = declared_total.checked_add(length).ok_or_else(|| {
                failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "native source byte count overflow",
                )
            })?;
            if length > DISK_SOURCE_MAX_BYTES as u64
                || declared_total > DISK_INVENTORY_MAX_BYTES as u64
            {
                return Err(failure(
                    ProjectErrorCode::SourceBudgetExceeded,
                    "native source exceeds acquisition budget",
                ));
            }
        }
        let directory = self.subdirectory(root)?;
        let mut captured = Vec::with_capacity(files.len());
        let mut total = 0usize;
        for file in files {
            checkpoint(stop)?;
            let bytes = directory.read(
                file,
                DISK_SOURCE_MAX_BYTES.min(DISK_INVENTORY_MAX_BYTES - total),
                stop,
            )?;
            total += bytes.len();
            let digest = crate::identity::source_digest(&bytes);
            let text = String::from_utf8(bytes).map_err(|_| {
                failure(
                    ProjectErrorCode::InvalidEncoding,
                    "native source must contain UTF-8",
                )
            })?;
            captured.push(PinnedLuaSource {
                path: file.path.clone(),
                text,
                digest,
            });
        }
        captured.sort_by(|a, b| a.path.cmp(&b.path));
        checkpoint(stop)?;
        Ok(captured)
    }
}
