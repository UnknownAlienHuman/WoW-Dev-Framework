//! Composed status lifecycle over genuine exact retained owners.
use super::*;
use crate::context::{
    CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES, CONTEXT_COMBINED_STATUS_REQUEST_SCHEMA,
    ContextCombinedStatusRequest, execute_context_combined_status,
};
use std::io::Write;
use std::path::Path;
use wow_reference::{CoverageStatus, ReferencePartition, ReferenceView};

fn combined_status_request(
    primary_generation: &StoreGenerationId,
    primary_publication: &str,
    platform_generation: &str,
    platform_publication: &Value,
    candidate_id: &Value,
    candidate_digest: &Value,
) -> Value {
    json!({
        "schema": CONTEXT_COMBINED_STATUS_REQUEST_SCHEMA,
        "primary": {
            "project_store_id": "project-store:fixture-combined-primary",
            "store_generation_id": primary_generation,
            "publication_set_id": primary_publication
        },
        "platform": {
            "project_store_id": "project-store:fixture-combined-platform",
            "store_generation_id": platform_generation,
            "publication_set_id": platform_publication,
            "candidate_id": candidate_id,
            "candidate_digest": candidate_digest,
            "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID
        },
        "detail": "summary"
    })
}

fn write_new(path: &Path, bytes: &[u8], max: usize) -> TestResult {
    if bytes.len() > max {
        return Err("combined status export exceeds its byte bound".into());
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn combined_status_export_root(fixture: &FixtureRoot) -> TestResult<Option<PathBuf>> {
    let Some(raw) = std::env::var_os("WOW_CONTEXT_COMBINED_STATUS_EXPORT") else {
        return Ok(None);
    };
    let path = PathBuf::from(raw);
    if !path.is_absolute() || path.as_os_str().len() > 32768 {
        return Err("combined status export requires a finite absolute fresh directory".into());
    }
    let parent = path
        .parent()
        .ok_or("combined status export parent")?
        .canonicalize()?;
    let path = parent.join(path.file_name().ok_or("combined status export directory")?);
    if path.starts_with(fixture.0.canonicalize()?) {
        return Err("combined status export and stores must survive fixture removal".into());
    }
    std::fs::create_dir(&path)?;
    Ok(Some(path))
}

#[test]
fn exact_primary_and_platform_status_is_composed_over_retained_owners() -> TestResult {
    let stop = AtomicBool::new(false);
    let SelectedSkeletonFixture {
        root,
        config,
        path,
        project,
        request,
        ids,
        intent,
        selected_selection,
        selection_wire,
        selected_transport,
        ..
    } = selected_skeleton_fixture(&stop)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let reference = input.reference_view().clone();
    let profile = input
        .project_bundle()
        .configuration()
        .selected_profile()
        .clone();
    drop(input);
    let (primary_path, primary_query) =
        combined_primary_fixture(&root, &config, &profile, &reference, &stop)?;
    let export = combined_status_export_root(&root)?;
    let stores = export.as_ref().unwrap_or(&root.0);
    let primary_store = stores.join("registered-primary-store");
    let platform_store = stores.join("registered-platform-store");
    let primary = serde_json::to_value(publish_local_project(
        &primary_path,
        &primary_query,
        &primary_store,
        &LiveProjectPublishRequest::new("fixture:combined-status-primary", "absent", true, true)?,
        &stop,
    )?)?;
    let platform = serde_json::to_value(publish_selected_local_project(
        &path,
        &request,
        &platform_store,
        &selected_transport,
        &LiveProjectPublishRequest::new("fixture:combined-status-platform", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(primary["status"], "activated");
    assert_eq!(platform["status"], "activated");
    let primary_generation = StoreGenerationId::parse(
        primary["operation"]["generation_id"]
            .as_str()
            .ok_or("exact retained Primary generation")?,
    )?;
    let platform_generation = platform["operation"]["generation_id"]
        .as_str()
        .ok_or("exact retained platform generation")?
        .to_owned();

    // The candidate and Primary root come from the same retained owner helpers
    // already used by the combined map lifecycle, never from hand-authored rows.
    drop(project);
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    std::fs::remove_dir_all(root.0.join("primary-source"))?;
    let (native_request, native_candidate) = source_absent_skeleton(
        &platform_store,
        &selected_transport,
        &selected_selection,
        &platform_generation,
        &ids,
        &intent,
        &stop,
    )?;
    let native_request: Value = serde_json::from_slice(&native_request)?;
    let native_input = &native_request["input"];
    let (_, primary_publication, _, _, _) =
        combined_primary_observation(&primary_store, &primary_generation, &stop)?;
    let platform_publication = native_candidate["publication_set_id"].clone();

    let limits = json!({
        "neighbors": native_input["neighborhood"]["limits"],
        "explain": native_input["limits"]["root_explanation"],
        "evidence": native_input["limits"]["root_evidence"],
        "max_direct_edges": native_input["neighborhood"]["query"]["max_edges"],
        "max_input_bytes": 8_000_000,
        "max_metadata_bytes": 16 * 1024 * 1024,
        "max_context_work": 16_000_000
    });
    let primary_host = json!({
        "schema": CONTEXT_MAP_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-combined-primary",
        "project_store_root": primary_store,
        "project_id": "fixture-context-primary-map",
        "reference_profile": profile,
        "reference": reference,
        "limits": limits
    });
    let platform_host = json!({
        "schema": CONTEXT_PLATFORM_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-combined-platform",
        "project_store_root": platform_store,
        "selection": selection_wire,
        "reference_profile": profile,
        "reference": reference,
        "limits": limits
    });
    let host_wire = json!({
        "schema": CONTEXT_COMBINED_HOST_SCHEMA,
        "primary": primary_host,
        "platform": platform_host
    });
    let request_wire = combined_status_request(
        &primary_generation,
        &primary_publication,
        &platform_generation,
        &platform_publication,
        &native_candidate["candidate_id"],
        &native_candidate["candidate_digest"],
    );
    let host_bytes = wow_core::canonical_json_bytes(&host_wire)?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let request_bytes = wow_core::canonical_json_bytes(&request_wire)?;
    let status_request = ContextCombinedStatusRequest::from_json_slice(&request_bytes, &stop)?;
    let result = execute_context_combined_status(&host, &status_request, &stop)?;
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(
        envelope["schema"],
        "wow-service/context-combined-status-result/1"
    );
    assert_eq!(envelope["operation"], "context_combined_status");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(
        payload["primary"]["store_generation_id"],
        primary_generation.as_str()
    );
    assert_eq!(
        payload["primary"]["publication_set_id"],
        primary_publication
    );
    assert_eq!(
        payload["platform"]["store_generation_id"],
        platform_generation
    );
    assert_eq!(
        payload["platform"]["publication_set_id"],
        platform_publication
    );
    assert_eq!(
        payload["platform"]["candidate_id"],
        native_candidate["candidate_id"]
    );
    assert_eq!(
        payload["platform"]["candidate_digest"],
        native_candidate["candidate_digest"]
    );
    assert_eq!(
        payload["shared_reference"]["generation_id"],
        reference.generation_id()
    );
    for owner in ["primary", "platform"] {
        for field in [
            "project_snapshot_id",
            "project_generation",
            "graph_snapshot_id",
            "graph_generation",
            "graph_universe",
            "graph_registry_digest",
            "source_context_id",
        ] {
            assert!(
                payload[owner][field].is_string(),
                "retained {owner} metadata includes original {field}"
            );
        }
    }
    assert_eq!(payload["context_generation"], "NotEvaluated");
    assert_eq!(payload["evaluation"], "NotEvaluated");
    assert_eq!(payload["root_coverage"], "NotEvaluated");
    assert_eq!(
        payload["optional_platform_status"],
        "selected_metadata_only"
    );
    for forbidden in ["project_map", "artifact", "semantic_pack", "source_text"] {
        assert!(
            payload.get(forbidden).is_none(),
            "status excludes {forbidden}"
        );
    }

    // Each wrong exact publication/candidate is refused after acquisition and
    // both retained read scopes close. Invalid request bytes use serde_json,
    // avoiding the canonical serializer's null fallback for malformed values.
    for (owner, field, bad) in [
        (
            "primary",
            "publication_set_id",
            json!("publication-set:wrong"),
        ),
        (
            "platform",
            "publication_set_id",
            json!("publication-set:wrong"),
        ),
        ("platform", "candidate_id", json!("candidate:wrong")),
        (
            "platform",
            "candidate_digest",
            json!("sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ),
    ] {
        let mut wrong = request_wire.clone();
        wrong[owner][field] = bad;
        let bytes = serde_json::to_vec(&wrong)?;
        let decoded = ContextCombinedStatusRequest::from_json_slice(&bytes, &stop)?;
        let refused = execute_context_combined_status(&host, &decoded, &stop)?;
        assert!(refused.failure_code().is_some());
        let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
        assert_eq!(refused["resource_closure"], "closed");
        assert!(refused.get("payload").is_none());
    }
    let mut malformed = request_wire.clone();
    malformed["unexpected"] = json!(true);
    assert_eq!(
        ContextCombinedStatusRequest::from_json_slice(&serde_json::to_vec(&malformed)?, &stop)
            .err()
            .ok_or("unknown combined status request field was accepted")?
            .code(),
        ServiceErrorCode::InvalidRequest
    );

    // A self-consistent but wrong Reference generation is rejected against the
    // exact configurations retained by both genuine owners.
    let wrong_reference = ReferenceView::new(
        format!("{}-wrong", reference.generation_id()),
        reference.partitions().to_vec(),
        reference.conflicts().to_vec(),
    )?;
    let mut wrong_host_wire = host_wire.clone();
    wrong_host_wire["primary"]["reference"] = serde_json::to_value(&wrong_reference)?;
    wrong_host_wire["platform"]["reference"] = serde_json::to_value(&wrong_reference)?;
    let wrong_host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&wrong_host_wire)?,
        &stop,
    )?;
    let wrong_reference_result =
        execute_context_combined_status(&wrong_host, &status_request, &stop)?;
    assert!(wrong_reference_result.failure_code().is_some());
    let wrong_reference_envelope: Value =
        serde_json::from_slice(wrong_reference_result.canonical_bytes())?;
    assert_eq!(wrong_reference_envelope["resource_closure"], "closed");
    assert!(wrong_reference_envelope.get("payload").is_none());

    // Preserve the genuine shared generation and claimed digest while changing
    // platform Reference content. Combined host admission compares identity
    // claims; the service must still validate each configured view under the
    // shared finite work ledger and reject the forged second copy.
    let mut hostile_partitions = reference.partitions().to_vec();
    hostile_partitions.push(ReferencePartition::new(
        "fixture:forged-status-partition",
        CoverageStatus::Partial,
        Vec::new(),
    )?);
    let hostile_reference = ReferenceView::new(
        reference.generation_id(),
        hostile_partitions,
        reference.conflicts().to_vec(),
    )?;
    let mut forged_reference = serde_json::to_value(&hostile_reference)?;
    forged_reference["self_digest"] = json!(reference.self_digest());
    let mut forged_host_wire = host_wire.clone();
    forged_host_wire["platform"]["reference"] = forged_reference;
    let forged_host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&forged_host_wire)?,
        &stop,
    )?;
    let forged = execute_context_combined_status(&forged_host, &status_request, &stop)?;
    assert_eq!(
        forged.failure_code(),
        Some(ServiceErrorCode::IdentityMismatch)
    );
    let forged: Value = serde_json::from_slice(forged.canonical_bytes())?;
    assert_eq!(forged["resource_closure"], "closed");
    assert!(forged.get("payload").is_none());

    let cancelled =
        execute_context_combined_status(&host, &status_request, &AtomicBool::new(true))?;
    assert_eq!(cancelled.failure_code(), Some(ServiceErrorCode::Cancelled));
    let cancelled: Value = serde_json::from_slice(cancelled.canonical_bytes())?;
    assert_eq!(cancelled["resource_closure"], "not_acquired");

    let mut low_limit_wire = host_wire.clone();
    low_limit_wire["primary"]["limits"]["max_context_work"] = json!(1);
    low_limit_wire["platform"]["limits"]["max_context_work"] = json!(1);
    let low_limit_host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&low_limit_wire)?,
        &stop,
    )?;
    let limited = execute_context_combined_status(&low_limit_host, &status_request, &stop)?;
    assert_eq!(
        limited.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let limited: Value = serde_json::from_slice(limited.canonical_bytes())?;
    assert_eq!(limited["resource_closure"], "closed");
    assert!(limited.get("payload").is_none());

    // Both exact owners remain reopenable after the nested checked custody.
    let primary_store_handle = LiveProjectStore::open(&primary_store)?;
    let primary_read =
        primary_store_handle.read(&ReadSelector::Exact(primary_generation.clone()), &stop)?;
    primary_read.close()?;
    primary_store_handle.close()?;
    let platform_store_handle =
        LiveProjectStore::open_in_namespace(&platform_store, &selected_selection)?;
    let platform_read = platform_store_handle.read_in_namespace(
        &selected_selection,
        &ReadSelector::Exact(StoreGenerationId::parse(&platform_generation)?),
        &stop,
    )?;
    platform_read.close()?;
    platform_store_handle.close()?;

    if let Some(directory) = export {
        write_new(
            &directory.join("host.json"),
            &host_bytes,
            CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
        )?;
        write_new(
            &directory.join("status-request.json"),
            &request_bytes,
            CONTEXT_COMBINED_STATUS_REQUEST_MAX_BYTES,
        )?;
        write_new(
            &directory.join("expected-envelope.json"),
            result.canonical_bytes(),
            CONTEXT_MAP_RESULT_MAX_BYTES,
        )?;
    }
    Ok(())
}
