//! E1-D transport-neutral ReferenceView validation and durable publication.

use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_reference::{
    ReferenceView,
    persistent::{
        PersistentReferenceStore, ReferencePublicationKey, ReferenceStoreError,
        ReferenceStoreErrorCode,
    },
    publication::PreparedReferencePublication,
};
use wow_store::{
    CatalogExpectation, LogicalEpoch, ObjectId, OperationBegin, OperationId, OperationState,
    PendingObject, RequestDigest, Store, StoreConfiguration, StoreError, StoreErrorCode,
    StoreLimits, WriteBatch,
};

pub const REFERENCE_ADMIN_SCHEMA: &str = "wow-service/reference-admin/e1-d/1";
const PUBLISH_RESULT_KIND: &str = "wow.service.reference_publish_result";
const PUBLISH_RESULT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAdminErrorCode {
    ConfigurationInvalid,
    InputTooLarge,
    ReferenceViewInvalid,
    PublicationConflict,
    OperationConflict,
    OperationIncomplete,
    OutcomeUnknown,
    ResultInvalid,
    StoreFailure,
    ReferenceOwnerFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceAdminError {
    code: ReferenceAdminErrorCode,
    lower_code: Option<Box<str>>,
    message: Box<str>,
}

impl ReferenceAdminError {
    fn new(code: ReferenceAdminErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            lower_code: None,
            message: message.into(),
        }
    }

    fn lower(
        code: ReferenceAdminErrorCode,
        lower_code: impl Into<Box<str>>,
        message: impl Into<Box<str>>,
    ) -> Self {
        Self {
            code,
            lower_code: Some(lower_code.into()),
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ReferenceAdminErrorCode {
        self.code
    }

    #[must_use]
    pub fn lower_code(&self) -> Option<&str> {
        self.lower_code.as_deref()
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ReferenceAdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ReferenceAdminError {}

impl From<StoreError> for ReferenceAdminError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::CatalogConflict => ReferenceAdminErrorCode::PublicationConflict,
            StoreErrorCode::OperationConflict => ReferenceAdminErrorCode::OperationConflict,
            StoreErrorCode::OperationStateInvalid => ReferenceAdminErrorCode::OperationIncomplete,
            _ => ReferenceAdminErrorCode::StoreFailure,
        };
        Self::lower(code, format!("{:?}", source.code()), source.message())
    }
}

impl From<ReferenceStoreError> for ReferenceAdminError {
    fn from(source: ReferenceStoreError) -> Self {
        let code = match source.code() {
            ReferenceStoreErrorCode::StoreCatalogConflict => {
                ReferenceAdminErrorCode::PublicationConflict
            }
            ReferenceStoreErrorCode::ArtifactDecodeFailed
            | ReferenceStoreErrorCode::ArtifactKindMismatch
            | ReferenceStoreErrorCode::ArtifactSchemaMismatch => {
                ReferenceAdminErrorCode::ReferenceViewInvalid
            }
            _ => ReferenceAdminErrorCode::ReferenceOwnerFailure,
        };
        Self::lower(code, format!("{:?}", source.code()), source.message())
    }
}

pub type ReferenceAdminResult<T> = Result<T, ReferenceAdminError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceAdminConfiguration {
    schema: &'static str,
    store_configuration: StoreConfiguration,
    max_input_bytes: u64,
    configuration_id: Box<str>,
}

impl ReferenceAdminConfiguration {
    pub fn new(
        store_profile_id: impl Into<Box<str>>,
        store_limits: StoreLimits,
        max_input_bytes: u64,
    ) -> ReferenceAdminResult<Self> {
        if max_input_bytes == 0 || max_input_bytes > 64 * 1024 * 1024 {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::ConfigurationInvalid,
                "reference input byte limit is outside the reviewed profile",
            ));
        }
        let store_configuration = StoreConfiguration::new(store_profile_id, store_limits)?;
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            store_configuration_id: &'a str,
            max_input_bytes: u64,
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: REFERENCE_ADMIN_SCHEMA,
            store_configuration_id: store_configuration.configuration_id(),
            max_input_bytes,
        })
        .map_err(|_| {
            ReferenceAdminError::new(
                ReferenceAdminErrorCode::ConfigurationInvalid,
                "reference administration configuration cannot be canonicalized",
            )
        })?;
        Ok(Self {
            schema: REFERENCE_ADMIN_SCHEMA,
            store_configuration,
            max_input_bytes,
            configuration_id: format!(
                "reference-admin-configuration:sha256:{}",
                hex(&Sha256::digest(bytes))
            )
            .into(),
        })
    }

    #[must_use]
    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    #[must_use]
    pub const fn max_input_bytes(&self) -> u64 {
        self.max_input_bytes
    }

    #[must_use]
    pub fn store_configuration(&self) -> &StoreConfiguration {
        &self.store_configuration
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishReferenceRequest {
    operation_id: OperationId,
    publication_key: ReferencePublicationKey,
    expectation: CatalogExpectation,
    reference_view_json: Box<[u8]>,
}

impl PublishReferenceRequest {
    #[must_use]
    pub fn new(
        operation_id: OperationId,
        publication_key: ReferencePublicationKey,
        expectation: CatalogExpectation,
        reference_view_json: impl Into<Box<[u8]>>,
    ) -> Self {
        Self {
            operation_id,
            publication_key,
            expectation,
            reference_view_json: reference_view_json.into(),
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePublishResult {
    schema: Box<str>,
    result_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    publication_profile: Box<str>,
    publication_channel: Box<str>,
    reference_object_id: ObjectId,
    reference_view_sha256: Box<str>,
}

impl ReferencePublishResult {
    fn build(
        configuration_id: &str,
        operation_id: OperationId,
        request_digest: RequestDigest,
        publication_key: &ReferencePublicationKey,
        reference_object_id: ObjectId,
        reference_view_sha256: Box<str>,
    ) -> ReferenceAdminResult<Self> {
        let result_id = publish_result_id(
            configuration_id,
            &operation_id,
            &request_digest,
            publication_key,
            &reference_object_id,
            &reference_view_sha256,
        )?;
        Ok(Self {
            schema: REFERENCE_ADMIN_SCHEMA.into(),
            result_id,
            configuration_id: configuration_id.into(),
            operation_id,
            request_digest,
            publication_profile: publication_key.profile().into(),
            publication_channel: publication_key.channel().into(),
            reference_object_id,
            reference_view_sha256,
        })
    }

    fn validate(&self, configuration_id: &str) -> ReferenceAdminResult<()> {
        if self.schema.as_ref() != REFERENCE_ADMIN_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
        {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored reference publish result has incompatible configuration",
            ));
        }
        let key = ReferencePublicationKey::new(
            self.publication_profile.clone(),
            self.publication_channel.clone(),
        )?;
        let expected = publish_result_id(
            &self.configuration_id,
            &self.operation_id,
            &self.request_digest,
            &key,
            &self.reference_object_id,
            &self.reference_view_sha256,
        )?;
        if self.result_id != expected {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored reference publish result identity does not match",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
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
    pub fn reference_object_id(&self) -> &ObjectId {
        &self.reference_object_id
    }

    #[must_use]
    pub fn reference_view_sha256(&self) -> &str {
        &self.reference_view_sha256
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceAdminStatus {
    schema: &'static str,
    configuration_id: Box<str>,
    publication_profile: Box<str>,
    publication_channel: Box<str>,
    current_reference_object_id: Option<ObjectId>,
    logical_manifest_id: Box<str>,
    integrity_complete: bool,
}

impl ReferenceAdminStatus {
    #[must_use]
    pub fn current_reference_object_id(&self) -> Option<&ObjectId> {
        self.current_reference_object_id.as_ref()
    }

    #[must_use]
    pub fn logical_manifest_id(&self) -> &str {
        &self.logical_manifest_id
    }

    #[must_use]
    pub const fn integrity_complete(&self) -> bool {
        self.integrity_complete
    }
}

pub struct ReferenceAdminService {
    configuration: ReferenceAdminConfiguration,
    store: Store,
}

impl ReferenceAdminService {
    pub fn open(
        path: impl AsRef<Path>,
        configuration: ReferenceAdminConfiguration,
    ) -> ReferenceAdminResult<Self> {
        let store = Store::open(path, configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    pub fn open_in_memory(
        configuration: ReferenceAdminConfiguration,
    ) -> ReferenceAdminResult<Self> {
        let store = Store::open_in_memory(configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &ReferenceAdminConfiguration {
        &self.configuration
    }

    pub fn validate_view_bytes(&self, bytes: &[u8]) -> ReferenceAdminResult<Box<str>> {
        let (_, canonical) = self.parse_view(bytes)?;
        Ok(format!("sha256:{}", hex(&Sha256::digest(canonical))).into())
    }

    pub fn publish(
        &mut self,
        request: PublishReferenceRequest,
    ) -> ReferenceAdminResult<ReferencePublishResult> {
        let (view, canonical) = self.parse_view(&request.reference_view_json)?;
        let view_sha256 =
            format!("sha256:{}", hex(&Sha256::digest(&canonical))).into_boxed_str();
        let prepared = PreparedReferencePublication::new(
            request.publication_key.clone(),
            &view,
            request.expectation.clone(),
            self.configuration.store_configuration().limits(),
        )?;
        let request_digest = request_digest(
            self.configuration.configuration_id(),
            &request.publication_key,
            &request.expectation,
            prepared.object_id(),
            &view_sha256,
        )?;
        match self
            .store
            .begin_operation(request.operation_id.clone(), request_digest.clone())?
        {
            OperationBegin::Started(_) => self.publish_or_reconcile(
                request.operation_id,
                request_digest,
                prepared,
                view_sha256,
                false,
            ),
            OperationBegin::Replay(record) => match record.state() {
                OperationState::Completed => {
                    let result_object_id = record.result_object_id().ok_or_else(|| {
                        ReferenceAdminError::new(
                            ReferenceAdminErrorCode::ResultInvalid,
                            "completed operation is missing its result object",
                        )
                    })?;
                    self.read_publish_result(result_object_id, &request_digest)
                }
                OperationState::Prepared => self.publish_or_reconcile(
                    request.operation_id,
                    request_digest,
                    prepared,
                    view_sha256,
                    true,
                ),
                OperationState::OutcomeUnknown => Err(ReferenceAdminError::new(
                    ReferenceAdminErrorCode::OutcomeUnknown,
                    "reference publication outcome requires external reconciliation",
                )),
                OperationState::NoEffect | OperationState::Failed => {
                    Err(ReferenceAdminError::new(
                        ReferenceAdminErrorCode::OperationIncomplete,
                        "reference publication operation is already terminal without success",
                    ))
                }
            },
        }
    }

    pub fn status(
        &mut self,
        publication_key: &ReferencePublicationKey,
        integrity_budget: u32,
    ) -> ReferenceAdminResult<ReferenceAdminStatus> {
        let facade = PersistentReferenceStore::new(&mut self.store);
        let current = facade.read_current(publication_key)?;
        let integrity = facade.validate_integrity(integrity_budget)?;
        let manifest = facade.logical_manifest()?;
        Ok(ReferenceAdminStatus {
            schema: REFERENCE_ADMIN_SCHEMA,
            configuration_id: self.configuration.configuration_id().into(),
            publication_profile: publication_key.profile().into(),
            publication_channel: publication_key.channel().into(),
            current_reference_object_id: current.map(|item| item.object_id().clone()),
            logical_manifest_id: manifest.manifest_id().into(),
            integrity_complete: integrity.complete(),
        })
    }

    pub fn read_exact(
        &mut self,
        object_id: &ObjectId,
    ) -> ReferenceAdminResult<Option<ReferenceView>> {
        Ok(PersistentReferenceStore::new(&mut self.store).read_exact(object_id)?)
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> ReferenceAdminResult<Vec<ObjectId>> {
        Ok(PersistentReferenceStore::new(&mut self.store)
            .collect_garbage(now, max_deletes)?
            .deleted()
            .to_vec())
    }

    fn publish_or_reconcile(
        &mut self,
        operation_id: OperationId,
        request_digest: RequestDigest,
        prepared: PreparedReferencePublication,
        view_sha256: Box<str>,
        replaying_prepared: bool,
    ) -> ReferenceAdminResult<ReferencePublishResult> {
        let key = prepared.publication_key().clone();
        let target = prepared.object_id().clone();
        let current = PersistentReferenceStore::new(&mut self.store)
            .read_current(&key)?
            .map(|item| item.object_id().clone());
        if current.as_ref() != Some(&target) {
            if !prepared.expectation_matches(current.as_ref()) {
                if replaying_prepared {
                    self.store
                        .mark_outcome_unknown(&operation_id, &request_digest)?;
                    return Err(ReferenceAdminError::new(
                        ReferenceAdminErrorCode::OutcomeUnknown,
                        "prepared publication cannot be safely replayed after current changed",
                    ));
                }
                self.store.record_failed(&operation_id, &request_digest)?;
                return Err(ReferenceAdminError::new(
                    ReferenceAdminErrorCode::PublicationConflict,
                    "reference publication compare-and-swap guard does not match",
                ));
            }
            if let Err(error) = self.store.commit(prepared.into_batch()?) {
                self.store.record_failed(&operation_id, &request_digest)?;
                return Err(error.into());
            }
        }
        let result = ReferencePublishResult::build(
            self.configuration.configuration_id(),
            operation_id.clone(),
            request_digest.clone(),
            &key,
            target,
            view_sha256,
        )?;
        let pending = PendingObject::from_json(
            PUBLISH_RESULT_KIND,
            PUBLISH_RESULT_SCHEMA_VERSION,
            &result,
            self.configuration.store_configuration().limits(),
        )?;
        let result_object_id = pending.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending)?;
        self.store.commit(batch)?;
        self.store
            .complete_operation(&operation_id, &request_digest, &result_object_id)?;
        Ok(result)
    }

    fn read_publish_result(
        &self,
        object_id: &ObjectId,
        expected_request_digest: &RequestDigest,
    ) -> ReferenceAdminResult<ReferencePublishResult> {
        let record = self.store.object(object_id)?.ok_or_else(|| {
            ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored reference publish result object is missing",
            )
        })?;
        if record.kind() != PUBLISH_RESULT_KIND
            || record.schema_version() != PUBLISH_RESULT_SCHEMA_VERSION
        {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored operation result has an incompatible type",
            ));
        }
        let result: ReferencePublishResult = record.decode().map_err(|_| {
            ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored reference publish result cannot be decoded",
            )
        })?;
        result.validate(self.configuration.configuration_id())?;
        if result.request_digest() != expected_request_digest {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::ResultInvalid,
                "stored reference publish result belongs to another request",
            ));
        }
        Ok(result)
    }

    fn parse_view(&self, bytes: &[u8]) -> ReferenceAdminResult<(ReferenceView, Vec<u8>)> {
        if bytes.is_empty() || bytes.len() as u64 > self.configuration.max_input_bytes() {
            return Err(ReferenceAdminError::new(
                ReferenceAdminErrorCode::InputTooLarge,
                "reference view input is empty or exceeds the configured byte budget",
            ));
        }
        let view: ReferenceView = serde_json::from_slice(bytes).map_err(|_| {
            ReferenceAdminError::new(
                ReferenceAdminErrorCode::ReferenceViewInvalid,
                "reference view failed strict deserialization",
            )
        })?;
        let canonical = canonical_json_bytes(&view).map_err(|_| {
            ReferenceAdminError::new(
                ReferenceAdminErrorCode::ReferenceViewInvalid,
                "reference view cannot be canonicalized",
            )
        })?;
        Ok((view, canonical))
    }
}

fn request_digest(
    configuration_id: &str,
    publication_key: &ReferencePublicationKey,
    expectation: &CatalogExpectation,
    target: &ObjectId,
    reference_view_sha256: &str,
) -> ReferenceAdminResult<RequestDigest> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        publication_key: &'a ReferencePublicationKey,
        expectation: &'a CatalogExpectation,
        target: &'a ObjectId,
        reference_view_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: REFERENCE_ADMIN_SCHEMA,
        configuration_id,
        publication_key,
        expectation,
        target,
        reference_view_sha256,
    })
    .map_err(|_| {
        ReferenceAdminError::new(
            ReferenceAdminErrorCode::ConfigurationInvalid,
            "reference publish request cannot be canonicalized",
        )
    })?;
    Ok(RequestDigest::new(format!(
        "sha256:{}",
        hex(&Sha256::digest(bytes))
    ))?)
}

fn publish_result_id(
    configuration_id: &str,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    publication_key: &ReferencePublicationKey,
    reference_object_id: &ObjectId,
    reference_view_sha256: &str,
) -> ReferenceAdminResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        publication_key: &'a ReferencePublicationKey,
        reference_object_id: &'a ObjectId,
        reference_view_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: REFERENCE_ADMIN_SCHEMA,
        configuration_id,
        operation_id,
        request_digest,
        publication_key,
        reference_object_id,
        reference_view_sha256,
    })
    .map_err(|_| {
        ReferenceAdminError::new(
            ReferenceAdminErrorCode::ResultInvalid,
            "reference publish result identity cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "reference-publish-result:sha256:{}",
        hex(&Sha256::digest(bytes))
    )
    .into())
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
