//! Typed E1-C persistence for deterministic annotation artifacts.
//!
//! Generation remains owned by the existing native projection pipeline. This module
//! freezes its canonical serialized result, binds it to exact producer inputs, and
//! delegates durable objects/catalogs/leases/GC to `wow-store`.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, CommitReceipt,
    GarbageCollectionReceipt, IntegrityReport, LeaseId, LeaseRecord, LogicalEpoch, LogicalManifest,
    ObjectId, ObjectRecord, PendingObject, Store, StoreError, StoreErrorCode, WriteBatch,
};

use crate::native::NativeLibrary;

pub const ANNOTATION_STORE_SCHEMA: &str = "wow-annotations/store/e1-c/1";
pub const ANNOTATION_ARTIFACT_SCHEMA: &str = "wow-annotations/artifact/1";
pub const ANNOTATION_OBJECT_KIND: &str = "wow.annotation.artifact";
pub const ANNOTATION_OBJECT_SCHEMA_VERSION: u32 = 1;
const CURRENT_CATALOG: &str = "annotation.current";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationStoreErrorCode {
    StoreConfigurationInvalid,
    StoreIdentifierInvalid,
    StoreJsonInvalid,
    StoreObjectTooLarge,
    StoreBatchTooLarge,
    StoreObjectMissing,
    StoreObjectConflict,
    StoreCatalogConflict,
    StoreOperationConflict,
    StoreOperationStateInvalid,
    StoreLeaseConflict,
    StoreLeaseInvalid,
    StoreIntegrityViolation,
    StoreBudgetExceeded,
    StoreDatabaseUnavailable,
    ArtifactIdentityInvalid,
    ArtifactPayloadInvalid,
    ArtifactKindMismatch,
    ArtifactSchemaMismatch,
    ArtifactDecodeFailed,
    PublicationKeyInvalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationStoreError {
    code: AnnotationStoreErrorCode,
    message: Box<str>,
}

impl AnnotationStoreError {
    fn new(code: AnnotationStoreErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> AnnotationStoreErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for AnnotationStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AnnotationStoreError {}

impl From<StoreError> for AnnotationStoreError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::ConfigurationInvalid => {
                AnnotationStoreErrorCode::StoreConfigurationInvalid
            }
            StoreErrorCode::IdentifierInvalid => AnnotationStoreErrorCode::StoreIdentifierInvalid,
            StoreErrorCode::JsonInvalid => AnnotationStoreErrorCode::StoreJsonInvalid,
            StoreErrorCode::ObjectTooLarge => AnnotationStoreErrorCode::StoreObjectTooLarge,
            StoreErrorCode::BatchTooLarge => AnnotationStoreErrorCode::StoreBatchTooLarge,
            StoreErrorCode::ObjectMissing => AnnotationStoreErrorCode::StoreObjectMissing,
            StoreErrorCode::ObjectConflict => AnnotationStoreErrorCode::StoreObjectConflict,
            StoreErrorCode::CatalogConflict => AnnotationStoreErrorCode::StoreCatalogConflict,
            StoreErrorCode::OperationConflict => AnnotationStoreErrorCode::StoreOperationConflict,
            StoreErrorCode::OperationStateInvalid => {
                AnnotationStoreErrorCode::StoreOperationStateInvalid
            }
            StoreErrorCode::LeaseConflict => AnnotationStoreErrorCode::StoreLeaseConflict,
            StoreErrorCode::LeaseInvalid => AnnotationStoreErrorCode::StoreLeaseInvalid,
            StoreErrorCode::IntegrityViolation => AnnotationStoreErrorCode::StoreIntegrityViolation,
            StoreErrorCode::BudgetExceeded => AnnotationStoreErrorCode::StoreBudgetExceeded,
            StoreErrorCode::DatabaseUnavailable => {
                AnnotationStoreErrorCode::StoreDatabaseUnavailable
            }
        };
        Self::new(code, source.message())
    }
}

