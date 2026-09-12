use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;

use crate::{
    CatalogName, CatalogPath, LeaseId, LogicalEpoch, ObjectId, OperationId, RequestDigest,
    StoreError, StoreErrorCode, StoreResult,
};

pub const STORE_SCHEMA: &str = "wow-store/e1-a/1";
pub const STORE_OBJECT_SCHEMA: &str = "wow-store/object/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreLimits {
    pub max_object_bytes: u64,
    pub max_batch_objects: u32,
    pub max_catalog_mutations: u32,
    pub max_manifest_records: u32,
    pub max_gc_deletes: u32,
}

impl StoreLimits {
    pub fn new(
        max_object_bytes: u64,
        max_batch_objects: u32,
        max_catalog_mutations: u32,
        max_manifest_records: u32,
        max_gc_deletes: u32,
    ) -> StoreResult<Self> {
        let limits = Self {
            max_object_bytes,
            max_batch_objects,
            max_catalog_mutations,
            max_manifest_records,
            max_gc_deletes,
        };
        limits.validate()?;
        Ok(limits)
    }

    pub(crate) fn validate(self) -> StoreResult<()> {
        if self.max_object_bytes == 0
            || self.max_object_bytes > 64 * 1024 * 1024
            || self.max_batch_objects == 0
            || self.max_batch_objects > 16_384
            || self.max_catalog_mutations == 0
            || self.max_catalog_mutations > 16_384
            || self.max_manifest_records == 0
            || self.max_manifest_records > 1_000_000
            || self.max_gc_deletes == 0
            || self.max_gc_deletes > 100_000
        {
            return Err(StoreError::new(
                StoreErrorCode::ConfigurationInvalid,
                "store limits are outside the reviewed profile",
            ));
        }
        Ok(())
    }
}

