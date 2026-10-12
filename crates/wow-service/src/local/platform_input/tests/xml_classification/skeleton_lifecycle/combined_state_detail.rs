use super::*;
use std::io::{Read, Write};
use std::time::Instant;
use wow_core::ProfileIdentity;
use wow_project::{
    disk::ProjectDiskFile,
    graph::SOURCE_GRAPH_PARTITION,
    load::{TocLoadContext, TocSavedVariableScope, TocSavedVariableState},
};
use wow_store::project::{ReadSelector, StoreGenerationId};

const STATE_TOC: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/state-core-pipeline/StateCore.toc"
));
const STATE_LUA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/data/state-core-pipeline/state.lua"
));
const STATE_DETAIL_L1_PROFILE: &str = wow_context::PRIMARY_STATE_DETAIL_L1_PROFILE;
const LIBRARY: &str = "---@meta\n---@return number\nfunction External() end\n";

fn read_bounded(path: &std::path::Path, limit: usize) -> TestResult<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(u64::try_from(limit)?.checked_add(1).ok_or("read bound")?)
        .read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err("retained fixture exceeds its byte bound".into());
    }
    Ok(bytes)
}

fn write_new(path: &std::path::Path, bytes: &[u8]) -> TestResult {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    file.write_all(bytes)?;
    Ok(())
}

fn success_envelope(result: &crate::context::ContextMapResult) -> TestResult<Value> {
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    Ok(envelope)
}

fn refusal(result: crate::ServiceResult<crate::context::ContextMapResult>) -> TestResult {
    let result = result?;
    assert_eq!(
        result.failure_code(),
        Some(ServiceErrorCode::BudgetExceeded)
    );
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("payload").is_none());
    Ok(())
}

fn assert_original_entity_support(origins: &Value, expected: &Value, native: &Value) -> TestResult {
    let node = &expected["node"];
    let receipts = origins["supports"]
        .as_array()
        .ok_or("native support receipts")?;
    let receipt = receipts
        .iter()
        .find(|row| {
            row["explanation"]["record"]["kind"] == "entity"
                && row["explanation"]["record"]["node"] == *node
        })
        .ok_or("exact accepted native entity support receipt")?;
    let explanation = &receipt["explanation"];
    assert_eq!(explanation["support_complete"], true);
    assert_eq!(
        explanation["source_context_id"],
        native["source_context_id"]
    );
    assert!(
        explanation["supports"]
            .as_array()
            .ok_or("original producer supports")?
            .iter()
            .any(|support| {
                support["producer"]["partition_id"] == SOURCE_GRAPH_PARTITION
                    && support["proposal"] == expected["proposal"]
                    && support["accepted"] == expected["accepted"]
            })
    );
    let evidence = &receipt["evidence_resolution"];
    assert_eq!(evidence["closure_complete"], true);
    assert!(
        evidence["unresolved"]
            .as_array()
            .ok_or("unresolved support")?
            .is_empty()
    );
    let handles = evidence["source_handles"]
        .as_array()
        .ok_or("resolved source handles")?;
    let records = evidence["evidence_records"]
        .as_array()
        .ok_or("resolved evidence records")?;
    for id in expected["origin"]["source_handle_ids"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let original = native["source_handles"]
            .as_array()
            .ok_or("original source handles")?
            .iter()
            .find(|row| row["id"] == *id)
            .ok_or("original handle record")?;
        assert!(handles.contains(&original["record"]));
    }
    for id in expected["node"]["evidence_ids"]
        .as_array()
        .ok_or("native evidence IDs")?
    {
        let original = native["source_evidence"]
            .as_array()
            .ok_or("original evidence records")?
            .iter()
            .find(|row| row["id"] == *id)
            .ok_or("original evidence record")?;
        assert!(records.contains(&original["record"]));
    }
    Ok(())
}

