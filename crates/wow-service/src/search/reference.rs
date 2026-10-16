//! One atomic native ReferenceView + Search projection publication and exact reads.
use super::*;
use serde::Deserialize;
use wow_reference::persistent::{
    PersistentReferenceStore, ReferenceStoreError, ReferenceStoreErrorCode,
};
use wow_search::reference::{
    ReferenceQueryRequest, ReferenceSearchBinding, ReferenceSearchProfile,
    open_reference_callable_shard, prepare_reference_callable_shard,
};
use wow_store::ObjectId;
pub(super) mod pagination;
pub use pagination::{ReferenceContinueRequest, ReferenceResultReleaseRequest};
use wow_search::reference::ReferencePagePolicy;
const SOURCE_ROOTS: &str = "wow-search.reference-sources";
const HANDLE_SCHEMA: &str = "wow-service/native-reference-search-handle/1";
const DOCUMENT_HANDLE_SCHEMA: &str = "wow-service/native-reference-document-search-handle/1";

pub struct ReferenceIndexBuildRequest {
    operation_id: OperationId,
    source_config: PathBuf,
    config_digest: RequestDigest,
    search_store: PathBuf,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    permission: LocalMetadataPermission,
}
impl ReferenceIndexBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        source_config: PathBuf,
        expected_config_digest: &str,
        search_store: PathBuf,
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        permission: LocalMetadataPermission,
    ) -> ServiceResult<Self> {
        if !source_config.is_absolute() || !search_store.is_absolute() {
            return Err(invalid());
        }
        ReferenceSearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            source_config,
            config_digest: RequestDigest::new(expected_config_digest.to_owned())
                .map_err(map_store_error)?,
            search_store,
            bounds,
            store_limits,
            max_output_bytes,
            permission,
        })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Handle {
    schema: String,
    logical_shard_id: String,
    object_id: ObjectId,
    reference_view_object_id: ObjectId,
    reference_generation: String,
    reference_view_digest: String,
    search_profile_digest: String,
    bounds: SearchBounds,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    native_documents: Option<wow_reference::native_view::NativeReferenceDocuments>,
}
fn handle_profile(handle: &Handle) -> ServiceResult<ReferenceSearchProfile> {
    if handle.schema
        != if handle.native_documents.is_some() {
            DOCUMENT_HANDLE_SCHEMA
        } else {
            HANDLE_SCHEMA
        }
    {
        return Err(mismatch());
    }
    let profile = ReferenceSearchProfile::new_documents(handle.bounds, handle.native_documents)
        .map_err(map_search_error)?;
    if profile.digest() != handle.search_profile_digest {
        return Err(mismatch());
    }
    Ok(profile)
}
pub struct ReferenceShardRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    handle: Handle,
    query: Option<ReferenceQueryRequest>,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
    page_policy: Option<ReferencePagePolicy>,
}
#[derive(Serialize)]
struct ShardIdentity<'a> {
    handle: &'a Handle,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<&'a ReferenceQueryRequest>,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    privacy_scope: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    page_policy: Option<&'a ReferencePagePolicy>,
}
impl ReferenceShardRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        handle_json: &[u8],
        query_json: Option<&[u8]>,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || lease_expires_after == 0 || handle_json.len() > 16384 {
            return Err(invalid());
        }
        let handle: Handle = serde_json::from_slice(handle_json).map_err(|_| invalid())?;
        handle_profile(&handle)?;
        handle
            .reference_generation
            .parse::<wow_core::ReferenceGenerationId>()
            .map_err(|_| invalid())?;
        let query = query_json
            .map(|json| {
                ReferenceQueryRequest::from_json_slice(json, stop).map_err(map_search_error)
            })
            .transpose()?;
        Ok(Self {
            operation_id,
            search_store,
            handle,
            query,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            page_policy: None,
        })
    }
    pub fn with_pagination_json(mut self, bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        if self.query.is_none() {
            return Err(invalid());
        }
        self.page_policy =
            Some(ReferencePagePolicy::from_json_slice(bytes, stop).map_err(map_search_error)?);
        Ok(self)
    }
}

