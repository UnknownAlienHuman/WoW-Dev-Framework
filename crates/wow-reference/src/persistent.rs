//! Typed E1-B persistence over the generic `wow-store` owner.
//!
//! Reference semantics remain owned by `ReferenceView`; this module only binds its
//! strict wire representation to immutable store objects and explicit publication CAS.

use std::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, CommitReceipt,
    GarbageCollectionReceipt, IntegrityReport, LeaseId, LeaseRecord, LogicalEpoch,
    LogicalManifest, ObjectId, ObjectRecord, PendingObject, Store, StoreError, StoreErrorCode,
    WriteBatch,
};

use crate::ReferenceView;

pub const REFERENCE_STORE_SCHEMA: &str = "wow-reference/store/e1-b/1";
pub const REFERENCE_VIEW_OBJECT_KIND: &str = "wow.reference.view";
pub const REFERENCE_VIEW_OBJECT_SCHEMA_VERSION: u32 = 1;
const CURRENT_CATALOG: &str = "reference.current";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceStoreErrorCode {
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
    PublicationKeyInvalid,
    ArtifactKindMismatch,
    ArtifactSchemaMismatch,
    ArtifactDecodeFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceStoreError {
    code: ReferenceStoreErrorCode,
    message: Box<str>,
}

impl ReferenceStoreError {
    fn new(code: ReferenceStoreErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ReferenceStoreErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReferenceStoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReferenceStoreError {}

impl From<StoreError> for ReferenceStoreError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::ConfigurationInvalid => {
                ReferenceStoreErrorCode::StoreConfigurationInvalid
            }
            StoreErrorCode::IdentifierInvalid => ReferenceStoreErrorCode::StoreIdentifierInvalid,
            StoreErrorCode::JsonInvalid => ReferenceStoreErrorCode::StoreJsonInvalid,
            StoreErrorCode::ObjectTooLarge => ReferenceStoreErrorCode::StoreObjectTooLarge,
            StoreErrorCode::BatchTooLarge => ReferenceStoreErrorCode::StoreBatchTooLarge,
            StoreErrorCode::ObjectMissing => ReferenceStoreErrorCode::StoreObjectMissing,
            StoreErrorCode::ObjectConflict => ReferenceStoreErrorCode::StoreObjectConflict,
            StoreErrorCode::CatalogConflict => ReferenceStoreErrorCode::StoreCatalogConflict,
            StoreErrorCode::OperationConflict => ReferenceStoreErrorCode::StoreOperationConflict,
            StoreErrorCode::OperationStateInvalid => {
                ReferenceStoreErrorCode::StoreOperationStateInvalid
            }
            StoreErrorCode::LeaseConflict => ReferenceStoreErrorCode::StoreLeaseConflict,
            StoreErrorCode::LeaseInvalid => ReferenceStoreErrorCode::StoreLeaseInvalid,
            StoreErrorCode::IntegrityViolation => {
                ReferenceStoreErrorCode::StoreIntegrityViolation
            }
            StoreErrorCode::BudgetExceeded => ReferenceStoreErrorCode::StoreBudgetExceeded,
            StoreErrorCode::DatabaseUnavailable => {
                ReferenceStoreErrorCode::StoreDatabaseUnavailable
            }
        };
        Self::new(code, source.message())
    }
}

pub type ReferenceStoreResult<T> = Result<T, ReferenceStoreError>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePublicationKey {
    profile: Box<str>,
    channel: Box<str>,
    catalog_path: CatalogPath,
}

impl ReferencePublicationKey {
    pub fn new(
        profile: impl Into<Box<str>>,
        channel: impl Into<Box<str>>,
    ) -> ReferenceStoreResult<Self> {
        let profile = profile.into();
        let channel = channel.into();
        if !valid_component(&profile) || !valid_component(&channel) {
            return Err(ReferenceStoreError::new(
                ReferenceStoreErrorCode::PublicationKeyInvalid,
                "invalid reference publication key",
            ));
        }
        let catalog_path = CatalogPath::new(format!("{profile}/{channel}"))?;
        Ok(Self {
            profile,
            channel,
            catalog_path,
        })
    }

    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub fn channel(&self) -> &str {
        &self.channel
    }

