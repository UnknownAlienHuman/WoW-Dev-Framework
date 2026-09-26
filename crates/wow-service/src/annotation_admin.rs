//! E1-C annotation artifact publication over the generic durable store.
//!
//! `wow-annotations` owns canonical artifact semantics and identity. This module
//! owns durable object/catalog effects, exact operation replay, read-back,
//! retention and garbage collection without exposing a mutable store handle.

use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_annotations::artifact::{
    AnnotationArtifact, AnnotationArtifactError, AnnotationArtifactErrorCode,
    AnnotationPublicationKey,
};
use wow_core::canonical_json_bytes;
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, LeaseId, LeaseRecord,
    LogicalEpoch, ObjectId, ObjectRecord, OperationBegin, OperationId, OperationState,
    PendingObject, RequestDigest, Store, StoreConfiguration, StoreError, StoreErrorCode,
    StoreLimits, WriteBatch,
};

pub const ANNOTATION_ADMIN_SCHEMA: &str = "wow-service/annotation-admin/e1-c/1";
pub const ANNOTATION_STORE_SCHEMA: &str = "wow-annotations/store/e1-c/1";
pub const ANNOTATION_OBJECT_KIND: &str = "wow.annotation.artifact";
pub const ANNOTATION_OBJECT_SCHEMA_VERSION: u32 = 1;
const CURRENT_CATALOG: &str = "annotation.current";
const PUBLISH_RESULT_KIND: &str = "wow.service.annotation_publish_result";
const PUBLISH_RESULT_SCHEMA_VERSION: u32 = 1;