pub(super) fn build(
    request: &ReferenceIndexBuildRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (operation_id, digest, public_digest) = durable_identity(
        "wow-service/reference-search-build/1",
        &request.operation_id,
        &(
            &request.config_digest,
            request.bounds,
            request.store_limits,
            request.max_output_bytes,
            "local_metadata_only",
        ),
    )?;
    let mut store = open_durable_search_store(&request.search_store, request.store_limits)?;
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
        // Completed replay above requires no config/source reads.
        let source =
            crate::source_reference::read_local_source_reference(&request.source_config, stop)?;
        if source.configuration_digest().to_string() != request.config_digest.as_str() {
            return Err(mismatch());
        }
        let owner_object = PersistentReferenceStore::new(&mut store)
            .prepare_view(&source.reference().view)
            .map_err(reference_error)?;
        let source_object_id = owner_object.object_id().clone();
        #[derive(Serialize)]
        struct AcquisitionFailure<'a> {
            path: &'a str,
            source_sha256: &'a str,
            error_code: &'a wow_reference::native::NativeErrorCode,
            #[serde(skip_serializing_if = "Option::is_none")]
            byte_range: Option<[usize; 2]>,
        }
        let failures = source
            .acquisition_failures()
            .map(|(path, source_sha256, error)| {
                serde_json::to_value(AcquisitionFailure {
                    path,
                    source_sha256,
                    error_code: &error.code,
                    byte_range: error.span.map(|span| [span.start, span.end]),
                })
                .map_err(|_| invalid())
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let binding = ReferenceSearchBinding::new(
            source.profile().clone(),
            &store,
            source_object_id.clone(),
            source.is_partial(),
            source.source_revision(),
            failures,
        )
        .map_err(map_search_error)?;
        let profile =
            ReferenceSearchProfile::new_documents(request.bounds, source.native_documents())
                .map_err(map_search_error)?;
        let shard = prepare_reference_callable_shard(
            source.reference(),
            binding,
            &profile,
            request.permission,
            request.store_limits,
            stop,
        )
        .map_err(map_search_error)?;
        let handle = Handle {
            schema: if source.native_documents().is_some() {
                DOCUMENT_HANDLE_SCHEMA
            } else {
                HANDLE_SCHEMA
            }
            .into(),
            logical_shard_id: shard.logical_id().into(),
            object_id: shard.object_id().clone(),
            reference_view_object_id: source_object_id,
            reference_generation: shard.reference_generation().into(),
            reference_view_digest: shard.reference_view_digest().into(),
            search_profile_digest: profile.digest().into(),
            bounds: request.bounds,
            native_documents: source.native_documents(),
        };
        let count = shard.document_count();
        add_root(
            &store,
            &mut batch,
            SEARCH_SHARD_CATALOG,
            &handle.logical_shard_id,
            &handle.object_id,
        )?;
        add_root(
            &store,
            &mut batch,
            SOURCE_ROOTS,
            &handle.logical_shard_id,
            &handle.reference_view_object_id,
        )?;
        batch.add_object(owner_object).map_err(map_store_error)?;
        batch
            .add_object(shard.into_object())
            .map_err(map_store_error)?;
        durable_response(
            "search_reference_build",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":"retained","resource_closure":"closed","handle":handle,
                "document_count":count,"projection":match source.native_documents() {
                    Some(wow_reference::native_view::NativeReferenceDocuments::SourceDeclarationsV1) => "native_source_declarations",
                    Some(wow_reference::native_view::NativeReferenceDocuments::CallablesAndEventsV1) => "native_callables_and_events",
                    None => "native_global_and_namespace_callables",
                },
                "source_acquisition_partial":source.is_partial(),"source_freshness":"unverified-current",
                "privacy_scope":"local_metadata_only","license_state":"not_evaluated"}),
            request.max_output_bytes.get(),
        )
    })();
    finish(
        &mut store,
        &operation_id,
        &digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}

