//! Genuine selected native Source owners, with raw materialization gone.
use super::*;
use wow_graph::lineage::*;
use wow_project::graph::fingerprints::FunctionFingerprintLimits;
use wow_project::graph::lineage_input::build_source_fingerprint_partition;
use wow_project::replay::publication::{RetainedSourceGenerationPair, SourceGenerationSide};

fn input(bundle: &ProjectInputBundle) -> TestResult<crate::LocalProjectInput> {
    let packages = bundle
        .configuration()
        .platform_packages()
        .ok_or("Source packages")?;
    let reference = wow_reference::ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    Ok(crate::LocalProjectInput::new_with_package_plans(
        bundle.clone(),
        reference,
        packages.load_plan().clone(),
        packages.main_plan().clone(),
    )?)
}
#[test]
fn native_source_lineage_pair_and_fingerprints_preserve_distinct_universes() -> TestResult {
    let stop = AtomicBool::new(false);
    let retained = std::env::var_os("WOW_SOURCE_LINEAGE_PROOF_ROOT");
    let path = match &retained {
        Some(path) => {
            let path = std::path::PathBuf::from(path);
            if !path.is_absolute() || path.exists() {
                return Err("proof root must be a fresh absolute directory".into());
            }
            std::fs::create_dir(&path)?;
            path
        }
        None => root("source-lineage-native")?,
    };
    let caps = wow_project::PlatformGraphCapabilities::new(&[
        wow_project::PlatformGraphCapability::InventorySpansV1,
        wow_project::PlatformGraphCapability::StructuralRolesV1,
        wow_project::PlatformGraphCapability::XmlSourceMapsV1,
    ])?;
    let profile = PlatformGraphProfile::DirectCapabilitiesV1(caps);
    let toc = b"## Interface: 120100\ndefs.lua\n";
    let before_bytes = b"local a=function() return 1 end\nreturn a\n";
    let after_bytes =
        b"local a=function() return 1 end\nlocal b=function() return 1 end\nreturn a,b\n";
    let before = platform_bundle_with_profile_source(
        &path.join("before-input"),
        1,
        true,
        Some(profile),
        Some(&PlatformFixtureSource {
            toc,
            lua: before_bytes,
        }),
        &stop,
    )?;
    let after = platform_bundle_with_profile_source(
        &path.join("after-input"),
        2,
        true,
        Some(profile),
        Some(&PlatformFixtureSource {
            toc,
            lua: after_bytes,
        }),
        &stop,
    )?;
    let selection = PlatformStoreSelection::new(
        before.configuration().project_id().clone(),
        SOURCE_PROFILE.parse()?,
    )?
    .with_composed_candidate_catalog();
    let request = crate::graph::GraphBuildRequest::new(
        before.configuration().project_id().as_str().into(),
        "current".into(),
    )?
    .with_platform_graph_profile(profile);
    let store_path = path.join("store");
    let published = publish_input_in_namespace(
        input(&before)?,
        &request,
        &store_path,
        &selection,
        &LiveProjectPublishRequest::new("source-lineage-before", "absent", true, true)?,
        &stop,
    )?;
    assert_eq!(published.exit_code(), 2);
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    let a = store.current()?.ok_or("before Current")?;
    store.close()?;
    let published = publish_input_in_namespace(
        input(&after)?,
        &request,
        &store_path,
        &selection,
        &LiveProjectPublishRequest::new("source-lineage-after", a.record_id.as_str(), false, true)?,
        &stop,
    )?;
    assert_eq!(published.exit_code(), 2);
    drop(before);
    drop(after);
    let store = LiveProjectStore::open_in_namespace(&store_path, &selection)?;
    let b = store.current()?.ok_or("after Current")?;
    let before = store.read_in_namespace(
        &selection,
        &ReadSelector::Exact(a.generation_id.clone()),
        &stop,
    )?;
    let after = store.read_in_namespace(
        &selection,
        &ReadSelector::Exact(b.generation_id.clone()),
        &stop,
    )?;
    let left = before.candidate().ok_or("before genuine candidate")?;
    let right = after.candidate().ok_or("after genuine candidate")?;
    let wrong_namespace = PlatformStoreSelection::new(
        ProjectId::new("other-source-owner")?,
        SOURCE_PROFILE.parse()?,
    )?
    .with_composed_candidate_catalog();
    assert!(
        RetainedSourceGenerationPair::bind(
            SourceGenerationSide {
                pair: before.pair(),
                native: left.native(),
                custody: before.custody()
            },
            SourceGenerationSide {
                pair: after.pair(),
                native: right.native(),
                custody: after.custody()
            },
            wrong_namespace.namespace(),
            &stop
        )
        .is_err()
    );
    assert!(
        RetainedSourceGenerationPair::bind(
            SourceGenerationSide {
                pair: before.pair(),
                native: right.native(),
                custody: before.custody()
            },
            SourceGenerationSide {
                pair: after.pair(),
                native: right.native(),
                custody: after.custody()
            },
            selection.namespace(),
            &stop
        )
        .is_err()
    );
    assert!(
        RetainedSourceGenerationPair::bind(
            SourceGenerationSide {
                pair: before.pair(),
                native: left.native(),
                custody: after.custody()
            },
            SourceGenerationSide {
                pair: after.pair(),
                native: right.native(),
                custody: after.custody()
            },
            selection.namespace(),
            &stop
        )
        .is_err()
    );
    assert_ne!(
        before.graph().snapshot().universe(),
        after.graph().snapshot().universe()
    );
    let pair = RetainedSourceGenerationPair::bind(
        SourceGenerationSide {
            pair: before.pair(),
            native: left.native(),
            custody: before.custody(),
        },
        SourceGenerationSide {
            pair: after.pair(),
            native: right.native(),
            custody: after.custody(),
        },
        selection.namespace(),
        &stop,
    )?;
    let limits = LineageLimits {
        max_entities: 64,
        max_bucket_members: 32,
        max_pairs: 128,
        max_component_nodes: 64,
        max_component_edges: 128,
        max_work: 128_000_000,
        max_metadata_bytes: 32 * 1024 * 1024,
    };
    let fp = FunctionFingerprintLimits::new(64, 8 * 1024 * 1024, 32 * 1024 * 1024)?;
    let partition = build_source_fingerprint_partition(&pair, fp, limits, &stop)?;
    assert_eq!(
        partition.data().binding.universe_class,
        LineageUniverseClass::BlizzardUiSource
    );
    assert_eq!(
        partition.data().binding.profile_id.as_ref(),
        SOURCE_SPAN_LINEAGE_PROFILE
    );
    let metadata = partition
        .data()
        .binding
        .source
        .as_ref()
        .ok_or("Source provenance")?;
    assert_ne!(
        metadata.before.source_snapshot_id,
        metadata.after.source_snapshot_id
    );
    assert_eq!(
        metadata.before.admission_coverage.inventory,
        wow_core::CoverageStatus::Partial
    );
    assert!(metadata.before.omissions.excluded > 0);
    let left_graph = before.pair().graph_read(&stop)?;
    let right_graph = after.pair().graph_read(&stop)?;
    let mut wrong = partition.data().clone();
    wrong
        .binding
        .source
        .as_mut()
        .ok_or("Source metadata")?
        .after
        .channel = "other-channel".into();
    assert!(
        validate_source_fingerprint_partition(wrong, &left_graph, &right_graph, limits, &stop)
            .is_err()
    );
    let mut wrong = partition.data().clone();
    wrong
        .binding
        .source
        .as_mut()
        .ok_or("Source metadata")?
        .after
        .source_class = LineageSourceClass::VendorUiSourceMirror;
    assert!(
        validate_source_fingerprint_partition(wrong, &left_graph, &right_graph, limits, &stop)
            .is_err()
    );
    let mut wrong = partition.data().clone();
    wrong.binding.profile_id = PROJECT_SPAN_LINEAGE_PROFILE.into();
    assert!(
        validate_source_fingerprint_partition(wrong, &left_graph, &right_graph, limits, &stop)
            .is_err()
    );
    let mut wrong = partition.data().clone();
    wrong.binding.after.analyzer_pin_id = "different-analyzer-pin".into();
    assert!(
        validate_source_fingerprint_partition(wrong, &left_graph, &right_graph, limits, &stop)
            .is_err()
    );
    let mut wrong = partition.data().clone();
    let mut different_flavor = serde_json::to_value(&wrong.binding.after.selected_profile)?;
    different_flavor["flavor_id"] = serde_json::json!("classic");
    let different_flavor = serde_json::from_value::<wow_core::ProfileIdentity>(different_flavor)?;
    wrong.binding.after.selected_profile = different_flavor.clone();
    wrong
        .binding
        .source
        .as_mut()
        .ok_or("Source metadata")?
        .after
        .target_reference_profile = different_flavor;
    assert!(
        validate_source_fingerprint_partition(wrong, &left_graph, &right_graph, limits, &stop)
            .is_err()
    );
    assert_eq!(
        build_source_fingerprint_partition(
            &pair,
            fp,
            LineageLimits {
                max_work: 1,
                ..limits
            },
            &stop
        )
        .err()
        .map(|e| e.code()),
        Some(wow_project::ProjectErrorCode::SourceBudgetExceeded)
    );
    let candidates = generate_lineage_candidate_components(&partition, &stop)?;
    assert!(
        candidates
            .components
            .iter()
            .any(|c| c.shape == LineageComponentShape::OneToMany && c.ambiguous)
    );
    assert!(
        candidates
            .proposals
            .iter()
            .all(|p| p.confidence == wow_graph::GraphConfidence::Candidate)
    );
    let prepared = PreparedLineageSnapshot::build(partition, &stop)?;
    assert_eq!(prepared.candidates().accepted_assertion_count, 0);
    let record = wow_core::canonical_json_bytes(prepared.input())?;
    assert!(!String::from_utf8_lossy(&record).contains("local a=function()"));
    assert_eq!(
        build_source_fingerprint_partition(&pair, fp, limits, &AtomicBool::new(true))
            .err()
            .map(|e| e.code()),
        Some(wow_project::ProjectErrorCode::AnalysisCancelled)
    );
    drop(pair);
    before.close()?;
    after.close()?;
    store.close()?;
    source_service_roundtrip(&path, &store_path, &selection, &a, &b, limits, &stop)?;
    if retained.is_none() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn source_service_roundtrip(
    path: &Path,
    store_path: &Path,
    selection: &PlatformStoreSelection,
    a: &wow_store::project::CurrentPublication,
    b: &wow_store::project::CurrentPublication,
    limits: LineageLimits,
    stop: &AtomicBool,
) -> TestResult {
    use crate::lineage::{
        LineageBuildRequest, LineageCandidateQuery, LineageExplanationTarget, LineageQueryLimits,
        LineageQueryRequest, LineageReadRequest, LineageStoreLimits, build_lineage_snapshot,
        compare_lineage_snapshot, query_lineage_snapshot, retire_lineage_snapshot,
    };
    let document = serde_json::json!({"schema":"wow-service/platform-project-selection/1","owner_project_id":selection.namespace().owner_project_id(),
        "source_profile_id":SOURCE_PROFILE,"catalogs":{"xml_schema_classification":false,"physical_function_declarations":"absent","physical_function_signatures":"absent","composed_candidate":true}});
    let selected = crate::live_project::PlatformProjectSelection::from_json_slice(
        &serde_json::to_vec(&document)?,
        stop,
    )?;
    let build = LineageBuildRequest {
        schema: "wow-service/source-lineage-build/1".into(),
        operation_id: "source-retained-lineage-build".into(),
        before_generation: a.generation_id.as_str().into(),
        after_generation: b.generation_id.as_str().into(),
        limits,
        max_source_bytes: 8 * 1024 * 1024,
        store_limits: LineageStoreLimits::default(),
        max_output_bytes: 8 * 1024 * 1024,
        source_selection: Some(selected),
        reference_selection: None,
    };
    let lineage_path = path.join("lineage.sqlite");
    // Explicit test transport artifact, never native authority or a DB copy.
    let fixture = serde_json::json!({"source_store_root":store_path.to_str().ok_or("fixture path encoding")?,"build_request":&build});
    std::fs::write(
        path.join("source-fixture.json"),
        serde_json::to_vec_pretty(&fixture)?,
    )?;
    let result = build_lineage_snapshot(store_path, &lineage_path, &build, stop)?;
    let held = LiveProjectStore::open_in_namespace(store_path, selection)?;
    let roots = held.retained_owner().retention_roots(stop)?;
    assert_eq!(roots.len(), 2);
    assert_ne!(roots[0].root_id(), roots[1].root_id());
    held.close()?;
    let receipt: serde_json::Value = serde_json::from_slice(&result)?;
    assert_eq!(
        receipt["schema"],
        "wow-service/source-lineage-build-result/1"
    );
    let digest: wow_store::RequestDigest =
        serde_json::from_value(receipt["request_digest"].clone())?;
    let snapshot_id: Box<str> = receipt["snapshot"]["snapshot_id"]
        .as_str()
        .ok_or("Source snapshot ID")?
        .into();
    let read = LineageReadRequest {
        schema: "wow-service/source-lineage-read/1".into(),
        build_operation_id: build.operation_id.clone(),
        build_request_digest: digest.clone(),
        snapshot_id: snapshot_id.clone(),
        store_limits: build.store_limits,
        max_output_bytes: build.max_output_bytes,
        source_selection: build.source_selection.clone(),
        reference_selection: None,
    };
    let saved = compare_lineage_snapshot(store_path, &lineage_path, &read, stop)?;
    let data: serde_json::Value = serde_json::from_slice(&saved)?;
    assert_eq!(data["schema"], SOURCE_LINEAGE_SNAPSHOT_SCHEMA);
    assert_eq!(
        build_lineage_snapshot(store_path, &lineage_path, &build, stop)?,
        result
    );
    let proposal = data["candidates"]["proposals"]
        .as_array()
        .ok_or("Source proposals")?
        .first()
        .ok_or("Source candidate pair")?;
    let query = LineageQueryRequest {
        schema: "wow-service/source-lineage-query/1".into(),
        operation_id: "source-retained-lineage-explain".into(),
        build_operation_id: build.operation_id.clone(),
        build_request_digest: digest,
        snapshot_id,
        store_limits: build.store_limits,
        query: LineageCandidateQuery::Explain {
            target: LineageExplanationTarget::Proposal {
                id: proposal["proposal_id"]
                    .as_str()
                    .ok_or("proposal ID")?
                    .into(),
            },
        },
        limits: LineageQueryLimits {
            max_work: 128_000_000,
            max_metadata_bytes: 32 * 1024 * 1024,
            max_output_bytes: 8 * 1024 * 1024,
            max_components: 64,
            max_entities: 64,
            max_proposals: 128,
        },
        lease_expires_after: 10,
        source_selection: build.source_selection.clone(),
        reference_selection: None,
    };
    let observation = query_lineage_snapshot(store_path, &lineage_path, &query, stop)?;
    let observation_data: serde_json::Value = serde_json::from_slice(&observation)?;
    assert_eq!(
        observation_data["result"]["selected_validation"],
        "native_blizzard_ui_source_span_candidates_valid"
    );
    assert_eq!(observation_data["result"]["accepted_assertion_count"], 0);
    retire_lineage_snapshot(
        store_path,
        &lineage_path,
        &read,
        "source-retained-lineage-retire",
        stop,
    )?;
    let held = LiveProjectStore::open_in_namespace(store_path, selection)?;
    assert!(held.retained_owner().retention_roots(stop)?.is_empty());
    held.close()?;
    assert_eq!(
        query_lineage_snapshot(store_path, &lineage_path, &query, stop)?,
        observation
    );
    let mut fresh = query.clone();
    fresh.operation_id = "source-retired-fresh-explain".into();
    assert!(query_lineage_snapshot(store_path, &lineage_path, &fresh, stop).is_err());
    assert!(compare_lineage_snapshot(store_path, &lineage_path, &read, stop).is_err());
    Ok(())
}
