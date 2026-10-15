//! One-shot build-and-query service operation over one exact retained project generation.
//!
//! The first route returns an inline result only. It never returns a continuation,
//! and it requires explicit finite owner, Store, lease, and output bounds.

mod documents;
mod reference;
pub use reference::{
    ReferenceContinueRequest, ReferenceIndexBuildRequest, ReferenceResultReleaseRequest,
    ReferenceShardRequest,
};
mod retrieval;
pub use retrieval::{SearchContinueRequest, SearchResultReleaseRequest, SearchRetrievalRequest};
mod file_set;
use documents::{Documents, Profile as DurableProfile, Selection as DurableSelection};

use std::{
    num::NonZeroU64,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, canonical_json_bytes, domain_separated_digest};
use wow_project::{ProjectErrorCode, ProjectFileId};
use wow_search::{
    PreparedProjectFileShard, SearchBuildProfile, SearchError, SearchEvidenceClass,
    SearchShardBuildRequest, SearchShardOpenRequest, SearchShardPrepareRequest,
    SearchStoreLeaseRequest, build_project_file_shard, open_project_file_shard,
    prepare_project_file_shard,
};
use wow_store::{
    CatalogExpectation, CatalogMutation, CatalogName, CatalogPath, LeaseId, LogicalEpoch,
    OperationBegin as StoreOperationBegin, OperationId as StoreOperationId,
    OperationState as StoreOperationState, PendingObject, RequestDigest, Store, StoreConfiguration,
    StoreError, StoreErrorCode, WriteBatch,
    project::{
        ProjectStore, ReadSelector, RetentionRoot, RetentionRootId, RetentionRootKind,
        StoreGenerationId,
    },
};

pub use wow_search::{LocalMetadataPermission, SearchBounds};
pub use wow_store::StoreLimits as SearchStoreLimits;

use crate::{
    OperationId, ServiceError, ServiceErrorCode, ServiceResult,
    identity::canonical_digest,
    operation::{CompletedResult, OperationKind, OperationRegistry, RegistryDecision},
};

const SEARCH_RESULT_SCHEMA: &str = "wow-service/search-result-envelope/1";
const SEARCH_STORE_PROFILE_ID: &str = "wow-search.one-shot-store.v1";
const DURABLE_SEARCH_STORE_PROFILE_ID: &str = "wow-search.durable-shard-store.v2";
const SEARCH_SHARD_CATALOG: &str = "wow-search.shards";
const SEARCH_OPERATION_RESULT_KIND: &str = "wow.search.operation-result";
const SEARCH_OPERATION_RESULT_SCHEMA_VERSION: u32 = 1;
const SEARCH_SOURCE_HOLD_HOLDER: &str = "wow-service.search-shard";

/// Strict request for one explicit exact-generation build followed by one bounded
/// exact-ID, exact-label, and literal FTS query. The path is transport only and is
/// excluded from canonical request identity.
pub struct SearchBuildQueryRequest {
    operation_id: OperationId,
    project_store_root: PathBuf,
    store_generation_id: Box<str>,
    file_id: Box<str>,
    literal_query: Box<str>,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
    lease_expires_after: u64,
    file_set: Option<(
        wow_search::ProjectFileSetSelection,
        wow_search::SearchFileSetBuildProfile,
    )>,
}

impl SearchBuildQueryRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        project_store_root: PathBuf,
        store_generation_id: impl Into<Box<str>>,
        file_id: impl Into<Box<str>>,
        literal_query: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
        lease_expires_after: u64,
    ) -> ServiceResult<Self> {
        if !project_store_root.is_absolute() || lease_expires_after == 0 {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search requires an explicit absolute store root and positive lease epoch",
            ));
        }
        Ok(Self {
            operation_id,
            project_store_root,
            store_generation_id: store_generation_id.into(),
            file_id: file_id.into(),
            literal_query: literal_query.into(),
            bounds,
            store_limits,
            max_output_bytes,
            permission,
            lease_expires_after,
            file_set: None,
        })
    }

    /// Select an explicit finite set while keeping file_id as the exact query root.
    pub fn with_file_set(mut self, ids: Vec<String>, stop: &AtomicBool) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if ids.len() > self.bounds.max_documents() as usize {
            return Err(failure(
                ServiceErrorCode::BudgetExceeded,
                "ProjectFile set exceeds the document cap",
            ));
        }
        let profile =
            wow_search::SearchFileSetBuildProfile::new(self.bounds).map_err(map_search_error)?;
        let ids = ids
            .into_iter()
            .map(|id| {
                check_cancelled(stop)?;
                ProjectFileId::parse(&id).map_err(map_project_error)
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let selection = wow_search::ProjectFileSetSelection::new(ids, &profile, stop)
            .map_err(map_search_error)?;
        let query_id = ProjectFileId::parse(&self.file_id).map_err(map_project_error)?;
        if selection.ids().binary_search(&query_id).is_err() {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "query file must belong to the explicit set",
            ));
        }
        self.file_set = Some((selection, profile));
        Ok(self)
    }
}

pub struct SearchIndexBuildRequest {
    operation_id: OperationId,
    project_store_root: PathBuf,
    search_store_path: PathBuf,
    store_generation_id: Box<str>,
    selection: DurableSelection,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}

impl SearchIndexBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        project_store_root: PathBuf,
        search_store_path: PathBuf,
        store_generation_id: impl Into<Box<str>>,
        file_id: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !project_store_root.is_absolute() || !search_store_path.is_absolute() {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search requires explicit absolute source and SearchStore paths",
            ));
        }
        Ok(Self {
            operation_id,
            project_store_root,
            search_store_path,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::Single(file_id.into()),
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}

pub struct SearchShardQueryRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    store_generation_id: Box<str>,
    file_id: Box<str>,
    logical_shard_id: Box<str>,
    object_id: Box<str>,
    profile_digest: Box<str>,
    literal_query: Box<str>,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
    file_set_component: bool,
}

impl SearchShardQueryRequest {
    /// Select the exact file-set reader; profile mismatches are refused.
    #[must_use]
    pub fn with_file_set_component(mut self) -> Self {
        self.file_set_component = true;
        self
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        store_generation_id: impl Into<Box<str>>,
        file_id: impl Into<Box<str>>,
        logical_shard_id: impl Into<Box<str>>,
        object_id: impl Into<Box<str>>,
        profile_digest: impl Into<Box<str>>,
        literal_query: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !search_store_path.is_absolute() || lease_expires_after == 0 {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search query requires an absolute SearchStore path and positive lease epoch",
            ));
        }
        Ok(Self {
            operation_id,
            search_store_path,
            store_generation_id: store_generation_id.into(),
            file_id: file_id.into(),
            logical_shard_id: logical_shard_id.into(),
            object_id: object_id.into(),
            profile_digest: profile_digest.into(),
            literal_query: literal_query.into(),
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            file_set_component: false,
        })
    }
}

pub struct SearchShardRetireRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    source_hold: SearchSourceHoldBinding,
    store_generation_id: Box<str>,
    selection: DurableSelection,
    logical_shard_id: Box<str>,
    object_id: Box<str>,
    profile_digest: Box<str>,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}

/// Explicit transport and exact owner receipt required to retire a durable
/// shard and release the ProjectStore generation it retained.
pub struct SearchSourceHoldBinding {
    project_store_root: PathBuf,
    build_operation_id: OperationId,
    source_store_epoch_id: Box<str>,
    retention_root_id: Box<str>,
    retention_root_digest: Box<str>,
}

impl SearchSourceHoldBinding {
    pub fn new(
        project_store_root: PathBuf,
        build_operation_id: OperationId,
        source_store_epoch_id: impl Into<Box<str>>,
        retention_root_id: impl Into<Box<str>>,
        retention_root_digest: impl Into<Box<str>>,
    ) -> ServiceResult<Self> {
        let source_store_epoch_id = source_store_epoch_id.into();
        let retention_root_id = retention_root_id.into();
        let retention_root_digest = retention_root_digest.into();
        if !project_store_root.is_absolute()
            || source_store_epoch_id.is_empty()
            || source_store_epoch_id.len() > 128
            || retention_root_digest.is_empty()
            || retention_root_digest.len() > 128
        {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search retirement requires an explicit exact source hold binding",
            ));
        }
        RetentionRootId::new(retention_root_id.to_string()).map_err(map_store_error)?;
        Ok(Self {
            project_store_root,
            build_operation_id,
            source_store_epoch_id,
            retention_root_id,
            retention_root_digest,
        })
    }
}

impl SearchShardRetireRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        source_hold: SearchSourceHoldBinding,
        store_generation_id: impl Into<Box<str>>,
        file_id: impl Into<Box<str>>,
        logical_shard_id: impl Into<Box<str>>,
        object_id: impl Into<Box<str>>,
        profile_digest: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !search_store_path.is_absolute() || lease_expires_after == 0 {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search retirement requires an absolute SearchStore path and positive lease epoch",
            ));
        }
        Ok(Self {
            operation_id,
            search_store_path,
            source_hold,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::Single(file_id.into()),
            logical_shard_id: logical_shard_id.into(),
            object_id: object_id.into(),
            profile_digest: profile_digest.into(),
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}

pub struct SearchStoreGcRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    target_object_id: Box<str>,
    store_limits: SearchStoreLimits,
    now: u64,
    max_deletes: u32,
    max_output_bytes: NonZeroU64,
}

impl SearchStoreGcRequest {
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        target_object_id: impl Into<Box<str>>,
        store_limits: SearchStoreLimits,
        now: u64,
        max_deletes: u32,
        max_output_bytes: NonZeroU64,
    ) -> ServiceResult<Self> {
        if !search_store_path.is_absolute()
            || max_deletes == 0
            || max_deletes > store_limits.max_gc_deletes
            || now > i64::MAX as u64
        {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "search GC requires an absolute SearchStore path and positive delete bound",
            ));
        }
        Ok(Self {
            operation_id,
            search_store_path,
            target_object_id: target_object_id.into(),
            store_limits,
            now,
            max_deletes,
            max_output_bytes,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchBuildQueryStatus {
    ExactFound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SearchCandidateRecord {
    entity_id: Box<str>,
    owner_metadata_label: Box<str>,
    evidence: &'static str,
}

impl SearchCandidateRecord {
    #[must_use]
    pub fn entity_id(&self) -> &str {
        &self.entity_id
    }

    #[must_use]
    pub fn owner_metadata_label(&self) -> &str {
        &self.owner_metadata_label
    }

    #[must_use]
    pub const fn evidence(&self) -> &'static str {
        self.evidence
    }
}

#[derive(Debug, Clone)]
pub struct SearchBuildQueryResult {
    result_id: Box<str>,
    canonical_bytes: Box<[u8]>,
}

impl SearchBuildQueryResult {
    #[must_use]
    pub fn result_id(&self) -> &str {
        &self.result_id
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

#[derive(Debug, Default)]
pub struct SearchOperationService {
    operations: OperationRegistry,
}

impl SearchOperationService {
    pub fn continue_reference_results(
        &self,
        request: &ReferenceContinueRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        reference::pagination::continue_result(request, stop)
    }
    pub fn release_reference_results(
        &self,
        request: &ReferenceResultReleaseRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        reference::pagination::release_result(request, stop)
    }
    pub fn build_reference_shard(
        &self,
        request: &ReferenceIndexBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        reference::build(request, stop)
    }
    pub fn query_reference_shard(
        &self,
        request: &ReferenceShardRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        reference::execute(request, false, stop)
    }
    pub fn retire_reference_shard(
        &self,
        request: &ReferenceShardRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        reference::execute(request, true, stop)
    }
    pub fn continue_results(
        &self,
        request: &SearchContinueRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        retrieval::pagination::continue_result(request, stop)
    }

    pub fn release_results(
        &self,
        request: &SearchResultReleaseRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        retrieval::pagination::release_result(request, stop)
    }

    pub fn retrieve_shard(
        &self,
        request: &SearchRetrievalRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        retrieval::execute(request, stop)
    }
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn operation_registry(&self) -> ServiceResult<crate::OperationRegistrySnapshot> {
        self.operations.snapshot()
    }

    pub fn build_query(
        &self,
        request: &SearchBuildQueryRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Arc<SearchBuildQueryResult>> {
        check_cancelled(stop)?;
        let profile = SearchBuildProfile::new(request.bounds).map_err(map_search_error)?;
        let request_digest = if let Some((selection, selected_profile)) = &request.file_set {
            file_set::request_digest(request, selection, selected_profile)?
        } else {
            canonical_digest(
                "wow-service/search-build-query-request/1:",
                &SearchRequestIdentity {
                    operation_id: request.operation_id.as_str(),
                    store_generation_id: &request.store_generation_id,
                    file_id: &request.file_id,
                    literal_query: &request.literal_query,
                    search_profile_digest: profile.digest(),
                    store_limits: request.store_limits,
                    max_output_bytes: request.max_output_bytes.get(),
                    lease_expires_after: request.lease_expires_after,
                    privacy_scope: "local_metadata_only",
                },
            )?
        };
        match self.operations.begin(
            &request.operation_id,
            &request_digest,
            OperationKind::Search,
        )? {
            RegistryDecision::Replay(CompletedResult::Search(result)) => return Ok(result),
            RegistryDecision::Replay(_) => {
                return Err(ServiceError::for_operation(
                    ServiceErrorCode::InternalContractViolation,
                    "search operation replay resolved to another result type",
                    request.operation_id.as_str(),
                ));
            }
            RegistryDecision::Started => {}
        }

        let outcome = match &request.file_set {
            Some((selection, selected_profile)) => {
                file_set::execute(request, selection, selected_profile, &request_digest, stop)
            }
            None => execute_build_query(request, &profile, &request_digest, stop),
        };
        match outcome {
            Ok(result) => {
                let result = Arc::new(result);
                self.operations.complete(
                    &request.operation_id,
                    &request_digest,
                    OperationKind::Search,
                    CompletedResult::Search(result.clone()),
                )?;
                Ok(result)
            }
            Err(error) => {
                self.operations.abandon(
                    &request.operation_id,
                    &request_digest,
                    OperationKind::Search,
                );
                Err(ServiceError::for_operation(
                    error.code(),
                    error.message().to_owned(),
                    request.operation_id.as_str(),
                ))
            }
        }
    }

    pub fn build_shard(
        &self,
        request: &SearchIndexBuildRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        durable_build_shard(request, stop)
    }

    pub fn query_shard(
        &self,
        request: &SearchShardQueryRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        durable_query_shard(request, stop)
    }

    pub fn retire_shard(
        &self,
        request: &SearchShardRetireRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        durable_retire_shard(request, stop)
    }

    pub fn collect_search_garbage(
        &self,
        request: &SearchStoreGcRequest,
        stop: &AtomicBool,
    ) -> ServiceResult<Box<[u8]>> {
        durable_collect_garbage(request, stop)
    }
}

#[derive(Serialize)]
struct DurableRequestIdentity<'a, T> {
    schema: &'static str,
    operation_id: &'a str,
    input: &'a T,
}

#[derive(Serialize)]
struct DurableResponseIdentity<'a, T> {
    operation: &'static str,
    request_digest: &'a str,
    payload: &'a T,
}

#[derive(Serialize)]
struct DurableResponseEnvelope<'a, T> {
    schema: &'static str,
    operation: &'static str,
    operation_id: &'a str,
    request_digest: &'a str,
    result_id: &'a str,
    payload: &'a T,
}

#[derive(Serialize)]
struct DurableBuildInput<'a> {
    store_generation_id: &'a str,
    #[serde(flatten)]
    selection: documents::Identity<'a>,
    search_profile_digest: &'a str,
    source_store_epoch_id: &'a str,
    source_generation_hold_id: &'a str,
    source_generation_hold_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    privacy_scope: &'static str,
}

#[derive(Serialize)]
struct SearchSourceHoldIdentity<'a> {
    search_store_instance_id: &'a wow_store::StoreInstanceId,
    build_operation_id: &'a str,
    source_store_epoch_id: &'a str,
    store_generation_id: &'a str,
    #[serde(flatten)]
    selection: documents::Identity<'a>,
    search_profile_digest: &'a str,
    store_limits: SearchStoreLimits,
}

