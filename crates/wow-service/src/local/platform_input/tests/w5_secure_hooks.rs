use super::*;
#[path = "w5_hook_targets.rs"]
mod literal_targets;
use crate::graph::{
    GraphBuildRequest, GraphDirection, GraphReadOperation, GraphReadQuery, GraphReadRequest,
    GraphReadStatus, GraphSubgraphLimits, GraphSubgraphQuery, execute_graph_build,
    execute_graph_read,
};
use wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe;
use wow_graph::{GraphNodeId, GraphPartitionSnapshot, GraphRelationKind};

const LIBRARY_HOOK: &[u8] = b"function hooksecurefunc(name, callback) end\n";
const MAIN_SOURCE: &str = concat!(
    "function Mixin(value) return value end\n",
    "function Callback(...) end\n",
    "function capturedInstallHooks()\n",
    "    hooksecurefunc(\"Mixin\", Callback)\n",
    "end\n",
    "function capturedWrongForm()\n",
    "    hooksecurefunc(true, Callback)\n",
    "end\n",
    "local function capturedMainShadow()\n",
    "    local function hooksecurefunc(name, callback) return nil end\n",
    "    hooksecurefunc(\"Mixin\", Callback)\n",
    "end\n",
);

fn replace_main_source(root: &FixtureRoot, config: &mut Value, lua: &str) -> TestResult {
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"]
        .as_array_mut()
        .ok_or("fixture manifest files")?;
    let main = files
        .iter_mut()
        .find(|file| file["path"] == "UI/main.lua")
        .ok_or("fixture Main Lua member")?;
    main["bytes"] = json!(lua.len());
    main["content_sha256"] = json!(format!("{:x}", Sha256::digest(lua.as_bytes())));
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("manifest member bytes")?)
                .ok_or("manifest byte sum")?)
        })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("fixture manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    std::fs::write(root.0.join("UI/main.lua"), lua)?;
    config["source"]["manifest"] =
        serde_json::to_value(write_json(root, "manifest.json", &manifest)?)?;
    config["source"]["origin"]["revision"] =
        serde_json::to_value(PlatformSourceRevision::Fixture {
            digest: digest(&serde_json::to_vec(&manifest)?),
        })?;
    write_json(root, "project.json", config)?;
    Ok(())
}

fn admit_library_hook(root: &FixtureRoot, config: &mut Value) -> TestResult {
    let library_root = root.0.join("Library");
    std::fs::create_dir(&library_root)?;
    std::fs::write(library_root.join("hooksecurefunc.lua"), LIBRARY_HOOK)?;
    let file = ProjectDiskFile::new("hooksecurefunc.lua")
        .with_identity(digest(LIBRARY_HOOK), LIBRARY_HOOK.len() as u64);
    config["library"] = json!({"root":"Library","files":[file]});
    write_json(root, "project.json", config)?;
    Ok(())
}

