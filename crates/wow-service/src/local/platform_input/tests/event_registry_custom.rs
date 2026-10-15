use super::*;

use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
use std::sync::atomic::AtomicBool;
use wow_graph::{GraphConfidence, GraphPartitionSnapshot, GraphProposalValue, GraphRelationKind};
use wow_recognizers::source_signals::{W3_SIGNAL_PARTITION, W3_SIGNAL_PROFILE};

const EVENT_REGISTRY_LIBRARY_SOURCE: &str = r#"
EventRegistry = {}
function EventRegistry:TriggerEvent(event) end
function EventRegistry:RegisterCallback(event, callback, owner) end
"#;

pub(super) fn replace_source_file(
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
            sum.checked_add(file["bytes"].as_u64().ok_or("member bytes")?)
                .ok_or_else(|| "manifest source byte sum".into())
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
        &AtomicBool::new(false),
    )?;
    config["source_profile"]["target"]["reference_profile"] =
        serde_json::to_value(expected.profile())?;
    config["source_profile"]["target"]["reference_generation"] =
        serde_json::to_value(expected.reference_generation())?;
    super::write_json(root, "reference.json", &expected.reference().view)?;
    super::write_json(root, "project.json", config)?;
    Ok(())
}

fn register_library(root: &FixtureRoot, config: &mut Value) -> TestResult {
    let path = "Blizzard_SharedXMLBase/GlobalCallbackRegistry.lua";
    let library_root = "registry-library";
    let file_path = root.0.join(library_root).join(path);
    std::fs::create_dir_all(file_path.parent().ok_or("Library parent path")?)?;
    std::fs::write(&file_path, EVENT_REGISTRY_LIBRARY_SOURCE)?;
    let file = ProjectDiskFile::new(path).with_identity(
        super::digest(EVENT_REGISTRY_LIBRARY_SOURCE.as_bytes()),
        EVENT_REGISTRY_LIBRARY_SOURCE.len() as u64,
    );
    config["library"] = json!({
        "root": library_root,
        "files": [file],
    });
    Ok(())
}

#[test]
fn library_owned_event_registry_preserves_scoped_custom_signal_proofs() -> TestResult {
    use wow_project::{
        PlatformGraphCapabilities, PlatformGraphCapability, PlatformGraphProfile, ProjectPublisher,
    };

    const SHARED: &str = "Signal.Shared";
    const MISSING: &str = "Signal.Missing";

    let (root, mut config) = super::native_fixture()?;
    let capabilities = PlatformGraphCapabilities::new(&[
        PlatformGraphCapability::InventorySpansV1,
        PlatformGraphCapability::StructuralRolesV1,
        PlatformGraphCapability::XmlSourceMapsV1,
        PlatformGraphCapability::CustomSignalRelationsV1,
    ])?;
    config["graph_profile"] =
        serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(capabilities))?;
    register_library(&root, &mut config)?;

    // No Main EventRegistry declaration is present: call receivers must bind
    // to the configured Library source.
    let main = r#"
local function OnEvent(event) end
function Main()
    EventRegistry:TriggerEvent("Signal.Shared")
    EventRegistry:TriggerEvent("Signal.Shared")
    EventRegistry:RegisterCallback("Signal.Shared", OnEvent, EventRegistry)
    EventRegistry:RegisterCallback("Signal.Missing", OnEvent, EventRegistry)
