use super::*;
use wow_emmy::function_calls::{FunctionCallBudget, HookTargetRecipe};
use wow_project::replay::publication::{self, AcquiredProjectPair, ProjectPublicationBundle};
use wow_project::{ProjectBudgetPolicy, ProjectPublisher, replay::ProjectReplay};
use wow_store::project::{ProjectStore, PublicationRequest, ReadSelector, RecordCatalog};

const LITERAL_SOURCE: &str = concat!(
    "function ArbitraryTarget() end\n",
    "ExactTable = {}\n",
    "function ExactTable.ArbitraryMember() end\n",
    "function Callback() end\n",
    "function InstallGlobal()\n",
    "hooksecurefunc(\"ArbitraryTarget\", Callback)\n",
    "end\n",
    "function InstallTable()\n",
    "hooksecurefunc(ExactTable, \"ArbitraryMember\", Callback)\n",
    "end\n",
    "function Unsupported(dynamic, callback)\n",
    "hooksecurefunc(dynamic, Callback)\n",
    "hooksecurefunc(\"ArbitraryTarget\", callback)\n",
    "hooksecurefunc(true, Callback)\n",
    "hooksecurefunc(ExactTable, dynamic, Callback)\n",
    "local ExactTable = {}\n",
    "hooksecurefunc(ExactTable, \"ArbitraryMember\", Callback)\n",
    "local function hooksecurefunc(...) end\n",
    "hooksecurefunc(\"ArbitraryTarget\", Callback)\n",
    "end\n",
);

fn literal_fixture(selected: bool) -> TestResult<(FixtureRoot, Value)> {
    let (root, mut config) = fixture()?;
    replace_main_source(&root, &mut config, LITERAL_SOURCE)?;
    admit_library_hook(&root, &mut config)?;
    select_main_function_declarations(&root, &mut config)?;
    config["schema"] = json!(crate::local::LOCAL_PLATFORM_BUDGET_SCHEMA);
    config["retained_package_limits"] = serde_json::to_value(
        wow_project::platform_source::PlatformPackageRetentionLimits::new(2, 1024 * 1024)?,
    )?;
    config["project_budgets"] = serde_json::to_value(
        ProjectBudgetPolicy::fixture_e0()?.with_function_call_budget(
            FunctionCallBudget::default()
                .with_global_access_budget(wow_emmy::global_access::GlobalAccessBudget::default()),
        ),
    )?;
    if selected {
        config["project_budgets"]["hook_target_recipe"] =
            serde_json::to_value(HookTargetRecipe::HookTargetLiteralsV1)?;
    } else {
        config
            .as_object_mut()
            .ok_or("config")?
            .remove("physical_function_declarations");
    }
    write_json(&root, "project.json", &config)?;
    Ok((root, config))
}

