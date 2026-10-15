use super::*;

fn replace_source_file(
    root: &FixtureRoot,
    config: &mut Value,
    path: &str,
    source: &str,
) -> TestResult {
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"]
        .as_array_mut()
        .ok_or("fixture manifest files")?;
    let member = files
        .iter_mut()
        .find(|file| file["path"].as_str() == Some(path))
        .ok_or("fixture source manifest member")?;
    member["bytes"] = json!(source.len());
    member["content_sha256"] = json!(format!("{:x}", Sha256::digest(source.as_bytes())));
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
    std::fs::write(root.0.join(path), source)?;
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
fn native_service_graph_keeps_main_registry_shadows_out_of_donor_adapters() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    use std::sync::atomic::AtomicBool;
    use wow_graph::{GraphConfidence, GraphPartitionSnapshot, GraphProposalValue};
    use wow_recognizers::{
        source_bridge::W2_PARTITION,
        source_signals::{W3_SIGNAL_PARTITION, W4_PARTITION},
    };

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
    let main = r#"EventRegistry = {}
function EventRegistry:TriggerEvent(event) end
function EventRegistry:RegisterCallback(event, callback, owner) end
function EventRegistry:RegisterFrameEvent(event) end
function EventRegistry:RegisterFrameEventAndCallback(event, callback, owner) end
function EventRegistry:RegisterFrameEventAndCallbackWithHandle(event, callback, owner) end
CVarCallbackRegistry = {}
function CVarCallbackRegistry:RegisterCallback(cvar, callback, owner) end
local function onEvent() end
function Main()
    EventRegistry:TriggerEvent("Audit.Equal")
    EventRegistry:TriggerEvent("Audit.Equal")
    EventRegistry:RegisterCallback("Audit.Equal", onEvent, EventRegistry)
    EventRegistry:TriggerEvent(" Audit.Leading")
    EventRegistry:RegisterCallback("Audit.Leading", onEvent, EventRegistry)
    EventRegistry:TriggerEvent("Audit.Trailing ")
    EventRegistry:RegisterCallback("Audit.Trailing", onEvent, EventRegistry)
    EventRegistry:TriggerEvent("")
    EventRegistry:RegisterCallback("", onEvent, EventRegistry)
    EventRegistry:RegisterFrameEvent("PLAYER_LOGOUT")
    EventRegistry:RegisterFrameEventAndCallback(" Native.Trim", onEvent)
    EventRegistry:RegisterFrameEventAndCallback(" Native.Trim", onEvent)
    EventRegistry:RegisterFrameEventAndCallback("Native.Trim ", onEvent)
    EventRegistry:RegisterFrameEventAndCallback("Callback.Missing")
    EventRegistry:RegisterFrameEventAndCallbackWithHandle("Native.Handle", onEvent)
    CVarCallbackRegistry:RegisterCallback("cvar.name", onEvent, CVarCallbackRegistry)
    CVarCallbackRegistry:RegisterCallback("cvar.name", onEvent, CVarCallbackRegistry)
end
"#
    .to_owned();
    replace_source_file(&root, &mut config, "UI/main.lua", &main)?;
    let api = r#"APIDocumentation:AddDocumentationTable({Name="Transport",Type="System",Namespace="C_Transport",Environment="All",Functions={{Name="Read",Type="Function",Arguments={{Name="flag",Type="bool",Nilable=false,Default=false}},Returns={{Name="value",Type="number",Nilable=true}}}}})"#;
    replace_source_file(
        &root,
        &mut config,
        "Blizzard_APIDocumentationGenerated/API.lua",
        api,
    )?;

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
    assert_eq!(producers.len(), 4);
    assert_eq!(subscriptions.len(), 3);
    assert!(subscriptions.iter().all(|row| {
        row.get("callback_proposal_id").is_none() && row.get("owner_proposal_id").is_none()
    }));
    let unconfirmed = custom["unconfirmed_subscriptions"]
        .as_array()
        .ok_or("unconfirmed custom-signal subscriptions")?;
    assert_eq!(unconfirmed.len(), 2);
    assert!(
        unconfirmed
            .iter()
            .all(|row| row["reason"] == "producer_event_key_mismatch")
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
    assert!(
        equal["receiver_workspace_id"]
            .as_str()
            .is_some_and(|workspace| !workspace.is_empty())
    );
    assert!(
        equal["receiver_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("receiver:sha256:"))
    );
    let mut expected_producer_call_ids = producers
        .iter()
        .filter(|row| row["event_key"] == "Audit.Equal")
        .filter_map(|row| row["call_id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    expected_producer_call_ids.sort();
    assert_eq!(expected_producer_call_ids.len(), 2);
    let producer_call_ids = equal["producer_call_ids"]
        .as_array()
        .ok_or("all compatible producer call IDs")?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    assert_eq!(producer_call_ids.len(), 2);
    assert_eq!(producer_call_ids, expected_producer_call_ids);
    assert_eq!(
        equal["producer_call_id"].as_str(),
        expected_producer_call_ids.first().map(String::as_str)
    );
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
    assert_eq!(custom_partition.report().accepted_relations().len(), 7);
    let equal_entity = custom_partition
        .report()
        .accepted_entities()
        .iter()
        .find_map(|accepted| {
            let proposal = custom_partition
                .batch()
                .entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("signal") {
                Some(GraphProposalValue::String(key))
                    if key.splitn(3, '|').nth(2) == Some("Audit.Equal") =>
                {
                    Some(accepted)
                }
                _ => None,
            }
        })
        .ok_or("receiver-scoped exact signal entity")?;
    assert!(equal_entity.node().evidence_ids().len() >= 3);
    let custom_keys = custom_partition
        .report()
        .accepted_entities()
        .iter()
        .filter_map(|accepted| {
            let proposal = custom_partition
                .batch()
                .entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("signal") {
                Some(GraphProposalValue::String(value)) => {
                    value.splitn(3, '|').nth(2).map(str::to_owned)
                }
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
    assert!(
        custom_relation(custom_partition, equal)?
            .edge()
            .evidence_ids()
            .len()
            >= 3
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
    assert!(
        bridges.is_empty(),
        "Main shadows are not Blizzard registries"
    );
    let bridge_partition = snapshot
        .partition(W2_PARTITION)
        .ok_or("native-event bridge graph partition")?;
    assert!(bridge_partition.report().accepted_entities().is_empty());
    assert!(bridge_partition.report().accepted_relations().is_empty());

    let cvar = result_json
        .get("cvar_recognition")
        .ok_or("CVar callback recognition")?;
    let cvar_matches = cvar["matches"].as_array().ok_or("CVar callback matches")?;
    assert!(
        cvar_matches.is_empty(),
        "Main CVar shadows are not donor registries"
    );
    let cvar_partition = snapshot
        .partition(W4_PARTITION)
        .ok_or("CVar callback partition")?;
    assert!(cvar_partition.report().accepted_entities().is_empty());
    assert!(cvar_partition.report().accepted_relations().is_empty());
    Ok(())
}