#[derive(Serialize)]
struct DurableQueryInput<'a> {
    store_generation_id: &'a str,
    file_id: &'a str,
    logical_shard_id: &'a str,
    object_id: &'a str,
    search_profile_digest: &'a str,
    literal_query_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_component: Option<&'static str>,
}

#[derive(Serialize)]
struct DurableRetireInput<'a> {
    store_generation_id: &'a str,
    #[serde(flatten)]
    selection: documents::Identity<'a>,
    logical_shard_id: &'a str,
    object_id: &'a str,
    search_profile_digest: &'a str,
    source_build_operation_id: &'a str,
    source_store_epoch_id: &'a str,
    source_generation_hold_id: &'a str,
    source_generation_hold_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
}

#[derive(Serialize)]
struct DurableGcInput<'a> {
    target_object_id: &'a str,
    store_limits: SearchStoreLimits,
    now: u64,
    max_deletes: u32,
    max_output_bytes: u64,
}

#[derive(Serialize)]
struct NativeGarbageCollectionRequest<'a> {
    target_object_id: &'a str,
    store_limits: SearchStoreLimits,
    now: u64,
    max_deletes: u32,
}

#[derive(Serialize)]
struct DurableGcInnerIdentity<'a> {
    outer_operation_id: &'a str,
    outer_request_digest: &'a str,
    native_request_digest: &'a str,
}

#[derive(Serialize)]
struct DurableBuildPayload<'a> {
    status: &'static str,
    resource_closure: &'static str,
    source_store_generation_id: &'a str,
    source_store_epoch_id: &'a str,
    source_generation_hold_id: &'a str,
    source_generation_hold_digest: &'a str,
    logical_shard_id: &'a str,
    object_id: &'a str,
    search_profile_digest: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    owner_metadata_label: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    documents: Option<Vec<DurableDocument<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_component: Option<&'static str>,
    privacy_scope: &'static str,
    license_state: &'static str,
}

#[derive(Serialize)]
struct DurableQueryPayload<'a> {
    status: &'static str,
    resource_closure: &'static str,
    source_store_generation_id: &'a str,
    logical_shard_id: &'a str,
    object_id: &'a str,
    search_profile_digest: &'a str,
    entity_id: &'a str,
    owner_metadata_label: &'a str,
    exact_id_match: bool,
    exact_owner_metadata_label_matches: u32,
    literal_query_digest: &'a str,
    literal_fts_candidates: &'a [SearchCandidateRecord],
    privacy_scope: &'static str,
    license_state: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_component: Option<&'static str>,
}

#[derive(Serialize)]
struct DurableRetirePayload<'a> {
    status: &'static str,
    resource_closure: &'static str,
    source_store_generation_id: &'a str,
    source_build_operation_id: &'a str,
    source_store_epoch_id: &'a str,
    source_generation_hold_id: &'a str,
    source_generation_hold_digest: &'a str,
    logical_shard_id: &'a str,
    object_id: &'a str,
    search_profile_digest: &'a str,
    privacy_scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    document_component: Option<&'static str>,
}

#[derive(Serialize)]
struct DurableDocument<'a> {
    entity_id: &'a str,
    owner_metadata_label: &'a str,
}

#[derive(Serialize)]
struct DurableGcPayload<'a> {
    status: &'static str,
    target_object_id: &'a str,
    target_object_deleted: bool,
    deleted_object_count: u32,
    more_available: bool,
}

enum DurableOperationStart {
    Started,
    Replay(Box<[u8]>),
}

enum DurableSourceOperationStart {
    Started,
    Prepared,
    Released,
    Replay(Box<[u8]>),
}

enum DurableGcOuterStart {
    Started,
    Prepared,
    Released,
    Replay(Box<[u8]>),
}

fn durable_identity<T: Serialize>(
    schema: &'static str,
    operation_id: &OperationId,
    input: &T,
) -> ServiceResult<(StoreOperationId, RequestDigest, Box<str>)> {
    let operation = StoreOperationId::new(operation_id.as_str()).map_err(map_store_error)?;
    let digest = canonical_digest(
        "sha256:",
        &DurableRequestIdentity {
            schema,
            operation_id: operation_id.as_str(),
            input,
        },
    )?;
    let request_digest = RequestDigest::new(digest.to_string()).map_err(map_store_error)?;
    Ok((operation, request_digest, digest))
}

fn search_source_hold_root(
    project_store: &ProjectStore,
    search_store_instance_id: &wow_store::StoreInstanceId,
    build_operation_id: &OperationId,
    generation: &StoreGenerationId,
    documents: &Documents<'_>,
    profile: &DurableProfile,
    store_limits: SearchStoreLimits,
) -> ServiceResult<RetentionRoot> {
    let source_store_epoch_id = project_store.epoch().epoch_id();
    let identity_digest = domain_separated_digest(
        "wow-service/search-source-generation-hold/2",
        &SearchSourceHoldIdentity {
            search_store_instance_id,
            build_operation_id: build_operation_id.as_str(),
            source_store_epoch_id: source_store_epoch_id.as_str(),
            store_generation_id: generation.as_str(),
            selection: documents.identity(),
            search_profile_digest: profile.digest(),
            store_limits,
        },
    )
    .map_err(|_| {
        failure(
            ServiceErrorCode::CanonicalizationFailed,
            "search hold identity encoding failed",
        )
    })?;
    let root_id = RetentionRootId::new(format!(
        "wow-service.search-source-hold:{}",
        ContentDigest::<CanonicalResult>::from_bytes(identity_digest)
    ))
    .map_err(map_store_error)?;
    RetentionRoot::new(
        source_store_epoch_id.clone(),
        root_id,
        RetentionRootKind::Policy,
        generation.clone(),
        SEARCH_SOURCE_HOLD_HOLDER,
    )
    .map_err(map_store_error)
}

fn durable_build_identity(
    request: &SearchIndexBuildRequest,
    profile: &DurableProfile,
    source_hold: &RetentionRoot,
) -> ServiceResult<(StoreOperationId, RequestDigest, Box<str>)> {
    let input = DurableBuildInput {
        store_generation_id: &request.store_generation_id,
        selection: request.selection.identity(),
        search_profile_digest: profile.digest(),
        source_store_epoch_id: source_hold.epoch_id().as_str(),
        source_generation_hold_id: source_hold.root_id().as_str(),
        source_generation_hold_digest: source_hold.pin_digest(),
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes.get(),
        privacy_scope: "local_metadata_only",
    };
    durable_identity(
        "wow-service/search-index-build-request/1",
        &request.operation_id,
        &input,
    )
}

fn source_hold_present(
    project_store: &ProjectStore,
    expected: &RetentionRoot,
    stop: &AtomicBool,
) -> ServiceResult<bool> {
    let roots = project_store
        .retention_roots(stop)
        .map_err(map_store_error)?;
    match roots
        .into_iter()
        .find(|candidate| candidate.root_id() == expected.root_id())
    {
        None => Ok(false),
        Some(actual) if actual == *expected => Ok(true),
        Some(_) => Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "ProjectStore source hold identity conflicts with the exact Search shard",
        )),
    }
}

fn ensure_source_hold(
    project_store: &mut ProjectStore,
    expected: &RetentionRoot,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    if source_hold_present(project_store, expected, stop)? {
        return Ok(());
    }
    project_store
        .put_retention_root(expected, stop)
        .map_err(map_store_error)?;
    if source_hold_present(project_store, expected, stop)? {
        Ok(())
    } else {
        Err(source_hold_outcome_unknown(expected))
    }
}

fn release_source_hold(
    project_store: &mut ProjectStore,
    expected: &RetentionRoot,
    stop: &AtomicBool,
    missing_is_reconciled: bool,
) -> ServiceResult<()> {
    if !source_hold_present(project_store, expected, stop)? {
        return if missing_is_reconciled {
            Ok(())
        } else {
            Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "exact ProjectStore source hold is missing before Search shard retirement",
            ))
        };
    }
    match project_store.remove_retention_root(expected.root_id(), expected.pin_digest(), stop) {
        Ok(true) => Ok(()),
        Ok(false) => Err(source_hold_outcome_unknown(expected)),
        Err(error) => Err(map_store_error(error)),
    }
}

fn validate_retirement_source_hold(
    project_store: &ProjectStore,
    search_store_instance_id: &wow_store::StoreInstanceId,
    request: &SearchShardRetireRequest,
    generation: &StoreGenerationId,
    documents: &Documents<'_>,
    profile: &DurableProfile,
) -> ServiceResult<RetentionRoot> {
    let binding = &request.source_hold;
    let source_store_epoch_id = project_store.epoch().epoch_id().as_str();
    if binding.source_store_epoch_id.as_ref() != source_store_epoch_id {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "retirement source path does not open the exact retained ProjectStore epoch",
        ));
    }
    let expected = search_source_hold_root(
        project_store,
        search_store_instance_id,
        &binding.build_operation_id,
        generation,
        documents,
        profile,
        request.store_limits,
    )?;
    if expected.root_id().as_str() != binding.retention_root_id.as_ref()
        || expected.pin_digest() != binding.retention_root_digest.as_ref()
    {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "retirement hold receipt does not bind the exact source epoch and shard request",
        ));
    }
    Ok(expected)
}

fn source_hold_outcome_unknown(root: &RetentionRoot) -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::StoreOutcomeUnknown,
        format!(
            "search source hold outcome is unknown; reconcile epoch={} hold={} digest={}",
            root.epoch_id().as_str(),
            root.root_id().as_str(),
            root.pin_digest(),
        ),
    )
}

fn source_hold_binding_outcome_unknown(binding: &SearchSourceHoldBinding) -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::StoreOutcomeUnknown,
        format!(
            "search source hold outcome is unknown; reconcile epoch={} hold={} digest={}",
            binding.source_store_epoch_id, binding.retention_root_id, binding.retention_root_digest,
        ),
    )
}

fn cleanup_source_hold(
    project_store_root: &std::path::Path,
    expected: &RetentionRoot,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    let mut project_store =
        crate::live_project::LiveProjectStore::open_retained_owner_store(project_store_root)?;
    if project_store.epoch().epoch_id() != expected.epoch_id() {
        let closed = project_store.close();
        return if closed.is_err() {
            Err(source_hold_outcome_unknown(expected))
        } else {
            Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "source hold cleanup opened a different physical ProjectStore epoch",
            ))
        };
    }
    let outcome = release_source_hold(&mut project_store, expected, stop, true);
    let closed = project_store.close();
    match (outcome, closed) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (_, Err(_)) => Err(source_hold_outcome_unknown(expected)),
    }
}

fn record_no_effect_and_release_source_hold(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    error: ServiceError,
    project_store: &mut ProjectStore,
    source_hold: &RetentionRoot,
    stop: &AtomicBool,
) -> ServiceError {
    let recorded = record_no_effect(store, operation_id, request_digest, error);
    if recorded.code() == ServiceErrorCode::StoreOutcomeUnknown {
        return recorded;
    }
    match release_source_hold(project_store, source_hold, stop, true) {
        Ok(()) => recorded,
        Err(_) => source_hold_outcome_unknown(source_hold),
    }
}

fn fail_build_before_publish(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    error: ServiceError,
    project_store: &mut ProjectStore,
    source_hold: &RetentionRoot,
    stop: &AtomicBool,
) -> ServiceError {
    if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
        return source_hold_outcome_unknown(source_hold);
    }
    record_no_effect_and_release_source_hold(
        store,
        operation_id,
        request_digest,
        error,
        project_store,
        source_hold,
        stop,
    )
}

