use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};

use crate::identity::valid_sha256;
use crate::{StoreError, StoreErrorCode, StoreResult};

pub(crate) const SNAPSHOT_SCHEMA: &str = "wow-store/snapshot/1";
pub(crate) const REF_RECORD_SCHEMA: &str = "wow-store/ref-record/1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct BlobId(Box<str>);

impl BlobId {
    pub fn parse(value: impl Into<Box<str>>) -> StoreResult<Self> {
        let value = value.into();
        if !valid_sha256(&value) {
            return Err(StoreError::new(
                StoreErrorCode::InvalidIdentity,
                "blob identity is not canonical SHA-256",
                Some(&value),
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn from_verified(value: Box<str>) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn hex(&self) -> &str {
        &self.0[7..]
    }
}

impl<'de> Deserialize<'de> for BlobId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Box::<str>::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SnapshotId(Box<str>);

impl SnapshotId {
    pub fn parse(value: impl Into<Box<str>>) -> StoreResult<Self> {
        let value = value.into();
        let digest = value.strip_prefix("store-snapshot:").ok_or_else(|| {
            StoreError::new(
                StoreErrorCode::InvalidIdentity,
                "snapshot identity has an unsupported prefix",
                Some(&value),
            )
        })?;
        if !valid_sha256(digest) {
            return Err(StoreError::new(
                StoreErrorCode::InvalidIdentity,
                "snapshot identity is not canonical SHA-256",
                Some(&value),
            ));
        }
        Ok(Self(value))
    }

    pub(crate) fn from_verified(value: Box<str>) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn hex(&self) -> &str {
        &self.0[22..]
    }
}

impl<'de> Deserialize<'de> for SnapshotId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Box::<str>::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RefName(Box<str>);

impl RefName {
    pub fn parse(value: impl Into<Box<str>>) -> StoreResult<Self> {
        let value = value.into();
        if value.is_empty()
            || value.len() > 128
            || value.starts_with('.')
            || value.ends_with('.')
            || !value.bytes().all(|byte| {
                byte == b'-' || byte == b'_' || byte == b'.' || byte.is_ascii_alphanumeric()
            })
        {
            return Err(StoreError::new(
                StoreErrorCode::InvalidIdentity,
                "ref name is not a bounded static identifier",
                Some(&value),
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RefName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Box::<str>::deserialize(deserializer)?;
        Self::parse(value).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StoreLimits {
    pub max_blob_bytes: u64,
    pub max_snapshot_entries: usize,
    pub max_snapshot_bytes: u64,
    pub max_ref_journal_bytes: u64,
}

impl StoreLimits {
    pub fn new(
        max_blob_bytes: u64,
        max_snapshot_entries: usize,
        max_snapshot_bytes: u64,
        max_ref_journal_bytes: u64,
    ) -> StoreResult<Self> {
        if max_blob_bytes == 0
            || max_snapshot_entries == 0
            || max_snapshot_bytes < max_blob_bytes
            || max_ref_journal_bytes < 1024
        {
            return Err(StoreError::new(
                StoreErrorCode::InputLimitExceeded,
                "store limits are zero or internally inconsistent",
                None,
            ));
        }
        Ok(Self {
            max_blob_bytes,
            max_snapshot_entries,
            max_snapshot_bytes,
            max_ref_journal_bytes,
        })
    }
}

impl Default for StoreLimits {
    fn default() -> Self {
        Self {
            max_blob_bytes: 64 * 1024 * 1024,
            max_snapshot_entries: 100_000,
            max_snapshot_bytes: 4 * 1024 * 1024 * 1024,
            max_ref_journal_bytes: 16 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotEntryInput {
    path: Box<str>,
    blob_id: BlobId,
}

impl SnapshotEntryInput {
    pub fn new(path: impl Into<Box<str>>, blob_id: BlobId) -> StoreResult<Self> {
        let path = path.into();
        validate_entry_path(&path)?;
        Ok(Self { path, blob_id })
    }

    pub(crate) fn into_parts(self) -> (Box<str>, BlobId) {
        (self.path, self.blob_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotEntry {
    path: Box<str>,
    blob_id: BlobId,
    bytes: u64,
}

impl SnapshotEntry {
    pub(crate) fn new(path: Box<str>, blob_id: BlobId, bytes: u64) -> Self {
        Self {
            path,
            blob_id,
            bytes,
        }
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub const fn blob_id(&self) -> &BlobId {
        &self.blob_id
    }

    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreSnapshot {
    schema: Box<str>,
    snapshot_id: SnapshotId,
    total_bytes: u64,
    entries: Vec<SnapshotEntry>,
}

impl StoreSnapshot {
    pub(crate) fn new(
        snapshot_id: SnapshotId,
        total_bytes: u64,
        entries: Vec<SnapshotEntry>,
    ) -> Self {
        Self {
            schema: SNAPSHOT_SCHEMA.into(),
            snapshot_id,
            total_bytes,
            entries,
        }
    }

    #[must_use]
    pub fn snapshot_id(&self) -> &SnapshotId {
        &self.snapshot_id
    }

    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total_bytes
    }

    #[must_use]
    pub fn entries(&self) -> &[SnapshotEntry] {
        &self.entries
    }

    pub(crate) fn schema(&self) -> &str {
        &self.schema
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RefRecord {
    pub schema: Box<str>,
    pub sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<SnapshotId>,
    pub next: SnapshotId,
    pub record_id: Box<str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RefRecordIdentity<'a> {
    pub schema: &'static str,
    pub sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous: Option<&'a SnapshotId>,
    pub next: &'a SnapshotId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefUpdate {
    Created,
    Advanced,
    NoChange,
}

pub(crate) fn validate_entry_path(path: &str) -> StoreResult<()> {
    if path.is_empty()
        || path.len() > 4096
        || path.starts_with('/')
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|segment| segment.is_empty() || matches!(segment, "." | ".."))
    {
        return Err(StoreError::new(
            StoreErrorCode::InvalidPath,
            "snapshot entry path is not a normalized repository-relative path",
            Some(path),
        ));
    }
    Ok(())
}
