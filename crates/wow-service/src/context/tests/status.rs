//! Synthetic retained status lifecycle; no live/Ketho/runtime authority.
use super::*;
use std::{fs::OpenOptions, io::Write};

const STATUS_EXPORT_ENV: &str = "WOW_CONTEXT_STATUS_EXPORT";
const STATUS_REQUEST_SCHEMA: &str = "wow-service/context-status-request/1";
const STATUS_RESULT_SCHEMA: &str = "wow-service/context-status-result/1";
const NEXT_LUA: &str = "local function ContextFixture(value)\n    return value + 1\nend\n";

struct StatusScratch {
    root: PathBuf,
    preserve: bool,
}
impl StatusScratch {
    fn new() -> TestResult<Self> {
        let selected = std::env::var_os(STATUS_EXPORT_ENV);
        let preserve = selected.is_some();
        let root = match selected {
            Some(path) => PathBuf::from(path),
            None => std::env::temp_dir().join(format!(
                "wow-context-status-synthetic-{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            )),
        };
        if !root.is_absolute() || root.as_os_str().len() > 32768 {
            return Err("status witness export requires a finite absolute fresh path".into());
        }
        // Own only a newly created directory; never overwrite or adopt an existing export.
        std::fs::create_dir(&root)?;
        Ok(Self { root, preserve })
    }
}
impl Drop for StatusScratch {
    fn drop(&mut self) {
        if !self.preserve {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn status_request(
    publication: ContextMapPublication,
    detail: ContextStatusDetail,
    current: Option<ContextMapCurrentGuard>,
    profile: &ProfileIdentity,
    reference: &ReferenceView,
) -> TestResult<ContextStatusRequest> {
    Ok(ContextStatusRequest::primary(
        STORE_ID,
        publication,
        detail,
        current,
        Some(reference_guard(profile, reference)?),
    )?)
}

fn assert_status(
    result: &ContextMapResult,
    expected_store_generation: &str,
    expected_publication_set: &str,
    expected_current: &CurrentPublication,
    profile: &ProfileIdentity,
    reference: &ReferenceView,
    detail: ContextStatusDetail,
) -> TestResult<Value> {
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], STATUS_RESULT_SCHEMA);
    assert_eq!(envelope["operation"], "context_status");
    assert_eq!(envelope["status"], "partial");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(payload["configured_registration_id"], STORE_ID);
    assert_eq!(payload["store_generation_id"], expected_store_generation);
    assert_eq!(payload["publication_set_id"], expected_publication_set);
    assert_eq!(
        payload["current_at_acquisition"],
        serde_json::to_value(expected_current)?
    );
    assert_eq!(
        payload["current_record_digest"],
        observed_current_digest(expected_current)?
    );
    assert_eq!(payload["project_id"], "fixture-context-primary-map");
    assert_eq!(
        payload["reference_profile_id"],
        profile.profile_id().as_str()
    );
    assert_eq!(
        payload["reference_generation_id"],
        reference.generation_id()
    );
    assert_eq!(payload["reference_view_digest"], reference.self_digest());
    assert!(payload["project_coverage"].as_array().is_some());
    assert!(payload["reference_coverage"].as_array().is_some());
    assert!(payload["graph_coverage"].as_array().is_some());
    assert!(payload["reference_conflict_count"].as_u64().is_some());
    assert!(payload["file_count"].as_u64().is_some());
    assert!(payload["graph_node_count"].as_u64().is_some());
    assert!(payload["graph_edge_count"].as_u64().is_some());
    assert_eq!(payload["context_generation"], "NotEvaluated");
    assert_eq!(payload["evaluation"], "NotEvaluated");
    assert_eq!(payload["continuation_retention"], "not_implemented");
    assert_eq!(payload["optional_platform_status"], "not_selected");
    assert_eq!(payload["last_known_good"], "NotEvaluated");
    assert_eq!(payload["failed_targets"], "NotEvaluated");

    match detail {
        ContextStatusDetail::Summary => {
            for key in ["capabilities", "profiles", "generations"] {
                assert!(payload.get(key).is_none(), "summary omits {key}");
            }
        }
        ContextStatusDetail::Capabilities => {
            assert!(
                payload
                    .get("capabilities")
                    .and_then(Value::as_array)
                    .is_some()
            );
            assert!(payload.get("profiles").is_none());
            assert!(payload.get("generations").is_none());
        }
        ContextStatusDetail::Profiles => {
            assert!(payload.get("capabilities").is_none());
            assert!(payload.get("profiles").and_then(Value::as_array).is_some());
            assert!(payload.get("generations").is_none());
        }
        ContextStatusDetail::Generations => {
            assert!(payload.get("capabilities").is_none());
            assert!(payload.get("profiles").is_none());
            let generations = payload
                .get("generations")
                .and_then(Value::as_object)
                .ok_or("generation detail")?;
            for key in [
                "project_snapshot_id",
                "project_generation",
                "graph_snapshot_id",
                "graph_generation",
                "graph_universe",
                "graph_registry_digest",
                "source_registry_id",
                "analyzer_snapshot_id",
            ] {
                assert!(
                    generations.get(key).is_some_and(|value| !value.is_null()),
                    "generation detail retains {key}"
                );
            }
        }
    }

    let bytes = std::str::from_utf8(result.canonical_bytes())?;
    for body in [LUA, NEXT_LUA, PROJECT_OTHER_LUA] {
        assert!(!bytes.contains(body));
        assert!(!bytes.contains(&serde_json::to_string(body)?));
    }
    for forbidden in [
        "project_map",
        "l0_skeleton",
        "l1_skeleton",
        "semantic_pack",
        "source_excerpt",
        "rendered_artifact",
    ] {
        assert!(!contains_key_recursive(payload, forbidden));
    }
    assert!(!bytes.contains("project_store_root"));
    Ok(envelope)
}

fn contains_key_recursive(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(object) => {
            object.contains_key(key)
                || object
                    .values()
                    .any(|value| contains_key_recursive(value, key))
        }
        Value::Array(values) => values
            .iter()
            .any(|value| contains_key_recursive(value, key)),
        _ => false,
    }
}

fn assert_status_refused(
    result: &ContextMapResult,
    code: ServiceErrorCode,
    expected_closure: &str,
    stop: bool,
) -> TestResult {
    assert_eq!(result.failure_code(), Some(code));
    assert_eq!(
        result.status(),
        if stop {
            ContextMapStatus::Cancelled
        } else {
            ContextMapStatus::Failed
        }
    );
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], STATUS_RESULT_SCHEMA);
    assert_eq!(envelope["failure_code"], serde_json::to_value(code)?);
    assert_eq!(envelope["resource_closure"], expected_closure);
    assert!(envelope.get("payload").is_none());
    Ok(())
}

