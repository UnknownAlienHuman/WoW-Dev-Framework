//! Shared native fixture lifecycle: selected publication/update/removal, exact
//! source-absent reopen and candidate-required Skeleton. No decoded authority.
use super::*;
use crate::{
    ServiceErrorCode,
    context::{
        COMBINED_NATIVE_MAP_PROFILE_ID, CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
        CONTEXT_COMBINED_HOST_SCHEMA, CONTEXT_COMBINED_REQUEST_MAX_BYTES,
        CONTEXT_COMBINED_REQUEST_SCHEMA, CONTEXT_COMBINED_RESULT_SCHEMA, CONTEXT_MAP_HOST_SCHEMA,
        CONTEXT_MAP_RESULT_MAX_BYTES, CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES,
        CONTEXT_PLATFORM_HOST_SCHEMA, CONTEXT_PLATFORM_REQUEST_MAX_BYTES,
        CONTEXT_PLATFORM_REQUEST_SCHEMA, CONTEXT_PLATFORM_RESULT_SCHEMA, ContextCombinedHostInput,
        ContextCombinedRequest, ContextMapHostInput, ContextMapPublication, ContextMapRequest,
        ContextMapRoot, ContextMapStatus, ContextPlatformHostInput, ContextPlatformRequest,
        PLATFORM_NATIVE_ONE_HOP_PROFILE_ID, PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        execute_context_combined, execute_context_map, execute_context_platform,
    },
    graph::{
        FunctionDirectNeighborhoodRecipe, FunctionFingerprintSelection, GRAPH_RESULT_MAX_BYTES,
        GraphBuildRequest, GraphReadStatus, XmlFingerprintSelection, XmlFingerprintTarget,
        XmlSchemaTypeShapeRecipe, execute_graph_build,
    },
    live_project::{
        LIVE_SKELETON_REQUEST_SCHEMA, LIVE_SKELETON_RESULT_SCHEMA, LiveProjectLibraryMode,
        LiveProjectPublishRequest, LiveProjectStore, LiveProjectUpdateRequest,
        PLATFORM_PROJECT_SELECTION_SCHEMA, PlatformProjectSelection, PlatformStoreSelection,
        execute_live_skeleton_read, publish_input_in_namespace, publish_local_project,
        publish_selected_local_project, read_selected_live_project, update_selected_local_project,
    },
};
use wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe;
use wow_graph::{
    GraphDirection, GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborQuery,
    GraphNeighborReadLimits, GraphNeighborReadQuery, GraphPathConfidence, GraphQueryState,
};
use wow_project::{
    ProjectView,
    load::schema::classification::RegionTypeWitness,
    skeleton_input::{OpenSkeletonInputRequest, SkeletonInputLimits},
};
use wow_store::project::{ReadSelector, StoreGenerationId};

struct SelectedSkeletonFixture {
    root: FixtureRoot,
    config: Value,
    path: PathBuf,
    project: ProjectView,
    request: GraphBuildRequest,
    ids: Vec<Box<str>>,
    intent: RegionTypeWitness,
    ordinary_selection: PlatformStoreSelection,
    selected_selection: PlatformStoreSelection,
    selection_wire: Value,
    selected_transport: PlatformProjectSelection,
}

// Identical original candidate setup, shared by standalone and combined witnesses.
fn selected_skeleton_fixture(stop: &AtomicBool) -> TestResult<SelectedSkeletonFixture> {
    // Reuse the existing reviewed classification fixture and its positive closed
    // TextureType shape; no second fixture manifest/schema authority is invented.
    let xsd = skeleton_xsd();
    let (root, mut config) = classification_fixture(&xsd)?;
    config["physical_function_declarations"] =
        serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
    write_json(&root, "project.json", &config)?;
    let path = root.0.join("project.json");
    let (project, request, ids, _, intent) = selected_skeleton_query(&path, &config, stop)?;

    let ordinary_selection = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        project
            .configuration()
            .platform_packages()
            .ok_or("admitted packages")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?
    .with_xml_classification_catalog()
    .with_physical_function_declarations_catalog();
    let selected_selection = ordinary_selection.clone().with_composed_candidate_catalog();
    let selection_wire = json!({
        "schema": PLATFORM_PROJECT_SELECTION_SCHEMA,
        "owner_project_id": project.configuration().project_id(),
        "source_profile_id": project.configuration().platform_packages().ok_or("source profile")?
            .source().profile().profile_id(),
        "catalogs": {
            "xml_schema_classification": true,
            "physical_function_declarations": "declared_syntax_v1",
            "physical_function_signatures": "absent",
            "composed_candidate": true
        }
    });
    let selected_transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, stop)?;
    Ok(SelectedSkeletonFixture {
        root,
        config,
        path,
        project,
        request,
        ids,
        intent,
        ordinary_selection,
        selected_selection,
        selection_wire,
        selected_transport,
    })
}

#[test]
fn selected_candidate_source_absent_reopen_reaches_skeleton_and_absence_refuses() -> TestResult {
    let stop = AtomicBool::new(false);
    let SelectedSkeletonFixture {
        root,
        config: _,
        path,
        project,
        request,
        ids,
        intent,
        ordinary_selection,
        selected_selection,
        mut selection_wire,
        selected_transport,
    } = selected_skeleton_fixture(&stop)?;
    let context_export = platform_context_export_root(&root)?;
    let selected_root = context_export.as_ref().map_or_else(
        || root.0.join("selected-candidate-store"),
        |export| export.join("registered-platform-store"),
    );
    let publish =
        LiveProjectPublishRequest::new("fixture:skeleton-selected", "absent", true, true)?;
    let published = publish_selected_local_project(
        &path,
        &request,
        &selected_root,
        &selected_transport,
        &publish,
        &stop,
    )
    .map_err(|error| format!("selected candidate publication: {error:?}"))?;
    let published = serde_json::to_value(published)?;
    assert_eq!(published["status"], "activated");
    let exact = published["operation"]["generation_id"]
        .as_str()
        .ok_or("selected store generation")?
        .to_owned();

    // Genuine ordinary epoch supplies the absence premise, not a manufactured
    // candidate receipt or a raw ProjectView/graph constructor.
    let ordinary_root = root.0.join("ordinary-component-store");
    let ordinary_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let context_reference = ordinary_input.reference_view().clone();
    let context_profile = project.configuration().selected_profile().clone();
    let ordinary_request =
        GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
            .with_input_profile(&ordinary_input);
    let ordinary_publish =
        LiveProjectPublishRequest::new("fixture:skeleton-ordinary", "absent", true, true)?;
    publish_input_in_namespace(
        ordinary_input,
        &ordinary_request,
        &ordinary_root,
        &ordinary_selection,
        &ordinary_publish,
        &stop,
    )
    .map_err(|error| format!("ordinary comparison publication: {error:?}"))?;
    drop(project);
    // These are only the existing temporary fixture files when ROOT runs this
    // test. No production source operation is performed by this private patch.
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    assert!(!root.0.join("UI/main.lua").exists());
    assert!(!root.0.join("UI/classification.xsd").exists());

    let (request, native_skeleton) = source_absent_skeleton(
        &selected_root,
        &selected_transport,
        &selected_selection,
        &exact,
        &ids,
        &intent,
        &stop,
    )?;

    // Existing genuine #116 candidate, exact native query and source-absent page.
    // Transport documents are inert; context reacquires the selected owner proof.
    let native_request: Value = serde_json::from_slice(&request)?;
    let page = &native_skeleton["page"];
    let body = &page["body"];
    let input = &native_request["input"];
    let host_wire = json!({
        "schema": CONTEXT_PLATFORM_HOST_SCHEMA,
        "project_store_id": "project-store:fixture-context-platform",
        "project_store_root": selected_root,
        "selection": selection_wire,
        "reference_profile": context_profile,
        "reference": context_reference,
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
    assert!(host_wire["selection"].is_object());
    let host = ContextPlatformHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&host_wire)?,
        &stop,
    )?;
    let current_store = LiveProjectStore::open_in_namespace(&selected_root, &selected_selection)?;
    let current_before = current_store.current()?.ok_or("platform context Current")?;
    current_store.close()?;
    let mut context_requests = Vec::new();
    let mut context_results = Vec::new();
    let mut context_envelopes = Vec::new();
    for (operation, name) in [
        ("map", "context_platform_map"),
        ("inspect", "context_platform_inspect"),
    ] {
        let wire = json!({
            "schema": CONTEXT_PLATFORM_REQUEST_SCHEMA,
            "operation": operation,
            "project_store_id": "project-store:fixture-context-platform",
            "store_generation_id": exact,
            "publication_set_id": native_skeleton["publication_set_id"],
            "candidate_id": native_skeleton["candidate_id"],
            "candidate_digest": native_skeleton["candidate_digest"],
            "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID,
            "input": input
        });
        let selected = ContextPlatformRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wire)?,
            &stop,
        )?;
        let observed = execute_context_platform(&host, &selected, &stop)?;
        assert_eq!(observed.status(), ContextMapStatus::Partial);
        assert_eq!(observed.failure_code(), None);
        assert_eq!(observed.resource_closure(), "closed");
        let envelope: Value = serde_json::from_slice(observed.canonical_bytes())?;
        assert_eq!(envelope["schema"], CONTEXT_PLATFORM_RESULT_SCHEMA);
        assert_eq!(envelope["operation"], name);
        assert_eq!(envelope["resource_closure"], "closed");
        assert!(envelope.get("failure_code").is_none());
        let payload = &envelope["payload"];
        assert_eq!(
            payload["selection"],
            serde_json::to_value(selected_transport.intent())?
        );
        assert_eq!(
            payload["publication_set_id"],
            native_skeleton["publication_set_id"]
        );
        assert_eq!(payload["store_generation_id"], exact);
        assert_eq!(payload["candidate_id"], native_skeleton["candidate_id"]);
        assert_eq!(
            payload["candidate_digest"],
            native_skeleton["candidate_digest"]
        );
        assert_eq!(
            payload["native_project_store_id"],
            selected_selection.namespace().id().as_str()
        );
        let artifact = &payload["artifact"];
        assert_eq!(artifact["family"], PLATFORM_NATIVE_ONE_HOP_PROFILE_ID);
        assert_eq!(artifact["artifact_kind"], operation);
        assert_eq!(
            payload["artifact_byte_length"],
            wow_core::canonical_json_bytes(artifact)?.len()
        );
        let map = if operation == "map" {
            artifact
        } else {
            &artifact["map"]
        };
        assert_eq!(map["family"], PLATFORM_NATIVE_ONE_HOP_PROFILE_ID);
        assert_eq!(map["artifact_kind"], "map");
        assert_eq!(
            map["core"]["native"], *page,
            "original admitted native page unchanged"
        );
        assert_eq!(map["core"]["absence_authoritative"], false);
        let binding = &map["core"]["binding"];
        assert_eq!(binding["native_request"], *input);
        assert_eq!(
            binding["target_profile"],
            serde_json::to_value(&context_profile)?
        );
        assert_eq!(
            binding["reference_generation"],
            serde_json::to_value(context_reference.generation_id())?
        );
        assert_eq!(
            binding["reference_view_digest"],
            context_reference.self_digest()
        );
        let summary = &payload["summary"];
        assert_eq!(
            summary,
            &serde_json::to_value(observed.summary().ok_or("typed native summary")?)?
        );
        assert_eq!(
            summary["resolved_request_id"],
            payload["resolved_request_id"]
        );
        assert_eq!(
            summary["publication_set_id"],
            native_skeleton["publication_set_id"]
        );
        assert_eq!(summary["store_generation_id"], exact);
        assert_eq!(summary["candidate_id"], native_skeleton["candidate_id"]);
        assert_eq!(summary["map_id"], map["map_id"]);
        let expected_artifact_id = if operation == "map" {
            &map["map_id"]
        } else {
            &artifact["inspect_id"]
        };
        assert_eq!(&summary["artifact_id"], expected_artifact_id);
        assert_eq!(summary["root_id"], body["neighborhood"]["root"]["node_id"]);
        assert_eq!(summary["counts"], body["counts"]);
        assert_eq!(summary["not_evaluated"], body["not_evaluated"]);
        for (field, owner, rows) in [
            ("root_conflicts", "explanation", "conflicts"),
            (
                "root_unresolved_evidence",
                "evidence_resolution",
                "unresolved",
            ),
            ("root_support_truncations", "explanation", "truncations"),
            (
                "root_evidence_truncations",
                "evidence_resolution",
                "truncations",
            ),
        ] {
            assert_eq!(
                summary[field],
                body["root_support"][owner][rows]
                    .as_array()
                    .ok_or("original native support records")?
                    .len()
            );
        }
        assert_eq!(summary["absence_authoritative"], false);
        assert!(
            !summary["not_evaluated"]
                .as_array()
                .ok_or("native unavailable facets")?
                .is_empty()
        );
        assert_eq!(body["state"], "partial");
        assert_eq!(
            body["root_support"]["explanation"]["support_complete"],
            true
        );
        assert_eq!(
            body["root_support"]["evidence_resolution"]["closure_complete"],
            true
        );
        let text = std::str::from_utf8(observed.canonical_bytes())?;
        assert!(!text.contains("project_store_root"));
        assert!(!text.contains("selection_json"));
        for path in [&root.0, &selected_root] {
            assert!(
                !text.contains(&serde_json::to_string(path)?),
                "host/source paths are excluded"
            );
        }
        assert!(!observed.canonical_bytes().ends_with(b"\n"));
        context_requests.push(selected);
        context_results.push(observed);
        context_envelopes.push(envelope);
    }
    // The owner map is identical across operations; service operation IDs differ.
    assert_eq!(
        context_envelopes[0]["payload"]["artifact"],
        context_envelopes[1]["payload"]["artifact"]["map"]
    );
    assert_eq!(
        context_results[0].summary().ok_or("map summary")?.map_id,
        context_results[1]
            .summary()
            .ok_or("inspect summary")?
            .map_id
    );

    // Exactly the original native component floor: every edge is scanned,
    // including unselected edges. Prior context admission reservations must still
    // be held, so the full operation cannot fit this component-only allowance.
    let native_floor = body["neighborhood"]["scanned_edges"]
        .as_u64()
        .ok_or("native scanned edges")?
        .checked_add(
            input["limits"]["root_explanation"]["max_scanned_assertions"]
                .as_u64()
                .ok_or("native explanation ceiling")?,
        )
        .and_then(|n| n.checked_add(input["limits"]["root_evidence"]["max_work"].as_u64()?))
        .and_then(|n| n.checked_add(1))
        .ok_or("native reservation floor overflow")?;
    assert!(native_floor < 8_000_000);
    let mut bounded_wire = host_wire.clone();
    bounded_wire["limits"]["max_context_work"] = json!(native_floor);
    let bounded = ContextPlatformHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&bounded_wire)?,
        &stop,
    )?;
    let refused = execute_context_platform(&bounded, &context_requests[1], &stop)?;
    assert_eq!(refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        refused.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    assert_eq!(refused.resource_closure(), "closed");
    assert!(refused.summary().is_none());
    let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
    assert!(refused.get("payload").is_none());
    assert_eq!(refused["resource_closure"], "closed");

    // One valid-shape inert mismatch, not manufactured candidate authority.
    let mut mismatch_wire = serde_json::to_value(&context_requests[1])?;
    let mut wrong_digest = mismatch_wire["candidate_digest"]
        .as_str()
        .ok_or("original candidate digest")?
        .to_owned();
    let last = wrong_digest.pop().ok_or("nonempty candidate digest")?;
    wrong_digest.push(if last == '0' { '1' } else { '0' });
    mismatch_wire["candidate_digest"] = json!(wrong_digest);
    let mismatch = ContextPlatformRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&mismatch_wire)?,
        &stop,
    )?;
    let refused = execute_context_platform(&host, &mismatch, &stop)?;
    assert_eq!(refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        refused.failure_code(),
        Some(ServiceErrorCode::IdentityMismatch)
    );
    assert_eq!(refused.resource_closure(), "closed");
    assert!(refused.summary().is_none());
    let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
    assert!(refused.get("payload").is_none());
    let current_store = LiveProjectStore::open_in_namespace(&selected_root, &selected_selection)?;
    assert_eq!(current_store.current()?, Some(current_before));
    current_store.close()?;

    if let Some(destination) = &context_export {
        use std::io::Write;
        let source_absent_paths = [root.0.join("UI"), root.0.join("version.txt")];
        assert!(source_absent_paths.iter().all(|path| !path.exists()));
        let cases = json!({
            "authority": "synthetic_fixture_only_not_live_ketho_or_runtime",
            "source_absent_paths": source_absent_paths,
            "root_id": body["neighborhood"]["root"]["node_id"],
            "candidate_id": native_skeleton["candidate_id"],
            "candidate_digest": native_skeleton["candidate_digest"],
            "store_generation_id": exact,
            "cases": [
                {"argv": ["context", "map", "--platform", "native", "--native-host-config",
                    destination.join("host.json"), "--platform-request-input", destination.join("map-request.json"),
                    "--output", "envelope-json"], "expected_envelope": "expected-map-envelope.json",
                    "expected_exit": 2, "stdout": "expected service bytes followed by one LF", "resource_closure": "closed"},
                {"argv": ["context", "inspect", "--platform", "native", "--native-host-config",
                    destination.join("host.json"), "--platform-request-input", destination.join("inspect-request.json"),
                    "--output", "envelope-json"], "expected_envelope": "expected-inspect-envelope.json",
                    "expected_exit": 2, "stdout": "expected service bytes followed by one LF", "resource_closure": "closed"}
            ]
        });
        for (name, bytes, limit) in [
            (
                "host.json",
                wow_core::canonical_json_bytes(&host_wire)?,
                CONTEXT_PLATFORM_HOST_INPUT_MAX_BYTES,
            ),
            (
                "map-request.json",
                wow_core::canonical_json_bytes(&context_requests[0])?,
                CONTEXT_PLATFORM_REQUEST_MAX_BYTES,
            ),
            (
                "inspect-request.json",
                wow_core::canonical_json_bytes(&context_requests[1])?,
                CONTEXT_PLATFORM_REQUEST_MAX_BYTES,
            ),
            (
                "cli-cases.json",
                wow_core::canonical_json_bytes(&cases)?,
                64 * 1024,
            ),
            (
                "expected-map-envelope.json",
                context_results[0].canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
            (
                "expected-inspect-envelope.json",
                context_results[1].canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
        ] {
            if bytes.len() > limit {
                return Err("platform fixture export exceeded finite byte bound".into());
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(name))?;
            file.write_all(&bytes)?;
        }
    }

    let ordinary = LiveProjectStore::open_in_namespace(&ordinary_root, &ordinary_selection)?;
    let absent = ordinary.read_in_namespace(&ordinary_selection, &ReadSelector::Current, &stop)?;
    assert!(absent.candidate().is_none());
    drop(absent);
    drop(ordinary);
    selection_wire["catalogs"]["composed_candidate"] = json!(false);
    let absent_transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, &stop)?;
    // Intent without candidate selection is rejected before any Skeleton read.
    assert_eq!(
        execute_live_skeleton_read(
            &ordinary_root,
            &absent_transport,
            "current",
            &request,
            &stop,
        )
        .err()
        .ok_or("absent-candidate request succeeded")?
        .code(),
        ServiceErrorCode::InvalidRequest
    );
    // Selecting candidate intent cannot reinterpret an ordinary epoch either.
    assert_eq!(
        execute_live_skeleton_read(
            &ordinary_root,
            &selected_transport,
            "current",
            &request,
            &stop,
        )
        .err()
        .ok_or("ordinary epoch bypassed candidate prerequisite")?
        .code(),
        ServiceErrorCode::IdentityMismatch
    );
    Ok(())
}