pub type AnnotationAdminExpectation = CatalogExpectation;
pub type AnnotationAdminOperationId = OperationId;
pub type AnnotationAdminStoreLimits = StoreLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationAdminErrorCode {
    ConfigurationInvalid,
    ArtifactInvalid,
    PublicationKeyInvalid,
    PublicationConflict,
    OperationConflict,
    OperationIncomplete,
    OutcomeUnknown,
    ResultInvalid,
    StoreFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnnotationAdminError {
    code: AnnotationAdminErrorCode,
    lower_code: Option<Box<str>>,
    message: Box<str>,
}

impl AnnotationAdminError {
    fn new(code: AnnotationAdminErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            lower_code: None,
            message: message.into(),
        }
    }

    fn lower(
        code: AnnotationAdminErrorCode,
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
    pub const fn code(&self) -> AnnotationAdminErrorCode {
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

impl fmt::Display for AnnotationAdminError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AnnotationAdminError {}

impl From<StoreError> for AnnotationAdminError {
    fn from(source: StoreError) -> Self {
        let code = match source.code() {
            StoreErrorCode::CatalogConflict => AnnotationAdminErrorCode::PublicationConflict,
            StoreErrorCode::OperationConflict => AnnotationAdminErrorCode::OperationConflict,
            StoreErrorCode::OperationStateInvalid => AnnotationAdminErrorCode::OperationIncomplete,
            StoreErrorCode::OutcomeUnknown => AnnotationAdminErrorCode::OutcomeUnknown,
            _ => AnnotationAdminErrorCode::StoreFailure,
        };
        Self::lower(code, format!("{:?}", source.code()), source.message())
    }
}

impl From<AnnotationArtifactError> for AnnotationAdminError {
    fn from(source: AnnotationArtifactError) -> Self {
        let code = match source.code() {
            AnnotationArtifactErrorCode::PublicationKeyInvalid => {
                AnnotationAdminErrorCode::PublicationKeyInvalid
            }
            _ => AnnotationAdminErrorCode::ArtifactInvalid,
        };
        Self::lower(code, format!("{:?}", source.code()), source.message())
    }
}

pub type AnnotationAdminResult<T> = Result<T, AnnotationAdminError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationAdminConfiguration {
    schema: &'static str,
    store_configuration: StoreConfiguration,
    configuration_id: Box<str>,
}

impl AnnotationAdminConfiguration {
    pub fn new(
        store_profile_id: impl Into<Box<str>>,
        store_limits: StoreLimits,
    ) -> AnnotationAdminResult<Self> {
        let store_configuration = StoreConfiguration::new(store_profile_id, store_limits)?;
        #[derive(Serialize)]
        struct Identity<'a> {
            schema: &'static str,
            store_configuration_id: &'a str,
        }
        let bytes = canonical_json_bytes(&Identity {
            schema: ANNOTATION_ADMIN_SCHEMA,
            store_configuration_id: store_configuration.configuration_id(),
        })
        .map_err(|_| {
            AnnotationAdminError::new(
                AnnotationAdminErrorCode::ConfigurationInvalid,
                "annotation administration configuration cannot be canonicalized",
            )
        })?;
        Ok(Self {
            schema: ANNOTATION_ADMIN_SCHEMA,
            store_configuration,
            configuration_id: format!(
                "annotation-admin-configuration:sha256:{}",
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
    pub fn store_configuration(&self) -> &StoreConfiguration {
        &self.store_configuration
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishAnnotationRequest {
    operation_id: OperationId,
    publication_key: AnnotationPublicationKey,
    expectation: CatalogExpectation,
    artifact: AnnotationArtifact,
}

impl PublishAnnotationRequest {
    #[must_use]
    pub fn new(
        operation_id: OperationId,
        publication_key: AnnotationPublicationKey,
        expectation: CatalogExpectation,
        artifact: AnnotationArtifact,
    ) -> Self {
        Self {
            operation_id,
            publication_key,
            expectation,
            artifact,
        }
    }

    #[must_use]
    pub fn operation_id(&self) -> &OperationId {
        &self.operation_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationPublishResult {
    schema: Box<str>,
    result_id: Box<str>,
    configuration_id: Box<str>,
    operation_id: OperationId,
    request_digest: RequestDigest,
    publication_profile: Box<str>,
    publication_environment: Box<str>,
    artifact_family: Box<str>,
    annotation_object_id: ObjectId,
    annotation_artifact_id: Box<str>,
    payload_sha256: Box<str>,
}

impl AnnotationPublishResult {
    fn build(
        configuration_id: &str,
        operation_id: OperationId,
        request_digest: RequestDigest,
        publication_key: &AnnotationPublicationKey,
        annotation_object_id: ObjectId,
        artifact: &AnnotationArtifact,
    ) -> AnnotationAdminResult<Self> {
        let result_id = publish_result_id(
            configuration_id,
            &operation_id,
            &request_digest,
            publication_key,
            &annotation_object_id,
            artifact.artifact_id(),
            artifact.payload_sha256(),
        )?;
        Ok(Self {
            schema: ANNOTATION_ADMIN_SCHEMA.into(),
            result_id,
            configuration_id: configuration_id.into(),
            operation_id,
            request_digest,
            publication_profile: publication_key.profile().into(),
            publication_environment: publication_key.environment().into(),
            artifact_family: publication_key.artifact_family().into(),
            annotation_object_id,
            annotation_artifact_id: artifact.artifact_id().into(),
            payload_sha256: artifact.payload_sha256().into(),
        })
    }

    fn validate(&self, configuration_id: &str) -> AnnotationAdminResult<()> {
        if self.schema.as_ref() != ANNOTATION_ADMIN_SCHEMA
            || self.configuration_id.as_ref() != configuration_id
        {
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation publish result has incompatible configuration",
            ));
        }
        let key = AnnotationPublicationKey::new(
            self.publication_profile.clone(),
            self.publication_environment.clone(),
            self.artifact_family.clone(),
        )?;
        let expected = publish_result_id(
            &self.configuration_id,
            &self.operation_id,
            &self.request_digest,
            &key,
            &self.annotation_object_id,
            &self.annotation_artifact_id,
            &self.payload_sha256,
        )?;
        if self.result_id != expected {
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation publish result identity does not match",
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
    pub fn annotation_object_id(&self) -> &ObjectId {
        &self.annotation_object_id
    }

    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub fn payload_sha256(&self) -> &str {
        &self.payload_sha256
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationAdminStatus {
    schema: &'static str,
    configuration_id: Box<str>,
    publication_profile: Box<str>,
    publication_environment: Box<str>,
    artifact_family: Box<str>,
    current_annotation_object_id: Option<ObjectId>,
    logical_manifest_id: Box<str>,
    integrity_complete: bool,
}

impl AnnotationAdminStatus {
    #[must_use]
    pub fn current_annotation_object_id(&self) -> Option<&ObjectId> {
        self.current_annotation_object_id.as_ref()
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

pub struct AnnotationAdminService {
    configuration: AnnotationAdminConfiguration,
    store: Store,
}

impl AnnotationAdminService {
    pub fn open(
        path: impl AsRef<Path>,
        configuration: AnnotationAdminConfiguration,
    ) -> AnnotationAdminResult<Self> {
        let store = Store::open(path, configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    pub fn open_in_memory(
        configuration: AnnotationAdminConfiguration,
    ) -> AnnotationAdminResult<Self> {
        let store = Store::open_in_memory(configuration.store_configuration().clone())?;
        Ok(Self {
            configuration,
            store,
        })
    }

    #[must_use]
    pub fn configuration(&self) -> &AnnotationAdminConfiguration {
        &self.configuration
    }

    pub fn publish(
        &mut self,
        request: PublishAnnotationRequest,
    ) -> AnnotationAdminResult<AnnotationPublishResult> {
        request.artifact.validate()?;
        if request.publication_key.profile() != request.artifact.profile_id() {
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::PublicationKeyInvalid,
                "annotation publication profile differs from the artifact profile",
            ));
        }
        let pending = pending_artifact(
            &request.artifact,
            self.configuration.store_configuration().limits(),
        )?;
        let target = pending.object_id().clone();
        let request_digest = request_digest(
            self.configuration.configuration_id(),
            &request.publication_key,
            &request.expectation,
            &target,
            request.artifact.artifact_id(),
            request.artifact.payload_sha256(),
        )?;
        match self
            .store
            .begin_operation(request.operation_id.clone(), request_digest.clone())?
        {
            OperationBegin::Started(_) => self.publish_or_reconcile(
                request.operation_id,
                request_digest,
                request.publication_key,
                request.expectation,
                request.artifact,
                pending,
                false,
            ),
            OperationBegin::Replay(record) => match record.state() {
                OperationState::Completed => {
                    let result_object_id = record.result_object_id().ok_or_else(|| {
                        AnnotationAdminError::new(
                            AnnotationAdminErrorCode::ResultInvalid,
                            "completed annotation operation is missing its result object",
                        )
                    })?;
                    self.read_publish_result(result_object_id, &request_digest)
                }
                OperationState::Prepared => self.publish_or_reconcile(
                    request.operation_id,
                    request_digest,
                    request.publication_key,
                    request.expectation,
                    request.artifact,
                    pending,
                    true,
                ),
                OperationState::OutcomeUnknown => Err(AnnotationAdminError::new(
                    AnnotationAdminErrorCode::OutcomeUnknown,
                    "annotation publication outcome requires explicit reconciliation",
                )),
                OperationState::NoEffect | OperationState::Failed => {
                    Err(AnnotationAdminError::new(
                        AnnotationAdminErrorCode::OperationIncomplete,
                        "annotation publication operation is terminal without success",
                    ))
                }
            },
        }
    }

    pub fn read_exact(
        &self,
        object_id: &ObjectId,
    ) -> AnnotationAdminResult<Option<AnnotationArtifact>> {
        read_exact(&self.store, object_id)
    }

    pub fn read_current(
        &self,
        publication_key: &AnnotationPublicationKey,
    ) -> AnnotationAdminResult<Option<PublishedAnnotationArtifact>> {
        read_current(&self.store, publication_key)
    }

    pub fn status(
        &self,
        publication_key: &AnnotationPublicationKey,
        integrity_budget: u32,
    ) -> AnnotationAdminResult<AnnotationAdminStatus> {
        let current = self.read_current(publication_key)?;
        let integrity = self.store.validate_integrity(integrity_budget)?;
        let manifest = self.store.logical_manifest()?;
        Ok(AnnotationAdminStatus {
            schema: ANNOTATION_ADMIN_SCHEMA,
            configuration_id: self.configuration.configuration_id().into(),
            publication_profile: publication_key.profile().into(),
            publication_environment: publication_key.environment().into(),
            artifact_family: publication_key.artifact_family().into(),
            current_annotation_object_id: current.map(|item| item.object_id().clone()),
            logical_manifest_id: manifest.manifest_id().into(),
            integrity_complete: integrity.complete(),
        })
    }

    pub fn retain(
        &mut self,
        lease_id: LeaseId,
        object_id: ObjectId,
        holder: impl Into<Box<str>>,
        expires_after: LogicalEpoch,
    ) -> AnnotationAdminResult<LeaseRecord> {
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
    ) -> AnnotationAdminResult<LeaseRecord> {
        Ok(self
            .store
            .renew_lease(lease_id, holder, expected_expiry, new_expiry)?)
    }

    pub fn release_retention(
        &mut self,
        lease_id: &LeaseId,
        holder: &str,
    ) -> AnnotationAdminResult<()> {
        Ok(self.store.release_lease(lease_id, holder)?)
    }

    pub fn collect_garbage(
        &mut self,
        now: LogicalEpoch,
        max_deletes: u32,
    ) -> AnnotationAdminResult<Vec<ObjectId>> {
        Ok(self
            .store
            .collect_garbage(now, max_deletes)?
            .deleted()
            .to_vec())
    }

    #[allow(clippy::too_many_arguments)]
    fn publish_or_reconcile(
        &mut self,
        operation_id: OperationId,
        request_digest: RequestDigest,
        publication_key: AnnotationPublicationKey,
        expectation: CatalogExpectation,
        artifact: AnnotationArtifact,
        pending: PendingObject,
        replaying_prepared: bool,
    ) -> AnnotationAdminResult<AnnotationPublishResult> {
        let target = pending.object_id().clone();
        let current = current_object_id(&self.store, &publication_key)?;
        if current.as_ref() != Some(&target) {
            if !expectation_matches(&expectation, current.as_ref()) {
                if replaying_prepared {
                    self.store
                        .mark_outcome_unknown(&operation_id, &request_digest)?;
                    return Err(AnnotationAdminError::new(
                        AnnotationAdminErrorCode::OutcomeUnknown,
                        "prepared annotation publication cannot be replayed after current changed",
                    ));
                }
                self.store.record_failed(&operation_id, &request_digest)?;
                return Err(AnnotationAdminError::new(
                    AnnotationAdminErrorCode::PublicationConflict,
                    "annotation publication compare-and-swap guard does not match",
                ));
            }
            let mut batch = WriteBatch::new();
            batch.add_object(pending)?;
            batch.add_catalog_mutation(CatalogMutation::set(
                current_catalog()?,
                catalog_path(&publication_key)?,
                expectation,
                target.clone(),
            ))?;
            if let Err(error) = self.store.commit(batch) {
                if error.code() == StoreErrorCode::OutcomeUnknown {
                    self.store
                        .mark_outcome_unknown(&operation_id, &request_digest)?;
                } else {
                    self.store.record_failed(&operation_id, &request_digest)?;
                }
                return Err(error.into());
            }
        }

        let persisted = self.read_exact(&target)?.ok_or_else(|| {
            AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "published annotation object is missing during read-back",
            )
        });
        let persisted = match persisted {
            Ok(value) if value == artifact => value,
            Ok(_) | Err(_) => {
                self.store
                    .mark_outcome_unknown(&operation_id, &request_digest)?;
                return Err(AnnotationAdminError::new(
                    AnnotationAdminErrorCode::OutcomeUnknown,
                    "annotation publication read-back did not match the requested artifact",
                ));
            }
        };
        if current_object_id(&self.store, &publication_key)?.as_ref() != Some(&target) {
            self.store
                .mark_outcome_unknown(&operation_id, &request_digest)?;
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::OutcomeUnknown,
                "annotation publication current pointer failed read-back",
            ));
        }

        let result = AnnotationPublishResult::build(
            self.configuration.configuration_id(),
            operation_id.clone(),
            request_digest.clone(),
            &publication_key,
            target,
            &persisted,
        )?;
        let pending_result = PendingObject::from_json(
            PUBLISH_RESULT_KIND,
            PUBLISH_RESULT_SCHEMA_VERSION,
            &result,
            self.configuration.store_configuration().limits(),
        )?;
        let result_object_id = pending_result.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(pending_result)?;
        if self.store.commit(batch).is_err() {
            self.store
                .mark_outcome_unknown(&operation_id, &request_digest)?;
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::OutcomeUnknown,
                "annotation publication succeeded but its result receipt was not committed",
            ));
        }
        if self
            .store
            .complete_operation(&operation_id, &request_digest, &result_object_id)
            .is_err()
        {
            self.store
                .mark_outcome_unknown(&operation_id, &request_digest)?;
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::OutcomeUnknown,
                "annotation publication result could not be bound to its operation",
            ));
        }
        Ok(result)
    }

    fn read_publish_result(
        &self,
        object_id: &ObjectId,
        expected_request_digest: &RequestDigest,
    ) -> AnnotationAdminResult<AnnotationPublishResult> {
        let record = self.store.object(object_id)?.ok_or_else(|| {
            AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation publish result object is missing",
            )
        })?;
        if record.kind() != PUBLISH_RESULT_KIND
            || record.schema_version() != PUBLISH_RESULT_SCHEMA_VERSION
        {
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation operation result has an incompatible type",
            ));
        }
        let result: AnnotationPublishResult = record.decode().map_err(|_| {
            AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation publish result cannot be decoded",
            )
        })?;
        result.validate(self.configuration.configuration_id())?;
        if result.request_digest() != expected_request_digest {
            return Err(AnnotationAdminError::new(
                AnnotationAdminErrorCode::ResultInvalid,
                "stored annotation publish result belongs to another request",
            ));
        }
        Ok(result)
    }
}

fn pending_artifact(
    artifact: &AnnotationArtifact,
    limits: StoreLimits,
) -> AnnotationAdminResult<PendingObject> {
    artifact.validate()?;
    Ok(PendingObject::from_json(
        ANNOTATION_OBJECT_KIND,
        ANNOTATION_OBJECT_SCHEMA_VERSION,
        artifact,
        limits,
    )?)
}

fn current_catalog() -> AnnotationAdminResult<CatalogName> {
    Ok(CatalogName::new(CURRENT_CATALOG)?)
}

fn catalog_path(publication_key: &AnnotationPublicationKey) -> AnnotationAdminResult<CatalogPath> {
    Ok(CatalogPath::new(publication_key.catalog_path())?)
}

fn current_object_id(
    store: &Store,
    publication_key: &AnnotationPublicationKey,
) -> AnnotationAdminResult<Option<ObjectId>> {
    Ok(store
        .catalog_entry(&current_catalog()?, &catalog_path(publication_key)?)?
        .map(|entry| entry.object_id().clone()))
}

fn read_exact(
    store: &Store,
    object_id: &ObjectId,
) -> AnnotationAdminResult<Option<AnnotationArtifact>> {
    let Some(record) = store.object(object_id)? else {
        return Ok(None);
    };
    Ok(Some(decode_annotation_artifact(&record)?))
}

fn read_current(
    store: &Store,
    publication_key: &AnnotationPublicationKey,
) -> AnnotationAdminResult<Option<PublishedAnnotationArtifact>> {
    let Some(object_id) = current_object_id(store, publication_key)? else {
        return Ok(None);
    };
    let artifact = read_exact(store, &object_id)?.ok_or_else(|| {
        AnnotationAdminError::new(
            AnnotationAdminErrorCode::ResultInvalid,
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

fn decode_annotation_artifact(record: &ObjectRecord) -> AnnotationAdminResult<AnnotationArtifact> {
    if record.kind() != ANNOTATION_OBJECT_KIND
        || record.schema_version() != ANNOTATION_OBJECT_SCHEMA_VERSION
    {
        return Err(AnnotationAdminError::new(
            AnnotationAdminErrorCode::ArtifactInvalid,
            "stored object is not a supported annotation artifact",
        ));
    }
    let artifact: AnnotationArtifact = record.decode().map_err(|_| {
        AnnotationAdminError::new(
            AnnotationAdminErrorCode::ArtifactInvalid,
            "stored annotation artifact cannot be decoded",
        )
    })?;
    artifact.validate()?;
    Ok(artifact)
}

fn expectation_matches(expectation: &CatalogExpectation, current: Option<&ObjectId>) -> bool {
    match expectation {
        CatalogExpectation::Absent => current.is_none(),
        CatalogExpectation::Exact(expected) => current == Some(expected),
    }
}

fn request_digest(
    configuration_id: &str,
    publication_key: &AnnotationPublicationKey,
    expectation: &CatalogExpectation,
    target: &ObjectId,
    annotation_artifact_id: &str,
    payload_sha256: &str,
) -> AnnotationAdminResult<RequestDigest> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        publication_key: &'a AnnotationPublicationKey,
        expectation: &'a CatalogExpectation,
        target: &'a ObjectId,
        annotation_artifact_id: &'a str,
        payload_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: ANNOTATION_ADMIN_SCHEMA,
        configuration_id,
        publication_key,
        expectation,
        target,
        annotation_artifact_id,
        payload_sha256,
    })
    .map_err(|_| {
        AnnotationAdminError::new(
            AnnotationAdminErrorCode::ConfigurationInvalid,
            "annotation publish request cannot be canonicalized",
        )
    })?;
    Ok(RequestDigest::new(format!(
        "sha256:{}",
        hex(&Sha256::digest(bytes))
    ))?)
}

#[allow(clippy::too_many_arguments)]
fn publish_result_id(
    configuration_id: &str,
    operation_id: &OperationId,
    request_digest: &RequestDigest,
    publication_key: &AnnotationPublicationKey,
    annotation_object_id: &ObjectId,
    annotation_artifact_id: &str,
    payload_sha256: &str,
) -> AnnotationAdminResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        configuration_id: &'a str,
        operation_id: &'a OperationId,
        request_digest: &'a RequestDigest,
        publication_key: &'a AnnotationPublicationKey,
        annotation_object_id: &'a ObjectId,
        annotation_artifact_id: &'a str,
        payload_sha256: &'a str,
    }
    let bytes = canonical_json_bytes(&Identity {
        schema: ANNOTATION_ADMIN_SCHEMA,
        configuration_id,
        operation_id,
        request_digest,
        publication_key,
        annotation_object_id,
        annotation_artifact_id,
        payload_sha256,
    })
    .map_err(|_| {
        AnnotationAdminError::new(
            AnnotationAdminErrorCode::ResultInvalid,
            "annotation publish result identity cannot be canonicalized",
        )
    })?;
    Ok(format!(
        "annotation-publish-result:sha256:{}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn configuration() -> AnnotationAdminResult<AnnotationAdminConfiguration> {
        AnnotationAdminConfiguration::new("annotation-test", StoreLimits::default())
    }

    fn artifact(value: serde_json::Value) -> AnnotationAdminResult<AnnotationArtifact> {
        Ok(AnnotationArtifact::from_serializable(
            "wow-annotations",
            "1.0.0",
            "retail-12.1",
            "reference-generation:fixture",
            &value,
        )?)
    }

    fn operation(value: &str) -> AnnotationAdminResult<OperationId> {
        Ok(OperationId::new(value)?)
    }

    #[test]
    fn exact_publish_read_cas_and_same_request_replay_are_service_owned()
    -> Result<(), Box<dyn std::error::Error>> {
        let mut service = AnnotationAdminService::open_in_memory(configuration()?)?;
        let key = AnnotationPublicationKey::new("retail-12.1", "mainline", "emmy-library")?;
        let first = artifact(json!({"files":[{"path":"A.lua"}]}))?;
        let request = PublishAnnotationRequest::new(
            operation("annotation-publish-1")?,
            key.clone(),
            CatalogExpectation::Absent,
            first.clone(),
        );
        let stored = service.publish(request.clone())?;
        let replay = service.publish(request)?;
        assert_eq!(stored, replay);
        let current = service
            .read_current(&key)?
            .ok_or("missing current artifact")?;
        assert_eq!(current.object_id(), stored.annotation_object_id());
        assert_eq!(current.artifact(), &first);

        let second = artifact(json!({"files":[{"path":"B.lua"}]}))?;
        let conflict = service
            .publish(PublishAnnotationRequest::new(
                operation("annotation-publish-2")?,
                key.clone(),
                CatalogExpectation::Absent,
                second.clone(),
            ))
            .err()
            .ok_or("expected catalog conflict")?;
        assert_eq!(
            conflict.code(),
            AnnotationAdminErrorCode::PublicationConflict
        );
        let replaced = service.publish(PublishAnnotationRequest::new(
            operation("annotation-publish-3")?,
            key.clone(),
            CatalogExpectation::Exact(stored.annotation_object_id().clone()),
            second.clone(),
        ))?;
        assert_eq!(
            service
                .read_current(&key)?
                .ok_or("missing replacement")?
                .object_id(),
            replaced.annotation_object_id()
        );
        assert_eq!(
            service
                .read_exact(replaced.annotation_object_id())?
                .ok_or("missing exact replacement")?,
            second
        );
        Ok(())
    }

    #[test]
    fn operation_id_cannot_be_rebound_to_another_request() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut service = AnnotationAdminService::open_in_memory(configuration()?)?;
        let key = AnnotationPublicationKey::new("retail-12.1", "mainline", "emmy-library")?;
        let operation_id = operation("annotation-rebind")?;
        service.publish(PublishAnnotationRequest::new(
            operation_id.clone(),
            key.clone(),
            CatalogExpectation::Absent,
            artifact(json!({"value":1}))?,
        ))?;
        let current = service
            .status(&key, 32)?
            .current_annotation_object_id()
            .ok_or("missing current object")?
            .clone();
        let error = service
            .publish(PublishAnnotationRequest::new(
                operation_id,
                key,
                CatalogExpectation::Exact(current),
                artifact(json!({"value":2}))?,
            ))
            .err()
            .ok_or("expected operation conflict")?;
        assert_eq!(error.code(), AnnotationAdminErrorCode::OperationConflict);
        Ok(())
    }

    #[test]
    fn wrong_object_kind_never_decodes_as_annotation_artifact()
    -> Result<(), Box<dyn std::error::Error>> {
        let configuration = configuration()?;
        let mut store = Store::open_in_memory(configuration.store_configuration().clone())?;
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
        let error = read_exact(&store, &object_id)
            .err()
            .ok_or("expected kind mismatch")?;
        assert_eq!(error.code(), AnnotationAdminErrorCode::ArtifactInvalid);
        Ok(())
    }
}