fn export_status_witness(
    root: &Path,
    host: &Value,
    request: &ContextStatusRequest,
    result: &ContextMapResult,
) -> TestResult {
    let host_bytes = wow_core::canonical_json_bytes(host)?;
    let request_bytes = wow_core::canonical_json_bytes(request)?;
    for (name, bytes, limit) in [
        ("host.json", host_bytes, CONTEXT_MAP_HOST_INPUT_MAX_BYTES),
        ("status-request.json", request_bytes, 64 * 1024),
        (
            "expected-envelope.json",
            result.canonical_bytes().to_vec(),
            CONTEXT_MAP_RESULT_MAX_BYTES,
        ),
    ] {
        if bytes.len() > limit {
            return Err("status witness export exceeded its byte bound".into());
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join(name))?;
        file.write_all(&bytes)?;
    }
    Ok(())
}

#[test]
fn retained_context_status_reports_exact_generations_details_and_closes_without_artifacts()
-> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = StatusScratch::new()?;
    let store_root = scratch.root.join("registered-store");
    let (publisher, graph, reference, profile) = owners(LUA, &stop)?;
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let first = store.publish(
        &publisher,
        &graph,
        "fixture-context-status-first",
        None,
        &stop,
    )?;
    let first_current = store.current()?.ok_or("first retained Current")?;
    assert_eq!(first_current.generation_id, first.generation_id);
    store.close()?;
    drop(graph);
    drop(publisher);
    let first_observation = retained(&store_root, &first.generation_id, &stop)?;

    // Publish a genuine second native generation so Current and exact-old selection differ.
    let (publisher, graph, next_reference, next_profile) = owners(NEXT_LUA, &stop)?;
    assert_eq!(reference, next_reference);
    assert_eq!(profile, next_profile);
    let mut store = LiveProjectStore::open(&store_root)?;
    let second = store.publish(
        &publisher,
        &graph,
        "fixture-context-status-second",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let second_current = store.current()?.ok_or("second retained Current")?;
    assert_eq!(second_current.generation_id, second.generation_id);
    assert_ne!(first_current.record_id, second_current.record_id);
    store.close()?;
    drop(graph);
    drop(publisher);
    let second_observation = retained(&store_root, &second.generation_id, &stop)?;
    assert_eq!(second_observation.current, second_current);

    let mut host_json = host_value(&store_root, &profile, &reference);
    host_json["limits"]["max_context_work"] = json!(8_000_000);
    host_json["limits"]["max_metadata_bytes"] = json!(4 * 1024 * 1024);
    let host = decode_host(&host_json, &stop)?;

    // Capture the existing command output before status executes.
    let map_request = exact_request(
        &first.generation_id,
        &first_observation.root,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let map_before = execute_context_map(&host, &map_request, &stop)?;

    let old_exact = ContextStatusRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(first.generation_id.as_str())?,
        ContextStatusDetail::Generations,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let old_result = execute_context_status(&host, &old_exact, &stop)?;
    assert_status(
        &old_result,
        first.generation_id.as_str(),
        &first_observation.publication_set_id,
        &second_current,
        &profile,
        &reference,
        ContextStatusDetail::Generations,
    )?;
    let old_envelope: Value = serde_json::from_slice(old_result.canonical_bytes())?;
    assert_ne!(
        old_envelope["payload"]["store_generation_id"],
        old_envelope["payload"]["current_at_acquisition"]["generation_id"]
    );
    assert_eq!(
        old_envelope["payload"]["source_context_id"],
        first_observation.source_context_id
    );

    let current_guard = ContextMapCurrentGuard::new(
        Some(second_current.record_id.as_str()),
        Some(&observed_current_digest(&second_current)?),
    )?;
    let current_request = status_request(
        ContextMapPublication::Current,
        ContextStatusDetail::Summary,
        Some(current_guard),
        &profile,
        &reference,
    )?;
    let request_json: Value =
        serde_json::from_slice(&wow_core::canonical_json_bytes(&current_request)?)?;
    assert_eq!(request_json["schema"], STATUS_REQUEST_SCHEMA);
    let current_result = execute_context_status(&host, &current_request, &stop)?;
    let current_envelope = assert_status(
        &current_result,
        second.generation_id.as_str(),
        &second_observation.publication_set_id,
        &second_current,
        &profile,
        &reference,
        ContextStatusDetail::Summary,
    )?;
    assert_eq!(
        current_envelope["payload"]["source_context_id"],
        second_observation.source_context_id
    );
    let current_bytes = std::str::from_utf8(current_result.canonical_bytes())?;
    assert!(!current_bytes.contains(&scratch.root.to_string_lossy().to_string()));
    assert!(!current_bytes.contains(&store_root.to_string_lossy().to_string()));

    for detail in [
        ContextStatusDetail::Capabilities,
        ContextStatusDetail::Profiles,
    ] {
        let request = status_request(
            ContextMapPublication::Current,
            detail,
            None,
            &profile,
            &reference,
        )?;
        let result = execute_context_status(&host, &request, &stop)?;
        let envelope = assert_status(
            &result,
            second.generation_id.as_str(),
            &second_observation.publication_set_id,
            &second_current,
            &profile,
            &reference,
            detail,
        )?;
        if detail == ContextStatusDetail::Capabilities {
            let capabilities = envelope["payload"]["capabilities"]
                .as_array()
                .ok_or("capabilities")?;
            assert!(capabilities.iter().any(|row| {
                row["operation"] == "context_continue" && row["availability"] == "not_implemented"
            }));
            assert!(capabilities.iter().any(|row| {
                row["operation"] == "context_validate" && row["availability"] == "not_implemented"
            }));
            assert!(capabilities.iter().any(|row| {
                row["operation"] == "context_render" && row["availability"] == "not_implemented"
            }));
            assert!(capabilities.iter().any(|row| {
                row["operation"] == "context_map" && row["availability"] == "partial_functional"
            }));
        } else {
            let profiles = envelope["payload"]["profiles"]
                .as_array()
                .ok_or("profiles")?;
            assert!(profiles.iter().any(|row| {
                row["profile_id"] == PRIMARY_NAVIGATION_MAP_PROFILE_ID
                    && row["availability"] == "partial_functional"
            }));
            assert!(profiles.iter().any(|row| {
                row["profile_id"] == PRIMARY_CONTEXT_PACK_PROFILE_ID
                    && row["availability"] == "partial_functional"
            }));
            for profile_id in [
                PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
                COMBINED_NATIVE_MAP_PROFILE_ID,
                COMBINED_CONTEXT_PACK_PROFILE_ID,
                COMBINED_CONTEXT_JSON_RENDERER_PROFILE_ID,
                COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            ] {
                assert!(profiles.iter().any(|row| {
                    row["profile_id"] == profile_id
                        && row["availability"] == "requires_platform_selection"
                }));
            }
        }
    }

    // Preserve the existing map command's exact output across status calls.
    let map_after = execute_context_map(&host, &map_request, &stop)?;
    assert_eq!(map_before.canonical_bytes(), map_after.canonical_bytes());

    if scratch.preserve {
        export_status_witness(&scratch.root, &host_json, &current_request, &current_result)?;
    }

    // A stale Current guard refuses after the new Current has been observed and closes the read.
    let stale_guard = ContextMapCurrentGuard::new(
        Some(first_current.record_id.as_str()),
        Some(&observed_current_digest(&first_current)?),
    )?;
    let stale = status_request(
        ContextMapPublication::Current,
        ContextStatusDetail::Summary,
        Some(stale_guard),
        &profile,
        &reference,
    )?;
    let refused = execute_context_status(&host, &stale, &stop)?;
    assert_status_refused(
        &refused,
        ServiceErrorCode::StoreCurrentConflict,
        "closed",
        false,
    )?;

    let wrong_generation =
        ReferenceGenerationId::derive(&"fixture:wrong-context-status-reference")?.to_string();
    let wrong_reference = ContextMapReferenceGuard::new(
        profile.profile_id().as_str(),
        &wrong_generation,
        reference.self_digest(),
    )?;
    let wrong_reference_request = ContextStatusRequest::primary(
        STORE_ID,
        ContextMapPublication::Current,
        ContextStatusDetail::Summary,
        None,
        Some(wrong_reference),
    )?;
    let refused = execute_context_status(&host, &wrong_reference_request, &stop)?;
    assert_status_refused(
        &refused,
        ServiceErrorCode::IdentityMismatch,
        "closed",
        false,
    )?;

    assert_status_refused(
        &execute_context_status(&host, &current_request, &AtomicBool::new(true))?,
        ServiceErrorCode::Cancelled,
        "not_acquired",
        true,
    )?;
    let mut low_work_json = host_json.clone();
    low_work_json["limits"]["max_context_work"] = json!(1);
    let low_work = decode_host(&low_work_json, &stop)?;
    assert_status_refused(
        &execute_context_status(&low_work, &current_request, &stop)?,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        false,
    )?;

    // Summary pays only for metadata it traverses, not either optional registry.
    let payload = &current_envelope["payload"];
    let graph_coverage = payload["graph_coverage"]
        .as_array()
        .ok_or("graph coverage")?;
    let selected_work = 1
        + payload["project_coverage"]
            .as_array()
            .ok_or("project coverage")?
            .len()
        + payload["project_deferred_capabilities"]
            .as_array()
            .ok_or("deferred")?
            .len()
        + payload["reference_coverage"]
            .as_array()
            .ok_or("reference coverage")?
            .len()
        + graph_coverage.len()
        + graph_coverage
            .iter()
            .map(|row| row["coverage"].as_array().map_or(0, Vec::len))
            .sum::<usize>();
    let mut selected_work_json = host_json.clone();
    selected_work_json["limits"]["max_context_work"] = json!(selected_work);
    let selected_work_host = decode_host(&selected_work_json, &stop)?;
    assert_eq!(
        execute_context_status(&selected_work_host, &current_request, &stop)?.failure_code(),
        None
    );
    let profiles_request = status_request(
        ContextMapPublication::Current,
        ContextStatusDetail::Profiles,
        None,
        &profile,
        &reference,
    )?;
    assert_status_refused(
        &execute_context_status(&selected_work_host, &profiles_request, &stop)?,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        false,
    )?;

    // Input admission refuses before acquisition, even for a missing store path.
    let mut low_input_json = host_json.clone();
    low_input_json["project_store_root"] = json!(scratch.root.join("absent-store"));
    low_input_json["limits"]["max_input_bytes"] = json!(1);
    let low_input = decode_host(&low_input_json, &stop)?;
    assert_status_refused(
        &execute_context_status(&low_input, &current_request, &stop)?,
        ServiceErrorCode::BudgetExceeded,
        "not_acquired",
        false,
    )?;

    // Reopen after all success and refusal paths; no status lease remains held.
    let after = retained(&store_root, &second.generation_id, &stop)?;
    assert_eq!(after.current, second_current);
    assert_eq!(
        after.publication_set_id,
        second_observation.publication_set_id
    );
    Ok(())
}