#[test]
fn canonical_hook_literals_reach_service_graph_readback() -> TestResult {
    let (root, _) = literal_fixture(true)?;
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let result = execute_graph_build(input, &request, &stop)?;
    assert_ne!(
        result.status(),
        GraphReadStatus::Failed,
        "{:?}",
        result.failure_message()
    );
    assert_ne!(result.status(), GraphReadStatus::Cancelled);
    let graph = serde_json::to_value(&result)?;
    let report = &graph["provenance"]["function_call_report"];
    assert_eq!(report["hook_target_recipe"], "hook_target_literals_v1");
    assert!(report["named_targets"].get("ArbitraryTarget").is_none());
    assert!(
        report["named_targets"]
            .get("ExactTable.ArbitraryMember")
            .is_none()
    );
    let matches = graph["hook_recognition"]["secure_posthook_matches"]
        .as_array()
        .ok_or("matches")?;
    let exact = matches
        .iter()
        .filter(|matched| {
            matched["target_proposal_id"].is_string() && matched["callback_proposal_id"].is_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(exact.len(), 2, "{matches:?}");
    // Independent output identities retain their own confidence. Putting a
    // known and unknown callback under one caller/target deliberately invokes
    // the matcher's semantic-competition policy instead of this positive case.
    for matched in matches {
        assert_eq!(
            matched["confidence"] == "derived",
            matched["target_proposal_id"].is_string()
                && matched["callback_proposal_id"].is_string(),
            "{matched:?}"
        );
    }
    let snapshot_bytes = result.snapshot_bytes()?.ok_or("Service graph snapshot")?;
    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(&snapshot_bytes)?;
    let mut nodes = Vec::new();
    let mut expected_edges = Vec::new();
    let calls = report["calls"].as_array().ok_or("calls")?;
    for matched in exact {
        let call = calls
            .iter()
            .find(|call| call["fact_id"] == matched["call_id"])
            .ok_or("exact call")?;
        assert_eq!(call["target"], report["named_targets"]["hooksecurefunc"]);
        assert_eq!(call["target"]["state"], "library_function");
        let target_id = call["hook_target"]["function_id"]
            .as_str()
            .ok_or("typed Main target")?;
        let query = call["hook_target"]["query"]
            .as_str()
            .ok_or("typed lookup request")?;
        assert!(matches!(
            query,
            "ArbitraryTarget" | "ExactTable.ArbitraryMember"
        ));
        assert_eq!(
            matched["target_proposal_id"],
            function_proposal(&graph["provenance"], target_id)?
        );
        let caller_id = call["caller_function_id"].as_str().ok_or("caller")?;
        let caller = function_node(&graph, caller_id)?;
        let target = function_node(&graph, target_id)?;
        let callback_id = graph["provenance"]["functions"]
            .as_array()
            .ok_or("function provenance")?
            .iter()
            .find(|function| function["proposal_id"] == matched["callback_proposal_id"])
            .and_then(|function| function["function_id"].as_str())
            .ok_or("callback function")?;
        nodes.push(GraphNodeId::new(function_node(&graph, callback_id)?)?);
        expected_edges.push((caller.clone(), target.clone()));
        nodes.extend([GraphNodeId::new(caller)?, GraphNodeId::new(target)?]);
    }
    nodes.sort();
    nodes.dedup();
    let query = GraphSubgraphQuery::new(
        snapshot.snapshot().snapshot_id().clone(),
        nodes,
        GraphDirection::Both,
        vec![GraphRelationKind::SecureHooksFunction],
        GraphSubgraphLimits::default(),
    )?;
    let read = execute_graph_read(
        GraphReadOperation::Subgraph,
        &snapshot_bytes,
        &wow_core::canonical_json_bytes(&GraphReadRequest::new(GraphReadQuery::Subgraph(query)))?,
        &stop,
    )?;
    let all_edges = read.payload().ok_or("readback payload")?["edges"]
        .as_array()
        .ok_or("edges")?;
    let edges = all_edges
        .iter()
        .filter(|edge| edge["confidence"] == "derived")
        .collect::<Vec<_>>();
    assert_eq!(
        edges.len(),
        2,
        "conservative calls cannot mint extra Derived hook edges: actual={all_edges:?}, expected={expected_edges:?}"
    );
    for (from, to) in expected_edges {
        assert!(
            edges.iter().any(|edge| {
                edge["from"] == from
                    && edge["to"] == to
                    && edge["relation"] == "secure_hooks_function"
                    && edge["confidence"] == "derived"
            }),
            "exact Derived hook edge absent from readback"
        );
    }
    // The explicitly registered selected-components catalog is the real
    // AcquiredProjectPair consumer; no expected-hash reader search is involved.
    let bundle = ProjectPublicationBundle::build(&publisher, &snapshot, &stop)?;
    assert_eq!(
        bundle.replay_storage_schema(),
        publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1[0]
    );
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1);
    let catalog = RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )?;
    let store_path = root.0.join("literal-store");
    let mut store = ProjectStore::create(
        &store_path,
        snapshot.snapshot().universe().as_str(),
        catalog.clone(),
    )?;
    let (records, bindings) = bundle.into_parts();
    let publication = PublicationRequest::new(
        store.epoch(),
        wow_store::OperationId::new("fixture:hook-literals")?,
        None,
        bindings,
        records,
    )?;
    store.prepare(&publication, &stop)?;
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_dir_all(root.0.join("Library"))?;
    drop(store);
    let reopened = ProjectStore::open(&store_path, &catalog)?;
    let read = reopened.read(
        &ReadSelector::Exact(publication.generation().generation_id.clone()),
        &stop,
    )?;
    let acquired = AcquiredProjectPair::read(&read, &stop)?;
    assert_eq!(
        acquired.project().snapshot_id(),
        publisher.open_current()?.snapshot_id()
    );
    let restored = acquired
        .project()
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("acquired hook report")?;
    assert_eq!(serde_json::to_value(restored)?, *report);
    assert_eq!(
        acquired.graph().snapshot().snapshot_id(),
        snapshot.snapshot().snapshot_id()
    );
    drop(acquired);
    read.close()?;
    Ok(())
}

#[test]
fn canonical_hook_literals_replay_source_absent_preserves_original_ids_and_old_reader() -> TestResult
{
    let stop = AtomicBool::new(false);
    let mut cases = Vec::new();
    for selected in [false, true] {
        let (root, _) = literal_fixture(selected)?;
        let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
        let mut publisher = ProjectPublisher::with_function_call_facts();
        publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
        let view = publisher.open_current()?;
        let report = view
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("report")?;
        let bytes = serde_json::to_vec(report)?;
        assert_eq!(report.hook_target_recipe().is_some(), selected);
        let replay = ProjectReplay::capture(&publisher, &stop)?;
        let wire = serde_json::to_value(&replay)?;
        assert_eq!(
            wire["schema"],
            if selected {
                "wow-project/native-project-replay/selected-components/1"
            } else {
                "wow-project/native-project-replay/16"
            }
        );
        std::fs::remove_dir_all(root.0.join("UI"))?;
        std::fs::remove_dir_all(root.0.join("Library"))?;
        cases.push((
            root,
            replay,
            view.snapshot_id().to_owned(),
            view.analyzer_snapshot_id().to_owned(),
            bytes,
            selected,
        ));
    }
    for (root, replay, snapshot_id, analyzer_id, report_bytes, selected) in cases {
        assert!(!root.0.join("UI").exists() && !root.0.join("Library").exists());
        let restored =
            ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
        assert_eq!(restored.snapshot_id(), snapshot_id);
        assert_eq!(restored.analyzer_snapshot_id(), analyzer_id);
        let report = restored
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("restored report")?;
        assert_eq!(serde_json::to_vec(report)?, report_bytes);
        assert_eq!(
            report
                .calls()
                .iter()
                .filter(|call| call.hook_target().is_some())
                .count(),
            if selected { 3 } else { 0 }
        );
    }
    Ok(())
}
