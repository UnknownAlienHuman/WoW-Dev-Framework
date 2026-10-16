//! Original completed Source query data, opened through the exact Search owner.
use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    query_operation_id: String,
    query_request_digest: String,
    result_set_id: String,
    candidate_id: String,
}

pub struct SourceObservationExplainRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: Input,
    bounds: SearchBounds,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
    lease_expires_after: u64,
    permission: LocalMetadataPermission,
}
impl SourceObservationExplainRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        bounds: SearchBounds,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        lease_expires_after: u64,
        permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 4096 || lease_expires_after == 0 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/source-observation-explain-input/1"
            || input.query_operation_id == operation_id.as_str()
        {
            return Err(invalid());
        }
        OperationId::new(input.query_operation_id.clone()).map_err(|_| invalid())?;
        for (value, prefix) in [
            (&input.query_request_digest, ""),
            (&input.result_set_id, "source-observation-result:"),
            (&input.candidate_id, "source-observation-candidate:"),
        ] {
            RequestDigest::new(value.strip_prefix(prefix).ok_or_else(invalid)?.to_owned())
                .map_err(|_| invalid())?;
        }
        ObservationSearchProfile::new(bounds).map_err(map_search_error)?;
        Ok(Self {
            operation_id,
            search_store,
            input,
            bounds,
            store_limits,
            max_output_bytes,
            lease_expires_after,
            permission,
        })
    }
}

pub(in crate::search) fn execute(
    request: &SourceObservationExplainRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_source_observations_explain",
        &request.operation_id,
        &(
            &request.input,
            request.bounds,
            request.store_limits,
            request.max_output_bytes,
            request.lease_expires_after,
            "local_metadata_only",
        ),
    )?;
    let configuration = StoreConfiguration::new_with_instance_identity(
        DURABLE_SEARCH_STORE_PROFILE_ID,
        request.store_limits,
    )
    .map_err(map_store_error)?;
    let mut store =
        Store::open_existing(&request.search_store, configuration).map_err(map_store_error)?;
    match start_durable_operation(&mut store, &id, &digest, request.max_output_bytes.get())? {
        DurableOperationStart::Replay(bytes) => return Ok(bytes),
        DurableOperationStart::Started => {}
    }
    let mut batch = WriteBatch::new();
    let outcome = (|| {
        let original = store
            .operation(
                &StoreOperationId::new(request.input.query_operation_id.clone())
                    .map_err(map_store_error)?,
            )
            .map_err(map_store_error)?
            .ok_or_else(unavailable)?;
        if original.state() != StoreOperationState::Completed
            || original.request_digest().as_str() != request.input.query_request_digest
        {
            return Err(mismatch());
        }
        // The original whole result may exceed this explanation's output cap.
        // Its stored-object bound still applies before parsing the retained data.
        let bytes =
            read_completed_response(&store, &original, request.store_limits.max_object_bytes)?;
        let original: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| mismatch())?;
        if original
            .get("operation")
            .and_then(serde_json::Value::as_str)
            != Some("search_source_observations")
        {
            return Err(mismatch());
        }
        let payload = original.get("payload").ok_or_else(mismatch)?;
        if field(payload, "resource_closure")? != "closed" {
            return Err(mismatch());
        }
        let result = payload.get("result_set").ok_or_else(mismatch)?;
        if payload.get("status") != result.get("status") {
            return Err(mismatch());
        }
        let component = match field(result, "schema")? {
            "wow-search/source-observation-result-set/1" => {
                ObservationComponent::SourceObservations
            }
            "wow-search/source-observation-declaration-result-set/1" => {
                ObservationComponent::SourceDeclarations
            }
            "wow-search/source-inventory-result-set/1" => ObservationComponent::SourceInventory,
            _ => return Err(mismatch()),
        };
        let binding = ObservationRetainedBinding {
            component,
            generation: StoreGenerationId::parse(
                result
                    .pointer("/owner/source/store_generation_id")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(mismatch)?
                    .to_owned(),
            )
            .map_err(map_store_error)?,
            logical_id: field(payload, "logical_shard_id")?.into(),
            object_id: ObjectId::new(field(payload, "object_id")?.to_owned())
                .map_err(map_store_error)?,
            bounds: request.bounds,
            profile_digest: field(payload, "profile_digest")?.into(),
        };
        let (catalog, path) = search_catalog_key(&binding.logical_id)?;
        if store
            .catalog_entry(&catalog, &path)
            .map_err(map_store_error)?
            .is_none_or(|entry| entry.object_id() != &binding.object_id)
        {
            return Err(unavailable());
        }
        batch
            .add_catalog_mutation(CatalogMutation::set(
                catalog,
                path,
                CatalogExpectation::Exact(binding.object_id.clone()),
                binding.object_id.clone(),
            ))
            .map_err(map_store_error)?;
        let retained = canonical_json_bytes(result).map_err(|_| mismatch())?;
        let shard = open_retained(
            &binding,
            &mut store,
            &public_digest,
            "source-observation-explain",
            request.lease_expires_after,
            request.permission,
            stop,
        )?;
        let explained = shard
            .explain_retained_result(
                &retained,
                &request.input.result_set_id,
                &request.input.candidate_id,
                request.store_limits.max_object_bytes,
                request.max_output_bytes.get(),
                stop,
            )
            .map_err(map_search_error);
        // Even validation/cancellation errors must close the genuine native read.
        shard.release(&mut store).map_err(map_search_error)?;
        let explained = explained?;
        let value: serde_json::Value =
            serde_json::from_slice(explained.canonical_bytes()).map_err(|_| mismatch())?;
        durable_response(
            "search_source_observations_explain",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":explained.status(), "resource_closure":"closed",
                "query_operation_id":request.input.query_operation_id,
                "query_request_digest":request.input.query_request_digest,
                "explanation":value
            }),
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
                let _ = store.mark_outcome_unknown(&id, &digest);
                error
            } else {
                record_no_effect(&mut store, &id, &digest, error)
            };
            return Err(operation_failure(error, &request.operation_id));
        }
    };
    complete_durable_operation(
        &mut store,
        &id,
        &digest,
        batch,
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}

fn field<'a>(value: &'a serde_json::Value, key: &str) -> ServiceResult<&'a str> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(mismatch)
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "original Source query receipt or exact explanation guard differs",
    )
}