#[test]
fn selected_candidate_and_primary_source_absent_reopen_reaches_combined_map() -> TestResult {
    let stop = AtomicBool::new(false);
    let SelectedSkeletonFixture {
        root,
        config,
        path,
        project,
        request,
        ids,
        intent,
        ordinary_selection: _,
        selected_selection,
        selection_wire,
        selected_transport,
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
    let export = combined_context_export_root(&root)?;
    let stores = export.as_ref().unwrap_or(&root.0);
    let primary_store = stores.join("registered-primary-store");
    let platform_store = stores.join("registered-platform-store");
    let primary = serde_json::to_value(publish_local_project(
        &primary_path,
        &primary_query,
        &primary_store,
        &LiveProjectPublishRequest::new("fixture:combined-primary", "absent", true, true)?,
        &stop,
    )?)?;
    let platform = serde_json::to_value(publish_selected_local_project(
        &path,
        &request,
        &platform_store,
        &selected_transport,
        &LiveProjectPublishRequest::new("fixture:combined-platform", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(primary["status"], "activated");
    assert_eq!(platform["status"], "activated");
    let primary_exact = StoreGenerationId::parse(
        primary["operation"]["generation_id"]
            .as_str()
            .ok_or("Primary exact generation")?,
    )?;
    let platform_exact = platform["operation"]["generation_id"]
        .as_str()
        .ok_or("platform exact generation")?
        .to_owned();
    drop(project);
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    std::fs::remove_dir_all(root.0.join("primary-source"))?;
    let source_absent_paths = [
        root.0.join("UI"),
        root.0.join("version.txt"),
        root.0.join("primary-source"),
    ];
    assert!(source_absent_paths.iter().all(|path| !path.exists()));
    let (native_request, original_native) = source_absent_skeleton(
        &platform_store,
        &selected_transport,
        &selected_selection,
        &platform_exact,
        &ids,
        &intent,
        &stop,
    )?;
    let native_request: Value = serde_json::from_slice(&native_request)?;
    let native_input = &native_request["input"];
    let (primary_root, primary_publication, primary_node, primary_proposal, primary_accepted) =
        combined_primary_observation(&primary_store, &primary_exact, &stop)?;
    let limits = json!({
        "neighbors": native_input["neighborhood"]["limits"],
        "explain": native_input["limits"]["root_explanation"],
        "evidence": native_input["limits"]["root_evidence"],
        "max_direct_edges": native_input["neighborhood"]["query"]["max_edges"],
        "max_input_bytes": 8_000_000, "max_metadata_bytes": 16 * 1024 * 1024,
        "max_context_work": 16_000_000
    });
    let primary_host = json!({
        "schema": CONTEXT_MAP_HOST_SCHEMA, "project_store_id": "project-store:fixture-combined-primary",
        "project_store_root": primary_store, "project_id": "fixture-context-primary-map",
        "reference_profile": profile, "reference": reference, "limits": limits
    });
    let platform_host = json!({
        "schema": CONTEXT_PLATFORM_HOST_SCHEMA, "project_store_id": "project-store:fixture-combined-platform",
        "project_store_root": platform_store, "selection": selection_wire,
        "reference_profile": profile, "reference": reference, "limits": limits
    });
    let platform_request = json!({
        "schema": CONTEXT_PLATFORM_REQUEST_SCHEMA, "operation": "map",
        "project_store_id": "project-store:fixture-combined-platform", "store_generation_id": platform_exact,
        "publication_set_id": original_native["publication_set_id"],
        "candidate_id": original_native["candidate_id"], "candidate_digest": original_native["candidate_digest"],
        "profile_id": PLATFORM_NATIVE_ONE_HOP_PROFILE_ID, "input": native_input
    });
    // Only original positive leaf reads, no repetition of their control campaigns.
    let primary_leaf = execute_context_map(
        &ContextMapHostInput::from_json_slice(
            &wow_core::canonical_json_bytes(&primary_host)?,
            &stop,
        )?,
        &ContextMapRequest::primary(
            "project-store:fixture-combined-primary",
            ContextMapPublication::StoreGeneration(primary_exact.clone()),
            PRIMARY_NAVIGATION_MAP_PROFILE_ID,
            Some(ContextMapRoot::GraphEntity(primary_root.clone())),
            None,
            None,
        )?,
        &stop,
    )?;
    let mut platform_inspect_wire = platform_request.clone();
    platform_inspect_wire["operation"] = json!("inspect");
    let platform_leaf = execute_context_platform(
        &ContextPlatformHostInput::from_json_slice(
            &wow_core::canonical_json_bytes(&platform_host)?,
            &stop,
        )?,
        &ContextPlatformRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&platform_inspect_wire)?,
            &stop,
        )?,
        &stop,
    )?;
    assert_eq!(primary_leaf.status(), ContextMapStatus::Partial);
    assert_eq!(platform_leaf.status(), ContextMapStatus::Partial);
    assert_eq!(primary_leaf.failure_code(), None);
    assert_eq!(platform_leaf.failure_code(), None);
    let primary_leaf: Value = serde_json::from_slice(primary_leaf.canonical_bytes())?;
    let platform_leaf: Value = serde_json::from_slice(platform_leaf.canonical_bytes())?;
    assert_eq!(primary_leaf["resource_closure"], "closed");
    assert_eq!(platform_leaf["resource_closure"], "closed");
    let original_primary_map = &primary_leaf["payload"]["project_map"];
    let original_platform_inspect = &platform_leaf["payload"]["artifact"];
    let original_platform_map = &original_platform_inspect["map"];
    let host_wire = json!({"schema": CONTEXT_COMBINED_HOST_SCHEMA, "primary": primary_host, "platform": platform_host});
    let request_wire = json!({
        "schema": CONTEXT_COMBINED_REQUEST_SCHEMA, "operation": "map", "profile_id": COMBINED_NATIVE_MAP_PROFILE_ID,
        "primary": {"project_store_id": "project-store:fixture-combined-primary",
            "store_generation_id": primary_exact, "publication_set_id": primary_publication,
            "map_profile_id": PRIMARY_NAVIGATION_MAP_PROFILE_ID, "root": ContextMapRoot::GraphEntity(primary_root.clone())},
        "platform": platform_request
    });
    let host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&host_wire)?,
        &stop,
    )?;
    let request = ContextCombinedRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&request_wire)?,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(&request)?, request_wire);
    assert!(serde_json::to_value(&request)?.get("inspection").is_none());
    let mut inspect_wire = request_wire.clone();
    inspect_wire["operation"] = json!("inspect");
    inspect_wire["inspection"] = json!({"level": "both"});
    let inspect_request_bytes = wow_core::canonical_json_bytes(&inspect_wire)?;
    let inspect_request = ContextCombinedRequest::from_json_slice(&inspect_request_bytes, &stop)?;
    assert_eq!(serde_json::to_value(&inspect_request)?, inspect_wire);

    // Decode-only controls never acquire either store or rerun a producer.
    let mut missing_inspection = inspect_wire.clone();
    missing_inspection
        .as_object_mut()
        .ok_or("combined inspect request object")?
        .remove("inspection");
    let mut null_inspection = inspect_wire.clone();
    null_inspection["inspection"] = Value::Null;
    let mut null_map_inspection = request_wire.clone();
    null_map_inspection["inspection"] = Value::Null;
    let mut map_with_inspection = inspect_wire.clone();
    map_with_inspection["operation"] = json!("map");
    let mut unknown_level = inspect_wire.clone();
    unknown_level["inspection"]["level"] = json!("unknown");
    let mut nested_inspect = inspect_wire.clone();
    nested_inspect["platform"]["operation"] = json!("inspect");
    for invalid in [
        missing_inspection,
        null_inspection,
        null_map_inspection,
        map_with_inspection,
        unknown_level,
        nested_inspect,
    ] {
        assert_eq!(
            ContextCombinedRequest::from_json_slice(&serde_json::to_vec(&invalid)?, &stop,)
                .err()
                .ok_or("malformed combined inspection accepted")?
                .code(),
            ServiceErrorCode::InvalidRequest
        );
    }
    assert_eq!(
        ContextCombinedRequest::from_json_slice(&inspect_request_bytes, &AtomicBool::new(true),)
            .err()
            .ok_or("pre-admission combined inspection cancellation ignored")?
            .code(),
        ServiceErrorCode::Cancelled
    );

    let observed = execute_context_combined(&host, &request, &stop)?;
    assert_eq!(observed.status(), ContextMapStatus::Partial);
    assert_eq!(observed.failure_code(), None);
    let envelope: Value = serde_json::from_slice(observed.canonical_bytes())?;
    assert_eq!(envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(envelope["operation"], "context_combined_map");
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(
        payload["primary"]["store_generation_id"],
        primary_exact.as_str()
    );
    assert_eq!(
        payload["primary"]["publication_set_id"],
        primary_publication
    );
    assert_eq!(
        payload["primary"]["configured_registration_id"],
        primary_host["project_store_id"]
    );
    assert!(payload["primary"].get("native_project_store_id").is_none());
    assert_eq!(payload["platform"]["store_generation_id"], platform_exact);
    assert_eq!(
        payload["platform"]["publication_set_id"],
        original_native["publication_set_id"]
    );
    assert_eq!(
        payload["platform"]["configured_registration_id"],
        platform_host["project_store_id"]
    );
    assert_eq!(
        payload["platform"]["native_project_store_id"],
        selected_selection.namespace().id().as_str()
    );
    assert_eq!(
        payload["platform_selection"],
        serde_json::to_value(selected_transport.intent())?
    );
    let map = &payload["combined_map"];
    assert_eq!(map["family"], COMBINED_NATIVE_MAP_PROFILE_ID);
    assert_eq!(map["artifact_kind"], "map");
    assert_eq!(
        payload["combined_map_byte_length"],
        wow_core::canonical_json_bytes(map)?.len()
    );
    let core = &map["core"];
    assert_eq!(
        core["primary"], *original_primary_map,
        "original Primary artifact and support"
    );
    assert_eq!(
        core["platform"], *original_platform_map,
        "original platform artifact and support"
    );
    assert_eq!(core["platform"]["core"]["native"], original_native["page"]);
    assert_ne!(
        core["primary"]["project_map_id"],
        core["platform"]["map_id"]
    );
    assert_ne!(
        core["primary"]["core"]["binding"]["graph_snapshot_id"],
        native_input["binding"]["graph_snapshot_id"]
    );
    assert_eq!(core["primary"]["core"]["navigation"]["root"], primary_node);
    let root_support = core["primary"]["core"]["supports"]
        .as_array()
        .ok_or("original Primary supports")?
        .iter()
        .find(|row| row["explanation"]["record"]["node"] == primary_node)
        .ok_or("original Primary root explanation")?;
    assert_eq!(root_support["explanation"]["support_complete"], true);
    assert_eq!(
        root_support["evidence_resolution"]["closure_complete"],
        true
    );
    assert!(
        root_support["explanation"]["supports"]
            .as_array()
            .ok_or("original accepted root supports")?
            .iter()
            .any(|row| row["proposal"] == primary_proposal && row["accepted"] == primary_accepted)
    );
    for key in [
        "target_profile",
        "reference_generation",
        "reference_schema",
        "reference_view_digest",
    ] {
        assert_eq!(
            core["primary"]["core"]["binding"][key],
            core["platform"]["core"]["binding"][key]
        );
    }
    assert_eq!(
        core["primary"]["core"]["binding"]["target_profile"],
        serde_json::to_value(&profile)?
    );
    assert_eq!(
        core["primary"]["core"]["binding"]["reference_generation"],
        reference.generation_id()
    );
    assert_eq!(
        core["primary"]["core"]["binding"]["reference_schema"],
        reference.schema()
    );
    assert_eq!(
        core["primary"]["core"]["binding"]["reference_view_digest"],
        reference.self_digest()
    );
    assert_eq!(core["state"], "partial");
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(core["cross_universe_relations"], "not_evaluated");
    assert!(
        !core["platform"]["core"]["native"]["body"]["not_evaluated"]
            .as_array()
            .ok_or("native omissions")?
            .is_empty()
    );
    let text = std::str::from_utf8(observed.canonical_bytes())?;
    assert!(
        !text.contains("project_store_root") && !text.contains("local function ContextFixture")
    );
    for source in [&root.0, &primary_store, &platform_store] {
        assert!(
            !text.contains(&serde_json::to_string(source)?),
            "host paths excluded"
        );
    }
    assert!(!observed.canonical_bytes().ends_with(b"\n"));

    // L0 on the SAME exact source-file root consumes the navigation map
    // unchanged. Both intentionally selects the existing native detail origin.
    let mut l0_wire = inspect_wire.clone();
    l0_wire["inspection"]["level"] = json!("l0");
    let l0_request =
        ContextCombinedRequest::from_json_slice(&wow_core::canonical_json_bytes(&l0_wire)?, &stop)?;
    let l0_observed = execute_context_combined(&host, &l0_request, &stop)?;
    assert_eq!(l0_observed.status(), ContextMapStatus::Partial);
    assert_eq!(l0_observed.failure_code(), None);
    let l0_envelope: Value = serde_json::from_slice(l0_observed.canonical_bytes())?;
    assert_eq!(l0_envelope["resource_closure"], "closed");
    assert_eq!(l0_envelope["operation"], "context_combined_inspect");
    let l0_inspect = &l0_envelope["payload"]["combined_inspect"];
    assert_eq!(
        l0_inspect["core"]["map"], *map,
        "unchanged combined map origin"
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&l0_inspect["core"]["map"])?,
        wow_core::canonical_json_bytes(map)?
    );
    assert_eq!(
        l0_inspect["core"]["platform_inspect"],
        *original_platform_inspect
    );
    assert_eq!(
        l0_envelope["payload"]["combined_inspect_byte_length"],
        wow_core::canonical_json_bytes(l0_inspect)?.len()
    );
    let l0_skeletons = &l0_inspect["core"]["primary_skeletons"];
    assert_eq!(l0_skeletons["level"], "l0");
    assert!(l0_skeletons.get("l1").is_none());
    assert_eq!(l0_skeletons["l0"]["core"]["scope"]["node"], primary_node);
    assert_eq!(
        l0_skeletons["l0"]["core"]["context"]["origins"],
        *original_primary_map
    );

    let inspect_observed = execute_context_combined(&host, &inspect_request, &stop)?;
    assert_eq!(inspect_observed.status(), ContextMapStatus::Partial);
    assert_eq!(inspect_observed.failure_code(), None);
    let inspect_envelope: Value = serde_json::from_slice(inspect_observed.canonical_bytes())?;
    assert_eq!(inspect_envelope["resource_closure"], "closed");
    assert_eq!(inspect_envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(inspect_envelope["operation"], "context_combined_inspect");
    assert_eq!(inspect_envelope["request"], inspect_wire);
    assert_ne!(inspect_envelope["request_id"], envelope["request_id"]);
    assert!(inspect_envelope.get("failure_code").is_none());
    let inspect_payload = &inspect_envelope["payload"];
    for key in ["primary", "platform", "platform_selection"] {
        assert_eq!(inspect_payload[key], payload[key]);
    }
    assert_ne!(
        inspect_payload["resolved_request_id"],
        payload["resolved_request_id"]
    );
    let inspect = &inspect_payload["combined_inspect"];
    assert_eq!(inspect["family"], COMBINED_NATIVE_MAP_PROFILE_ID);
    assert_eq!(inspect["artifact_kind"], "inspect");
    assert!(
        inspect["inspect_id"]
            .as_str()
            .ok_or("native combined inspect ID")?
            .starts_with("combined-context-inspect:sha256:")
    );
    assert_eq!(
        inspect_payload["combined_inspect_byte_length"],
        wow_core::canonical_json_bytes(inspect)?.len()
    );
    let inspect_core = &inspect["core"];
    let inspect_map = &inspect_core["map"];
    let inspect_primary_map = &inspect_map["core"]["primary"];
    assert_eq!(inspect_map["family"], COMBINED_NATIVE_MAP_PROFILE_ID);
    assert_eq!(inspect_map["artifact_kind"], "map");
    assert_ne!(inspect_map["combined_map_id"], map["combined_map_id"]);
    assert_eq!(
        inspect_primary_map["core"]["profile"]["profile"],
        "wow-context/primary-entity-detail-origins/1"
    );
    assert_eq!(
        inspect_primary_map["core"]["binding"],
        core["primary"]["core"]["binding"]
    );
    assert_eq!(
        inspect_primary_map["core"]["navigation"]["root"],
        primary_node
    );
    let inspect_root_support = inspect_primary_map["core"]["supports"]
        .as_array()
        .ok_or("native detail-origin supports")?
        .iter()
        .find(|row| row["explanation"]["record"]["node"] == primary_node)
        .ok_or("native detail-origin root support")?;
    assert_eq!(inspect_root_support, root_support);
    let skeletons = &inspect_core["primary_skeletons"];
    assert_eq!(
        skeletons["schema"],
        "wow-context/primary-skeleton-inspection/1"
    );
    assert_eq!(skeletons["level"], "both");
    assert_eq!(skeletons["l0"]["core"]["scope"]["node"], primary_node);
    assert_eq!(skeletons["l1"]["core"]["root"], primary_node);
    for level in ["l0", "l1"] {
        let context = &skeletons[level]["core"]["context"];
        assert_eq!(
            context["origin_map_id"],
            inspect_primary_map["project_map_id"]
        );
        assert_eq!(context["origins"], *inspect_primary_map);
        assert_eq!(context["state"], "partial");
        assert_eq!(context["absence_authoritative"], false);
    }
    assert_eq!(inspect_map["core"]["platform"], *original_platform_map);
    assert_eq!(inspect_core["platform_inspect"], *original_platform_inspect);
    assert_eq!(
        wow_core::canonical_json_bytes(&inspect_core["platform_inspect"])?,
        wow_core::canonical_json_bytes(original_platform_inspect)?
    );
    assert_eq!(
        inspect_core["platform_inspect"]["map"]["core"]["native"],
        original_native["page"]
    );
    assert_eq!(inspect_core["state"], "partial");
    assert_eq!(inspect_core["absence_authoritative"], false);
    assert_eq!(inspect_core["cross_universe_relations"], "not_evaluated");
    assert!(!inspect_observed.canonical_bytes().ends_with(b"\n"));
    let inspect_text = std::str::from_utf8(inspect_observed.canonical_bytes())?;
    assert!(
        !inspect_text.contains("project_store_root")
            && !inspect_text.contains("local function ContextFixture")
    );
    for source in [&root.0, &primary_store, &platform_store] {
        assert!(!inspect_text.contains(&serde_json::to_string(source)?));
    }

    // Original Primary reservations remain held; additional platform bind work
    // makes this exact Primary-plus-native-scan floor insufficient on one ledger.
    let primary_work = inspect_primary_map["core"]["budget"]["context_work_reserved"]
        .as_u64()
        .ok_or("original Primary reserved work")?;
    let native_floor = original_native["page"]["body"]["neighborhood"]["scanned_edges"]
        .as_u64()
        .ok_or("original native edge census")?
        .checked_add(
            native_input["limits"]["root_explanation"]["max_scanned_assertions"]
                .as_u64()
                .ok_or("native explanation ceiling")?,
        )
        .and_then(|n| n.checked_add(native_input["limits"]["root_evidence"]["max_work"].as_u64()?))
        .and_then(|n| n.checked_add(1))
        .ok_or("native reservation overflow")?;
    let selected_work = primary_work
        .checked_add(native_floor)
        .ok_or("combined work overflow")?;
    assert!(primary_work > 0 && native_floor > 0 && selected_work < 8_000_000);
    assert!(
        skeletons["work_reserved"]
            .as_u64()
            .ok_or("continuing Primary inspection work")?
            > selected_work
    );
    assert!(
        skeletons["bytes_reserved"]
            .as_u64()
            .ok_or("continuing Primary inspection metadata")?
            > inspect_primary_map["core"]["budget"]["retained_rows_bytes_reserved"]
                .as_u64()
                .ok_or("original Primary map metadata")?
    );
    let mut bounded_wire = host_wire.clone();
    bounded_wire["primary"]["limits"]["max_context_work"] = json!(selected_work);
    bounded_wire["platform"]["limits"]["max_context_work"] = json!(selected_work);
    let bounded = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&bounded_wire)?,
        &stop,
    )?;
    // Keep the existing navigation-map refusal independently of the new
    // detail/inspection selection; its original ledger regression stays covered.
    let original_map_work = original_primary_map["core"]["budget"]["context_work_reserved"]
        .as_u64()
        .and_then(|work| work.checked_add(native_floor))
        .ok_or("original combined map work overflow")?;
    let mut original_map_bound = host_wire.clone();
    original_map_bound["primary"]["limits"]["max_context_work"] = json!(original_map_work);
    original_map_bound["platform"]["limits"]["max_context_work"] = json!(original_map_work);
    let original_map_bound = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&original_map_bound)?,
        &stop,
    )?;
    let original_refused = execute_context_combined(&original_map_bound, &request, &stop)?;
    assert_eq!(original_refused.status(), ContextMapStatus::Failed);
    assert_eq!(
        original_refused.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let original_refused: Value = serde_json::from_slice(original_refused.canonical_bytes())?;
    assert_eq!(original_refused["resource_closure"], "closed");
    assert!(original_refused.get("payload").is_none());
    // The genuine inspection artifact alone fits this positive byte allowance
    // at the original limits. Earlier retained construction and final delivery
    // must also fit the same ledger; no specific refusal stage is assumed.
    let inspection_bytes = inspect_payload["combined_inspect_byte_length"]
        .as_u64()
        .ok_or("native combined inspection byte count")?;
    assert!(inspection_bytes > 0 && inspection_bytes < 8_000_000);
    let mut metadata_wire = host_wire.clone();
    metadata_wire["primary"]["limits"]["max_metadata_bytes"] = json!(inspection_bytes);
    metadata_wire["platform"]["limits"]["max_metadata_bytes"] = json!(inspection_bytes);
    let metadata_bounded = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&metadata_wire)?,
        &stop,
    )?;
    for selected_host in [&bounded, &metadata_bounded] {
        let refused = execute_context_combined(selected_host, &inspect_request, &stop)?;
        assert_eq!(refused.status(), ContextMapStatus::Failed);
        assert_eq!(
            refused.failure_code(),
            Some(ServiceErrorCode::BudgetExceeded)
        );
        let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
        assert_eq!(refused["operation"], "context_combined_inspect");
        assert_eq!(refused["resource_closure"], "closed");
        assert!(refused.get("payload").is_none());
    }
    // One new Build witness reuses the SAME published/source-absent owners.
    // Renderer selection is inert service intent, not semantic pack identity.
    let mut build_none_wire = request_wire.clone();
    build_none_wire["operation"] = json!("build");
    build_none_wire["build"] = json!({
        "level": "both", "context_profile_id": wow_context::COMBINED_CONTEXT_PACK_PROFILE,
        "renderer_profile_ids": []
    });
    let build_none = ContextCombinedRequest::from_json_slice(
        &wow_core::canonical_json_bytes(&build_none_wire)?,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(&build_none)?, build_none_wire);
    let mut build_wire = build_none_wire.clone();
    build_wire["build"]["renderer_profile_ids"] = json!([
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
    ]);
    let build_request_bytes = wow_core::canonical_json_bytes(&build_wire)?;
    let build_request = ContextCombinedRequest::from_json_slice(&build_request_bytes, &stop)?;
    assert_eq!(serde_json::to_value(&build_request)?, build_wire);
    let mut reversed_renderers = build_wire.clone();
    reversed_renderers["build"]["renderer_profile_ids"] = json!([
        wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE
    ]);
    assert_eq!(
        serde_json::to_value(ContextCombinedRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&reversed_renderers)?,
            &stop,
        )?)?,
        build_wire,
        "Both intent normalizes before request identity"
    );
    let mut missing_build = build_wire.clone();
    missing_build
        .as_object_mut()
        .ok_or("Build request object")?
        .remove("build");
    let mut null_build = build_wire.clone();
    null_build["build"] = Value::Null;
    let mut null_render = build_wire.clone();
    null_render["build"]["renderer_profile_ids"] = Value::Null;
    let mut primary_recipe = build_wire.clone();
    primary_recipe["build"]["context_profile_id"] = json!("wow-context/primary-structural-pack/1");
    let mut mixed_renderer = build_wire.clone();
    mixed_renderer["build"]["renderer_profile_ids"] = json!(["wow-context/primary-pack-json/1"]);
    let mut duplicate_renderer = build_wire.clone();
    duplicate_renderer["build"]["renderer_profile_ids"] = json!([
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE
    ]);
    let mut conflicting_inspection = build_wire.clone();
    conflicting_inspection["inspection"] = Value::Null;
    for invalid in [
        missing_build,
        null_build,
        null_render,
        primary_recipe,
        mixed_renderer,
        duplicate_renderer,
        conflicting_inspection,
    ] {
        assert_eq!(
            ContextCombinedRequest::from_json_slice(&serde_json::to_vec(&invalid)?, &stop)
                .err()
                .ok_or("invalid combined Build recipe accepted")?
                .code(),
            ServiceErrorCode::InvalidRequest
        );
    }
    assert_eq!(
        ContextCombinedRequest::from_json_slice(&build_request_bytes, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled combined Build selection accepted")?
            .code(),
        ServiceErrorCode::Cancelled
    );
    let none_observed = execute_context_combined(&host, &build_none, &stop)?;
    assert_eq!(none_observed.status(), ContextMapStatus::Partial);
    assert_eq!(none_observed.failure_code(), None);
    let none_envelope: Value = serde_json::from_slice(none_observed.canonical_bytes())?;
    assert_eq!(none_envelope["operation"], "context_combined_build");
    assert_eq!(none_envelope["resource_closure"], "closed");
    let none_payload = &none_envelope["payload"];
    assert_eq!(none_payload["rendered_artifacts"], json!([]));
    let build_observed = execute_context_combined(&host, &build_request, &stop)?;
    assert_eq!(build_observed.status(), ContextMapStatus::Partial);
    assert_eq!(build_observed.failure_code(), None);
    let build_envelope: Value = serde_json::from_slice(build_observed.canonical_bytes())?;
    assert_eq!(build_envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(build_envelope["operation"], "context_combined_build");
    assert_eq!(build_envelope["resource_closure"], "closed");
    assert_eq!(build_envelope["request"], build_wire);
    assert_ne!(build_envelope["request_id"], none_envelope["request_id"]);
    let build_payload = &build_envelope["payload"];
    for key in ["primary", "platform", "platform_selection"] {
        assert_eq!(build_payload[key], inspect_payload[key]);
        assert_eq!(none_payload[key], inspect_payload[key]);
    }
    assert_eq!(
        build_payload["validation"],
        "exact_native_binding_and_canonical_bytes"
    );
    assert_eq!(none_payload["validation"], build_payload["validation"]);
    assert_combined_pack_and_renderers(
        build_payload,
        none_payload,
        inspect,
        &original_native["page"],
    )?;

    // Derive NEW-stage ceilings from the actual None Build usage. Prove the
    // predecessor still succeeds under each ceiling; preserve every old refusal.
    let none_usage = &none_payload["operation_budget_usage"];
    let both_usage = &build_payload["operation_budget_usage"];
    assert_eq!(none_usage["reservation_basis"], "before_service_delivery");
    assert_eq!(both_usage["reservation_basis"], "before_service_delivery");
    let none_work = none_usage["work_reserved"]
        .as_u64()
        .ok_or("None Build actual work")?;
    let none_bytes = none_usage["bytes_reserved"]
        .as_u64()
        .ok_or("None Build actual bytes")?;
    assert!(none_work > 0 && none_work < 8_000_000);
    assert!(none_bytes > 0 && none_bytes < 8_000_000);
    assert!(
        both_usage["work_reserved"]
            .as_u64()
            .ok_or("Both Build actual work")?
            > none_work
    );
    assert!(
        both_usage["bytes_reserved"]
            .as_u64()
            .ok_or("Both Build actual bytes")?
            > none_bytes
    );
    for (field, ceiling, predecessor, selected) in [
        ("max_context_work", none_work, &build_none, &build_request),
        (
            "max_metadata_bytes",
            none_bytes,
            &inspect_request,
            &build_none,
        ),
    ] {
        let mut stage_host = host_wire.clone();
        stage_host["primary"]["limits"][field] = json!(ceiling);
        stage_host["platform"]["limits"][field] = json!(ceiling);
        let stage_host = ContextCombinedHostInput::from_json_slice(
            &wow_core::canonical_json_bytes(&stage_host)?,
            &stop,
        )?;
        let before = execute_context_combined(&stage_host, predecessor, &stop)?;
        assert_eq!(
            before.status(),
            ContextMapStatus::Partial,
            "predecessor must fit {field}"
        );
        assert_eq!(before.failure_code(), None);
        let before: Value = serde_json::from_slice(before.canonical_bytes())?;
        assert_eq!(before["resource_closure"], "closed");
        assert!(before.get("payload").is_some());
        let refused = execute_context_combined(&stage_host, selected, &stop)?;
        assert_eq!(refused.status(), ContextMapStatus::Failed);
        assert_eq!(
            refused.failure_code(),
            Some(ServiceErrorCode::BudgetExceeded)
        );
        let refused: Value = serde_json::from_slice(refused.canonical_bytes())?;
        assert_eq!(refused["operation"], "context_combined_build");
        assert_eq!(refused["resource_closure"], "closed");
        assert!(refused.get("payload").is_none());
    }
    assert!(source_absent_paths.iter().all(|path| !path.exists()));

    if let Some(destination) = &export {
        use std::io::Write;
        let cli_case = json!({
            "authority": "synthetic_fixture_only_not_live_ketho_or_runtime",
            "source_absent_paths": source_absent_paths,
            "primary": {"store_generation_id": primary_exact, "publication_set_id": primary_publication, "root_id": primary_root},
            "platform": {"store_generation_id": platform_exact, "publication_set_id": original_native["publication_set_id"],
                "root_id": original_native["page"]["body"]["neighborhood"]["root"]["node_id"],
                "candidate_id": original_native["candidate_id"], "candidate_digest": original_native["candidate_digest"]},
            "argv": ["context", "map", "--platform", "combined", "--native-host-config", destination.join("host.json"),
                "--combined-request-input", destination.join("request.json"), "--output", "envelope-json"],
            "expected_envelope": "expected-envelope.json", "expected_exit": 2,
            "stdout": "expected service bytes followed by one LF", "resource_closure": "closed"
        });
        let mut inspect_cli_case = cli_case.clone();
        inspect_cli_case["argv"] = json!([
            "context",
            "inspect",
            "--platform",
            "combined",
            "--native-host-config",
            destination.join("host.json"),
            "--combined-request-input",
            destination.join("inspect-request.json"),
            "--output",
            "envelope-json"
        ]);
        inspect_cli_case["expected_envelope"] = json!("expected-inspect-envelope.json");
        let mut build_cli_case = cli_case.clone();
        build_cli_case["argv"] = json!([
            "context",
            "build",
            "--platform",
            "combined",
            "--native-host-config",
            destination.join("host.json"),
            "--combined-request-input",
            destination.join("build-request.json"),
            "--output",
            "envelope-json"
        ]);
        build_cli_case["expected_envelope"] = json!("expected-build-envelope.json");
        for (name, bytes, limit) in [
            (
                "host.json",
                wow_core::canonical_json_bytes(&host_wire)?,
                CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
            ),
            (
                "request.json",
                wow_core::canonical_json_bytes(&request)?,
                CONTEXT_COMBINED_REQUEST_MAX_BYTES,
            ),
            (
                "cli-case.json",
                wow_core::canonical_json_bytes(&cli_case)?,
                64 * 1024,
            ),
            (
                "expected-envelope.json",
                observed.canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
            (
                "inspect-request.json",
                wow_core::canonical_json_bytes(&inspect_request)?,
                CONTEXT_COMBINED_REQUEST_MAX_BYTES,
            ),
            (
                "inspect-cli-case.json",
                wow_core::canonical_json_bytes(&inspect_cli_case)?,
                64 * 1024,
            ),
            (
                "expected-inspect-envelope.json",
                inspect_observed.canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
            (
                "build-request.json",
                wow_core::canonical_json_bytes(&build_request)?,
                CONTEXT_COMBINED_REQUEST_MAX_BYTES,
            ),
            (
                "build-cli-case.json",
                wow_core::canonical_json_bytes(&build_cli_case)?,
                64 * 1024,
            ),
            (
                "expected-build-envelope.json",
                build_observed.canonical_bytes().to_vec(),
                CONTEXT_MAP_RESULT_MAX_BYTES,
            ),
        ] {
            if bytes.len() > limit {
                return Err("combined fixture export exceeded finite byte bound".into());
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination.join(name))?;
            file.write_all(&bytes)?;
        }
    }
    Ok(())
}

fn assert_combined_pack_and_renderers(
    build_payload: &Value,
    none_payload: &Value,
    inspect: &Value,
    original_platform_page: &Value,
) -> TestResult {
    let inspect_core = &inspect["core"];
    let inspect_map = &inspect_core["map"];
    let inspect_primary_map = &inspect_map["core"]["primary"];
    let skeletons = &inspect_core["primary_skeletons"];
    let original_platform_map = &inspect_map["core"]["platform"];
    let original_platform_inspect = &inspect_core["platform_inspect"];
    let pack = &build_payload["semantic_pack"];
    let pack_core = &pack["core"];
    let semantic_bytes = wow_core::canonical_json_bytes(pack)?;
    assert_eq!(
        build_payload["semantic_pack_byte_length"],
        semantic_bytes.len()
    );
    assert_eq!(
        *pack, none_payload["semantic_pack"],
        "renderer-independent native pack"
    );
    assert_eq!(
        semantic_bytes,
        wow_core::canonical_json_bytes(&none_payload["semantic_pack"])?
    );
    assert_eq!(pack_core["schema"], "wow-context/semantic-pack/combined/1");
    let profiles: Value = serde_json::from_slice(wow_context::COMBINED_CONTEXT_PROFILES_BYTES)?;
    assert_eq!(pack_core["profile"], profiles["pack"]);
    assert_eq!(
        pack_core["origins"], *inspect,
        "exact original map/skeleton/support closure"
    );
    assert_eq!(pack_core["state"], "partial");
    assert_eq!(pack_core["absence_authoritative"], false);
    let normalized = &pack_core["request"]["core"];
    assert_eq!(
        normalized["profile_id"],
        wow_context::COMBINED_CONTEXT_PACK_PROFILE
    );
    assert_eq!(normalized["origin_inspect_id"], inspect["inspect_id"]);
    assert_eq!(
        normalized["combined_map_id"],
        inspect_map["combined_map_id"]
    );
    assert_eq!(
        normalized["primary_map_id"],
        inspect_primary_map["project_map_id"]
    );
    assert_eq!(
        normalized["primary_l0_id"],
        skeletons["l0"]["l0_skeleton_id"]
    );
    assert_eq!(
        normalized["primary_l1_id"],
        skeletons["l1"]["l1_skeleton_id"]
    );
    assert_eq!(
        normalized["platform_map_id"],
        original_platform_map["map_id"]
    );
    assert_eq!(
        normalized["platform_inspect_id"],
        original_platform_inspect["inspect_id"]
    );
    assert!(normalized.get("renderer_profile_ids").is_none());

    // Compare every qualified item to the ORIGINAL native populations. These
    // decoded test observations are assertions, never reconstructed authority.
    let items = pack_core["items"]
        .as_array()
        .ok_or("qualified native pack items")?;
    let selection = pack_core["selection_trace"]
        .as_array()
        .ok_or("qualified native selection trace")?;
    assert_eq!(items.len(), selection.len());
    let mut item_cursor = 0usize;
    let mut item_ids = std::collections::BTreeSet::new();
    let mut qualified_keys = std::collections::BTreeSet::new();
    let mut relation_count = 0usize;
    for (owner, original_map, neighborhood) in [
        (
            "primary",
            inspect_primary_map,
            &inspect_primary_map["core"]["navigation"],
        ),
        (
            "platform",
            original_platform_map,
            &original_platform_page["body"]["neighborhood"],
        ),
    ] {
        let root_node = &neighborhood["root"];
        let root_id = root_node["node_id"]
            .as_str()
            .ok_or("native owner root ID")?;
        let map_id = original_map[if owner == "primary" {
            "project_map_id"
        } else {
            "map_id"
        }]
        .as_str()
        .ok_or("original native owner map ID")?;
        let mut nodes = std::collections::BTreeMap::new();
        for node in std::iter::once(root_node).chain(
            neighborhood["adjacent_nodes"]
                .as_array()
                .ok_or("native adjacent nodes")?
                .iter(),
        ) {
            let id = node["node_id"].as_str().ok_or("native member node ID")?;
            if let Some(previous) = nodes.insert(id, node) {
                assert_eq!(previous, node, "same-owner repeated root must be exact");
            }
        }
        assert!(!nodes.is_empty());
        let edges = neighborhood["edges"]
            .as_array()
            .ok_or("native selected edges")?;
        for pair in edges.windows(2) {
            assert!(
                pair[0]["edge_id"]
                    .as_str()
                    .ok_or("ordered native edge ID")?
                    < pair[1]["edge_id"]
                        .as_str()
                        .ok_or("ordered native edge ID")?
            );
        }
        for (kind, record, native_id) in nodes
            .values()
            .map(|node| ("entity", *node, "node_id"))
            .chain(edges.iter().map(|edge| ("relation", edge, "edge_id")))
        {
            let item = items.get(item_cursor).ok_or("native item omitted")?;
            let row = &item["core"];
            let id = item["context_item_id"]
                .as_str()
                .ok_or("qualified item ID")?;
            let native_id = record[native_id].as_str().ok_or("native record ID")?;
            assert!(item_ids.insert(id), "duplicate qualified item ID");
            assert_eq!(row["schema"], "wow-context/context-item/combined/1");
            assert_eq!(row["owner"], owner);
            assert_eq!(row["origin_map_id"], map_id);
            assert_eq!(row["universe"], root_node["universe"]);
            assert_eq!(row["generation"], root_node["generation"]);
            assert_eq!(row["scope"], json!({"kind": "entity", "root_id": root_id}));
            assert_eq!(row["payload"]["kind"], kind);
            assert_eq!(row["privacy"], "local_metadata_only");
            assert_eq!(row["license"], "source_inclusion_not_permitted");
            assert_eq!(row["source_boundary"], "untrusted_native_metadata");
            assert!(qualified_keys.insert((
                owner,
                row["universe"].as_str().ok_or("native universe")?,
                row["generation"].as_str().ok_or("native generation")?,
                root_id,
                map_id,
                kind,
                native_id,
            )));
            if kind == "entity" {
                let reason = if native_id == root_id {
                    "exact_native_root"
                } else if owner == "primary" {
                    "selected_original_primary_native_record"
                } else {
                    "selected_original_platform_native_neighbor"
                };
                assert_eq!(row["inclusion_reason"], reason);
                assert_eq!(row["payload"]["node"], *record);
            } else {
                assert_eq!(
                    row["inclusion_reason"],
                    "original_selected_direct_native_assertion"
                );
                assert_eq!(row["payload"]["edge"], *record);
                for endpoint in ["from", "to"] {
                    let endpoint = record[endpoint].as_str().ok_or("native edge endpoint")?;
                    let node = nodes.get(endpoint).ok_or("native endpoint omitted")?;
                    assert_eq!(node["universe"], row["universe"]);
                    assert_eq!(node["generation"], row["generation"]);
                }
                relation_count += 1;
            }
            let trace = &selection[item_cursor];
            assert_eq!(trace["item_id"], id);
            assert_eq!(trace["owner"], owner);
            assert_eq!(trace["origin_map_id"], map_id);
            assert_eq!(trace["reason"], row["inclusion_reason"]);
            assert_eq!(trace["required"], true);
            item_cursor += 1;
        }
    }
    assert_eq!(
        item_cursor,
        items.len(),
        "extra cross-owner or unqualified native item"
    );
    assert!(
        relation_count > 0,
        "genuine fixture must exercise relation membership"
    );

    let renderers = build_payload["rendered_artifacts"]
        .as_array()
        .ok_or("validated combined rendered artifacts")?;
    assert_eq!(renderers.len(), 2);
    let markdown_templates: Vec<_> = [
        "pack_header",
        "pack_profile",
        "combined_native_origins",
        "pack_request",
        "boundary_notice",
    ]
    .into_iter()
    .chain(items.iter().map(|item| {
        if item["core"]["payload"]["kind"] == "entity" {
            "qualified_native_entity_item"
        } else {
            "qualified_native_relation_item"
        }
    }))
    .chain(["selection_trace", "coverage_and_omissions", "pack_budget"])
    .collect();
    for (index, renderer) in renderers.iter().enumerate() {
        let row = &renderer["core"];
        let profile = if index == 0 { "json" } else { "markdown" };
        assert_eq!(
            row["schema"],
            "wow-context/rendered-context-artifact/combined/1"
        );
        assert_eq!(row["profile"], profiles[profile]);
        assert_eq!(row["semantic_pack_id"], pack["semantic_pack_id"]);
        let content = row["content"].as_str().ok_or("exact renderer content")?;
        assert_eq!(row["byte_length"], content.len());
        assert_eq!(
            row["content_digest"],
            serde_json::to_value(digest(content.as_bytes()))?
        );
        assert_eq!(
            row["content_digest_basis"],
            "sha256_exact_rendered_utf8_bytes"
        );
        let traces = row["trace"].as_array().ok_or("exact renderer byte trace")?;
        if index == 0 {
            assert_eq!(content.as_bytes(), semantic_bytes.as_slice());
            assert!(!content.ends_with('\n'));
            assert_eq!(traces.len(), 1);
            assert_eq!(traces[0]["record_id"], pack["semantic_pack_id"]);
            assert_eq!(traces[0]["template_id"], "canonical_pack_json");
            assert_eq!(
                traces[0]["semantic_fields"],
                json!(["semantic_pack_id", "core"])
            );
            assert!(traces[0].get("item_index").is_none());
            assert_eq!(traces[0]["byte_start"], 0);
            assert_eq!(traces[0]["byte_end"], content.len());
            continue;
        }
        assert_eq!(traces.len(), markdown_templates.len());
        let mut offset = 0usize;
        let mut rendered_item_cursor = 0usize;
        for (trace, template) in traces.iter().zip(&markdown_templates) {
            assert_eq!(trace["template_id"], *template);
            let start = usize::try_from(trace["byte_start"].as_u64().ok_or("trace start")?)?;
            let end = usize::try_from(trace["byte_end"].as_u64().ok_or("trace end")?)?;
            assert_eq!(start, offset);
            assert!(start < end && end <= content.len());
            let record = content.get(start..end).ok_or("UTF8 trace boundary")?;
            assert_eq!(record.bytes().filter(|byte| *byte == b'\n').count(), 1);
            let (label, quoted) = record
                .strip_suffix('\n')
                .ok_or("Markdown record LF")?
                .split_once(' ')
                .ok_or("Markdown typed record framing")?;
            let decoded: String = serde_json::from_str(quoted)?;
            let observed_record: Value = serde_json::from_str(&decoded)?;
            let (expected_label, expected_record, fields) = match *template {
                "pack_header" => (
                    "PACK",
                    json!({"schema": pack_core["schema"],
                    "semantic_pack_id": pack["semantic_pack_id"], "item_count": items.len()}),
                    json!(["semantic_pack_id", "core.schema"]),
                ),
                "pack_profile" => (
                    "PROFILE",
                    pack_core["profile"].clone(),
                    json!(["core.profile"]),
                ),
                "combined_native_origins" => (
                    "ORIGINS",
                    pack_core["origins"].clone(),
                    json!(["core.origins"]),
                ),
                "pack_request" => (
                    "REQUEST",
                    pack_core["request"].clone(),
                    json!(["core.request"]),
                ),
                "boundary_notice" => (
                    "BOUNDARY",
                    pack_core["boundary_notice"].clone(),
                    json!(["core.boundary_notice"]),
                ),
                "qualified_native_entity_item" | "qualified_native_relation_item" => {
                    let item_index =
                        usize::try_from(trace["item_index"].as_u64().ok_or("trace item index")?)?;
                    assert_eq!(item_index, rendered_item_cursor);
                    rendered_item_cursor += 1;
                    let item = items
                        .get(item_index)
                        .ok_or("rendered item outside sealed pack")?;
                    assert_eq!(trace["record_id"], item["context_item_id"]);
                    (
                        if *template == "qualified_native_entity_item" {
                            "ENTITY"
                        } else {
                            "RELATION"
                        },
                        item.clone(),
                        json!(["core.items"]),
                    )
                }
                "selection_trace" => (
                    "SELECTION_TRACE",
                    pack_core["selection_trace"].clone(),
                    json!(["core.selection_trace"]),
                ),
                "coverage_and_omissions" => (
                    "COVERAGE",
                    json!([
                        pack_core["omissions"],
                        pack_core["state"],
                        pack_core["absence_authoritative"]
                    ]),
                    json!(["core.omissions", "core.state", "core.absence_authoritative"]),
                ),
                "pack_budget" => (
                    "BUDGET",
                    pack_core["budget"].clone(),
                    json!(["core.budget"]),
                ),
                _ => return Err("unknown combined Markdown record".into()),
            };
            assert_eq!(label, expected_label);
            assert_eq!(
                observed_record, expected_record,
                "exact lossless Markdown record"
            );
            assert_eq!(trace["semantic_fields"], fields);
            if !matches!(
                *template,
                "qualified_native_entity_item" | "qualified_native_relation_item"
            ) {
                assert!(trace.get("item_index").is_none());
                assert_eq!(
                    &trace["record_id"],
                    if *template == "combined_native_origins" {
                        &inspect["inspect_id"]
                    } else {
                        &pack["semantic_pack_id"]
                    }
                );
            }
            offset = end;
        }
        assert_eq!(rendered_item_cursor, items.len());
        assert_eq!(offset, content.len(), "complete Markdown trace closure");
    }

    Ok(())
}

// Reuse the exact source-absent stores from a successful cold lifecycle. This
// selected witness skips publication/control campaigns, not native acquisition.
#[test]
#[ignore = "requires explicit retained source-absent fixture and fresh export directory"]
fn retained_source_absent_combined_pack_and_renderers() -> TestResult {
    use std::io::{Read, Write};
    let stop = AtomicBool::new(false);
    let fixture = PathBuf::from(
        std::env::var_os("WOW_CONTEXT_COMBINED_RETAINED_FIXTURE")
            .ok_or("explicit retained combined fixture required")?,
    );
    let export = PathBuf::from(
        std::env::var_os("WOW_CONTEXT_COMBINED_BUILD_EXPORT")
            .ok_or("explicit fresh combined Build export required")?,
    );
    if !fixture.is_absolute() || !export.is_absolute() {
        return Err("absolute fixture/export paths required".into());
    }
    let read = |name: &str, limit: usize| -> TestResult<Vec<u8>> {
        let mut bytes = Vec::new();
        std::fs::File::open(fixture.join(name))?
            .take(u64::try_from(limit)? + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > limit {
            return Err("retained fixture exceeds byte bound".into());
        }
        Ok(bytes)
    };
    let old: Value = serde_json::from_slice(&read(
        "expected-inspect-envelope.json",
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )?)?;
    assert_eq!(old["resource_closure"], "closed");
    assert_eq!(old["status"], "partial");
    let mut case: Value = serde_json::from_slice(&read("inspect-cli-case.json", 65536)?)?;
    let absent = case["source_absent_paths"]
        .as_array()
        .ok_or("source-absent witness paths")?;
    assert!(!absent.is_empty());
    for path in absent {
        assert!(!PathBuf::from(path.as_str().ok_or("source-absent path")?).exists());
    }
    let mut host_wire: Value =
        serde_json::from_slice(&read("host.json", CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES)?)?;
    // Measured Both: 11.87m work, 10.15m retained + 1.85m delivery. The original
    // 8m refusal remains retained; select finite headroom within the existing cap.
    for owner in ["primary", "platform"] {
        host_wire[owner]["limits"]["max_context_work"] = json!(16_000_000);
        host_wire[owner]["limits"]["max_metadata_bytes"] = json!(16 * 1024 * 1024);
    }
    let host_bytes = wow_core::canonical_json_bytes(&host_wire)?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;
    let mut wire: Value =
        serde_json::from_slice(&read("request.json", CONTEXT_COMBINED_REQUEST_MAX_BYTES)?)?;
    wire["operation"] = json!("build");
    wire["build"] = json!({"level":"both", "context_profile_id": wow_context::COMBINED_CONTEXT_PACK_PROFILE, "renderer_profile_ids":[]});
    let none =
        ContextCombinedRequest::from_json_slice(&wow_core::canonical_json_bytes(&wire)?, &stop)?;
    let none_result = execute_context_combined(&host, &none, &stop)?;
    assert_eq!(
        none_result.status(),
        ContextMapStatus::Partial,
        "None failure: {:?}",
        none_result.failure_code()
    );
    assert_eq!(none_result.failure_code(), None);
    let none_envelope: Value = serde_json::from_slice(none_result.canonical_bytes())?;
    assert_eq!(none_envelope["resource_closure"], "closed");
    assert_eq!(none_envelope["payload"]["rendered_artifacts"], json!([]));
    wire["build"]["renderer_profile_ids"] = json!([
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
    ]);
    let request_bytes = wow_core::canonical_json_bytes(&wire)?;
    let request = ContextCombinedRequest::from_json_slice(&request_bytes, &stop)?;
    let start = std::time::Instant::now();
    let result = execute_context_combined(&host, &request, &stop)?;
    let operation_seconds = start.elapsed().as_secs_f64();
    assert_eq!(
        result.status(),
        ContextMapStatus::Partial,
        "Both failure: {:?}",
        result.failure_code()
    );
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], CONTEXT_COMBINED_RESULT_SCHEMA);
    assert_eq!(envelope["operation"], "context_combined_build");
    assert_eq!(envelope["request"], wire);
    assert_eq!(envelope["resource_closure"], "closed");
    assert_ne!(envelope["request_id"], none_envelope["request_id"]);
    let payload = &envelope["payload"];
    let inspect = &none_envelope["payload"]["semantic_pack"]["core"]["origins"];
    let original = &old["payload"]["combined_inspect"]["core"]["map"]["core"];
    let current = &inspect["core"]["map"]["core"];
    // Changed caller limits affect envelope identities; original populations and
    // native support must still equal the independent cold/reopen observation.
    for field in ["binding", "navigation", "supports"] {
        assert_eq!(
            current["primary"]["core"][field],
            original["primary"]["core"][field]
        );
    }
    assert_eq!(
        current["platform"]["core"]["native"],
        original["platform"]["core"]["native"]
    );
    assert_combined_pack_and_renderers(
        payload,
        &none_envelope["payload"],
        inspect,
        &original["platform"]["core"]["native"],
    )?;
    for key in ["primary", "platform", "platform_selection"] {
        assert_eq!(payload[key], old["payload"][key]);
    }
    assert_eq!(
        payload["validation"],
        "exact_native_binding_and_canonical_bytes"
    );
    let usage = &payload["operation_budget_usage"];
    assert!(usage["work_reserved"].as_u64().ok_or("work")? > 8_000_000);
    assert!(usage["work_reserved"].as_u64().ok_or("work")? < 16_000_000);
    assert!(
        usage["bytes_reserved"].as_u64().ok_or("bytes")?
            + u64::try_from(result.canonical_bytes().len())?
            < 16 * 1024 * 1024
    );
    eprintln!(
        "combined retained Both: seconds={operation_seconds} usage={usage} envelope_bytes={}",
        result.canonical_bytes().len()
    );
    std::fs::create_dir(&export)?;
    case["argv"] = json!([
        "context",
        "build",
        "--platform",
        "combined",
        "--native-host-config",
        export.join("host.json"),
        "--combined-request-input",
        export.join("build-request.json"),
        "--output",
        "envelope-json"
    ]);
    case["expected_envelope"] = json!("expected-build-envelope.json");
    for (name, bytes) in [
        ("host.json", host_bytes),
        ("build-request.json", request_bytes),
        (
            "expected-build-envelope.json",
            result.canonical_bytes().to_vec(),
        ),
        (
            "build-cli-case.json",
            wow_core::canonical_json_bytes(&case)?,
        ),
    ] {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(export.join(name))?
            .write_all(&bytes)?;
    }
    Ok(())
}

fn skeleton_xsd() -> String {
    CLASSIFICATION_XSD.replace(
        "<xs:complexType name=\"TextureType\"/>",
        r#"<xs:complexType name="TextureType"><xs:sequence>
          <xs:element ref="u:ShapeLeaf"/>
        </xs:sequence></xs:complexType>
        <xs:complexType name="ShapeLeafType"><xs:sequence>
          <xs:element name="leaf" minOccurs="0" maxOccurs="2"><xs:complexType/></xs:element>
        </xs:sequence></xs:complexType>
        <xs:element name="ShapeLeaf" type="u:ShapeLeafType"/>"#,
    )
}

type SelectedSkeletonQuery = (
    ProjectView,
    GraphBuildRequest,
    Vec<Box<str>>,
    usize,
    RegionTypeWitness,
);
fn selected_skeleton_query(
    path: &std::path::Path,
    config: &Value,
    stop: &AtomicBool,
) -> TestResult<SelectedSkeletonQuery> {
    let (project, request, ids, source_bytes) =
        super::super::function_fingerprints::declaration_query(path, stop)?;
    let functions = FunctionFingerprintSelection::new(
        ids.clone(),
        ids.len(),
        source_bytes,
        GRAPH_RESULT_MAX_BYTES,
    )?
    .with_direct_neighborhood(FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1)?;
    let request = request.with_function_fingerprints(functions)?;
    // This ordinary operation discovers exact native XML proposal IDs only.
    // It is never used as candidate authority or an independently chosen final owner.
    let discovery = execute_graph_build(
        LocalProjectInput::from_config_path(path, stop)?,
        &request,
        stop,
    )?;
    assert_eq!(
        discovery.status(),
        GraphReadStatus::Partial,
        "{:?}",
        discovery.failure_message()
    );
    let wire = serde_json::to_value(&discovery)?;
    let documents = wire["provenance"]["xml_classification"]["documents"]
        .as_array()
        .ok_or("native classification documents")?;
    let [document] = documents.as_slice() else {
        return Err("expected one fixture classification document".into());
    };
    let observations = document["report"]["occurrences"]
        .as_array()
        .ok_or("native XML observations")?;
    let projections = document["occurrences"]
        .as_array()
        .ok_or("native XML projections")?;
    assert_eq!(observations.len(), projections.len());
    let texture = observations
        .iter()
        .zip(projections)
        .filter(|(observation, _)| observation["qname"]["name"]["local_name"] == "Texture")
        .collect::<Vec<_>>();
    let [(observation, projection)] = texture.as_slice() else {
        return Err("expected one exact fixture Texture occurrence".into());
    };
    assert_eq!(observation["occurrence"], projection["occurrence"]);
    let intent: RegionTypeWitness = serde_json::from_value(
        config["xml_schema_classification"]["profile_selection"]["region_like"]["members"][0]
            .clone(),
    )?;
    let target = XmlFingerprintTarget::new(
        projection["occurrence_proposal_id"]
            .as_str()
            .ok_or("native Texture proposal")?
            .into(),
        Some(
            projection["class_proposal_id"]
                .as_str()
                .ok_or("native Texture class")?
                .into(),
        ),
    )
    .with_type_shape(intent.element, intent.type_component)?;
    let xml = XmlFingerprintSelection::new_with_type_shapes(
        vec![target],
        1,
        GRAPH_RESULT_MAX_BYTES,
        XmlSchemaTypeShapeRecipe::ClosedComplexTreeV1,
    )?;
    assert_eq!(
        serde_json::to_value(&xml)?["schema"],
        "wow-service/xml-fingerprint-selection/selected-components/1"
    );
    let request = request.with_xml_fingerprints(xml)?;
    Ok((project, request, ids, source_bytes, intent))
}

// Acquires the actual sealed candidate again, then uses the registered service
// operation after releasing the first store/read/lookup. JSON is inert assertions.
fn source_absent_skeleton(
    selected_root: &std::path::Path,
    selected_transport: &PlatformProjectSelection,
    selected_selection: &PlatformStoreSelection,
    exact: &str,
    ids: &[Box<str>],
    intent: &RegionTypeWitness,
    stop: &AtomicBool,
) -> TestResult<(Vec<u8>, Value)> {
    let reopened = read_selected_live_project(selected_root, selected_transport, exact, stop)
        .map_err(|error| format!("source-absent selected reopen: {error:?}"))?;
    assert_eq!(reopened.exit_code(), 2);
    let reopened: Value = serde_json::from_slice(reopened.canonical_bytes())?;
    let store = LiveProjectStore::open_in_namespace(selected_root, selected_selection)
        .map_err(|error| format!("typed selected store open: {error:?}"))?;
    let selector = ReadSelector::Exact(StoreGenerationId::parse(exact)?);
    let read = store
        .read_in_namespace(selected_selection, &selector, stop)
        .map_err(|error| format!("typed source-absent read: {error:?}"))?;
    let candidate = read
        .candidate()
        .ok_or("selected rebuild omitted genuine candidate")?;
    assert_eq!(
        reopened["candidate"]["candidate_id"],
        candidate.manifest().candidate_id()
    );
    assert_eq!(
        reopened["candidate"]["candidate_digest"],
        candidate.manifest().candidate_digest()
    );
    let functions = candidate
        .function_fingerprints()
        .ok_or("candidate function manifest")?;
    assert_eq!(functions.records().len(), ids.len());
    assert_eq!(
        functions
            .records()
            .iter()
            .map(|record| record.function_id())
            .collect::<Vec<_>>(),
        ids.iter().map(AsRef::as_ref).collect::<Vec<&str>>()
    );
    assert!(
        functions
            .records()
            .iter()
            .all(|record| record.declared_header().is_some())
    );
    assert_eq!(
        functions
            .selected_components()
            .ok_or("function component receipt")?
            .direct_neighborhood(),
        Some(FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1)
    );
    assert!(functions.neighborhood_binding().is_some());
    let xml = candidate
        .xml_fingerprints()
        .ok_or("candidate selected XML manifest")?;
    assert_eq!(
        xml.schema(),
        "wow-service/xml-fingerprint-manifest/selected-components/1"
    );
    assert_eq!(
        xml.type_shape_recipe(),
        Some(XmlSchemaTypeShapeRecipe::ClosedComplexTreeV1)
    );
    assert_eq!(
        xml.type_shape_for(0)
            .ok_or("candidate type-shape mapping")?
            .type_component(),
        intent.type_component
    );
    let raw = xml.type_shape_support(0).ok_or("candidate XML support")?;
    assert!(!raw.is_empty());
    assert!(
        raw.iter()
            .all(|support| support.source().path().as_str() == "UI/classification.xsd")
    );

    // Select the exact fixture function already named in the query. Resolve its
    // genuine native support in the candidate's one final immutable owner.
    let function = functions
        .records()
        .iter()
        .find(|record| record.function_id() == ids[0].as_ref())
        .ok_or("selected native function record")?;
    let lookup = candidate.graph().producer_lookup(stop)?;
    let accepted_node = lookup
        .entity(
            candidate.native().scope(),
            function.support().entity_assertion(),
            stop,
        )?
        .accepted()
        .node();
    // Accepted producer IDs belong to the input graph. The consumer selects the
    // same exact entity rebound by the graph owner into the final generation.
    let rebound = wow_graph::GraphNode::new(
        candidate.graph().snapshot().universe().clone(),
        candidate.graph().snapshot().generation().clone(),
        accepted_node.kind(),
        accepted_node.owner_key(),
        accepted_node.evidence_ids().to_vec(),
        candidate.graph().snapshot().limits(),
    )?;
    let root_node = candidate
        .graph()
        .snapshot()
        .node(rebound.node_id())
        .ok_or("exact selected entity is absent from final materialization")?
        .node_id()
        .clone();
    let relation = lookup
        .relation(
            candidate.native().scope(),
            function.support().incoming_ownership_assertion(),
            stop,
        )?
        .accepted()
        .edge()
        .relation();
    let neighborhood = GraphNeighborReadQuery::new(
        candidate.graph().snapshot().snapshot_id().clone(),
        GraphNeighborQuery::new(root_node.clone(), GraphDirection::Both, vec![relation], 8)?,
        GraphNeighborReadLimits::default(),
    )?
    .with_confidence(GraphPathConfidence::IncludeCandidate);
    let limits = SkeletonInputLimits {
        max_entities: 64,
        max_support_records: 4096,
        max_output_bytes: 1_048_576,
        root_explanation: GraphExplainLimits::default(),
        root_evidence: GraphEvidenceResolveLimits::default(),
    };
    let input =
        OpenSkeletonInputRequest::new(candidate.native(), candidate.graph(), neighborhood, limits)?;
    assert_eq!(
        serde_json::to_value(input.binding())?,
        reopened["candidate"]["skeleton_binding"]
    );
    let request = serde_json::to_vec(&json!({
        "schema": LIVE_SKELETON_REQUEST_SCHEMA,
        "candidate_id": candidate.manifest().candidate_id(),
        "candidate_digest": candidate.manifest().candidate_digest(),
        "input": input,
    }))?;
    let native = wow_project::skeleton_input::open_skeleton_input_view(
        candidate.native(),
        candidate.graph(),
        input.clone(),
        stop,
    )
    .map_err(|error| format!("held native Skeleton open: {error:?}"))?;
    let native_page = native
        .read(stop)
        .map_err(|error| format!("held native Skeleton page: {error:?}"))?;
    assert_eq!(native_page.state(), GraphQueryState::Partial);
    drop(native_page);
    drop(native);
    drop(lookup);
    drop(read);
    drop(store);
    let observed =
        execute_live_skeleton_read(selected_root, selected_transport, exact, &request, stop)
            .map_err(|error| format!("selected Skeleton service read: {error:?}"))?;
    assert_eq!(observed.state(), GraphQueryState::Partial);
    assert!(observed.canonical_bytes().len() <= limits.max_output_bytes as usize);
    let observed: Value = serde_json::from_slice(observed.canonical_bytes())?;
    assert_eq!(observed["schema"], LIVE_SKELETON_RESULT_SCHEMA);
    assert_eq!(
        observed["candidate_id"],
        reopened["candidate"]["candidate_id"]
    );
    assert_eq!(
        observed["candidate_digest"],
        reopened["candidate"]["candidate_digest"]
    );
    assert_eq!(observed["store_generation_id"], exact);
    assert_eq!(observed["page"]["body"]["state"], "partial");
    assert_eq!(observed["page"]["body"]["no_new_evidence"], false);
    assert_eq!(
        observed["page"]["body"]["neighborhood"]["root"]["node_id"],
        root_node.as_str()
    );
    assert!(
        observed["page"]["body"]["counts"]["returned_root_support_records"]
            .as_u64()
            .ok_or("support count")?
            > 0
    );
    assert!(
        observed["page"]["body"]["counts"]["root_source_handles"]
            .as_u64()
            .ok_or("handle count")?
            > 0
    );
    assert!(
        !observed["page"]["body"]["not_evaluated"]
            .as_array()
            .ok_or("explicit unavailable facets")?
            .is_empty()
    );
    Ok((request, observed))
}

#[test]
fn selected_candidate_update_removal_and_query_fences_reopen_to_skeleton() -> TestResult {
    let stop = AtomicBool::new(false);
    let xsd = skeleton_xsd();
    let extra = [(
        "UI/removable.lua",
        "function RemovedFunction() return false end\n",
    )];
    let both_toc = "main.lua\nremovable.lua\nframes.xml\n";
    let make =
        |lua: &str, toc: &str, extra_main: &[(&str, &str)]| -> TestResult<(FixtureRoot, Value)> {
            let (root, mut config) = classification_fixture_schema_and_main_members(
                &[("UI/classification.xsd", &xsd)],
                CLASSIFICATION_XML,
                Some(lua),
                Some(toc),
                extra_main,
            )?;
            config["physical_function_declarations"] =
                serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
            write_json(&root, "project.json", &config)?;
            Ok((root, config))
        };
    let (root, config) = make(
        "function KeptFunction() return true end\n",
        both_toc,
        &extra,
    )?;
    let path = root.0.join("project.json");
    let (project, graph, ids, _, intent) = selected_skeleton_query(&path, &config, &stop)?;
    let ordinary = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        project
            .configuration()
            .platform_packages()
            .ok_or("source profile")?
            .source()
            .profile()
            .profile_id()
            .clone(),
    )?
    .with_xml_classification_catalog()
    .with_physical_function_declarations_catalog();
    let selected = ordinary.with_composed_candidate_catalog();
    let selection_wire = json!({
        "schema": PLATFORM_PROJECT_SELECTION_SCHEMA,
        "owner_project_id": project.configuration().project_id(),
        "source_profile_id": project.configuration().platform_packages().ok_or("source profile")?
            .source().profile().profile_id(),
        "catalogs": { "xml_schema_classification": true,
            "physical_function_declarations": "declared_syntax_v1",
            "physical_function_signatures": "absent", "composed_candidate": true }
    });
    let transport =
        PlatformProjectSelection::from_json_slice(&serde_json::to_vec(&selection_wire)?, &stop)?;
    let store_root = root.0.join("update-candidate-store");
    let initial = serde_json::to_value(publish_selected_local_project(
        &path,
        &graph,
        &store_root,
        &transport,
        &LiveProjectPublishRequest::new("fixture:candidate-update-base", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(initial["status"], "activated");
    let initial_exact = initial["operation"]["generation_id"]
        .as_str()
        .ok_or("base generation")?
        .to_owned();
    let base = LiveProjectStore::open_in_namespace(&store_root, &selected)?
        .current()?
        .ok_or("base Current")?;

    let (updated_root, updated_config) = make(
        "function KeptFunction() return false end\n",
        both_toc,
        &extra,
    )?;
    let updated_path = updated_root.0.join("project.json");
    let (updated_project, updated_graph, updated_ids, updated_bytes, updated_intent) =
        selected_skeleton_query(&updated_path, &updated_config, &stop)?;
    // Exact loaded Main declarations, not fixture annotations. A changed Lua
    // digest MUST reach activated, never be accepted as a declaration refusal.
    let before = LocalProjectInput::from_config_path(&path, &stop)?;
    let after = LocalProjectInput::from_config_path(&updated_path, &stop)?;
    assert_eq!(
        before.project_bundle().configuration().project_id(),
        after.project_bundle().configuration().project_id()
    );
    let before_main = before
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("old packages")?
        .main_plan()
        .resolve_source("Fixture", "main.lua")
        .ok_or("old Main path")?;
    let after_main = after
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("new packages")?
        .main_plan()
        .resolve_source("Fixture", "main.lua")
        .ok_or("new Main path")?;
    assert_eq!(before_main, after_main);
    let previous = before
        .project_bundle()
        .inventory()
        .files()
        .iter()
        .find(|file| file.relative_path().as_str() == before_main)
        .ok_or("old Main file")?;
    let next = after
        .project_bundle()
        .inventory()
        .files()
        .iter()
        .find(|file| file.relative_path().as_str() == after_main)
        .ok_or("new Main file")?;
    assert_eq!(previous.role(), next.role());
    assert_eq!(previous.language_kind(), next.language_kind());
    assert_eq!(previous.source_fixture_ref(), None);
    assert_eq!(next.source_fixture_ref(), previous.source_fixture_ref());
    assert_ne!(previous.content_digest(), next.content_digest());
    let removable_path = before
        .project_bundle()
        .configuration()
        .platform_packages()
        .ok_or("old packages")?
        .main_plan()
        .resolve_source("Fixture", "removable.lua")
        .ok_or("removable Main path")?
        .to_owned();
    let removable_id = project
        .file_by_path(&removable_path)?
        .ok_or("removable native file")?
        .file_id()
        .clone();
    let updated = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &updated_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-content-update",
            base.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        updated["status"], "activated",
        "positive text update was refused"
    );
    let updated_exact = updated["operation"]["generation_id"]
        .as_str()
        .ok_or("updated generation")?
        .to_owned();
    assert_ne!(updated_exact, initial_exact);
    let content_current = LiveProjectStore::open_in_namespace(&store_root, &selected)?
        .current()?
        .ok_or("updated Current")?;
    assert_ne!(content_current.record_id, base.record_id);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let active = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    assert_eq!(
        active.project().snapshot_id(),
        updated_project.snapshot_id()
    );
    assert_eq!(
        active
            .project()
            .file_by_path(after_main)?
            .ok_or("updated Main file")?
            .content_digest(),
        next.content_digest()
    );
    drop(active);
    drop(store);
    drop(before);
    drop(after);

    // The project/config/source is IDENTICAL: only the final query selection is
    // narrowed to one native fact. A changed candidate must still publish.
    assert!(updated_ids.len() > 1);
    let query_ids = vec![updated_ids[0].clone()];
    let query_graph = updated_graph.clone().with_function_fingerprints(
        FunctionFingerprintSelection::new(
            query_ids.clone(),
            1,
            updated_bytes,
            GRAPH_RESULT_MAX_BYTES,
        )?
        .with_direct_neighborhood(
            FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1,
        )?,
    )?;
    let queried = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &query_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-query-update",
            content_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        queried["status"], "activated",
        "query-only candidate change was discarded"
    );
    let query_exact = queried["operation"]["generation_id"]
        .as_str()
        .ok_or("query generation")?
        .to_owned();
    assert_ne!(query_exact, updated_exact);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let query_current = store.current()?.ok_or("query Current")?;
    assert_ne!(query_current.record_id, content_current.record_id);
    let old = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    let new = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&query_exact)?),
        &stop,
    )?;
    assert_eq!(old.project().snapshot_id(), updated_project.snapshot_id());
    assert_eq!(new.project().snapshot_id(), old.project().snapshot_id());
    assert_eq!(
        new.project().project_generation(),
        old.project().project_generation()
    );
    assert_eq!(
        new.project().configuration().configuration_digest(),
        old.project().configuration().configuration_digest()
    );
    assert_eq!(
        new.project().analyzer_snapshot_id(),
        old.project().analyzer_snapshot_id()
    );
    assert_eq!(new.graph(), old.graph());
    assert_ne!(
        new.candidate()
            .ok_or("query candidate")?
            .manifest()
            .candidate_id(),
        old.candidate()
            .ok_or("content candidate")?
            .manifest()
            .candidate_id()
    );
    assert_eq!(
        new.candidate()
            .ok_or("query candidate")?
            .function_fingerprints()
            .ok_or("query manifest")?
            .records()
            .len(),
        1
    );
    drop(new);
    drop(old);
    drop(store);
    let unchanged = serde_json::to_value(update_selected_local_project(
        &updated_path,
        &query_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-query-no-change",
            query_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(unchanged["status"], "no_change");
    assert!(unchanged.get("operation").is_none());
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    assert_eq!(store.current()?, Some(query_current.clone()));
    assert!(
        store
            .reconcile("fixture:candidate-query-no-change")?
            .is_none()
    );
    drop(store);

    let stale = LiveProjectUpdateRequest::new(
        "fixture:candidate-update-stale",
        base.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    assert_eq!(
        update_selected_local_project(
            &updated_path,
            &query_graph,
            &store_root,
            &transport,
            &stale,
            &stop
        )
        .err()
        .ok_or("stale Current accepted")?
        .code(),
        ServiceErrorCode::StoreCurrentConflict
    );
    let cancelled = LiveProjectUpdateRequest::new(
        "fixture:candidate-update-cancelled",
        query_current.record_id.as_str(),
        LiveProjectLibraryMode::Keep,
        true,
    )?;
    // Pre-admission cancellation only; no timing claim about a late CAS probe.
    assert_eq!(
        update_selected_local_project(
            &updated_path,
            &query_graph,
            &store_root,
            &transport,
            &cancelled,
            &AtomicBool::new(true)
        )
        .err()
        .ok_or("cancelled update accepted")?
        .code(),
        ServiceErrorCode::Cancelled
    );
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    assert_eq!(store.current()?, Some(query_current.clone()));
    assert!(store.reconcile("fixture:candidate-update-stale")?.is_none());
    assert!(
        store
            .reconcile("fixture:candidate-update-cancelled")?
            .is_none()
    );
    drop(store);

    // Remove the physical member from BOTH exact source manifest and native TOC,
    // while keeping main.lua so the existing nonempty Main contract is satisfied.
    let (removed_root, removed_config) = make(
        "function KeptFunction() return false end\n",
        "main.lua\nframes.xml\n",
        &[],
    )?;
    let removed_path = removed_root.0.join("project.json");
    let (removed_project, removed_graph, removed_ids, _, removed_intent) =
        selected_skeleton_query(&removed_path, &removed_config, &stop)?;
    let packages = removed_project
        .configuration()
        .platform_packages()
        .ok_or("removed packages")?;
    assert!(
        packages
            .source()
            .receipt()
            .inventory()
            .entries
            .iter()
            .all(|entry| entry.path != "UI/removable.lua")
    );
    assert!(
        packages
            .main_plan()
            .resolve_source("Fixture", "removable.lua")
            .is_none()
    );
    assert!(
        packages
            .main_plan()
            .resolve_source("Fixture", "main.lua")
            .is_some()
    );
    assert!(!removed_root.0.join("UI/removable.lua").exists());
    assert!(removed_project.file_by_id(&removable_id).is_none());
    let removed_report = removed_project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("removed report")?;
    assert!(
        removed_report
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert!(removed_ids.len() < updated_ids.len());
    let removed = serde_json::to_value(update_selected_local_project(
        &removed_path,
        &removed_graph,
        &store_root,
        &transport,
        &LiveProjectUpdateRequest::new(
            "fixture:candidate-member-remove",
            query_current.record_id.as_str(),
            LiveProjectLibraryMode::Keep,
            true,
        )?,
        &stop,
    )?)?;
    assert_eq!(
        removed["status"], "activated",
        "positive physical removal was refused"
    );
    let removed_exact = removed["operation"]["generation_id"]
        .as_str()
        .ok_or("removal generation")?
        .to_owned();
    assert_ne!(removed_exact, query_exact);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let removal_current = store.current()?.ok_or("removal Current")?;
    assert_ne!(removal_current.record_id, query_current.record_id);
    let read = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&removed_exact)?),
        &stop,
    )?;
    assert_eq!(read.project().snapshot_id(), removed_project.snapshot_id());
    assert!(read.project().file_by_id(&removable_id).is_none());
    assert!(read.project().file_by_path(&removable_path)?.is_none());
    assert!(
        read.project()
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("reopened removed report")?
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert_ne!(
        read.project().project_generation(),
        updated_project.project_generation()
    );
    drop(read);
    drop(store);
    let updated_snapshot = updated_project.snapshot_id().to_owned();
    let removed_snapshot = removed_project.snapshot_id().to_owned();
    drop(project);
    drop(updated_project);
    drop(removed_project);
    // Optional root-owned CLI export: preserve final bytes before fixture deletion.
    let cli_export = export_cli_fixture_if_requested(
        &removed_root,
        &[&root, &updated_root, &removed_root],
        &removed_graph,
        &selection_wire,
    )?;
    // Only temporary fixture files, only when root executes the test. Every
    // reopened generation below must rebuild from the same retained owner route.
    for fixture in [&root, &updated_root, &removed_root] {
        std::fs::remove_dir_all(fixture.0.join("UI"))?;
        std::fs::remove_file(fixture.0.join("version.txt"))?;
        assert!(!fixture.0.join("UI/main.lua").exists());
        assert!(!fixture.0.join("UI/classification.xsd").exists());
    }
    let (_, initial_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &initial_exact,
        &ids,
        &intent,
        &stop,
    )?;
    let (content_request, content_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &updated_exact,
        &updated_ids,
        &updated_intent,
        &stop,
    )?;
    let (_, query_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &query_exact,
        &query_ids,
        &updated_intent,
        &stop,
    )?;
    let (removed_request, removed_skeleton) = source_absent_skeleton(
        &store_root,
        &transport,
        &selected,
        &removed_exact,
        &removed_ids,
        &removed_intent,
        &stop,
    )?;
    assert_ne!(
        initial_skeleton["candidate_id"],
        content_skeleton["candidate_id"]
    );
    assert_ne!(
        content_skeleton["candidate_id"],
        query_skeleton["candidate_id"]
    );
    assert_ne!(
        query_skeleton["candidate_id"],
        removed_skeleton["candidate_id"]
    );
    // Exact source-absent snapshots, removal invalidation and final Current all
    // survive actual selected reopen, not only the prior in-memory observations.
    let store = LiveProjectStore::open_in_namespace(&store_root, &selected)?;
    let content = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&updated_exact)?),
        &stop,
    )?;
    assert_eq!(content.project().snapshot_id(), updated_snapshot);
    assert!(content.project().file_by_id(&removable_id).is_some());
    let removal = store.read_in_namespace(
        &selected,
        &ReadSelector::Exact(StoreGenerationId::parse(&removed_exact)?),
        &stop,
    )?;
    assert_eq!(removal.project().snapshot_id(), removed_snapshot);
    assert!(removal.project().file_by_id(&removable_id).is_none());
    assert!(
        removal
            .project()
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("source-absent removed report")?
            .functions()
            .iter()
            .all(|fact| fact.path() != removable_path)
    );
    assert_eq!(store.current()?, Some(removal_current));
    if let Some(destination) = &cli_export {
        // Existing genuine content candidate is stale against the final removal.
        std::fs::write(destination.join("skeleton-stale.json"), &content_request)?;
        std::fs::write(destination.join("skeleton.json"), &removed_request)?;
    }
    Ok(())
}

