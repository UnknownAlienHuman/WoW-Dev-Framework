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
fn native_library_frame_event_reaches_service_graph_and_readback() -> TestResult {
    use crate::graph::GraphBuildRequest;
    use crate::live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project};
    use std::sync::atomic::AtomicBool;
    use wow_graph::{GraphCoverageState, GraphProposalValue, GraphRelationKind};
    use wow_recognizers::source_signals::W1_SIGNAL_PARTITION;
    use wow_store::project::ReadSelector;

    let (root, mut config) = super::native_fixture()?;
    let api_path = "Blizzard_APIDocumentationGenerated/API.lua";
    let api = r#"APIDocumentation:AddDocumentationTable({Name="Frame",Type="System",Namespace="Frame",Environment="All",Functions={{Name="RegisterEvent",Type="Function",Arguments={{Name="self",Type="Frame",Nilable=false},{Name="event",Type="string",Nilable=false}}}}})
APIDocumentation:AddDocumentationTable({Name="Global",Type="System",Environment="All",Functions={{Name="CreateFrame",Type="Function",Arguments={{Name="frameType",Type="string",Nilable=false}},Returns={{Name="frame",Type="Frame",Nilable=false}}}}})"#;
    replace_source_file(&root, &mut config, api_path, api)?;
    // This synthetic namespace receiver is source-resolved from Library; no client/runtime claim is made.
    let main = r#"function Main()
    CreateFrame("Frame")
    Frame:RegisterEvent("PLAYER_LOGIN")
end
"#;
    replace_source_file(&root, &mut config, "UI/main.lua", main)?;
    let native_manifest = json!({
        "schema": crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id": config["project_id"],
        "workspace_id": config["workspace_id"],
        "source_origin_id": config["source_origin_id"],
        "logical_root": config["logical_root"],
        "profile": config["native_reference"]["profile"],
        "analyzer": config["analyzer"],
        "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
        "native_source": config["native_reference"]["native_source"]
    });
    super::write_json(&root, "native-project.json", &native_manifest)?;
    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("native-project.json"), &stop)?;
    assert!(
        input
            .bundle
            .configuration()
            .native_library_input()
            .is_some()
    );

    let mut publisher = wow_project::ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let project = publisher.open_current()?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native Library function-call report")?;
    let report_id = report.analysis_id().to_owned();
    let report_json = serde_json::to_string(report)?;
    let event_call = report
        .calls()
        .iter()
        .find(|call| {
            call.is_colon_call() && call.resolved_callable_key() == Some("Frame.RegisterEvent")
        })
        .ok_or_else(|| format!("native Frame call missing: {report_json}"))?;
    assert_eq!(
        event_call.resolved_callable_key(),
        Some("Frame.RegisterEvent"),
        "native Frame call was not resolved: {report_json}"
    );
    let event_target = match event_call.target() {
        wow_emmy::function_calls::SourceCallTarget::LibraryFunction { target } => target,
        target => {
            return Err(format!("Frame receiver did not resolve in Library: {target:?}").into());
        }
    };
    assert_eq!(event_target.role, "library");
    assert!(
        report
            .library_snapshot_ids()
            .contains(&event_target.workspace_id)
    );
    let create_call = report
        .calls()
        .iter()
        .find(|call| call.resolved_callable_key() == Some("CreateFrame"))
        .ok_or_else(|| format!("native CreateFrame call missing: {report_json}"))?;
    assert!(matches!(
        create_call.target(),
        wow_emmy::function_calls::SourceCallTarget::LibraryFunction { target }
            if target.role == "library"
                && report.library_snapshot_ids().contains(&target.workspace_id)
    ));
    assert!(matches!(
        event_call
            .arguments()
            .first()
            .and_then(|argument| argument.literal()),
        Some(wow_emmy::function_calls::SourceCallLiteral::String(value))
            if value == "PLAYER_LOGIN"
    ));
    let call_id = event_call.fact_id().to_owned();
    let caller_id = event_call.caller_function_id().to_owned();
    assert!(
        report
            .functions()
            .iter()
            .any(|function| function.fact_id() == caller_id)
    );

    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let store_path = root.0.join("native-w1-store");
    assert!(!store_path.exists());
    let publication = publish_local_project(
        &root.0.join("native-project.json"),
        &request,
        &store_path,
        &LiveProjectPublishRequest::new("fixture-native-w1-positive", "absent", true, true)?,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(publication)?["status"], "activated");
    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    let stored_report = read
        .project()
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("read-back function-call report")?;
    assert_eq!(stored_report.analysis_id(), report_id);
    assert!(stored_report.calls().iter().any(|call| {
        call.fact_id() == call_id
            && call.caller_function_id() == caller_id
            && call.resolved_callable_key() == Some("Frame.RegisterEvent")
    }));
    let partition = read
        .graph()
        .partition(W1_SIGNAL_PARTITION)
        .ok_or("native W1 graph partition")?;
    let event = partition
        .report()
        .accepted_entities()
        .iter()
        .find_map(|accepted| {
            let proposal = partition.batch().entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("event") {
                Some(GraphProposalValue::String(event)) if event.as_ref() == "PLAYER_LOGIN" => {
                    Some((accepted, proposal))
                }
                _ => None,
            }
        })
        .ok_or("W1 event entity read back from the Service graph")?;
    let (event, event_proposal) = event;
    assert_eq!(event_proposal.entity_kind_id(), "native_event");
    assert!(!event.proposal_id().is_empty());
    let handle_coverage = partition
        .coverage()
        .iter()
        .find(|coverage| coverage.relation() == GraphRelationKind::HandlesNativeEvent)
        .ok_or("W1 HandlesNativeEvent coverage")?;
    assert_eq!(handle_coverage.state(), GraphCoverageState::NotEvaluated);
    assert!(
        partition
            .report()
            .accepted_relations()
            .iter()
            .all(|accepted| {
                partition
                    .batch()
                    .relation_proposal(accepted.proposal_id())
                    .is_some_and(|proposal| {
                        proposal.relation_kind_id() != "lua_handles_native_event"
                    })
            })
    );
    let relation = partition
        .report()
        .accepted_relations()
        .iter()
        .find_map(|accepted| {
            let proposal = partition
                .batch()
                .relation_proposal(accepted.proposal_id())?;
            (proposal.relation_kind_id() == "lua_registers_native_event").then_some(accepted)
        })
        .ok_or("W1 registration relation read back from the Service graph")?;
    assert!(!relation.edge().evidence_ids().is_empty());
    read.close()?;
    Ok(())
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