pub(super) fn execute(
    request: &ReferenceShardRequest,
    retire: bool,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    if retire == request.query.is_some() {
        return Err(invalid());
    }
    let operation = if retire {
        "search_reference_retire"
    } else {
        "search_reference_query"
    };
    let (operation_id, digest, public_digest) = durable_identity(
        operation,
        &request.operation_id,
        &ShardIdentity {
            handle: &request.handle,
            query: request.query.as_ref(),
            store_limits: request.store_limits,
            max_output_bytes: request.max_output_bytes,
            lease_expires_after: request.lease_expires_after,
            privacy_scope: "local_metadata_only",
            page_policy: request.page_policy.as_ref(),
        },
    )?;
    let mut store = open_durable_search_store(&request.search_store, request.store_limits)?;
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
        let h = &request.handle;
        let query = acquire_result(request, &mut store, &public_digest, stop)?;
        if retire {
            for (catalog, target) in [
                (SEARCH_SHARD_CATALOG, &h.object_id),
                (SOURCE_ROOTS, &h.reference_view_object_id),
            ] {
                batch
                    .add_catalog_mutation(CatalogMutation::delete(
                        CatalogName::new(catalog).map_err(map_store_error)?,
                        CatalogPath::new(h.logical_shard_id.clone()).map_err(map_store_error)?,
                        target.clone(),
                    ))
                    .map_err(map_store_error)?;
            }
            durable_response(
                operation,
                &request.operation_id,
                &public_digest,
                &serde_json::json!({"status":"retired","resource_closure":"closed","handle":h}),
                request.max_output_bytes.get(),
            )
        } else {
            let result = query.ok_or_else(invalid)??;
            // Fence concurrent retirement at the atomic query completion.
            for (catalog, target) in [
                (SEARCH_SHARD_CATALOG, &h.object_id),
                (SOURCE_ROOTS, &h.reference_view_object_id),
            ] {
                batch
                    .add_catalog_mutation(CatalogMutation::set(
                        CatalogName::new(catalog).map_err(map_store_error)?,
                        CatalogPath::new(h.logical_shard_id.clone()).map_err(map_store_error)?,
                        CatalogExpectation::Exact(target.clone()),
                        target.clone(),
                    ))
                    .map_err(map_store_error)?;
            }
            if let Some(policy) = &request.page_policy {
                return pagination::start(
                    request,
                    policy,
                    &result,
                    &public_digest,
                    &mut store,
                    &mut batch,
                    stop,
                );
            }
            let result_value: serde_json::Value =
                serde_json::from_slice(result.canonical_bytes()).map_err(|_| invalid())?;
            durable_response(
                operation,
                &request.operation_id,
                &public_digest,
                &serde_json::json!({"status":result.status(),"resource_closure":"closed",
                    "handle":h,"result_set":result_value}),
                request.max_output_bytes.get(),
            )
        }
    })();
    finish(
        &mut store,
        &operation_id,
        &digest,
        &request.operation_id,
        outcome,
        batch,
        request.max_output_bytes.get(),
        stop,
    )
}

fn require_root(store: &Store, catalog: &str, path: &str, id: &ObjectId) -> ServiceResult<()> {
    let entry = store
        .catalog_entry(
            &CatalogName::new(catalog).map_err(map_store_error)?,
            &CatalogPath::new(path.to_owned()).map_err(map_store_error)?,
        )
        .map_err(map_store_error)?;
    match entry {
        Some(entry) if entry.object_id() == id => Ok(()),
        Some(_) => Err(mismatch()),
        None => Err(unavailable()),
    }
}
fn add_root(
    store: &Store,
    batch: &mut WriteBatch,
    catalog: &str,
    path: &str,
    id: &ObjectId,
) -> ServiceResult<()> {
    let catalog = CatalogName::new(catalog).map_err(map_store_error)?;
    let path = CatalogPath::new(path.to_owned()).map_err(map_store_error)?;
    let expectation = match store
        .catalog_entry(&catalog, &path)
        .map_err(map_store_error)?
    {
        Some(entry) if entry.object_id() == id => CatalogExpectation::Exact(id.clone()),
        Some(_) => return Err(mismatch()),
        None => CatalogExpectation::Absent,
    };
    batch
        .add_catalog_mutation(CatalogMutation::set(catalog, path, expectation, id.clone()))
        .map_err(map_store_error)
}
#[allow(clippy::too_many_arguments)]
fn finish(
    store: &mut Store,
    id: &StoreOperationId,
    digest: &RequestDigest,
    public_id: &OperationId,
    outcome: ServiceResult<Box<[u8]>>,
    batch: WriteBatch,
    max_bytes: u64,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    let bytes = match outcome.and_then(|bytes| {
        check_cancelled(stop)?;
        Ok(bytes)
    }) {
        Ok(bytes) => bytes,
        Err(error) => {
            let error = if error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(id, digest);
                error
            } else {
                record_no_effect(store, id, digest, error)
            };
            return Err(operation_failure(error, public_id));
        }
    };
    complete_durable_operation(store, id, digest, batch, &bytes, max_bytes)
        .map_err(|error| operation_failure(error, public_id))
}
fn reference_error(error: ReferenceStoreError) -> ServiceError {
    let code = match error.code() {
        ReferenceStoreErrorCode::StoreOutcomeUnknown => ServiceErrorCode::StoreOutcomeUnknown,
        ReferenceStoreErrorCode::StoreCancelled => ServiceErrorCode::Cancelled,
        ReferenceStoreErrorCode::StoreObjectTooLarge
        | ReferenceStoreErrorCode::StoreBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        ReferenceStoreErrorCode::StoreDatabaseUnavailable => ServiceErrorCode::ComponentUnavailable,
        _ => ServiceErrorCode::IdentityMismatch,
    };
    failure(code, "exact ReferenceStore owner operation failed")
}
fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid native Reference Search request",
    )
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "native Reference Search identity differs from its exact handle",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact native Reference Search roots or owner object are unavailable",
    )
}

