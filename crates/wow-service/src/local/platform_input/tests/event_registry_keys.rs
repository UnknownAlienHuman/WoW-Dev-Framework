use super::*;

fn replace_main_source(root: &FixtureRoot, config: &mut Value, main: &str) -> TestResult {
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"]
        .as_array_mut()
        .ok_or("fixture manifest files")?;
    let member = files
        .iter_mut()
        .find(|file| file["path"] == "UI/main.lua")
        .ok_or("fixture Main manifest member")?;
    member["bytes"] = json!(main.len());
    member["content_sha256"] = json!(format!("{:x}", Sha256::digest(main.as_bytes())));
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("member bytes")?)
                .ok_or("manifest source byte sum")?)
        })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("fixture manifest")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    std::fs::write(root.0.join("UI/main.lua"), main)?;
    let manifest_file = serde_json::to_value(super::write_json(root, "manifest.json", &manifest)?)?;
    config["source"]["manifest"] = manifest_file.clone();
    config["native_reference"]["native_source"]["manifest"] = manifest_file;
    let mut reference_input = config["native_reference"].clone();
    reference_input["schema"] = json!(crate::local::SOURCE_REFERENCE_INPUT_SCHEMA);
    super::write_json(root, "source-reference.json", &reference_input)?;
    let expected = crate::local::read_local_source_reference(
        &root.0.join("source-reference.json"),
        &std::sync::atomic::AtomicBool::new(false),
    )?;
    config["source_profile"]["target"]["reference_profile"] =
        serde_json::to_value(expected.profile())?;
    config["source_profile"]["target"]["reference_generation"] =
        serde_json::to_value(expected.reference_generation())?;
    super::write_json(root, "reference.json", &expected.reference().view)?;
    super::write_json(root, "project.json", config)?;
    Ok(())
}

fn custom_relation<'a>(
    partition: &'a wow_graph::GraphProducerPartition,
    match_row: &Value,
) -> TestResult<&'a wow_graph::GraphAcceptedRelationProposal> {
    let relation_id = match_row["relation_proposal_id"]
        .as_str()
        .ok_or("subscription relation proposal ID")?;
    partition
        .report()
        .accepted_relations()
        .iter()
        .find(|accepted| accepted.proposal_id() == relation_id)
        .ok_or_else(|| "accepted custom-signal subscription relation".into())
}

#[test]
fn native_service_graph_preserves_exact_eventregistry_literal_keys() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    use std::sync::atomic::AtomicBool;
    use wow_graph::{GraphConfidence, GraphPartitionSnapshot, GraphProposalValue};
    use wow_recognizers::{source_bridge::W2_PARTITION, source_signals::W3_SIGNAL_PARTITION};

    let (root, mut config) = super::native_fixture()?;
    let capabilities = wow_project::PlatformGraphCapabilities::new(&[
        wow_project::PlatformGraphCapability::InventorySpansV1,
        wow_project::PlatformGraphCapability::StructuralRolesV1,
        wow_project::PlatformGraphCapability::XmlSourceMapsV1,
        wow_project::PlatformGraphCapability::CustomSignalRelationsV1,
    ])?;
    config["graph_profile"] = serde_json::to_value(
        wow_project::PlatformGraphProfile::DirectCapabilitiesV1(capabilities),
    )?;
    // RegisterCallback's real owner argument carries this same Main receiver into Project's declaration crosswalk.
    let oversized_key = "E".repeat(1_025);
    let oversized_call = format!("    EventRegistry:TriggerEvent(\"{oversized_key}\")\n");
    let main = r#"EventRegistry = {}
function EventRegistry:TriggerEvent(key) end
function EventRegistry:RegisterCallback(key, callback) end
function EventRegistry:RegisterFrameEventAndCallback(key, callback) end
local function onEvent() end
function Main()
    EventRegistry:TriggerEvent("Audit.Equal")
    EventRegistry:RegisterCallback("Audit.Equal", onEvent, EventRegistry)
    EventRegistry:TriggerEvent(" Audit.Leading")
    EventRegistry:RegisterCallback("Audit.Leading", onEvent, EventRegistry)
    EventRegistry:TriggerEvent("Audit.Trailing ")
    EventRegistry:RegisterCallback("Audit.Trailing", onEvent, EventRegistry)
    EventRegistry:TriggerEvent("")
    EventRegistry:RegisterCallback("", onEvent, EventRegistry)
    EventRegistry:RegisterFrameEventAndCallback(" Native.Trim", onEvent)
    EventRegistry:RegisterFrameEventAndCallback("Native.Trim ", onEvent)
    -- W6 oversized unrelated event argument