// This is intentionally ignored and requires an existing source-absent combined
// platform fixture plus a fresh export root. It publishes a new ordinary Primary
// TOC project, then exercises only the consuming combined service route.
#[test]
#[ignore = "requires retained source-absent platform fixture and fresh StateDetail export root"]
fn retained_combined_state_detail_l1_source_absent() -> TestResult {
    let stop = AtomicBool::new(false);
    let retained = PathBuf::from(
        std::env::var_os("WOW_CONTEXT_COMBINED_RETAINED_FIXTURE")
            .ok_or("explicit retained combined fixture required")?,
    );
    let export = PathBuf::from(
        std::env::var_os("WOW_CONTEXT_COMBINED_STATE_EXPORT")
            .ok_or("explicit fresh combined StateDetail export required")?,
    );
    if !retained.is_absolute() || !export.is_absolute() || !retained.is_dir() {
        return Err(
            "retained fixture and export paths must be absolute; fixture must exist".into(),
        );
    }
    if std::env::var("WOW_CONTEXT_COMBINED_STATE_RESUME").as_deref() == Ok("1") {
        return verify_retained_state_detail(&retained, &export, None, None, &stop);
    }
    if export.exists() {
        return Err("StateDetail export path must be fresh; refusing to reuse it".into());
    }
    let export_parent = export.parent().ok_or("StateDetail export parent")?;
    if !export_parent.is_dir() {
        return Err("StateDetail export parent must already exist".into());
    }

    let retained_host: Value = serde_json::from_slice(&read_bounded(
        &retained.join("host.json"),
        CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
    )?)?;
    let mut build_wire: Value = serde_json::from_slice(&read_bounded(
        &retained.join("build-request.json"),
        CONTEXT_COMBINED_REQUEST_MAX_BYTES,
    )?)?;
    assert_eq!(
        retained_host["primary"]["limits"]["max_context_work"],
        16_000_000
    );
    assert_eq!(
        retained_host["primary"]["limits"]["max_metadata_bytes"],
        16 * 1024 * 1024
    );
    assert_eq!(
        retained_host["primary"]["limits"]["max_input_bytes"],
        8_000_000
    );
    assert_eq!(retained_host["primary"]["limits"]["max_direct_edges"], 8);
    assert_eq!(
        retained_host["platform"]["limits"],
        retained_host["primary"]["limits"]
    );

    // fixture() owns the exact deterministic transport profile, Reference and
    // analyzer artifacts already represented by the retained platform host.
    let (fixture_root, platform_config) = fixture()?;
    let reference_profile: ProfileIdentity =
        serde_json::from_value(retained_host["primary"]["reference_profile"].clone())?;
    let reference: wow_reference::ReferenceView =
        serde_json::from_value(retained_host["primary"]["reference"].clone())?;
    let fixture_profile: ProfileIdentity = serde_json::from_value(
        platform_config["source_profile"]["target"]["reference_profile"].clone(),
    )?;
    let fixture_reference_path = platform_config["reference_view"]["path"]
        .as_str()
        .ok_or("fixture Reference artifact path")?;
    let fixture_reference: wow_reference::ReferenceView =
        serde_json::from_slice(&std::fs::read(fixture_root.0.join(fixture_reference_path))?)?;
    assert_eq!(
        fixture_profile, reference_profile,
        "full typed ProfileIdentity match"
    );
    assert_eq!(
        fixture_reference, reference,
        "full typed ReferenceView match"
    );

    let source_root = fixture_root.0.join("primary-state-source");
    std::fs::create_dir(&source_root)?;
    std::fs::write(source_root.join("StateCore.toc"), STATE_TOC)?;
    std::fs::write(source_root.join("state.lua"), STATE_LUA)?;
    std::fs::write(source_root.join("library.lua"), LIBRARY)?;
    let profile_file = write_json(
        &fixture_root,
        "state-detail-profile.json",
        &reference_profile,
    )?;
    let load_context = TocLoadContext {
        game_types: std::collections::BTreeMap::from([("retail".into(), true)]),
        family: None,
        game: None,
        text_locale: None,
        location: None,
        environment: None,
    };
    let primary_config = json!({
        "schema": crate::local::LOCAL_TOC_SCHEMA,
        "project_id": "fixture-context-primary-map",
        "workspace_id": "workspace:fixture-context-primary-map",
        "source_origin_id": "project-origin:fixture-context-primary-map",
        "logical_root": "fixtures/context-primary/state-detail",
        "profile": profile_file,
        "reference_view": platform_config["reference_view"],
        "analyzer": platform_config["analyzer"],
        "main": {
            "root": "primary-state-source",
            "toc": ProjectDiskFile::new("StateCore.toc"),
            "load_context": load_context
        },
        "library": {"root": "primary-state-source", "files": [
            ProjectDiskFile::new("library.lua").with_identity(digest(LIBRARY.as_bytes()), LIBRARY.len() as u64)
        ]}
    });
    let primary_config_path = fixture_root.0.join("state-detail-primary.json");
    write_json(&fixture_root, "state-detail-primary.json", &primary_config)?;
    let primary_input = LocalProjectInput::from_config_path(&primary_config_path, &stop)?;
    assert_eq!(
        primary_input
            .project_bundle()
            .configuration()
            .selected_profile(),
        &reference_profile
    );
    assert_eq!(primary_input.reference_view(), &reference);
    assert_eq!(
        primary_input
            .project_bundle()
            .configuration()
            .reference_generation()
            .to_string(),
        reference.generation_id()
    );
    let primary_query =
        GraphBuildRequest::new("fixture-context-primary-map".into(), "current".into())?
            .with_input_profile(&primary_input);
    drop(primary_input);

    std::fs::create_dir(&export)?;
    let primary_store_root = export.join("registered-primary-store");
    let publication = serde_json::to_value(publish_local_project(
        &primary_config_path,
        &primary_query,
        &primary_store_root,
        &LiveProjectPublishRequest::new("fixture:combined-state-primary", "absent", true, true)?,
        &stop,
    )?)?;
    assert_eq!(publication["status"], "activated");
    let generation = StoreGenerationId::parse(
        publication["operation"]["generation_id"]
            .as_str()
            .ok_or("new Primary generation")?,
    )?;
    // Capture expectations from the accepted native source partition, never
    // from the exported CLI documents. Keep all original TOC rows and supports.
    let store = LiveProjectStore::open(&primary_store_root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), &stop)?;
    let native_expected = {
        let graph = read.graph();
        let pair = read.pair();
        let graph_read = pair.graph_read(&stop)?;
        let provenance = pair.source_provenance().ok_or("native source provenance")?;
        let account = provenance
            .state_roots()
            .iter()
            .find(|row| row.scope == TocSavedVariableScope::Account)
            .ok_or("accepted account StateCore root")?;
        assert_eq!(account.name, "StateCoreAccountDB");
        assert!(!account.ambiguous);
        let root_node =
            graph_read.published_entity(SOURCE_GRAPH_PARTITION, &account.proposal_id, &stop)?;
        assert_eq!(root_node.kind(), "state_root");
        let account_declarations = provenance
            .state_declarations()
            .iter()
            .filter(|row| {
                row.state == TocSavedVariableState::Declared
                    && row.name == account.name
                    && row.scope == account.scope
                    && account.source_handle_ids.contains(&row.source_handle_id)
                    && account.evidence_ids.contains(&row.evidence_id)
            })
            .collect::<Vec<_>>();
        assert!(!account_declarations.is_empty());
        let source_partition = graph
            .partition(SOURCE_GRAPH_PARTITION)
            .ok_or("native source graph partition")?;
        let root_proposal = source_partition
            .batch()
            .entity_proposals()
            .iter()
            .find(|row| row.proposal_id() == account.proposal_id)
            .ok_or("original account root proposal")?;
        let root_accepted = source_partition
            .report()
            .accepted_entities()
            .iter()
            .find(|row| row.proposal_id() == account.proposal_id)
            .ok_or("accepted account root proposal")?;
        let path_origin = provenance
            .state_paths()
            .iter()
            .find(|row| {
                row.root_id == account.root_id
                    && row.keys.as_slice()
                        == [wow_emmy::global_access::GlobalAccessKey::String(
                            "settings".into(),
                        )]
            })
            .ok_or("accepted account StateCore path")?;
        let path_node =
            graph_read.published_entity(SOURCE_GRAPH_PARTITION, &path_origin.proposal_id, &stop)?;
        assert_eq!(path_node.kind(), "state_path");
        let path_proposal = source_partition
            .batch()
            .entity_proposals()
            .iter()
            .find(|row| row.proposal_id() == path_origin.proposal_id)
            .ok_or("original account path proposal")?;
        let path_accepted = source_partition
            .report()
            .accepted_entities()
            .iter()
            .find(|row| row.proposal_id() == path_origin.proposal_id)
            .ok_or("accepted account path proposal")?;
        let edge_rows = graph
            .snapshot()
            .edges()
            .iter()
            .filter(|edge| {
                edge.from() == root_node.node_id()
                    && edge.to() == path_node.node_id()
                    && edge.relation() == wow_graph::GraphRelationKind::Owns
            })
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?;
        assert!(!edge_rows.is_empty(), "native root/path ownership edge");
        let source_file = provenance
            .files()
            .iter()
            .find(|row| row.path == "state.lua")
            .ok_or("unrelated accepted state.lua source file")?;
        let source_file_node =
            graph_read.published_entity(SOURCE_GRAPH_PARTITION, &source_file.proposal_id, &stop)?;
        assert_eq!(source_file_node.kind(), "source_file");
        let source_evidence = provenance
            .evidence()
            .iter()
            .map(|(id, row)| json!({"id": id, "record": row}))
            .collect::<Vec<_>>();
        let source_handles = provenance
            .source_handles()
            .iter()
            .map(|(id, row)| json!({"id": id, "record": row}))
            .collect::<Vec<_>>();
        let mut all_state_roots = Vec::new();
        for origin in provenance.state_roots() {
            let node =
                graph_read.published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, &stop)?;
            let declarations = provenance
                .state_declarations()
                .iter()
                .filter(|row| {
                    row.state == TocSavedVariableState::Declared
                        && row.name == origin.name
                        && row.scope == origin.scope
                        && origin.source_handle_ids.contains(&row.source_handle_id)
                        && origin.evidence_ids.contains(&row.evidence_id)
                })
                .collect::<Vec<_>>();
            let proposal = source_partition
                .batch()
                .entity_proposals()
                .iter()
                .find(|row| row.proposal_id() == origin.proposal_id)
                .ok_or("original native state-root proposal")?;
            let accepted = source_partition
                .report()
                .accepted_entities()
                .iter()
                .find(|row| row.proposal_id() == origin.proposal_id)
                .ok_or("accepted native state-root proposal")?;
            all_state_roots.push(json!({
                "node": node,
                "origin": origin,
                "declarations": declarations,
                "proposal": proposal,
                "accepted": accepted
            }));
        }
        let mut all_state_paths = Vec::new();
        for origin in provenance.state_paths() {
            let node =
                graph_read.published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, &stop)?;
            let proposal = source_partition
                .batch()
                .entity_proposals()
                .iter()
                .find(|row| row.proposal_id() == origin.proposal_id)
                .ok_or("original native state-path proposal")?;
            let accepted = source_partition
                .report()
                .accepted_entities()
                .iter()
                .find(|row| row.proposal_id() == origin.proposal_id)
                .ok_or("accepted native state-path proposal")?;
            all_state_paths.push(json!({
                "node": node,
                "origin": origin,
                "proposal": proposal,
                "accepted": accepted
            }));
        }
        json!({
            "source_context_id": graph.source_context_id(),
            "publication_set_id": read.publication_set_id(),
            "generation_id": read.store_generation_id(),
            "account_root": {"node": root_node, "origin": account, "declarations": account_declarations,
                "proposal": root_proposal, "accepted": root_accepted},
            "account_path": {"node": path_node, "origin": path_origin,
                "proposal": path_proposal, "accepted": path_accepted},
            "root_path_owns_edges": edge_rows,
            "all_state_roots": all_state_roots,
            "all_state_paths": all_state_paths,
            "all_state_declarations": provenance.state_declarations(),
            "unrelated_source_file": {"node": source_file_node, "origin": source_file},
            "source_evidence": source_evidence,
            "source_handles": source_handles
        })
    };
    read.close()?;
    store.close()?;

    // The only source directory removed here is the one this fixture created.
    // The retained platform store and the newly published Primary store live
    // outside it and remain available to service and CLI acquisition.
    std::fs::remove_dir_all(&source_root)?;
    assert!(!source_root.exists());

    let mut host_wire = retained_host;
    host_wire["primary"]["project_store_root"] = json!(primary_store_root);
    let host_bytes = wow_core::canonical_json_bytes(&host_wire)?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, &stop)?;

    build_wire["operation"] = json!("build");
    build_wire["primary"]["store_generation_id"] = json!(generation);
    build_wire["primary"]["publication_set_id"] = native_expected["publication_set_id"].clone();
    build_wire["primary"]["root"] = json!({
        "kind": "graph_entity",
        "id": native_expected["account_path"]["node"]["node_id"]
    });
    build_wire["build"]["level"] = json!("l1");
    build_wire["build"]["renderer_profile_ids"] = json!([
        wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE,
        wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
    ]);
    build_wire["build"]["l1_profile_id"] = json!(STATE_DETAIL_L1_PROFILE);
    let mut inspect_wire = build_wire.clone();
    inspect_wire["operation"] = json!("inspect");
    inspect_wire
        .as_object_mut()
        .ok_or("Inspect request object")?
        .remove("build");
    inspect_wire["inspection"] = json!({
        "level": "l1",
        "l1_profile_id": STATE_DETAIL_L1_PROFILE
    });

    // Wrong-kind (project root) selection is refused during strict service
    // request decoding. No alternate identity or source-name lookup is tried.
    let mut wrong_kind = inspect_wire.clone();
    wrong_kind["primary"]["root"]["kind"] = json!("project");
    assert!(
        ContextCombinedRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wrong_kind)?,
            &stop,
        )
        .is_err()
    );

    let inspect_request_bytes = wow_core::canonical_json_bytes(&inspect_wire)?;
    // Preserve genuine retained inputs for repairs without another cold run.
    write_new(&export.join("diagnostic-host.json"), &host_bytes)?;
    write_new(
        &export.join("diagnostic-inspect-request.json"),
        &inspect_request_bytes,
    )?;
    write_new(
        &export.join("diagnostic-build-request.json"),
        &wow_core::canonical_json_bytes(&build_wire)?,
    )?;
    write_new(
        &export.join("diagnostic-native-state.json"),
        &wow_core::canonical_json_bytes(&native_expected)?,
    )?;
    write_new(
        &export.join("diagnostic-source-absence.json"),
        &wow_core::canonical_json_bytes(&json!({"removed_source_root": source_root}))?,
    )?;
    let inspect_request = ContextCombinedRequest::from_json_slice(&inspect_request_bytes, &stop)?;
    let inspect_start = Instant::now();
    let inspect_result = execute_context_combined(&host, &inspect_request, &stop)?;
    let inspect_seconds = inspect_start.elapsed().as_secs_f64();
    write_new(
        &export.join("observed-inspect-envelope.json"),
        inspect_result.canonical_bytes(),
    )?;
    success_envelope(&inspect_result)?;
    verify_retained_state_detail(
        &retained,
        &export,
        Some(source_root),
        Some(inspect_seconds),
        &stop,
    )
}