fn open_durable_search_store(
    path: &std::path::Path,
    limits: SearchStoreLimits,
) -> ServiceResult<Store> {
    let configuration =
        StoreConfiguration::new_with_instance_identity(DURABLE_SEARCH_STORE_PROFILE_ID, limits)
            .map_err(map_store_error)?;
    Store::open(path, configuration).map_err(map_store_error)
}

fn start_durable_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    max_output_bytes: u64,
) -> ServiceResult<DurableOperationStart> {
    match store
        .begin_operation(operation_id.clone(), request_digest.clone())
        .map_err(map_store_error)?
    {
        StoreOperationBegin::Started(_) => Ok(DurableOperationStart::Started),
        StoreOperationBegin::Replay(record) => match record.state() {
            StoreOperationState::Completed => Ok(DurableOperationStart::Replay(
                read_completed_response(store, &record, max_output_bytes)?,
            )),
            StoreOperationState::Prepared => {
                let _ = store.mark_outcome_unknown(operation_id, request_digest);
                Err(outcome_unknown())
            }
            StoreOperationState::OutcomeUnknown => Err(outcome_unknown()),
            StoreOperationState::NoEffect | StoreOperationState::Failed => Err(ServiceError::new(
                ServiceErrorCode::OperationReleased,
                "exact search operation has a recorded noncompleted outcome",
            )),
        },
    }
}

/// Source-hold build/retirement operations may resume Prepared only after the
/// exact ProjectStore hold has been reconciled. Other durable operations keep
/// the stricter non-resumable start helper above.
fn start_durable_source_hold_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    max_output_bytes: u64,
) -> ServiceResult<DurableSourceOperationStart> {
    match store
        .begin_operation(operation_id.clone(), request_digest.clone())
        .map_err(map_store_error)?
    {
        StoreOperationBegin::Started(_) => Ok(DurableSourceOperationStart::Started),
        StoreOperationBegin::Replay(record) => match record.state() {
            StoreOperationState::Completed => Ok(DurableSourceOperationStart::Replay(
                read_completed_response(store, &record, max_output_bytes)?,
            )),
            StoreOperationState::Prepared => Ok(DurableSourceOperationStart::Prepared),
            StoreOperationState::OutcomeUnknown => Err(outcome_unknown()),
            StoreOperationState::NoEffect | StoreOperationState::Failed => {
                Ok(DurableSourceOperationStart::Released)
            }
        },
    }
}

fn start_durable_gc_outer_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    max_output_bytes: u64,
) -> ServiceResult<DurableGcOuterStart> {
    match store
        .begin_operation(operation_id.clone(), request_digest.clone())
        .map_err(map_store_error)?
    {
        StoreOperationBegin::Started(_) => Ok(DurableGcOuterStart::Started),
        StoreOperationBegin::Replay(record) => match record.state() {
            StoreOperationState::Completed => Ok(DurableGcOuterStart::Replay(
                read_completed_response(store, &record, max_output_bytes)?,
            )),
            StoreOperationState::Prepared => Ok(DurableGcOuterStart::Prepared),
            StoreOperationState::OutcomeUnknown => Err(outcome_unknown()),
            StoreOperationState::NoEffect | StoreOperationState::Failed => {
                Ok(DurableGcOuterStart::Released)
            }
        },
    }
}

fn durable_gc_inner_identity(
    request: &SearchStoreGcRequest,
    outer_operation_id: &StoreOperationId,
    outer_request_digest: &RequestDigest,
) -> ServiceResult<(StoreOperationId, RequestDigest)> {
    let native_digest = canonical_digest(
        "sha256:",
        &NativeGarbageCollectionRequest {
            target_object_id: &request.target_object_id,
            store_limits: request.store_limits,
            now: request.now,
            max_deletes: request.max_deletes,
        },
    )?;
    let request_digest = RequestDigest::new(native_digest.clone()).map_err(map_store_error)?;
    let operation_identity = canonical_digest(
        "wow-service.search-gc-inner:",
        &DurableGcInnerIdentity {
            outer_operation_id: outer_operation_id.as_str(),
            outer_request_digest: outer_request_digest.as_str(),
            native_request_digest: &native_digest,
        },
    )?;
    let operation_id = StoreOperationId::new(operation_identity).map_err(map_store_error)?;
    if &operation_id == outer_operation_id {
        return Err(outcome_unknown());
    }
    Ok((operation_id, request_digest))
}

fn complete_durable_gc_outer_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    response_bytes: &[u8],
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    check_output_bound(response_bytes, max_output_bytes).map_err(|_| outcome_unknown())?;
    let response = response_object(store, response_bytes).map_err(|_| outcome_unknown())?;
    let result_object_id = response.object_id().clone();
    let mut batch = WriteBatch::new();
    batch.add_object(response).map_err(|_| outcome_unknown())?;
    match store.commit_operation(batch, operation_id, request_digest, &result_object_id) {
        Ok(record) if record.result_object_id() == Some(&result_object_id) => {
            read_completed_response(store, &record, max_output_bytes)
        }
        Ok(_) | Err(_) => {
            let record = store
                .operation(operation_id)
                .map_err(|_| outcome_unknown())?
                .ok_or_else(outcome_unknown)?;
            if record.request_digest() != request_digest
                || record.state() != StoreOperationState::Completed
            {
                return Err(outcome_unknown());
            }
            read_completed_response(store, &record, max_output_bytes)
        }
    }
}

fn read_completed_response(
    store: &Store,
    operation: &wow_store::OperationRecord,
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    let object_id = operation.result_object_id().ok_or_else(outcome_unknown)?;
    let object = store
        .object(object_id)
        .map_err(|_| outcome_unknown())?
        .ok_or_else(outcome_unknown)?;
    if object.kind() != SEARCH_OPERATION_RESULT_KIND
        || object.schema_version() != SEARCH_OPERATION_RESULT_SCHEMA_VERSION
    {
        return Err(outcome_unknown());
    }
    let bytes = object.canonical_json();
    check_output_bound(bytes, max_output_bytes)?;
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| outcome_unknown())?;
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(SEARCH_RESULT_SCHEMA)
        || value
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            != Some(operation.operation_id().as_str())
        || value
            .get("request_digest")
            .and_then(serde_json::Value::as_str)
            != Some(operation.request_digest().as_str())
    {
        return Err(outcome_unknown());
    }
    Ok(bytes.into())
}

fn response_object(store: &Store, bytes: &[u8]) -> ServiceResult<PendingObject> {
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| {
        failure(
            ServiceErrorCode::CanonicalizationFailed,
            "search response is not canonical JSON",
        )
    })?;
    let object = PendingObject::from_json(
        SEARCH_OPERATION_RESULT_KIND,
        SEARCH_OPERATION_RESULT_SCHEMA_VERSION,
        &value,
        store.configuration().limits(),
    )
    .map_err(map_store_error)?;
    if object.canonical_json() != bytes {
        return Err(failure(
            ServiceErrorCode::CanonicalizationFailed,
            "stored search response would change canonical bytes",
        ));
    }
    Ok(object)
}

fn complete_durable_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    mut batch: WriteBatch,
    response_bytes: &[u8],
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    if let Err(error) = check_output_bound(response_bytes, max_output_bytes) {
        return Err(record_no_effect(store, operation_id, request_digest, error));
    }
    let response = match response_object(store, response_bytes) {
        Ok(response) => response,
        Err(error) => return Err(record_no_effect(store, operation_id, request_digest, error)),
    };
    let result_object_id = response.object_id().clone();
    if let Err(error) = batch.add_object(response) {
        return Err(record_no_effect(
            store,
            operation_id,
            request_digest,
            map_store_error(error),
        ));
    }
    match store.commit_operation(batch, operation_id, request_digest, &result_object_id) {
        Ok(record) if record.result_object_id() == Some(&result_object_id) => {
            read_completed_response(store, &record, max_output_bytes)
        }
        Ok(_) => Err(outcome_unknown()),
        Err(error)
            if matches!(
                error.code(),
                StoreErrorCode::DatabaseUnavailable | StoreErrorCode::OutcomeUnknown
            ) =>
        {
            reconcile_durable_operation(store, operation_id, request_digest, max_output_bytes)
        }
        Err(error) => Err(record_no_effect(
            store,
            operation_id,
            request_digest,
            map_store_error(error),
        )),
    }
}

fn reconcile_durable_operation(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    let record = store
        .operation(operation_id)
        .map_err(|_| outcome_unknown())?
        .ok_or_else(outcome_unknown)?;
    if record.request_digest() != request_digest {
        return Err(outcome_unknown());
    }
    match record.state() {
        StoreOperationState::Completed => read_completed_response(store, &record, max_output_bytes),
        StoreOperationState::Prepared => {
            let _ = store.mark_outcome_unknown(operation_id, request_digest);
            Err(outcome_unknown())
        }
        StoreOperationState::OutcomeUnknown => Err(outcome_unknown()),
        StoreOperationState::NoEffect | StoreOperationState::Failed => Err(ServiceError::new(
            ServiceErrorCode::OperationReleased,
            "exact search operation has a recorded noncompleted outcome",
        )),
    }
}

fn record_no_effect(
    store: &mut Store,
    operation_id: &StoreOperationId,
    request_digest: &RequestDigest,
    error: ServiceError,
) -> ServiceError {
    match store.record_no_effect(operation_id, request_digest) {
        Ok(_) => error,
        Err(_) => outcome_unknown(),
    }
}

fn outcome_unknown() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::StoreOutcomeUnknown,
        "search operation outcome is unknown; reconcile the exact operation ID and request digest",
    )
}

fn durable_response<T: Serialize>(
    operation: &'static str,
    operation_id: &OperationId,
    request_digest: &str,
    payload: &T,
    max_output_bytes: u64,
) -> ServiceResult<Box<[u8]>> {
    let result_id = canonical_digest(
        "wow-service/search-durable-result/1:",
        &DurableResponseIdentity {
            operation,
            request_digest,
            payload,
        },
    )?;
    let bytes = canonical_json_bytes(&DurableResponseEnvelope {
        schema: SEARCH_RESULT_SCHEMA,
        operation,
        operation_id: operation_id.as_str(),
        request_digest,
        result_id: &result_id,
        payload,
    })
    .map_err(|_| {
        failure(
            ServiceErrorCode::CanonicalizationFailed,
            "search response encoding failed",
        )
    })?;
    check_output_bound(&bytes, max_output_bytes)?;
    Ok(bytes.into_boxed_slice())
}

fn check_output_bound(bytes: &[u8], max_output_bytes: u64) -> ServiceResult<()> {
    let framed = u64::try_from(bytes.len())
        .ok()
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| {
            failure(
                ServiceErrorCode::BudgetExceeded,
                "search output size overflow",
            )
        })?;
    if framed > max_output_bytes {
        return Err(failure(
            ServiceErrorCode::BudgetExceeded,
            "search response exceeds the caller output bound",
        ));
    }
    Ok(())
}

fn search_catalog_key(logical_shard_id: &str) -> ServiceResult<(CatalogName, CatalogPath)> {
    Ok((
        CatalogName::new(SEARCH_SHARD_CATALOG).map_err(map_store_error)?,
        CatalogPath::new(logical_shard_id.to_owned()).map_err(map_store_error)?,
    ))
}

fn operation_failure(error: ServiceError, operation_id: &OperationId) -> ServiceError {
    ServiceError::for_operation(
        error.code(),
        error.message().to_owned(),
        operation_id.as_str(),
    )
}