#[test]
fn aceaddon_local_result_embed_reaches_service_graph_readback() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    use std::sync::atomic::AtomicBool;
    use wow_graph::{
        GraphAssertionSupport, GraphExplainLimits, GraphExplainQuery, GraphExplainSubject,
        GraphExplainedRecord, GraphPartitionSnapshot, GraphProposalEndpoint, GraphProposalValue,
    };
    use wow_project::disk::ProjectDiskFile;
    use wow_project::{
        PlatformGraphCapabilities, PlatformGraphCapability, PlatformGraphProfile,
        ProjectBudgetPolicy, ProjectPublisher, replay::ProjectReplay,
    };
    use wow_recognizers::source_state::SOURCE_STATE_LIBRARY_PARTITION;

    let (root, mut config) = super::native_fixture()?;
    let capabilities = PlatformGraphCapabilities::new(&[
        PlatformGraphCapability::InventorySpansV1,
        PlatformGraphCapability::StructuralRolesV1,
        PlatformGraphCapability::XmlSourceMapsV1,
        PlatformGraphCapability::CustomSignalRelationsV1,
    ])?;
    config["graph_profile"] =
        serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(capabilities))?;
    let budgets = ProjectBudgetPolicy::fixture_e0()?
        .with_function_call_budget(
            wow_emmy::function_calls::FunctionCallBudget::default()
                .with_global_access_budget(wow_emmy::global_access::GlobalAccessBudget::default()),
        )
        .with_function_call_evidence_profile(
            wow_emmy::function_calls::FunctionCallEvidenceProfile::AceAddonLocalResultV1,
        );
    config["schema"] = json!(crate::local::LOCAL_PLATFORM_BUDGET_SCHEMA);
    config["retained_package_limits"] = serde_json::to_value(
        wow_project::platform_source::PlatformPackageRetentionLimits::new(2, 1024 * 1024)?,
    )?;
    config["project_budgets"] = serde_json::to_value(budgets)?;

    std::fs::create_dir(root.0.join("Libs"))?;
    // Minimal typed Library inventory for exact Emmy API selection; it does not
    // claim a current installed API surface.
    let library = concat!(
        "---@meta _\n",
        "---@class AceAddon\n",
        "AceAddon = {}\n",
        "---@param addon AceAddon\n",
        "---@param libname string\n",
        "---@param silent boolean|nil\n",
        "---@param offset number|nil\n",
        "function AceAddon:EmbedLibrary(addon, libname, silent, offset) end\n",
        "---@class AceAddonLib\n",
        "AceAddonLib = {}\n",
        "---@param addon AceAddon\n",
        "function AceAddonLib:Embed(addon) end\n",
        "---@class LibStub\n",
        "LibStub = {}\n",
        "---@param major string\n",
        "---@param minor integer\n",
        "---@return AceAddon\n",
        "---@return integer\n",
        "function LibStub:NewLibrary(major, minor) end\n",
        "---@param major string\n",
        "---@param silent boolean\n",
        "---@return AceAddonLib\n",
        "function LibStub:GetLibrary(major, silent) end\n",
    );
    std::fs::write(root.0.join("Libs/AceAddon.lua"), library)?;
    config["library"] = json!({
        "root":"Libs",
        "files":[ProjectDiskFile::new("AceAddon.lua")]
    });
    super::write_json(&root, "project.json", &config)?;
    // The focused Main excerpt follows WoWUIDev/Ace3 commit
    // d295b12f8b889a30e86e0e901c5494df4b149c49, AceAddon-3.0.lua blob
    // 00e4e4837ae9b6c77f74a3b3a2a5f1b601b48e63.
    let main = concat!(
        "local MAJOR, MINOR = \"AceAddon-3.0\", 13\n",
        "local AceAddon, oldminor = LibStub:NewLibrary(MAJOR, MINOR)\n",
        "function AceAddon:EmbedLibrary(addon, libname, silent, offset)\n",
        "  local lib = LibStub:GetLibrary(libname, true)\n",
        "  lib:Embed(addon)\n",
        "end\n",
        "MyAddon = {}\n",
        "AceAddon:EmbedLibrary(MyAddon, \"AceConsole-3.0\")\n",
        "local other, minor = LibStub:NewLibrary(\"AceAddon-3.0\", 13)\n",
        "minor:EmbedLibrary(MyAddon, \"Rejected-Slot-1\")\n",
        "local rebound, oldminor2 = LibStub:NewLibrary(\"AceAddon-3.0\", 13)\n",
        "rebound = {}\n",
        "rebound:EmbedLibrary(MyAddon, \"Rejected-Reassigned\")\n",
        "local function_rebound, function_oldminor = LibStub:NewLibrary(MAJOR, MINOR)\n",
        "function function_rebound() end\n",
        "function_rebound:EmbedLibrary(MyAddon, \"Rejected-FunctionStatement\")\n",
    );
    replace_source_file(&root, &mut config, "UI/main.lua", main)?;

    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let replay_json = serde_json::to_value(&replay)?;
    assert_eq!(
        replay_json["schema"],
        "wow-project/native-project-replay/aceaddon-local-result/1"
    );
    let restored =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(
        restored
            .configuration()
            .budget_policy()
            .function_call_evidence_profile(),
        Some(wow_emmy::function_calls::FunctionCallEvidenceProfile::AceAddonLocalResultV1)
    );
    assert_eq!(
        restored
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .and_then(|report| report.local_call_result_profile()),
        Some(wow_emmy::function_calls::FunctionCallEvidenceProfile::AceAddonLocalResultV1)
    );
    let report = restored
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("round-tripped AceAddon function-call report")?;
    assert!(
        report.named_targets().contains_key("AceAddon.EmbedLibrary"),
        "selected {:?}; calls {:?}",
        report.named_targets().keys().collect::<Vec<_>>(),
        report
            .calls()
            .iter()
            .map(|call| (call.resolved_callable_key(), call.target()))
            .collect::<Vec<_>>()
    );
    assert!(!report.named_targets().contains_key("LibStub.EmbedLibrary"));
    let embed_call = report
        .calls()
        .iter()
        .find(|call| {
            call.resolved_callable_key() == Some("AceAddon.EmbedLibrary")
                && call.arguments().get(1).is_some_and(|argument| {
                    matches!(
                        argument.literal(),
                        Some(wow_emmy::function_calls::SourceCallLiteral::String(value))
                            if value == "AceConsole-3.0"
                    )
                })
        })
        .ok_or("typed AceAddon embed call")?;
    let selected_embed_target = report
        .named_targets()
        .get("AceAddon.EmbedLibrary")
        .ok_or("selected AceAddon API target")?;
    assert!(matches!(
        embed_call.target(),
        wow_emmy::function_calls::SourceCallTarget::MainFunction { .. }
            | wow_emmy::function_calls::SourceCallTarget::LibraryFunction { .. }
    ));
    assert_eq!(selected_embed_target, embed_call.target());
    let receiver_binding = report
        .call_result_binding(
            embed_call
                .receiver_binding_id()
                .ok_or("AceAddon local receiver binding")?,
        )
        .ok_or("AceAddon result binding")?;
    assert_eq!(receiver_binding.result_slot(), 0);
    let initializer = report
        .calls()
        .iter()
        .find(|call| call.fact_id() == receiver_binding.initializer_call_fact_id())
        .ok_or("AceAddon receiver initializer call")?;
    assert_eq!(
        initializer.resolved_callable_key(),
        Some("LibStub.NewLibrary")
    );
    assert!(
        report
            .named_targets()
            .get("LibStub.NewLibrary")
            .is_some_and(|selected| selected == initializer.target())
    );
    let function_rebound_call = report
        .calls()
        .iter()
        .find(|call| {
            call.arguments().get(1).is_some_and(|argument| {
                matches!(
                    argument.literal(),
                    Some(wow_emmy::function_calls::SourceCallLiteral::String(value))
                        if value == "Rejected-FunctionStatement"
                )
            })
        })
        .ok_or("function-statement rebinding call")?;
    assert_eq!(function_rebound_call.receiver_binding_id(), None);
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "Service graph build did not complete: {}",
        result
            .failure_message()
            .unwrap_or("graph build returned no failure details")
    );
    let result_json: serde_json::Value = serde_json::from_slice(&result.canonical_bytes()?)?;
    let recognition_json = result_json
        .get("library_recognition")
        .ok_or("W6 recognition in canonical Service result")?;
    assert_eq!(
        recognition_json["profile"],
        "wow-recognizers/source-library/aceaddon-local-result/1"
    );
    let wire_matches = recognition_json["matches"]
        .as_array()
        .ok_or("W6 recognition matches")?;
    assert!(
        wire_matches.iter().any(|matched| {
            matched["rule_id"] == "core.library.libstub_new"
                && matched["library_name"] == "AceAddon-3.0"
                && matched["minor"] == 13
        }),
        "exact local NewLibrary minor remains numeric"
    );
    assert!(
        wire_matches
            .iter()
            .any(|matched| matched.get("minor").is_none()),
        "unknown W6 minor is represented by omission"
    );
    let embed_match = wire_matches
        .iter()
        .find(|matched| matched["relation"] == "lua_embeds_library")
        .ok_or("retained W6 embed match")?;
    assert_eq!(
        embed_match["receiver_binding_id"],
        receiver_binding.binding_id()
    );
    assert_eq!(
        embed_match["initializer_call_fact_id"],
        initializer.fact_id()
    );
    let initializer_source_handle_id = embed_match["initializer_source_handle_id"]
        .as_str()
        .ok_or("retained initializer source-handle identity")?;
    let initializer_evidence_id = embed_match["initializer_evidence_id"]
        .as_str()
        .ok_or("retained initializer evidence identity")?;
    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(
        &result
            .snapshot_bytes()?
            .ok_or("Service graph snapshot readback")?,
    )?;
    let partition = snapshot
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("W6 library graph partition")?;
    let embed_relations = partition
        .report()
        .accepted_relations()
        .iter()
        .filter(|accepted| {
            partition
                .batch()
                .relation_proposal(accepted.proposal_id())
                .is_some_and(|proposal| proposal.relation_kind_id() == "lua_embeds_library")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        embed_relations.len(),
        1,
        "only the exact result-zero call embeds"
    );
    let relation = partition
        .batch()
        .relation_proposal(embed_relations[0].proposal_id())
        .ok_or("W6 embed relation proposal")?;
    let materialized_edges = snapshot
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| {
            edge.relation() == embed_relations[0].edge().relation()
                && edge
                    .evidence_ids()
                    .iter()
                    .any(|evidence| evidence.as_ref() == initializer_evidence_id)
        })
        .collect::<Vec<_>>();
    let mut explained_edge_ids = Vec::new();
    for edge in materialized_edges {
        let candidate = GraphExplainQuery::new(
            snapshot.snapshot().snapshot_id().clone(),
            GraphExplainSubject::Relation(edge.edge_id().clone()),
            GraphExplainLimits::default(),
        )?
        .execute(&snapshot, &stop)?;
        if candidate.supports().iter().any(|support| {
            matches!(
                support,
                GraphAssertionSupport::ProducerRelation { proposal, .. }
                    if proposal.proposal_id() == relation.proposal_id()
            )
        }) {
            explained_edge_ids.push(edge.edge_id().clone());
        }
    }
    assert_eq!(
        explained_edge_ids.len(),
        1,
        "exact W6 proposal identity selects one materialized edge"
    );
    let explanation = GraphExplainQuery::new(
        snapshot.snapshot().snapshot_id().clone(),
        GraphExplainSubject::Relation(explained_edge_ids[0].clone()),
        GraphExplainLimits::default(),
    )?
    .execute(&snapshot, &stop)?;
    let explained_relation = explanation
        .supports()
        .iter()
        .find_map(|support| match support {
            GraphAssertionSupport::ProducerRelation { proposal, .. }
                if proposal.proposal_id() == relation.proposal_id() =>
            {
                Some(proposal)
            }
            _ => None,
        })
        .ok_or("W6 relation support retained by graph explain")?;
    let explained_handles = serde_json::to_value(explained_relation.source_handle_ids())?;
    assert!(
        explained_handles
            .as_array()
            .is_some_and(|handles| handles.contains(&json!(initializer_source_handle_id))),
        "graph explain retains the validated initializer source handle"
    );
    let explained_evidence = serde_json::to_value(explained_relation.evidence_ids())?;
    assert!(
        explained_evidence
            .as_array()
            .is_some_and(|evidence| evidence.contains(&json!(initializer_evidence_id))),
        "graph explain retains the validated initializer evidence"
    );
    let GraphExplainedRecord::Relation { edge, .. } = explanation.record() else {
        return Err("W6 graph explain returned the exact relation record".into());
    };
    assert!(
        edge.evidence_ids()
            .iter()
            .any(|evidence| evidence.as_ref() == initializer_evidence_id)
    );
    let GraphProposalEndpoint::Proposed(entity_id) = relation.endpoints()[1] else {
        return Err("W6 embed relation target is not the exact library entity".into());
    };
    let entity = partition
        .batch()
        .entity_proposal(entity_id)
        .ok_or("W6 embedded library entity")?;
    assert_eq!(
        entity.semantic_key().get("library"),
        Some(&GraphProposalValue::String("AceConsole-3.0".into()))
    );
    assert!(!embed_relations[0].edge().evidence_ids().is_empty());
    assert!(partition.coverage().iter().any(|coverage| {
        coverage.relation() == wow_graph::GraphRelationKind::UsesApi
            && coverage.state() == wow_graph::GraphCoverageState::Partial
    }));
    Ok(())
}

