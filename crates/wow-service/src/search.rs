//! One-shot build-and-query service operation over one exact retained project generation.
//!
//! The first route returns an inline result only. It never returns a continuation,
//! and it requires explicit finite owner, Store, lease, and output bounds.

use std::{
    num::NonZeroU64,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use serde::Serialize;
use wow_core::canonical_json_bytes;
use wow_graph::GraphPartitionSnapshot;
use wow_project::{ProjectErrorCode, ProjectFileId, replay::publication};
use wow_search::{
    SearchBuildProfile, SearchError, SearchEvidenceClass, SearchShardBuildRequest,
    SearchStoreLeaseRequest, build_project_file_shard,
};
use wow_store::{
    LeaseId, LogicalEpoch, Store, StoreConfiguration, StoreError, StoreErrorCode,
    project::{ProjectStore, ReadSelector, RecordCatalog, StoreGenerationId},
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
        let request_digest = canonical_digest(
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
        )?;
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

        let outcome = execute_build_query(request, &profile, &request_digest, stop);
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
    let catalog = project_catalog()?;
    let project_store =
        ProjectStore::open(&request.project_store_root, &catalog).map_err(map_store_error)?;
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

fn project_catalog() -> ServiceResult<RecordCatalog> {
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(publication::STORAGE_SCHEMAS);
    RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )
    .map_err(map_store_error)
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
        SearchError::OwnerEvidenceInvalid => failure(
            ServiceErrorCode::IdentityMismatch,
            "typed owner evidence failed validation",
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
        StoreErrorCode::BudgetExceeded | StoreErrorCode::ObjectTooLarge => {
            ServiceErrorCode::BudgetExceeded
        }
        StoreErrorCode::GenerationMissing => ServiceErrorCode::ExactGenerationUnavailable,
        StoreErrorCode::IntegrityViolation => ServiceErrorCode::IdentityMismatch,
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