/// Test-only explicit export. No file/serialization work when the root flag is absent.
fn export_cli_fixture_if_requested(
    source: &FixtureRoot,
    fixtures: &[&FixtureRoot],
    graph: &GraphBuildRequest,
    selection: &Value,
) -> TestResult<Option<std::path::PathBuf>> {
    let Some(destination) = std::env::var_os("WOW_W17_116_CLI_EXPORT_ROOT") else {
        return Ok(None);
    };
    let destination = std::path::PathBuf::from(destination);
    if !destination.is_absolute() {
        return Err("CLI fixture export requires an explicit absolute directory".into());
    }
    // Resolve the existing parent before creating the fresh output; it must
    // survive every FixtureRoot destructor and the existing source removals.
    let parent = destination
        .parent()
        .ok_or("CLI export parent")?
        .canonicalize()?;
    let destination = parent.join(destination.file_name().ok_or("CLI export directory name")?);
    for fixture in fixtures {
        if destination.starts_with(fixture.0.canonicalize()?) {
            return Err("CLI export must be outside the temporary fixture roots".into());
        }
    }
    std::fs::create_dir(&destination)?;
    std::fs::create_dir(destination.join("UI"))?;
    // Exact final removal fixture; no directory scan, store copy or second parser.
    for path in [
        "project.json",
        "manifest.json",
        "reference.json",
        "compatibility.json",
        "version.txt",
        "UI/Fixture.toc",
        "UI/main.lua",
        "UI/frames.xml",
        "UI/classification.xsd",
    ] {
        std::fs::copy(source.0.join(path), destination.join(path))?;
    }
    std::fs::write(
        destination.join("selection.json"),
        serde_json::to_vec(selection)?,
    )?;
    let request = serde_json::to_value(graph)?;
    for (file, field) in [
        ("function-query.json", "function_fingerprints"),
        ("xml-query.json", "xml_fingerprints"),
    ] {
        let query = request
            .get(field)
            .filter(|value| value.is_object())
            .ok_or("CLI export requires both actual selected query recipes")?;
        std::fs::write(destination.join(file), serde_json::to_vec(query)?)?;
    }
    Ok(Some(destination))
}