impl Default for StoreLimits {
    fn default() -> Self {
        Self {
            max_object_bytes: 8 * 1024 * 1024,
            max_batch_objects: 1024,
            max_catalog_mutations: 1024,
            max_manifest_records: 100_000,
            max_gc_deletes: 10_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoreConfiguration {
    schema: &'static str,
    profile_id: Box<str>,
    limits: StoreLimits,
    configuration_id: Box<str>,
}

impl StoreConfiguration {
    pub fn new(profile_id: impl Into<Box<str>>, limits: StoreLimits) -> StoreResult<Self> {
        limits.validate()?;
        let profile_id = profile_id.into();
        if profile_id.is_empty()
            || profile_id.len() > 256
            || !profile_id.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
            })
        {
            return Err(StoreError::new(
                StoreErrorCode::ConfigurationInvalid,
                "invalid store profile id",
            ));
        }
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            profile_id: &'a str,
            limits: StoreLimits,
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: STORE_SCHEMA,
            profile_id: &profile_id,
            limits,
        })
        .map_err(|_| {
            StoreError::new(
                StoreErrorCode::JsonInvalid,
                "configuration canonicalization failed",
            )
        })?;
        let configuration_id =
            format!("store-configuration:sha256:{}", hex(&Sha256::digest(bytes))).into();
        Ok(Self {
            schema: STORE_SCHEMA,
            profile_id,
            limits,
            configuration_id,
        })
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub const fn limits(&self) -> StoreLimits {
        self.limits
    }

    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PendingObject {
    schema: &'static str,
    object_id: ObjectId,
    kind: Box<str>,
    schema_version: u32,
    content_sha256: Box<str>,
    canonical_json: Box<[u8]>,
}

impl PendingObject {
    pub fn from_json<T: Serialize + ?Sized>(
        kind: impl Into<Box<str>>,
        schema_version: u32,
        value: &T,
        limits: StoreLimits,
    ) -> StoreResult<Self> {
        limits.validate()?;
        let kind = kind.into();
        validate_kind(&kind)?;
        if schema_version == 0 {
            return Err(StoreError::new(
                StoreErrorCode::ConfigurationInvalid,
                "object schema version must be nonzero",
            ));
        }
        let canonical_json = canonical_json_bytes(value).map_err(|_| {
            StoreError::new(
                StoreErrorCode::JsonInvalid,
                "object canonicalization failed",
            )
        })?;
        if canonical_json.len() as u64 > limits.max_object_bytes {
            return Err(StoreError::new(
                StoreErrorCode::ObjectTooLarge,
                "object exceeds the configured byte budget",
            ));
        }
        serde_json::from_slice::<serde_json::Value>(&canonical_json).map_err(|_| {
            StoreError::new(
                StoreErrorCode::JsonInvalid,
                "canonical object is not valid JSON",
            )
        })?;
        let content_sha256 =
            format!("sha256:{}", hex(&Sha256::digest(&canonical_json))).into_boxed_str();
        let object_id = derive_object_id(&kind, schema_version, &canonical_json)?;
        Ok(Self {
            schema: STORE_OBJECT_SCHEMA,
            object_id,
            kind,
            schema_version,
            content_sha256,
            canonical_json: canonical_json.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    #[must_use]
    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectRecord {
    object_id: ObjectId,
    kind: Box<str>,
    schema_version: u32,
    content_sha256: Box<str>,
    canonical_json: Box<[u8]>,
}

impl ObjectRecord {
    pub(crate) fn from_parts(
        object_id: ObjectId,
        kind: Box<str>,
        schema_version: u32,
        content_sha256: Box<str>,
        canonical_json: Box<[u8]>,
    ) -> StoreResult<Self> {
        let expected = derive_object_id(&kind, schema_version, &canonical_json)?;
        let content = format!("sha256:{}", hex(&Sha256::digest(&canonical_json)));
        if object_id != expected || content_sha256.as_ref() != content {
            return Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "stored object identity does not match its bytes",
            ));
        }
        Ok(Self {
            object_id,
            kind,
            schema_version,
            content_sha256,
            canonical_json,
        })
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub fn content_sha256(&self) -> &str {
        &self.content_sha256
    }

    #[must_use]
    pub fn canonical_json(&self) -> &[u8] {
        &self.canonical_json
    }

    pub fn decode<T: DeserializeOwned>(&self) -> StoreResult<T> {
        serde_json::from_slice(&self.canonical_json).map_err(|_| {
            StoreError::new(StoreErrorCode::JsonInvalid, "stored object JSON is invalid")
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogExpectation {
    Absent,
    Exact(ObjectId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogMutation {
    catalog: CatalogName,
    path: CatalogPath,
    expectation: CatalogExpectation,
    target: Option<ObjectId>,
}

impl CatalogMutation {
    #[must_use]
    pub fn set(
        catalog: CatalogName,
        path: CatalogPath,
        expectation: CatalogExpectation,
        target: ObjectId,
    ) -> Self {
        Self {
            catalog,
            path,
            expectation,
            target: Some(target),
        }
    }

    #[must_use]
    pub fn delete(catalog: CatalogName, path: CatalogPath, expected: ObjectId) -> Self {
        Self {
            catalog,
            path,
            expectation: CatalogExpectation::Exact(expected),
            target: None,
        }
    }

    #[must_use]
    pub fn catalog(&self) -> &CatalogName {
        &self.catalog
    }

    #[must_use]
    pub fn path(&self) -> &CatalogPath {
        &self.path
    }

    #[must_use]
    pub fn expectation(&self) -> &CatalogExpectation {
        &self.expectation
    }

    #[must_use]
    pub fn target(&self) -> Option<&ObjectId> {
        self.target.as_ref()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WriteBatch {
    objects: Vec<PendingObject>,
    catalog_mutations: Vec<CatalogMutation>,
}

impl WriteBatch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&mut self, object: PendingObject) -> StoreResult<()> {
        if self
            .objects
            .iter()
            .any(|current| current.object_id() == object.object_id())
        {
            return Err(StoreError::new(
                StoreErrorCode::ObjectConflict,
                "batch contains a duplicate object id",
            ));
        }
        self.objects.push(object);
        Ok(())
    }

    pub fn add_catalog_mutation(&mut self, mutation: CatalogMutation) -> StoreResult<()> {
        if self.catalog_mutations.iter().any(|current| {
            current.catalog() == mutation.catalog() && current.path() == mutation.path()
        }) {
            return Err(StoreError::new(
                StoreErrorCode::CatalogConflict,
                "batch contains duplicate catalog mutation",
            ));
        }
        self.catalog_mutations.push(mutation);
        Ok(())
    }

    #[must_use]
    pub fn objects(&self) -> &[PendingObject] {
        &self.objects
    }

    #[must_use]
    pub fn catalog_mutations(&self) -> &[CatalogMutation] {
        &self.catalog_mutations
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    catalog: CatalogName,
    path: CatalogPath,
    object_id: ObjectId,
}

impl CatalogEntry {
    pub(crate) fn new(catalog: CatalogName, path: CatalogPath, object_id: ObjectId) -> Self {
        Self {
            catalog,
            path,
            object_id,
        }
    }

    #[must_use]
    pub fn catalog(&self) -> &CatalogName {
        &self.catalog
    }

    #[must_use]
    pub fn path(&self) -> &CatalogPath {
        &self.path
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogChange {
    catalog: CatalogName,
    path: CatalogPath,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current: Option<ObjectId>,
}

impl CatalogChange {
    pub(crate) fn new(
        catalog: CatalogName,
        path: CatalogPath,
        previous: Option<ObjectId>,
        current: Option<ObjectId>,
    ) -> Self {
        Self {
            catalog,
            path,
            previous,
            current,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CommitReceipt {
    receipt_id: Box<str>,
    inserted: Vec<ObjectId>,
    deduplicated: Vec<ObjectId>,
    catalog_changes: Vec<CatalogChange>,
}

impl CommitReceipt {
    pub(crate) fn build(
        mut inserted: Vec<ObjectId>,
        mut deduplicated: Vec<ObjectId>,
        mut catalog_changes: Vec<CatalogChange>,
    ) -> StoreResult<Self> {
        inserted.sort();
        deduplicated.sort();
        catalog_changes.sort();
        #[derive(Serialize)]
        struct Identity<'a> {
            inserted: &'a [ObjectId],
            deduplicated: &'a [ObjectId],
            catalog_changes: &'a [CatalogChange],
        }
        let bytes = canonical_json_bytes(&Identity {
            inserted: &inserted,
            deduplicated: &deduplicated,
            catalog_changes: &catalog_changes,
        })
        .map_err(|_| {
            StoreError::new(
                StoreErrorCode::JsonInvalid,
                "receipt canonicalization failed",
            )
        })?;
        Ok(Self {
            receipt_id: format!("store-commit:sha256:{}", hex(&Sha256::digest(bytes))).into(),
            inserted,
            deduplicated,
            catalog_changes,
        })
    }

    #[must_use]
    pub fn receipt_id(&self) -> &str {
        &self.receipt_id
    }

    #[must_use]
    pub fn inserted(&self) -> &[ObjectId] {
        &self.inserted
    }

    #[must_use]
    pub fn deduplicated(&self) -> &[ObjectId] {
        &self.deduplicated
    }

    #[must_use]
    pub fn catalog_changes(&self) -> &[CatalogChange] {
        &self.catalog_changes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Prepared,
    Completed,
    NoEffect,
    OutcomeUnknown,
    Failed,
}

impl OperationState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Prepared => "prepared",
            Self::Completed => "completed",
            Self::NoEffect => "no_effect",
            Self::OutcomeUnknown => "outcome_unknown",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn parse(value: &str) -> StoreResult<Self> {
        match value {
            "prepared" => Ok(Self::Prepared),
            "completed" => Ok(Self::Completed),
            "no_effect" => Ok(Self::NoEffect),
            "outcome_unknown" => Ok(Self::OutcomeUnknown),
            "failed" => Ok(Self::Failed),
            _ => Err(StoreError::new(
                StoreErrorCode::IntegrityViolation,
                "stored operation state is invalid",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationRecord {
    operation_id: OperationId,
    request_digest: RequestDigest,
    state: OperationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_object_id: Option<ObjectId>,
}

impl OperationRecord {
    pub(crate) fn new(
        operation_id: OperationId,
        request_digest: RequestDigest,
        state: OperationState,
        result_object_id: Option<ObjectId>,
    ) -> StoreResult<Self> {
        if result_object_id.is_some() != matches!(state, OperationState::Completed) {
            return Err(StoreError::new(
                StoreErrorCode::OperationStateInvalid,
                "operation result identity does not match terminal state",
            ));
        }
        Ok(Self {
            operation_id,
            request_digest,
            state,
            result_object_id,
        })
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }

    #[must_use]
    pub fn request_digest(&self) -> &RequestDigest {
        &self.request_digest
    }

    #[must_use]
    pub const fn state(&self) -> OperationState {
        self.state
    }

    #[must_use]
    pub fn result_object_id(&self) -> Option<&ObjectId> {
        self.result_object_id.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationBegin {
    Started(OperationRecord),
    Replay(OperationRecord),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LeaseRecord {
    lease_id: LeaseId,
    object_id: ObjectId,
    holder: Box<str>,
    expires_after: LogicalEpoch,
}

impl LeaseRecord {
    pub(crate) fn new(
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: Box<str>,
        expires_after: LogicalEpoch,
    ) -> StoreResult<Self> {
        if holder.is_empty()
            || holder.len() > 256
            || !holder
                .bytes()
                .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'"' | b'\\'))
        {
            return Err(StoreError::new(
                StoreErrorCode::LeaseInvalid,
                "invalid lease holder",
            ));
        }
        Ok(Self {
            lease_id,
            object_id,
            holder,
            expires_after,
        })
    }

    #[must_use]
    pub fn lease_id(&self) -> &LeaseId {
        &self.lease_id
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn holder(&self) -> &str {
        &self.holder
    }

    #[must_use]
    pub const fn expires_after(&self) -> LogicalEpoch {
        self.expires_after
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GarbageCollectionReceipt {
    deleted: Vec<ObjectId>,
    expired_leases_deleted: u64,
    more_available: bool,
}

impl GarbageCollectionReceipt {
    pub(crate) fn new(
        deleted: Vec<ObjectId>,
        expired_leases_deleted: u64,
        more_available: bool,
    ) -> Self {
        Self {
            deleted,
            expired_leases_deleted,
            more_available,
        }
    }

    #[must_use]
    pub fn deleted(&self) -> &[ObjectId] {
        &self.deleted
    }

    #[must_use]
    pub const fn expired_leases_deleted(&self) -> u64 {
        self.expired_leases_deleted
    }

    #[must_use]
    pub const fn more_available(&self) -> bool {
        self.more_available
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntegrityReport {
    checked_objects: u64,
    checked_catalog_entries: u64,
    checked_operations: u64,
    checked_leases: u64,
    complete: bool,
}

impl IntegrityReport {
    pub(crate) fn new(
        checked_objects: u64,
        checked_catalog_entries: u64,
        checked_operations: u64,
        checked_leases: u64,
        complete: bool,
    ) -> Self {
        Self {
            checked_objects,
            checked_catalog_entries,
            checked_operations,
            checked_leases,
            complete,
        }
    }

    #[must_use]
    pub const fn complete(&self) -> bool {
        self.complete
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LogicalManifest {
    schema: &'static str,
    configuration_id: Box<str>,
    objects: Vec<ObjectId>,
    catalog_entries: Vec<CatalogEntry>,
    operations: Vec<OperationRecord>,
    leases: Vec<LeaseRecord>,
    manifest_id: Box<str>,
}

impl LogicalManifest {
    pub(crate) fn build(
        configuration_id: Box<str>,
        mut objects: Vec<ObjectId>,
        mut catalog_entries: Vec<CatalogEntry>,
        mut operations: Vec<OperationRecord>,
        mut leases: Vec<LeaseRecord>,
    ) -> StoreResult<Self> {
        objects.sort();
        catalog_entries.sort();
        operations.sort_by(|left, right| left.operation_id().cmp(right.operation_id()));
        leases.sort_by(|left, right| left.lease_id().cmp(right.lease_id()));
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            configuration_id: &'a str,
            objects: &'a [ObjectId],
            catalog_entries: &'a [CatalogEntry],
            operations: &'a [OperationRecord],
            leases: &'a [LeaseRecord],
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: STORE_SCHEMA,
            configuration_id: &configuration_id,
            objects: &objects,
            catalog_entries: &catalog_entries,
            operations: &operations,
            leases: &leases,
        })
        .map_err(|_| {
            StoreError::new(
                StoreErrorCode::JsonInvalid,
                "manifest canonicalization failed",
            )
        })?;
        Ok(Self {
            schema: STORE_SCHEMA,
            configuration_id,
            objects,
            catalog_entries,
            operations,
            leases,
            manifest_id: format!("store-manifest:sha256:{}", hex(&Sha256::digest(bytes))).into(),
        })
    }

    #[must_use]
    pub fn manifest_id(&self) -> &str {
        &self.manifest_id
    }

    #[must_use]
    pub fn objects(&self) -> &[ObjectId] {
        &self.objects
    }
}

pub(crate) fn derive_object_id(
    kind: &str,
    schema_version: u32,
    canonical_json: &[u8],
) -> StoreResult<ObjectId> {
    validate_kind(kind)?;
    let mut hash = Sha256::new();
    hash.update(b"wow-store-object-v1\0");
    hash.update(kind.as_bytes());
    hash.update([0]);
    hash.update(schema_version.to_be_bytes());
    hash.update([0]);
    hash.update(canonical_json);
    ObjectId::new(format!("store-object:sha256:{}", hex(&hash.finalize())))
}

fn validate_kind(kind: &str) -> StoreResult<()> {
    if kind.is_empty()
        || kind.len() > 256
        || !kind.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
    {
        return Err(StoreError::new(
            StoreErrorCode::IdentifierInvalid,
            "invalid object kind",
        ));
    }
    Ok(())
}

pub(crate) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
