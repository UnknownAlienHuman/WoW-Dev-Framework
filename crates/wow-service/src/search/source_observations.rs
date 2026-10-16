//! Exact source-observation Search orchestration; no source acquisition on query.
use super::*;
use serde::Deserialize;
use wow_search::source_observations::{
    ObservationQueryRequest, ObservationSearchProfile, open_source_observation_shard,
};
use wow_store::ObjectId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationComponent {
    SourceObservations,
    SourceDeclarations,
    SourceInventory,
}
impl ObservationComponent {
    pub(in crate::search) const fn from_flags(declarations: bool, globals: bool) -> Self {
        if globals {
            Self::SourceInventory
        } else if declarations {
            Self::SourceDeclarations
        } else {
            Self::SourceObservations
        }
    }
    pub const fn declarations(self) -> bool {
        matches!(self, Self::SourceDeclarations | Self::SourceInventory)
    }
    pub(in crate::search) fn profile(
        self,
        bounds: SearchBounds,
    ) -> wow_search::SearchResult<ObservationSearchProfile> {
        match self {
            Self::SourceObservations => ObservationSearchProfile::new(bounds),
            Self::SourceDeclarations => {
                ObservationSearchProfile::with_function_declarations(bounds)
            }
            Self::SourceInventory => ObservationSearchProfile::inventory(bounds),
        }
    }
    pub(in crate::search) const fn id(self) -> &'static str {
        match self {
            Self::SourceObservations => "project_source_observations_v1",
            Self::SourceDeclarations => "project_source_declarations_v1",
            Self::SourceInventory => "project_source_inventory_v1",
        }
    }
}
pub(in crate::search) struct BoundSourceRequest {
    pub(in crate::search) query: SourceObservationRetrievalRequest,
    pub(in crate::search) hold: SearchSourceHoldBinding,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::search) struct ObservationRetainedBinding {
    component: ObservationComponent,
    generation: StoreGenerationId,
    logical_id: Box<str>,
    pub(in crate::search) object_id: ObjectId,
    bounds: SearchBounds,
    profile_digest: Box<str>,
}