// Only root opts in. Publish the genuine store here before source removal,
// outside FixtureRoot; no store/candidate/graph copy or source materialization.
fn platform_context_export_root(fixture: &FixtureRoot) -> TestResult<Option<std::path::PathBuf>> {
    let Some(path) = std::env::var_os("WOW_CONTEXT_PLATFORM_FIXTURE_EXPORT") else {
        return Ok(None);
    };
    let path = std::path::PathBuf::from(path);
    if !path.is_absolute() || path.as_os_str().len() > 32768 {
        return Err("platform context export requires a finite absolute fresh path".into());
    }
    let parent = path
        .parent()
        .ok_or("platform export parent")?
        .canonicalize()?;
    let path = parent.join(path.file_name().ok_or("platform export directory")?);
    if path.starts_with(fixture.0.canonicalize()?) {
        return Err("platform export must survive fixture/source removal".into());
    }
    std::fs::create_dir(&path)?;
    Ok(Some(path))
}

// Same physical Main/Library source as context/tests.rs input_bundle/owners.
// The shared target profile/Reference are configured BEFORE initial admission.
fn combined_primary_fixture(
    fixture: &FixtureRoot,
    platform_config: &Value,
    profile: &wow_core::ProfileIdentity,
    reference: &wow_reference::ReferenceView,
    stop: &AtomicBool,
) -> TestResult<(PathBuf, GraphBuildRequest)> {
    const MAIN: &str = "local function ContextFixture(value)\n    return value\nend\n";
    const LIBRARY: &str = "---@meta\n---@return number\nfunction External() end\n";
    std::fs::create_dir(fixture.0.join("primary-source"))?;
    std::fs::create_dir(fixture.0.join("primary-source/library"))?;
    std::fs::write(fixture.0.join("primary-source/main.lua"), MAIN)?;
    std::fs::write(fixture.0.join("primary-source/library/Core.lua"), LIBRARY)?;
    let profile_file = write_json(fixture, "combined-primary-profile.json", profile)?;
    let config = json!({
        "schema": crate::local::LOCAL_FILES_SCHEMA,
        "project_id": "fixture-context-primary-map",
        "workspace_id": "workspace:fixture:context-primary-map",
        "source_origin_id": "project-origin:fixture-context-primary-map",
        "logical_root": "fixtures/context-primary/main",
        "profile": profile_file,
        "reference_view": platform_config["reference_view"],
        "analyzer": platform_config["analyzer"],
        "main": {"root": "primary-source", "files": [
            ProjectDiskFile::new("main.lua").with_identity(digest(MAIN.as_bytes()), MAIN.len() as u64)
        ]},
        "library": {"root": "primary-source", "files": [
            ProjectDiskFile::new("library/Core.lua").with_identity(digest(LIBRARY.as_bytes()), LIBRARY.len() as u64)
        ]}
    });
    write_json(fixture, "combined-primary.json", &config)?;
    let path = fixture.0.join("combined-primary.json");
    let input = LocalProjectInput::from_config_path(&path, stop)?;
    assert_eq!(
        input.project_bundle().configuration().selected_profile(),
        profile
    );
    assert_eq!(input.reference_view(), reference);
    assert_eq!(
        input
            .project_bundle()
            .configuration()
            .reference_generation()
            .to_string(),
        reference.generation_id()
    );
    let query = GraphBuildRequest::new("fixture-context-primary-map".into(), "current".into())?
        .with_input_profile(&input);
    Ok((path, query))
}