// Resume only from the retained native store and observations written after
// actual source removal. These JSON records are test oracles, never owner input
// authority: Build still acquires and reconstructs the exact native publication.
fn verify_retained_state_detail(
    retained: &std::path::Path,
    export: &std::path::Path,
    source_root: Option<PathBuf>,
    inspect_seconds: Option<f64>,
    stop: &AtomicBool,
) -> TestResult {
    let host_bytes = read_bounded(
        &export.join("diagnostic-host.json"),
        CONTEXT_COMBINED_HOST_INPUT_MAX_BYTES,
    )?;
    let host_wire: Value = serde_json::from_slice(&host_bytes)?;
    let host = ContextCombinedHostInput::from_json_slice(&host_bytes, stop)?;
    let inspect_request_bytes = read_bounded(
        &export.join("diagnostic-inspect-request.json"),
        CONTEXT_COMBINED_REQUEST_MAX_BYTES,
    )?;
    let inspect_wire: Value = serde_json::from_slice(&inspect_request_bytes)?;
    let mut wrong_kind = inspect_wire.clone();
    wrong_kind["primary"]["root"]["kind"] = json!("project");
    assert!(
        ContextCombinedRequest::from_json_slice(
            &wow_core::canonical_json_bytes(&wrong_kind)?,
            stop,
        )
        .is_err()
    );
    let build_request_bytes = read_bounded(
        &export.join("diagnostic-build-request.json"),
        CONTEXT_COMBINED_REQUEST_MAX_BYTES,
    )?;
    let build_request = ContextCombinedRequest::from_json_slice(&build_request_bytes, stop)?;
    let native_expected: Value = serde_json::from_slice(&read_bounded(
        &export.join("diagnostic-native-state.json"),
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )?)?;
    let inspect_path = export.join("observed-inspect-envelope.json");
    let inspect_seconds = if inspect_path.exists() {
        inspect_seconds
    } else {
        let request = ContextCombinedRequest::from_json_slice(&inspect_request_bytes, stop)?;
        let start = Instant::now();
        let result = execute_context_combined(&host, &request, stop)?;
        let elapsed = start.elapsed().as_secs_f64();
        write_new(&inspect_path, result.canonical_bytes())?;
        Some(elapsed)
    };
    let inspect_expected_bytes = read_bounded(&inspect_path, CONTEXT_MAP_RESULT_MAX_BYTES)?;
    let inspect_envelope: Value = serde_json::from_slice(&inspect_expected_bytes)?;
    assert_eq!(inspect_envelope["status"], "partial");
    assert_eq!(inspect_envelope["resource_closure"], "closed");
    let retained_build: Value = serde_json::from_slice(&read_bounded(
        &retained.join("expected-build-envelope.json"),
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )?)?;
    let mut cli_case: Value = serde_json::from_slice(&read_bounded(
        &retained.join("build-cli-case.json"),
        65_536,
    )?)?;
    assert_eq!(inspect_envelope["operation"], "context_combined_inspect");
    let inspect_payload = &inspect_envelope["payload"];
    let combined_inspect = &inspect_payload["combined_inspect"];
    let primary_l1 = &combined_inspect["core"]["primary_skeletons"]["l1"]["core"];
    assert_eq!(
        primary_l1["schema"],
        "wow-context/l1-skeleton/primary-state-detail/1"
    );
    assert_eq!(primary_l1["profile"], STATE_DETAIL_L1_PROFILE);
    let detail_rows = primary_l1["declarations"]
        .as_array()
        .ok_or("native L1 declarations")?;
    let root_id = native_expected["account_root"]["node"]["node_id"]
        .as_str()
        .ok_or("native root node ID")?;
    let root_row = detail_rows
        .iter()
        .find(|row| row["kind"] == "state_root" && row["node"]["node_id"] == root_id)
        .ok_or("original account root row in StateDetail L1")?;
    assert_eq!(
        root_row["origin"],
        native_expected["account_root"]["origin"]
    );
    assert_eq!(
        root_row["declarations"],
        native_expected["account_root"]["declarations"]
    );
    let primary_origins = &primary_l1["context"]["origins"]["core"];
    // This is the exact literal settings path in the original producer records,
    // not an arbitrary first provenance row that may lie outside the prefix.
    let path_expected = native_expected["all_state_paths"]
        .as_array()
        .ok_or("original native paths")?
        .iter()
        .find(|row| {
            row["origin"]["root_id"] == native_expected["account_root"]["origin"]["root_id"]
                && row["origin"]["keys"] == json!([{"kind": "string", "value": "settings"}])
        })
        .ok_or("exact original account settings path")?;
    assert_eq!(primary_l1["root"], path_expected["node"]);
    assert_eq!(
        inspect_wire["primary"]["root"]["id"],
        path_expected["node"]["node_id"]
    );
    assert_original_entity_support(
        primary_origins,
        &native_expected["account_root"],
        &native_expected,
    )?;
    assert_original_entity_support(primary_origins, path_expected, &native_expected)?;
    let all_state_declarations = native_expected["all_state_declarations"]
        .as_array()
        .ok_or("all original TOC declaration rows")?;
    assert!(
        all_state_declarations.len() >= 2,
        "plural TOC declaration custody"
    );
    assert!(
        all_state_declarations
            .iter()
            .any(|row| row["scope"] == "account")
    );
    assert!(
        all_state_declarations
            .iter()
            .any(|row| row["scope"] == "character")
    );
    let path_id = path_expected["node"]["node_id"]
        .as_str()
        .ok_or("native path node ID")?;
    let path_row = detail_rows
        .iter()
        .find(|row| row["kind"] == "state_path" && row["node"]["node_id"] == path_id)
        .ok_or("original account path row in StateDetail L1")?;
    assert_eq!(path_row["origin"], path_expected["origin"]);
    let local_entities = primary_l1["local_entities"]
        .as_array()
        .ok_or("native L1 entities")?;
    assert!(local_entities.iter().any(|row| row["node_id"] == root_id));
    assert!(local_entities.iter().any(|row| row["node_id"] == path_id));
    let selected_state_entities = local_entities
        .iter()
        .filter(|row| matches!(row["kind"].as_str(), Some("state_root" | "state_path")))
        .collect::<Vec<_>>();
    let emitted_state_rows = detail_rows
        .iter()
        .filter(|row| matches!(row["kind"].as_str(), Some("state_root" | "state_path")))
        .collect::<Vec<_>>();
    assert_eq!(selected_state_entities.len(), emitted_state_rows.len());
    for entity in &selected_state_entities {
        let node_id = entity["node_id"].as_str().ok_or("selected state node ID")?;
        let expected_rows = ["all_state_roots", "all_state_paths"]
            .into_iter()
            .flat_map(|field| native_expected[field].as_array().into_iter().flatten())
            .collect::<Vec<_>>();
        let original = expected_rows
            .into_iter()
            .find(|row| row["node"]["node_id"] == node_id)
            .ok_or("selected state row has original typed source custody")?;
        let output = emitted_state_rows
            .iter()
            .find(|row| row["node"]["node_id"] == node_id)
            .ok_or("selected state row emitted by StateDetail")?;
        assert_eq!(output["origin"], original["origin"]);
        if output["kind"] == "state_root" {
            assert_eq!(output["declarations"], original["declarations"]);
        }
        assert_original_entity_support(primary_origins, original, &native_expected)?;
    }
    let direct_relations = primary_l1["direct_relations"]
        .as_array()
        .ok_or("native L1 relations")?;
    for expected_edge in native_expected["root_path_owns_edges"]
        .as_array()
        .ok_or("native root/path edges")?
    {
        if expected_edge["from"] == path_id || expected_edge["to"] == path_id {
            assert!(direct_relations.iter().any(|edge| edge == expected_edge));
        }
    }
    assert!(direct_relations.iter().any(|edge| {
        edge["from"] == root_id && edge["to"] == path_id && edge["relation"] == "owns"
    }));
    let root_handles = native_expected["account_root"]["origin"]["source_handle_ids"]
        .as_array()
        .ok_or("StateRoot source handles")?;
    let root_evidence = native_expected["account_root"]["origin"]["evidence_ids"]
        .as_array()
        .ok_or("StateRoot evidence IDs")?;
    assert!(!root_handles.is_empty() && !root_evidence.is_empty());
    for handle in root_handles {
        assert!(
            native_expected["source_handles"]
                .as_array()
                .ok_or("source handle custody")?
                .iter()
                .any(|row| row["id"] == *handle)
        );
    }
    for evidence in root_evidence {
        assert!(
            native_expected["source_evidence"]
                .as_array()
                .ok_or("source evidence custody")?
                .iter()
                .any(|row| row["id"] == *evidence)
        );
    }
    let prior_inspect = &retained_build["payload"]["semantic_pack"]["core"]["origins"];
    let prior_map_core = &prior_inspect["core"]["map"]["core"];
    let current_core = &combined_inspect["core"];
    assert_eq!(
        current_core["map"]["core"]["platform"],
        prior_map_core["platform"]
    );
    assert_eq!(
        current_core["platform_inspect"],
        prior_inspect["core"]["platform_inspect"]
    );

    let build_path = export.join("observed-build-envelope.json");
    let build_seconds = if build_path.exists() {
        None
    } else {
        let start = Instant::now();
        let result = execute_context_combined(&host, &build_request, stop)?;
        let elapsed = start.elapsed().as_secs_f64();
        write_new(&build_path, result.canonical_bytes())?;
        success_envelope(&result)?;
        Some(elapsed)
    };
    let build_expected_bytes = read_bounded(&build_path, CONTEXT_MAP_RESULT_MAX_BYTES)?;
    let build_envelope: Value = serde_json::from_slice(&build_expected_bytes)?;
    assert_eq!(build_envelope["status"], "partial");
    assert_eq!(build_envelope["resource_closure"], "closed");
    assert!(build_envelope.get("failure_code").is_none());
    assert_eq!(build_envelope["operation"], "context_combined_build");
    let build_payload = &build_envelope["payload"];
    let pack = &build_payload["semantic_pack"];
    assert_eq!(pack["core"]["origins"], *combined_inspect);
    assert_eq!(pack["core"]["state"], "partial");
    assert_eq!(pack["core"]["absence_authoritative"], false);
    let renderers = build_payload["rendered_artifacts"]
        .as_array()
        .ok_or("combined renderers")?;
    assert_eq!(renderers.len(), 2, "JSON and Markdown renderer selection");
    let semantic_bytes = wow_core::canonical_json_bytes(pack)?;
    for (index, renderer) in renderers.iter().enumerate() {
        let row = &renderer["core"];
        let profile_id = row["profile"]["profile_id"]
            .as_str()
            .ok_or("renderer profile")?;
        let expected_profile = if index == 0 {
            wow_context::COMBINED_CONTEXT_JSON_RENDERER_PROFILE
        } else {
            wow_context::COMBINED_CONTEXT_MARKDOWN_RENDERER_PROFILE
        };
        assert_eq!(profile_id, expected_profile);
        let content = row["content"].as_str().ok_or("renderer content")?;
        assert_eq!(row["byte_length"], content.len());
        assert_eq!(
            row["content_digest"],
            serde_json::to_value(digest(content.as_bytes()))?
        );
        if index == 0 {
            assert_eq!(content.as_bytes(), semantic_bytes.as_slice());
            assert_eq!(
                row["trace"].as_array().ok_or("JSON renderer trace")?.len(),
                1
            );
        } else {
            let traces = row["trace"].as_array().ok_or("Markdown renderer traces")?;
            assert!(
                traces.len() >= 8,
                "Markdown retains pack and native item traces"
            );
            let end = traces.last().ok_or("Markdown final trace")?["byte_end"]
                .as_u64()
                .ok_or("Markdown final byte end")?;
            assert_eq!(usize::try_from(end)?, content.len());
        }
    }
    let build_usage = &build_payload["operation_budget_usage"];
    let build_work = build_usage["work_reserved"]
        .as_u64()
        .ok_or("combined work reservation")?;
    assert!(build_work <= 16_000_000);
    assert!(
        build_usage["bytes_reserved"]
            .as_u64()
            .ok_or("combined byte reservation")?
            + u64::try_from(build_expected_bytes.len())?
            < 16 * 1024 * 1024
    );

    // A below-profile neighborhood output limit is invalid configuration,
    // refused before acquisition. It cannot prove runtime budget/root custody.
    let path_id = path_expected["node"]["node_id"].clone();
    let mut truncated_host_wire = host_wire.clone();
    for owner in ["primary", "platform"] {
        truncated_host_wire[owner]["limits"]["neighbors"]["max_output_bytes"] = json!(1);
    }
    let admission_error = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&truncated_host_wire)?,
        stop,
    )
    .err()
    .ok_or("below-profile output limit must refuse during host decoding")?;
    assert_eq!(
        admission_error.code(),
        ServiceErrorCode::InvalidConfiguration
    );

    let mut low_budget_wire = host_wire.clone();
    low_budget_wire["primary"]["limits"]["max_context_work"] =
        json!(build_work.checked_sub(1).ok_or("positive work allowance")?);
    low_budget_wire["platform"]["limits"]["max_context_work"] =
        low_budget_wire["primary"]["limits"]["max_context_work"].clone();
    let low_budget_host = ContextCombinedHostInput::from_json_slice(
        &wow_core::canonical_json_bytes(&low_budget_wire)?,
        stop,
    )?;
    refusal(execute_context_combined(
        &low_budget_host,
        &build_request,
        stop,
    ))?;

    let measurement = json!({
        "schema": "wow-service/combined-state-detail-fixture-measurement/1",
        "measured_totals_seconds": {
            "combined_inspect_service_operation": inspect_seconds.map_or_else(
                || json!("not_measured_for_retained_observation"),
                |seconds| json!(seconds),
            ),
            "combined_build_service_operation": build_seconds.map_or_else(
                || json!("not_measured_for_retained_observation"),
                |seconds| json!(seconds),
            )
        },
        "operation_budget_usage": {
            "inspect": "full continuing reservation not exposed by this service envelope",
            "build": build_usage
        },
        "unmeasured": {
            "internal_phase_totals": "not_measured",
            "graph_rebuilds": "not_measured",
            "physical_graph_scans": "not_measured",
            "os_peak_memory": "not_measured",
            "cli_runtime": "not_measured"
        },
        "refusal_probes": {
            "wrong_kind": "strict_request_decode_refused",
            "below_profile_neighborhood_output": "InvalidConfiguration before acquisition",
            "mandatory_neighborhood_output": "not_evaluated_by_this_decode_probe",
            "missing_local_root_custody": "not_evaluated_by_this_probe",
            "cumulative_budget": "BudgetExceeded below observed complete Build work"
        },
        "state_coverage": {
            "accepted_account_state_root": root_id,
            "accepted_state_path_witness": path_id,
            "original_state_rows_and_toc_declaration_support": "compared",
            "unrelated_state_lua_source_file": native_expected["unrelated_source_file"]["node"]["node_id"]
        },
        "parent_assert_combined_pack_and_renderers": "not_applicable_l1_only_selection"
    });
    let native_expected_bytes = wow_core::canonical_json_bytes(&native_expected)?;
    // Timings are auxiliary observations, outside the canonical semantic wire.
    let measurement_bytes = serde_json::to_vec(&measurement)?;
    cli_case["argv"] = json!([
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
    cli_case["expected_envelope"] = json!("expected-build-envelope.json");
    if let Some(source_root) = source_root {
        assert!(!source_root.exists());
        cli_case["source_absent_paths"]
            .as_array_mut()
            .ok_or("original source absence witnesses")?
            .push(json!(source_root));
    }
    let cli_case_bytes = wow_core::canonical_json_bytes(&cli_case)?;
    let exports = [
        ("host.json", host_bytes.as_slice()),
        ("inspect-request.json", inspect_request_bytes.as_slice()),
        ("build-request.json", build_request_bytes.as_slice()),
        (
            "expected-inspect-envelope.json",
            inspect_expected_bytes.as_slice(),
        ),
        (
            "expected-build-envelope.json",
            build_expected_bytes.as_slice(),
        ),
        (
            "native-state-expected.json",
            native_expected_bytes.as_slice(),
        ),
        ("measurement.json", measurement_bytes.as_slice()),
        ("build-cli-case.json", cli_case_bytes.as_slice()),
    ];
    for (name, bytes) in exports {
        write_new(&export.join(name), bytes)?;
    }
    Ok(())
}
