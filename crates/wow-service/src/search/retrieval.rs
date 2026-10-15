//! Bound explicit retrieval over an exact retained file-set shard.
use super::*;
use wow_search::{
    RetrievalRequest, RetrievalState, SearchFileSetBuildProfile, SearchFileSetOpenRequest,
    open_project_file_set_shard,
};
use wow_store::ObjectId;

pub struct SearchRetrievalRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    generation: StoreGenerationId,
    logical_id: Box<str>,
    object_id: ObjectId,
    profile: SearchFileSetBuildProfile,
    query: RetrievalRequest,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}

impl SearchRetrievalRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        store_generation_id: &str,
        logical_id: &str,
        object_id: &str,
        profile_digest: &str,
        query_json: &[u8],
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store_path.is_absolute() || lease_expires_after == 0 {
            return Err(failure(
                ServiceErrorCode::InvalidRequest,
                "retrieval requires an absolute exact store and finite lease",
            ));
        }
        let profile = SearchFileSetBuildProfile::new(bounds).map_err(map_search_error)?;
        if profile.digest() != profile_digest {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "retrieval component/profile does not match the exact handle",
            ));
        }
        let query =
            RetrievalRequest::from_json_slice(query_json, stop).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            search_store_path,
            generation: StoreGenerationId::parse(store_generation_id.to_owned())
                .map_err(map_store_error)?,
            logical_id: logical_id.into(),
            object_id: ObjectId::new(object_id.to_owned()).map_err(map_store_error)?,
            profile,
            query,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}

#[derive(Serialize)]
struct Identity<'a> {
    generation: &'a str,
    logical_shard_id: &'a str,
    object_id: &'a str,
    profile_digest: &'a str,
    query: &'a RetrievalRequest,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
}

#[derive(Serialize)]
struct Payload {
    status: &'static str,
    resource_closure: &'static str,
    result_set_id: Box<str>,
    result_set: serde_json::Value,
}

pub(super) fn execute(
    request: &SearchRetrievalRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, request_digest, digest_bytes) = durable_identity(
        "wow-service/search-retrieval-request/1",
        &request.operation_id,
        &Identity {
            generation: request.generation.as_str(),
            logical_shard_id: &request.logical_id,
            object_id: request.object_id.as_str(),
            profile_digest: request.profile.digest(),
            query: &request.query,
            store_limits: request.store_limits,
            max_output_bytes: request.max_output_bytes.get(),
            lease_expires_after: request.lease_expires_after,
            privacy_scope: "local_metadata_only",
        },
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
    let outcome = (|| {
        let (catalog, path) = search_catalog_key(&request.logical_id)?;
        let entry = store
            .catalog_entry(&catalog, &path)
            .map_err(map_store_error)?
            .ok_or_else(|| {
                failure(
                    ServiceErrorCode::ExactGenerationUnavailable,
                    "exact retrieval shard is not retained",
                )
            })?;
        if entry.object_id() != &request.object_id {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "retrieval catalog names another immutable object",
            ));
        }
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("search-retrieval", &digest_bytes)?,
            "wow-service-search",
            LogicalEpoch::new(request.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_project_file_set_shard(
            &mut store,
            SearchFileSetOpenRequest::new(
                request.logical_id.clone(),
                request.object_id.clone(),
                request.generation.clone(),
                request.profile.clone(),
                request.permission,
                lease,
            ),
            stop,
        )
        .map_err(map_search_error)?;
        let query = shard
            .retrieve(&request.query, stop)
            .map_err(map_search_error);
        // Checked closure takes precedence over a query/serialization error.
        shard.release(&mut store).map_err(map_search_error)?;
        let result = query?;
        let status = match result.state() {
            RetrievalState::ExactFound => "exact_found",
            RetrievalState::ExactNotFoundPartial => "exact_not_found_partial",
            RetrievalState::CandidateOnly => "candidate_only",
            RetrievalState::NoCandidatesUnderExecutedLanes => "no_candidates_under_executed_lanes",
            RetrievalState::LaneUnavailable => "lane_unavailable",
            RetrievalState::Truncated => "truncated",
        };
        let payload = Payload {
            status,
            resource_closure: "closed",
            result_set_id: result.result_set_id().into(),
            result_set: serde_json::from_slice(result.canonical_bytes()).map_err(|_| {
                failure(
                    ServiceErrorCode::InternalContractViolation,
                    "owner result manifest could not be transported",
                )
            })?,
        };
        durable_response(
            "search_retrieve",
            &request.operation_id,
            &digest_bytes,
            &payload,
            request.max_output_bytes.get(),
        )
    })();
    let bytes = match outcome {
        Ok(bytes) => bytes,
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
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}
