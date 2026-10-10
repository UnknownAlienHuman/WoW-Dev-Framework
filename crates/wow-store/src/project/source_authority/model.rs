//! Bounded data-only references to retained original-source authority.
use crate::project::{
    EpochManifest, RegistrySelection,
    model::{digest, encode, failure, hashed, invalid},
    quarantine::archives::{self, QuarantineReference},
    registry::MAX_REGISTRY,
};
use crate::{StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};

pub(super) const MAX_MANIFEST: usize = 128 * 1024;
pub(super) const MAX_AUTHORITIES: usize = 32;
pub(super) const MAX_BYTES: usize = 64 * 1024 * 1024;

/// Manifest identity and exact byte length; decoded data grants no owner authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAuthorityReference {
    pub(super) manifest_digest: String,
    pub(super) manifest_length: usize,
}
impl SourceAuthorityReference {
    pub fn manifest_digest(&self) -> &str {
        &self.manifest_digest
    }
    pub fn manifest_length(&self) -> usize {
        self.manifest_length
    }
    pub(in crate::project) fn validate(&self) -> StoreResult<()> {
        if !hashed(&self.manifest_digest, "project-source-authority")
            || !(1..=MAX_MANIFEST).contains(&self.manifest_length)
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub(in crate::project) fn from_bytes(bytes: &[u8]) -> StoreResult<Self> {
        if bytes.len() > MAX_MANIFEST {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        let reference = Self {
            manifest_digest: digest("project-source-authority", bytes),
            manifest_length: bytes.len(),
        };
        reference.validate()?;
        Ok(reference)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceAuthorityManifest {
    pub(super) schema: String,
    pub(super) epoch: EpochManifest,
    pub(super) selection: RegistrySelection,
    pub(super) selection_length: usize,
    pub(super) source_snapshot: String,
    pub(super) retained_quarantines: Vec<QuarantineReference>,
    pub(super) dependencies: Vec<SourceAuthorityReference>,
}
impl SourceAuthorityManifest {
    pub(super) fn bytes(&self) -> StoreResult<Vec<u8>> {
        if self.schema != "wow-store/project-source-authority/1"
            || !(1..=MAX_REGISTRY).contains(&self.selection_length)
            || !hashed(&self.source_snapshot, "project-backup-snapshot")
        {
            return Err(invalid());
        }
        self.selection.validate()?;
        archives::validate_references(&self.retained_quarantines)?;
        validate_references(&self.dependencies)?;
        encode(self, MAX_MANIFEST)
    }
}

/// Canonical inventories contain unique digests, ordered strictly by digest.
pub(in crate::project) fn validate_references(
    references: &[SourceAuthorityReference],
) -> StoreResult<()> {
    if references.len() > MAX_AUTHORITIES {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    for reference in references {
        reference.validate()?;
    }
    if references
        .windows(2)
        .any(|pair| pair[0].manifest_digest >= pair[1].manifest_digest)
    {
        return Err(invalid());
    }
    Ok(())
}