// Reuse context/tests.rs source_file_root: original accepted native definition,
// then exact final-generation membership. No label/path/owner-key lookup.
fn combined_primary_observation(
    store_root: &std::path::Path,
    exact: &StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<(wow_graph::GraphNodeId, String, Value, Value, Value)> {
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(&ReadSelector::Exact(exact.clone()), stop)?;
    assert_eq!(read.store_generation_id(), exact);
    let graph = read.graph();
    let source = graph
        .partition(wow_project::graph::SOURCE_GRAPH_PARTITION)
        .ok_or("original Primary source partition")?;
    let mut files = source
        .batch()
        .entity_proposals()
        .iter()
        .filter(|proposal| proposal.entity_kind_id() == "source_file");
    let proposal = files
        .next()
        .ok_or("Primary physical source-file proposal")?;
    assert!(files.next().is_none(), "one original Primary Main file");
    let accepted = source.report().accepted_entities();
    let index = accepted
        .binary_search_by(|row| row.proposal_id().cmp(proposal.proposal_id()))
        .map_err(|_| "Primary source-file proposal was not accepted")?;
    let input = accepted[index].node();
    let rebound = wow_graph::GraphNode::new(
        graph.snapshot().universe().clone(),
        graph.snapshot().generation().clone(),
        input.kind(),
        input.owner_key(),
        input.evidence_ids().to_vec(),
        graph.snapshot().limits(),
    )?;
    let node = graph
        .snapshot()
        .node(rebound.node_id())
        .ok_or("accepted Primary root absent from final graph")?;
    let result = (
        node.node_id().clone(),
        read.publication_set_id().to_owned(),
        serde_json::to_value(node)?,
        serde_json::to_value(proposal)?,
        serde_json::to_value(&accepted[index])?,
    );
    read.close()?;
    store.close()?;
    Ok(result)
}

// Explicit root-only retained stores outside the original fixture destructor.
fn combined_context_export_root(fixture: &FixtureRoot) -> TestResult<Option<PathBuf>> {
    let Some(path) = std::env::var_os("WOW_CONTEXT_COMBINED_FIXTURE_EXPORT") else {
        return Ok(None);
    };
    let path = PathBuf::from(path);
    if !path.is_absolute() || path.as_os_str().len() > 32768 {
        return Err("combined context export requires a finite absolute fresh directory".into());
    }
    let parent = path
        .parent()
        .ok_or("combined export parent")?
        .canonicalize()?;
    let path = parent.join(path.file_name().ok_or("combined export directory")?);
    if path.starts_with(fixture.0.canonicalize()?) {
        return Err("combined stores/export must survive fixture/source removal".into());
    }
    std::fs::create_dir(&path)?;
    Ok(Some(path))
}
mod combined_state_detail;
