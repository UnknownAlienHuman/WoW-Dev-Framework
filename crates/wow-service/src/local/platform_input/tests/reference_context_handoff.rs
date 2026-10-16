//! One real retained Reference query/explain/selection -> native Context pipeline.
use super::*;
use crate::OperationId;
use crate::search::{
    LocalMetadataPermission, ReferenceExplainRequest, ReferenceIndexBuildRequest,
    ReferenceShardRequest, SearchBounds, SearchContextBuildRequest, SearchContextRequest,
    SearchOperationService, SearchSelectionRequest, SearchStoreLimits,
};
use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

fn limits() -> TestResult<SearchStoreLimits> {
    Ok(SearchStoreLimits::new(16_777_216, 16, 32, 512, 16)?)
}
fn output() -> TestResult<NonZeroU64> {
    NonZeroU64::new(16_777_216).ok_or_else(|| "zero output".into())
}
fn permission() -> LocalMetadataPermission {
    LocalMetadataPermission::authorize_local_metadata_only()
}
fn bytes(value: &Value) -> TestResult<Vec<u8>> {
    Ok(wow_core::canonical_json_bytes(value)?)
}
fn id(value: &str) -> TestResult<OperationId> {
    Ok(OperationId::new(value)?)
}
fn hash(value: &Value) -> TestResult<String> {
    Ok(format!("sha256:{:x}", Sha256::digest(bytes(value)?)))
}
fn save(root: &Path, name: &str, value: &Value) -> TestResult {
    std::fs::write(root.join(name), bytes(value)?)?;
    Ok(())
}

pub(super) fn prepare(source: &Path, native: &Value, root: &Path, stop: &AtomicBool) -> TestResult {
    let mut config = native.clone();
    config["schema"] = json!(crate::local::SOURCE_REFERENCE_INPUT_SCHEMA);
    let config_path = source.join("reference-context-source.json");
    let config_bytes = bytes(&config)?;
    std::fs::write(&config_path, &config_bytes)?;
    let n32 = |n| NonZeroU32::new(n).ok_or("zero fixture bound");
    let n64 = |n| NonZeroU64::new(n).ok_or("zero fixture bound");
    let bounds = SearchBounds::new(
        n32(16)?,
        n32(256)?,
        n32(1024)?,
        n32(128)?,
        n32(16)?,
        n32(512)?,
        n64(1_048_576)?,
        n64(1_048_576)?,
        n64(4_194_304)?,
    )?;
    let request = ReferenceIndexBuildRequest::new(
        id("reference-context-build-shard")?,
        config_path,
        &format!("sha256:{:x}", Sha256::digest(config_bytes)),
        root.join("search.sqlite"),
        bounds,
        limits()?,
        output()?,
        permission(),
    )?;
    let result = SearchOperationService::default().build_reference_shard(&request, stop)?;
    save(
        root,
        "reference-build.json",
        &serde_json::from_slice(&result)?,
    )
}

