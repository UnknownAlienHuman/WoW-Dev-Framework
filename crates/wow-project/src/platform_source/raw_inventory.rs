//! Borrowed access to the complete admitted Included byte set. This does not
//! decode members or reinterpret the source owner's coverage and provenance.
use std::{collections::btree_map, slice, sync::atomic::AtomicBool};

use wow_core::{ContentDigest, SourceContent};

use super::{
    AdmittedPlatformSource, PlatformEntryDisposition, PlatformFileKind, PlatformInventoryEntry,
    PlatformSourceAdmissionReceipt, budget, invalid,
};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult, disk};

enum CursorState {
    Active,
    Exhausted,
    Failed(ProjectError),
}

/// A bounded fallible cursor over one immutable native source owner. Only a
/// successfully exhausted cursor establishes complete Included-member traversal.
pub struct PlatformRawInventory<'a> {
    source: &'a AdmittedPlatformSource,
    entries: slice::Iter<'a, PlatformInventoryEntry>,
    files: btree_map::Iter<'a, String, Vec<u8>>,
    scanned: usize,
    yielded: usize,
    bytes: u64,
    state: CursorState,
}

/// Exact admitted bytes and metadata, borrowed without decoding or copying.
pub struct PlatformRawMember<'a> {
    entry: &'a PlatformInventoryEntry,
    digest: ContentDigest<SourceContent>,
    bytes: &'a [u8],
}

impl AdmittedPlatformSource {
    /// Borrow one exact original Included member from this admitted source.
    /// Explicit inventory omissions never become a missing-file result.
    pub fn raw_member(
        &self,
        path: &str,
        stop: &AtomicBool,
    ) -> ProjectResult<PlatformRawMember<'_>> {
        disk::checkpoint(stop)?;
        super::path(path)?;
        let limits = self.profile.limits();
        if self.receipt.inventory.entries.len() > limits.max_entries
            || self.files.len() > limits.max_entries
            || self.receipt.coverage.verified_bytes() > limits.max_total_bytes
        {
            return Err(budget("raw source exceeds its admitted owner limits"));
        }
        let entry = self
            .receipt
            .inventory
            .entries
            .binary_search_by(|entry| entry.path.as_str().cmp(path))
            .ok()
            .and_then(|index| self.receipt.inventory.entries.get(index))
            .ok_or_else(|| {
                member_failure(
                    ProjectErrorCode::FileNotPresent,
                    "raw platform path has no declared inventory entry",
                    path,
                )
            })?;
        disk::checkpoint(stop)?;
        let (member_path, bytes) = match &entry.disposition {
            PlatformEntryDisposition::Included { .. } => self
                .files
                .get_key_value(path)
                .ok_or_else(|| invalid("raw inventory omits an Included member"))?,
            PlatformEntryDisposition::Excluded { .. } => {
                return Err(member_failure(
                    ProjectErrorCode::PackageTargetExcluded,
                    "raw platform member is explicitly excluded",
                    path,
                ));
            }
            PlatformEntryDisposition::Unsupported { .. } => {
                return Err(member_failure(
                    ProjectErrorCode::InvalidFileLanguage,
                    "raw platform member is an unsupported special entry",
                    path,
                ));
            }
            PlatformEntryDisposition::External { .. } => {
                return Err(member_failure(
                    ProjectErrorCode::PackageTargetUnresolved,
                    "raw platform member is not materialized external content",
                    path,
                ));
            }
            PlatformEntryDisposition::Conflict { .. } => {
                return Err(member_failure(
                    ProjectErrorCode::PackageTargetUnresolved,
                    "raw platform member has conflicting inventory evidence",
                    path,
                ));
            }
            PlatformEntryDisposition::Failed { .. } => {
                return Err(member_failure(
                    ProjectErrorCode::PackageTargetUnresolved,
                    "raw platform member has failed materialization evidence",
                    path,
                ));
            }
        };
        let member =
            PlatformRawMember::from_included(entry, member_path, bytes, limits.max_file_bytes)?;
        disk::checkpoint(stop)?;
        Ok(member)
    }

    pub fn raw_inventory(&self, stop: &AtomicBool) -> ProjectResult<PlatformRawInventory<'_>> {
        disk::checkpoint(stop)?;
        // Admission has already validated identities, content, canonical order
        // and finite limits. Decoded receipts cannot construct this source owner.
        Ok(PlatformRawInventory {
            source: self,
            entries: self.receipt.inventory.entries.iter(),
            files: self.files.iter(),
            scanned: 0,
            yielded: 0,
            bytes: 0,
            state: CursorState::Active,
        })
    }
}

