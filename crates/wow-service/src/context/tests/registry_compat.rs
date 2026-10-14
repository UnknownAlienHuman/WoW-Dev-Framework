//! Reopen the retained source-absent Project generation against this candidate.
use super::*;
use wow_project::load::TocSavedVariableScope;

// Read-back guards from the existing source-absent continuation checkpoint;
// this test reproduces them and never re-blesses values from its own output.
const RETAINED_GENERATION: &str = "project-store-generation:sha256:8bb56e1a142df7bf917ad4a9e204dacb55ba2924c195940697731906014dcf51";
const RETAINED_PACK: &str =
    "context-pack:sha256:98d80864814a557ea69e19f55f4c76d4841b0d17cdcdb3c19ea8c27d0e484774";

#[test]
fn retained_source_absent_project_registry_replays_exact_context_prefix() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let input = scratch.root.join("source-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(inspect::state_consumer_bundle(&input, &stop)?, &stop)?;
    assert_eq!(graph.registry().version(), "15");

    let store_root = scratch.root.join("registered-store");
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let publication = store.publish(
        &publisher,
        &graph,
        "fixture-context-continuation",
        None,
        &stop,
    )?;
    assert_eq!(publication.generation_id.as_str(), RETAINED_GENERATION);
    store.close()?;
    drop(graph);
    drop(publisher);
    std::fs::remove_dir_all(&input)?;

    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    let native = read.pair().graph_read(&stop)?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("retained provenance")?;
    let account = provenance
        .state_roots()
        .iter()
        .find(|root| root.scope == TocSavedVariableScope::Account)
        .ok_or("retained account declaration")?;
    let path = provenance
        .state_paths()
        .iter()
        .find(|path| {
            path.root_id == account.root_id
                && serde_json::to_value(&path.keys).ok()
                    == Some(json!([{"kind":"string","value":"profile"}]))
        })
        .ok_or("retained account profile path")?;
    let file = provenance
        .files()
        .iter()
        .find(|file| file.path == account.document)
        .ok_or("retained declaration document")?;
    let mut selected = Vec::new();
    for proposal in [&path.proposal_id, &account.proposal_id, &file.proposal_id] {
        selected.push(
            native
                .published_entity(SOURCE_GRAPH_PARTITION, proposal, &stop)?
                .node_id()
                .clone(),
        );
    }
    assert_eq!(selected.iter().collect::<BTreeSet<_>>().len(), 3);
    read.close()?;
    store.close()?;

    let mut host_json = host_value(&store_root, &profile, &reference);
    host_json["limits"]["max_context_work"] = json!(32_000_000);
    host_json["limits"]["max_metadata_bytes"] = json!(16 * 1024 * 1024);
    let host = decode_host(&host_json, &stop)?;
    let request = ContextBuildRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(publication.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(selected[0].as_str())?,
        ContextRenderSelection::None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?
    .with_progressive_expansion(vec![vec![ContextMapRoot::graph_entity(
        selected[1].as_str(),
    )?]])?;
    let result = execute_context_build(&host, &request, &stop)?;
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    assert_eq!(
        envelope["payload"]["store_generation_id"],
        RETAINED_GENERATION
    );
    assert_eq!(
        envelope["payload"]["semantic_pack"]["semantic_pack_id"],
        RETAINED_PACK
    );
    assert!(!input.exists(), "reopen ran after source removal");
    Ok(())
}
