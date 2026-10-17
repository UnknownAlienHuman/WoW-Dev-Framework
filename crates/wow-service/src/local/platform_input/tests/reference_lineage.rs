use super::*;
use crate::lineage::*;
use crate::live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project};
use std::path::Path;
use wow_store::project::ReadSelector;

fn refresh_manifest(root: &FixtureRoot, config: &mut Value) -> TestResult {
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    for file in files.iter_mut() {
        let path = file["path"].as_str().ok_or("manifest path")?;
        let bytes = std::fs::read(root.0.join(path))?;
        file["bytes"] = json!(bytes.len());
        file["content_sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    }
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("file bytes")?)
                .ok_or("byte sum")?)
        })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("manifest")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    let file = write_json(root, "manifest.json", &manifest)?;
    config["native_reference"]["native_source"]["manifest"] = serde_json::to_value(file)?;
    Ok(())
}
fn native_config(config: &Value) -> Value {
    json!({"schema":crate::local::LOCAL_NATIVE_SCHEMA,"project_id":config["project_id"],"workspace_id":config["workspace_id"],"source_origin_id":config["source_origin_id"],"logical_root":config["logical_root"],"profile":config["native_reference"]["profile"],"analyzer":config["analyzer"],"main":{"root":"UI","files":[ProjectDiskFile::new("main.lua")]},"native_source":config["native_reference"]["native_source"],"native_api_uses":"reference_v1"})
}
fn guard(read: &crate::live_project::LiveProjectRead) -> TestResult<ReferenceLineageGuard> {
    let library = read
        .native_api()
        .ok_or("native API proof")?
        .original_reference_library();
    assert!(library.reference_target().is_some());
    Ok(ReferenceLineageGuard {
        profile: library.profile().clone(),
        reference_generation: library.reference().view.generation_id().parse()?,
        view_digest: library.reference().view.self_digest().parse()?,
    })
}
fn held(root: &Path, generation: &str, expected: usize, stop: &AtomicBool) -> TestResult {
    let store = LiveProjectStore::open(root)?;
    let roots = store.retained_owner().retention_roots(stop)?;
    let selected = roots
        .iter()
        .filter(|root| root.held_by() == "wow-graph.lineage-reference-callable-key")
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), expected);
    assert!(
        selected
            .iter()
            .all(|root| root.generation_id().as_str() == generation)
    );
    store.close()?;
    Ok(())
}

