use super::*;
use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
use wow_graph::{GraphConfidence, GraphPartitionSnapshot, GraphProposalValue};
use wow_project::{
    PlatformGraphCapabilities, PlatformGraphCapability, PlatformGraphProfile, ProjectPublisher,
};
use wow_recognizers::source_signals::W3_SIGNAL_PARTITION;

const METHODS: &str = r#"
EventRegistry = {}
function EventRegistry:TriggerEvent(event) end
function EventRegistry:RegisterCallback(event, callback, owner) end
"#;
const REGISTRIES: &str = r#"
RegistryA = {TriggerEvent = EventRegistry.TriggerEvent, RegisterCallback = EventRegistry.RegisterCallback}
RegistryB = {TriggerEvent = EventRegistry.TriggerEvent, RegisterCallback = EventRegistry.RegisterCallback}
"#;

#[test]
fn distinct_registries_do_not_cross_confirm_the_same_event_key() -> TestResult {
    assert_receiver_scope(true, false)?;
    assert_receiver_scope(false, false)
}

#[test]
fn distinct_registry_witnesses_keep_main_and_library_proof_scopes() -> TestResult {
    assert_receiver_scope(true, true)?;
    assert_receiver_scope(false, true)
}

fn assert_receiver_scope(library_receivers: bool, witnesses: bool) -> TestResult {
    let (root, mut config) = native_fixture()?;
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
            PlatformGraphCapability::XmlSourceMapsV1,
            PlatformGraphCapability::CustomSignalRelationsV1,
        ])?,
    ))?;
    std::fs::create_dir(root.0.join("registry-library"))?;
    let library = if library_receivers {
        format!("{METHODS}{REGISTRIES}")
    } else {
        METHODS.to_owned()
    };
    std::fs::write(root.0.join("registry-library/Registry.lua"), &library)?;
    let library_file = ProjectDiskFile::new("Registry.lua")
        .with_identity(digest(library.as_bytes()), library.len() as u64);
    config["library"] = json!({"root":"registry-library","files":[library_file]});
    let main = format!(
        "{}{}",
        if library_receivers { "" } else { REGISTRIES },
        r#"
local function OnEvent(event) end
function Main()
    RegistryA:TriggerEvent("Signal.Changed")
    RegistryA:TriggerEvent("Signal.Changed")
    RegistryA:RegisterCallback("Signal.Changed", OnEvent, RegistryA)
    RegistryB:RegisterCallback("Signal.Changed", OnEvent, RegistryB)
end
"#
    );
    event_registry_custom::replace_source_file(&root, &mut config, "UI/main.lua", &main)?;
    let stop = AtomicBool::new(false);
    let mut input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    if witnesses {
        input.bundle = input
            .bundle
            .with_signal_witnesses(wow_project::SignalWitnessRecipe::MatcherV1)?;
    }
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let project = publisher.open_current()?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native call report")?;
    let mut receivers = std::collections::BTreeMap::new();
    for call in report.calls().iter().filter(|call| {
        matches!(
            call.resolved_callable_key(),
            Some("EventRegistry.TriggerEvent" | "EventRegistry.RegisterCallback")
        )
    }) {
        let receiver = report
            .exact_call_receiver(call)
            .ok_or("exact registry receiver")?;
        if library_receivers {
            assert_eq!(receiver.target().role, "library");
            assert!(
                report
                    .library_snapshot_ids()
                    .contains(&receiver.target().workspace_id)
            );
        } else {
            assert_eq!(receiver.target().role, "main");
            assert_eq!(receiver.target().workspace_id, report.main_snapshot_id());
        }
        receivers.insert(receiver.key().to_owned(), receiver.target().clone());
    }
    assert_eq!(receivers.len(), 2);
    assert_ne!(receivers.get("RegistryA"), receivers.get("RegistryB"));

    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(result.status(), GraphReadStatus::Partial);
    let value = serde_json::to_value(&result)?;
    let custom = &value["custom_recognition"];
    assert_eq!(
        custom["profile"],
        if witnesses {
            "wow-recognizers/lua-custom-signals/witness-derivation/1"
        } else {
            wow_recognizers::source_signals::W3_RECEIVER_SIGNAL_IDENTITY_PROFILE
        }
    );
    assert_eq!(custom["analyzer_report_id"], report.analysis_id());
    let producers = custom["producers"].as_array().ok_or("producers")?;
    let subscriptions = custom["subscriptions"].as_array().ok_or("subscriptions")?;
    assert_eq!(producers.len(), 2);
    assert_eq!(subscriptions.len(), 2);
    let confirmed = subscriptions
        .iter()
        .find(|row| row["confirmed"] == true)
        .ok_or("confirmed A")?;
    let unconfirmed = subscriptions
        .iter()
        .find(|row| row["confirmed"] == false)
        .ok_or("unconfirmed B")?;
    assert_ne!(
        confirmed["receiver_identity"],
        unconfirmed["receiver_identity"]
    );
    assert_eq!(confirmed["event_key"], "Signal.Changed");
    assert_eq!(unconfirmed["event_key"], confirmed["event_key"]);
    assert_eq!(
        confirmed["producer_call_ids"]
            .as_array()
            .ok_or("A proofs")?
            .len(),
        2
    );
    assert!(
        unconfirmed["producer_call_ids"]
            .as_array()
            .ok_or("B proofs")?
            .is_empty()
    );
    assert!(
        producers
            .iter()
            .all(|row| row["receiver_identity"] == confirmed["receiver_identity"])
    );

    let snapshot: GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("graph snapshot")?)?;
    let partition = snapshot
        .partition(W3_SIGNAL_PARTITION)
        .ok_or("W3 partition")?;
    let signals = partition
        .report()
        .accepted_entities()
        .iter()
        .filter_map(|accepted| {
            let proposal = partition.batch().entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("signal") {
                Some(GraphProposalValue::String(key))
                    if key.splitn(3, '|').nth(2) == Some("Signal.Changed") =>
                {
                    Some(key)
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(signals.len(), 2);
    assert_ne!(signals[0], signals[1]);
    for (row, confidence) in [
        (
            confirmed,
            if witnesses {
                GraphConfidence::Possible
            } else {
                GraphConfidence::Derived
            },
        ),
        (unconfirmed, GraphConfidence::Possible),
    ] {
        let id = row["relation_proposal_id"]
            .as_str()
            .ok_or("subscription relation")?;
        let accepted = partition
            .report()
            .accepted_relations()
            .iter()
            .find(|accepted| accepted.proposal_id() == id)
            .ok_or("accepted subscription relation")?;
        assert_eq!(accepted.edge().confidence(), confidence);
    }
    let unconfirmed = custom["unconfirmed_subscriptions"]
        .as_array()
        .ok_or("unconfirmed")?;
    assert_eq!(unconfirmed.len(), 1);
    assert_eq!(unconfirmed[0]["reason"], "producer_receiver_mismatch");

    use wow_project::replay::publication::{self, AcquiredProjectPair, ProjectPublicationBundle};
    use wow_store::project::{ProjectStore, PublicationRequest, ReadSelector, RecordCatalog};
    let bundle = ProjectPublicationBundle::build(&publisher, &snapshot, &stop)?;
    let project_schemas = publication::STORAGE_SCHEMA_CATALOGS
        .iter()
        .find(|schemas| schemas.contains(&bundle.replay_storage_schema()))
        .ok_or("registered captured schema catalog")?;
    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    schemas.extend_from_slice(project_schemas);
    let catalog = RecordCatalog::new(
        &schemas,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )?;
    let store_root = root.0.join("cross-registry-store");
    let mut store = ProjectStore::create(
        &store_root,
        snapshot.snapshot().universe().as_str(),
        catalog.clone(),
    )?;
    let (records, bindings) = bundle.into_parts();
    let publication = PublicationRequest::new(
        store.epoch(),
        wow_store::OperationId::new("fixture:cross-registry")?,
        None,
        bindings,
        records,
    )?;
    store.prepare(&publication, &stop)?;
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_dir_all(root.0.join("registry-library"))?;
    drop(store);
    let reopened = ProjectStore::open(&store_root, &catalog)?;
    let read = reopened.read(
        &ReadSelector::Exact(publication.generation().generation_id.clone()),
        &stop,
    )?;
    let acquired = AcquiredProjectPair::read(&read, &stop)?;
    if witnesses {
        assert_eq!(
            acquired.project().configuration().signal_witnesses(),
            Some(wow_project::SignalWitnessRecipe::MatcherV1)
        );
        crate::graph::validate_native_api_reopen(&acquired, &stop)?;
        assert_eq!(
            confirmed["witness"]["corroborating_relations"]
                .as_array()
                .ok_or("all same-receiver proofs")?
                .len(),
            2
        );
        let unresolved = subscriptions
            .iter()
            .find(|row| row["confirmed"] == false)
            .ok_or("unconfirmed receiver")?;
        assert!(
            unresolved["witness"]
                .get("corroborating_relations")
                .is_none()
        );
    }
    assert_eq!(acquired.project().snapshot_id(), project.snapshot_id());
    assert_eq!(
        acquired.graph().snapshot().snapshot_id(),
        snapshot.snapshot().snapshot_id()
    );
    drop(acquired);
    read.close()?;
    Ok(())
}