fn select_main_function_declarations(root: &FixtureRoot, config: &mut Value) -> TestResult {
    config["physical_function_declarations"] =
        serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
    write_json(root, "project.json", config)?;
    Ok(())
}
fn function_proposal(provenance: &Value, function_id: &str) -> TestResult<String> {
    provenance["functions"]
        .as_array()
        .ok_or("held Project function provenance")?
        .iter()
        .find(|function| function["function_id"].as_str() == Some(function_id))
        .and_then(|function| function["proposal_id"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("missing held Project function proposal for {function_id}").into())
}

fn function_node(graph: &Value, function_id: &str) -> TestResult<String> {
    graph["function_nodes"]
        .as_array()
        .ok_or("Service function nodes")?
        .iter()
        .find(|function| function["function_id"].as_str() == Some(function_id))
        .and_then(|function| function["node_id"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| format!("missing Service graph node for {function_id}").into())
}

#[test]
fn canonical_service_secure_posthook_builds_and_reads_back_exact_main_endpoints() -> TestResult {
    let (root, mut config) = fixture()?;
    replace_main_source(&root, &mut config, MAIN_SOURCE)?;
    admit_library_hook(&root, &mut config)?;
    select_main_function_declarations(&root, &mut config)?;

    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_ne!(
        result.status(),
        GraphReadStatus::Failed,
        "canonical Service build failed: {:?}",
        result.failure_message()
    );
    assert_ne!(result.status(), GraphReadStatus::Cancelled);

    let graph = serde_json::to_value(&result)?;
    let provenance = &graph["provenance"];
    let report = &provenance["function_call_report"];
    let named = &report["named_targets"];
    let library_target = &named["hooksecurefunc"];
    let target = &named["Mixin"];
    assert_eq!(library_target["state"], "library_function");
    assert_eq!(target["state"], "main_function");

    let calls = report["calls"]
        .as_array()
        .ok_or("held Emmy call facts")?
        .iter()
        .collect::<Vec<_>>();
    assert_eq!(
        calls.len(),
        3,
        "one valid, one known-wrong-form, one Main shadow"
    );

    let valid = calls
        .iter()
        .find(|call| {
            call["target"] == *library_target
                && call["arguments"][0]["literal"]["kind"] == "string"
                && call["arguments"][0]["literal"]["value"] == "Mixin"
        })
        .copied()
        .ok_or("exact selected Library hook call")?;
    assert_eq!(valid["arguments"][1]["reference_key"], "Callback");
    let callback_reference = &valid["arguments"][1]["reference_target"];
    assert_eq!(callback_reference["role"], "main");
    assert_eq!(
        callback_reference["workspace_id"],
        report["main_snapshot_id"]
    );

    let wrong_form = calls
        .iter()
        .find(|call| {
            call["target"] == *library_target
                && call["arguments"][0]["literal"]["kind"] == "boolean"
                && call["arguments"][0]["literal"]["value"] == true
        })
        .ok_or("known-wrong-form call to selected Library hook")?;
    let main_shadow = calls
        .iter()
        .find(|call| {
            call["target"]["state"] == "main_function"
                && call["arguments"][0]["literal"]["value"] == "Mixin"
        })
        .ok_or("same-spelled Main hook shadow call")?;

    let matches = graph["hook_recognition"]["secure_posthook_matches"]
        .as_array()
        .ok_or("Service W5 secure-posthook recognition")?;
    assert_eq!(matches.len(), 1, "only the exact Library form may match");
    let matched = &matches[0];
    assert_eq!(matched["call_id"], valid["fact_id"]);
    assert_ne!(matched["call_id"], wrong_form["fact_id"]);
    assert_ne!(matched["call_id"], main_shadow["fact_id"]);

    let target_function_id = target["function_id"]
        .as_str()
        .ok_or("exact Main Mixin function id")?;
    let callback_proposal_id = matched["callback_proposal_id"]
        .as_str()
        .ok_or("W5 exact callback proposal id")?;
    let callback_function = provenance["functions"]
        .as_array()
        .ok_or("held Project function provenance")?
        .iter()
        .find(|function| function["proposal_id"].as_str() == Some(callback_proposal_id))
        .ok_or("W5 callback proposal has no held Project function")?;
    let callback_function_id = callback_function["function_id"]
        .as_str()
        .ok_or("exact callback Project function id")?;
    assert_eq!(callback_function["path"], callback_reference["path"]);
    let caller_function_id = valid["caller_function_id"]
        .as_str()
        .ok_or("exact capturedInstallHooks caller id")?;
    let target_proposal = function_proposal(provenance, target_function_id)?;
    let caller_proposal = function_proposal(provenance, caller_function_id)?;
    assert_eq!(
        matched["target_proposal_id"].as_str(),
        Some(target_proposal.as_str())
    );

    assert_eq!(
        matched["caller_proposal_id"].as_str(),
        Some(caller_proposal.as_str())
    );

    let snapshot_bytes = result
        .snapshot_bytes()?
        .ok_or("canonical Service graph snapshot")?;
    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(&snapshot_bytes)?;
    let hook_partition = snapshot
        .partition(wow_recognizers::source_scripts::W5_HOOK_PARTITION)
        .ok_or("accepted W5 graph partition")?;
    let relation_id = matched["relation_proposal_id"]
        .as_str()
        .ok_or("W5 relation proposal id")?;
    let accepted = hook_partition
        .report()
        .accepted_relations()
        .iter()
        .find(|relation| relation.proposal_id() == relation_id)
        .ok_or("accepted secure-hook relation")?;
    assert_eq!(
        accepted.edge().relation(),
        GraphRelationKind::SecureHooksFunction
    );
    assert_eq!(
        accepted.edge().confidence(),
        wow_graph::GraphConfidence::Derived
    );

    let caller_node = function_node(&graph, caller_function_id)?;
    let target_node = function_node(&graph, target_function_id)?;
    let callback_node = function_node(&graph, callback_function_id)?;
    let signal_edges = graph["signal_edges"]
        .as_array()
        .ok_or("materialized Service signal edges")?;
    let secure_edges = signal_edges
        .iter()
        .filter(|edge| edge["relation"] == "secure_hooks_function")
        .collect::<Vec<_>>();
    let [service_edge] = secure_edges.as_slice() else {
        return Err("expected one materialized secure-hook edge".into());
    };
    assert_eq!(service_edge["function_node_id"], caller_node);
    assert_eq!(service_edge["target_node_id"], target_node);

    let final_edge = snapshot
        .snapshot()
        .edges()
        .iter()
        .find(|edge| {
            edge.from().as_str() == caller_node.as_str()
                && edge.to().as_str() == target_node.as_str()
                && edge.relation() == GraphRelationKind::SecureHooksFunction
        })
        .ok_or("secure-hook edge in final graph snapshot")?;
    assert_eq!(
        service_edge["edge_id"].as_str(),
        Some(final_edge.edge_id().as_str())
    );

    let query = GraphSubgraphQuery::new(
        snapshot.snapshot().snapshot_id().clone(),
        vec![
            GraphNodeId::new(caller_node.clone())?,
            GraphNodeId::new(target_node.clone())?,
            GraphNodeId::new(callback_node.clone())?,
        ],
        GraphDirection::Both,
        vec![GraphRelationKind::SecureHooksFunction],
        GraphSubgraphLimits::default(),
    )?;
    let read_request = GraphReadRequest::new(GraphReadQuery::Subgraph(query));
    let read = execute_graph_read(
        GraphReadOperation::Subgraph,
        &snapshot_bytes,
        &wow_core::canonical_json_bytes(&read_request)?,
        &stop,
    )?;
    let payload = read.payload().ok_or("Service graph-read payload")?;
    let read_nodes = payload["nodes"]
        .as_array()
        .ok_or("Service graph-read nodes")?;
    for expected in [&caller_node, &target_node, &callback_node] {
        assert!(
            read_nodes
                .iter()
                .any(|node| node["node"]["node_id"].as_str() == Some(expected)),
            "exact endpoint absent from Service graph read-back: {expected}"
        );
    }
    let read_edges = payload["edges"]
        .as_array()
        .ok_or("Service graph-read edges")?;
    assert!(
        read_edges.iter().any(|edge| {
            edge["from"].as_str() == Some(&caller_node)
                && edge["to"].as_str() == Some(&target_node)
                && edge["relation"] == "secure_hooks_function"
                && edge["edge_id"] == service_edge["edge_id"]
        }),
        "accepted secure-hook edge did not survive Service graph read-back"
    );
    Ok(())
}
