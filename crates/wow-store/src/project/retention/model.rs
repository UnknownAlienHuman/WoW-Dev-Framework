//! Typed retention-root declarations over an owned manifested generation.
//!
//! A root is exact, attributable and listable. It names one generation and one
//! holder; it never carries a directory, path, wildcard or host location.
use serde::{Deserialize, Serialize};

use super::super::model::{EpochId, StoreGenerationId, digest, encode, failure, invalid, named};
use crate::{StoreError, StoreErrorCode, StoreResult};

/// Explicit policy holds. Current, leases and publication-in-progress roots
/// come from observable store state and are never declared here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetentionRootKind {
    Evidence,
    Debug,
    Rollback,
    User,
    Recovery,
    Quarantine,
    Backup,
    Export,
    Policy,
}

/// Caller-owned, finite idempotency key for one exact hold.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RetentionRootId(String);

impl RetentionRootId {
    pub fn new(value: impl Into<String>) -> StoreResult<Self> {
        Self::try_from(value.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for RetentionRootId {
    type Error = StoreError;
    fn try_from(value: String) -> StoreResult<Self> {
        if !named(&value) {
            return Err(failure(StoreErrorCode::IdentifierInvalid));
        }
        Ok(Self(value))
    }
}
impl From<RetentionRootId> for String {
    fn from(value: RetentionRootId) -> Self {
        value.0
    }
}

/// A persistent hold over one exact generation in one physical epoch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionRoot {
    schema: String,
    epoch_id: EpochId,
    root_id: RetentionRootId,
    kind: RetentionRootKind,
    generation_id: StoreGenerationId,
    held_by: String,
    pin_digest: String,
}

impl RetentionRoot {
    pub fn new(
        epoch_id: EpochId,
        root_id: RetentionRootId,
        kind: RetentionRootKind,
        generation_id: StoreGenerationId,
        held_by: &str,
    ) -> StoreResult<Self> {
        const SCHEMA: &str = "wow-store/retention-root/1";
        if !named(held_by) {
            return Err(failure(StoreErrorCode::IdentifierInvalid));
        }
        let pin_digest = digest(
            "project-retention-root",
            &encode(
                &(SCHEMA, &epoch_id, &root_id, kind, &generation_id, held_by),
                65536,
            )?,
        );
        Ok(Self {
            schema: SCHEMA.into(),
            epoch_id,
            root_id,
            kind,
            generation_id,
            held_by: held_by.into(),
            pin_digest,
        })
    }
    pub fn epoch_id(&self) -> &EpochId {
        &self.epoch_id
    }
    pub fn root_id(&self) -> &RetentionRootId {
        &self.root_id
    }
    pub fn kind(&self) -> RetentionRootKind {
        self.kind
    }
    pub fn generation_id(&self) -> &StoreGenerationId {
        &self.generation_id
    }
    pub fn held_by(&self) -> &str {
        &self.held_by
    }
    pub fn pin_digest(&self) -> &str {
        &self.pin_digest
    }

    pub(super) fn validate(&self) -> StoreResult<()> {
        let expected = Self::new(
            self.epoch_id.clone(),
            self.root_id.clone(),
            self.kind,
            self.generation_id.clone(),
            &self.held_by,
        )?;
        if self != &expected {
            return Err(invalid());
        }
        Ok(())
    }
}
impl RetentionRootKind {
    /// Stable lowercase label used in the canonical digest projection.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Debug => "debug",
            Self::Rollback => "rollback",
            Self::User => "user",
            Self::Recovery => "recovery",
            Self::Quarantine => "quarantine",
            Self::Backup => "backup",
            Self::Export => "export",
            Self::Policy => "policy",
        }
    }
}
