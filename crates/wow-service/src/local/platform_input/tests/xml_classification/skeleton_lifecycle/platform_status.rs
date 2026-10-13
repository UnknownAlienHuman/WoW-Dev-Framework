//! Genuine exact retained platform metadata status through owner custody.
use super::*;
use crate::context::{
    CONTEXT_PLATFORM_HOST_SCHEMA, CONTEXT_PLATFORM_STATUS_REQUEST_SCHEMA,
    ContextPlatformStatusRequest, PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
    execute_context_platform_status,
};

fn status_request(
    store_generation_id: &str,
    publication_set_id: &Value,
    candidate_id: &Value,
    candidate_digest: &Value,
    detail: &str,
) -> Value {
    json!({
        "schema": CONTEXT_PLATFORM_STATUS_REQUEST_SCHEMA,
        "project_store_id": "project-store:fixture-context-platform-status",
        "store_generation_id": store_generation_id,
        "publication_set_id": publication_set_id,
        "candidate_id": candidate_id,
        "candidate_digest": candidate_digest,
        "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
        "detail": detail
    })
}

#[test]
fn exact_retained_platform_status_survives_current_advance_and_source_absence() -> TestResult {
    let stop = AtomicBool::new(false);
    let fixture = selected_skeleton_fixture(&stop)?;
    let export = platform_status_export_root(&fixture.root)?;
    let store_root = export.as_ref().map_or_else(
        || fixture.root.0.join("status-selected-store"),
        |path| path.join("registered-platform-store"),
    );
    let initial = serde_json::to_value(publish_selected_local_project(
        &fixture.path,
        &fixture.request,
        &store_root,
        &fixture.selected_transport,
        &LiveProjectPublishRequest::new("fixture:status-base", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(initial["status"], "activated");
    let original_generation = initial["operation"]["generation_id"]
        .as_str()
        .ok_or("original exact generation")?
        .to_owned();

    // A changed, valid XML source produces a genuine new candidate and advances
    // Current. The status request below still names the original generation.
    let changed_xml = format!("{CLASSIFICATION_XML}\n<!-- retained status advance -->");
    let next_fixture = selected_skeleton_fixture_from_xml_seed(&stop, false, &changed_xml)?;
    let current_store =
        LiveProjectStore::open_in_namespace(&store_root, &fixture.selected_selection)?;
    let current_before = current_store.current()?.ok_or("initial Current")?;
    current_store.close()?;
    let advanced = serde_json::to_value(update_selected_local_project(
        &next_fixture.path,
        &next_fixture.request,
        &store_root,
        &fixture.selected_transport,
        &LiveProjectUpdateRequest::new(
            "fixture:status-advance-current",
            current_before.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(advanced["status"], "activated");
    let current_generation = advanced["operation"]["generation_id"]
        .as_str()
        .ok_or("advanced exact generation")?;
    assert_ne!(current_generation, original_generation);
    let store = LiveProjectStore::open_in_namespace(&store_root, &fixture.selected_selection)?;
    let current_after = store.current()?.ok_or("advanced Current")?;
    assert_ne!(current_after.record_id, current_before.record_id);
    store.close()?;

    // Remove the original sources, then capture the genuine candidate from its
    // exact retained generation through the existing source-absent helper.
    drop(fixture.project);
    std::fs::remove_dir_all(fixture.root.0.join("UI"))?;
    std::fs::remove_file(fixture.root.0.join("version.txt"))?;
    assert!(!fixture.root.0.join("UI/main.lua").exists());
    assert!(!fixture.root.0.join("UI/classification.xsd").exists());
    let (native_request_bytes, native_skeleton) = source_absent_skeleton(
        &store_root,
        &fixture.selected_transport,
        &fixture.selected_selection,
        &original_generation,
        &fixture.ids,
        &fixture.intent,
        &stop,
    )?;
    let original_publication = native_skeleton["publication_set_id"].clone();

    let context_input = LocalProjectInput::from_config_path(&next_fixture.path, &stop)?;
    let reference = context_input.reference_view().clone();
    let reference_profile = next_fixture
        .project
        .configuration()
        .selected_profile()
        .clone();
    let selected_request: Value = serde_json::from_slice(&native_request_bytes)?;
    let input = &selected_request["input"];
    let selected_root = &store_root;
    let host_wire = json!({
        "schema": CONTEXT_PLATFORM_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-context-platform-status",
        "project_store_root": selected_root,
        "selection": fixture.selection_wire,
        "reference_profile": reference_profile,
        "reference": reference,
        "limits": {
            "neighbors": input["neighborhood"]["limits"],
            "explain": input["limits"]["root_explanation"],
            "evidence": input["limits"]["root_evidence"],
            "max_direct_edges": input["neighborhood"]["query"]["max_edges"],
            "max_input_bytes": 8_000_000,
            "max_metadata_bytes": 8_000_000,
            "max_context_work": 8_000_000
        }
    });
    let host_bytes = wow_core::canonical_json_bytes(&host_wire)?;
    let host = ContextPlatformHostInput::from_json_slice(&host_bytes, &stop)?;
    let mut baseline = Vec::new();
    for operation in ["map", "inspect"] {
        let wire = json!({
            "schema": CONTEXT_PLATFORM_REQUEST_SCHEMA,
            "operation": operation,
            "project_store_id": "project-store:fixture-context-platform-status",
            "store_generation_id": original_generation,
            "publication_set_id": original_publication,
            "candidate_id": native_skeleton["candidate_id"],
            "candidate_digest": native_skeleton["candidate_digest"],
            "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
            "input": selected_request["input"]
        });
        let request = ContextPlatformRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wire)?,
            &stop,
        )?;
        let result = execute_context_platform(&host, &request, &stop)?;
        let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
        assert_eq!(envelope["resource_closure"], "closed");
        baseline.push(result.canonical_bytes().to_vec());
    }

    let mut expected_status_bytes = None;
    let mut summary_request_bytes = None;
    for detail in ["summary", "capabilities", "profiles", "generations"] {
        let mut wire = status_request(
            &original_generation,
            &original_publication,
            &native_skeleton["candidate_id"],
            &native_skeleton["candidate_digest"],
            detail,
        );
        if detail == "summary" {
            wire["reference"] = json!({
                "profile": reference_profile.profile_id(),
                "generation": reference.generation_id(),
                "view": reference.self_digest()
            });
        }
        let request_bytes = wow_core::canonical_json_bytes(&wire)?;
        if detail == "summary" {
            summary_request_bytes = Some(request_bytes.clone());
        }
        let request = ContextPlatformStatusRequest::from_json_slice(&request_bytes, &stop)?;
        let result = execute_context_platform_status(&host, &request, &stop)?;
        assert_eq!(result.status(), ContextMapStatus::Partial);
        assert_eq!(result.failure_code(), None);
        if detail == "summary" {
            expected_status_bytes = Some(result.canonical_bytes().to_vec());
        }
        let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
        assert_eq!(
            envelope["schema"],
            "wow-service/context-platform-status-result/1"
        );
        assert_eq!(envelope["operation"], "context_platform_status");
        assert_eq!(envelope["status"], "partial");
        assert_eq!(envelope["resource_closure"], "closed");
        assert!(envelope.get("failure_code").is_none());
        let payload = &envelope["payload"];
        assert_eq!(
            payload["optional_platform_status"],
            "selected_metadata_only"
        );
        assert_eq!(payload["store_generation_id"], original_generation);
        assert_eq!(payload["publication_set_id"], original_publication);
        assert_eq!(payload["candidate_id"], native_skeleton["candidate_id"]);
        assert_eq!(
            payload["candidate_digest"],
            native_skeleton["candidate_digest"]
        );
        for forbidden in ["artifact", "skeleton", "root", "source_body", "source_text"] {
            assert!(
                payload.get(forbidden).is_none(),
                "status excludes {forbidden}"
            );
        }
        match detail {
            "summary" => {
                assert!(payload.get("capabilities").is_none());
                assert!(payload.get("profiles").is_none());
                assert!(payload.get("generations").is_none());
            }
            "capabilities" => {
                assert!(payload["capabilities"].is_array());
                assert!(payload.get("profiles").is_none());
                assert!(payload.get("generations").is_none());
            }
            "profiles" => {
                assert!(payload["profiles"].is_array());
                assert!(payload.get("capabilities").is_none());
                assert!(payload.get("generations").is_none());
            }
            "generations" => {
                assert!(payload["generations"].is_object());
                assert!(payload.get("capabilities").is_none());
                assert!(payload.get("profiles").is_none());
            }
            _ => unreachable!(),
        }
    }

    // Wrong exact identities and incomplete caller guards fail closed.
    for (field, bad) in [
        ("candidate_id", json!("candidate:wrong")),
        ("publication_set_id", json!("publication-set:wrong")),
        ("project_store_id", json!("project-store:wrong")),
    ] {
        let mut wire = status_request(
            &original_generation,
            &original_publication,
            &native_skeleton["candidate_id"],
            &native_skeleton["candidate_digest"],
            "summary",
        );
        wire[field] = bad;
        let request = ContextPlatformStatusRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wire)?,
            &stop,
        )?;
        let result = execute_context_platform_status(&host, &request, &stop)?;
        assert!(result.failure_code().is_some());
        let refused: Value = serde_json::from_slice(result.canonical_bytes())?;
        assert_eq!(
            refused["resource_closure"],
            if field == "project_store_id" {
                "not_acquired"
            } else {
                "closed"
            }
        );
        assert!(refused.get("payload").is_none());
    }
    let mut wrong_candidate_digest = status_request(
        &original_generation,
        &original_publication,
        &native_skeleton["candidate_id"],
        &native_skeleton["candidate_digest"],
        "summary",
    );
    let mut digest = wrong_candidate_digest["candidate_digest"]
        .as_str()
        .ok_or("genuine candidate digest")?
        .to_owned();
    let last = digest.pop().ok_or("nonempty candidate digest")?;
    digest.push(if last == '0' { '1' } else { '0' });
    wrong_candidate_digest["candidate_digest"] = json!(digest);
    let request = ContextPlatformStatusRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&wrong_candidate_digest)?,
        &stop,
    )?;
    let refused = execute_context_platform_status(&host, &request, &stop)?;
    assert!(refused.failure_code().is_some());
    assert!(serde_json::from_slice::<Value>(refused.canonical_bytes())?["payload"].is_null());
    let mut wrong_reference = status_request(
        &original_generation,
        &original_publication,
        &native_skeleton["candidate_id"],
        &native_skeleton["candidate_digest"],
        "summary",
    );
    wrong_reference["reference"] = json!({
        "profile": reference_profile.profile_id(),
        "generation": reference.generation_id(),
        "view": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    });
    let request = ContextPlatformStatusRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&wrong_reference)?,
        &stop,
    )?;
    let refused = execute_context_platform_status(&host, &request, &stop)?;
    assert!(refused.failure_code().is_some());
    assert_eq!(
        serde_json::from_slice::<Value>(refused.canonical_bytes())?["resource_closure"],
        "closed"
    );
    wrong_reference["reference"] = json!({
        "profile": reference_profile.profile_id(),
        "generation": reference.generation_id()
    });
    assert!(
        ContextPlatformStatusRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wrong_reference)?,
            &stop,
        )
        .is_err()
    );

    // Registered host admission still requires the composed candidate catalog.
    let mut no_candidate_host = host_wire.clone();
    no_candidate_host["selection"]["catalogs"]["composed_candidate"] = json!(false);
    assert!(
        ContextPlatformHostInput::from_json_slice(
            &wow_core::canonical_json_bytes(&no_candidate_host)?,
            &stop,
        )
        .is_err()
    );

    let request_wire = status_request(
        &original_generation,
        &original_publication,
        &native_skeleton["candidate_id"],
        &native_skeleton["candidate_digest"],
        "summary",
    );
    let request_bytes = wow_core::canonical_json_bytes(&request_wire)?;
    let request = ContextPlatformStatusRequest::from_json_slice(&request_bytes, &stop)?;
    let cancelled = AtomicBool::new(true);
    let cancelled_result = execute_context_platform_status(&host, &request, &cancelled)?;
    assert!(cancelled_result.failure_code().is_some());
    assert_eq!(
        serde_json::from_slice::<Value>(cancelled_result.canonical_bytes())?["resource_closure"],
        "not_acquired"
    );

    let mut limited_wire = host_wire.clone();
    limited_wire["limits"]["max_context_work"] = json!(1);
    let limited_host = ContextPlatformHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&limited_wire)?,
        &stop,
    )?;
    let work_refusal = execute_context_platform_status(&limited_host, &request, &stop)?;
    assert_eq!(
        work_refusal.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(work_refusal.canonical_bytes())?["resource_closure"],
        "closed"
    );

    let mut tiny_wire = host_wire.clone();
    tiny_wire["limits"]["max_metadata_bytes"] = json!(128);
    let tiny_host = ContextPlatformHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&tiny_wire)?,
        &stop,
    )?;
    let metadata_refusal = execute_context_platform_status(&tiny_host, &request, &stop)?;
    assert_eq!(
        metadata_refusal.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );

    // A checked status call leaves the retained exact reader/store reopenable;
    // the original generation remains readable after Current advanced.
    let reopened = LiveProjectStore::open_in_namespace(&store_root, &fixture.selected_selection)?;
    let exact = reopened.read_in_namespace(
        &fixture.selected_selection,
        &ReadSelector::Exact(StoreGenerationId::parse(&original_generation)?),
        &stop,
    )?;
    assert_eq!(exact.store_generation_id().as_str(), original_generation);
    exact.close()?;
    reopened.close()?;
    for operation in ["map", "inspect"] {
        let wire = json!({
            "schema": CONTEXT_PLATFORM_REQUEST_SCHEMA,
            "operation": operation,
            "project_store_id": "project-store:fixture-context-platform-status",
            "store_generation_id": original_generation,
            "publication_set_id": original_publication,
            "candidate_id": native_skeleton["candidate_id"],
            "candidate_digest": native_skeleton["candidate_digest"],
            "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
            "input": selected_request["input"]
        });
        let request = ContextPlatformRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wire)?,
            &stop,
        )?;
        let result = execute_context_platform(&host, &request, &stop)?;
        assert_eq!(
            result.canonical_bytes(),
            baseline[if operation == "map" { 0 } else { 1 }]
        );
    }
    if let Some(directory) = export {
        use std::io::Write;
        for (name, bytes) in [
            ("host.json", host_bytes),
            (
                "status-request.json",
                summary_request_bytes
                    .take()
                    .ok_or("guarded summary request bytes")?,
            ),
            (
                "expected-envelope.json",
                expected_status_bytes
                    .take()
                    .ok_or("summary envelope bytes")?,
            ),
        ] {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(name))?
                .write_all(&bytes)?;
        }
    }
    drop(context_input);
    drop(next_fixture.project);
    Ok(())
}

fn platform_status_export_root(fixture: &FixtureRoot) -> TestResult<Option<PathBuf>> {
    let Some(raw) = std::env::var_os("WOW_CONTEXT_PLATFORM_STATUS_EXPORT") else {
        return Ok(None);
    };
    let path = PathBuf::from(raw);
    if !path.is_absolute() || path.as_os_str().len() > 32768 {
        return Err("platform status export requires a finite absolute fresh path".into());
    }
    let parent = path
        .parent()
        .ok_or("status export parent")?
        .canonicalize()?;
    let path = parent.join(path.file_name().ok_or("status export directory")?);
    if path.starts_with(fixture.0.canonicalize()?) {
        return Err("status export must survive fixture/source removal".into());
    }
    std::fs::create_dir(&path)?;
    Ok(Some(path))
}
