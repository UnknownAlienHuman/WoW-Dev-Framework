use super::*;
use crate::graph::{GraphBuildRequest, execute_graph_build};
use crate::live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project};
use std::sync::atomic::AtomicBool;
use wow_graph::{GraphExplainLimits, GraphExplainQuery, GraphExplainSubject};
use wow_project::{ProjectPublisher, SignalWitnessRecipe, replay::ProjectReplay};
use wow_recognizers::source_signals::{W1_SIGNAL_PARTITION, W3_SIGNAL_PARTITION, W4_PARTITION};
use wow_store::project::ReadSelector;

const API: &str = r#"
APIDocumentation:AddDocumentationTable({Name="Frame",Type="System",Namespace="Frame",Environment="All",Functions={{Name="RegisterEvent",Type="Function",Arguments={{Name="self",Type="Frame",Nilable=false},{Name="event",Type="string",Nilable=false}}}}})
APIDocumentation:AddDocumentationTable({Name="Global",Type="System",Environment="All",Functions={{Name="CreateFrame",Type="Function",Arguments={{Name="frameType",Type="string",Nilable=false}},Returns={{Name="frame",Type="Frame",Nilable=false}}}}})
APIDocumentation:AddDocumentationTable({Name="Registry",Type="System",Namespace="EventRegistry",Environment="All",Functions={{Name="TriggerEvent",Type="Function",Arguments={{Name="self",Type="table"},{Name="event",Type="string"}}},{Name="RegisterCallback",Type="Function",Arguments={{Name="self",Type="table"},{Name="event",Type="string"},{Name="callback",Type="Function"}}}}})
APIDocumentation:AddDocumentationTable({Name="CVars",Type="System",Namespace="CVarCallbackRegistry",Environment="All",Functions={{Name="RegisterCallback",Type="Function",Arguments={{Name="self",Type="table"},{Name="name",Type="string"},{Name="callback",Type="Function"}}}}})
"#;
const MAIN: &str = r#"
local function OnSignal() end
function First()
    CreateFrame("Frame")
    Frame:RegisterEvent("PLAYER_LOGIN")
    EventRegistry:TriggerEvent("Signal.Ready")
    EventRegistry:TriggerEvent("Signal.Ready")
    EventRegistry:RegisterCallback("Signal.Ready", OnSignal)
    EventRegistry:RegisterCallback("Signal.Missing", OnSignal)
    CVarCallbackRegistry:RegisterCallback("first_cvar", OnSignal)
    CVarCallbackRegistry:RegisterCallback("first_cvar", OnSignal)
end
function Second()
    Frame:RegisterEvent("PLAYER_LOGIN")
    Frame:RegisterEvent("PLAYER_LOGOUT")
    EventRegistry:TriggerEvent("Signal.Other")
    CVarCallbackRegistry:RegisterCallback("other_cvar", OnSignal)
end
"#;