fn durable_build_shard(
    request: &SearchIndexBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let profile = DurableProfile::new(request.bounds, request.selection.is_file_set())?;
    let generation = StoreGenerationId::parse(request.store_generation_id.to_string())
        .map_err(map_store_error)?;
    let documents = request.selection.normalize()?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    let search_store_instance_id = store.store_instance_id().map_err(map_store_error)?.clone();
    let identity_store = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &request.project_store_root,
    )?;
    let source_hold = search_source_hold_root(
        &identity_store,
        &search_store_instance_id,
        &request.operation_id,
        &generation,
        &documents,
        &profile,
        request.store_limits,
    );
    let identity_store_close = identity_store.close();
    let source_hold = match (source_hold, identity_store_close) {
        (Ok(root), Ok(())) => root,
        (Err(error), Ok(())) => return Err(error),
        (_, Err(_)) => return Err(outcome_unknown()),
    };
    let (operation_id, request_digest, digest_bytes) =
        durable_build_identity(request, &profile, &source_hold)?;
    let start = start_durable_source_hold_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )?;
    let mut project_store = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &request.project_store_root,
    )
    .map_err(|_| source_hold_outcome_unknown(&source_hold))?;
    if project_store.epoch().epoch_id() != source_hold.epoch_id() {
        let closed = project_store.close();
        return if closed.is_err() {
            Err(source_hold_outcome_unknown(&source_hold))
        } else {
            Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "ProjectStore source path changed physical epoch during exact build acquisition",
            ))
        };
    }
    match start {
        DurableSourceOperationStart::Replay(bytes) => {
            let outcome = reconcile_completed_build(
                &mut store,
                &mut project_store,
                CompletedBuildReconciliation {
                    request,
                    generation: &generation,
                    documents: &documents,
                    profile: &profile,
                    source_hold: &source_hold,
                    request_digest: &request_digest,
                    stop,
                },
                bytes,
            );
            let closed = project_store.close();
            match (outcome, closed) {
                (Ok(bytes), Ok(())) => return Ok(bytes),
                (Err(error), Ok(())) => return Err(error),
                (_, Err(_)) => return Err(source_hold_outcome_unknown(&source_hold)),
            }
        }
        DurableSourceOperationStart::Released => {
            let outcome = release_source_hold(&mut project_store, &source_hold, stop, true);
            let closed = project_store.close();
            return match (outcome, closed) {
                (Ok(()), Ok(())) => Err(ServiceError::new(
                    ServiceErrorCode::OperationReleased,
                    "exact search build has a recorded noncompleted outcome",
                )),
                (Err(error), Ok(())) => Err(error),
                (_, Err(_)) => Err(source_hold_outcome_unknown(&source_hold)),
            };
        }
        DurableSourceOperationStart::Started | DurableSourceOperationStart::Prepared => {}
    }

    if source_hold_present(&project_store, &source_hold, stop).is_err() {
        let _ = project_store.close();
        return Err(source_hold_outcome_unknown(&source_hold));
    }
    let prepared = match prepare_exact_documents_from_store(
        &project_store,
        generation.clone(),
        &documents,
        &profile,
        request.permission,
        request.store_limits,
        stop,
    ) {
        Ok(prepared) => prepared,
        Err(error) => {
            let error = fail_build_before_publish(
                &mut store,
                &operation_id,
                &request_digest,
                error,
                &mut project_store,
                &source_hold,
                stop,
            );
            let closed = project_store.close();
            return Err(if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                operation_failure(error, &request.operation_id)
            });
        }
    };
    if let Err(error) = check_cancelled(stop) {
        let error = fail_build_before_publish(
            &mut store,
            &operation_id,
            &request_digest,
            error,
            &mut project_store,
            &source_hold,
            stop,
        );
        let closed = project_store.close();
        return Err(if closed.is_err() {
            source_hold_outcome_unknown(&source_hold)
        } else {
            operation_failure(error, &request.operation_id)
        });
    }

    let logical_id = prepared.logical_id().to_owned();
    let object_id = prepared.object_id().clone();
    let single_document = match &prepared {
        documents::Prepared::Single(value) => Some(value.document()),
        documents::Prepared::FileSet(_) => None,
    };
    let payload = DurableBuildPayload {
        status: "retained",
        resource_closure: "closed",
        source_store_generation_id: prepared.source_generation(),
        source_store_epoch_id: source_hold.epoch_id().as_str(),
        source_generation_hold_id: source_hold.root_id().as_str(),
        source_generation_hold_digest: source_hold.pin_digest(),
        logical_shard_id: &logical_id,
        object_id: object_id.as_str(),
        search_profile_digest: prepared.profile_digest(),
        entity_id: single_document.map(|document| document.entity_id().as_str()),
        owner_metadata_label: single_document
            .map(|document| document.owner_metadata_label().as_str()),
        documents: if profile.component().is_some() {
            Some(
                prepared
                    .documents()
                    .iter()
                    .map(|document| DurableDocument {
                        entity_id: document.entity_id().as_str(),
                        owner_metadata_label: document.owner_metadata_label().as_str(),
                    })
                    .collect(),
            )
        } else {
            None
        },
        document_component: profile.component(),
        privacy_scope: "local_metadata_only",
        license_state: "not_evaluated",
    };
    let response_bytes = match durable_response(
        "search_index_build",
        &request.operation_id,
        &digest_bytes,
        &payload,
        request.max_output_bytes.get(),
    ) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = fail_build_before_publish(
                &mut store,
                &operation_id,
                &request_digest,
                error,
                &mut project_store,
                &source_hold,
                stop,
            );
            let closed = project_store.close();
            return Err(if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                operation_failure(error, &request.operation_id)
            });
        }
    };

    let (catalog, path) = match search_catalog_key(&logical_id) {
        Ok(key) => key,
        Err(error) => {
            let error = fail_build_before_publish(
                &mut store,
                &operation_id,
                &request_digest,
                error,
                &mut project_store,
                &source_hold,
                stop,
            );
            let closed = project_store.close();
            return Err(if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                operation_failure(error, &request.operation_id)
            });
        }
    };
    let mut batch = WriteBatch::new();
    if let Err(error) = batch.add_object(prepared.into_object()).and_then(|()| {
        batch.add_catalog_mutation(CatalogMutation::set(
            catalog,
            path,
            CatalogExpectation::Absent,
            object_id,
        ))
    }) {
        let error = fail_build_before_publish(
            &mut store,
            &operation_id,
            &request_digest,
            map_store_error(error),
            &mut project_store,
            &source_hold,
            stop,
        );
        let closed = project_store.close();
        return Err(if closed.is_err() {
            source_hold_outcome_unknown(&source_hold)
        } else {
            operation_failure(error, &request.operation_id)
        });
    }
    if let Err(error) = check_cancelled(stop) {
        let error = fail_build_before_publish(
            &mut store,
            &operation_id,
            &request_digest,
            error,
            &mut project_store,
            &source_hold,
            stop,
        );
        let closed = project_store.close();
        return Err(if closed.is_err() {
            source_hold_outcome_unknown(&source_hold)
        } else {
            operation_failure(error, &request.operation_id)
        });
    }
    if let Err(error) = ensure_source_hold(&mut project_store, &source_hold, stop) {
        let reconciled = source_hold_present(&project_store, &source_hold, stop);
        match reconciled {
            Ok(true) => {}
            Ok(false) if error.code() != ServiceErrorCode::StoreOutcomeUnknown => {
                let error = record_no_effect(&mut store, &operation_id, &request_digest, error);
                let closed = project_store.close();
                return Err(if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    operation_failure(error, &request.operation_id)
                });
            }
            _ => {
                let _ = project_store.close();
                return Err(source_hold_outcome_unknown(&source_hold));
            }
        }
    }
    if project_store.close().is_err() {
        return Err(source_hold_outcome_unknown(&source_hold));
    }
    if let Err(error) = check_cancelled(stop) {
        let cleanup_error = cleanup_source_hold(&request.project_store_root, &source_hold, stop);
        let recorded = record_no_effect(&mut store, &operation_id, &request_digest, error);
        return Err(
            if recorded.code() == ServiceErrorCode::StoreOutcomeUnknown || cleanup_error.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                operation_failure(recorded, &request.operation_id)
            },
        );
    }
    match complete_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        batch,
        &response_bytes,
        request.max_output_bytes.get(),
    ) {
        Ok(bytes) => Ok(bytes),
        Err(error) => {
            let operation = store
                .operation(&operation_id)
                .map_err(|_| source_hold_outcome_unknown(&source_hold))?
                .ok_or_else(|| source_hold_outcome_unknown(&source_hold))?;
            match operation.state() {
                StoreOperationState::Completed => {
                    read_completed_response(&store, &operation, request.max_output_bytes.get())
                }
                StoreOperationState::NoEffect | StoreOperationState::Failed => {
                    cleanup_source_hold(&request.project_store_root, &source_hold, stop)?;
                    Err(operation_failure(error, &request.operation_id))
                }
                StoreOperationState::Prepared | StoreOperationState::OutcomeUnknown => {
                    Err(source_hold_outcome_unknown(&source_hold))
                }
            }
        }
    }
}

fn prepare_exact_documents_from_store(
    project_store: &ProjectStore,
    generation: StoreGenerationId,
    documents: &Documents<'_>,
    profile: &DurableProfile,
    permission: LocalMetadataPermission,
    object_limits: SearchStoreLimits,
    stop: &AtomicBool,
) -> ServiceResult<documents::Prepared> {
    let read = project_store
        .read(&ReadSelector::Exact(generation.clone()), stop)
        .map_err(map_store_error)?;
    let prepared = profile.prepare(
        &read,
        &generation,
        documents,
        permission,
        object_limits,
        stop,
    );
    let read_close = read.close();
    if read_close.is_err() {
        return Err(outcome_unknown());
    }
    prepared
}

struct CompletedBuildReconciliation<'a> {
    request: &'a SearchIndexBuildRequest,
    generation: &'a StoreGenerationId,
    documents: &'a Documents<'a>,
    profile: &'a DurableProfile,
    source_hold: &'a RetentionRoot,
    request_digest: &'a RequestDigest,
    stop: &'a AtomicBool,
}

fn reconcile_completed_build(
    store: &mut Store,
    project_store: &mut ProjectStore,
    reconciliation: CompletedBuildReconciliation<'_>,
    response_bytes: Box<[u8]>,
) -> ServiceResult<Box<[u8]>> {
    let CompletedBuildReconciliation {
        request,
        generation,
        documents,
        profile,
        source_hold,
        request_digest,
        stop,
    } = reconciliation;
    let value: serde_json::Value =
        serde_json::from_slice(&response_bytes).map_err(|_| outcome_unknown())?;
    let payload = value.get("payload").ok_or_else(outcome_unknown)?;
    let field = |name: &str| {
        payload
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(outcome_unknown)
    };
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(SEARCH_RESULT_SCHEMA)
        || value.get("operation").and_then(serde_json::Value::as_str) != Some("search_index_build")
        || value
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            != Some(request.operation_id.as_str())
        || value
            .get("request_digest")
            .and_then(serde_json::Value::as_str)
            != Some(request_digest.as_str())
        || field("status")? != "retained"
        || field("source_store_generation_id")? != request.store_generation_id.as_ref()
        || field("source_store_epoch_id")? != source_hold.epoch_id().as_str()
        || field("source_generation_hold_id")? != source_hold.root_id().as_str()
        || field("source_generation_hold_digest")? != source_hold.pin_digest()
        || field("search_profile_digest")? != profile.digest()
    {
        return Err(outcome_unknown());
    }
    let logical_shard_id = field("logical_shard_id")?;
    let object_id =
        wow_store::ObjectId::new(field("object_id")?.to_owned()).map_err(|_| outcome_unknown())?;
    let (catalog, path) = search_catalog_key(logical_shard_id)?;
    let held = source_hold_present(project_store, source_hold, stop)?;
    match store
        .catalog_entry(&catalog, &path)
        .map_err(|_| outcome_unknown())?
    {
        Some(entry) if entry.object_id() == &object_id => {
            if !held {
                let prepared = prepare_exact_documents_from_store(
                    project_store,
                    generation.clone(),
                    documents,
                    profile,
                    request.permission,
                    request.store_limits,
                    stop,
                )?;
                if prepared.logical_id() != logical_shard_id || prepared.object_id() != &object_id {
                    return Err(failure(
                        ServiceErrorCode::IdentityMismatch,
                        "active Search shard no longer matches its exact source generation",
                    ));
                }
                ensure_source_hold(project_store, source_hold, stop)?;
            }
        }
        None => {
            // A completed build whose catalog root is gone was explicitly
            // retired (and may already be collected). Do not resurrect its hold.
            release_source_hold(project_store, source_hold, stop, true)?;
        }
        Some(_) => {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "completed Search build catalog names a different immutable object",
            ));
        }
    }
    Ok(response_bytes)
}