pub(super) fn acquire_result(
    request: &ReferenceShardRequest,
    store: &mut Store,
    public_digest: &str,
    stop: &AtomicBool,
) -> ServiceResult<Option<ServiceResult<wow_search::reference::ReferenceQueryResult>>> {
    let h = &request.handle;
    require_root(
        store,
        SEARCH_SHARD_CATALOG,
        &h.logical_shard_id,
        &h.object_id,
    )?;
    require_root(
        store,
        SOURCE_ROOTS,
        &h.logical_shard_id,
        &h.reference_view_object_id,
    )?;
    let owner = PersistentReferenceStore::new(store)
        .read_exact(&h.reference_view_object_id)
        .map_err(reference_error)?
        .ok_or_else(unavailable)?;
    if owner.generation_id() != h.reference_generation
        || owner.self_digest() != h.reference_view_digest
    {
        return Err(mismatch());
    }
    let profile = handle_profile(h)?;
    let lease = SearchStoreLeaseRequest::new(
        lease_id_for("reference-search", public_digest)?,
        "wow-service-search",
        LogicalEpoch::new(request.lease_expires_after),
    )
    .map_err(map_store_error)?;
    let shard = open_reference_callable_shard(
        store,
        &h.logical_shard_id,
        &h.object_id,
        &profile,
        &h.reference_view_object_id,
        &owner,
        request.permission,
        lease,
        stop,
    )
    .map_err(map_search_error)?;
    let query = request.query.as_ref().map(|query| {
        shard
            .retrieve(query, &owner, stop)
            .map_err(map_search_error)
    });
    shard.release(store).map_err(map_search_error)?;

    Ok(query)
}
pub(super) fn federation_input(request: &ReferenceShardRequest) -> serde_json::Value {
    serde_json::json!({"handle":request.handle,"query":request.query})
}
pub(super) fn fence_result(
    request: &ReferenceShardRequest,
    batch: &mut WriteBatch,
) -> ServiceResult<()> {
    let h = &request.handle;
    for (catalog, target) in [
        (SEARCH_SHARD_CATALOG, &h.object_id),
        (SOURCE_ROOTS, &h.reference_view_object_id),
    ] {
        batch
            .add_catalog_mutation(CatalogMutation::set(
                CatalogName::new(catalog).map_err(map_store_error)?,
                CatalogPath::new(h.logical_shard_id.clone()).map_err(map_store_error)?,
                CatalogExpectation::Exact(target.clone()),
                target.clone(),
            ))
            .map_err(map_store_error)?;
    }
    Ok(())
}
pub(super) fn retained_handle(request: &ReferenceShardRequest) -> Handle {
    request.handle.clone()
}
pub(super) fn retained_objects(handle: &Handle) -> (ObjectId, ObjectId) {
    (
        handle.object_id.clone(),
        handle.reference_view_object_id.clone(),
    )
}
pub(super) fn open_retained_shard(
    handle: &Handle,
    store: &mut Store,
    digest: &str,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
    stop: &AtomicBool,
) -> ServiceResult<wow_search::reference::ReferenceSearchShard> {
    let profile = handle_profile(handle)?;
    let owner = PersistentReferenceStore::new(store)
        .read_exact(&handle.reference_view_object_id)
        .map_err(reference_error)?
        .ok_or_else(unavailable)?;
    if owner.generation_id() != handle.reference_generation
        || owner.self_digest() != handle.reference_view_digest
    {
        return Err(mismatch());
    }
    let lease = SearchStoreLeaseRequest::new(
        lease_id_for("federated-page-reference", digest)?,
        "wow-service-search",
        LogicalEpoch::new(lease_expires_after),
    )
    .map_err(map_store_error)?;
    open_reference_callable_shard(
        store,
        &handle.logical_shard_id,
        &handle.object_id,
        &profile,
        &handle.reference_view_object_id,
        &owner,
        permission,
        lease,
        stop,
    )
    .map_err(map_search_error)
}
