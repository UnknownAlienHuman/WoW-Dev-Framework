use crate::{OperationId, StoreError, StoreErrorCode, StoreResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

pub const PHYSICAL_PROFILE: &str = "project-store-wal-manifested-partitions-v1";
pub const RECORD_PROFILE: &str = "wow-store/retained-partition-records/1";
pub const MAX_RECORD_BYTES: usize = 32 * 1024 * 1024;
pub const MAX_GENERATION_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_PARTITIONS: usize = 256;
pub const MAX_GENERATIONS: i64 = 1024;
pub const MAX_VERSIONS: i64 = 8192;
pub const MAX_READERS: usize = 16;

pub(super) fn failure(code: StoreErrorCode) -> StoreError {
    StoreError::new(code, "manifested project storage operation failed")
}
pub(super) fn invalid() -> StoreError {
    failure(StoreErrorCode::IntegrityViolation)
}
pub(super) fn checkpoint(stop: &AtomicBool) -> StoreResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(StoreErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
pub(super) fn encode(value: &impl Serialize, max: usize) -> StoreResult<Vec<u8>> {
    // Limit serialization before canonicalization allocates its output.
    struct Counter {
        n: usize,
        max: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.n = self
                .n
                .checked_add(bytes.len())
                .filter(|n| *n <= self.max)
                .ok_or_else(|| std::io::Error::other("record byte limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { n: 0, max }, value)
        .map_err(|_| failure(StoreErrorCode::BudgetExceeded))?;
    let bytes = wow_core::canonical_json_bytes(value).map_err(|_| invalid())?;
    if bytes.len() > max {
        return Err(failure(StoreErrorCode::BudgetExceeded));
    }
    Ok(bytes)
}
pub(super) fn digest(prefix: &str, bytes: &[u8]) -> String {
    let hex = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("{prefix}:sha256:{hex}")
}
pub(super) fn named(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b':' | b'@'))
}
pub(super) fn hashed(value: &str, prefix: &str) -> bool {
    value
        .strip_prefix(&format!("{prefix}:sha256:"))
        .is_some_and(|hex| {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        })
}
macro_rules! id {
    ($name:ident, $prefix:literal) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);
        impl $name {
            pub fn parse(value: impl Into<String>) -> StoreResult<Self> {
                Self::try_from(value.into())
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
            pub(super) fn derive(bytes: &[u8]) -> Self {
                Self(digest($prefix, bytes))
            }
        }
        impl TryFrom<String> for $name {
            type Error = StoreError;
            fn try_from(value: String) -> StoreResult<Self> {
                if hashed(&value, $prefix) {
                    Ok(Self(value))
                } else {
                    Err(failure(StoreErrorCode::IdentifierInvalid))
                }
            }
        }
        impl From<$name> for String {
            fn from(value: $name) -> String {
                value.0
            }
        }
    };
}
id!(EpochId, "project-epoch");
id!(PartitionVersionId, "project-partition");
id!(StoreGenerationId, "project-store-generation");
id!(CurrentRecordId, "project-current");
id!(ValidationId, "project-validation");

/// Static Rust registrations, never SQL or registrations loaded from project data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordCatalog {
    schemas: BTreeSet<String>,
    checks: BTreeSet<String>,
}
impl RecordCatalog {
    pub fn new(schemas: &[&'static str], checks: &[&'static str]) -> StoreResult<Self> {
        let result = Self {
            schemas: schemas.iter().map(|s| (*s).to_owned()).collect(),
            checks: checks.iter().map(|s| (*s).to_owned()).collect(),
        };
        result.validate()?;
        if result.schemas.len() != schemas.len() || result.checks.len() != checks.len() {
            return Err(invalid());
        }
        Ok(result)
    }
    fn validate(&self) -> StoreResult<()> {
        if self.schemas.is_empty()
            || self.schemas.len() > 64
            || self.checks.is_empty()
            || self.checks.len() > 64
            || self.schemas.iter().chain(&self.checks).any(|s| !named(s))
        {
            return Err(failure(StoreErrorCode::ConfigurationInvalid));
        }
        Ok(())
    }
    pub(super) fn admits(&self, schema: &str) -> bool {
        self.schemas.contains(schema)
    }
    pub(super) fn checks(&self) -> &BTreeSet<String> {
        &self.checks
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochManifest {
    pub(super) schema: String,
    pub(super) physical_profile: String,
    pub(super) owner: String,
    pub(super) catalog: RecordCatalog,
    pub(super) sqlite_runtime_digest: String,
    pub(super) schema_digest: String,
    pub(super) epoch_id: EpochId,
}
impl EpochManifest {
    pub(super) fn new(
        owner: &str,
        catalog: RecordCatalog,
        runtime: String,
        schema_digest: String,
    ) -> StoreResult<Self> {
        if !named(owner) {
            return Err(failure(StoreErrorCode::ConfigurationInvalid));
        }
        catalog.validate()?;
        let bytes = encode(
            &(
                RECORD_PROFILE,
                PHYSICAL_PROFILE,
                owner,
                &catalog,
                &runtime,
                &schema_digest,
            ),
            65536,
        )?;
        Ok(Self {
            schema: RECORD_PROFILE.into(),
            physical_profile: PHYSICAL_PROFILE.into(),
            owner: owner.into(),
            catalog,
            sqlite_runtime_digest: runtime,
            schema_digest,
            epoch_id: EpochId::derive(&bytes),
        })
    }
    pub fn epoch_id(&self) -> &EpochId {
        &self.epoch_id
    }
    pub fn owner(&self) -> &str {
        &self.owner
    }
}

/// One owner-defined immutable logical partition. Payload is canonical JSON.
/// Neither a caller-provided table nor a whole SQLite database image is accepted.
#[derive(Debug, Clone)]
pub struct PartitionRecord {
    pub(super) key: String,
    pub(super) schema: String,
    pub(super) bytes: Vec<u8>,
    pub(super) version: PartitionVersionId,
}
impl PartitionRecord {
    pub fn new(
        key: impl Into<String>,
        schema: &'static str,
        value: &impl Serialize,
    ) -> StoreResult<Self> {
        let key = key.into();
        let bytes = encode(value, MAX_RECORD_BYTES)?;
        Self::from_bytes(key, schema.to_owned(), bytes)
    }
    pub(super) fn from_bytes(key: String, schema: String, bytes: Vec<u8>) -> StoreResult<Self> {
        if !named(&key) || !named(&schema) || bytes.len() > MAX_RECORD_BYTES {
            return Err(invalid());
        }
        let value: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        if encode(&value, MAX_RECORD_BYTES)? != bytes {
            return Err(invalid());
        }
        let header = encode(
            &(RECORD_PROFILE, &key, &schema, digest("record", &bytes)),
            2048,
        )?;
        Ok(Self {
            key,
            schema,
            version: PartitionVersionId::derive(&header),
            bytes,
        })
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn version(&self) -> &PartitionVersionId {
        &self.version
    }
    pub fn decode<T: serde::de::DeserializeOwned>(&self) -> StoreResult<T> {
        serde_json::from_slice(&self.bytes).map_err(|_| invalid())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionMember {
    pub key: String,
    pub schema: String,
    pub version: PartitionVersionId,
    pub byte_length: usize,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GenerationManifest {
    pub epoch_id: EpochId,
    pub owner: String,
    /// Opaque exact owner identities; only their owner interprets them.
    pub bindings: BTreeMap<String, String>,
    /// Full membership; no base/delta lookup is needed to read a generation.
    pub members: Vec<PartitionMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_current: Option<CurrentRecordId>,
    pub generation_id: StoreGenerationId,
}
impl GenerationManifest {
    fn bytes_for_id(&self) -> StoreResult<Vec<u8>> {
        // Serialize optional base as a zero/one list, not prohibited JSON null.
        encode(
            &(
                RECORD_PROFILE,
                &self.epoch_id,
                &self.owner,
                &self.bindings,
                &self.members,
                self.expected_current.iter().collect::<Vec<_>>(),
            ),
            256 * 1024,
        )
    }
    pub(super) fn validate(&self, epoch: &EpochManifest) -> StoreResult<()> {
        if self.epoch_id != epoch.epoch_id
            || self.owner != epoch.owner
            || self.members.is_empty()
            || self.members.len() > MAX_PARTITIONS
            || self.bindings.is_empty()
            || self.bindings.len() > 32
            || self.bindings.iter().any(|(k, v)| !named(k) || !named(v))
        {
            return Err(invalid());
        }
        let mut last: Option<&str> = None;
        let mut total = 0usize;
        for m in &self.members {
            if !named(&m.key)
                || !epoch.catalog.admits(&m.schema)
                || m.byte_length > MAX_RECORD_BYTES
                || last.is_some_and(|s| s >= m.key.as_str())
            {
                return Err(invalid());
            }
            last = Some(&m.key);
            total = total
                .checked_add(m.byte_length)
                .filter(|n| *n <= MAX_GENERATION_BYTES)
                .ok_or_else(invalid)?;
        }
        if self.generation_id != StoreGenerationId::derive(&self.bytes_for_id()?) {
            return Err(invalid());
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct PublicationRequest {
    pub(super) operation_id: OperationId,
    pub(super) digest: String,
    pub(super) manifest: GenerationManifest,
    pub(super) records: Vec<PartitionRecord>,
}
impl PublicationRequest {
    pub fn new(
        epoch: &EpochManifest,
        operation_id: OperationId,
        expected_current: Option<CurrentRecordId>,
        bindings: BTreeMap<String, String>,
        mut records: Vec<PartitionRecord>,
    ) -> StoreResult<Self> {
        OperationId::new(operation_id.as_str())?;
        records.sort_by(|a, b| a.key.cmp(&b.key));
        let members = records
            .iter()
            .map(|p| PartitionMember {
                key: p.key.clone(),
                schema: p.schema.clone(),
                version: p.version.clone(),
                byte_length: p.bytes.len(),
            })
            .collect();
        let mut manifest = GenerationManifest {
            epoch_id: epoch.epoch_id.clone(),
            owner: epoch.owner.clone(),
            bindings,
            members,
            expected_current,
            generation_id: StoreGenerationId::derive(b""),
        };
        manifest.generation_id = StoreGenerationId::derive(&manifest.bytes_for_id()?);
        manifest.validate(epoch)?;
        let digest = digest("project-request", &encode(&manifest, 256 * 1024)?);
        Ok(Self {
            operation_id,
            digest,
            manifest,
            records,
        })
    }
    pub fn generation(&self) -> &GenerationManifest {
        &self.manifest
    }
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
    pub fn request_digest(&self) -> &str {
        &self.digest
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationState {
    Prepared,
    PublishedInactive,
    ValidatedInactive,
    Activated,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentPublication {
    pub epoch_id: EpochId,
    pub generation_id: StoreGenerationId,
    pub validation_id: ValidationId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub predecessor: Option<CurrentRecordId>,
    pub record_id: CurrentRecordId,
}
impl CurrentPublication {
    pub(super) fn new(
        generation: &GenerationManifest,
        validation_id: ValidationId,
    ) -> StoreResult<Self> {
        let bytes = encode(
            &(
                &generation.epoch_id,
                &generation.generation_id,
                &validation_id,
                generation.expected_current.iter().collect::<Vec<_>>(),
            ),
            4096,
        )?;
        Ok(Self {
            epoch_id: generation.epoch_id.clone(),
            generation_id: generation.generation_id.clone(),
            validation_id,
            predecessor: generation.expected_current.clone(),
            record_id: CurrentRecordId::derive(&bytes),
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicationOperation {
    pub operation_id: OperationId,
    pub request_digest: String,
    pub generation_id: StoreGenerationId,
    pub state: PublicationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_id: Option<ValidationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation: Option<CurrentPublication>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ValidationRecord {
    pub generation_id: StoreGenerationId,
    pub checks: BTreeSet<String>,
    pub validation_id: ValidationId,
}
impl ValidationRecord {
    pub fn new(manifest: &GenerationManifest, checks: BTreeSet<String>) -> StoreResult<Self> {
        let bytes = encode(&(RECORD_PROFILE, &manifest.generation_id, &checks), 65536)?;
        Ok(Self {
            generation_id: manifest.generation_id.clone(),
            checks,
            validation_id: ValidationId::derive(&bytes),
        })
    }
}
