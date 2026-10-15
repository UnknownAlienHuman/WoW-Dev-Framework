//! ProjectEntity consumers reuse the admitted durable Source-hold lifecycle.
use super::*;
use wow_search::entities::{EntityQueryRequest, EntitySearchProfile, open_project_entity_shard};
use wow_store::ObjectId;

impl SearchIndexBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new_graph_entities(
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
        EntitySearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            project_store_root,
            search_store_path,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::GraphEntities,
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}
impl SearchShardRetireRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new_graph_entities(
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
        EntitySearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            search_store_path,
            source_hold,
            store_generation_id: store_generation_id.into(),
            selection: DurableSelection::GraphEntities,
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
pub struct EntityRetrievalRequest {
    operation_id: OperationId,
    search_store_path: PathBuf,
    source_hold: SearchSourceHoldBinding,
    generation: StoreGenerationId,
    logical_id: Box<str>,
    object_id: ObjectId,
    profile: EntitySearchProfile,
    query: EntityQueryRequest,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}
impl EntityRetrievalRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store_path: PathBuf,
        source_hold: SearchSourceHoldBinding,
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
        let profile = EntitySearchProfile::new(bounds).map_err(map_search_error)?;
        if profile.digest() != profile_digest {
            return Err(failure(
                ServiceErrorCode::IdentityMismatch,
                "ProjectEntity component/profile differs from handle",
            ));
        }
        Ok(Self {
            operation_id,
            search_store_path,
            source_hold,
            generation: StoreGenerationId::parse(generation.to_owned()).map_err(map_store_error)?,
            logical_id: logical_id.into(),
            object_id: ObjectId::new(object_id.to_owned()).map_err(map_store_error)?,
            profile,
            query: EntityQueryRequest::from_json_slice(query_json, stop)
                .map_err(map_search_error)?,
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
    profile: &'a str,
    query: &'a EntityQueryRequest,
    build_operation_id: &'a str,
    source_epoch: &'a str,
    source_hold_id: &'a str,
    source_hold_digest: &'a str,
    store_limits: SearchStoreLimits,
    max_output_bytes: u64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
}
pub(super) fn execute(
    request: &EntityRetrievalRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let binding = &request.source_hold;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/project-entity-retrieval/1",
        &request.operation_id,
        &Identity {
            generation: request.generation.as_str(),
            logical_shard_id: &request.logical_id,
            object_id: request.object_id.as_str(),
            profile: request.profile.digest(),
            query: &request.query,
            build_operation_id: binding.build_operation_id.as_str(),
            source_epoch: &binding.source_store_epoch_id,
            source_hold_id: &binding.retention_root_id,
            source_hold_digest: &binding.retention_root_digest,
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
            .is_none_or(|entry| entry.object_id() != &request.object_id)
        {
            return Err(unavailable());
        }
        let source = crate::live_project::LiveProjectStore::open_retained_owner_store(
            &binding.project_store_root,
        )?;
        let source_result = (|| {
            let profile = DurableProfile::GraphEntities(request.profile.clone());
            let docs = Documents::GraphEntities;
            let hold = validate_source_hold_binding(
                &source,
                store.store_instance_id().map_err(map_store_error)?,
                binding,
                &request.generation,
                &docs,
                &profile,
                request.store_limits,
            )?;
            if !source_hold_present(&source, &hold, stop)? {
                return Err(unavailable());
            }
            let read = source
                .read(&ReadSelector::Exact(request.generation.clone()), stop)
                .map_err(map_store_error)?;
            let result = (|| {
                let pair = wow_project::replay::publication::AcquiredProjectPair::read(&read, stop)
                    .map_err(map_project_error)?;
                let lease = SearchStoreLeaseRequest::new(
                    lease_id_for("entity-search", &public_digest)?,
                    "wow-service-search",
                    LogicalEpoch::new(request.lease_expires_after),
                )
                .map_err(map_store_error)?;
                let shard = open_project_entity_shard(
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
                let result = shard
                    .retrieve(&read, &pair, &request.query, stop)
                    .map_err(map_search_error);
                shard.release(&mut store).map_err(map_search_error)?;
                result
            })();
            let close = read.close();
            if close.is_err() {
                return Err(source_hold_outcome_unknown(&hold));
            }
            if !source_hold_present(&source, &hold, stop)? {
                return Err(unavailable());
            }
            result
        })();
        let close = source.close();
        if close.is_err() {
            return Err(source_hold_binding_outcome_unknown(binding));
        }
        let result = source_result?;
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
            "search_project_entities",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":result.status(),"resource_closure":"closed",
                "logical_shard_id":request.logical_id,"object_id":request.object_id,
                "profile_digest":request.profile.digest(),"result_set":value}),
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
        "invalid ProjectEntity Search request",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact ProjectEntity shard/source hold unavailable",
    )
}
