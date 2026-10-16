//! Explicit receipt-to-qualified-native platform Context Map/Inspect handoff.
use super::*;
use crate::context::{
    ContextPlatformHostInput, ContextPlatformOperation, ContextPlatformRequest,
    execute_context_platform,
};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    selection_operation_id: String,
    selection_request_digest: String,
    selection_result_id: String,
    receipt_id: String,
    #[serde(deserialize_with = "ContextPlatformRequest::deserialize_nested")]
    context_request: ContextPlatformRequest,
}
pub struct SearchPlatformContextRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: Input,
    host: ContextPlatformHostInput,
    host_binding: Box<str>,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl SearchPlatformContextRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        host: ContextPlatformHostInput,
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        _permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 8192 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/search-platform-context-input/1"
            || input.selection_operation_id == operation_id.as_str()
        {
            return Err(invalid());
        }
        OperationId::new(input.selection_operation_id.clone()).map_err(|_| invalid())?;
        for (value, prefix) in [
            (&input.selection_request_digest, "sha256:"),
            (
                &input.selection_result_id,
                "wow-service/search-durable-result/1:",
            ),
            (&input.receipt_id, "search-selection-receipt:"),
        ] {
            RequestDigest::new(format!(
                "sha256:{}",
                value.strip_prefix(prefix).ok_or_else(invalid)?
            ))
            .map_err(|_| invalid())?;
        }
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
    request: &SearchPlatformContextRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_platform_context",
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
    let outcome = (|| {
        let input = &request.input;
        let (receipt, explained) = load_selection_by_ids(
            &store,
            &input.selection_operation_id,
            &input.selection_request_digest,
            &input.selection_result_id,
            &input.receipt_id,
            request.store_limits.max_object_bytes,
            stop,
        )?;
        let owner = receipt.get("owner_entity").ok_or_else(mismatch)?;
        let federated =
            field(&explained, "schema")? == "wow-search/federated-candidate-explanation/1";
        let unsupported = field(owner, "kind")? != "project_graph_entity"
            || (federated
                && explained
                    .pointer("/candidate/universe_class")
                    .and_then(serde_json::Value::as_str)
                    != Some("blizzard_ui_source"));
        if unsupported {
            return durable_response(
                "search_platform_context",
                &request.operation_id,
                &public_digest,
                &serde_json::json!({"status":"not_evaluated","resource_closure":"closed",
                    "context_admission":"not_evaluated","reason":"qualified_platform_graph_root_required",
                    "selection_receipt":receipt,"search_explanation":explained,
                    "context_host_binding":request.host_binding}),
                request.max_output_bytes.get(),
            );
        }
        let (summary, entity, reference_view) = if federated {
            (explained.pointer("/query_context/components/blizzard_ui_source/result_summary").ok_or_else(mismatch)?,
             explained.pointer("/candidate/original_candidate/document/entity").ok_or_else(mismatch)?,
             Some(explained.pointer("/query_context/components/reference_platform_contract/result_summary/owner/reference_view_digest")
                 .and_then(serde_json::Value::as_str).ok_or_else(mismatch)?))
        } else {
            (
                explained.get("query_context").ok_or_else(mismatch)?,
                explained
                    .pointer("/candidate/document/entity")
                    .ok_or_else(mismatch)?,
                None,
            )
        };
        let source = summary.pointer("/owner/source").ok_or_else(mismatch)?;
        let root = field(owner, "id")?;
        if field(entity, "node_id")? != root
            || field(entity, "universe")? != field(source, "graph_universe_id")?
            || field(entity, "generation")? != field(source, "graph_generation_id")?
        {
            return Err(mismatch());
        }
        input.context_request.guard_search_selection(
            &request.host,
            source,
            root,
            reference_view,
        )?;
        check_cancelled(stop)?;
        // The existing E3 path alone acquires the selected native candidate,
        // verifies its full original binding/root, constructs Context and closes.
        let context = execute_context_platform(&request.host, &input.context_request, stop)?;
        if !matches!(context.resource_closure(), "closed" | "not_acquired") {
            return Err(outcome_unknown());
        }
        if let Some(code) = context.failure_code() {
            return Err(failure(code, "native selected platform Context refused"));
        }
        if context.resource_closure() != "closed"
            || context.canonical_bytes().len() > crate::context::CONTEXT_MAP_RESULT_MAX_BYTES
        {
            return Err(outcome_unknown());
        }
        check_output_bound(context.canonical_bytes(), request.max_output_bytes.get())?;
        let context_value: serde_json::Value =
            serde_json::from_slice(context.canonical_bytes()).map_err(|_| outcome_unknown())?;
        if context_value
            .pointer("/payload/store_generation_id")
            .and_then(serde_json::Value::as_str)
            != Some(field(source, "store_generation_id")?)
            || context.summary().map(|summary| summary.root_id.as_ref()) != Some(root)
        {
            return Err(mismatch());
        }
        let status = match context.status() {
            ContextMapStatus::Cancelled => {
                return Err(failure(
                    ServiceErrorCode::Cancelled,
                    "selected platform Context cancelled",
                ));
            }
            ContextMapStatus::Failed => {
                return Err(failure(
                    ServiceErrorCode::InternalContractViolation,
                    "native platform Context failed without typed cause",
                ));
            }
            ContextMapStatus::Partial => fold_status(field(&explained, "status")?, "partial"),
            ContextMapStatus::Truncated => fold_status(field(&explained, "status")?, "truncated"),
            ContextMapStatus::Complete => fold_status(field(&explained, "status")?, "exact_found"),
        };
        let admission = match input.context_request.operation() {
            ContextPlatformOperation::Map => "exact_native_platform_map",
            ContextPlatformOperation::Inspect => "exact_native_platform_inspection",
        };
        check_cancelled(stop)?;
        durable_response(
            "search_platform_context",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({"status":status,"resource_closure":"closed","context_admission":admission,
                "selection_receipt":receipt,"search_explanation":explained,"context":context_value,
                "context_host_binding":request.host_binding}),
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
        WriteBatch::new(),
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}
