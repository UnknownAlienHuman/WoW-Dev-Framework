use super::*;

use crate::graph::{GraphBuildRequest, execute_graph_build};
use std::sync::atomic::AtomicBool;
use wow_graph::{
    GraphCoverageState, GraphPartitionSnapshot, GraphProposalValue, GraphRelationKind,
};
use wow_recognizers::{source_bridge::W2_PARTITION, source_signals::W4_PARTITION};

use wow_emmy::function_calls::FunctionCallReport;

const EVENT_REGISTRY_LIBRARY_SOURCE: &str = "EventRegistry = {}\n\
function EventRegistry:RegisterFrameEvent(frameEvent) end\n\
function EventRegistry:RegisterCallback(frameEvent, callback, owner) end\n\
function EventRegistry:RegisterFrameEventAndCallback(frameEvent, callback, owner) end\n\
function EventRegistry:RegisterFrameEventAndCallbackWithHandle(frameEvent, callback, owner) end\n";
const CVAR_REGISTRY_LIBRARY_SOURCE: &str = "CVarCallbackRegistry = {}\n\
function CVarCallbackRegistry:RegisterCallback(cvar, callback, owner) end\n";

struct ServiceGraphWitness {
    result: Value,
    snapshot: GraphPartitionSnapshot,
    analyzer_report_id: String,
    event_registry_workspace_id: String,
    cvar_registry_workspace_id: String,
    function_call_report: FunctionCallReport,
}

impl ServiceGraphWitness {
    fn function_call_report(&self) -> &FunctionCallReport {
        &self.function_call_report
    }
}

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
    let files = [
        (
            "Blizzard_SharedXMLBase/GlobalCallbackRegistry.lua",
            EVENT_REGISTRY_LIBRARY_SOURCE,
        ),
        (
            "Blizzard_SharedXMLBase/CvarUtil.lua",
            CVAR_REGISTRY_LIBRARY_SOURCE,
        ),
    ];
    for (path, source) in files {
        let file_path = root.0.join("registry-library").join(path);
        std::fs::create_dir_all(file_path.parent().ok_or("Library parent path")?)?;
        std::fs::write(file_path, source)?;
    }
    let library_files = files
        .iter()
        .map(|(path, source)| {
            serde_json::to_value(
                ProjectDiskFile::new(*path)
                    .with_identity(digest(source.as_bytes()), source.len() as u64),
            )
            .map_err(Into::into)
        })
        .collect::<TestResult<Vec<_>>>()?;
    config["library"] = json!({"root":"registry-library","files":library_files});
    Ok(())
}

fn service_graph(
    main_source: &str,
    expected_receiver_role: &str,
) -> TestResult<ServiceGraphWitness> {
    let (root, mut config) = super::native_fixture()?;
    let capabilities = PlatformGraphCapabilities::new(&[
        PlatformGraphCapability::InventorySpansV1,
        PlatformGraphCapability::StructuralRolesV1,
        PlatformGraphCapability::XmlSourceMapsV1,
        PlatformGraphCapability::CustomSignalRelationsV1,
    ])?;
    config["graph_profile"] = serde_json::to_value(
        wow_project::PlatformGraphProfile::DirectCapabilitiesV1(capabilities),
    )?;
    register_library(&root, &mut config)?;
    replace_source_file(&root, &mut config, "UI/main.lua", main_source)?;

    let stop = AtomicBool::new(false);
    let input = LocalProjectInput::from_config_path(&root.0.join("project.json"), &stop)?;
    let mut publisher = wow_project::ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let project = publisher.open_current()?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native Library function-call report")?;
    let analyzer_report_id = report.analysis_id().to_owned();
    let main_snapshot_id = report.main_snapshot_id().to_owned();

    let receiver = |callable_key: &str| -> TestResult<String> {
        let call = report
            .calls()
            .iter()
            // The exact resolved Main callback form is the one whose second argument
            // resolves to a Main declaration; the negative controls do not establish it and
            // are intentionally skipped here (their absence from a bridge is asserted
            // separately below).
            .find(|call| {
                call.resolved_callable_key() == Some(callable_key)
                    && call
                        .arguments()
                        .get(1)
                        .and_then(|argument| argument.reference_target())
                        .is_some_and(|target| target.role == "main")
            })
            .ok_or_else(|| {
                let observed = report
                    .calls()
                    .iter()
                    .filter_map(|call| call.resolved_callable_key())
                    .collect::<Vec<_>>();
                format!("native call missing: {callable_key}; observed: {observed:?}")
            })?;
        let receiver = report
            .exact_call_receiver(call)
            .ok_or_else(|| format!("exact receiver missing: {callable_key}"))?;
        let target = receiver.target();
        assert_eq!(
            receiver.key(),
            callable_key.split('.').next().unwrap_or_default()
        );
        assert_eq!(target.role, expected_receiver_role);
        if expected_receiver_role == "library" {
            assert!(report.library_snapshot_ids().contains(&target.workspace_id));
        } else {
            assert_eq!(target.workspace_id, main_snapshot_id);
        }
        let callback = call
            .arguments()
            .get(1)
            .and_then(|argument| argument.reference_target())
            .ok_or_else(|| format!("exact Main callback target missing: {callable_key}"))?;
        assert_eq!(callback.role, "main");
        assert_eq!(callback.workspace_id, main_snapshot_id);
        Ok(target.workspace_id.clone())
    };
    let event_registry_workspace_id = receiver("EventRegistry.RegisterFrameEventAndCallback")?;
    let cvar_registry_workspace_id = receiver("CVarCallbackRegistry.RegisterCallback")?;
    assert_eq!(event_registry_workspace_id, cvar_registry_workspace_id);

    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(input, &request, &stop)?;
    let snapshot_bytes = result.snapshot_bytes()?.ok_or("Service graph snapshot")?;
    let snapshot = serde_json::from_slice(&snapshot_bytes)?;
    let result = serde_json::to_value(&result)?;
    assert_eq!(
        result["bridge_recognition"]["analyzer_report_id"],
        analyzer_report_id
    );
    assert_eq!(
        result["cvar_recognition"]["analyzer_report_id"],
        analyzer_report_id
    );
    Ok(ServiceGraphWitness {
        result,
        snapshot,
        analyzer_report_id,
        event_registry_workspace_id,
        cvar_registry_workspace_id,
        function_call_report: report.clone(),
    })
}

