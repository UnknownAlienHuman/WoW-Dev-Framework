use super::*;
use std::io::{Read, Write};
use std::time::Instant;
use wow_store::project::StoreGenerationId;

fn write_new(path: &std::path::Path, bytes: &[u8]) -> TestResult {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)?;
    Ok(())
}

fn read_retained(path: &std::path::Path, limit: usize) -> TestResult<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("retained witness exceeds its owner bound".into());
    }
    Ok(bytes)
}

// Finish assertions on retained actual results after an oracle-only failure.
// Exact native owners are reopened independently; decoded output cannot mint custody.
fn qualify_retained(retained: &std::path::Path) -> TestResult {
    let stop = AtomicBool::new(false);
    let timer = Instant::now();
    let host_bytes = read_retained(
        &retained.join("host.json"),
        CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
    )?;
    let build_bytes = read_retained(
        &retained.join("build-request.json"),
        CONTEXT_COMBINED_REQUEST_MAX_BYTES,
    )?;
    let inspect_bytes = read_retained(
        &retained.join("expected-inspect-envelope.json"),
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )?;
    let result_bytes = read_retained(
        &retained.join("expected-build-envelope.json"),
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )?;
    let mut host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let build_wire: Value = serde_json::from_slice(&build_bytes)?;
    let primary_root = host_wire["primary"]["project_store_root"]
        .as_str()
        .ok_or("retained Primary root")?;
    assert_eq!(
        PathBuf::from(primary_root).canonicalize()?,
        retained.join("registered-primary-store").canonicalize()?
    );
    let exact = StoreGenerationId::parse(
        build_wire["primary"]["store_generation_id"]
            .as_str()
            .ok_or("retained generation")?,
    )?;
    let (node, publication, signature) =
        combined_reference_primary_observation(std::path::Path::new(primary_root), &exact, &stop)?;
    assert_eq!(build_wire["primary"]["publication_set_id"], publication);
    assert_eq!(build_wire["primary"]["root"]["id"], node.as_str());
    let inspect: Value = serde_json::from_slice(&inspect_bytes)?;
    let build: Value = serde_json::from_slice(&result_bytes)?;
    assert_reference_artifacts(&inspect, &build, &signature, &node)?;
    let owner_ms = timer.elapsed().as_millis();
    let ceiling = build["payload"]["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("retained Build work")?
        / 2;
    assert!(ceiling > 0);
    host_wire["primary"]["limits"]["max_context_work"] = json!(ceiling);
    host_wire["platform"]["limits"]["max_context_work"] = json!(ceiling);
    let host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&host_wire)?,
        &stop,
    )?;
    let request = ContextCombinedRequest::from_json_slice(&build_bytes, &stop)?;
    let timer = Instant::now();
    let refused = execute_context_combined(&host, &request, &stop)?;
    write_new(
        &retained.join("qualified-work-refusal.json"),
        refused.canonical_bytes(),
    )?;
    write_new(
        &retained.join("qualified-owner-signature.json"),
        &wow_core::canonical_json_bytes(&signature)?,
    )?;
    let proof = json!({"retained_owner_requalification_ms":owner_ms, "work_refusal_ms":timer.elapsed().as_millis(),
        "initial_cold_phase_times":"unmeasured", "graph_rebuilds":"unmeasured", "physical_edge_visits":"unmeasured",
        "work_ceiling":ceiling, "original_signature_and_full_inspect_pack_origins_equal":true});
    write_new(
        &retained.join("qualified-proof.json"),
        &wow_core::canonical_json_bytes(&proof)?,
    )?;
    assert_eq!(refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        refused.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
    assert_eq!(refused["resource_closure"], "closed");
    assert!(refused.get("payload").is_none());
    Ok(())
}