#[test]
fn aceaddon_indeterminate_local_result_does_not_emit_embed() -> TestResult {
    use crate::graph::{GraphBuildRequest, GraphReadStatus, execute_graph_build};
    use std::sync::atomic::AtomicBool;
    use wow_emmy::function_calls::SourceCallTarget;
    use wow_graph::GraphPartitionSnapshot;
    use wow_project::disk::ProjectDiskFile;
    use wow_project::{
        PlatformGraphCapabilities, PlatformGraphCapability, PlatformGraphProfile,
        ProjectBudgetPolicy, ProjectPublisher,
    };
    use wow_recognizers::source_state::SOURCE_STATE_LIBRARY_PARTITION;

    let (root, mut config) = super::native_fixture()?;
    let capabilities = PlatformGraphCapabilities::new(&[
        PlatformGraphCapability::InventorySpansV1,
        PlatformGraphCapability::StructuralRolesV1,
        PlatformGraphCapability::XmlSourceMapsV1,
        PlatformGraphCapability::CustomSignalRelationsV1,
    ])?;
    config["graph_profile"] =
        serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(capabilities))?;
    let budgets = ProjectBudgetPolicy::fixture_e0()?
        .with_function_call_budget(
            wow_emmy::function_calls::FunctionCallBudget::default()
                .with_global_access_budget(wow_emmy::global_access::GlobalAccessBudget::default()),
        )
        .with_function_call_evidence_profile(
            wow_emmy::function_calls::FunctionCallEvidenceProfile::AceAddonLocalResultV1,
        );
    config["schema"] = json!(crate::local::LOCAL_PLATFORM_BUDGET_SCHEMA);
    config["retained_package_limits"] = serde_json::to_value(
        wow_project::platform_source::PlatformPackageRetentionLimits::new(2, 1024 * 1024)?,
    )?;
    config["project_budgets"] = serde_json::to_value(budgets)?;

    std::fs::create_dir(root.0.join("Libs"))?;
    let library = concat!(
        "---@meta _\n",
        "---@class AceAddon\n",
        "---@field EmbedLibrary fun(addon: table, libname: string)\n",
        "AceAddon = {}\n",
        "---@class LibStub\n",
        "LibStub = {}\n",
        "---@param major string\n",
        "---@param minor integer\n",
        "---@return AceAddon\n",
        "---@return integer\n",
        "function LibStub:NewLibrary(major, minor) end\n",
    );
    std::fs::write(root.0.join("Libs/AceAddon.lua"), library)?;
    config["library"] = json!({
        "root":"Libs",
        "files":[ProjectDiskFile::new("AceAddon.lua")]
    });
    super::write_json(&root, "project.json", &config)?;
    let main = concat!(
        "local MAJOR, MINOR = \"AceAddon-3.0\", 13\n",
        "local AceAddon, oldminor = LibStub:NewLibrary(MAJOR, MINOR)\n",
        "MyAddon = {}\n",
        "AceAddon:EmbedLibrary(MyAddon, \"Uncaptured-Signature\")\n",
    );
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
        .ok_or("native uncertain-target function-call report")?;
    assert!(
        !report.named_targets().contains_key("AceAddon.EmbedLibrary"),
        "the native doc-field route does not select an AceAddon callable"
    );
    let call = report
        .calls()
        .iter()
        .find(|call| {
            call.arguments().get(1).is_some_and(|argument| {
                matches!(
                    argument.literal(),
                    Some(wow_emmy::function_calls::SourceCallLiteral::String(value))
                        if value == "Uncaptured-Signature"
                )
            })
        })
        .ok_or("native doc-field call in the function-call report")?;
    assert_eq!(call.target(), &SourceCallTarget::Indeterminate);
    let binding = report
        .call_result_binding(
            call.receiver_binding_id()
                .ok_or("result-zero receiver binding")?,
        )
        .ok_or("result-zero receiver receipt")?;
    assert_eq!(binding.result_slot(), 0);

    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "uncertain API graph build did not complete: {}",
        result
            .failure_message()
            .unwrap_or("graph build returned no failure details")
    );
    let result_json: serde_json::Value = serde_json::from_slice(&result.canonical_bytes()?)?;
    let recognition = result_json
        .get("library_recognition")
        .ok_or("W6 uncertain-target recognition")?;
    assert!(recognition["matches"].as_array().is_some_and(|matches| {
        matches
            .iter()
            .all(|matched| matched["relation"] != "lua_embeds_library")
    }));
    let snapshot: GraphPartitionSnapshot = serde_json::from_slice(
        &result
            .snapshot_bytes()?
            .ok_or("uncertain-target Service graph snapshot")?,
    )?;
    let partition = snapshot
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("uncertain-target W6 partition")?;
    assert!(
        partition
            .report()
            .accepted_relations()
            .iter()
            .all(|accepted| partition
                .batch()
                .relation_proposal(accepted.proposal_id())
                .is_some_and(|proposal| proposal.relation_kind_id() != "lua_embeds_library"))
    );
    Ok(())
}
