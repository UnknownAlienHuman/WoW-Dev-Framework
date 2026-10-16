//! Exact outer selection provenance around the existing E3 continuation journal.
use super::*;
use crate::context::{ContextContinueRequest, execute_context_continue};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContinueInput {
    schema: String,
    prior_context_operation_id: String,
    prior_context_request_digest: String,
    prior_context_result_id: String,
    continuation_id: String,
}

pub struct SearchContextContinueRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: ContinueInput,
    host: ContextMapHostInput,
    host_binding: Box<str>,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl SearchContextContinueRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        host: ContextMapHostInput,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        _permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 8192 {
            return Err(invalid());
        }
        let input: ContinueInput = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/search-context-continue-input/1"
            || input.prior_context_operation_id == operation_id.as_str()
        {
            return Err(invalid());
        }
        OperationId::new(input.prior_context_operation_id.clone()).map_err(|_| invalid())?;
        RequestDigest::new(input.prior_context_request_digest.clone()).map_err(|_| invalid())?;
        let result_digest = input
            .prior_context_result_id
            .strip_prefix("wow-service/search-durable-result/1:")
            .ok_or_else(invalid)?;
        RequestDigest::new(format!("sha256:{result_digest}")).map_err(|_| invalid())?;
        ContextContinueRequest::new(&input.continuation_id, None, None)?;
        let host_binding = host.semantic_binding_digest()?;
        check_cancelled(stop)?;
        Ok(Self {
            operation_id,
            search_store,
            input,
            host,
            host_binding,
            store_limits,
            max_output_bytes,
        })
    }
}

pub(in crate::search) fn execute(
    request: &SearchContextContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_context_continue",
        &request.operation_id,
        &(
            &request.input,
            &request.host_binding,
            request.store_limits,
            request.max_output_bytes,
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
    let mut native_effect = false;
    let outcome = (|| {
        let prior = load_prior(&store, request, stop)?;
        let payload = prior.get("payload").ok_or_else(mismatch)?;
        let native = payload.get("context").ok_or_else(mismatch)?;
        let native_payload = native.get("payload").ok_or_else(mismatch)?;
        if field(native_payload, "continuation_id")? != request.input.continuation_id {
            return Err(mismatch());
        }
        let original_request = match field(&prior, "operation")? {
            "search_context_build"
                if field(native, "schema")? == "wow-service/context-build-result/2" =>
            {
                field(native, "request_id")?
            }
            "search_context_continue"
                if field(native, "schema")? == crate::context::CONTEXT_CONTINUE_RESULT_SCHEMA =>
            {
                field(native_payload, "original_request_id")?
            }
            _ => return Err(mismatch()),
        };
        let native_request = ContextContinueRequest::new(
            &request.input.continuation_id,
            Some(original_request),
            Some(field(
                native_payload.get("semantic_pack").ok_or_else(mismatch)?,
                "semantic_pack_id",
            )?),
        )?;
        let generation = field(native_payload, "store_generation_id")?;
        let receipt = payload.get("selection_receipt").ok_or_else(mismatch)?;
        if field(receipt, "schema")? != "wow-service/search-selection-receipt/1"
            || receipt
                .pointer("/owner_entity/kind")
                .and_then(serde_json::Value::as_str)
                != Some("project_graph_entity")
        {
            return Err(mismatch());
        }
        let explained = payload.get("search_explanation").ok_or_else(mismatch)?;
        check_cancelled(stop)?;
        native_effect = true;
        let result = execute_context_continue(&request.host, &native_request, stop)?;
        let (context, status) = checked_native_result(
            &result,
            field(explained, "status")?,
            generation,
            request.max_output_bytes.get(),
            &mut native_effect,
            stop,
        )?;
        if context
            .pointer("/payload/original_request_id")
            .and_then(serde_json::Value::as_str)
            != Some(original_request)
        {
            return Err(mismatch());
        }
        durable_response(
            "search_context_continue",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":status,"resource_closure":"closed","context_admission":"exact_native_continuation",
                "selection_receipt":receipt,"search_explanation":explained,"context":context,
                "context_host_binding":request.host_binding,
                "prior_context":{"operation_id":request.input.prior_context_operation_id,
                    "request_digest":request.input.prior_context_request_digest,
                    "result_id":request.input.prior_context_result_id,
                    "continuation_id":request.input.continuation_id}
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
            let error = if native_effect || error.code() == ServiceErrorCode::StoreOutcomeUnknown {
                let _ = store.mark_outcome_unknown(&id, &digest);
                outcome_unknown()
            } else {
                record_no_effect(&mut store, &id, &digest, error)
            };
            return Err(operation_failure(error, &request.operation_id));
        }
    };
    complete_native_effect(
        &mut store,
        &id,
        &digest,
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}

fn load_prior(
    store: &Store,
    request: &SearchContextContinueRequest,
    stop: &AtomicBool,
) -> ServiceResult<serde_json::Value> {
    check_cancelled(stop)?;
    let record = store
        .operation(
            &StoreOperationId::new(request.input.prior_context_operation_id.clone())
                .map_err(map_store_error)?,
        )
        .map_err(map_store_error)?
        .ok_or_else(unavailable)?;
    if record.state() != StoreOperationState::Completed
        || record.request_digest().as_str() != request.input.prior_context_request_digest
    {
        return Err(mismatch());
    }
    let bytes = read_completed_response(store, &record, request.store_limits.max_object_bytes)?;
    let prior: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| mismatch())?;
    let admission = match field(&prior, "operation")? {
        "search_context_build" => "exact_native_build",
        "search_context_continue" => "exact_native_continuation",
        _ => return Err(mismatch()),
    };
    let payload = prior.get("payload").ok_or_else(mismatch)?;
    if field(&prior, "operation_id")? != request.input.prior_context_operation_id
        || field(&prior, "result_id")? != request.input.prior_context_result_id
        || field(payload, "resource_closure")? != "closed"
        || field(payload, "context_admission")? != admission
        || field(payload, "context_host_binding")? != request.host_binding.as_ref()
        || payload
            .pointer("/context/resource_closure")
            .and_then(serde_json::Value::as_str)
            != Some("closed")
    {
        return Err(mismatch());
    }
    Ok(prior)
}
