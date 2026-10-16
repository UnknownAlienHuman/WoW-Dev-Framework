//! One exact Store, two genuinely validated Search owners, one durable operation.
use super::*;
use serde::Deserialize;
use wow_search::federation::{FederationPagePolicy, FederationQueryRequest, federate};
pub(in crate::search) mod pagination;
pub use pagination::{FederationContinueRequest, FederationResultReleaseRequest};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    primary: Primary,
    reference: Reference,
    query: serde_json::Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Primary {
    project_store_root: PathBuf,
    generation: String,
    logical_shard_id: String,
    object_id: String,
    profile_digest: String,
    bounds: SearchBounds,
    build_operation_id: String,
    source_store_epoch_id: String,
    source_generation_hold_id: String,
    source_generation_hold_digest: String,
    query: serde_json::Value,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference {
    handle: serde_json::Value,
    query: serde_json::Value,
}
pub struct FederationRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    primary: EntityRetrievalRequest,
    reference: ReferenceShardRequest,
    query: FederationQueryRequest,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    page_policy: Option<FederationPagePolicy>,
}
impl FederationRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 65536 || lease_expires_after == 0 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/exact-search-federation-input/1" {
            return Err(invalid());
        }
        let p = input.primary;
        let hold = SearchSourceHoldBinding::new(
            p.project_store_root,
            OperationId::new(p.build_operation_id).map_err(|_| invalid())?,
            p.source_store_epoch_id,
            p.source_generation_hold_id,
            p.source_generation_hold_digest,
        )?;
        let primary = EntityRetrievalRequest::new(
            operation_id.clone(),
            search_store.clone(),
            hold,
            &p.generation,
            &p.logical_shard_id,
            &p.object_id,
            &p.profile_digest,
            &encode(&p.query)?,
            p.bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            stop,
        )?;
        let reference = ReferenceShardRequest::new(
            operation_id.clone(),
            search_store.clone(),
            &encode(&input.reference.handle)?,
            Some(&encode(&input.reference.query)?),
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
            stop,
        )?;
        let query = FederationQueryRequest::from_json_slice(&encode(&input.query)?, stop)
            .map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            search_store,
            primary,
            reference,
            query,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            page_policy: None,
        })
    }
    pub fn with_page_policy(mut self, bytes: &[u8], stop: &AtomicBool) -> ServiceResult<Self> {
        self.page_policy =
            Some(FederationPagePolicy::from_json_slice(bytes, stop).map_err(map_search_error)?);
        Ok(self)
    }
}
fn encode(value: &serde_json::Value) -> ServiceResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| invalid())
}
pub(super) fn execute(request: &FederationRequest, stop: &AtomicBool) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let identity = (
        entity::federation_input(&request.primary),
        reference::federation_input(&request.reference),
        &request.query,
        request.store_limits,
        request.max_output_bytes,
        request.lease_expires_after,
        "local_metadata_only",
    );
    let (operation_id, digest, public_digest) = if let Some(policy) = &request.page_policy {
        durable_identity(
            "wow-service/exact-search-federation-page-request/1",
            &request.operation_id,
            &(&identity, policy),
        )?
    } else {
        durable_identity(
            "wow-service/exact-search-federation-request/1",
            &request.operation_id,
            &identity,
        )?
    };
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
        let primary = entity::acquire_result(&request.primary, &mut store, &public_digest, stop)?;
        let reference =
            reference::acquire_result(&request.reference, &mut store, &public_digest, stop)?
                .ok_or_else(invalid)??;
        let result =
            federate(&primary, &reference, &request.query, stop).map_err(map_search_error)?;
        entity::fence_result(&request.primary, &mut batch)?;
        reference::fence_result(&request.reference, &mut batch)?;
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
        let value: serde_json::Value =
            serde_json::from_slice(result.canonical_bytes()).map_err(|_| invalid())?;
        durable_response(
            "search_federated",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":result.status(),"resource_closure":"closed","result_set":value}),
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
#[allow(clippy::too_many_arguments)]
fn finish(
    store: &mut Store,
    operation_id: &StoreOperationId,
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
                let _ = store.mark_outcome_unknown(operation_id, digest);
                error
            } else {
                record_no_effect(store, operation_id, digest, error)
            };
            return Err(operation_failure(error, public_id));
        }
    };
    complete_durable_operation(store, operation_id, digest, batch, &bytes, max_bytes)
        .map_err(|error| operation_failure(error, public_id))
}
fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid exact Search federation request",
    )
}