end
"#
    .replace(
        "    -- W6 oversized unrelated event argument\n",
        &oversized_call,
    );
    replace_main_source(&root, &mut config, &main)?;

    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{}; result={}",
        result
            .failure_message()
            .unwrap_or("graph did not retain a failure message"),
        serde_json::to_string(&result)?
    );
    let result_json = serde_json::to_value(&result)?;
    let library_matches = result_json
        .get("library_recognition")
        .and_then(|recognition| recognition.get("matches"))
        .and_then(Value::as_array)
        .ok_or("native library recognition matches")?;
    assert!(library_matches.iter().all(|row| {
        row.get("library_name").and_then(Value::as_str) != Some(oversized_key.as_str())
    }));
    let custom = result_json
        .get("custom_recognition")
        .ok_or("Service custom-signal recognition")?;
    assert_eq!(
        custom["profile"],
        wow_recognizers::source_signals::W3_SIGNAL_PROFILE
    );
    let producers = custom["producers"]
        .as_array()
        .ok_or("custom-signal producers")?;
    let subscriptions = custom["subscriptions"]
        .as_array()
        .ok_or("custom-signal subscriptions")?;
    assert_eq!(producers.len(), 3);
    assert_eq!(subscriptions.len(), 3);
    assert!(subscriptions.iter().all(|row| {
        row.get("callback_proposal_id").is_none() && row.get("owner_proposal_id").is_none()
    }));
    assert_eq!(
        custom["unconfirmed_subscriptions"]
            .as_array()
            .ok_or("unconfirmed custom-signal subscriptions")?
            .len(),
        2
    );

    let equal = subscriptions
        .iter()
        .find(|row| row["event_key"] == "Audit.Equal")
        .ok_or("equal-key subscription")?;
    let leading = subscriptions
        .iter()
        .find(|row| row["event_key"] == "Audit.Leading")
        .ok_or("leading-space subscription")?;
    let trailing = subscriptions
        .iter()
        .find(|row| row["event_key"] == "Audit.Trailing")
        .ok_or("trailing-space subscription")?;
    assert_eq!(equal["confirmed"], true);
    assert!(equal["producer_call_id"].as_str().is_some());
    assert_eq!(leading["confirmed"], false);
    assert!(leading.get("producer_call_id").is_none());
    assert_eq!(trailing["confirmed"], false);
    assert!(trailing.get("producer_call_id").is_none());

    let snapshot: GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("native graph snapshot")?)?;
    assert_eq!(snapshot.registry().version(), capabilities.version_id());
    let library_partition = snapshot
        .partition(wow_recognizers::source_state::SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("native library graph partition")?;
    assert!(
        library_partition
            .coverage()
            .iter()
            .all(|record| record.state() != wow_graph::GraphCoverageState::Complete)
    );
    let custom_partition = snapshot
        .partition(W3_SIGNAL_PARTITION)
        .ok_or("custom-signal graph partition")?;
    assert_eq!(custom_partition.report().accepted_entities().len(), 5);
    assert_eq!(custom_partition.report().accepted_relations().len(), 6);
    let custom_keys = custom_partition
        .report()
        .accepted_entities()
        .iter()
        .filter_map(|accepted| {
            let proposal = custom_partition
                .batch()
                .entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("signal") {
                Some(GraphProposalValue::String(value)) => Some(value.to_string()),
                _ => None,
            }
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        custom_keys,
        std::collections::BTreeSet::from([
            "Audit.Equal".to_owned(),
            " Audit.Leading".to_owned(),
            "Audit.Leading".to_owned(),
            "Audit.Trailing ".to_owned(),
            "Audit.Trailing".to_owned(),
        ])
    );
    assert_eq!(
        custom_relation(custom_partition, equal)?
            .edge()
            .confidence(),
        GraphConfidence::Derived
    );
    assert_eq!(
        custom_relation(custom_partition, leading)?
            .edge()
            .confidence(),
        GraphConfidence::Possible
    );
    assert_eq!(
        custom_relation(custom_partition, trailing)?
            .edge()
            .confidence(),
        GraphConfidence::Possible
    );

    let bridges = result_json["bridge_recognition"]["bridges"]
        .as_array()
        .ok_or("native EventRegistry bridges")?;
    assert_eq!(bridges.len(), 2);
    assert!(bridges.iter().all(|row| {
        [
            "frame_declaration_proposal_id",
            "callback_declaration_proposal_id",
        ]
        .iter()
        .all(|key| row.get(*key).is_none_or(|value| value.as_str().is_some()))
    }));
    assert!(bridges.iter().any(|row| row["event_key"] == " Native.Trim"));
    assert!(bridges.iter().any(|row| row["event_key"] == "Native.Trim "));
    let bridge_partition = snapshot
        .partition(W2_PARTITION)
        .ok_or("native-event bridge graph partition")?;
    assert_eq!(bridge_partition.report().accepted_entities().len(), 2);
    let bridge_keys = bridge_partition
        .report()
        .accepted_entities()
        .iter()
        .filter_map(|accepted| {
            let proposal = bridge_partition
                .batch()
                .entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("event") {
                Some(GraphProposalValue::String(value)) => Some(value.to_string()),
                _ => None,
            }
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        bridge_keys,
        std::collections::BTreeSet::from([" Native.Trim".to_owned(), "Native.Trim ".to_owned(),])
    );
    Ok(())
}