impl SearchIndexBuildRequest {
    pub fn with_source_inventory(mut self) -> ServiceResult<Self> {
        if !matches!(
            self.selection,
            DurableSelection::SourceObservations(ObservationComponent::SourceObservations)
        ) {
            return Err(invalid());
        }
        ObservationSearchProfile::inventory(self.bounds).map_err(map_search_error)?;
        self.selection =
            DurableSelection::SourceObservations(ObservationComponent::SourceInventory);
        Ok(self)
    }
    pub fn with_function_declarations(mut self) -> ServiceResult<Self> {
        if !matches!(
            self.selection,
            DurableSelection::SourceObservations(ObservationComponent::SourceObservations)
        ) {
            return Err(invalid());
        }
        ObservationSearchProfile::with_function_declarations(self.bounds)
            .map_err(map_search_error)?;
        self.selection =
            DurableSelection::SourceObservations(ObservationComponent::SourceDeclarations);
        Ok(self)
    }
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
            selection: DurableSelection::SourceObservations(
                ObservationComponent::SourceObservations,
            ),
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}
impl SearchShardRetireRequest {
    pub fn with_source_inventory(mut self) -> ServiceResult<Self> {
        if !matches!(
            self.selection,
            DurableSelection::SourceObservations(ObservationComponent::SourceObservations)
        ) {
            return Err(invalid());
        }
        ObservationSearchProfile::inventory(self.bounds).map_err(map_search_error)?;
        self.selection =
            DurableSelection::SourceObservations(ObservationComponent::SourceInventory);
        Ok(self)
    }
    pub fn with_function_declarations(mut self) -> ServiceResult<Self> {
        if !matches!(
            self.selection,
            DurableSelection::SourceObservations(ObservationComponent::SourceObservations)
        ) {
            return Err(invalid());
        }
        ObservationSearchProfile::with_function_declarations(self.bounds)
            .map_err(map_search_error)?;
        self.selection =
            DurableSelection::SourceObservations(ObservationComponent::SourceDeclarations);
        Ok(self)
    }
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
            selection: DurableSelection::SourceObservations(
                ObservationComponent::SourceObservations,
            ),
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
        Self::new_selected(
            operation_id,
            search_store_path,
            generation,
            logical_id,
            object_id,
            profile_digest,
            query_json,
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            false,
            stop,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new_selected(
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
        include_function_declarations: bool,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        Self::new_component(
            operation_id,
            search_store_path,
            generation,
            logical_id,
            object_id,
            profile_digest,
            query_json,
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            if include_function_declarations {
                ObservationComponent::SourceDeclarations
            } else {
                ObservationComponent::SourceObservations
            },
            stop,
        )
    }
    #[allow(clippy::too_many_arguments)]
    pub fn new_component(
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
        component: ObservationComponent,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store_path.is_absolute() || lease_expires_after == 0 {
            return Err(invalid());
        }
        let profile = component.profile(bounds).map_err(map_search_error)?;
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
pub(in crate::search) fn retained_binding(
    request: &BoundSourceRequest,
) -> ObservationRetainedBinding {
    let q = &request.query;
    ObservationRetainedBinding {
        component: ObservationComponent::from_flags(
            q.profile.includes_function_declarations(),
            q.profile.includes_global_accesses(),
        ),
        generation: q.generation.clone(),
        logical_id: q.logical_id.clone(),
        object_id: q.object_id.clone(),
        bounds: q.profile.bounds(),
        profile_digest: q.profile.digest().into(),
    }
}
pub(in crate::search) fn federation_input(request: &BoundSourceRequest) -> serde_json::Value {
    let q = &request.query;
    let h = &request.hold;
    serde_json::json!({"component":ObservationComponent::from_flags(q.profile.includes_function_declarations(),q.profile.includes_global_accesses()),
        "generation":q.generation,"logical_id":q.logical_id,"object_id":q.object_id,"profile_digest":q.profile.digest(),"query":q.query,
        "build_operation_id":h.build_operation_id.as_str(),"source_epoch":h.source_store_epoch_id,"hold_id":h.retention_root_id,"hold_digest":h.retention_root_digest})
}
pub(in crate::search) fn fence_result(
    request: &BoundSourceRequest,
    batch: &mut WriteBatch,
) -> ServiceResult<()> {
    let q = &request.query;
    let (catalog, path) = search_catalog_key(&q.logical_id)?;
    batch
        .add_catalog_mutation(CatalogMutation::set(
            catalog,
            path,
            CatalogExpectation::Exact(q.object_id.clone()),
            q.object_id.clone(),
        ))
        .map_err(map_store_error)
}
pub(in crate::search) fn acquire_bound(
    request: &BoundSourceRequest,
    store: &mut Store,
    digest: &str,
    purpose: &str,
    stop: &AtomicBool,
) -> ServiceResult<wow_search::source_observations::BoundObservationQueryResult> {
    let q = &request.query;
    let binding = &request.hold;
    let (catalog, path) = search_catalog_key(&q.logical_id)?;
    if store
        .catalog_entry(&catalog, &path)
        .map_err(map_store_error)?
        .is_none_or(|e| e.object_id() != &q.object_id)
    {
        return Err(unavailable());
    }
    let source = crate::live_project::LiveProjectStore::open_retained_owner_store(
        &binding.project_store_root,
    )?;
    let result = (|| {
        let selected = ObservationComponent::from_flags(
            q.profile.includes_function_declarations(),
            q.profile.includes_global_accesses(),
        );
        let docs = Documents::SourceObservations(selected);
        let profile = DurableProfile::SourceObservations(q.profile.clone());
        let hold = validate_source_hold_binding(
            &source,
            store.store_instance_id().map_err(map_store_error)?,
            binding,
            &q.generation,
            &docs,
            &profile,
            q.store_limits,
        )?;
        if !source_hold_present(&source, &hold, stop)? {
            return Err(unavailable());
        }
        let read = source
            .read(&ReadSelector::Exact(q.generation.clone()), stop)
            .map_err(map_store_error)?;
        let result = (|| {
            let pair = wow_project::replay::publication::AcquiredProjectPair::read(&read, stop)
                .map_err(map_project_error)?;
            let lease = SearchStoreLeaseRequest::new(
                lease_id_for(purpose, digest)?,
                "wow-service-search",
                LogicalEpoch::new(q.lease_expires_after),
            )
            .map_err(map_store_error)?;
            let shard = open_source_observation_shard(
                store,
                &q.logical_id,
                &q.object_id,
                &q.generation,
                &q.profile,
                q.permission,
                lease,
                stop,
            )
            .map_err(map_search_error)?;
            let query = shard
                .retrieve_bound(&read, &pair, &q.query, stop)
                .map_err(map_search_error);
            shard.release(store).map_err(map_search_error)?;
            query
        })();
        if read.close().is_err() {
            return Err(source_hold_outcome_unknown(&hold));
        }
        if !source_hold_present(&source, &hold, stop)? {
            return Err(unavailable());
        }
        result
    })();
    if source.close().is_err() {
        return Err(source_hold_binding_outcome_unknown(binding));
    }
    result
}
#[allow(clippy::too_many_arguments)]
pub(in crate::search) fn open_retained(
    binding: &ObservationRetainedBinding,
    store: &mut Store,
    digest: &str,
    purpose: &str,
    expires: u64,
    permission: LocalMetadataPermission,
    stop: &AtomicBool,
) -> ServiceResult<wow_search::source_observations::ObservationSearchShard> {
    let profile = binding
        .component
        .profile(binding.bounds)
        .map_err(map_search_error)?;
    if profile.digest() != binding.profile_digest.as_ref() {
        return Err(failure(
            ServiceErrorCode::IdentityMismatch,
            "retained source-observation profile differs",
        ));
    }
    let lease = SearchStoreLeaseRequest::new(
        lease_id_for(purpose, digest)?,
        "wow-service-search",
        LogicalEpoch::new(expires),
    )
    .map_err(map_store_error)?;
    open_source_observation_shard(
        store,
        &binding.logical_id,
        &binding.object_id,
        &binding.generation,
        &profile,
        permission,
        lease,
        stop,
    )
    .map_err(map_search_error)
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact observed source shard/owner hold unavailable",
    )
}
fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid exact source observation Search request",
    )
}
