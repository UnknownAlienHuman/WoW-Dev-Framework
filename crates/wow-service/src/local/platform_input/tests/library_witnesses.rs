use super::*;
use crate::graph::{GraphBuildRequest, execute_graph_build};
use crate::live_project::{LiveProjectPublishRequest, LiveProjectStore, publish_local_project};
use std::sync::atomic::AtomicBool;
use wow_graph::{GraphExplainLimits, GraphExplainQuery, GraphExplainSubject};
use wow_project::{LibraryWitnessRecipe, ProjectPublisher, replay::ProjectReplay};
use wow_recognizers::source_state::{
    SOURCE_STATE_LIBRARY_PARTITION, SourceLibraryInput, recognize_source_library_witnesses,
};
use wow_store::project::ReadSelector;

#[test]
fn original_library_witnesses_survive_native_publication_and_source_absent_reopen() -> TestResult {
    let (root, mut config) = native_fixture()?;
    let main = "function LibStub(name) return {} end\n\
        function First() return LibStub(\"Fixture-Library\") end\n\
        function Second() return LibStub(\"Fixture-Library\") end\n\
        function Other() return LibStub(\"Different-Library\") end\n";
    event_registry_custom::replace_source_file(&root, &mut config, "UI/main.lua", main)?;
    let mut native = json!({
        "schema": crate::local::LOCAL_NATIVE_SCHEMA,
        "project_id": config["project_id"], "workspace_id": config["workspace_id"],
        "source_origin_id": config["source_origin_id"], "logical_root": config["logical_root"],
        "profile": config["native_reference"]["profile"], "analyzer": config["analyzer"],
        "main": {"root": "UI", "files": [ProjectDiskFile::new("main.lua")]},
        "native_source": config["native_reference"]["native_source"]
    });
    let stop = AtomicBool::new(false);
    write_json(&root, "legacy.json", &native)?;
    let legacy = LocalProjectInput::from_config_path(&root.0.join("legacy.json"), &stop)?;
    let legacy_request =
        GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
            .with_input_profile(&legacy);
    let legacy_result = execute_graph_build(legacy, &legacy_request, &stop)?;
    let legacy_json = serde_json::to_value(&legacy_result)?;
    assert!(
        legacy_json["library_recognition"]["matches"]
            .as_array()
            .ok_or("legacy matches")?
            .iter()
            .all(|receipt| receipt.get("witness").is_none())
    );
    native["library_witnesses"] = json!("matcher_v1");
    write_json(&root, "selected.json", &native)?;
    let input = LocalProjectInput::from_config_path(&root.0.join("selected.json"), &stop)?;
    assert_eq!(
        input.bundle.configuration().library_witnesses(),
        Some(LibraryWitnessRecipe::MatcherV1)
    );
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&root.0.join("selected.json"), &stop)?,
        &request,
        &stop,
    )?;
    let result_json = serde_json::to_value(&result)?;
    let receipts = result_json["library_recognition"]["matches"]
        .as_array()
        .ok_or("original witnesses")?;
    let materialized = result.snapshot_bytes()?.ok_or("selected snapshot")?;
    let selected_owner: wow_graph::GraphPartitionSnapshot = serde_json::from_slice(&materialized)?;
    let subject = selected_owner
        .snapshot()
        .nodes()
        .iter()
        .find(|node| node.kind() == "library")
        .ok_or("library subject")?;
    let query = crate::graph::GraphReadRequest::new(crate::graph::GraphReadQuery::Explain(
        GraphExplainQuery::new(
            selected_owner.snapshot().snapshot_id().clone(),
            GraphExplainSubject::Entity(subject.node_id().clone()),
            GraphExplainLimits::default(),
        )?,
    ));
    let query_bytes = serde_json::to_vec(&query)?;
    let imported = crate::graph::execute_graph_bundle_read(
        crate::graph::GraphReadOperation::Explain,
        &result.canonical_bytes()?,
        &query_bytes,
        &stop,
    )?;
    assert_ne!(imported.status(), crate::graph::GraphReadStatus::Failed);
    let mut wrong_selector = result_json.clone();
    wrong_selector["request"]["library_witnesses"] = json!("unknown");
    wrong_selector["request_digest"] = json!(format!(
        "sha256:{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&wrong_selector["request"])?)
    ));
    wrong_selector
        .as_object_mut()
        .ok_or("bundle object")?
        .remove("result_digest");
    wrong_selector["result_digest"] = json!(format!(
        "sha256:{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&wrong_selector)?)
    ));
    let refused = crate::graph::execute_graph_bundle_read(
        crate::graph::GraphReadOperation::Explain,
        &serde_json::to_vec(&wrong_selector)?,
        &query_bytes,
        &stop,
    )?;
    assert_eq!(refused.status(), crate::graph::GraphReadStatus::Failed);
    assert_eq!(receipts.len(), 3);
    let shared = receipts
        .iter()
        .filter(|receipt| receipt["library_name"] == "Fixture-Library")
        .collect::<Vec<_>>();
    assert_eq!(shared.len(), 2);
    assert_eq!(
        shared[0]["witness"]["entity_proposal_id"],
        shared[1]["witness"]["entity_proposal_id"]
    );
    assert_ne!(
        shared[0]["witness"]["matched"]["match_id"],
        shared[1]["witness"]["matched"]["match_id"]
    );
    assert_ne!(
        shared[0]["witness"]["matched"]["decisive_fact_ids"],
        shared[1]["witness"]["matched"]["decisive_fact_ids"]
    );
    let mut publisher = ProjectPublisher::with_function_call_facts();
    publisher.publish_initial_cancellable(input.bundle.clone(), &stop)?;
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    assert_eq!(
        serde_json::to_value(&replay)?["configuration"]["library_witnesses"],
        "matcher_v1"
    );
    for invalid in [Value::Null, json!("unknown")] {
        let mut bad = native.clone();
        bad["library_witnesses"] = invalid;
        write_json(&root, "bad.json", &bad)?;
        assert!(LocalProjectInput::from_config_path(&root.0.join("bad.json"), &stop).is_err());
    }
    let store_parent_path = root.0.with_extension("library-witness-store");
    std::fs::create_dir(&store_parent_path)?;
    let store_parent = FixtureRoot(store_parent_path);
    let store_path = store_parent.0.join("store");
    publish_local_project(
        &root.0.join("selected.json"),
        &request,
        &store_path,
        &LiveProjectPublishRequest::new("fixture-library-witnesses", "absent", true, true)?,
        &stop,
    )?;
    // A generic graph may omit producer derivations. That is not admission for
    // a publication whose exact configuration requires the matcher-witness recipe.
    let mut held_store = LiveProjectStore::open(&store_path)?;
    let previous = held_store.current()?.ok_or("selected publication")?;
    let held = held_store.read(&ReadSelector::Current, &stop)?;
    let graph = held.graph();
    let original = graph
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("selected partition")?;
    let without_records = wow_graph::GraphProposalBatch::build(
        graph.registry().bundle_id(),
        graph.registry().registry_digest(),
        original.batch().universe().clone(),
        original.batch().generation().clone(),
        graph.source_context_id(),
        SOURCE_STATE_LIBRARY_PARTITION,
        original.batch().entity_proposals().to_vec(),
        original
            .report()
            .accepted_relations()
            .iter()
            .map(|accepted| {
                original
                    .batch()
                    .relation_proposal(accepted.proposal_id())
                    .cloned()
                    .ok_or_else(|| "accepted W6 relation proposal absent".into())
            })
            .collect::<TestResult<Vec<_>>>()?,
    )?;
    let altered = graph
        .prepare_replacement(
            wow_graph::GraphPartitionReplacement {
                expected_snapshot_id: graph.snapshot().snapshot_id().clone(),
                expected_partition_digest: Some(original.partition_digest().into()),
                producer_version: original.producer_version().into(),
                batch: without_records,
                coverage: original.coverage().to_vec(),
            },
            &stop,
        )?
        .into_candidate();
    held.close()?;
    let error = held_store
        .publish(
            &publisher,
            &altered,
            "fixture-library-witness-tamper",
            Some(previous.record_id.clone()),
            &stop,
        )
        .err()
        .ok_or("removed derivations activated")?;
    assert_eq!(error.code(), crate::ServiceErrorCode::IdentityMismatch);
    assert_eq!(held_store.current()?, Some(previous));
    held_store.close()?;
    drop(input);
    drop(publisher);
    std::fs::remove_dir_all(&root.0)?;
    let hydrated =
        ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?.hydrate(&stop)?;
    assert_eq!(
        hydrated.configuration().library_witnesses(),
        Some(LibraryWitnessRecipe::MatcherV1)
    );
    let store = LiveProjectStore::open(&store_path)?;
    let read = store.read(&ReadSelector::Current, &stop)?;
    assert_eq!(
        read.project().configuration().library_witnesses(),
        Some(LibraryWitnessRecipe::MatcherV1)
    );
    let (_, _, _, provenance, _) =
        wow_project::graph::build_source_graph_proposals(read.project(), &stop)?.into_parts();
    let rebuilt = recognize_source_library_witnesses(
        SourceLibraryInput {
            owner: read.graph(),
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report: provenance
                .function_call_report()
                .ok_or("retained call report")?,
            context: provenance.context(),
            function_proposals: provenance
                .functions()
                .iter()
                .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
                .collect(),
            call_support: provenance
                .call_sites()
                .iter()
                .map(|site| {
                    (
                        site.call_id.as_str(),
                        (site.source_handle_id, site.evidence_id),
                    )
                })
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        &stop,
    )?;
    let partition = read
        .graph()
        .partition(SOURCE_STATE_LIBRARY_PARTITION)
        .ok_or("retained W6")?;
    assert_eq!(&rebuilt.batch, partition.batch());
    assert_eq!(
        serde_json::to_value(&rebuilt.recognition)?,
        result_json["library_recognition"]
    );
    assert_eq!(partition.report().accepted_entities().len(), 2);
    assert_eq!(partition.report().accepted_relations().len(), 3);
    let entity_id = shared[0]["witness"]["entity_proposal_id"]
        .as_str()
        .ok_or("shared ID")?;
    let accepted = partition
        .report()
        .accepted_entities()
        .iter()
        .find(|entity| entity.proposal_id() == entity_id)
        .ok_or("shared entity")?;
    assert_eq!(accepted.node().evidence_ids().len(), 2);
    let node = read
        .graph()
        .snapshot()
        .nodes()
        .iter()
        .find(|node| {
            node.kind() == accepted.node().kind() && node.owner_key() == accepted.node().owner_key()
        })
        .ok_or("materialized shared entity")?;
    let explanation = GraphExplainQuery::new(
        read.graph().snapshot().snapshot_id().clone(),
        GraphExplainSubject::Entity(node.node_id().clone()),
        GraphExplainLimits::default(),
    )?
    .execute(read.graph(), &stop)?;
    assert_eq!(
        explanation
            .derivations()
            .iter()
            .filter(|item| item.producer.partition_id == SOURCE_STATE_LIBRARY_PARTITION)
            .count(),
        3
    );
    assert!(
        explanation.derivations().len() > 3,
        "original caller derivations remain present"
    );
    drop(explanation);
    read.close()?;
    store.close()?;
    Ok(())
}
