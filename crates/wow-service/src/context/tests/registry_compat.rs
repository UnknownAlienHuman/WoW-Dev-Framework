//! Reopen a fresh current-catalog publication against the frozen semantic prefix.
use super::*;
use wow_project::load::TocSavedVariableScope;

// The historical checkpoint's physical generation is retained as provenance,
// not an expected result of a fresh catalog/SQLite-runtime-bound publication.
// Semantic owner identities below remain frozen and are never re-blessed.
const HISTORICAL_GENERATION: &str = "project-store-generation:sha256:8bb56e1a142df7bf917ad4a9e204dacb55ba2924c195940697731906014dcf51";
const RETAINED_PACK: &str =
    "context-pack:sha256:98d80864814a557ea69e19f55f4c76d4841b0d17cdcdb3c19ea8c27d0e484774";

// Original state-core-pipeline inputs at e76eb465, before later W7 cases were
// added to that shared fixture (Lua blob 71816c95). These are historical data.
const HISTORICAL_TOC: &str = concat!(
    "## Interface: 120100\n",
    "## Dependencies: None\n",
    "## LoadOnDemand: 1\n",
    "## SavedVariables: StateCoreAccountDB\n",
    "## SavedVariablesPerCharacter: StateCoreCharacterDB\n",
    "state.lua\n",
);
const HISTORICAL_LUA: &str = r#"StateCoreAccountDB = {}
StateCoreCharacterDB = {}

-- Direct literal read and write on the account root.
local function read_account()
  return StateCoreAccountDB.profile and StateCoreAccountDB.profile.name
end

local function write_account()
  StateCoreAccountDB.profile = { name = "fixture" }
  StateCoreAccountDB.settings = {}
  StateCoreAccountDB.settings.volume = 10
end

-- Direct literal write on the character root.
local function write_character()
  StateCoreCharacterDB.gold = 5
end

-- Lexical alias reads: the alias carries the exact root identity.
local function alias_read()
  local account = StateCoreAccountDB
  local character = StateCoreCharacterDB
  return account.profile, character.gold
end

-- Shadowed local: the local binding must not create an edge. This is the
-- honest negative case: no fabricated state path.
local function shadowed_local()
  local StateCoreAccountDB = {}
  return StateCoreAccountDB.profile
end

-- Dynamic key: the resolved prefix is retained but the whole path stays
-- unresolved, so no exact path entity is created.
local function dynamic_key(key)
  StateCoreAccountDB[key] = 1
end
"#;

fn historical_bundle(input: &Path, stop: &AtomicBool) -> TestResult<ProjectInputBundle> {
    use std::collections::BTreeMap;
    use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
    use wow_project::load::TocLoadContext;

    std::fs::create_dir(input)?;
    write_export(input, "StateCore.toc", HISTORICAL_TOC.as_bytes(), 8192)?;
    write_export(input, "state.lua", HISTORICAL_LUA.as_bytes(), 8192)?;
    let base = input_bundle(LUA)?;
    let original = base.configuration();
    let directory = ProjectInputDirectory::open(input)?;
    let (files, plan) = directory
        .read_toc_project_with_context(
            ".",
            &ProjectDiskFile::new("StateCore.toc"),
            original.selected_profile(),
            Some(&TocLoadContext {
                game_types: BTreeMap::from([("retail".into(), true)]),
                family: None,
                game: None,
                text_locale: None,
                location: None,
                environment: None,
            }),
            stop,
        )?
        .into_parts();
    let configuration = ProjectConfigurationBuilder::new(
        original.project_id().clone(),
        original.project_kind(),
        original.selected_profile().clone(),
        original.reference_generation(),
        original.analyzer_binding().clone(),
    )
    .workspace_id(original.workspace_id().clone())
    .source_origin_id(original.source_origin_id().clone())
    .logical_root(original.logical_root().as_str())
    .capability_policy(original.capability_policy().clone())
    .budget_policy(original.budget_policy())
    .load_plan(&plan)?
    .build()?;
    Ok(ProjectInputBundle::closed(
        configuration,
        files,
        base.libraries().to_vec(),
    )?)
}

#[test]
fn current_source_absent_project_registry_preserves_frozen_context_prefix() -> TestResult {
    let stop = AtomicBool::new(false);
    let frozen: Value = serde_json::from_str(include_str!(
        "../../../e3/examples/context-continuation-native-input-freeze.json"
    ))?;
    let prefix = &frozen["selected_prefix"];
    assert_eq!(prefix["store_generation_id"], HISTORICAL_GENERATION);
    assert_eq!(prefix["semantic_pack_id"], RETAINED_PACK);
    let scratch = Scratch::new()?;
    let input = scratch.root.join("source-input");
    let (publisher, graph, reference, profile) =
        owners_from_bundle(historical_bundle(&input, &stop)?, &stop)?;
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
    store.close()?;
    drop(graph);
    drop(publisher);
    std::fs::remove_dir_all(&input)?;

    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(
        &ReadSelector::Exact(publication.generation_id.clone()),
        &stop,
    )?;
    assert_eq!(read.store_generation_id(), &publication.generation_id);
    assert_eq!(
        read.publication_set_id(),
        prefix["publication_set_id"]
            .as_str()
            .ok_or("frozen publication set")?
    );
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
    assert_eq!(selected[0].as_str(), prefix["initial_root"]);
    assert_eq!(selected[1].as_str(), prefix["completed_frontier"][0]);
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
        publication.generation_id.as_str()
    );
    assert_eq!(
        envelope["payload"]["publication_set_id"],
        prefix["publication_set_id"]
    );
    assert_eq!(
        envelope["payload"]["semantic_pack"]["semantic_pack_id"],
        RETAINED_PACK
    );
    assert!(!input.exists(), "reopen ran after source removal");
    Ok(())
}