fn assert_reference_artifacts(
    inspect: &Value,
    build: &Value,
    signature: &Value,
    node: &wow_graph::GraphNodeId,
) -> TestResult {
    for envelope in [inspect, build] {
        assert_eq!(envelope["status"], "partial");
        assert_eq!(envelope["resource_closure"], "closed");
        assert!(envelope.get("failure_code").is_none());
    }
    let origins = &inspect["payload"]["combined_inspect"];
    let skeletons = &origins["core"]["primary_skeletons"];
    assert_eq!(skeletons["level"], "l1");
    assert!(skeletons.get("l0").is_none());
    let l1 = &skeletons["l1"]["core"];
    assert_eq!(
        l1["schema"],
        "wow-context/l1-skeleton/primary-reference-facts/1"
    );
    assert_eq!(
        l1["profile"],
        wow_context::PRIMARY_REFERENCE_FACTS_L1_PROFILE
    );
    assert_eq!(l1["root"]["node_id"], node.as_str());
    assert_eq!(l1["context"]["absence_authoritative"], false);
    let facets = l1["reference_records"]
        .as_array()
        .ok_or("Reference facets")?;
    let [facet] = facets.as_slice() else {
        return Err("one accepted API facet required".into());
    };
    assert_eq!(facet["node"]["node_id"], node.as_str());
    assert_eq!(facet["signature"], *signature);
    assert_eq!(facet["record"]["key"], signature["record_key"]);
    assert_eq!(facet["record_digest"], signature["record_digest"]);
    assert_eq!(facet["source"]["origin_id"], signature["source_id"]);
    assert_eq!(facet["absence_authoritative"], false);
    let map = &l1["context"]["origins"]["core"];
    assert!(
        map["source_custody"]
            .as_array()
            .ok_or("source custody")?
            .iter()
            .any(|row| row["handle"] == facet["source"])
    );
    let support = map["supports"]
        .as_array()
        .ok_or("original support")?
        .iter()
        .find(|row| row["explanation"]["record"]["node"] == facet["node"])
        .ok_or("accepted API support")?;
    assert_eq!(support["explanation"]["support_complete"], true);
    assert_eq!(support["evidence_resolution"]["closure_complete"], true);
    assert!(
        support["evidence_resolution"]["source_handles"]
            .as_array()
            .ok_or("support sources")?
            .contains(&facet["source"])
    );
    assert!(
        support["evidence_resolution"]["evidence_records"]
            .as_array()
            .ok_or("support evidence")?
            .contains(&facet["evidence"])
    );
    let pack = &build["payload"]["semantic_pack"];
    assert_eq!(pack["core"]["origins"], *origins);
    assert_eq!(pack["core"]["absence_authoritative"], false);
    let renderers = build["payload"]["rendered_artifacts"]
        .as_array()
        .ok_or("renderers")?;
    assert_eq!(renderers.len(), 2);
    for (renderer, profile) in renderers.iter().zip([
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE,
    ]) {
        assert_eq!(renderer["core"]["profile"]["profile_id"], profile);
        let content = renderer["core"]["content"]
            .as_str()
            .ok_or("renderer content")?;
        assert_eq!(renderer["core"]["byte_length"], content.len());
        if profile == wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE {
            assert_eq!(content.as_bytes(), wow_core::canonical_json_bytes(pack)?);
        }
    }
    Ok(())
}