fn durable_query_shard(
    request: &SearchShardQueryRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let profile = DurableProfile::new(request.bounds, request.file_set_component)?;
    if profile.digest() != request.profile_digest.as_ref()
        || request.literal_query.trim().is_empty()
    {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "search request does not match the exact shard profile",
        ));
    }
    let generation = StoreGenerationId::parse(request.store_generation_id.to_string())
        .map_err(map_store_error)?;
    let file_id = ProjectFileId::parse(request.file_id.as_ref()).map_err(map_project_error)?;
    let object_id =
        wow_store::ObjectId::new(request.object_id.to_string()).map_err(map_store_error)?;
    let literal_query_digest = canonical_digest(
        "wow-service/search-literal-query/1:",
        request.literal_query.as_ref(),
    )?;
    let input = DurableQueryInput {
        store_generation_id: &request.store_generation_id,
        file_id: &request.file_id,
        logical_shard_id: &request.logical_shard_id,
        object_id: &request.object_id,
        search_profile_digest: profile.digest(),
        literal_query_digest: &literal_query_digest,
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes.get(),
        lease_expires_after: request.lease_expires_after,
        privacy_scope: "local_metadata_only",
        document_component: profile.component(),
    };
    let (operation_id, request_digest, digest_bytes) = durable_identity(
        "wow-service/search-query-request/1",
        &request.operation_id,
        &input,
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }

    let (catalog, catalog_path) = match search_catalog_key(&request.logical_shard_id) {
        Ok(key) => key,
        Err(error) => {
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
    };
    match store.catalog_entry(&catalog, &catalog_path) {
        Ok(Some(entry)) if entry.object_id() == &object_id => {}
        Ok(Some(_)) => {
            let error = failure(
                ServiceErrorCode::IdentityMismatch,
                "exact search shard catalog entry names another object",
            );
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
        Ok(None) => {
            let error = failure(
                ServiceErrorCode::ExactGenerationUnavailable,
                "exact search shard is not retained in the SearchStore catalog",
            );
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
        Err(error) => {
            return Err(operation_failure(
                record_no_effect(
                    &mut store,
                    &operation_id,
                    &request_digest,
                    map_store_error(error),
                ),
                &request.operation_id,
            ));
        }
    }

    let lease_id = match lease_id_for("search-query", &digest_bytes) {
        Ok(lease_id) => lease_id,
        Err(error) => {
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
    };
    let lease = match SearchStoreLeaseRequest::new(
        lease_id,
        "wow-service-search",
        LogicalEpoch::new(request.lease_expires_after),
    ) {
        Ok(lease) => lease,
        Err(error) => {
            return Err(operation_failure(
                record_no_effect(
                    &mut store,
                    &operation_id,
                    &request_digest,
                    map_store_error(error),
                ),
                &request.operation_id,
            ));
        }
    };
    let shard = match profile.open(
        &mut store,
        generation,
        Some(&file_id),
        None,
        &request.logical_shard_id,
        object_id,
        lease,
        request.permission,
        stop,
    ) {
        Ok(shard) => shard,
        Err(error) => {
            let error = if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(&operation_id, &request_digest);
                error
            } else {
                record_no_effect(&mut store, &operation_id, &request_digest, error)
            };
            return Err(operation_failure(error, &request.operation_id));
        }
    };
    let query_response = durable_query_response(
        &shard,
        &file_id,
        request,
        &literal_query_digest,
        &digest_bytes,
        stop,
    );
    if let Err(error) = shard.release(&mut store) {
        let error = map_search_error(error);
        let _ = store.mark_outcome_unknown(&operation_id, &request_digest);
        return Err(operation_failure(error, &request.operation_id));
    }
    let response_bytes = match query_response {
        Ok(bytes) => bytes,
        Err(error) => {
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
    };
    if let Err(error) = check_cancelled(stop) {
        return Err(operation_failure(
            record_no_effect(&mut store, &operation_id, &request_digest, error),
            &request.operation_id,
        ));
    }
    complete_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        WriteBatch::new(),
        &response_bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}

fn durable_query_response(
    shard: &documents::Opened,
    file_id: &ProjectFileId,
    request: &SearchShardQueryRequest,
    literal_query_digest: &str,
    request_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let exact = shard
        .exact_id_lookup(file_id)
        .map_err(map_search_error)?
        .ok_or_else(|| {
            failure(
                ServiceErrorCode::IdentityMismatch,
                "exact shard ID lookup failed",
            )
        })?;
    let label = exact.document().owner_metadata_label().as_str();
    let exact_labels = shard.exact_label_lookup(label).map_err(map_search_error)?;
    if exact_labels.len() != 1
        || exact_labels[0].document().entity_id() != file_id
        || exact_labels[0].document().owner_metadata_label().as_str() != label
        || exact_labels[0].evidence() != SearchEvidenceClass::ExactOwnerMetadataLabel
    {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "exact owner metadata label lookup did not resolve the selected record",
        ));
    }
    let candidates = shard
        .literal_candidates(&request.literal_query)
        .map_err(map_search_error)?
        .into_iter()
        .map(|candidate| {
            if candidate.evidence() != SearchEvidenceClass::Candidate
                || (!shard.is_file_set()
                    && (candidate.entity_id() != file_id
                        || candidate.owner_metadata_label().as_str() != label))
            {
                return Err(failure(
                    ServiceErrorCode::IdentityMismatch,
                    "literal FTS returned evidence outside the exact one-file shard",
                ));
            }
            Ok(SearchCandidateRecord {
                entity_id: candidate.entity_id().as_str().into(),
                owner_metadata_label: candidate.owner_metadata_label().as_str().into(),
                evidence: "candidate",
            })
        })
        .collect::<ServiceResult<Vec<_>>>()?;
    let labels = u32::try_from(exact_labels.len())
        .map_err(|_| failure(ServiceErrorCode::BudgetExceeded, "label count overflow"))?;
    let payload = DurableQueryPayload {
        status: "candidate_only",
        resource_closure: "closed",
        source_store_generation_id: shard.source_generation(),
        logical_shard_id: shard.logical_id(),
        object_id: shard.object_id().as_str(),
        search_profile_digest: shard.profile_digest(),
        entity_id: exact.document().entity_id().as_str(),
        owner_metadata_label: label,
        exact_id_match: exact.evidence() == SearchEvidenceClass::ExactEntityIdentity,
        exact_owner_metadata_label_matches: labels,
        literal_query_digest,
        literal_fts_candidates: &candidates,
        privacy_scope: "local_metadata_only",
        license_state: "not_evaluated",
        document_component: shard.is_file_set().then_some(documents::FILE_SET_COMPONENT),
    };
    durable_response(
        "search_query",
        &request.operation_id,
        request_digest,
        &payload,
        request.max_output_bytes.get(),
    )
}

fn reconcile_completed_retire(
    store: &mut Store,
    project_store: &mut ProjectStore,
    request: &SearchShardRetireRequest,
    source_hold: &RetentionRoot,
    request_digest: &RequestDigest,
    response_bytes: Box<[u8]>,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let value: serde_json::Value =
        serde_json::from_slice(&response_bytes).map_err(|_| outcome_unknown())?;
    let payload = value.get("payload").ok_or_else(outcome_unknown)?;
    let field = |name: &str| {
        payload
            .get(name)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(outcome_unknown)
    };
    let binding = &request.source_hold;
    if value.get("schema").and_then(serde_json::Value::as_str) != Some(SEARCH_RESULT_SCHEMA)
        || value.get("operation").and_then(serde_json::Value::as_str) != Some("search_index_retire")
        || value
            .get("operation_id")
            .and_then(serde_json::Value::as_str)
            != Some(request.operation_id.as_str())
        || value
            .get("request_digest")
            .and_then(serde_json::Value::as_str)
            != Some(request_digest.as_str())
        || field("status")? != "retired"
        || field("source_store_generation_id")? != request.store_generation_id.as_ref()
        || field("source_build_operation_id")? != binding.build_operation_id.as_str()
        || field("source_store_epoch_id")? != source_hold.epoch_id().as_str()
        || field("source_generation_hold_id")? != source_hold.root_id().as_str()
        || field("source_generation_hold_digest")? != source_hold.pin_digest()
        || field("logical_shard_id")? != request.logical_shard_id.as_ref()
        || field("object_id")? != request.object_id.as_ref()
        || field("search_profile_digest")? != request.profile_digest.as_ref()
    {
        return Err(outcome_unknown());
    }
    let (catalog, path) = search_catalog_key(&request.logical_shard_id)?;
    if store
        .catalog_entry(&catalog, &path)
        .map_err(|_| outcome_unknown())?
        .is_some()
    {
        return Err(outcome_unknown());
    }
    release_source_hold(project_store, source_hold, stop, true)?;
    Ok(response_bytes)
}

fn durable_retire_shard(
    request: &SearchShardRetireRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let profile = DurableProfile::new(request.bounds, request.selection.is_file_set())?;
    if profile.digest() != request.profile_digest.as_ref() {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "retirement profile does not match the exact shard handle",
        ));
    }
    let generation = StoreGenerationId::parse(request.store_generation_id.to_string())
        .map_err(map_store_error)?;
    let documents = request.selection.normalize()?;
    let object_id =
        wow_store::ObjectId::new(request.object_id.to_string()).map_err(map_store_error)?;
    let binding = &request.source_hold;
    let input = DurableRetireInput {
        store_generation_id: &request.store_generation_id,
        selection: request.selection.identity(),
        logical_shard_id: &request.logical_shard_id,
        object_id: &request.object_id,
        search_profile_digest: profile.digest(),
        source_build_operation_id: binding.build_operation_id.as_str(),
        source_store_epoch_id: &binding.source_store_epoch_id,
        source_generation_hold_id: &binding.retention_root_id,
        source_generation_hold_digest: &binding.retention_root_digest,
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes.get(),
        lease_expires_after: request.lease_expires_after,
        privacy_scope: "local_metadata_only",
    };
    let (operation_id, request_digest, digest_bytes) = durable_identity(
        "wow-service/search-index-retire-request/1",
        &request.operation_id,
        &input,
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    let search_store_instance_id = store.store_instance_id().map_err(map_store_error)?.clone();
    let start = start_durable_source_hold_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )?;
    let mut project_store = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &binding.project_store_root,
    )
    .map_err(|_| source_hold_binding_outcome_unknown(binding))?;
    let source_hold = match validate_retirement_source_hold(
        &project_store,
        &search_store_instance_id,
        request,
        &generation,
        &documents,
        &profile,
    ) {
        Ok(root) => root,
        Err(error) => {
            let closed = project_store.close();
            return if closed.is_err() {
                Err(source_hold_binding_outcome_unknown(binding))
            } else {
                Err(error)
            };
        }
    };

    match start {
        DurableSourceOperationStart::Replay(bytes) => {
            let outcome = reconcile_completed_retire(
                &mut store,
                &mut project_store,
                request,
                &source_hold,
                &request_digest,
                bytes,
                stop,
            );
            let closed = project_store.close();
            return match (outcome, closed) {
                (Ok(bytes), Ok(())) => Ok(bytes),
                (Err(error), Ok(())) => Err(error),
                (_, Err(_)) => Err(source_hold_outcome_unknown(&source_hold)),
            };
        }
        DurableSourceOperationStart::Released => {
            let closed = project_store.close();
            return if closed.is_err() {
                Err(source_hold_outcome_unknown(&source_hold))
            } else {
                Err(ServiceError::new(
                    ServiceErrorCode::OperationReleased,
                    "exact Search retirement has a recorded noncompleted outcome",
                ))
            };
        }
        DurableSourceOperationStart::Started | DurableSourceOperationStart::Prepared => {}
    }

    match source_hold_present(&project_store, &source_hold, stop) {
        Ok(true) => {}
        Ok(false) => {
            let prepared = prepare_exact_documents_from_store(
                &project_store,
                generation.clone(),
                &documents,
                &profile,
                request.permission,
                request.store_limits,
                stop,
            );
            let prepared = match prepared {
                Ok(prepared) => prepared,
                Err(error) => {
                    let closed = project_store.close();
                    return if closed.is_err() {
                        Err(source_hold_outcome_unknown(&source_hold))
                    } else {
                        Err(error)
                    };
                }
            };
            if prepared.logical_id() != request.logical_shard_id.as_ref()
                || prepared.object_id() != &object_id
            {
                let closed = project_store.close();
                return if closed.is_err() {
                    Err(source_hold_outcome_unknown(&source_hold))
                } else {
                    Err(failure(
                        ServiceErrorCode::IdentityMismatch,
                        "source generation no longer matches the exact retiring Search shard",
                    ))
                };
            }
            if let Err(error) = ensure_source_hold(&mut project_store, &source_hold, stop) {
                let closed = project_store.close();
                return if closed.is_err() || error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                    Err(source_hold_outcome_unknown(&source_hold))
                } else {
                    Err(error)
                };
            }
        }
        Err(error) => {
            let closed = project_store.close();
            return if closed.is_err() {
                Err(source_hold_outcome_unknown(&source_hold))
            } else {
                Err(error)
            };
        }
    }

    let (catalog, path) = match search_catalog_key(&request.logical_shard_id) {
        Ok(key) => key,
        Err(error) => {
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(&mut store, &operation_id, &request_digest, error)
                },
                &request.operation_id,
            ));
        }
    };
    match store.catalog_entry(&catalog, &path) {
        Ok(Some(entry)) if entry.object_id() == &object_id => {}
        Ok(Some(_)) => {
            let error = failure(
                ServiceErrorCode::IdentityMismatch,
                "exact search shard catalog entry names another object",
            );
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(&mut store, &operation_id, &request_digest, error)
                },
                &request.operation_id,
            ));
        }
        Ok(None) => {
            let error = failure(
                ServiceErrorCode::ExactGenerationUnavailable,
                "exact search shard is already retired or unavailable",
            );
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(&mut store, &operation_id, &request_digest, error)
                },
                &request.operation_id,
            ));
        }
        Err(error) => {
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(
                        &mut store,
                        &operation_id,
                        &request_digest,
                        map_store_error(error),
                    )
                },
                &request.operation_id,
            ));
        }
    }
    let lease_id = match lease_id_for("search-retire", &digest_bytes) {
        Ok(lease_id) => lease_id,
        Err(error) => {
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(&mut store, &operation_id, &request_digest, error)
                },
                &request.operation_id,
            ));
        }
    };
    let lease = match SearchStoreLeaseRequest::new(
        lease_id,
        "wow-service-search",
        LogicalEpoch::new(request.lease_expires_after),
    ) {
        Ok(lease) => lease,
        Err(error) => {
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(
                        &mut store,
                        &operation_id,
                        &request_digest,
                        map_store_error(error),
                    )
                },
                &request.operation_id,
            ));
        }
    };
    let shard = match profile.open(
        &mut store,
        generation,
        documents.single_id(),
        Some(&documents),
        &request.logical_shard_id,
        object_id.clone(),
        lease,
        request.permission,
        stop,
    ) {
        Ok(shard) => shard,
        Err(error) => {
            let error = if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(&operation_id, &request_digest);
                error
            } else {
                record_no_effect(&mut store, &operation_id, &request_digest, error)
            };
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    error
                },
                &request.operation_id,
            ));
        }
    };
    if let Err(error) = shard.release(&mut store) {
        let error = map_search_error(error);
        let _ = store.mark_outcome_unknown(&operation_id, &request_digest);
        let closed = project_store.close();
        return Err(operation_failure(
            if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                error
            },
            &request.operation_id,
        ));
    }

    if let Err(error) = check_cancelled(stop) {
        let closed = project_store.close();
        return Err(operation_failure(
            if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                record_no_effect(&mut store, &operation_id, &request_digest, error)
            },
            &request.operation_id,
        ));
    }

    let payload = DurableRetirePayload {
        status: "retired",
        resource_closure: "closed",
        source_store_generation_id: &request.store_generation_id,
        source_build_operation_id: binding.build_operation_id.as_str(),
        source_store_epoch_id: source_hold.epoch_id().as_str(),
        source_generation_hold_id: source_hold.root_id().as_str(),
        source_generation_hold_digest: source_hold.pin_digest(),
        logical_shard_id: &request.logical_shard_id,
        object_id: object_id.as_str(),
        search_profile_digest: profile.digest(),
        privacy_scope: "local_metadata_only",
        document_component: profile.component(),
    };
    let response_bytes = match durable_response(
        "search_index_retire",
        &request.operation_id,
        &digest_bytes,
        &payload,
        request.max_output_bytes.get(),
    ) {
        Ok(bytes) => bytes,
        Err(error) => {
            let closed = project_store.close();
            return Err(operation_failure(
                if closed.is_err() {
                    source_hold_outcome_unknown(&source_hold)
                } else {
                    record_no_effect(&mut store, &operation_id, &request_digest, error)
                },
                &request.operation_id,
            ));
        }
    };
    let mut batch = WriteBatch::new();
    if let Err(error) =
        batch.add_catalog_mutation(CatalogMutation::delete(catalog, path, object_id))
    {
        let closed = project_store.close();
        return Err(operation_failure(
            if closed.is_err() {
                source_hold_outcome_unknown(&source_hold)
            } else {
                record_no_effect(
                    &mut store,
                    &operation_id,
                    &request_digest,
                    map_store_error(error),
                )
            },
            &request.operation_id,
        ));
    }
    match complete_durable_operation(
        &mut store,
        &operation_id,
        &request_digest,
        batch,
        &response_bytes,
        request.max_output_bytes.get(),
    ) {
        Ok(bytes) => {
            let release = release_source_hold(&mut project_store, &source_hold, stop, false);
            let closed = project_store.close();
            match (release, closed) {
                (Ok(()), Ok(())) => Ok(bytes),
                (Err(_), _) | (_, Err(_)) => Err(source_hold_outcome_unknown(&source_hold)),
            }
        }
        Err(error) => {
            let operation = match store.operation(&operation_id) {
                Ok(Some(operation)) => operation,
                _ => {
                    let _ = project_store.close();
                    return Err(source_hold_outcome_unknown(&source_hold));
                }
            };
            match operation.state() {
                StoreOperationState::Completed => {
                    let bytes = match read_completed_response(
                        &store,
                        &operation,
                        request.max_output_bytes.get(),
                    ) {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            let closed = project_store.close();
                            return Err(if closed.is_err() {
                                source_hold_outcome_unknown(&source_hold)
                            } else {
                                error
                            });
                        }
                    };
                    let outcome = reconcile_completed_retire(
                        &mut store,
                        &mut project_store,
                        request,
                        &source_hold,
                        &request_digest,
                        bytes,
                        stop,
                    );
                    let closed = project_store.close();
                    match (outcome, closed) {
                        (Ok(bytes), Ok(())) => Ok(bytes),
                        (Err(error), Ok(())) => Err(error),
                        (_, Err(_)) => Err(source_hold_outcome_unknown(&source_hold)),
                    }
                }
                StoreOperationState::NoEffect | StoreOperationState::Failed => {
                    let closed = project_store.close();
                    if closed.is_err() {
                        Err(source_hold_outcome_unknown(&source_hold))
                    } else {
                        Err(operation_failure(error, &request.operation_id))
                    }
                }
                StoreOperationState::Prepared | StoreOperationState::OutcomeUnknown => {
                    let _ = project_store.close();
                    Err(source_hold_outcome_unknown(&source_hold))
                }
            }
        }
    }
}