#[test]
fn all_signal_matcher_witnesses_survive_native_reopen_and_reject_missing_records() -> TestResult {
    let (root, mut config) = native_fixture()?;
    event_registry_custom::replace_source_file(
        &root,
        &mut config,
        "Blizzard_APIDocumentationGenerated/API.lua",
        API,
    )?;
    event_registry_custom::replace_source_file(&root, &mut config, "UI/main.lua", MAIN)?;
    let mut native = json!({"schema":crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id":config["project_id"],"workspace_id":config["workspace_id"],
        "source_origin_id":config["source_origin_id"],"logical_root":config["logical_root"],
        "profile":config["native_reference"]["profile"],"analyzer":config["analyzer"],
        "main":{"root":"UI","files":[ProjectDiskFile::new("main.lua")]},
        "native_source":config["native_reference"]["native_source"],"signal_witnesses":"matcher_v1"});
    write_json(&root, "signal-input.json", &native)?;
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("signal-input.json"), &stop)?;
    assert_eq!(
        input.bundle.configuration().signal_witnesses(),
        Some(SignalWitnessRecipe::MatcherV1)
    );
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&root.0.join("signal-input.json"), &stop)?,
        &request,
        &stop,
    )?;
    let value = serde_json::to_value(&result)?;
    let snapshot: wow_graph::GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or_else(|| {
            format!(
                "snapshot: {:?}: {:?}",
                result.status(),
                result.failure_message()
            )
        })?)?;
    let frames = value["signal_recognition"]["matches"]
        .as_array()
        .ok_or("frame receipts")?;
    assert_eq!(frames.len(), 3);
    for row in frames {
        assert_eq!(
            row["witness"]["matcher_outputs"]
                .as_object()
                .ok_or("original entity/relation")?
                .len(),
            2
        );
    }
    let custom = &value["custom_recognition"];
    assert_eq!(custom["producers"].as_array().ok_or("producers")?.len(), 3);
    let subscriptions = custom["subscriptions"].as_array().ok_or("subscriptions")?;
    let confirmed = subscriptions
        .iter()
        .find(|row| row["event_key"] == "Signal.Ready")
        .ok_or("confirmed")?;
    assert_eq!(
        confirmed["witness"]["corroborating_relations"]
            .as_array()
            .ok_or("all producers")?
            .len(),
        2
    );
    assert_eq!(
        confirmed["witness"]["matcher_outputs"]
            .as_object()
            .ok_or("original match")?
            .len(),
        1
    );
    let possible = subscriptions
        .iter()
        .find(|row| row["event_key"] == "Signal.Missing")
        .ok_or("possible")?;
    assert_eq!(possible["confirmed"], false);
    // The subscription site matched exactly, but its two producer proofs are
    // Possible semantic competitors. Dependent graph confidence cannot promote them.
    let custom_partition = snapshot.partition(W3_SIGNAL_PARTITION).ok_or("custom")?;
    let relation_id = confirmed["relation_proposal_id"]
        .as_str()
        .ok_or("subscription relation")?;
    let original_id = confirmed["witness"]["matcher_relation_proposal_id"]
        .as_str()
        .ok_or("original relation")?;
    assert_eq!(
        relation_id, original_id,
        "selected W3 binds its original match digest"
    );
    assert_eq!(
        confirmed["witness"]["matcher_outputs"][original_id]["matched"]["confidence"],
        "derived"
    );
    let accepted = custom_partition
        .report()
        .accepted_relations()
        .iter()
        .find(|relation| relation.proposal_id() == relation_id)
        .ok_or("accepted subscription")?;
    assert_eq!(
        accepted.edge().confidence(),
        wow_graph::GraphConfidence::Possible
    );
    let entity_id = confirmed["witness"]["entity_proposal_id"]
        .as_str()
        .ok_or("signal entity")?;
    assert_eq!(
        custom_partition
            .batch()
            .entity_proposal(entity_id)
            .ok_or("aggregate")?
            .confidence(),
        wow_graph::GraphConfidence::Possible
    );
    let other = custom["producers"]
        .as_array()
        .ok_or("producer list")?
        .iter()
        .find(|row| row["event_key"] == "Signal.Other")
        .ok_or("independent producer")?;
    let other_id = other["relation_proposal_id"]
        .as_str()
        .ok_or("independent ID")?;
    assert_eq!(
        custom_partition
            .batch()
            .relation_proposal(other_id)
            .ok_or("independent relation")?
            .confidence(),
        wow_graph::GraphConfidence::Derived
    );
    assert_eq!(
        value["cvar_recognition"]["matches"]
            .as_array()
            .ok_or("CVar receipts")?
            .len(),
        3
    );
    for (partition_id, count) in [
        (W1_SIGNAL_PARTITION, 3),
        (W3_SIGNAL_PARTITION, 5),
        (W4_PARTITION, 3),
    ] {
        let partition = snapshot
            .partition(partition_id)
            .ok_or("selected partition")?;
        assert_eq!(partition.report().accepted_relations().len(), count);
        assert!(partition.batch().assertion_records().is_some());
    }
    let missing_id = possible["witness"]["entity_proposal_id"]
        .as_str()
        .ok_or("missing entity")?;
    let missing_partition = snapshot
        .partition(W3_SIGNAL_PARTITION)
        .ok_or("custom partition")?;
    let missing = missing_partition
        .report()
        .accepted_entities()
        .iter()
        .find(|entity| entity.proposal_id() == missing_id)
        .ok_or("missing node")?;
    let node = snapshot
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.kind() == missing.node().kind() && node.owner_key() == missing.node().owner_key()
        })
        .ok_or("materialized missing")?;
    let query = GraphExplainQuery::new(
        snapshot.snapshot().snapshot_id().clone(),
        GraphExplainSubject::Entity(node.node_id().clone()),
        GraphExplainLimits::default(),
    )?;
    let explanation = query.execute(&snapshot, &stop)?;
    assert!(!explanation.derivation_complete());
    assert!(
        explanation
            .derivations()
            .iter()
            .any(|record| !record.record.missing.is_empty())
    );
    for invalid in [Value::Null, json!("unknown")] {
        native["signal_witnesses"] = invalid;
        write_json(&root, "bad.json", &native)?;
        assert!(LocalProjectInput::from_config_path(&root.0.join("bad.json"), &stop).is_err());
    }
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    assert_eq!(
        serde_json::to_value(&replay)?["configuration"]["signal_witnesses"],
        "matcher_v1"
    );
    let store_parent_path = root.0.with_extension("signal-witness-store");
    std::fs::create_dir(&store_parent_path)?;
    let store_parent = FixtureRoot(store_parent_path);
    let store_path = store_parent.0.join("store");
    publish_local_project(
        &root.0.join("signal-input.json"),
        &request,
        &store_path,
        &LiveProjectPublishRequest::new("fixture-signal-witnesses", "absent", true, true)?,
        &stop,
    )?;
    // Each selected family is requalified, not merely Graph-valid.
    let mut store = LiveProjectStore::open(&store_path)?;
    let previous = store.current()?.ok_or("current")?;
    for partition_id in [W1_SIGNAL_PARTITION, W3_SIGNAL_PARTITION, W4_PARTITION] {
        let read = store.read(&ReadSelector::Current, &stop)?;
        let graph = read.graph();
        let partition = graph.partition(partition_id).ok_or("family")?;
        let relations = partition
            .report()
            .accepted_relations()
            .iter()
            .map(|accepted| {
                partition
                    .batch()
                    .relation_proposal(accepted.proposal_id())
                    .cloned()
                    .ok_or_else(|| "relation".into())
            })
            .collect::<TestResult<Vec<_>>>()?;
        let batch = wow_graph::GraphProposalBatch::build(
            graph.registry().bundle_id(),
            graph.registry().registry_digest(),
            partition.batch().universe().clone(),
            partition.batch().generation().clone(),
            graph.source_context_id(),
            partition_id,
            partition.batch().entity_proposals().to_vec(),
            relations,
        )?;
        let altered = graph
            .prepare_replacement(
                wow_graph::GraphPartitionReplacement {
                    expected_snapshot_id: graph.snapshot().snapshot_id().clone(),
                    expected_partition_digest: Some(partition.partition_digest().into()),
                    producer_version: partition.producer_version().into(),
                    batch,
                    coverage: partition.coverage().to_vec(),
                },
                &stop,
            )?
            .into_candidate();
        read.close()?;
        let error = store
            .publish(
                &publisher,
                &altered,
                &format!("tamper-{partition_id}"),
                Some(previous.record_id.clone()),
                &stop,
            )
            .err()
            .ok_or("tamper admitted")?;
        assert_eq!(error.code(), crate::ServiceErrorCode::IdentityMismatch);
        assert_eq!(store.current()?, Some(previous.clone()));
    }
    store.close()?;
    drop(input);
    drop(publisher);
    std::fs::remove_dir_all(&root.0)?;
    let hydrated =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(
        hydrated.configuration().signal_witnesses(),
        Some(SignalWitnessRecipe::MatcherV1)
    );
    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        read.project().configuration().signal_witnesses(),
        Some(SignalWitnessRecipe::MatcherV1)
    );
    assert_eq!(read.graph(), &snapshot);
    read.close()?;
    store.close()?;
    Ok(())
}
