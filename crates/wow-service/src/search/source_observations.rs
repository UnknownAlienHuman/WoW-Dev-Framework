//! Exact source-observation Search orchestration; no source acquisition on query.
use super::*;
use wow_search::source_observations::{
    ObservationQueryRequest, ObservationSearchProfile, open_source_observation_shard,
};
use wow_store::ObjectId;

impl SearchIndexBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new_source_observations(
        operation_id: OperationId,
        project_store_root: PathBuf,
        search_store_path: PathBuf,
        store_generation_id: impl Into<Box<str>>,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !project_store_root.is_absolute() || !search_store_path.is_absolute() {
            return Err(invalid());
        }
        ObservationSearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            project_store_root,
            search_store_path,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::SourceObservations,
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}
impl SearchShardRetireRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new_source_observations(
        operation_id: OperationId,
        search_store_path: PathBuf,
        source_hold: SearchSourceHoldBinding,
        store_generation_id: impl Into<Box<str>>,
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
            return Err(invalid());
        }
        ObservationSearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            search_store_path,
            source_hold,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::SourceObservations,
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
pub struct SourceObservationRetrievalRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    generation: StoreGenerationId,
    logical_id: Box<str>,
    object_id: ObjectId,
    profile: ObservationSearchProfile,
    query: ObservationQueryRequest,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}
impl SourceObservationRetrievalRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        generation: &str,
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
            return Err(invalid());
        }
        let profile = ObservationSearchProfile::new(bounds).map_err(map_search_error)?;
        if profile.digest() != profile_digest {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "source observation component/profile differs from handle",
            ));
        }
        search_catalog_key(logical_id)?;
        Ok(Self {
            operation_id,
            search_store_path,
            generation: StoreGenerationId::parse(generation.to_owned()).map_err(map_store_error)?,
            logical_id: logical_id.into(),
            object_id: ObjectId::new(object_id.to_owned()).map_err(map_store_error)?,
            profile,
            query: ObservationQueryRequest::from_json_slice(query_json, stop)
                .map_err(map_search_error)?,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}
pub(super) fn execute(
    request: &SourceObservationRetrievalRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/source-observation-retrieval/1",
        &request.operation_id,
        &(
            &request.generation,
            &request.logical_id,
            &request.object_id,
            request.profile.digest(),
            &request.query,
            request.store_limits,
            request.max_output_bytes.get(),
            request.lease_expires_after,
            "local_metadata_only",
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store_path, request.store_limits)?;
    match start_durable_operation(
        &mut store,
        &operation_id,
        &digest,
        request.max_output_bytes.get(),
    )? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let (catalog, path) = search_catalog_key(&request.logical_id)?;
        if store
            .catalog_entry(&catalog, &path)
            .map_err(map_store_error)?
            .is_none_or(|e| e.object_id() != &request.object_id)
        {
            return Err(failure(
                ServiceErrorCode::ExactGenerationUnavailable,
                "exact source observation shard is not retained",
            ));
        }
        let lease = SearchStoreLeaseRequest::new(
            lease_id_for("source-observation-search", &public_digest)?,
            "wow-service-search",
            LogicalEpoch::new(request.lease_expires_after),
        )
        .map_err(map_store_error)?;
        let shard = open_source_observation_shard(
            &mut store,
            &request.logical_id,
            &request.object_id,
            &request.generation,
            &request.profile,
            request.permission,
            lease,
            stop,
        )
        .map_err(map_search_error)?;
        let query = shard
            .retrieve(&request.query, stop)
            .map_err(map_search_error);
        shard.release(&mut store).map_err(map_search_error)?;
        let result = query?;
        check_cancelled(stop)?;
        batch
            .add_catalog_mutation(CatalogMutation::set(
                catalog,
                path,
                CatalogExpectation::Exact(request.object_id.clone()),
                request.object_id.clone(),
            ))
            .map_err(map_store_error)?;
        let value: serde_json::Value =
            serde_json::from_slice(result.canonical_bytes()).map_err(|_| invalid())?;
        durable_response(
            "search_source_observations",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":result.status(),"resource_closure":"closed","logical_shard_id":request.logical_id,
                "object_id":request.object_id,"profile_digest":request.profile.digest(),"result_set":value}),
            request.max_output_bytes.get(),
        )
    })();
    let bytes = match outcome.and_then(|bytes| {
        check_cancelled(stop)?;
        Ok(bytes)
    }) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(&operation_id, &digest);
                error
            } else {
                record_no_effect(&mut store, &operation_id, &digest, error)
            };
            return Err(operation_failure(error, &request.operation_id));
        }
    };
    complete_durable_operation(
        &mut store,
        &operation_id,
        &digest,
        batch,
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}
fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid exact source observation Search request",
    )
}
