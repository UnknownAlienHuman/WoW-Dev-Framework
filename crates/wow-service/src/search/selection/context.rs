//! Exact selection-to-native Context inspection composition.
use super::*;
use crate::context::{
    ContextBuildRequest, ContextInspectLevel, ContextInspectRequest, ContextMapHostInput,
    ContextMapPublication, ContextMapRoot, ContextMapStatus, ContextRenderSelection,
    execute_context_build, execute_context_inspect,
};

pub(in crate::search) mod build;
pub(in crate::search) mod continuation;
pub(in crate::search) mod platform;
pub use build::SearchContextBuildRequest;
pub use continuation::SearchContextContinueRequest;
pub use platform::SearchPlatformContextRequest;

enum ContextAction {
    Inspect,
    Build(ContextRenderSelection),
    ProgressiveBuild(ContextRenderSelection, Vec<Vec<ContextMapRoot>>),
}
impl ContextAction {
    fn operation(&self) -> &'static str {
        match self {
            Self::Inspect => "search_context",
            Self::Build(_) | Self::ProgressiveBuild(..) => "search_context_build",
        }
    }
    fn admission(&self) -> &'static str {
        match self {
            Self::Inspect => "exact_native_inspection",
            Self::Build(_) | Self::ProgressiveBuild(..) => "exact_native_build",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Level {
    L0,
    L1,
    Both,
}
impl Level {
    fn native(&self) -> ContextInspectLevel {
        match self {
            Self::L0 => ContextInspectLevel::L0,
            Self::L1 => ContextInspectLevel::L1,
            Self::Both => ContextInspectLevel::Both,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextInput {
    schema: String,
    selection_operation_id: String,
    selection_request_digest: String,
    selection_result_id: String,
    receipt_id: String,
    project_store_id: String,
    level: Level,
}
pub struct SearchContextRequest {
    operation_id: OperationId,
    search_store: PathBuf,
    input: ContextInput,
    host: ContextMapHostInput,
    host_binding: Box<str>,
    store_limits: SearchStoreLimits,
    max_output_bytes: NonZeroU64,
}
impl SearchContextRequest {
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
        let input: ContextInput = serde_json::from_slice(input_json).map_err(|_| invalid())?;
        if input.schema != "wow-service/search-context-input/1"
            || input.selection_operation_id == operation_id.as_str()
            || input.project_store_id != host.registration_id()
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
    request: &SearchContextRequest,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    execute_plan(request, ContextAction::Inspect, stop)
}

fn execute_plan(
    request: &SearchContextRequest,
    action: ContextAction,
    stop: &AtomicBool,
) -> ServiceResult<Box<[u8]>> {
    check_cancelled(stop)?;
    let operation = action.operation();
    let admission = action.admission();
    let (id, digest, public_digest) = match &action {
        ContextAction::Inspect => durable_identity(
            "search_context",
            &request.operation_id,
            &(
                &request.input,
                &request.host_binding,
                request.store_limits,
                request.max_output_bytes,
                "local_metadata_only",
            ),
        ),
        ContextAction::Build(render) => durable_identity(
            operation,
            &request.operation_id,
            &(
                "wow-service/search-context-build-input/1",
                &request.input,
                render,
                &request.host_binding,
                request.store_limits,
                request.max_output_bytes,
                "local_metadata_only",
            ),
        ),
        ContextAction::ProgressiveBuild(render, rounds) => durable_identity(
            operation,
            &request.operation_id,
            &(
                "wow-service/search-context-build-input/1",
                &request.input,
                render,
                rounds,
                crate::context::PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID,
                "wow-context/primary-native-progressive/1",
                crate::context::CONTEXT_CONTINUATION_RETENTION_PROFILE_ID,
                &request.host_binding,
                request.store_limits,
                request.max_output_bytes,
                "local_metadata_only",
            ),
        ),
    }?;
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
        let (receipt, explained) = load_selection(
            &store,
            &request.input,
            request.store_limits.max_object_bytes,
            stop,
        )?;
        let owner = receipt.get("owner_entity").ok_or_else(mismatch)?;
        let search_status = field(&explained, "status")?;
        let unsupported = if field(owner, "kind")? != "project_graph_entity" {
            Some("native_owner_root_mapping_unavailable")
        } else if field(&explained, "schema")? == "wow-search/federated-candidate-explanation/1" {
            match explained
                .pointer("/candidate/universe_class")
                .and_then(serde_json::Value::as_str)
            {
                Some("primary_user_project") => None,
                Some("blizzard_ui_source") => Some("qualified_platform_context_required"),
                _ => return Err(mismatch()),
            }
        } else {
            None
        };
        if let Some(reason) = unsupported {
            return durable_response(
                operation,
                &request.operation_id,
                &public_digest,
                &serde_json::json!({
                    "status":"not_evaluated","resource_closure":"closed","context_admission":"not_evaluated",
                    "reason":reason,"selection_receipt":receipt,"search_explanation":explained,
                    "context_host_binding":request.host_binding
                }),
                request.max_output_bytes.get(),
            );
        }
        let federated =
            field(&explained, "schema")? == "wow-search/federated-candidate-explanation/1";
        let summary = if federated {
            explained
                .pointer("/query_context/components/primary_user_project/result_summary")
                .ok_or_else(mismatch)?
        } else {
            explained.get("query_context").ok_or_else(mismatch)?
        };
        let source = summary.pointer("/owner/source").ok_or_else(mismatch)?;
        let entity = if federated {
            explained
                .pointer("/candidate/original_candidate/document/entity")
                .ok_or_else(mismatch)?
        } else {
            explained
                .pointer("/candidate/document/entity")
                .ok_or_else(mismatch)?
        };
        if field(entity, "node_id")? != field(owner, "id")?
            || field(entity, "universe")? != field(source, "graph_universe_id")?
            || field(entity, "generation")? != field(source, "graph_generation_id")?
        {
            return Err(mismatch());
        }
        let expected_view = if federated {
            Some(explained.pointer("/query_context/components/reference_platform_contract/result_summary/owner/reference_view_digest")
                .and_then(serde_json::Value::as_str).ok_or_else(mismatch)?)
        } else {
            None
        };
        let reference = request.host.reference_guard(
            field(source, "reference_generation_id")?,
            source.get("project_profile").ok_or_else(mismatch)?,
            expected_view,
        )?;
        let generation = field(source, "store_generation_id")?;
        let publication = ContextMapPublication::store_generation(generation)?;
        let root = ContextMapRoot::graph_entity(field(owner, "id")?)?;
        check_cancelled(stop)?;
        // Both concrete E3 operations own native custody, semantic artifacts,
        // renderer validation and checked close. No source capability supplied.
        let context = match action {
            ContextAction::Inspect => {
                let inspect = ContextInspectRequest::primary(
                    &request.input.project_store_id,
                    publication,
                    request.input.level.native(),
                    root,
                    None,
                    Some(reference),
                )?;
                execute_context_inspect(&request.host, &inspect, stop)?
            }
            ContextAction::Build(render) => {
                let build = ContextBuildRequest::primary(
                    &request.input.project_store_id,
                    publication,
                    request.input.level.native(),
                    root,
                    render,
                    None,
                    Some(reference),
                )?;
                execute_context_build(&request.host, &build, stop)?
            }
            ContextAction::ProgressiveBuild(render, rounds) => {
                let build = ContextBuildRequest::primary(
                    &request.input.project_store_id,
                    publication,
                    request.input.level.native(),
                    root,
                    render,
                    None,
                    Some(reference),
                )?
                .with_progressive_expansion(rounds)?
                .with_continuation_retention_profile(
                    crate::context::CONTEXT_CONTINUATION_RETENTION_PROFILE_ID,
                )?;
                // E3 now owns a durable issuance/retention effect. A later
                // outer failure cannot be recorded as retryable no-effect.
                native_effect = true;
                execute_context_build(&request.host, &build, stop)?
            }
        };
        let (context_value, status) = checked_native_result(
            &context,
            search_status,
            generation,
            request.max_output_bytes.get(),
            stop,
        )?;
        durable_response(
            operation,
            &request.operation_id,
            &public_digest,
            &serde_json::json!({
                "status":status,"resource_closure":"closed","context_admission":admission,
                "selection_receipt":receipt,"search_explanation":explained,"context":context_value,
                "context_host_binding":request.host_binding
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
    if native_effect {
        return complete_native_effect(
            &mut store,
            &id,
            &digest,
            &bytes,
            request.max_output_bytes.get(),
        )
        .map_err(|error| operation_failure(error, &request.operation_id));
    }
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

fn checked_native_result(
    context: &crate::context::ContextMapResult,
    search_status: &str,
    generation: &str,
    max_output: u64,
    stop: &AtomicBool,
) -> ServiceResult<(serde_json::Value, Box<str>)> {
    if context.canonical_bytes().len() > crate::context::CONTEXT_MAP_RESULT_MAX_BYTES {
        return Err(outcome_unknown());
    }
    let value: serde_json::Value =
        serde_json::from_slice(context.canonical_bytes()).map_err(|_| outcome_unknown())?;
    let closure = value
        .get("resource_closure")
        .and_then(serde_json::Value::as_str);
    if !matches!(closure, Some("closed" | "not_acquired")) {
        return Err(outcome_unknown());
    }
    if let Some(code) = context.failure_code() {
        // A typed failure/checked close does not certify no durable native
        // mutation. The caller retains its dispatch fence on every error.
        return Err(failure(code, "native selected Context inspection refused"));
    }
    if closure != Some("closed") {
        return Err(outcome_unknown());
    }
    check_output_bound(context.canonical_bytes(), max_output)?;
    let status = match context.status() {
        ContextMapStatus::Cancelled => {
            return Err(failure(
                ServiceErrorCode::Cancelled,
                "selected Context inspection cancelled",
            ));
        }
        ContextMapStatus::Failed => {
            return Err(failure(
                ServiceErrorCode::InternalContractViolation,
                "native Context failed without a typed cause",
            ));
        }
        ContextMapStatus::Partial => fold_status(search_status, "partial"),
        ContextMapStatus::Truncated => fold_status(search_status, "truncated"),
        ContextMapStatus::Complete => fold_status(search_status, "exact_found"),
    };
    if value
        .pointer("/payload/store_generation_id")
        .and_then(serde_json::Value::as_str)
        != Some(generation)
    {
        return Err(mismatch());
    }
    check_cancelled(stop)?;
    Ok((value, status.into()))
}

// Unlike the read-only journal helper, this completion must never record
// no-effect after E3 has durably issued/consumed a continuation.
fn complete_native_effect(
    store: &mut Store,
    id: &StoreOperationId,
    digest: &RequestDigest,
    bytes: &[u8],
    max_output: u64,
) -> ServiceResult<Box<[u8]>> {
    let result = (|| {
        check_output_bound(bytes, max_output)?;
        let response = response_object(store, bytes)?;
        let object_id = response.object_id().clone();
        let mut batch = WriteBatch::new();
        batch.add_object(response).map_err(map_store_error)?;
        match store.commit_operation(batch, id, digest, &object_id) {
            Ok(record) if record.result_object_id() == Some(&object_id) => {
                read_completed_response(store, &record, max_output)
            }
            _ => reconcile_durable_operation(store, id, digest, max_output),
        }
    })();
    if result.is_err() {
        let _ = store.mark_outcome_unknown(id, digest);
        return Err(outcome_unknown());
    }
    result
}

fn load_selection(
    store: &Store,
    input: &ContextInput,
    max_read_bytes: u64,
    stop: &AtomicBool,
) -> ServiceResult<(serde_json::Value, serde_json::Value)> {
    load_selection_by_ids(
        store,
        &input.selection_operation_id,
        &input.selection_request_digest,
        &input.selection_result_id,
        &input.receipt_id,
        max_read_bytes,
        stop,
    )
}

fn load_selection_by_ids(
    store: &Store,
    operation_id: &str,
    request_digest: &str,
    result_id: &str,
    receipt_id: &str,
    max_read_bytes: u64,
    stop: &AtomicBool,
) -> ServiceResult<(serde_json::Value, serde_json::Value)> {
    check_cancelled(stop)?;
    let record = store
        .operation(&StoreOperationId::new(operation_id.to_owned()).map_err(map_store_error)?)
        .map_err(map_store_error)?
        .ok_or_else(unavailable)?;
    if record.state() != StoreOperationState::Completed
        || record.request_digest().as_str() != request_digest
    {
        return Err(mismatch());
    }
    let bytes = read_completed_response(store, &record, max_read_bytes)?;
    let original: serde_json::Value = serde_json::from_slice(&bytes).map_err(|_| mismatch())?;
    if field(&original, "operation")? != "search_select"
        || field(&original, "result_id")? != result_id
    {
        return Err(mismatch());
    }
    let payload = original.get("payload").ok_or_else(mismatch)?;
    if field(payload, "resource_closure")? != "closed" {
        return Err(mismatch());
    }
    let receipt = payload.get("selection_receipt").ok_or_else(mismatch)?;
    if field(receipt, "schema")? != "wow-service/search-selection-receipt/1"
        || field(receipt, "receipt_id")? != receipt_id
    {
        return Err(mismatch());
    }
    // Revalidate the immutable original explanation through the same pure
    // materialization path; no second selection algorithm or native owner DTO.
    let reconstructed:Input=serde_json::from_value(serde_json::json!({
        "schema":"wow-service/search-selection-input/1",
        "explanation_operation_id":receipt.get("explanation_operation_id").ok_or_else(mismatch)?,
        "explanation_request_digest":receipt.get("explanation_request_digest").ok_or_else(mismatch)?,
        "explanation_result_id":receipt.get("explanation_result_id").ok_or_else(mismatch)?,
        "result_set_id":receipt.get("result_set_id").ok_or_else(mismatch)?,
        "candidate_id":receipt.get("candidate_id").ok_or_else(mismatch)?,
        "expected_explanation_digest":receipt.get("explanation_digest").ok_or_else(mismatch)?,
        "expected_candidate_digest":receipt.get("candidate_digest").ok_or_else(mismatch)?,
        "entity_kind":receipt.pointer("/owner_entity/kind").ok_or_else(mismatch)?,
        "expected_entity_id":receipt.pointer("/owner_entity/id").ok_or_else(mismatch)?,
        "selection_origin":receipt.get("selection_origin").ok_or_else(mismatch)?,
        "target_action":receipt.get("target_action").ok_or_else(mismatch)?
    })).map_err(|_|mismatch())?;
    let (expected, explained) = materialize_selection(store, &reconstructed, max_read_bytes, stop)?;
    if &expected != receipt || payload.get("status") != receipt.get("original_status") {
        return Err(mismatch());
    }
    Ok((expected, explained))
}

fn fold_status<'a>(search: &'a str, context: &'a str) -> &'a str {
    if search == "conflict_blocked" {
        "conflict_blocked"
    } else if search == "truncated" || context == "truncated" {
        "truncated"
    } else if search == "lane_unavailable" {
        "lane_unavailable"
    } else if context == "partial" {
        "partial"
    } else {
        search
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_core::{
        ContentDigest, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry, SourceKind,
        SourceLogicalSnapshot,
    };

    fn host_value(root: &std::path::Path) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let profile = ProfileIdentityBuilder::new(
            "profile:fixture:cancelled-search-context".parse()?,
            ProfileKind::Fixture,
            "retail",
            120100,
            SourceKind::SyntheticFixture,
            "fixture:cancelled-context",
            ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
        )
        .schema_versions(vec![SchemaVersionEntry::new(
            "schema:wow:fixture-e0".parse()?,
            "1.0.0".parse()?,
        )])
        .fixture_scope("inert Context host fingerprint and cancellation test")
        .build()?;
        let view = wow_reference::ReferenceView::new(
            format!("generation:reference:sha256:{}", "0".repeat(64)),
            Vec::new(),
            Vec::new(),
        )?;
        Ok(serde_json::json!({
            "schema":"wow-service/context-map-native-host/1","project_store_id":"cancelled-context",
            "project_store_root":root,"project_id":"fixture-cancelled-context",
            "reference_profile":profile,"reference":view,
            "limits":{"neighbors":wow_graph::GraphNeighborReadLimits::default(),"explain":wow_graph::GraphExplainLimits::default(),
                "evidence":wow_graph::GraphEvidenceResolveLimits::default(),"max_direct_edges":16,
                "max_input_bytes":4194304,"max_metadata_bytes":2097152,"max_context_work":100000}
        }))
    }

    #[test]
    fn host_fingerprint_excludes_routing_and_binds_budgets()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join("wow-context-inert-host");
        let original = host_value(&root)?;
        let admit = |value: &serde_json::Value| {
            ContextMapHostInput::from_json_slice(
                &serde_json::to_vec(value).map_err(|_| invalid())?,
                &AtomicBool::new(false),
            )
        };
        let digest = admit(&original)?.semantic_binding_digest()?;
        let mut routed = original.clone();
        routed["project_store_root"] = serde_json::to_value(root.join("other-route"))?;
        assert_eq!(digest, admit(&routed)?.semantic_binding_digest()?);
        routed["limits"]["max_context_work"] = serde_json::json!(100001);
        assert_ne!(digest, admit(&routed)?.semantic_binding_digest()?);
        assert!(!root.exists());
        Ok(())
    }

    #[test]
    fn cancelled_context_does_not_open_any_store() -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-context-cancelled-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        let host = ContextMapHostInput::from_json_slice(
            &serde_json::to_vec(&host_value(&root)?)?,
            &AtomicBool::new(false),
        )?;
        let zero = "0".repeat(64);
        let input = serde_json::json!({"schema":"wow-service/search-context-input/1","selection_operation_id":"inert-selection",
            "selection_request_digest":format!("sha256:{zero}"),"selection_result_id":format!("wow-service/search-durable-result/1:{zero}"),
            "receipt_id":format!("search-selection-receipt:{zero}"),"project_store_id":"cancelled-context","level":"l1"});
        let search = root.with_extension("sqlite");
        let request = SearchContextRequest::new(
            OperationId::new("cancelled-context")?,
            search.clone(),
            &serde_json::to_vec(&input)?,
            host,
            SearchStoreLimits::new(65536, 8, 16, 128, 8)?,
            NonZeroU64::new(65536).ok_or("bad output cap")?,
            LocalMetadataPermission::authorize_local_metadata_only(),
            &AtomicBool::new(false),
        )?;
        let failure = SearchOperationService::new()
            .context_for_selection(&request, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled Context succeeded")?;
        assert_eq!(failure.code(), ServiceErrorCode::Cancelled);
        assert!(!search.exists() && !root.exists());
        Ok(())
    }

    #[test]
    fn failed_outer_delivery_after_native_effect_is_unknown()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-context-effect-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        std::fs::create_dir(&root)?;
        let path = root.join("search.sqlite");
        let limits = SearchStoreLimits::new(65536, 8, 16, 128, 8)?;
        let mut store = Store::open(
            &path,
            StoreConfiguration::new_with_instance_identity(
                DURABLE_SEARCH_STORE_PROFILE_ID,
                limits,
            )?,
        )?;
        let id = StoreOperationId::new("native-effect-delivery")?;
        let digest = RequestDigest::new(format!("sha256:{}", "7".repeat(64)))?;
        store.begin_operation(id.clone(), digest.clone())?;
        let error = complete_native_effect(&mut store, &id, &digest, b"{}", 1)
            .err()
            .ok_or("small outer delivery succeeded")?;
        assert_eq!(error.code(), ServiceErrorCode::StoreOutcomeUnknown);
        assert_eq!(
            store.operation(&id)?.ok_or("missing operation")?.state(),
            StoreOperationState::OutcomeUnknown
        );
        drop(store);
        std::fs::remove_file(&path)?;
        std::fs::remove_dir(&root)?;
        Ok(())
    }
}