#[test]
fn selected_reference_facts_source_absent_reopen_reaches_combined_inspect_and_build() -> TestResult
{
    if let Some(retained) = std::env::var_os("WOW_CONTEXT_REFERENCE_FIXTURE_REUSE") {
        return qualify_retained(&PathBuf::from(retained));
    }
    let stop = AtomicBool::new(false);
    let fixture_started = Instant::now();
    let SelectedSkeletonFixture {
        root,
        config,
        path,
        project,
        request: platform_publish_request,
        ids,
        intent,
        ordinary_selection: _,
        selected_selection,
        selection_wire,
        selected_transport,
    } = selected_reference_skeleton_fixture(&stop)?;
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let reference = input.reference_view().clone();
    let profile = input
        .project_bundle()
        .configuration()
        .selected_profile()
        .clone();
    drop(input);
    let (primary_path, primary_request) =
        combined_reference_primary_fixture(&root, &config, &profile, &reference, &stop)?;
    let export = combined_context_export_root(&root)?;
    let stores = export.as_ref().unwrap_or(&root.0);
    let primary_store = stores.join("registered-primary-store");
    let platform_store = stores.join("registered-platform-store");
    let fixture_ms = fixture_started.elapsed().as_millis();

    let publication_started = Instant::now();
    let primary_publication = serde_json::to_value(publish_local_project(
        &primary_path,
        &primary_request,
        &primary_store,
        &LiveProjectPublishRequest::new(
            "fixture:combined-reference-primary",
            "absent",
            true,
            true,
        )?,
        &stop,
    )?)?;
    let platform_publication = serde_json::to_value(publish_selected_local_project(
        &path,
        &platform_publish_request,
        &platform_store,
        &selected_transport,
        &LiveProjectPublishRequest::new(
            "fixture:combined-reference-platform",
            "absent",
            true,
            true,
        )?,
        &stop,
    )?)?;
    let primary_generation = StoreGenerationId::parse(
        primary_publication["operation"]["generation_id"]
            .as_str()
            .ok_or("exact Primary publication generation")?,
    )?;
    let platform_generation = platform_publication["operation"]["generation_id"]
        .as_str()
        .ok_or("exact selected platform generation")?
        .to_owned();
    let publication_ms = publication_started.elapsed().as_millis();

    drop(project);
    let remove_started = Instant::now();
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    let generated_docs = root.0.join("Blizzard_APIDocumentationGenerated");
    if generated_docs.exists() {
        std::fs::remove_dir_all(&generated_docs)?;
    }
    std::fs::remove_dir_all(root.0.join("primary-source"))?;
    let source_absent_paths = [
        root.0.join("UI"),
        root.0.join("version.txt"),
        root.0.join("Blizzard_APIDocumentationGenerated"),
        root.0.join("primary-source"),
    ];
    let source_absent = source_absent_paths.iter().all(|source| !source.exists());
    if !source_absent {
        return Err("fixture source inputs remained after the owned removals".into());
    }

    let (native_request_bytes, original_native) = source_absent_skeleton(
        &platform_store,
        &selected_transport,
        &selected_selection,
        &platform_generation,
        &ids,
        &intent,
        &stop,
    )?;
    let native_request: Value = serde_json::from_slice(&native_request_bytes)?;
    let native_input = &native_request["input"];
    let (primary_node, primary_publication_set, original_signature) =
        combined_reference_primary_observation(&primary_store, &primary_generation, &stop)?;
    let source_removal_ms = remove_started.elapsed().as_millis();
    let limits = json!({
        "neighbors": native_input["neighborhood"]["limits"],
        "explain": native_input["limits"]["root_explanation"],
        "evidence": native_input["limits"]["root_evidence"],
        "max_direct_edges": native_input["neighborhood"]["query"]["max_edges"],
        "max_input_bytes": 8_000_000,
        "max_metadata_bytes": 16 * 1024 * 1024,
        "max_context_work": 100_000_000
    });
    let primary_host = json!({
        "schema": CONTEXT_MAP_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-combined-reference-primary",
        "project_store_root": primary_store,
        "project_id": "fixture-context-primary-map",
        "reference_profile": profile,
        "reference": reference,
        "limits": limits
    });
    let platform_host = json!({
        "schema": CONTEXT_PLATFORM_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-combined-reference-platform",
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
    let platform_request = json!({
        "schema": CONTEXT_PLATFORM_REQUEST_SCHEMA,
        "operation": "map",
        "project_store_id": "project-store:fixture-combined-reference-platform",
        "store_generation_id": platform_generation,
        "publication_set_id": original_native["publication_set_id"],
        "candidate_id": original_native["candidate_id"],
        "candidate_digest": original_native["candidate_digest"],
        "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
        "input": native_input
    });
    let inspect_wire = json!({
        "schema": CONTEXT_COMBINED_REQUEST_SCHEMA,
        "operation": "inspect",
        "profile_id": COMBINED_NATIVE_MAP_PROFILE_ID,
        "primary": {
            "project_store_id": "project-store:fixture-combined-reference-primary",
            "store_generation_id": primary_generation,
            "publication_set_id": primary_publication_set,
            "map_profile_id": PRIMARY_NAVIGATION_MAP_PROFILE_ID,
            "root": {"kind": "graph_entity", "id": primary_node.as_str()}
        },
        "platform": platform_request,
        "inspection": {
            "level": "l1",
            "l1_profile_id": wow_context::PRIMARY_REFERENCE_FACTS_L1_PROFILE
        }
    });
    let build_wire = json!({
        "schema": CONTEXT_COMBINED_REQUEST_SCHEMA,
        "operation": "build",
        "profile_id": COMBINED_NATIVE_MAP_PROFILE_ID,
        "primary": inspect_wire["primary"],
        "platform": inspect_wire["platform"],
        "build": {
            "level": "l1",
            "context_profile_id": wow_context::COMBINED_CONTEXT_PACK_PROFILE,
            "renderer_profile_ids": [
                wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
                wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
            ],
            "l1_profile_id": wow_context::PRIMARY_REFERENCE_FACTS_L1_PROFILE
        }
    });
    let host_bytes = wow_core::canonical_json_bytes(&host_wire)?;
    let inspect_request_bytes = wow_core::canonical_json_bytes(&inspect_wire)?;
    let build_request_bytes = wow_core::canonical_json_bytes(&build_wire)?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let inspect_request = ContextCombinedRequest::from_json_slice(&inspect_request_bytes, &stop)?;
    let build_request = ContextCombinedRequest::from_json_slice(&build_request_bytes, &stop)?;

    if let Some(export) = &export {
        write_new(&export.join("host.json"), &host_bytes)?;
        write_new(&export.join("inspect-request.json"), &inspect_request_bytes)?;
        write_new(&export.join("build-request.json"), &build_request_bytes)?;
    }

    let inspect_started = Instant::now();
    let inspect_result = execute_context_combined(&host, &inspect_request, &stop)?;
    let inspect_ms = inspect_started.elapsed().as_millis();
    let inspect_bytes = inspect_result.canonical_bytes();
    if let Some(export) = &export {
        write_new(
            &export.join("expected-inspect-envelope.json"),
            inspect_bytes,
        )?;
    }

    let build_started = Instant::now();
    let build_result = execute_context_combined(&host, &build_request, &stop)?;
    let build_ms = build_started.elapsed().as_millis();
    let build_bytes = build_result.canonical_bytes();
    if let Some(export) = &export {
        write_new(&export.join("expected-build-envelope.json"), build_bytes)?;
    }

    let mut low_budget_host_wire = host_wire.clone();
    let inspect_envelope: Value = serde_json::from_slice(inspect_bytes)?;
    let build_envelope: Value = serde_json::from_slice(build_bytes)?;
    let work_ceiling = build_envelope["payload"]["operation_budget_usage"]["work_reserved"]
        .as_u64()
        .ok_or("actual Build construction work")?
        / 2;
    low_budget_host_wire["primary"]["limits"]["max_context_work"] = json!(work_ceiling);
    low_budget_host_wire["platform"]["limits"]["max_context_work"] = json!(work_ceiling);
    let low_budget_host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&low_budget_host_wire)?,
        &stop,
    )?;
    let low_budget_request = ContextCombinedRequest::from_json_slice(&build_request_bytes, &stop)?;
    let refusal_started = Instant::now();
    let refusal = execute_context_combined(&low_budget_host, &low_budget_request, &stop)?;
    let refusal_ms = refusal_started.elapsed().as_millis();

    let timings = json!({
        "fixture_setup_ms": fixture_ms,
        "publication_ms": publication_ms,
        "source_removal_and_owner_observation_ms": source_removal_ms,
        "inspect_ms": inspect_ms,
        "build_ms": build_ms,
        "low_budget_refusal_ms": refusal_ms,
        "graph_rebuilds": "unmeasured",
        "physical_edge_visits": "unmeasured"
    });
    if let Some(export) = &export {
        write_new(
            &export.join("expected-work-refusal-envelope.json"),
            refusal.canonical_bytes(),
        )?;
        write_new(
            &export.join("phase-timings.json"),
            &wow_core::canonical_json_bytes(&timings)?,
        )?;
    }

    assert!(
        source_absent,
        "all owned source inputs removed before reopen"
    );
    assert_eq!(primary_publication["status"], "activated");
    assert_eq!(platform_publication["status"], "activated");
    let build_envelope: Value = serde_json::from_slice(build_bytes)?;
    for envelope in [&inspect_envelope, &build_envelope] {
        assert_eq!(envelope["status"], "partial");
        assert_eq!(envelope["resource_closure"], "closed");
        assert!(envelope.get("failure_code").is_none());
    }
    assert_eq!(inspect_result.status(), ContextMapStatus::Partial);
    assert_eq!(inspect_result.failure_code(), None);
    assert_eq!(build_result.status(), ContextMapStatus::Partial);
    assert_eq!(build_result.failure_code(), None);
    assert_eq!(inspect_envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(build_envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(inspect_envelope["operation"], "context_combined_inspect");
    assert_eq!(build_envelope["operation"], "context_combined_build");
    assert_eq!(inspect_envelope["request"], inspect_wire);
    assert_eq!(build_envelope["request"], build_wire);

    assert_reference_artifacts(
        &inspect_envelope,
        &build_envelope,
        &original_signature,
        &primary_node,
    )?;

    assert_eq!(refusal.status(), ContextMapStatus::Failed);
    assert_eq!(
        refusal.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let refusal_envelope: Value = serde_json::from_slice(refusal.canonical_bytes())?;
    assert_eq!(refusal_envelope["resource_closure"], "closed");
    assert!(refusal_envelope.get("payload").is_none());
    assert!(refusal_envelope.get("failure_code").is_some());
    assert_eq!(serde_json::to_value(&low_budget_request)?, build_wire);
    Ok(())
}