    #[must_use]
    pub fn catalog_path(&self) -> &CatalogPath {
        &self.catalog_path
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredReferenceView {
    schema: &'static str,
    object_id: ObjectId,
    commit_receipt: CommitReceipt,
}

impl StoredReferenceView {
    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn commit_receipt(&self) -> &CommitReceipt {
        &self.commit_receipt
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublishedReferenceView {
    schema: &'static str,
    publication_key: ReferencePublicationKey,
    object_id: ObjectId,
    view: ReferenceView,
}

impl PublishedReferenceView {
    #[must_use]
    pub fn publication_key(&self) -> &ReferencePublicationKey {
        &self.publication_key
    }

    #[must_use]
    pub fn object_id(&self) -> &ObjectId {
        &self.object_id
    }

    #[must_use]
    pub fn view(&self) -> &ReferenceView {
        &self.view
    }

    #[must_use]
    pub fn into_view(self) -> ReferenceView {
        self.view
    }
}

/// Borrowed typed façade over one generic durable store.
pub struct PersistentReferenceStore<'store> {
    store: &'store mut Store,
}

impl<'store> PersistentReferenceStore<'store> {
    #[must_use]
    pub fn new(store: &'store mut Store) -> Self {
        Self { store }
    }

    pub fn store_view(
        &mut self,
        view: &ReferenceView,
    ) -> ReferenceStoreResult<StoredReferenceView> {
        let pending = self.pending_view(view)?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        let commit_receipt = self.store.commit(batch)?;
        Ok(StoredReferenceView {
            schema: REFERENCE_STORE_SCHEMA,
            object_id,
            commit_receipt,
        })
    }

    pub fn publish_current(
        &mut self,
        publication_key: ReferencePublicationKey,
        view: &ReferenceView,
        expectation: CatalogExpectation,
    ) -> ReferenceStoreResult<StoredReferenceView> {
        let pending = self.pending_view(view)?;
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
        Ok(StoredReferenceView {
            schema: REFERENCE_STORE_SCHEMA,
            object_id,
            commit_receipt,
        })
    }

    pub fn unpublish_current(
        &mut self,
        publication_key: &ReferencePublicationKey,
        expected: ObjectId,
    ) -> ReferenceStoreResult<CommitReceipt> {
        let mut batch = WriteBatch::new();
        batch.add_catalog_mutation(CatalogMutation::delete(
            current_catalog()?,
            publication_key.catalog_path().clone(),
            expected,
        ))?;
        Ok(self.store.commit(batch)?)
    }

    pub fn read_exact(
        &self,
        object_id: &ObjectId,
    ) -> ReferenceStoreResult<Option<ReferenceView>> {
        let Some(record) = self.store.object(object_id)? else {
            return Ok(None);
        };
        Ok(Some(decode_reference_view(&record)?))
    }

    pub fn read_current(
        &self,
        publication_key: &ReferencePublicationKey,
    ) -> ReferenceStoreResult<Option<PublishedReferenceView>> {
        let Some(entry) = self
            .store
            .catalog_entry(&current_catalog()?, publication_key.catalog_path())?
        else {
            return Ok(None);
        };
        let object_id = entry.object_id().clone();
        let view = self.read_exact(&object_id)?.ok_or_else(|| {
            ReferenceStoreError::new(
                ReferenceStoreErrorCode::StoreIntegrityViolation,
                "published reference object is missing",
            )
        })?;
        Ok(Some(PublishedReferenceView {
            schema: REFERENCE_STORE_SCHEMA,
            publication_key: publication_key.clone(),
            object_id,
            view,
        }))
    }

    pub fn retain(
        &mut self,
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> ReferenceStoreResult<LeaseRecord> {
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
    ) -> ReferenceStoreResult<LeaseRecord> {
        Ok(self
            .store
            .renew_lease(lease_id, holder, expected_expiry, new_expiry)?)
    }

    pub fn release_retention(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
    ) -> ReferenceStoreResult<()> {
        Ok(self.store.release_lease(lease_id, holder)?)
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> ReferenceStoreResult<GarbageCollectionReceipt> {
        Ok(self.store.collect_garbage(now, max_deletes)?)
    }

    pub fn validate_integrity(
        &self,
        max_objects: u32,
    ) -> ReferenceStoreResult<IntegrityReport> {
        Ok(self.store.validate_integrity(max_objects)?)
    }

    pub fn logical_manifest(&self) -> ReferenceStoreResult<LogicalManifest> {
        Ok(self.store.logical_manifest()?)
    }

    fn pending_view(&self, view: &ReferenceView) -> ReferenceStoreResult<PendingObject> {
        Ok(PendingObject::from_json(
            REFERENCE_VIEW_OBJECT_KIND,
            REFERENCE_VIEW_OBJECT_SCHEMA_VERSION,
            view,
            self.store.configuration().limits(),
        )?)
    }
}

fn current_catalog() -> ReferenceStoreResult<CatalogName> {
    Ok(CatalogName::new(CURRENT_CATALOG)?)
}

fn decode_reference_view(record: &ObjectRecord) -> ReferenceStoreResult<ReferenceView> {
    if record.kind() != REFERENCE_VIEW_OBJECT_KIND {
        return Err(ReferenceStoreError::new(
            ReferenceStoreErrorCode::ArtifactKindMismatch,
            "stored object is not a reference view",
        ));
    }
    if record.schema_version() != REFERENCE_VIEW_OBJECT_SCHEMA_VERSION {
        return Err(ReferenceStoreError::new(
            ReferenceStoreErrorCode::ArtifactSchemaMismatch,
            "stored reference view schema is unsupported",
        ));
    }
    record.decode().map_err(|_| {
        ReferenceStoreError::new(
            ReferenceStoreErrorCode::ArtifactDecodeFailed,
            "stored reference view failed strict validation",
        )
    })
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':' | b'@')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wow_store::{StoreConfiguration, StoreLimits};

    fn assert_wire_codec<T: Serialize + DeserializeOwned>() {}

    #[test]
    fn reference_view_wire_type_is_strictly_persistable() {
        assert_wire_codec::<ReferenceView>();
    }

    #[test]
    fn publication_key_is_explicit_and_path_safe() -> Result<(), Box<dyn std::error::Error>> {
        let key = ReferencePublicationKey::new("retail-12.1", "stable")?;
        assert_eq!(key.profile(), "retail-12.1");
        assert_eq!(key.channel(), "stable");
        assert_eq!(key.catalog_path().as_str(), "retail-12.1/stable");
        assert!(ReferencePublicationKey::new("../retail", "stable").is_err());
        assert!(ReferencePublicationKey::new("retail", "").is_err());
        Ok(())
    }

    #[test]
    fn wrong_object_kind_never_decodes_as_reference_view() -> Result<(), Box<dyn std::error::Error>> {
        let configuration = StoreConfiguration::new("reference-test", StoreLimits::default())?;
        let mut store = Store::open_in_memory(configuration)?;
        let pending = PendingObject::from_json(
            "fixture.not-reference",
            1,
            &json!({"value":1}),
            StoreLimits::default(),
        )?;
        let object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        store.commit(batch)?;
        let facade = PersistentReferenceStore::new(&mut store);
        let error = facade
            .read_exact(&object_id)
            .err()
            .ok_or("expected kind mismatch")?;
        assert_eq!(error.code(), ReferenceStoreErrorCode::ArtifactKindMismatch);
        Ok(())
    }
}