#[test]
fn native_reference_lineage_retains_actual_profiles_graph_support_and_lifecycle() -> TestResult {
    let stop = AtomicBool::new(false);
    let (root, mut config) = native_fixture()?;
    config["native_reference"]["profile"]["reference_target"] = json!({"schema":"wow-service/native-reference-target/1","product":"fixture","channel":"fixture"});
    std::fs::write(
        root.0.join("UI/main.lua"),
        "function FixtureFunction() return C_Transport.Read(false) end\n",
    )?;
    refresh_manifest(&root, &mut config)?;
    write_json(
        &root,
        "reference-lineage-native.json",
        &native_config(&config),
    )?;
    let export = std::env::var_os("WOW_REFERENCE_LINEAGE_PROOF_ROOT");
    let parent_path = export
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| root.0.with_extension("reference-lineage-store"));
    if !parent_path.is_absolute() || parent_path.as_os_str().len() > 32768 {
        return Err("finite fresh Reference proof root required".into());
    }
    std::fs::create_dir(&parent_path)?;
    let parent = FixtureRoot(parent_path);
    let before_path = parent.0.join("project-before");
    let after_path = parent.0.join("project-after");
    let graph = crate::graph::GraphBuildRequest::new(
        "fixture-platform-transport".into(),
        "current".into(),
    )?;
    eprintln!("reference-lineage stage=before-publication");
    publish_local_project(
        &root.0.join("reference-lineage-native.json"),
        &graph,
        &before_path,
        &LiveProjectPublishRequest::new("reference-before", "absent", true, true)?,
        &stop,
    )?;
    eprintln!("reference-lineage stage=before-current-read");
    let store = LiveProjectStore::open(&before_path)?;
    let a = store.current()?.ok_or("before Current")?;
    store.close()?;
    let api_path = root.0.join("Blizzard_APIDocumentationGenerated/API.lua");
    let api = std::fs::read_to_string(&api_path)?;
    eprintln!("reference-lineage stage=after-input");
    let changed = api.replace(
        "Type=\"number\",Nilable=true",
        "Type=\"string\",Nilable=false",
    );
    assert_ne!(api, changed);
    std::fs::write(&api_path, &changed)?;
    refresh_manifest(&root, &mut config)?;
    write_json(
        &root,
        "reference-lineage-native.json",
        &native_config(&config),
    )?;
    publish_local_project(
        &root.0.join("reference-lineage-native.json"),
        &graph,
        &after_path,
        &LiveProjectPublishRequest::new("reference-after", "absent", true, true)?,
        &stop,
    )?;
    let store = LiveProjectStore::open(&after_path)?;
    eprintln!("reference-lineage stage=after-current-read");
    let b = store.current()?.ok_or("after Current")?;
    eprintln!("reference-lineage stage=before-exact-read");
    let before_store = LiveProjectStore::open(&before_path)?;
    let before = before_store.read(&ReadSelector::Exact(a.generation_id.clone()), &stop)?;
    eprintln!("reference-lineage stage=after-exact-read");
    let after = store.read(&ReadSelector::Exact(b.generation_id.clone()), &stop)?;
    let left = guard(&before)?;
    let right = guard(&after)?;
    assert_ne!(left.profile, right.profile);
    assert_ne!(left.reference_generation, right.reference_generation);
    after.close()?;
    before.close()?;
    before_store.close()?;
    store.close()?;
    let selection = ReferenceLineageSelection {
        schema: "wow-service/reference-lineage-selection/1".into(),
        comparison_profile: wow_reference::native_view::comparison::CALLABLE_COMPARISON_PROFILE
            .into(),
        privacy: "local_metadata_only".into(),
        before: left,
        after: right,
        keys: vec![
            "function:C_Transport.Read".into(),
            "function:C_Transport.Missing".into(),
        ],
        limits: wow_reference::native_view::comparison::CallableComparisonLimits {
            max_keys: 16,
            max_view_bytes: 4 * 1024 * 1024,
            max_steps: 16 * 1024 * 1024,
        },
    };
    let request = LineageBuildRequest {
        schema: "wow-service/reference-lineage-build/1".into(),
        operation_id: "reference-candidate-build".into(),
        before_generation: a.generation_id.as_str().into(),
        after_generation: b.generation_id.as_str().into(),
        limits: LineageLimits {
            max_entities: 32,
            max_bucket_members: 16,
            max_pairs: 64,
            max_component_nodes: 32,
            max_component_edges: 64,
            max_work: 64_000_000,
            max_metadata_bytes: 16 * 1024 * 1024,
        },
        max_source_bytes: 4 * 1024 * 1024,
        store_limits: LineageStoreLimits::default(),
        max_output_bytes: 16 * 1024 * 1024,
        source_selection: None,
        reference_selection: Some(selection),
    };
    // Original raw directories are gone; only retained native input can qualify.
    drop(root);
    let lineage = parent.0.join("lineage.sqlite");
    let roots = ReferenceLineageRoots {
        before: &before_path,
        after: &after_path,
    };
    eprintln!("reference-lineage stage=candidate-build");
    let built = build_reference_lineage_snapshot(roots, &lineage, &request, &stop)?;
    let receipt: Value = serde_json::from_slice(&built)?;
    assert_eq!(
        receipt["schema"],
        "wow-service/reference-lineage-build-result/1"
    );
    assert_eq!(receipt["accepted_assertion_count"], 0);
    assert_eq!(receipt["negative_authority"], false);
    held(&before_path, a.generation_id.as_str(), 1, &stop)?;
    held(&after_path, b.generation_id.as_str(), 1, &stop)?;
    assert_eq!(
        receipt["reference_comparison"]["observations"][1]["comparison"]["state"],
        "selected_fields_differ"
    );
    assert_eq!(
        built,
        build_reference_lineage_snapshot(roots, &lineage, &request, &stop)?
    );
    let read = LineageReadRequest {
        schema: "wow-service/reference-lineage-read/1".into(),
        build_operation_id: request.operation_id.clone(),
        build_request_digest: serde_json::from_value(receipt["request_digest"].clone())?,
        snapshot_id: receipt["snapshot"]["snapshot_id"]
            .as_str()
            .ok_or("snapshot id")?
            .into(),
        store_limits: request.store_limits,
        max_output_bytes: request.max_output_bytes,
        source_selection: None,
        reference_selection: request.reference_selection.clone(),
    };
    let snapshot = compare_reference_lineage_snapshot(roots, &lineage, &read, &stop)?;
    assert!(
        compare_reference_lineage_snapshot(
            ReferenceLineageRoots {
                before: &after_path,
                after: &before_path
            },
            &lineage,
            &read,
            &stop
        )
        .is_err()
    );
    let value: Value = serde_json::from_slice(&snapshot)?;
    assert_eq!(
        value["input"]["binding"]["universe_class"],
        "reference_platform_contract"
    );
    assert_eq!(
        value["input"]["reference_records"]
            .as_array()
            .ok_or("actual Reference records")?
            .len(),
        2
    );
    assert_eq!(
        value["input"]["reference_omissions"]
            .as_array()
            .ok_or("missing selections")?
            .len(),
        2
    );
    assert_eq!(
        value["candidates"]["proposals"]
            .as_array()
            .ok_or("actual proposal")?
            .len(),
        1
    );
    let proposal = &value["candidates"]["proposals"][0];
    assert!(proposal["before"]["reference_entity"].is_object());
    assert_ne!(
        proposal["before"]["reference_entity"]["reference_generation"],
        proposal["after"]["reference_entity"]["reference_generation"]
    );
    let mut query = LineageQueryRequest {
        schema: "wow-service/reference-lineage-query/1".into(),
        operation_id: "reference-query-explain".into(),
        build_operation_id: request.operation_id.clone(),
        build_request_digest: read.build_request_digest.clone(),
        snapshot_id: read.snapshot_id.clone(),
        store_limits: request.store_limits,
        query: LineageCandidateQuery::Explain {
            target: LineageExplanationTarget::Proposal {
                id: proposal["proposal_id"]
                    .as_str()
                    .ok_or("proposal id")?
                    .into(),
            },
        },
        limits: LineageQueryLimits {
            max_work: 128_000_000,
            max_metadata_bytes: 32 * 1024 * 1024,
            max_output_bytes: 16 * 1024 * 1024,
            max_components: 32,
            max_entities: 32,
            max_proposals: 64,
        },
        lease_expires_after: 10,
        source_selection: None,
        reference_selection: request.reference_selection.clone(),
    };
    let explanation = query_reference_lineage_snapshot(roots, &lineage, &query, &stop)?;
    let explained: Value = serde_json::from_slice(&explanation)?;
    assert_eq!(
        explained["result"]["closure"]["reference_callables"]
            .as_array()
            .ok_or("Reference closure")?
            .len(),
        2
    );
    assert_eq!(
        explained["result"]["closure"]["native_reference_support"]
            .as_array()
            .ok_or("native accepted input support")?
            .len(),
        2
    );
    assert_eq!(explained["result"]["full_e4_b_validation"], "not_evaluated");
    let mut wrong = request.clone();
    wrong.operation_id = "reference-wrong-guard".into();
    wrong
        .reference_selection
        .as_mut()
        .ok_or("selection")?
        .before
        .view_digest = ContentDigest::from_bytes([0; 32]);
    assert!(build_reference_lineage_snapshot(roots, &lineage, &wrong, &stop).is_err());
    held(&before_path, a.generation_id.as_str(), 2, &stop)?;
    held(&after_path, b.generation_id.as_str(), 2, &stop)?;
    abandon_reference_lineage_snapshot(
        roots,
        &lineage,
        &wrong,
        "reference-wrong-abandon",
        request.max_output_bytes,
        &stop,
    )?;
    held(&before_path, a.generation_id.as_str(), 1, &stop)?;
    held(&after_path, b.generation_id.as_str(), 1, &stop)?;
    let retired =
        retire_reference_lineage_snapshot(roots, &lineage, &read, "reference-retire", &stop)?;
    assert_eq!(
        retired,
        retire_reference_lineage_snapshot(roots, &lineage, &read, "reference-retire", &stop)?
    );
    held(&before_path, a.generation_id.as_str(), 0, &stop)?;
    held(&after_path, b.generation_id.as_str(), 0, &stop)?;
    assert_eq!(
        explanation,
        query_reference_lineage_snapshot(roots, &lineage, &query, &stop)?
    );
    query.operation_id = "reference-fresh-after-retirement".into();
    assert!(query_reference_lineage_snapshot(roots, &lineage, &query, &stop).is_err());
    if export.is_some() {
        std::fs::write(
            parent.0.join("fixture.json"),
            serde_json::to_vec(
                &json!({"project_store_before":before_path,"project_store_after":after_path,"build_request":request,"read_request":read,"query_request":query,"retired":true}),
            )?,
        )?;
        std::mem::forget(parent);
    }
    Ok(())
}
