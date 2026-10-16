//! Explicit caller selection over immutable, owner-validated explanation data.
//! This receipt confers neither owner acquisition nor Context admission.
use super::*;
use serde::Deserialize;

pub(super) mod context;
pub use context::{
    SearchContextBuildRequest, SearchContextContinueRequest, SearchContextRequest,
    SearchPlatformContextRequest,
};

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntityKind {
    ProjectGraphEntity,
    ReferenceDocument,
    SourceObservation,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum SelectionOrigin {
    ExplicitCallerSelection,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TargetAction {
    ReceiptOnly,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    explanation_operation_id: String,
    explanation_request_digest: String,
    explanation_result_id: String,
    result_set_id: String,
    candidate_id: String,
    expected_explanation_digest: String,
    expected_candidate_digest: String,
    entity_kind: EntityKind,
    expected_entity_id: String,
    selection_origin: SelectionOrigin,
    target_action: TargetAction,
}

pub struct SearchSelectionRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: Input,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl SearchSelectionRequest {
    pub fn new(
        operation_id: OperationId,
        search_store: PathBuf,
        input_json: &[u8],
        store_limits: SearchStoreLimits,
        max_output_bytes: NonZeroU64,
        _permission: LocalMetadataPermission,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        check_cancelled(stop)?;
        if !search_store.is_absolute() || input_json.len() > 16384 {
            return Err(invalid());
        }
        let input: Input = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/search-selection-input/1"
            || input.explanation_operation_id == operation_id.as_str()
        {
            return Err(invalid());
        }
        OperationId::new(input.explanation_operation_id.clone()).map_err(|_| invalid())?;
        for digest in [
            &input.explanation_request_digest,
            &input.expected_explanation_digest,
            &input.expected_candidate_digest,
        ] {
            RequestDigest::new(digest.clone()).map_err(|_| invalid())?;
        }
        let result_digest = input
            .explanation_result_id
            .strip_prefix("wow-service/search-durable-result/1:")
            .ok_or_else(invalid)?;
        RequestDigest::new(format!("sha256:{result_digest}")).map_err(|_| invalid())?;
        for id in [
            &input.result_set_id,
            &input.candidate_id,
            &input.expected_entity_id,
        ] {
            if id.is_empty()
                || id.len() > 1024
                || id.trim() != id
                || id.chars().any(char::is_control)
            {
                return Err(invalid());
            }
        }
        Ok(Self {
            operation_id,
            search_store,
            input,
            store_limits,
            max_output_bytes,
        })
    }
}

pub(super) fn execute(
    request: &SearchSelectionRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let (id, digest, public_digest) = durable_identity(
        "search_select",
        &request.operation_id,
        &(
            &request.input,
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
        let (receipt, explained) = materialize_selection(
            &store,
            &request.input,
            request.store_limits.max_object_bytes,
            stop,
        )?;
        durable_response(
            "search_select",
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":field(&explained,"status")?,"resource_closure":"closed",
                "selection_receipt":receipt
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
        WriteBatch::new(),
        &bytes,
        request.max_output_bytes.get(),
    )
    .map_err(|error| operation_failure(error, &request.operation_id))
}

fn materialize_selection(
    store: &Store,
    input: &Input,
    max_read_bytes: u64,
    stop: &AtomicBool,
) -> ServiceResult<(serde_json::Value, serde_json::Value)> {
    check_cancelled(stop)?;
    let record = store
        .operation(
            &StoreOperationId::new(input.explanation_operation_id.clone())
                .map_err(map_store_error)?,
        )
        .map_err(map_store_error)?
        .ok_or_else(unavailable)?;
    if record.state() != StoreOperationState::Completed
        || record.request_digest().as_str() != input.explanation_request_digest
    {
        return Err(mismatch());
    }
    let original_bytes = read_completed_response(store, &record, max_read_bytes)?;
    let original: serde_json::Value =
        serde_json::from_slice(&original_bytes).map_err(|_| mismatch())?;
    let operation = field(&original, "operation")?;
    let schema = match operation {
        "search_project_entities_explain" => "wow-search/project-entity-candidate-explanation/1",
        "search_reference_explain" => "wow-search/native-reference-candidate-explanation/1",
        "search_source_observations_explain" => {
            "wow-search/source-observation-candidate-explanation/1"
        }
        "search_federated_explain" => "wow-search/federated-candidate-explanation/1",
        _ => return Err(mismatch()),
    };
    let payload = original.get("payload").ok_or_else(mismatch)?;
    let explained = payload.get("explanation").ok_or_else(mismatch)?;
    if field(&original, "result_id")? != input.explanation_result_id
        || field(payload, "resource_closure")? != "closed"
        || field(explained, "schema")? != schema
        || field(explained, "result_set_id")? != input.result_set_id
        || field(explained, "candidate_id")? != input.candidate_id
        || field(payload, "status")? != field(explained, "status")?
        || canonical_digest("sha256:", explained)?.as_ref() != input.expected_explanation_digest
    {
        return Err(mismatch());
    }
    // The journal object and operation are immutable. This is an inert
    // selection of already-validated Search data, not a native owner decoder.
    let candidate = explained.get("candidate").ok_or_else(mismatch)?;
    if field(candidate, "candidate_id")? != input.candidate_id
        || canonical_digest("sha256:", candidate)?.as_ref() != input.expected_candidate_digest
    {
        return Err(mismatch());
    }
    let component = if operation == "search_federated_explain" {
        candidate.get("original_candidate").ok_or_else(mismatch)?
    } else {
        candidate
    };
    let expected_operation = match input.entity_kind {
        EntityKind::ProjectGraphEntity => "search_project_entities_explain",
        EntityKind::ReferenceDocument => "search_reference_explain",
        EntityKind::SourceObservation => "search_source_observations_explain",
    };
    if operation != expected_operation && operation != "search_federated_explain" {
        return Err(mismatch());
    }
    let entity_path = match input.entity_kind {
        EntityKind::ProjectGraphEntity => "/document/entity/node_id",
        EntityKind::ReferenceDocument => "/document/entity_key",
        EntityKind::SourceObservation => "/document/fact_id",
    };
    if component
        .pointer(entity_path)
        .and_then(serde_json::Value::as_str)
        != Some(&input.expected_entity_id)
    {
        return Err(mismatch());
    }
    check_cancelled(stop)?;
    let binding_keys = [
        "universe_id",
        "universe_set_id",
        "shard_id",
        "query_id",
        "result_query_id",
        "normalized_query_id",
        "search_profile_digest",
        "federation_profile",
        "ranking_profile",
    ];
    let binding: serde_json::Map<String, serde_json::Value> = explained
        .as_object()
        .ok_or_else(mismatch)?
        .iter()
        .filter(|(key, _)| binding_keys.contains(&key.as_str()))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let mut receipt = serde_json::json!({
        "schema":"wow-service/search-selection-receipt/1",
        "explanation_operation_id":input.explanation_operation_id,
        "explanation_request_digest":input.explanation_request_digest,
        "explanation_result_id":input.explanation_result_id,
        "explanation_digest":input.expected_explanation_digest,
        "result_set_id":input.result_set_id,
        "candidate_id":input.candidate_id,
        "candidate_digest":input.expected_candidate_digest,
        "search_binding":binding,
        "original_rank_tuple":candidate.get("rank_tuple").ok_or_else(mismatch)?,
        "owner_entity":{"kind":input.entity_kind,"id":input.expected_entity_id},
        "selection_origin":input.selection_origin,
        "target_action":input.target_action,
        "original_status":field(explained,"status")?,
        "context_admission":"not_requested",
        "authority":"explicit_selection_only",
        "nonclaims":["intent_verification","entity_fact_authority_upgrade","context_owner_acquisition","context_root_mapping","edit_or_tool_authorization"]
    });
    let receipt_id = canonical_digest("search-selection-receipt:", &receipt)?;
    receipt
        .as_object_mut()
        .ok_or_else(mismatch)?
        .insert("receipt_id".into(), receipt_id.to_string().into());
    Ok((receipt, explained.clone()))
}

fn field<'a>(data: &'a serde_json::Value, key: &str) -> ServiceResult<&'a str> {
    data.get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(mismatch)
}
fn invalid() -> ServiceError {
    failure(
        ServiceErrorCode::InvalidRequest,
        "invalid explicit Search selection",
    )
}
fn mismatch() -> ServiceError {
    failure(
        ServiceErrorCode::IdentityMismatch,
        "original explanation or explicit selection guard differs",
    )
}
fn unavailable() -> ServiceError {
    failure(
        ServiceErrorCode::ExactGenerationUnavailable,
        "exact completed Search explanation is unavailable",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancelled_selection_never_opens_or_creates_a_store() -> Result<(), Box<dyn std::error::Error>>
    {
        let store_path = std::env::temp_dir().join(format!(
            "wow-selection-cancelled-{}-{}.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let zero = format!("sha256:{}", "0".repeat(64));
        let input = serde_json::json!({
            "schema":"wow-service/search-selection-input/1",
            "explanation_operation_id":"original-explanation",
            "explanation_request_digest":zero,
            "explanation_result_id":format!("wow-service/search-durable-result/1:{}","0".repeat(64)),
            "result_set_id":"inert-result", "candidate_id":"inert-candidate",
            "expected_explanation_digest":zero,"expected_candidate_digest":zero,
            "entity_kind":"source_observation","expected_entity_id":"inert-fact",
            "selection_origin":"explicit_caller_selection","target_action":"receipt_only"
        });
        let request = SearchSelectionRequest::new(
            OperationId::new("cancelled-selection")?,
            store_path.clone(),
            &serde_json::to_vec(&input)?,
            SearchStoreLimits::new(65536, 8, 16, 128, 8)?,
            NonZeroU64::new(65536).ok_or("invalid output cap")?,
            LocalMetadataPermission::authorize_local_metadata_only(),
            &AtomicBool::new(false),
        )?;
        let error = SearchOperationService::new()
            .select_candidate(&request, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled selection succeeded")?;
        assert_eq!(error.code(), ServiceErrorCode::Cancelled);
        assert!(!store_path.exists());
        Ok(())
    }
}