fn durable_collect_garbage(
    request: &SearchStoreGcRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let target_object_id =
        wow_store::ObjectId::new(request.target_object_id.to_string()).map_err(map_store_error)?;
    let input = DurableGcInput {
        target_object_id: &request.target_object_id,
        store_limits: request.store_limits,
        now: request.now,
        max_deletes: request.max_deletes,
        max_output_bytes: request.max_output_bytes.get(),
    };
    let (operation_id, request_digest, digest_bytes) = durable_identity(
        "wow-service/search-store-gc-request/1",
        &request.operation_id,
        &input,
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    let outer_start = start_durable_gc_outer_operation(
        &mut store,
        &operation_id,
        &request_digest,
        request.max_output_bytes.get(),
    )?;
    let outer_prepared = matches!(&outer_start, DurableGcOuterStart::Prepared);
    match outer_start {
        DurableGcOuterStart::Replay(bytes) => return Ok(bytes),
        DurableGcOuterStart::Released => {
            return Err(ServiceError::new(
                ServiceErrorCode::OperationReleased,
                "exact Search GC operation has a recorded noncompleted outcome",
            ));
        }
        DurableGcOuterStart::Started | DurableGcOuterStart::Prepared => {}
    }
    let (inner_operation_id, inner_request_digest) =
        durable_gc_inner_identity(request, &operation_id, &request_digest)?;
    let inner_record = match store.operation(&inner_operation_id) {
        Ok(record) => record,
        Err(_) => return Err(operation_failure(outcome_unknown(), &request.operation_id)),
    };
    let expected_replayed = match (outer_prepared, inner_record.as_ref()) {
        (false, None) => false,
        (_, Some(record))
            if record.request_digest() == &inner_request_digest
                && record.state() == StoreOperationState::Completed =>
        {
            true
        }
        // A prepared outer request without an atomic inner receipt may have
        // reached the old unjournaled GC path. Never invoke deletion again.
        _ => {
            return Err(operation_failure(outcome_unknown(), &request.operation_id));
        }
    };
    if !expected_replayed {
        let preflight_payload = DurableGcPayload {
            status: "collected",
            target_object_id: target_object_id.as_str(),
            target_object_deleted: false,
            deleted_object_count: u32::MAX,
            more_available: false,
        };
        let preflight_bytes = match durable_response(
            "search_store_gc",
            &request.operation_id,
            &digest_bytes,
            &preflight_payload,
            request.max_output_bytes.get(),
        ) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Err(operation_failure(
                    record_no_effect(&mut store, &operation_id, &request_digest, error),
                    &request.operation_id,
                ));
            }
        };
        if let Err(error) = response_object(&store, &preflight_bytes) {
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
        if let Err(error) = check_cancelled(stop) {
            return Err(operation_failure(
                record_no_effect(&mut store, &operation_id, &request_digest, error),
                &request.operation_id,
            ));
        }
    }
    let outcome = match store.collect_garbage_with_operation(
        &inner_operation_id,
        &inner_request_digest,
        LogicalEpoch::new(request.now),
        request.max_deletes,
    ) {
        Ok(outcome) => outcome,
        Err(_) => {
            let completed = store.operation(&inner_operation_id).ok().flatten();
            if completed.as_ref().is_some_and(|record| {
                record.request_digest() == &inner_request_digest
                    && record.state() == StoreOperationState::Completed
            }) {
                match store.collect_garbage_with_operation(
                    &inner_operation_id,
                    &inner_request_digest,
                    LogicalEpoch::new(request.now),
                    request.max_deletes,
                ) {
                    Ok(outcome) if outcome.replayed() => outcome,
                    _ => {
                        return Err(operation_failure(outcome_unknown(), &request.operation_id));
                    }
                }
            } else {
                return Err(operation_failure(outcome_unknown(), &request.operation_id));
            }
        }
    };
    if expected_replayed && !outcome.replayed() {
        return Err(operation_failure(outcome_unknown(), &request.operation_id));
    }
    let receipt = outcome.receipt();
    let deleted_count = match u32::try_from(receipt.deleted().len()) {
        Ok(count) => count,
        Err(_) => return Err(operation_failure(outcome_unknown(), &request.operation_id)),
    };
    let payload = DurableGcPayload {
        status: "collected",
        target_object_id: target_object_id.as_str(),
        target_object_deleted: receipt.deleted().contains(&target_object_id),
        deleted_object_count: deleted_count,
        more_available: receipt.more_available(),
    };
    let response_bytes = match durable_response(
        "search_store_gc",
        &request.operation_id,
        &digest_bytes,
        &payload,
        request.max_output_bytes.get(),
    ) {
        Ok(bytes) => bytes,
        Err(_) => return Err(operation_failure(outcome_unknown(), &request.operation_id)),
    };
    complete_durable_gc_outer_operation(
        &mut store,
        &operation_id,
        &request_digest,
        &response_bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|_| operation_failure(outcome_unknown(), &request.operation_id))
}

fn lease_id_for(prefix: &str, digest: &str) -> ServiceResult<LeaseId> {
    let hex = digest.strip_prefix("sha256:").ok_or_else(outcome_unknown)?;
    LeaseId::new(format!("{prefix}:{hex}")).map_err(map_store_error)
}

#[derive(Serialize)]
struct SearchRequestIdentity<'a> {
    operation_id: &'a str,
    store_generation_id: &'a str,
    file_id: &'a str,
    literal_query: &'a str,
    search_profile_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
}

#[derive(Serialize)]
struct SearchResultPayload {
    status: SearchBuildQueryStatus,
    resource_closure: &'static str,
    source_store_generation_id: Box<str>,
    logical_shard_id: Box<str>,
    search_profile_digest: Box<str>,
    entity_id: Box<str>,
    owner_metadata_label: Box<str>,
    exact_id_match: bool,
    exact_owner_metadata_label_matches: u32,
    literal_query_digest: Box<str>,
    literal_fts_candidates: Vec<SearchCandidateRecord>,
    privacy_scope: &'static str,
    license_state: &'static str,
}

#[derive(Serialize)]
struct SearchResultIdentity<'a> {
    request_digest: &'a str,
    payload: &'a SearchResultPayload,
}

#[derive(Serialize)]
struct SearchResultEnvelope<'a> {
    schema: &'static str,
    operation: &'static str,
    operation_id: &'a str,
    request_digest: &'a str,
    result_id: &'a str,
    payload: &'a SearchResultPayload,
}

fn execute_build_query(
    request: &SearchBuildQueryRequest,
    profile: &SearchBuildProfile,
    request_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<SearchBuildQueryResult> {
    check_cancelled(stop)?;
    let generation = StoreGenerationId::parse(request.store_generation_id.to_string())
        .map_err(map_store_error)?;
    let file_id = ProjectFileId::parse(request.file_id.as_ref()).map_err(map_project_error)?;
    let store_configuration =
        StoreConfiguration::new(SEARCH_STORE_PROFILE_ID, request.store_limits)
            .map_err(map_store_error)?;
    let mut search_store = Store::open_in_memory(store_configuration).map_err(map_store_error)?;
    let project_store = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &request.project_store_root,
    )?;
    let read = match project_store.read(&ReadSelector::Exact(generation.clone()), stop) {
        Ok(read) => read,
        Err(error) => {
            if project_store.close().is_err() {
                return Err(failure(
                    ServiceErrorCode::InternalContractViolation,
                    "project store close failed after exact read refusal",
                ));
            }
            return Err(map_store_error(error));
        }
    };

    let operation = execute_read_and_query(
        &read,
        &generation,
        &file_id,
        request,
        profile,
        request_digest,
        &mut search_store,
        stop,
    );
    // wow-store exposes checked close for the retained ProjectStore/read lease and
    // SearchShard lease release; its in-memory generic Store closes on drop.
    drop(search_store);
    let read_close = read.close();
    let store_close = project_store.close();
    if read_close.is_err() || store_close.is_err() {
        return Err(failure(
            ServiceErrorCode::InternalContractViolation,
            "search owner resource close failed",
        ));
    }
    check_cancelled(stop)?;
    operation
}

#[allow(clippy::too_many_arguments)]
fn execute_read_and_query(
    read: &wow_store::project::ReadSnapshot,
    generation: &StoreGenerationId,
    file_id: &ProjectFileId,
    request: &SearchBuildQueryRequest,
    profile: &SearchBuildProfile,
    request_digest: &str,
    search_store: &mut Store,
    stop: &AtomicBool,
) -> ServiceResult<SearchBuildQueryResult> {
    check_cancelled(stop)?;
    let lease_id = LeaseId::new(format!("search-{}", request.operation_id.as_str()))
        .map_err(map_store_error)?;
    let lease = SearchStoreLeaseRequest::new(
        lease_id,
        "wow-service-search",
        LogicalEpoch::new(request.lease_expires_after),
    )
    .map_err(map_store_error)?;
    let owner_request = SearchShardBuildRequest::new(
        generation.clone(),
        file_id.clone(),
        profile.clone(),
        request.permission,
        lease,
    );
    let shard = build_project_file_shard(read, owner_request, search_store, stop)
        .map_err(map_search_error)?;
    let query = (|| {
        check_cancelled(stop)?;
        let exact = shard
            .exact_id_lookup(file_id)
            .map_err(map_search_error)?
            .ok_or_else(|| {
                failure(
                    ServiceErrorCode::IdentityMismatch,
                    "exact owner ID lookup failed",
                )
            })?;
        let label = exact.document().owner_metadata_label().as_str();
        let exact_labels = shard.exact_label_lookup(label).map_err(map_search_error)?;
        if exact_labels.len() != 1
            || exact_labels[0].document().entity_id() != file_id
            || exact_labels[0].document().owner_metadata_label().as_str() != label
            || exact_labels[0].evidence() != SearchEvidenceClass::ExactOwnerMetadataLabel
        {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "exact owner metadata label lookup did not resolve the selected record",
            ));
        }
        let candidates = shard
            .literal_candidates(&request.literal_query)
            .map_err(map_search_error)?
            .into_iter()
            .map(|candidate| {
                if candidate.evidence() != SearchEvidenceClass::Candidate
                    || candidate.entity_id() != file_id
                    || candidate.owner_metadata_label().as_str() != label
                {
                    return Err(failure(
                        ServiceErrorCode::IdentityMismatch,
                        "literal FTS returned evidence outside the selected one-file shard",
                    ));
                }
                Ok(SearchCandidateRecord {
                    entity_id: candidate.entity_id().as_str().into(),
                    owner_metadata_label: candidate.owner_metadata_label().as_str().into(),
                    evidence: "candidate",
                })
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let payload = SearchResultPayload {
            status: SearchBuildQueryStatus::ExactFound,
            resource_closure: "closed",
            source_store_generation_id: generation.as_str().into(),
            logical_shard_id: shard.logical_id().into(),
            search_profile_digest: profile.digest().into(),
            entity_id: exact.document().entity_id().as_str().into(),
            owner_metadata_label: exact.document().owner_metadata_label().as_str().into(),
            exact_id_match: exact.evidence() == SearchEvidenceClass::ExactEntityIdentity,
            exact_owner_metadata_label_matches: u32::try_from(exact_labels.len()).map_err(
                |_| {
                    failure(
                        ServiceErrorCode::BudgetExceeded,
                        "label result count exceeded",
                    )
                },
            )?,
            literal_query_digest: canonical_digest(
                "wow-service/search-literal-query/1:",
                request.literal_query.as_ref(),
            )?,
            literal_fts_candidates: candidates,
            privacy_scope: "local_metadata_only",
            license_state: "not_evaluated",
        };
        let result_id = canonical_digest(
            "wow-service/search-build-query-result/1:",
            &SearchResultIdentity {
                request_digest,
                payload: &payload,
            },
        )?;
        let bytes = canonical_json_bytes(&SearchResultEnvelope {
            schema: SEARCH_RESULT_SCHEMA,
            operation: "search_build_query",
            operation_id: request.operation_id.as_str(),
            request_digest,
            result_id: &result_id,
            payload: &payload,
        })
        .map_err(|_| {
            failure(
                ServiceErrorCode::CanonicalizationFailed,
                "search result encoding failed",
            )
        })?;
        let byte_count = u64::try_from(bytes.len()).map_err(|_| {
            failure(
                ServiceErrorCode::BudgetExceeded,
                "search output size overflow",
            )
        })?;
        let framed_byte_count = byte_count.checked_add(1).ok_or_else(|| {
            failure(
                ServiceErrorCode::BudgetExceeded,
                "search output size overflow",
            )
        })?;
        if framed_byte_count > request.max_output_bytes.get() {
            return Err(failure(
                ServiceErrorCode::BudgetExceeded,
                "search result exceeds the caller output bound",
            ));
        }
        Ok(SearchBuildQueryResult {
            result_id,
            canonical_bytes: bytes.into_boxed_slice(),
        })
    })();
    let released = shard.release(search_store);
    if released.is_err() {
        return Err(failure(
            ServiceErrorCode::InternalContractViolation,
            "search shard lease close failed",
        ));
    }
    query
}

