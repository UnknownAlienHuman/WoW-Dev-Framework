//! Exact release receipts bound to an original publication operation.
use serde::{Deserialize, Serialize};

use super::super::model::{EpochId, PublicationOperation, digest, encode, failure, invalid, named};
use crate::{StoreErrorCode, StoreResult};

/// A canonical receipt identifying the original operation and its release holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationRelease {
    schema: String,
    epoch_id: EpochId,
    original_operation_digest: String,
    held_by: String,
    release_digest: String,
}

impl PublicationRelease {
    pub(super) fn new(
        epoch: &EpochId,
        original: &PublicationOperation,
        held_by: &str,
    ) -> StoreResult<Self> {
        const SCHEMA: &str = "wow-store/publication-release/1";
        if original.release.is_some() {
            return Err(invalid());
        }
        if !named(held_by) {
            return Err(failure(StoreErrorCode::IdentifierInvalid));
        }
        let original_operation_digest =
            digest("project-original-operation", &encode(original, 65536)?);
        let release_digest = digest(
            "project-publication-release",
            &encode(&(SCHEMA, epoch, &original_operation_digest, held_by), 65536)?,
        );
        Ok(Self {
            schema: SCHEMA.into(),
            epoch_id: epoch.clone(),
            original_operation_digest,
            held_by: held_by.into(),
            release_digest,
        })
    }

    pub fn schema(&self) -> &str {
        &self.schema
    }
    pub fn epoch_id(&self) -> &EpochId {
        &self.epoch_id
    }
    pub fn original_operation_digest(&self) -> &str {
        &self.original_operation_digest
    }
    pub fn held_by(&self) -> &str {
        &self.held_by
    }
    pub fn release_digest(&self) -> &str {
        &self.release_digest
    }

    pub(super) fn validate(
        &self,
        epoch: &EpochId,
        original: &PublicationOperation,
    ) -> StoreResult<()> {
        let expected = Self::new(epoch, original, &self.held_by)?;
        if self != &expected {
            return Err(invalid());
        }
        Ok(())
    }
}
