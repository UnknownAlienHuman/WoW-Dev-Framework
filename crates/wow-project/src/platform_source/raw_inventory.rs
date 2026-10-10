//! Borrowed access to the complete admitted Included byte set. This does not
//! decode members or reinterpret the source owner's coverage and provenance.
use std::{collections::btree_map, slice, sync::atomic::AtomicBool};

use wow_core::{ContentDigest, SourceContent};

use super::{
    AdmittedPlatformSource, PlatformEntryDisposition, PlatformFileKind, PlatformInventoryEntry,
    PlatformSourceAdmissionReceipt, budget, invalid,
};
use crate::{ProjectError, ProjectResult, disk};

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
            let PlatformEntryDisposition::Included {
                digest,
                byte_length,
                ..
            } = entry.disposition
            else {
                continue;
            };
            let (path, bytes) = self
                .files
                .next()
                .ok_or_else(|| invalid("raw inventory omits an Included member"))?;
            let actual_length = u64::try_from(bytes.len())
                .map_err(|_| budget("raw member length is not representable"))?;
            if path != &entry.path || actual_length != byte_length {
                return Err(invalid("raw inventory and native byte member disagree"));
            }
            if actual_length > limits.max_file_bytes {
                return Err(budget("raw inventory member exceeds its byte limit"));
            }
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
            return Ok(Some(PlatformRawMember {
                entry,
                digest,
                bytes,
            }));
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