fn map_search_error(error: SearchError) -> ServiceError {
    match error {
        SearchError::Store(error) => map_store_error(error),
        SearchError::Project(error) => map_project_error(error),
        SearchError::Cancelled => {
            failure(ServiceErrorCode::Cancelled, "search operation cancelled")
        }
        SearchError::InvalidProfile | SearchError::LocalMetadataPermissionRequired => failure(
            ServiceErrorCode::InvalidRequest,
            "search profile or permission is invalid",
        ),
        SearchError::FileSetEmpty | SearchError::FileSetDuplicate => failure(
            ServiceErrorCode::InvalidRequest,
            "explicit ProjectFile set is empty or contains duplicate IDs",
        ),
        SearchError::FileSetOverCapacity | SearchError::FileSetBudgetExceeded => failure(
            ServiceErrorCode::BudgetExceeded,
            "ProjectFile set exceeds its selected bound",
        ),
        SearchError::OwnerEvidenceInvalid => failure(
            ServiceErrorCode::IdentityMismatch,
            "typed owner evidence failed validation",
        ),
        SearchError::ShardLeaseCleanupOutcomeUnknown {
            lease_id,
            holder,
            object_id,
            shard_id,
            expires_after,
            release_error,
        } => ServiceError::new(
            ServiceErrorCode::StoreOutcomeUnknown,
            format!(
                "search shard lease cleanup outcome is unknown; reconcile lease={} holder={} object={} shard={} expires_after={} release_error={release_error:?}",
                lease_id.as_str(),
                holder,
                object_id.as_str(),
                shard_id,
                expires_after.get(),
            ),
        ),
        SearchError::SelectedFileUnavailable => failure(
            ServiceErrorCode::ExactGenerationUnavailable,
            "selected ProjectFile is absent from the exact generation",
        ),
    }
}

fn map_store_error(error: StoreError) -> ServiceError {
    let code = match error.code() {
        StoreErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        StoreErrorCode::OutcomeUnknown => {
            return ServiceError::new(
                ServiceErrorCode::StoreOutcomeUnknown,
                format!("search Store outcome is unknown: {}", error.message()),
            );
        }
        StoreErrorCode::OperationConflict => ServiceErrorCode::OperationConflict,
        StoreErrorCode::BudgetExceeded | StoreErrorCode::ObjectTooLarge => {
            ServiceErrorCode::BudgetExceeded
        }
        StoreErrorCode::GenerationMissing | StoreErrorCode::ObjectMissing => {
            ServiceErrorCode::ExactGenerationUnavailable
        }
        StoreErrorCode::IntegrityViolation => ServiceErrorCode::IdentityMismatch,
        StoreErrorCode::StoreInstanceIdentityUnavailable => ServiceErrorCode::ComponentUnavailable,
        StoreErrorCode::StoreInstanceIdentityMismatch => ServiceErrorCode::IdentityMismatch,
        StoreErrorCode::ConfigurationInvalid | StoreErrorCode::IdentifierInvalid => {
            ServiceErrorCode::InvalidRequest
        }
        _ => ServiceErrorCode::ComponentUnavailable,
    };
    ServiceError::new(
        code,
        format!("exact search store operation failed ({:?})", error.code()),
    )
}

fn map_project_error(error: wow_project::ProjectError) -> ServiceError {
    let code = match error.code() {
        ProjectErrorCode::AnalysisCancelled => ServiceErrorCode::Cancelled,
        ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        ProjectErrorCode::StoreReadUnavailable => ServiceErrorCode::ComponentUnavailable,
        _ => ServiceErrorCode::IdentityMismatch,
    };
    failure(code, "exact Project/Graph owner read failed")
}

fn check_cancelled(stop: &AtomicBool) -> ServiceResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(
            ServiceErrorCode::Cancelled,
            "search operation cancelled",
        ))
    } else {
        Ok(())
    }
}

fn failure(code: ServiceErrorCode, message: &'static str) -> ServiceError {
    ServiceError::new(code, message)
}

#[cfg(test)]
mod durable_source_retention_tests {
    use std::{
        collections::BTreeSet,
        error::Error,
        num::{NonZeroU32, NonZeroU64},
        sync::atomic::AtomicBool,
    };

    use serde_json::Value;
    use wow_store::project::ProjectGcPolicy;

    use super::*;
    use crate::live_project::{LiveProjectStore, tests::owners};

    #[test]
    fn prepared_build_reconciles_persistent_source_hold_through_exact_retirement()
    -> Result<(), Box<dyn Error>> {
        assert_source_hold_lifecycle(false)
    }

    #[test]
    fn prepared_file_set_build_reconciles_source_holds_and_exact_retirement()
    -> Result<(), Box<dyn Error>> {
        assert_source_hold_lifecycle(true)
    }