pub(super) fn verify(
    root: &Path,
    host: &Value,
    generation: &str,
    key: &str,
    direct_inspect: &[u8],
    direct_build: &[u8],
    stop: &AtomicBool,
) -> TestResult {
    let built: Value = serde_json::from_slice(&std::fs::read(root.join("reference-build.json"))?)?;
    let query = json!({"schema":"wow-search/native-reference-query/1","selectors":{"entity_key":key},
        "enabled_lanes":["exact_identity"],"required_lanes":["exact_identity"],
        "max_scanned_documents":16,"max_candidates":16,"max_signals":16,"max_result_bytes":1_048_576});
    let policy = json!({"schema":"wow-search/native-reference-page-policy/1","page_size":16,"max_pages":2,
        "max_delivered_candidates":16,"max_delivered_candidate_bytes":1_048_576,
        "max_delivered_page_bytes":2_097_152,"max_replayed_manifest_bytes":2_097_152});
    let query_request = ReferenceShardRequest::new(
        id("reference-context-query")?,
        root.join("search.sqlite"),
        &bytes(&built["payload"]["handle"])?,
        Some(&bytes(&query)?),
        limits()?,
        output()?,
        2000,
        permission(),
        stop,
    )?
    .with_pagination_json(&bytes(&policy)?, stop)?;
    let queried: Value = serde_json::from_slice(
        &SearchOperationService::default().query_reference_shard(&query_request, stop)?,
    )?;
    let page = &queried["payload"]["page"];
    let candidates = page["candidates"]
        .as_array()
        .ok_or("Reference page candidates")?;
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0]["document"]["entity_key"], key);
    let explain_input = json!({"schema":"wow-service/native-reference-explain-input/1",
        "session_object_id":queried["payload"]["session_object_id"],"result_set_id":page["result_set_id"],
        "candidate_id":candidates[0]["candidate_id"]});
    let explain_request = ReferenceExplainRequest::new(
        id("reference-context-explain")?,
        root.join("search.sqlite"),
        &bytes(&explain_input)?,
        limits()?,
        output()?,
        2000,
        permission(),
        stop,
    )?;
    let explained: Value = serde_json::from_slice(
        &SearchOperationService::default().explain_reference_candidate(&explain_request, stop)?,
    )?;
    let original = &explained["payload"]["explanation"];
    let selection = json!({"schema":"wow-service/search-selection-input/1",
        "explanation_operation_id":explained["operation_id"],"explanation_request_digest":explained["request_digest"],
        "explanation_result_id":explained["result_id"],"result_set_id":original["result_set_id"],
        "candidate_id":original["candidate_id"],"expected_explanation_digest":hash(original)?,
        "expected_candidate_digest":hash(&original["candidate"])?,"entity_kind":"reference_document",
        "expected_entity_id":key,"selection_origin":"explicit_caller_selection","target_action":"receipt_only"});
    let selected_request = SearchSelectionRequest::new(
        id("reference-context-select")?,
        root.join("search.sqlite"),
        &bytes(&selection)?,
        limits()?,
        output()?,
        permission(),
        stop,
    )?;
    let selected: Value = serde_json::from_slice(
        &SearchOperationService::default().select_candidate(&selected_request, stop)?,
    )?;
    let input = json!({"schema":"wow-service/search-context-input/2",
        "selection_operation_id":selected["operation_id"],"selection_request_digest":selected["request_digest"],
        "selection_result_id":selected["result_id"],"receipt_id":selected["payload"]["selection_receipt"]["receipt_id"],
        "project_store_id":host["project_store_id"],"level":"l1",
        "reference_projection":{"project_store_generation_id":generation}});
    let host_input = || -> TestResult<crate::context::ContextMapHostInput> {
        Ok(crate::context::ContextMapHostInput::from_json_slice(
            &bytes(host)?,
            stop,
        )?)
    };
    let inspect = SearchContextRequest::new(
        id("reference-context-inspect")?,
        root.join("search.sqlite"),
        &bytes(&input)?,
        host_input()?,
        limits()?,
        output()?,
        permission(),
        stop,
    )?;
    let inspected = SearchOperationService::default().context_for_selection(&inspect, stop)?;
    let result: Value = serde_json::from_slice(&inspected)?;
    assert_eq!(
        result["payload"]["context_admission"],
        "exact_native_reference_projection"
    );
    assert_eq!(bytes(&result["payload"]["context"])?, direct_inspect);
    assert_eq!(
        SearchOperationService::default().context_for_selection(&inspect, stop)?,
        inspected
    );
    let mut build_input = input.clone();
    build_input["schema"] = "wow-service/search-context-build-input/2".into();
    build_input["render"] = "both".into();
    let build = SearchContextBuildRequest::new(
        id("reference-context-pack")?,
        root.join("search.sqlite"),
        &bytes(&build_input)?,
        host_input()?,
        limits()?,
        output()?,
        permission(),
        stop,
    )?;
    let packed = SearchOperationService::default().build_context_for_selection(&build, stop)?;
    let packed_value: Value = serde_json::from_slice(&packed)?;
    assert_eq!(bytes(&packed_value["payload"]["context"])?, direct_build);
    assert_eq!(
        SearchOperationService::default().build_context_for_selection(&build, stop)?,
        packed
    );
    for (name, value) in [
        ("host.json", host),
        ("context-input.json", &input),
        ("build-input.json", &build_input),
        ("query.json", &queried),
        ("explanation.json", &explained),
        ("selection.json", &selected),
        ("context.json", &result),
        ("pack.json", &packed_value),
    ] {
        save(root, name, value)?;
    }
    std::fs::write(root.join("direct-inspect.json"), direct_inspect)?;
    std::fs::write(root.join("direct-build.json"), direct_build)?;
    Ok(())
}