impl<'a> PlatformRawInventory<'a> {
    /// Retains every disposition, omission and original coverage assertion.
    #[must_use]
    pub fn receipt(&self) -> &'a PlatformSourceAdmissionReceipt {
        &self.source.receipt
    }

    /// Errors are terminal: clearing cancellation cannot skip an advanced entry.
    /// Starting another cursor requires the same genuine immutable source owner.
    pub fn next(&mut self, stop: &AtomicBool) -> ProjectResult<Option<PlatformRawMember<'a>>> {
        if let CursorState::Failed(error) = &self.state {
            return Err(error.clone());
        }
        let result = self.advance(stop);
        if let Err(error) = &result {
            self.state = CursorState::Failed(error.clone());
        }
        result
    }

    fn advance(&mut self, stop: &AtomicBool) -> ProjectResult<Option<PlatformRawMember<'a>>> {
        disk::checkpoint(stop)?;
        if matches!(self.state, CursorState::Exhausted) {
            return Ok(None);
        }
        let limits = self.source.profile.limits();
        for entry in self.entries.by_ref() {
            disk::checkpoint(stop)?;
            self.scanned = self
                .scanned
                .checked_add(1)
                .filter(|count| *count <= limits.max_entries)
                .ok_or_else(|| budget("raw inventory traversal exceeds its entry limit"))?;
            let PlatformEntryDisposition::Included { .. } = entry.disposition else {
                continue;
            };
            let (path, bytes) = self
                .files
                .next()
                .ok_or_else(|| invalid("raw inventory omits an Included member"))?;
            let member =
                PlatformRawMember::from_included(entry, path, bytes, limits.max_file_bytes)?;
            let actual_length = member.byte_length();
            self.yielded = self
                .yielded
                .checked_add(1)
                .filter(|count| *count <= limits.max_entries)
                .ok_or_else(|| budget("raw inventory traversal exceeds its member limit"))?;
            self.bytes = self
                .bytes
                .checked_add(actual_length)
                .filter(|count| *count <= limits.max_total_bytes)
                .ok_or_else(|| budget("raw inventory traversal exceeds its byte limit"))?;
            disk::checkpoint(stop)?;
            return Ok(Some(member));
        }
        if self.files.next().is_some()
            || self.scanned != self.source.receipt.inventory.entries.len()
            || self.yielded != self.source.receipt.coverage.verified_files()
            || self.bytes != self.source.receipt.coverage.verified_bytes()
        {
            return Err(invalid(
                "raw inventory traversal did not close the Included set",
            ));
        }
        disk::checkpoint(stop)?;
        self.state = CursorState::Exhausted;
        Ok(None)
    }
}

impl<'a> PlatformRawMember<'a> {
    fn from_included(
        entry: &'a PlatformInventoryEntry,
        path: &str,
        bytes: &'a [u8],
        max_file_bytes: u64,
    ) -> ProjectResult<Self> {
        let PlatformEntryDisposition::Included {
            digest,
            byte_length,
            ..
        } = entry.disposition
        else {
            return Err(invalid("raw member is not an Included inventory entry"));
        };
        let actual_length = u64::try_from(bytes.len())
            .map_err(|_| budget("raw member length is not representable"))?;
        if path != entry.path.as_str() || actual_length != byte_length {
            return Err(invalid("raw inventory and native byte member disagree"));
        }
        if actual_length > max_file_bytes {
            return Err(budget("raw inventory member exceeds its byte limit"));
        }
        Ok(Self {
            entry,
            digest,
            bytes,
        })
    }

    /// Original inventory metadata, including its Included object ID.
    #[must_use]
    pub const fn entry(&self) -> &'a PlatformInventoryEntry {
        self.entry
    }

    #[must_use]
    pub fn path(&self) -> &'a str {
        &self.entry.path
    }

    /// Declared kind, without a decoding or semantic-health claim.
    #[must_use]
    pub fn kind(&self) -> PlatformFileKind {
        self.entry.kind
    }

    #[must_use]
    pub fn content_digest(&self) -> ContentDigest<SourceContent> {
        self.digest
    }

    #[must_use]
    pub fn byte_length(&self) -> u64 {
        self.bytes.len() as u64
    }

    #[must_use]
    pub const fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}

fn member_failure(code: ProjectErrorCode, message: &'static str, path: &str) -> ProjectError {
    ProjectError::new(code, ProjectPhase::Inventory, message).with_relative_path(path)
}