    fn assert_source_hold_lifecycle(file_set: bool) -> Result<(), Box<dyn Error>> {
        let stop = AtomicBool::new(false);
        let root = std::env::temp_dir().join(format!(
            "wow-search-source-hold-{}-{}-{}",
            std::process::id(),
            file_set,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&root)?;
        let project_root = root.join("project-store");
        let search_path = root.join("search-store.sqlite");

        let owner = |source: &str| {
            if file_set {
                crate::live_project::tests::owners_with_files(vec![
                    wow_project::ProjectInputFile::new("main.lua", source)?,
                    wow_project::ProjectInputFile::new("other.lua", "local other = true")?,
                ])
            } else {
                owners(source)
            }
        };
        let (publisher_a, graph_a) = owner("return External()")?;
        let (publisher_b, graph_b) = owner("return External() + 1")?;
        assert_eq!(graph_a.snapshot().universe(), graph_b.snapshot().universe());
        let mut project =
            LiveProjectStore::create(&project_root, graph_a.snapshot().universe().as_str())?;
        let publication_a = project.publish(
            &publisher_a,
            &graph_a,
            "fixture:search-source-hold-a",
            None,
            &stop,
        )?;
        let activation_a = publication_a
            .activation
            .clone()
            .ok_or("missing first source activation")?;
        let publication_b = project.publish(
            &publisher_b,
            &graph_b,
            "fixture:search-source-hold-b",
            Some(activation_a.record_id.clone()),
            &stop,
        )?;
        project.release_publication(
            "fixture:search-source-hold-a",
            &publication_a.canonical_digest()?,
            "fixture:search-retention-test",
            &stop,
        )?;
        project.release_publication(
            "fixture:search-source-hold-b",
            &publication_b.canonical_digest()?,
            "fixture:search-retention-test",
            &stop,
        )?;
        let gc_policy = ProjectGcPolicy::new(
            "fixture:search-source-hold-gc",
            BTreeSet::new(),
            1024,
            8192,
            64 * 1024 * 1024,
        )?;
        project.select_gc_policy(&gc_policy, None, &stop)?;
        let unheld = project.plan_gc(&gc_policy, &stop)?;
        assert!(
            unheld
                .report()
                .delete_generations()
                .contains(&publication_a.generation_id)
        );
        project.close()?;

        let bounds = SearchBounds::new(
            NonZeroU32::new(if file_set { 2 } else { 1 }).ok_or("zero document bound")?,
            NonZeroU32::new(256).ok_or("zero document ID bound")?,
            NonZeroU32::new(1024).ok_or("zero label bound")?,
            NonZeroU32::new(128).ok_or("zero literal bound")?,
            NonZeroU32::new(32).ok_or("zero candidate bound")?,
            NonZeroU32::new(128).ok_or("zero evidence bound")?,
            NonZeroU64::new(1_048_576).ok_or("zero manifest bound")?,
            NonZeroU64::new(1_048_576).ok_or("zero database bound")?,
            NonZeroU64::new(3_145_728).ok_or("zero search object bound")?,
        )?;
        let limits = SearchStoreLimits::new(4_194_304, 8, 8, 256, 16)?;
        let max_output_bytes = NonZeroU64::new(65_536).ok_or("invalid output bound")?;
        let build_operation_id = OperationId::new("fixture:search-source-hold-build")?;
        let search_path_b = root.join("search-store-b.sqlite");
        let make_build = |search_store_path| -> ServiceResult<SearchIndexBuildRequest> {
            if file_set {
                SearchIndexBuildRequest::new_file_set(
                    build_operation_id.clone(),
                    project_root.clone(),
                    search_store_path,
                    publication_a.generation_id.as_str(),
                    vec![
                        "project-file:other.lua".into(),
                        "project-file:main.lua".into(),
                    ],
                    bounds,
                    limits,
                    max_output_bytes,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                    &stop,
                )
            } else {
                SearchIndexBuildRequest::new(
                    build_operation_id.clone(),
                    project_root.clone(),
                    search_store_path,
                    publication_a.generation_id.as_str(),
                    "project-file:main.lua",
                    bounds,
                    limits,
                    max_output_bytes,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                )
            }
        };
        let build_request = make_build(search_path.clone())?;
        let build_request_b = make_build(search_path_b.clone())?;
        let profile = DurableProfile::new(bounds, file_set)?;
        let file_id = ProjectFileId::parse("project-file:main.lua")?;
        let documents = build_request.selection.normalize()?;
        let make_retire = |operation: &str,
                           path: PathBuf,
                           binding: SearchSourceHoldBinding,
                           payload: &Value,
                           expires|
         -> Result<SearchShardRetireRequest, Box<dyn Error>> {
            let logical = payload["logical_shard_id"].as_str().ok_or("shard ID")?;
            let object = payload["object_id"].as_str().ok_or("object ID")?;
            Ok(if file_set {
                SearchShardRetireRequest::new_file_set(
                    OperationId::new(operation)?,
                    path,
                    binding,
                    publication_a.generation_id.as_str(),
                    vec![
                        "project-file:main.lua".into(),
                        "project-file:other.lua".into(),
                    ],
                    logical,
                    object,
                    profile.digest(),
                    bounds,
                    limits,
                    max_output_bytes,
                    expires,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                    &stop,
                )?
            } else {
                SearchShardRetireRequest::new(
                    OperationId::new(operation)?,
                    path,
                    binding,
                    publication_a.generation_id.as_str(),
                    file_id.as_str(),
                    logical,
                    object,
                    profile.digest(),
                    bounds,
                    limits,
                    max_output_bytes,
                    expires,
                    LocalMetadataPermission::authorize_local_metadata_only(),
                )?
            })
        };

        let search_store_a = open_durable_search_store(&search_path, limits)?;
        let search_store_instance_id_a = search_store_a.store_instance_id()?.clone();
        drop(search_store_a);
        let search_store_b = open_durable_search_store(&search_path_b, limits)?;
        let search_store_instance_id_b = search_store_b.store_instance_id()?.clone();
        drop(search_store_b);
        assert_ne!(search_store_instance_id_a, search_store_instance_id_b);

        let source_store =
            crate::live_project::LiveProjectStore::open_retained_owner_store(&project_root)?;
        let source_hold_a = search_source_hold_root(
            &source_store,
            &search_store_instance_id_a,
            &build_operation_id,
            &publication_a.generation_id,
            &documents,
            &profile,
            limits,
        )?;
        let source_hold_b = search_source_hold_root(
            &source_store,
            &search_store_instance_id_b,
            &build_operation_id,
            &publication_a.generation_id,
            &documents,
            &profile,
            limits,
        )?;
        assert_ne!(source_hold_a.root_id(), source_hold_b.root_id());
        let (store_operation_id_a, request_digest_a, _) =
            durable_build_identity(&build_request, &profile, &source_hold_a)?;
        let (store_operation_id_b, request_digest_b, _) =
            durable_build_identity(&build_request_b, &profile, &source_hold_b)?;
        assert_eq!(store_operation_id_a, store_operation_id_b);
        assert_ne!(request_digest_a, request_digest_b);
        source_store.close()?;

        // Reproduce the process stop after the exact operation is Prepared and
        // both independent SearchStores and their persistent ProjectStore holds
        // are committed, before either Search publication.
        let mut search_store = open_durable_search_store(&search_path, limits)?;
        assert!(matches!(
            search_store.begin_operation(store_operation_id_a, request_digest_a)?,
            StoreOperationBegin::Started(_)
        ));
        drop(search_store);
        let mut search_store = open_durable_search_store(&search_path_b, limits)?;
        assert!(matches!(
            search_store.begin_operation(store_operation_id_b, request_digest_b)?,
            StoreOperationBegin::Started(_)
        ));
        drop(search_store);
        let mut source_store =
            crate::live_project::LiveProjectStore::open_retained_owner_store(&project_root)?;
        assert_eq!(
            source_store.put_retention_root(&source_hold_a, &stop)?,
            source_hold_a
        );
        assert_eq!(
            source_store.put_retention_root(&source_hold_b, &stop)?,
            source_hold_b
        );
        source_store.close()?;

        let service = SearchOperationService::new();
        let built_a = service.build_shard(&build_request, &stop)?;
        let built_b = service.build_shard(&build_request_b, &stop)?;
        let build_result_a: Value = serde_json::from_slice(&built_a)?;
        let build_result_b: Value = serde_json::from_slice(&built_b)?;
        let build_payload_a = &build_result_a["payload"];
        let build_payload_b = &build_result_b["payload"];
        if file_set {
            assert_eq!(
                build_payload_a["document_component"],
                documents::FILE_SET_COMPONENT
            );
            assert_eq!(
                build_payload_a["documents"]
                    .as_array()
                    .ok_or("full set documents")?
                    .len(),
                2
            );
            assert!(build_payload_a.get("entity_id").is_none());
        }
        assert_eq!(
            build_payload_a["source_store_epoch_id"],
            source_hold_a.epoch_id().as_str()
        );
        assert_eq!(
            build_payload_a["source_generation_hold_id"],
            source_hold_a.root_id().as_str()
        );
        assert_eq!(
            build_payload_a["source_generation_hold_digest"],
            source_hold_a.pin_digest()
        );
        assert_eq!(
            build_payload_b["source_generation_hold_id"],
            source_hold_b.root_id().as_str()
        );
        assert_ne!(
            build_payload_a["source_generation_hold_id"],
            build_payload_b["source_generation_hold_id"]
        );
        assert_eq!(
            build_payload_a["logical_shard_id"],
            build_payload_b["logical_shard_id"]
        );
        assert_eq!(build_payload_a["object_id"], build_payload_b["object_id"]);
        assert_eq!(
            service.build_shard(&build_request, &stop)?,
            built_a,
            "SearchStore A build replay must return its exact stored response"
        );
        assert_eq!(
            service.build_shard(&build_request_b, &stop)?,
            built_b,
            "SearchStore B build replay must return its exact stored response"
        );

        let project = LiveProjectStore::open(&project_root)?;
        let roots = project.retention_roots(&stop)?;
        assert!(roots.contains(&source_hold_a));
        assert!(roots.contains(&source_hold_b));
        let held_plan = project.plan_gc(&gc_policy, &stop)?;
        assert!(
            held_plan
                .report()
                .protected_generations()
                .contains(&publication_a.generation_id)
        );
        assert!(
            !held_plan
                .report()
                .delete_generations()
                .contains(&publication_a.generation_id)
        );
        project.close()?;

        let source_binding_a = SearchSourceHoldBinding::new(
            project_root.clone(),
            build_operation_id.clone(),
            source_hold_a.epoch_id().as_str(),
            source_hold_a.root_id().as_str(),
            source_hold_a.pin_digest(),
        )?;
        if file_set {
            // A genuine hold for a different complete selection cannot retire
            // this shard, even when its epoch/build operation/profile match.
            let smaller_request = SearchIndexBuildRequest::new_file_set(
                build_operation_id.clone(),
                project_root.clone(),
                search_path.clone(),
                publication_a.generation_id.as_str(),
                vec![file_id.as_str().into()],
                bounds,
                limits,
                max_output_bytes,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &stop,
            )?;
            let smaller_documents = smaller_request.selection.normalize()?;
            let mut source_store = LiveProjectStore::open_retained_owner_store(&project_root)?;
            let wrong_hold = search_source_hold_root(
                &source_store,
                &search_store_instance_id_a,
                &build_operation_id,
                &publication_a.generation_id,
                &smaller_documents,
                &profile,
                limits,
            )?;
            source_store.put_retention_root(&wrong_hold, &stop)?;
            source_store.close()?;
            let wrong_binding = SearchSourceHoldBinding::new(
                project_root.clone(),
                build_operation_id.clone(),
                wrong_hold.epoch_id().as_str(),
                wrong_hold.root_id().as_str(),
                wrong_hold.pin_digest(),
            )?;
            let wrong_retire = SearchShardRetireRequest::new_file_set(
                OperationId::new("fixture:wrong-set-retirement")?,
                search_path.clone(),
                wrong_binding,
                publication_a.generation_id.as_str(),
                vec![file_id.as_str().into()],
                build_payload_a["logical_shard_id"]
                    .as_str()
                    .ok_or("shard")?,
                build_payload_a["object_id"].as_str().ok_or("object")?,
                profile.digest(),
                bounds,
                limits,
                max_output_bytes,
                10,
                LocalMetadataPermission::authorize_local_metadata_only(),
                &stop,
            )?;
            assert_eq!(
                service
                    .retire_shard(&wrong_retire, &stop)
                    .err()
                    .ok_or("wrong set accepted")?
                    .code(),
                ServiceErrorCode::IdentityMismatch
            );
            let mut source_store = LiveProjectStore::open_retained_owner_store(&project_root)?;
            assert!(
                source_store
                    .retention_roots(&stop)?
                    .contains(&source_hold_a)
            );
            source_store.remove_retention_root(
                wrong_hold.root_id(),
                wrong_hold.pin_digest(),
                &stop,
            )?;
            source_store.close()?;
        }
        let retire_request_a = make_retire(
            "fixture:search-source-hold-retire-a",
            search_path.clone(),
            source_binding_a,
            build_payload_a,
            10,
        )?;
        let retired_a = service.retire_shard(&retire_request_a, &stop)?;
        let retire_result_a: Value = serde_json::from_slice(&retired_a)?;
        assert_eq!(retire_result_a["payload"]["status"], "retired");
        assert_eq!(
            service.retire_shard(&retire_request_a, &stop)?,
            retired_a,
            "SearchStore A retirement replay must return its exact stored response"
        );

        let mut project = LiveProjectStore::open(&project_root)?;
        let roots_after_a = project.retention_roots(&stop)?;
        assert!(!roots_after_a.contains(&source_hold_a));
        assert!(roots_after_a.contains(&source_hold_b));
        let held_for_b_plan = project.plan_gc(&gc_policy, &stop)?;
        assert!(
            held_for_b_plan
                .report()
                .protected_generations()
                .contains(&publication_a.generation_id)
        );
        assert!(
            !held_for_b_plan
                .report()
                .delete_generations()
                .contains(&publication_a.generation_id)
        );
        project.execute_gc(
            &held_for_b_plan,
            "fixture:search-source-hold-gc-after-a",
            &stop,
        )?;
        project.close()?;
        let source_owner = LiveProjectStore::open_retained_owner_store(&project_root)?;
        let rebuilt_from_b_hold = prepare_exact_documents_from_store(
            &source_owner,
            publication_a.generation_id.clone(),
            &documents,
            &profile,
            LocalMetadataPermission::authorize_local_metadata_only(),
            limits,
            &stop,
        )?;
        assert_eq!(
            rebuilt_from_b_hold.logical_id(),
            build_payload_b["logical_shard_id"]
                .as_str()
                .ok_or("missing B shard ID")?
        );
        assert_eq!(
            rebuilt_from_b_hold.object_id().as_str(),
            build_payload_b["object_id"]
                .as_str()
                .ok_or("missing B object ID")?
        );
        source_owner.close()?;

        assert_eq!(
            service.build_shard(&build_request_b, &stop)?,
            built_b,
            "SearchStore B must reopen and reconcile after A retirement and source GC"
        );
        let query_request_b = SearchShardQueryRequest::new(
            OperationId::new("fixture:search-source-hold-query-b")?,
            search_path_b.clone(),
            publication_a.generation_id.as_str(),
            file_id.as_str(),
            build_payload_b["logical_shard_id"]
                .as_str()
                .ok_or("missing B shard ID")?,
            build_payload_b["object_id"]
                .as_str()
                .ok_or("missing B object ID")?,
            profile.digest(),
            if file_set { "lua" } else { "External" },
            bounds,
            limits,
            max_output_bytes,
            11,
            LocalMetadataPermission::authorize_local_metadata_only(),
        )?;
        let query_request_b = if file_set {
            query_request_b.with_file_set_component()
        } else {
            query_request_b
        };
        let queried_b = service.query_shard(&query_request_b, &stop)?;
        let query_result_b: Value = serde_json::from_slice(&queried_b)?;
        assert_eq!(query_result_b["payload"]["resource_closure"], "closed");
        if file_set {
            assert_eq!(
                query_result_b["payload"]["literal_fts_candidates"]
                    .as_array()
                    .ok_or("set candidates")?
                    .len(),
                2
            );
        }
        assert_eq!(
            query_result_b["payload"]["source_store_generation_id"],
            publication_a.generation_id.as_str()
        );

        let source_binding_b = SearchSourceHoldBinding::new(
            project_root.clone(),
            build_operation_id.clone(),
            source_hold_b.epoch_id().as_str(),
            source_hold_b.root_id().as_str(),
            source_hold_b.pin_digest(),
        )?;
        let retire_request_b = make_retire(
            "fixture:search-source-hold-retire-b",
            search_path_b,
            source_binding_b,
            build_payload_b,
            12,
        )?;
        let retired_b = service.retire_shard(&retire_request_b, &stop)?;
        let retire_result_b: Value = serde_json::from_slice(&retired_b)?;
        assert_eq!(retire_result_b["payload"]["status"], "retired");
        assert_eq!(
            service.retire_shard(&retire_request_b, &stop)?,
            retired_b,
            "SearchStore B retirement replay must return its exact stored response"
        );

        let mut project = LiveProjectStore::open(&project_root)?;
        let roots_after_b = project.retention_roots(&stop)?;
        assert!(!roots_after_b.contains(&source_hold_a));
        assert!(!roots_after_b.contains(&source_hold_b));
        let released_plan = project.plan_gc(&gc_policy, &stop)?;
        assert!(
            released_plan
                .report()
                .delete_generations()
                .contains(&publication_a.generation_id)
        );
        project.execute_gc(
            &released_plan,
            "fixture:search-source-hold-gc-after-b",
            &stop,
        )?;
        project.close()?;

        assert_eq!(
            service.build_shard(&build_request, &stop)?,
            built_a,
            "replaying retired A must not resurrect its source generation hold"
        );
        assert_eq!(
            service.build_shard(&build_request_b, &stop)?,
            built_b,
            "replaying retired B must not resurrect its source generation hold"
        );
        std::fs::remove_dir_all(&root)?;
        Ok(())
    }

    #[test]
    fn prepared_gc_outer_replays_native_receipt_without_collecting_again()
    -> Result<(), Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-search-gc-reconcile-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&root)?;
        let search_path = root.join("search-store.sqlite");
        let store_limits = SearchStoreLimits::new(4_194_304, 8, 8, 256, 16)?;
        let max_output_bytes = NonZeroU64::new(65_536).ok_or("invalid output bound")?;
        let target_object = wow_store::PendingObject::from_json(
            "search-gc-orphan",
            1,
            &serde_json::json!({ "label": "collected-before-outer-completion" }),
            wow_store::StoreLimits::new(4_194_304, 8, 8, 256, 16)?,
        )?;
        let target_object_id = target_object.object_id().clone();
        let request = SearchStoreGcRequest::new(
            OperationId::new("fixture:search-store-gc-reconcile")?,
            search_path.clone(),
            target_object_id.as_str(),
            store_limits,
            10,
            4,
            max_output_bytes,
        )?;
        let input = DurableGcInput {
            target_object_id: &request.target_object_id,
            store_limits: request.store_limits,
            now: request.now,
            max_deletes: request.max_deletes,
            max_output_bytes: request.max_output_bytes.get(),
        };
        let (outer_id, outer_digest, _) = durable_identity(
            "wow-service/search-store-gc-request/1",
            &request.operation_id,
            &input,
        )?;
        let (inner_id, inner_digest) =
            durable_gc_inner_identity(&request, &outer_id, &outer_digest)?;
        let mut store = open_durable_search_store(&search_path, store_limits)?;
        assert!(matches!(
            store.begin_operation(outer_id.clone(), outer_digest.clone())?,
            StoreOperationBegin::Started(_)
        ));
        let mut batch = wow_store::WriteBatch::new();
        batch.add_object(target_object)?;
        store.commit(batch)?;
        let native = store.collect_garbage_with_operation(
            &inner_id,
            &inner_digest,
            LogicalEpoch::new(request.now),
            request.max_deletes,
        )?;
        assert!(!native.replayed());
        assert!(native.receipt().deleted().contains(&target_object_id));
        assert!(store.object(&target_object_id)?.is_none());

        let survivor = wow_store::PendingObject::from_json(
            "search-gc-orphan",
            1,
            &serde_json::json!({ "label": "must-survive-native-receipt-replay" }),
            store.configuration().limits(),
        )?;
        let survivor_id = survivor.object_id().clone();
        let mut batch = wow_store::WriteBatch::new();
        batch.add_object(survivor)?;
        store.commit(batch)?;
        assert!(store.object(&survivor_id)?.is_some());
        drop(store);

        let service = SearchOperationService::new();
        let stopped = AtomicBool::new(true);
        let response = service.collect_search_garbage(&request, &stopped)?;
        let envelope: Value = serde_json::from_slice(&response)?;
        assert_eq!(envelope["payload"]["deleted_object_count"], 1);
        assert!(
            envelope["payload"]["target_object_deleted"]
                .as_bool()
                .ok_or("missing target deletion receipt")?
        );
        assert_eq!(
            service.collect_search_garbage(&request, &AtomicBool::new(false))?,
            response,
            "completed GC retry must return the exact stored outer response"
        );

        let store = open_durable_search_store(&search_path, store_limits)?;
        assert!(
            store.object(&survivor_id)?.is_some(),
            "reconciliation must replay the completed native receipt without collecting later objects"
        );
        drop(store);
        std::fs::remove_dir_all(&root)?;
        Ok(())
    }
}