fn assert_coverage(
    partition: &wow_graph::GraphProducerPartition,
    relation: GraphRelationKind,
    blocker: &str,
) -> TestResult {
    let coverage = partition
        .coverage()
        .iter()
        .find(|record| record.relation() == relation)
        .ok_or("recognizer relation coverage")?;
    assert_eq!(coverage.state(), GraphCoverageState::Partial);
    assert!(!coverage.negative_authority());
    assert!(
        coverage
            .blocker_ids()
            .iter()
            .any(|id| id.as_ref() == blocker)
    );
    Ok(())
}

fn assert_proposal_evidence(partition: &wow_graph::GraphProducerPartition) {
    for accepted in partition.report().accepted_entities() {
        assert!(!accepted.node().evidence_ids().is_empty());
    }
    for accepted in partition.report().accepted_relations() {
        assert!(!accepted.edge().evidence_ids().is_empty());
    }
}

#[test]
fn library_event_registry_and_cvar_receivers_reach_service_without_main_shadow_authority()
-> TestResult {
    let library_positive = service_graph(
        r#"function OnNativeEvent(event) end
function OnCVarChanged(cvar, value) end
function Main()
    EventRegistry:RegisterFrameEvent("PLAYER_ENTERING_WORLD")
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_LOGOUT")
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_REGEN_DISABLED", dynamic_callback)
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_REGEN_ENABLED", EventRegistry.RegisterFrameEvent)
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_TALENT_MULTI_UPDATE", function(event) end)
    EventRegistry:RegisterFrameEventAndCallbackWithHandle("PLAYER_ALIVE", OnNativeEvent)
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_LOGIN", OnNativeEvent)
    CVarCallbackRegistry:RegisterCallback("test_cvar", OnCVarChanged)
end
"#,
        "library",
    )?;
    let bridges = library_positive.result["bridge_recognition"]["bridges"]
        .as_array()
        .ok_or("Library EventRegistry bridge matches")?;
    assert_eq!(bridges.len(), 1);
    assert_eq!(bridges[0]["event_key"], "PLAYER_LOGIN");
    assert_eq!(
        bridges[0]["registry_workspace_id"],
        library_positive.event_registry_workspace_id
    );
    assert!(
        bridges[0]["registry_target_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("event-registry:sha256:"))
    );
    assert!(
        bridges[0]["callback_declaration_proposal_id"]
            .as_str()
            .is_some()
    );

    let cvar_matches = library_positive.result["cvar_recognition"]["matches"]
        .as_array()
        .ok_or("Library CVar callback matches")?;
    assert_eq!(cvar_matches.len(), 1);
    assert_eq!(cvar_matches[0]["cvar_key"], "test_cvar");
    assert_eq!(
        cvar_matches[0]["receiver_workspace_id"],
        library_positive.cvar_registry_workspace_id
    );
    assert!(
        cvar_matches[0]["receiver_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("receiver:sha256:"))
    );
    assert!(cvar_matches[0]["callback_proposal_id"].as_str().is_some());
    assert_eq!(
        library_positive.event_registry_workspace_id,
        library_positive.cvar_registry_workspace_id
    );

    let bridge_partition = library_positive
        .snapshot
        .partition(W2_PARTITION)
        .ok_or("W2 Service graph partition")?;
    assert_eq!(bridge_partition.report().accepted_entities().len(), 1);
    assert_eq!(bridge_partition.report().accepted_relations().len(), 2);
    let bridge_event = bridge_partition
        .report()
        .accepted_entities()
        .iter()
        .find_map(|accepted| {
            let proposal = bridge_partition
                .batch()
                .entity_proposal(accepted.proposal_id())?;
            match proposal.semantic_key().get("event") {
                Some(GraphProposalValue::String(event)) if event.as_ref() == "PLAYER_LOGIN" => {
                    Some(accepted)
                }
                _ => None,
            }
        })
        .ok_or("accepted exact native event proposal")?;
    assert!(!bridge_event.node().evidence_ids().is_empty());
    assert_proposal_evidence(bridge_partition);
    assert_coverage(
        bridge_partition,
        GraphRelationKind::RegistersNativeEvent,
        "lua_native_event_bridges.exact_registry_receiver_and_event_key_only",
    )?;
    assert_coverage(
        bridge_partition,
        GraphRelationKind::BridgesNativeEvent,
        "lua_native_event_bridges.exact_resolved_callback_only",
    )?;

    let cvar_partition = library_positive
        .snapshot
        .partition(W4_PARTITION)
        .ok_or("W4 Service graph partition")?;
    assert_eq!(cvar_partition.report().accepted_entities().len(), 1);
    assert_eq!(cvar_partition.report().accepted_relations().len(), 1);
    assert_proposal_evidence(cvar_partition);
    assert_coverage(
        cvar_partition,
        GraphRelationKind::RegistersCvarCallback,
        "lua_cvar_callbacks.exact_resolved_registercallback_calls_only",
    )?;

    // The exact AndCallback callback-custody controls are appended to the SAME positive
    // Main source, so Emmy captured each one. Proof that Emmy actually observed each
    // control: every distinct event key is a captured String literal argument, and the
    // WithHandle callable itself is captured. None of them created an extra Derived
    // bridge because the W2 partition still holds exactly one accepted event entity and
    // two relations, and `bridges.len()` stayed 1 above.
    let report = library_positive.function_call_report();
    let captured_event_keys: Vec<&str> = report
        .calls()
        .iter()
        .filter_map(|call| {
            if call.resolved_callable_key() == Some("EventRegistry.RegisterFrameEventAndCallback") {
                match call
                    .arguments()
                    .first()
                    .and_then(|argument| argument.literal())
                {
                    Some(wow_emmy::function_calls::SourceCallLiteral::String(key)) => {
                        Some(key.as_str())
                    }
                    _ => None,
                }
            } else {
                None
            }
        })
        .collect();
    for control_key in [
        "PLAYER_LOGIN",
        "PLAYER_LOGOUT",
        "PLAYER_REGEN_DISABLED",
        "PLAYER_REGEN_ENABLED",
        "PLAYER_TALENT_MULTI_UPDATE",
    ] {
        assert!(
            captured_event_keys.contains(&control_key),
            "Emmy did not capture the control registration-site event key {control_key}"
        );
    }
    assert!(
        report
            .calls()
            .iter()
            .any(
                |call| call.resolved_callable_key() == Some("EventRegistry.RegisterFrameEvent")
                    && matches!(call.arguments().first().and_then(|argument| argument.literal()),
                    Some(wow_emmy::function_calls::SourceCallLiteral::String(key))
                        if key == "PLAYER_ENTERING_WORLD")
            ),
        "Emmy did not capture the plain registration-only control"
    );
    assert!(
        report
            .calls()
            .iter()
            .any(|call| call.resolved_callable_key()
                == Some("EventRegistry.RegisterFrameEventAndCallbackWithHandle")
                && matches!(call.arguments().first().and_then(|argument| argument.literal()),
                    Some(wow_emmy::function_calls::SourceCallLiteral::String(key))
                        if key == "PLAYER_ALIVE")),
        "Emmy did not capture the WithHandle control"
    );
    // The observed WithHandle call produced no additional bridge. The owner
    // coverage above remains Partial with no negative authority; no constructed
    // status value can qualify its unsupported handle semantics.

    let main_shadow = service_graph(
        r#"EventRegistry = {}
function EventRegistry:RegisterFrameEventAndCallback(event, callback, owner) end
CVarCallbackRegistry = {}
function CVarCallbackRegistry:RegisterCallback(cvar, callback, owner) end
function OnNativeEvent(event) end
function OnCVarChanged(cvar, value) end
function Main()
    EventRegistry:RegisterFrameEventAndCallback("PLAYER_LOGIN", OnNativeEvent)
    CVarCallbackRegistry:RegisterCallback("test_cvar", OnCVarChanged)
end
"#,
        "main",
    )?;
    assert!(
        main_shadow.result["bridge_recognition"]["bridges"]
            .as_array()
            .ok_or("Main-shadow bridge matches")?
            .is_empty()
    );
    assert!(
        main_shadow.result["cvar_recognition"]["matches"]
            .as_array()
            .ok_or("Main-shadow CVar matches")?
            .is_empty()
    );
    for (snapshot, partition_id) in [
        (&main_shadow.snapshot, W2_PARTITION),
        (&main_shadow.snapshot, W4_PARTITION),
    ] {
        let partition = snapshot
            .partition(partition_id)
            .ok_or("Main-shadow graph partition")?;
        assert!(partition.report().accepted_entities().is_empty());
        assert!(partition.report().accepted_relations().is_empty());
    }
    assert_eq!(
        main_shadow.analyzer_report_id,
        main_shadow.result["bridge_recognition"]["analyzer_report_id"]
    );
    Ok(())
}