end
"#;
    assert!(!main.contains("EventRegistry ="));
    replace_source_file(&root, &mut config, "UI/main.lua", main)?;

    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let project = publisher.open_current()?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native EventRegistry function-call report")?;

    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(result.status(), GraphReadStatus::Partial);
    let result_json = serde_json::to_value(&result)?;
    let custom = result_json
        .get("custom_recognition")
        .ok_or("Service custom-signal recognition")?;
    assert_eq!(custom["profile"], W3_SIGNAL_PROFILE);
    assert_eq!(custom["analyzer_report_id"], report.analysis_id());

    let producers = custom["producers"]
        .as_array()
        .ok_or("custom-signal producers")?;
    let subscriptions = custom["subscriptions"]
        .as_array()
        .ok_or("custom-signal subscriptions")?;
    let unconfirmed = custom["unconfirmed_subscriptions"]
        .as_array()
        .ok_or("unconfirmed custom-signal subscriptions")?;
    let library_subscription = subscriptions
        .iter()
        .find(|row| row["event_key"] == SHARED)
        .ok_or("Library-owned shared-key subscription")?;
    let library_identity = library_subscription["receiver_identity"]
        .as_str()
        .ok_or("Library receiver identity")?;
    let library_workspace = library_subscription["receiver_workspace_id"]
        .as_str()
        .ok_or("Library receiver workspace")?;
    assert!(
        report
            .library_snapshot_ids()
            .iter()
            .any(|id| id == library_workspace)
    );

    let shared_producers = producers
        .iter()
        .filter(|row| row["event_key"] == SHARED)
        .collect::<Vec<_>>();
    assert_eq!(shared_producers.len(), 2);
    let library_producer_ids = shared_producers
        .iter()
        .filter(|row| row["receiver_identity"] == library_identity)
        .filter_map(|row| row["call_id"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    assert_eq!(library_producer_ids.len(), 2);
    assert!(
        shared_producers
            .iter()
            .all(|row| row["receiver_identity"] == library_identity)
    );

    let library_subscription_id = library_subscription["call_id"]
        .as_str()
        .ok_or("Library subscriber call ID")?;
    for (call_id, expected_role, expected_workspace) in library_producer_ids
        .iter()
        .map(|id| (id.as_str(), "library", library_workspace))
        .chain(std::iter::once((
            library_subscription_id,
            "library",
            library_workspace,
        )))
    {
        let call = report
            .calls()
            .iter()
            .find(|call| call.fact_id() == call_id)
            .ok_or("Library-owned W3 call in analyzer report")?;
        let receiver = report
            .exact_call_receiver(call)
            .ok_or("exact Library EventRegistry receiver")?;
        assert_eq!(receiver.key(), "EventRegistry");
        assert_eq!(receiver.target().role, expected_role);
        assert_eq!(receiver.target().workspace_id, expected_workspace);
    }
    let mut expected_ids = library_producer_ids.clone();
    expected_ids.sort();
    let mut actual_ids = library_subscription["producer_call_ids"]
        .as_array()
        .ok_or("all matching Library producer proofs")?
        .iter()
        .filter_map(|id| id.as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    actual_ids.sort();
    assert_eq!(actual_ids, expected_ids);
    assert_eq!(library_subscription["confirmed"], true);
    assert_eq!(
        library_subscription["producer_call_id"].as_str(),
        expected_ids.first().map(String::as_str)
    );
    assert_eq!(unconfirmed.len(), 1);
    let missing = unconfirmed
        .iter()
        .find(|row| row["event_key"] == MISSING)
        .ok_or("different-key unconfirmed control")?;
    assert_eq!(missing["receiver_identity"], library_identity);
    assert_eq!(missing["reason"], "producer_event_key_mismatch");
    let missing_subscription = subscriptions
        .iter()
        .find(|row| row["event_key"] == MISSING)
        .ok_or("unconfirmed subscription match")?;
    assert_eq!(missing_subscription["confirmed"], false);
    assert!(missing_subscription.get("producer_call_id").is_none());
    assert!(
        missing_subscription["producer_call_ids"]
            .as_array()
            .is_some_and(Vec::is_empty)
    );

    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(
        &result
            .snapshot_bytes()?
            .ok_or("final Service graph snapshot")?,
    )?;
    let partition = snapshot
        .partition(W3_SIGNAL_PARTITION)
        .ok_or("final W3 custom-signal graph partition")?;
    let shared_entities = partition
        .report()
        .accepted_entities()
        .iter()
        .filter_map(|accepted| {
            let proposal = partition.batch().entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("signal") {
                Some(GraphProposalValue::String(value))
                    if value.splitn(3, '|').nth(2) == Some(SHARED) =>
                {
                    Some((value, accepted))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(shared_entities.len(), 1);
    assert_eq!(
        shared_entities[0].0.split('|').next(),
        Some(library_identity)
    );
    let library_entity = shared_entities
        .iter()
        .find(|(key, _)| key.split('|').next() == Some(library_identity))
        .ok_or("Library-owned shared signal entity")?;
    assert!(library_entity.1.node().evidence_ids().len() >= 3);

    let subscription_relation_id = library_subscription["relation_proposal_id"]
        .as_str()
        .ok_or("Library subscription relation proposal ID")?;
    let subscription_relation = partition
        .report()
        .accepted_relations()
        .iter()
        .find(|accepted| accepted.proposal_id() == subscription_relation_id)
        .ok_or("Library subscription relation in final graph")?;
    let relation_proposal = partition
        .batch()
        .relation_proposal(subscription_relation.proposal_id())
        .ok_or("Library subscription relation proposal")?;
    assert_eq!(
        relation_proposal.relation_kind_id(),
        "source_subscribes_custom_signal"
    );
    assert_eq!(
        subscription_relation.edge().confidence(),
        GraphConfidence::Derived
    );
    assert!(subscription_relation.edge().evidence_ids().len() >= 3);
    let missing_relation_id = missing_subscription["relation_proposal_id"]
        .as_str()
        .ok_or("unconfirmed subscription relation proposal ID")?;
    let missing_relation = partition
        .report()
        .accepted_relations()
        .iter()
        .find(|accepted| accepted.proposal_id() == missing_relation_id)
        .ok_or("unconfirmed subscription relation in final graph")?;
    assert_eq!(
        missing_relation.edge().confidence(),
        GraphConfidence::Possible
    );

    for relation in [
        GraphRelationKind::EmitsCustomSignal,
        GraphRelationKind::HandlesCustomSignal,
    ] {
        let coverage = partition
            .coverage()
            .iter()
            .find(|coverage| coverage.relation() == relation)
            .ok_or("W3 bounded relation coverage")?;
        assert_eq!(coverage.state(), wow_graph::GraphCoverageState::Partial);
        assert!(!coverage.negative_authority());
    }
    Ok(())
}