pub type AnnotationStoreResult<T> = Result<T, AnnotationStoreError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationArtifact {
    schema: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    payload_sha256: Box<str>,
    canonical_payload: Box<[u8]>,
    artifact_id: Box<str>,
}

impl AnnotationArtifact {
    pub fn from_native_library(
        producer_version: impl Into<Box<str>>,
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
        library: &NativeLibrary<'_>,
    ) -> AnnotationStoreResult<Self> {
        Self::from_serializable(
            "wow-annotations",
            producer_version,
            profile_id,
            source_generation_id,
            library,
        )
    }

    fn from_serializable<T: Serialize + ?Sized>(
        producer_id: impl Into<Box<str>>,
        producer_version: impl Into<Box<str>>,
        profile_id: impl Into<Box<str>>,
        source_generation_id: impl Into<Box<str>>,
        value: &T,
    ) -> AnnotationStoreResult<Self> {
        let producer_id = producer_id.into();
        let producer_version = producer_version.into();
        let profile_id = profile_id.into();
        let source_generation_id = source_generation_id.into();
        for value in [
            producer_id.as_ref(),
            producer_version.as_ref(),
            profile_id.as_ref(),
            source_generation_id.as_ref(),
        ] {
            validate_identity_component(value)?;
        }
        let canonical_payload = canonical_json_bytes(value).map_err(|_| {
            AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactPayloadInvalid,
                "annotation payload cannot be canonicalized",
            )
        })?;
        serde_json::from_slice::<serde_json::Value>(&canonical_payload).map_err(|_| {
            AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactPayloadInvalid,
                "annotation payload is not valid canonical JSON",
            )
        })?;
        let payload_sha256 =
            format!("sha256:{}", hex(&Sha256::digest(&canonical_payload))).into_boxed_str();
        let artifact_id = artifact_id(
            &producer_id,
            &producer_version,
            &profile_id,
            &source_generation_id,
            &payload_sha256,
        )?;
        Ok(Self {
            schema: ANNOTATION_ARTIFACT_SCHEMA.into(),
            producer_id,
            producer_version,
            profile_id,
            source_generation_id,
            payload_sha256,
            canonical_payload: canonical_payload.into_boxed_slice(),
            artifact_id,
        })
    }

    pub fn validate(&self) -> AnnotationStoreResult<()> {
        if self.schema.as_ref() != ANNOTATION_ARTIFACT_SCHEMA {
            return Err(AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactSchemaMismatch,
                "annotation artifact schema is unsupported",
            ));
        }
        for value in [
            self.producer_id.as_ref(),
            self.producer_version.as_ref(),
            self.profile_id.as_ref(),
            self.source_generation_id.as_ref(),
        ] {
            validate_identity_component(value)?;
        }
        serde_json::from_slice::<serde_json::Value>(&self.canonical_payload).map_err(|_| {
            AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactPayloadInvalid,
                "annotation artifact payload is invalid JSON",
            )
        })?;
        let payload_sha256 = format!("sha256:{}", hex(&Sha256::digest(&self.canonical_payload)));
        if self.payload_sha256.as_ref() != payload_sha256 {
            return Err(AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactPayloadInvalid,
                "annotation artifact payload digest does not match",
            ));
        }
        let expected = artifact_id(
            &self.producer_id,
            &self.producer_version,
            &self.profile_id,
            &self.source_generation_id,
            &self.payload_sha256,
        )?;
        if self.artifact_id != expected {
            return Err(AnnotationStoreError::new(
                AnnotationStoreErrorCode::ArtifactIdentityInvalid,
                "annotation artifact identity does not match its inputs",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub fn payload_sha256(&self) -> &str {
        &self.payload_sha256
    }

    #[must_use]
    pub fn canonical_payload(&self) -> &[u8] {
        &self.canonical_payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationPublicationKey {
    profile: Box<str>,
    environment: Box<str>,
    artifact_family: Box<str>,
    catalog_path: CatalogPath,
}

impl AnnotationPublicationKey {
    pub fn new(
        profile: impl Into<Box<str>>,
        environment: impl Into<Box<str>>,
        artifact_family: impl Into<Box<str>>,
    ) -> AnnotationStoreResult<Self> {
        let profile = profile.into();
        let environment = environment.into();
        let artifact_family = artifact_family.into();
        for value in [
            profile.as_ref(),
            environment.as_ref(),
            artifact_family.as_ref(),
        ] {
            validate_identity_component(value)?;
        }
        let catalog_path = CatalogPath::new(format!("{profile}/{environment}/{artifact_family}"))?;
        Ok(Self {
            profile,
            environment,
            artifact_family,
            catalog_path,
        })
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn environment(&self) -> &str {
        &self.environment
    }

    #[must_use]
    pub fn artifact_family(&self) -> &str {
        &self.artifact_family
    }

    #[must_use]
    pub fn catalog_path(&self) -> &CatalogPath {
        &self.catalog_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredAnnotationArtifact {
    schema: &'static str,
    object_id: ObjectId,
    artifact_id: Box<str>,
    commit_receipt: CommitReceipt,
}

impl StoredAnnotationArtifact {
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    #[must_use]
    pub fn commit_receipt(&self) -> &CommitReceipt {
        &self.commit_receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedAnnotationArtifact {
    schema: &'static str,
    publication_key: AnnotationPublicationKey,
    object_id: ObjectId,
    artifact: AnnotationArtifact,
}

impl PublishedAnnotationArtifact {
    #[must_use]
    pub fn publication_key(&self) -> &AnnotationPublicationKey {
        &self.publication_key
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn artifact(&self) -> &AnnotationArtifact {
        &self.artifact
    }

    #[must_use]
    pub fn into_artifact(self) -> AnnotationArtifact {
        self.artifact
    }
}

pub struct PersistentAnnotationStore<'store> {
    store: &'store mut Store,
}

impl<'store> PersistentAnnotationStore<'store> {
    #[must_use]
    pub fn new(store: &'store mut Store) -> Self {
        Self { store }
    }

    pub fn store_artifact(
        &mut self,
        artifact: &AnnotationArtifact,
    ) -> AnnotationStoreResult<StoredAnnotationArtifact> {
        artifact.validate()?;
        let pending = self.pending_artifact(artifact)?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        let commit_receipt = self.store.commit(batch)?;
        Ok(StoredAnnotationArtifact {
            schema: ANNOTATION_STORE_SCHEMA,
            object_id,
            artifact_id: artifact.artifact_id().into(),
            commit_receipt,
        })
    }

    pub fn publish_current(
        &mut self,
        publication_key: AnnotationPublicationKey,
        artifact: &AnnotationArtifact,
        expectation: CatalogExpectation,
    ) -> AnnotationStoreResult<StoredAnnotationArtifact> {
        artifact.validate()?;
        let pending = self.pending_artifact(artifact)?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        batch.add_catalog_mutation(CatalogMutation::set(
            current_catalog()?,
            publication_key.catalog_path().clone(),
            expectation,
            object_id.clone(),
        ))?;
        let commit_receipt = self.store.commit(batch)?;
        Ok(StoredAnnotationArtifact {
            schema: ANNOTATION_STORE_SCHEMA,
            object_id,
            artifact_id: artifact.artifact_id().into(),
            commit_receipt,
        })
    }

    pub fn read_exact(
        &self,
        object_id: &ObjectId,
    ) -> AnnotationStoreResult<Option<AnnotationArtifact>> {
        let Some(record) = self.store.object(object_id)? else {
            return Ok(None);
        };
        Ok(Some(decode_annotation_artifact(&record)?))
    }

    pub fn read_current(
        &self,
        publication_key: &AnnotationPublicationKey,
    ) -> AnnotationStoreResult<Option<PublishedAnnotationArtifact>> {
        let Some(entry) = self
            .store
            .catalog_entry(&current_catalog()?, publication_key.catalog_path())?
        else {
            return Ok(None);
        };
        let object_id = entry.object_id().clone();
        let artifact = self.read_exact(&object_id)?.ok_or_else(|| {
            AnnotationStoreError::new(
                AnnotationStoreErrorCode::StoreIntegrityViolation,
                "published annotation object is missing",
            )
        })?;
        Ok(Some(PublishedAnnotationArtifact {
            schema: ANNOTATION_STORE_SCHEMA,
            publication_key: publication_key.clone(),
            object_id,
            artifact,
        }))
    }

    pub fn unpublish_current(
        &mut self,
        publication_key: &AnnotationPublicationKey,
        expected: ObjectId,
    ) -> AnnotationStoreResult<CommitReceipt> {
        let mut batch = WriteBatch::new();
        batch.add_catalog_mutation(CatalogMutation::delete(
            current_catalog()?,
            publication_key.catalog_path().clone(),
            expected,
        ))?;
        Ok(self.store.commit(batch)?)
    }

    pub fn retain(
        &mut self,
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> AnnotationStoreResult<LeaseRecord> {
        Ok(self
            .store
            .acquire_lease(lease_id, object_id, holder, expires_after)?)
    }

    pub fn renew_retention(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
        expected_expiry: LogicalEpoch,
        new_expiry: LogicalEpoch,
    ) -> AnnotationStoreResult<LeaseRecord> {
        Ok(self
            .store
            .renew_lease(lease_id, holder, expected_expiry, new_expiry)?)
    }

    pub fn release_retention(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
    ) -> AnnotationStoreResult<()> {
        Ok(self.store.release_lease(lease_id, holder)?)
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> AnnotationStoreResult<GarbageCollectionReceipt> {
        Ok(self.store.collect_garbage(now, max_deletes)?)
    }

    pub fn validate_integrity(&self, max_objects: u32) -> AnnotationStoreResult<IntegrityReport> {
        Ok(self.store.validate_integrity(max_objects)?)
    }

    pub fn logical_manifest(&self) -> AnnotationStoreResult<LogicalManifest> {
        Ok(self.store.logical_manifest()?)
    }

    fn pending_artifact(
        &self,
        artifact: &AnnotationArtifact,
    ) -> AnnotationStoreResult<PendingObject> {
        Ok(PendingObject::from_json(
            ANNOTATION_OBJECT_KIND,
            ANNOTATION_OBJECT_SCHEMA_VERSION,
            artifact,
            self.store.configuration().limits(),
        )?)
    }
}

fn current_catalog() -> AnnotationStoreResult<CatalogName> {
    Ok(CatalogName::new(CURRENT_CATALOG)?)
}

fn decode_annotation_artifact(record: &ObjectRecord) -> AnnotationStoreResult<AnnotationArtifact> {
    if record.kind() != ANNOTATION_OBJECT_KIND {
        return Err(AnnotationStoreError::new(
            AnnotationStoreErrorCode::ArtifactKindMismatch,
            "stored object is not an annotation artifact",
        ));
    }
    if record.schema_version() != ANNOTATION_OBJECT_SCHEMA_VERSION {
        return Err(AnnotationStoreError::new(
            AnnotationStoreErrorCode::ArtifactSchemaMismatch,
            "stored annotation artifact schema is unsupported",
        ));
    }
    let artifact: AnnotationArtifact = record.decode().map_err(|_| {
        AnnotationStoreError::new(
            AnnotationStoreErrorCode::ArtifactDecodeFailed,
            "stored annotation artifact cannot be decoded",
        )
    })?;
    artifact.validate()?;
    Ok(artifact)
}

fn artifact_id(
    producer_id: &str,
    producer_version: &str,
    profile_id: &str,
    source_generation_id: &str,
    payload_sha256: &str,
) -> AnnotationStoreResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        producer_id: &'a str,
        producer_version: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        payload_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: ANNOTATION_ARTIFACT_SCHEMA,
        producer_id,
        producer_version,
        profile_id,
        source_generation_id,
        payload_sha256,
    })
    .map_err(|_| {
        AnnotationStoreError::new(
            AnnotationStoreErrorCode::ArtifactIdentityInvalid,
            "annotation artifact identity cannot be canonicalized",
        )
    })?;
    Ok(format!("annotation-artifact:sha256:{}", hex(&Sha256::digest(bytes))).into())
}

fn validate_identity_component(value: &str) -> AnnotationStoreResult<()> {
    if value.is_empty()
        || value.len() > 512
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
    {
        return Err(AnnotationStoreError::new(
            AnnotationStoreErrorCode::ArtifactIdentityInvalid,
            "annotation identity component is invalid",
        ));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wow_store::{StoreConfiguration, StoreLimits};

    fn artifact(value: serde_json::Value) -> AnnotationStoreResult<AnnotationArtifact> {
        AnnotationArtifact::from_serializable(
            "wow-annotations",
            "1.0.0",
            "retail-12.1",
            "reference-generation:fixture",
            &value,
        )
    }

    #[test]
    fn canonical_payload_and_inputs_define_artifact_identity()
    -> Result<(), Box<dyn std::error::Error>> {
        let left = artifact(json!({"zeta":2,"alpha":1}))?;
        let right = artifact(json!({"alpha":1,"zeta":2}))?;
        assert_eq!(left, right);
        assert!(
            left.artifact_id()
                .starts_with("annotation-artifact:sha256:")
        );
        left.validate()?;
        Ok(())
    }

    #[test]
    fn exact_publish_read_and_cas_are_store_owned() -> Result<(), Box<dyn std::error::Error>> {
        let configuration = StoreConfiguration::new("annotation-test", StoreLimits::default())?;
        let mut store = Store::open_in_memory(configuration)?;
        let mut facade = PersistentAnnotationStore::new(&mut store);
        let key = AnnotationPublicationKey::new("retail-12.1", "mainline", "emmy-library")?;
        let first = artifact(json!({"files":[{"path":"A.lua"}]}))?;
        let stored = facade.publish_current(key.clone(), &first, CatalogExpectation::Absent)?;
        let current = facade
            .read_current(&key)?
            .ok_or("missing current artifact")?;
        assert_eq!(current.object_id(), stored.object_id());
        assert_eq!(current.artifact(), &first);
        let second = artifact(json!({"files":[{"path":"B.lua"}]}))?;
        let error = facade
            .publish_current(key.clone(), &second, CatalogExpectation::Absent)
            .err()
            .ok_or("expected catalog conflict")?;
        assert_eq!(error.code(), AnnotationStoreErrorCode::StoreCatalogConflict);
        let replaced = facade.publish_current(
            key.clone(),
            &second,
            CatalogExpectation::Exact(stored.object_id().clone()),
        )?;
        assert_eq!(
            facade
                .read_current(&key)?
                .ok_or("missing replacement")?
                .object_id(),
            replaced.object_id()
        );
        Ok(())
    }

    #[test]
    fn wrong_kind_and_tampered_artifact_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
        let configuration = StoreConfiguration::new("annotation-kind", StoreLimits::default())?;
        let mut store = Store::open_in_memory(configuration)?;
        let pending = PendingObject::from_json(
            "fixture.not-annotation",
            1,
            &json!({"value":1}),
            StoreLimits::default(),
        )?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        store.commit(batch)?;
        let facade = PersistentAnnotationStore::new(&mut store);
        assert_eq!(
            facade
                .read_exact(&object_id)
                .err()
                .ok_or("expected kind mismatch")?
                .code(),
            AnnotationStoreErrorCode::ArtifactKindMismatch
        );

        let mut value = serde_json::to_value(artifact(json!({"value":1}))?)?;
        value["payload_sha256"] = json!(format!("sha256:{}", "0".repeat(64)));
        let tampered: AnnotationArtifact = serde_json::from_value(value)?;
        assert_eq!(
            tampered
                .validate()
                .err()
                .ok_or("expected digest mismatch")?
                .code(),
            AnnotationStoreErrorCode::ArtifactPayloadInvalid
        );
        Ok(())
    }

    #[test]
    fn native_library_type_remains_a_serializable_generation_input() {
        fn assert_serializable<T: Serialize + ?Sized>() {}
        assert_serializable::<NativeLibrary<'_>>();
    }
}
