//! One small synthetic retained native service lifecycle, not live/Ketho evidence.
mod build;
mod frozen_physical;
mod inspect;
mod source_excerpt;
mod status;

use super::*;
use crate::ServiceErrorCode;
use crate::live_project::LiveProjectStore;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    error::Error,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};
use wow_core::{
    CanonicalResult, ContentDigest, ProfileIdentity, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceKind, SourceLogicalSnapshot, ToolVersion,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};
use wow_graph::{
    GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborReadLimits, GraphNode,
    GraphNodeId, GraphPartitionReplacement, GraphPartitionSnapshot, GraphRelationKind,
    GraphSnapshot,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectId, ProjectInputBundle, ProjectInputFile, ProjectKind,
    ProjectPublisher,
    graph::{SOURCE_GRAPH_PARTITION, build_source_graph_proposals},
};
use wow_reference::ReferenceView;
use wow_store::project::{CurrentPublication, ReadSelector};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const STORE_ID: &str = "project-store:fixture-context-primary";
const LUA: &str = "local function ContextFixture(value)\n    return value\nend\n";

fn input_bundle(source: &str) -> TestResult<ProjectInputBundle> {
    input_bundle_files(vec![ProjectInputFile::new("main.lua", source)?])
}

fn input_bundle_files(files: Vec<ProjectInputFile>) -> TestResult<ProjectInputBundle> {
    // Same real backend/configuration/Library constructors as live_project/tests.rs.
    // These declared fixture identities convey no live compatibility authority.
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let profile = ProfileIdentityBuilder::new(
        "profile:fixture:context-primary-map".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:context-primary-map",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:fixture-e0".parse()?,
        ToolVersion::parse("1.0.0")?,
    )])
    .fixture_scope("synthetic native primary navigation map")
    .build()?;
    let binding = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    let config = ProjectConfigurationBuilder::new(
        ProjectId::new("fixture-context-primary-map")?,
        ProjectKind::Fixture,
        profile,
        ReferenceGenerationId::derive(&"fixture:context-primary-reference")?,
        binding,
    )
    .workspace_id(wow_project::ProjectWorkspaceId::new(
        "workspace:fixture:context-primary-map",
    )?)
    .source_origin_id(wow_project::ProjectSourceOriginId::new(
        "project-origin:fixture-context-primary-map",
    )?)
    .logical_root("fixtures/context-primary/main")
    .capability_policy(ProjectCapabilityPolicy::degraded_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?)
    .build()?;
    let library = LuaWorkspaceSnapshot::build(
        backend,
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/Core.lua",
            "---@meta\n---@return number\nfunction External() end\n",
        )],
        LuaWorkspaceLimits::new(8, 4096, 16384, 65536)?,
    )?;
    Ok(ProjectInputBundle::closed(config, files, vec![library])?)
}

fn owners(
    source: &str,
    stop: &AtomicBool,
) -> TestResult<(
    ProjectPublisher,
    GraphPartitionSnapshot,
    ReferenceView,
    ProfileIdentity,
)> {
    owners_from_bundle(input_bundle(source)?, stop)
}

fn owners_from_bundle(
    bundle: ProjectInputBundle,
    stop: &AtomicBool,
) -> TestResult<(
    ProjectPublisher,
    GraphPartitionSnapshot,
    ReferenceView,
    ProfileIdentity,
)> {
    let reference = ReferenceView::new(
        bundle.configuration().reference_generation().to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    let profile = bundle.configuration().selected_profile().clone();
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let view = publisher.publish_initial(bundle)?.open_view();
    let (registry, batch, coverage, _, limits) =
        build_source_graph_proposals(&view, stop)?.into_parts();
    let foundation = GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner = GraphPartitionSnapshot::new(registry, foundation, batch.source_context_id(), stop)?;
    let graph = owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch,
                coverage,
            },
            stop,
        )?
        .candidate()
        .clone();
    Ok((publisher, graph, reference, profile))
}

fn source_file_root(graph: &GraphPartitionSnapshot) -> TestResult<(GraphNodeId, Value, Value)> {
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("retained native source partition")?;
    let mut files = source
        .batch()
        .entity_proposals()
        .iter()
        .filter(|proposal| proposal.entity_kind_id() == "source_file");
    let proposal = files.next().ok_or("native source-file proposal")?;
    assert!(files.next().is_none(), "one physical source-file root");
    let accepted = source.report().accepted_entities();
    let index = accepted
        .binary_search_by(|row| row.proposal_id().cmp(proposal.proposal_id()))
        .map_err(|_| "source-file proposal was not accepted")?;
    let input_node = accepted[index].node();
    let rebound = GraphNode::new(
        graph.snapshot().universe().clone(),
        graph.snapshot().generation().clone(),
        input_node.kind(),
        input_node.owner_key(),
        input_node.evidence_ids().to_vec(),
        graph.snapshot().limits(),
    )?;
    let root = graph
        .snapshot()
        .node(rebound.node_id())
        .ok_or("source-file entity absent from final snapshot")?
        .node_id()
        .clone();
    Ok((
        root,
        serde_json::to_value(proposal)?,
        serde_json::to_value(&accepted[index])?,
    ))
}

struct RetainedObservation {
    root: GraphNodeId,
    root_node: Value,
    adjacent_nodes: Vec<Value>,
    edges: Vec<Value>,
    root_proposal: Value,
    root_accepted: Value,
    entity_proposals: Vec<Value>,
    relation_proposals: Vec<Value>,
    accepted_entities: Vec<Value>,
    accepted_relations: Vec<Value>,
    evidence: Vec<Value>,
    source_handles: Vec<Value>,
    source_context_id: String,
    publication_set_id: String,
    current: CurrentPublication,
}

fn retained(
    store_root: &Path,
    generation: &wow_store::project::StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<RetainedObservation> {
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
    assert_eq!(read.store_generation_id(), generation);
    let graph = read.graph();
    let (root, root_proposal, root_accepted) = source_file_root(graph)?;
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("source partition")?;
    let edges = graph
        .snapshot()
        .edges()
        .iter()
        .filter(|edge| edge.from() == &root || edge.to() == &root)
        .collect::<Vec<_>>();
    assert!(
        !edges.is_empty(),
        "native physical functions must supply real direct edges"
    );
    assert!(
        edges.len() <= 16,
        "this small fixture fits its explicit map edge selection"
    );
    assert!(
        edges
            .iter()
            .all(|edge| edge.relation() == GraphRelationKind::Owns)
    );
    let adjacent_ids = edges
        .iter()
        .flat_map(|edge| [edge.from(), edge.to()])
        .filter(|id| *id != &root)
        .cloned()
        .collect::<BTreeSet<_>>();
    let adjacent_nodes = adjacent_ids
        .iter()
        .map(|id| {
            let node = graph.snapshot().node(id).ok_or("retained edge endpoint")?;
            Ok::<Value, Box<dyn Error>>(serde_json::to_value(node)?)
        })
        .collect::<TestResult<Vec<_>>>()?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("retained native provenance")?;
    assert_eq!(provenance.context().context_id(), graph.source_context_id());
    let result = RetainedObservation {
        root_node: serde_json::to_value(graph.snapshot().node(&root).ok_or("retained root")?)?,
        root,
        adjacent_nodes,
        edges: edges
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        root_proposal,
        root_accepted,
        entity_proposals: source
            .batch()
            .entity_proposals()
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        relation_proposals: source
            .report()
            .accepted_relations()
            .iter()
            .map(|accepted| {
                let proposal = source
                    .batch()
                    .relation_proposal(accepted.proposal_id())
                    .ok_or("accepted native relation missing its original proposal")?;
                Ok(serde_json::to_value(proposal)?)
            })
            .collect::<TestResult<Vec<_>>>()?,
        accepted_entities: source
            .report()
            .accepted_entities()
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        accepted_relations: source
            .report()
            .accepted_relations()
            .iter()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        evidence: provenance
            .evidence()
            .values()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        source_handles: provenance
            .source_handles()
            .values()
            .map(serde_json::to_value)
            .collect::<Result<_, _>>()?,
        source_context_id: graph.source_context_id().to_string(),
        publication_set_id: read.publication_set_id().to_string(),
        current: read
            .current_at_acquisition()
            .ok_or("actual Current observation")?
            .clone(),
    };
    read.close()?;
    store.close()?;
    Ok(result)
}

fn host_value(store_root: &Path, profile: &ProfileIdentity, reference: &ReferenceView) -> Value {
    json!({
        "schema": CONTEXT_MAP_HOST_SCHEMA,
        "project_store_id": STORE_ID,
        "project_store_root": store_root,
        "project_id": "fixture-context-primary-map",
        "reference_profile": profile,
        "reference": reference,
        "limits": {
            "neighbors": GraphNeighborReadLimits::default(),
            "explain": GraphExplainLimits {
                max_output_bytes: 64 * 1024,
                ..GraphExplainLimits::default()
            },
            "evidence": GraphEvidenceResolveLimits::default(),
            "max_direct_edges": 16,
            "max_input_bytes": 4 * 1024 * 1024,
            "max_metadata_bytes": 2 * 1024 * 1024,
            "max_context_work": 100_000
        }
    })
}

fn decode_host(value: &Value, stop: &AtomicBool) -> TestResult<ContextMapHostInput> {
    let bytes = wow_core::canonical_json_bytes(value)?;
    Ok(ContextMapHostInput::from_json_slice(&bytes, stop)?)
}

fn reference_guard(
    profile: &ProfileIdentity,
    reference: &ReferenceView,
) -> TestResult<ContextMapReferenceGuard> {
    Ok(ContextMapReferenceGuard::new(
        profile.profile_id().as_str(),
        reference.generation_id(),
        reference.self_digest(),
    )?)
}

fn exact_request(
    generation: &wow_store::project::StoreGenerationId,
    root: &GraphNodeId,
    reference: Option<ContextMapReferenceGuard>,
) -> TestResult<ContextMapRequest> {
    Ok(ContextMapRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(generation.as_str())?,
        PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        Some(ContextMapRoot::graph_entity(root.as_str())?),
        None,
        reference,
    )?)
}

fn observed_current_digest(current: &CurrentPublication) -> TestResult<String> {
    // Exact observation bytes, deliberately not CurrentRecordId derivation material.
    let bytes = wow_core::canonical_json_bytes(current)?;
    Ok(ContentDigest::<CanonicalResult>::from_bytes(Sha256::digest(bytes).into()).to_string())
}

fn assert_map(
    result: &ContextMapResult,
    expected: &RetainedObservation,
    generation: &wow_store::project::StoreGenerationId,
    host_root: &Path,
) -> TestResult {
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["schema"], CONTEXT_MAP_RESULT_SCHEMA);
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(payload["store_generation_id"], generation.as_str());
    assert_eq!(payload["publication_set_id"], expected.publication_set_id);
    assert_eq!(payload["configured_registration_id"], STORE_ID);
    assert!(
        payload["resolved_request_id"]
            .as_str()
            .ok_or("resolved request identity")?
            .starts_with("context-map-resolved-request:sha256:")
    );
    // Ordinary manifested stores are valid. Their routing key is not a native namespace ID.
    assert!(payload.get("native_project_store_id").is_none());
    assert_eq!(
        payload["current_at_acquisition"],
        serde_json::to_value(&expected.current)?
    );
    assert_eq!(
        payload["current_record_digest"],
        observed_current_digest(&expected.current)?
    );
    let map = &payload["project_map"];
    let core = &map["core"];
    assert_eq!(
        payload["project_map_byte_length"].as_u64(),
        Some(wow_core::canonical_json_bytes(map)?.len() as u64)
    );
    assert_eq!(core["state"], "partial");
    assert_eq!(core["absence_authoritative"], false);
    assert_eq!(core["navigation"]["root"], expected.root_node);
    assert_eq!(
        core["navigation"]["adjacent_nodes"],
        json!(expected.adjacent_nodes)
    );
    assert_eq!(core["navigation"]["edges"], json!(expected.edges));
    assert!(
        !core["omissions"]
            .as_array()
            .ok_or("map omissions")?
            .is_empty()
    );

    let supports = core["supports"].as_array().ok_or("native map supports")?;
    assert_eq!(
        supports.len(),
        1 + expected.adjacent_nodes.len() + expected.edges.len()
    );
    let mut saw_root = false;
    let mut explained_edges = BTreeSet::new();
    for receipt in supports {
        let explanation = &receipt["explanation"];
        assert_eq!(explanation["source_context_id"], expected.source_context_id);
        assert_eq!(explanation["support_complete"], true);
        assert!(
            explanation["truncations"]
                .as_array()
                .ok_or("explanation truncations")?
                .is_empty()
        );
        let evidence = &receipt["evidence_resolution"];
        assert_eq!(evidence["closure_complete"], true);
        assert!(
            evidence["unresolved"]
                .as_array()
                .ok_or("unresolved support")?
                .is_empty()
        );
        assert!(
            evidence["truncations"]
                .as_array()
                .ok_or("evidence truncations")?
                .is_empty()
        );
        let records = evidence["evidence_records"]
            .as_array()
            .ok_or("native evidence")?;
        let handles = evidence["source_handles"]
            .as_array()
            .ok_or("native source handles")?;
        assert!(!records.is_empty() && !handles.is_empty());
        for record in records {
            assert!(
                expected.evidence.contains(record),
                "original retained EvidenceRecord"
            );
        }
        for handle in handles {
            assert!(
                expected.source_handles.contains(handle),
                "original retained SourceHandle"
            );
        }
        let native = explanation["supports"]
            .as_array()
            .ok_or("original producer supports")?;
        assert!(!native.is_empty());
        for support in native {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            match support["origin"].as_str().ok_or("support origin")? {
                "producer_entity" => {
                    assert!(expected.entity_proposals.contains(&support["proposal"]));
                    assert!(expected.accepted_entities.contains(&support["accepted"]));
                }
                "producer_relation" => {
                    assert!(expected.relation_proposals.contains(&support["proposal"]));
                    assert!(expected.accepted_relations.contains(&support["accepted"]));
                }
                _ => {
                    return Err(
                        "unexpected foundation support in this empty-foundation fixture".into(),
                    );
                }
            }
        }
        let record = &explanation["record"];
        match record["kind"].as_str().ok_or("explained record kind")? {
            "entity" => {
                if record["node"] == expected.root_node {
                    saw_root = true;
                    assert!(
                        native.iter().any(|support| {
                            support["proposal"] == expected.root_proposal
                                && support["accepted"] == expected.root_accepted
                        }),
                        "exact accepted physical-file proof"
                    );
                } else {
                    assert!(expected.adjacent_nodes.contains(&record["node"]));
                }
            }
            "relation" => {
                assert!(expected.edges.contains(&record["edge"]));
                let id = record["edge"]["edge_id"].as_str().ok_or("native edge ID")?;
                assert!(explained_edges.insert(id.to_string()));
            }
            _ => return Err("unexpected native explanation record".into()),
        }
    }
    assert!(saw_root);
    assert_eq!(explained_edges.len(), expected.edges.len());
    let custody = core["source_custody"]
        .as_array()
        .ok_or("physical source custody")?;
    assert!(!custody.is_empty());
    for row in custody {
        assert_eq!(row["scope"], "primary");
        assert!(expected.source_handles.contains(&row["handle"]));
    }
    let text = std::str::from_utf8(result.canonical_bytes())?;
    assert!(!text.contains("project_store_root"));
    assert!(
        !text.contains("local function ContextFixture"),
        "source bodies are excluded"
    );
    let root_name = host_root
        .file_name()
        .ok_or("fixture host root name")?
        .to_str()
        .ok_or("fixture host root UTF-8")?;
    assert!(
        !text.contains(root_name),
        "host registration paths are not semantic output"
    );
    Ok(())
}

fn assert_refused(
    host: &ContextMapHostInput,
    request: &ContextMapRequest,
    code: ServiceErrorCode,
    expected_closure: &str,
    stop: &AtomicBool,
) -> TestResult {
    let result = execute_context_map(host, request, stop)?;
    assert_eq!(result.failure_code(), Some(code));
    assert_eq!(
        result.status(),
        if code == ServiceErrorCode::Cancelled {
            ContextMapStatus::Cancelled
        } else {
            ContextMapStatus::Failed
        }
    );
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["failure_code"], serde_json::to_value(code)?);
    assert_eq!(envelope["resource_closure"], expected_closure);
    assert!(
        envelope.get("payload").is_none(),
        "no failed private draft may escape"
    );
    Ok(())
}

struct Scratch {
    root: PathBuf,
    export: bool,
}
impl Scratch {
    fn new() -> TestResult<Self> {
        let selected = std::env::var_os("WOW_CONTEXT_MAP_FIXTURE_EXPORT");
        let export = selected.is_some();
        let root = match selected {
            Some(path) => PathBuf::from(path),
            None => std::env::temp_dir().join(format!(
                "wow-context-map-synthetic-{}-{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
            )),
        };
        if !root.is_absolute() || root.as_os_str().len() > 32768 {
            return Err("fixture export requires a finite absolute fresh scratch path".into());
        }
        // This test owns only a newly created directory; never overwrite/adopt a preexisting one.
        std::fs::create_dir(&root)?;
        Ok(Self { root, export })
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        if !self.export {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn write_export(root: &Path, name: &str, bytes: &[u8], limit: usize) -> TestResult {
    if bytes.len() > limit {
        return Err("synthetic fixture export exceeded its byte bound".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(name))?;
    file.write_all(bytes)?;
    Ok(())
}

fn root_token(id: &GraphNodeId) -> String {
    format!("graph-entity@{}", encoded_root_id(id.as_str()))
}

fn encoded_root_id(id: &str) -> String {
    // Mechanical CLI transport encoding of the genuine native ID; no semantic lookup.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::new();
    for group in id.as_bytes().chunks(3) {
        encoded.push(ALPHABET[(group[0] >> 2) as usize] as char);
        encoded.push(
            ALPHABET[(((group[0] & 3) << 4) | group.get(1).map_or(0, |b| *b >> 4)) as usize]
                as char,
        );
        if let Some(second) = group.get(1) {
            encoded.push(
                ALPHABET[(((second & 15) << 2) | group.get(2).map_or(0, |b| *b >> 6)) as usize]
                    as char,
            );
        }
        if let Some(third) = group.get(2) {
            encoded.push(ALPHABET[(third & 63) as usize] as char);
        }
    }
    encoded
}

fn export_fixture(
    scratch: &Scratch,
    host: &Value,
    request: &ContextMapRequest,
    result: &ContextMapResult,
    profile: &ProfileIdentity,
    reference: &ReferenceView,
) -> TestResult {
    if !scratch.export {
        return Ok(());
    }
    let ContextMapPublication::StoreGeneration(generation) = &request.publication else {
        return Err("fixture export requires its exact retained generation".into());
    };
    let (root_token_text, map_profile, selection_note) = match &request.root {
        Some(ContextMapRoot::GraphEntity(root)) => (
            root_token(root),
            PRIMARY_NAVIGATION_MAP_PROFILE_ID,
            "exact retained store generation after both fixture publications",
        ),
        Some(ContextMapRoot::Project(project)) => (
            format!("project@{}", encoded_root_id(project.as_str())),
            PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
            "exact retained project-root publication",
        ),
        None => return Err("fixture export requires its exact explicit root".into()),
    };
    let arguments = json!([
        "context",
        "map",
        "--native-host-config",
        scratch.root.join("host.json"),
        "--project-store",
        STORE_ID,
        "--publication",
        format!("store-generation:{}", generation.as_str()),
        "--map-target",
        "primary",
        "--map-profile-id",
        map_profile,
        "--root",
        root_token_text,
        "--platform",
        "omitted",
        "--expect-profile",
        profile.profile_id().as_str(),
        "--expect-reference-generation",
        reference.generation_id(),
        "--expect-reference-view",
        reference.self_digest(),
        "--output",
        "envelope-json"
    ]);
    export_native_fixture(scratch, host, request, &arguments, result, selection_note)
}

fn export_native_fixture(
    scratch: &Scratch,
    host: &Value,
    request: &impl serde::Serialize,
    arguments: &Value,
    result: &ContextMapResult,
    selection_note: &str,
) -> TestResult {
    if !scratch.export {
        return Ok(());
    }
    // Transport paths are allowed only in these private host/argv inputs.
    let files = [
        (
            "host.json",
            wow_core::canonical_json_bytes(host)?,
            CONTEXT_MAP_HOST_INPUT_MAX_BYTES,
        ),
        (
            "request.json",
            wow_core::canonical_json_bytes(request)?,
            64 * 1024,
        ),
        (
            "cli-arguments.json",
            wow_core::canonical_json_bytes(arguments)?,
            64 * 1024,
        ),
        (
            "fixture.json",
            wow_core::canonical_json_bytes(&json!({
                "synthetic": true,
                "authority": "fixture_only_not_live_or_ketho",
                "expected_cli_exit": 2,
                "cli_stdout": "expected-envelope.json followed by one LF",
                "selection": selection_note,
                "semantic_host_paths": "excluded"
            }))?,
            64 * 1024,
        ),
    ];
    for (name, bytes, limit) in files {
        write_export(&scratch.root, name, &bytes, limit)?;
    }
    write_export(
        &scratch.root,
        "expected-envelope.json",
        result.canonical_bytes(),
        CONTEXT_MAP_RESULT_MAX_BYTES,
    )
}

#[test]
fn retained_native_primary_map_is_exact_repeatable_and_refuses_mixed_or_missing_custody()
-> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let store_root = scratch.root.join("registered-store");
    let (publisher, graph, reference, profile) = owners(LUA, &stop)?;
    assert_eq!(
        reference.generation_id(),
        publisher
            .current_snapshot()
            .ok_or("published native snapshot")?
            .open_view()
            .configuration()
            .reference_generation()
            .to_string()
    );
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let first = store.publish(&publisher, &graph, "fixture-context-initial", None, &stop)?;
    let first_current = store.current()?.ok_or("first native Current")?;
    assert_eq!(first_current.generation_id, first.generation_id);
    store.close()?;
    drop(graph);
    drop(publisher);
    let first_read = retained(&store_root, &first.generation_id, &stop)?;
    assert_eq!(first_read.current, first_current);

    let host_json = host_value(&store_root, &profile, &reference);
    let host = decode_host(&host_json, &stop)?;
    let request = exact_request(
        &first.generation_id,
        &first_read.root,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let first_map = execute_context_map(&host, &request, &stop)?;
    assert_map(&first_map, &first_read, &first.generation_id, &scratch.root)?;
    let repeated = execute_context_map(&host, &request, &stop)?;
    assert_map(&repeated, &first_read, &first.generation_id, &scratch.root)?;
    assert_eq!(first_map.canonical_bytes(), repeated.canonical_bytes());

    // Separate reservations would accept this input; the aggregate must refuse
    // before allocating the catalog. Charge actual borrowed native views.
    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(&ReadSelector::Exact(first.generation_id.clone()), &stop)?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("retained provenance")?;
    let reference_bytes = wow_core::canonical_json_bytes(&reference)?.len();
    let graph_bytes = wow_core::canonical_json_bytes(read.graph().snapshot())?.len();
    let catalog_bytes = wow_core::canonical_json_bytes(&(
        provenance.context(),
        provenance.evidence(),
        provenance.source_handles(),
    ))?
    .len();
    let isolated_limit = (reference_bytes + graph_bytes).max(catalog_bytes);
    assert!(reference_bytes + graph_bytes + catalog_bytes > isolated_limit);
    read.close()?;
    store.close()?;
    let mut aggregate_host = host_json.clone();
    aggregate_host["limits"]["max_input_bytes"] = json!(isolated_limit);
    assert_refused(
        &decode_host(&aggregate_host, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;

    // A genuine second source generation supplies both a stale Current witness
    // and a native root that can be proved absent from the selected first graph.
    let (publisher, graph, next_reference, next_profile) = owners(
        "local function ContextFixture(value)\n    return value + 1\nend\n",
        &stop,
    )?;
    assert_eq!(reference, next_reference);
    assert_eq!(profile, next_profile);
    let mut store = LiveProjectStore::open(&store_root)?;
    let second = store.publish(
        &publisher,
        &graph,
        "fixture-context-next",
        Some(first_current.record_id.clone()),
        &stop,
    )?;
    let second_current = store.current()?.ok_or("second native Current")?;
    assert_eq!(second_current.generation_id, second.generation_id);
    assert_ne!(first_current.record_id, second_current.record_id);
    store.close()?;
    drop(graph);
    drop(publisher);
    let selected = retained(&store_root, &second.generation_id, &stop)?;
    assert_eq!(selected.current, second_current);
    assert_ne!(first_read.root, selected.root);
    let store = LiveProjectStore::open(&store_root)?;
    let read = store.read(&ReadSelector::Exact(first.generation_id.clone()), &stop)?;
    assert!(read.graph().snapshot().node(&selected.root).is_none());
    read.close()?;
    store.close()?;

    let mut wrong_host = host_json.clone();
    wrong_host["project_store_id"] = json!("project-store:fixture-context-other");
    assert_refused(
        &decode_host(&wrong_host, &stop)?,
        &request,
        ServiceErrorCode::InvalidConfiguration,
        "not_acquired",
        &stop,
    )?;
    let mut wrong_project = host_json.clone();
    wrong_project["project_id"] = json!("fixture-context-other");
    assert_refused(
        &decode_host(&wrong_project, &stop)?,
        &request,
        ServiceErrorCode::IdentityMismatch,
        "closed",
        &stop,
    )?;

    let mixed_reference = ReferenceView::new(
        ReferenceGenerationId::derive(&"fixture:context-other-reference")?.to_string(),
        Vec::new(),
        Vec::new(),
    )?;
    let mut mixed_host = host_json.clone();
    mixed_host["reference"] = serde_json::to_value(mixed_reference)?;
    // Omit optional request guard so this specifically reaches cross-owner generation admission.
    let mixed_request = exact_request(&second.generation_id, &selected.root, None)?;
    assert_refused(
        &decode_host(&mixed_host, &stop)?,
        &mixed_request,
        ServiceErrorCode::IdentityMismatch,
        "closed",
        &stop,
    )?;

    // Separate record-only and digest-only stale checks use actual earlier observations.
    let stale_record = ContextMapCurrentGuard::new(Some(first_current.record_id.as_str()), None)?;
    let first_digest = observed_current_digest(&first_current)?;
    assert_ne!(first_digest, observed_current_digest(&second_current)?);
    let stale_digest = ContextMapCurrentGuard::new(None, Some(&first_digest))?;
    for guard in [stale_record, stale_digest] {
        let stale = ContextMapRequest::primary(
            STORE_ID,
            ContextMapPublication::Current,
            PRIMARY_NAVIGATION_MAP_PROFILE_ID,
            Some(ContextMapRoot::graph_entity(selected.root.as_str())?),
            Some(guard),
            Some(reference_guard(&profile, &reference)?),
        )?;
        assert_refused(
            &host,
            &stale,
            ServiceErrorCode::StoreCurrentConflict,
            "closed",
            &stop,
        )?;
    }
    let missing = exact_request(
        &first.generation_id,
        &selected.root,
        Some(reference_guard(&profile, &reference)?),
    )?;
    assert_refused(
        &host,
        &missing,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    let omitted = ContextMapRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(second.generation_id.as_str())?,
        PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    assert_refused(
        &host,
        &omitted,
        ServiceErrorCode::ComponentUnavailable,
        "closed",
        &stop,
    )?;
    assert_refused(
        &host,
        &request,
        ServiceErrorCode::Cancelled,
        "not_acquired",
        &AtomicBool::new(true),
    )?;

    let mut tiny_host = host_json.clone();
    tiny_host["limits"]["max_metadata_bytes"] = json!(1);
    assert_refused(
        &decode_host(&tiny_host, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;

    // Reopen after every refusal: no result draft or native lease leaked.
    let after = retained(&store_root, &second.generation_id, &stop)?;
    assert_eq!(after.current, selected.current);
    // Compare reviewed frozen bytes on the exact retained read after Current
    // advances. Host filesystem placement must not change semantic output.
    let exported = retained(&store_root, &first.generation_id, &stop)?;
    assert_eq!(exported.current, selected.current);
    let frozen = execute_context_map(&host, &request, &stop)?;
    assert_map(&frozen, &exported, &first.generation_id, &scratch.root)?;
    let (expected_envelope, expected_request) = frozen_physical::runtime_bound_fixture(
        include_bytes!("../../e3/native-primary-map/envelope.json"),
        include_bytes!("../../e3/native-primary-map/request.json"),
        &first.generation_id,
        &exported.current,
        &host_json,
    )?;
    assert_eq!(frozen.canonical_bytes(), expected_envelope);
    assert_eq!(
        wow_core::canonical_json_bytes(&request)?.as_slice(),
        expected_request
    );
    assert_eq!(
        wow_context::PRIMARY_MAP_PROFILE_BYTES,
        include_bytes!("../../e3/native-primary-map/profile.json")
    );
    if scratch.export {
        export_fixture(
            &scratch, &host_json, &request, &frozen, &profile, &reference,
        )?;
    }
    Ok(())
}

const PROJECT_OTHER_LUA: &str =
    "local function OtherContextFixture(value)\n    return value + 1\nend\n";

fn retained_project_expectation(
    store_root: &Path,
    generation: &wow_store::project::StoreGenerationId,
    stop: &AtomicBool,
) -> TestResult<Value> {
    let store = LiveProjectStore::open(store_root)?;
    let read = store.read(&ReadSelector::Exact(generation.clone()), stop)?;
    let graph = read.graph();
    let source = graph
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("retained source partition")?;
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("retained native provenance")?;
    assert_eq!(provenance.files().len(), 2);
    assert!(
        graph
            .snapshot()
            .nodes()
            .iter()
            .all(|node| node.kind() != "project")
    );
    let mut files = Vec::new();
    let mut nodes = std::collections::BTreeMap::new();
    let mut edges = std::collections::BTreeMap::new();
    for file in provenance.files() {
        let proposal = source
            .batch()
            .entity_proposal(&file.proposal_id)
            .ok_or("retained physical file proposal")?;
        assert_eq!(proposal.entity_kind_id(), "source_file");
        let index = source
            .report()
            .accepted_entities()
            .binary_search_by(|accepted| accepted.proposal_id().cmp(file.proposal_id.as_str()))
            .map_err(|_| "physical file proposal not accepted")?;
        let input = source.report().accepted_entities()[index].node();
        let rebound = GraphNode::new(
            graph.snapshot().universe().clone(),
            graph.snapshot().generation().clone(),
            input.kind(),
            input.owner_key(),
            input.evidence_ids().to_vec(),
            graph.snapshot().limits(),
        )?;
        let node = graph
            .snapshot()
            .node(rebound.node_id())
            .ok_or("exact final physical file root")?;
        let root = serde_json::to_value(node)?;
        nodes.insert(node.node_id().to_string(), root.clone());
        let incident = graph
            .snapshot()
            .edges()
            .iter()
            .filter(|edge| edge.from() == node.node_id() || edge.to() == node.node_id())
            .collect::<Vec<_>>();
        assert_eq!(
            incident.len(),
            2,
            "genuine Chunk plus function ownership per physical file"
        );
        assert!(
            incident
                .iter()
                .all(|edge| edge.relation() == GraphRelationKind::Owns)
        );
        let adjacent_ids = incident
            .iter()
            .flat_map(|edge| [edge.from(), edge.to()])
            .filter(|id| *id != node.node_id())
            .cloned()
            .collect::<BTreeSet<_>>();
        let adjacent = adjacent_ids
            .iter()
            .map(|id| {
                let endpoint = graph.snapshot().node(id).ok_or("final direct endpoint")?;
                let value = serde_json::to_value(endpoint)?;
                nodes.insert(id.to_string(), value.clone());
                Ok(value)
            })
            .collect::<TestResult<Vec<Value>>>()?;
        let incident = incident
            .into_iter()
            .map(|edge| {
                let value = serde_json::to_value(edge)?;
                edges.insert(edge.edge_id().to_string(), value.clone());
                Ok(value)
            })
            .collect::<TestResult<Vec<Value>>>()?;
        files.push(json!({ "file": file, "node": root, "edges": incident, "adjacent": adjacent }));
    }
    let entity_proposals = source
        .batch()
        .entity_proposals()
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let accepted_entities = source
        .report()
        .accepted_entities()
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let accepted_relations = source
        .report()
        .accepted_relations()
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let relation_proposals = source
        .report()
        .accepted_relations()
        .iter()
        .map(|accepted| {
            let proposal = source
                .batch()
                .relation_proposal(accepted.proposal_id())
                .ok_or("accepted relation missing native proposal")?;
            Ok(serde_json::to_value(proposal)?)
        })
        .collect::<TestResult<Vec<Value>>>()?;
    let evidence = provenance
        .evidence()
        .values()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let handles = provenance
        .source_handles()
        .values()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()?;
    let expected = json!({
        "project_id": read.project().configuration().project_id(),
        "publication_set_id": read.publication_set_id(),
        "store_generation_id": generation.as_str(),
        "current": read.current_at_acquisition().ok_or("retained Current observation")?,
        "source_context_id": graph.source_context_id(),
        "files": files,
        "nodes": nodes.into_values().collect::<Vec<_>>(),
        "edges": edges.into_values().collect::<Vec<_>>(),
        "entity_proposals": entity_proposals,
        "accepted_entities": accepted_entities,
        "relation_proposals": relation_proposals,
        "accepted_relations": accepted_relations,
        "evidence": evidence,
        "handles": handles
    });
    read.close()?;
    store.close()?;
    Ok(expected)
}

fn assert_project_map(
    result: &ContextMapResult,
    expected: &Value,
    host_root: &Path,
    state: ContextMapStatus,
) -> TestResult<Value> {
    assert_eq!(result.status(), state);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    assert!(envelope.get("failure_code").is_none());
    let payload = &envelope["payload"];
    assert_eq!(
        payload["publication_set_id"],
        expected["publication_set_id"]
    );
    assert_eq!(
        payload["store_generation_id"],
        expected["store_generation_id"]
    );
    assert_eq!(payload["current_at_acquisition"], expected["current"]);
    let map = &payload["project_map"];
    assert_eq!(
        payload["project_map_byte_length"].as_u64(),
        Some(wow_core::canonical_json_bytes(map)?.len() as u64)
    );
    let core = &map["core"];
    assert_eq!(core["state"], serde_json::to_value(state)?);
    assert_eq!(core["absence_authoritative"], false);
    let navigation = &core["navigation"];
    assert_eq!(navigation["schema"], "wow-context/project-navigation/1");
    assert_eq!(navigation["project_id"], expected["project_id"]);
    assert_eq!(
        navigation["root_enumeration"],
        "complete_for_retained_primary_source_files"
    );
    let files = expected["files"]
        .as_array()
        .ok_or("expected genuine files")?;
    let roots = navigation["source_file_roots"]
        .as_array()
        .ok_or("mandatory physical roots")?;
    let neighborhoods = navigation["neighborhoods"]
        .as_array()
        .ok_or("native neighborhoods")?;
    assert_eq!(roots.len(), files.len());
    assert_eq!(neighborhoods.len(), files.len());
    let mut seen_roots = BTreeSet::new();
    let mut total_edges = 0;
    let mut selected_records = BTreeSet::new();
    for file in files {
        assert!(
            roots
                .iter()
                .any(|root| root["file"] == file["file"] && root["node"] == file["node"]),
            "every actual ProjectGraphFile retains its exact final GraphNode"
        );
        let mut matching = neighborhoods
            .iter()
            .filter(|view| view["root"] == file["node"]);
        let view = matching
            .next()
            .ok_or("mandatory native file neighborhood")?;
        assert!(matching.next().is_none());
        assert!(seen_roots.insert(view["root"]["node_id"].as_str().ok_or("final root ID")?));
        let all_edges = file["edges"].as_array().ok_or("retained own edges")?;
        let count = view["edges"].as_array().ok_or("native direct edges")?.len();
        if state == ContextMapStatus::Truncated {
            assert_eq!(count, 1);
            assert_eq!(view["state"], "truncated");
            assert_eq!(view["edges"], json!([all_edges[0]]));
            let edge = &all_edges[0];
            let endpoint = if edge["from"] == file["node"]["node_id"] {
                &edge["to"]
            } else {
                &edge["from"]
            };
            let node = expected["nodes"]
                .as_array()
                .ok_or("native nodes")?
                .iter()
                .find(|node| &node["node_id"] == endpoint)
                .ok_or("retained truncated endpoint")?;
            assert_eq!(view["adjacent_nodes"], json!([node]));
            assert_eq!(view["matching_edges"], all_edges.len());
            assert_eq!(view["omitted_edges"], all_edges.len() - count);
            assert!(
                !view["truncations"]
                    .as_array()
                    .ok_or("native truncations")?
                    .is_empty()
            );
        } else {
            assert_eq!(count, 2);
            assert_eq!(view["edges"], file["edges"]);
            assert_eq!(view["adjacent_nodes"], file["adjacent"]);
            assert!(
                view["truncations"]
                    .as_array()
                    .ok_or("native truncations")?
                    .is_empty()
            );
            assert_eq!(view["omitted_edges"], 0);
        }
        for node in std::iter::once(&view["root"]).chain(
            view["adjacent_nodes"]
                .as_array()
                .ok_or("native endpoints")?
                .iter(),
        ) {
            selected_records.insert((
                "entity".to_string(),
                node["node_id"]
                    .as_str()
                    .ok_or("native node ID")?
                    .to_string(),
            ));
        }
        for edge in view["edges"].as_array().ok_or("selected native edges")? {
            selected_records.insert((
                "relation".to_string(),
                edge["edge_id"]
                    .as_str()
                    .ok_or("native edge ID")?
                    .to_string(),
            ));
        }
        total_edges += count;
    }
    assert_eq!(
        total_edges,
        if state == ContextMapStatus::Truncated {
            2
        } else {
            4
        },
        "per-query edge cap must not become a project-wide/root prefix"
    );
    let edge_omission = core["omissions"]
        .as_array()
        .ok_or("map omission accounting")?
        .iter()
        .find(|row| row["scope"] == "direct_navigation_prefix")
        .ok_or("native prefix omission")?;
    assert_eq!(edge_omission["known_omitted"], 4 - total_edges);
    assert_eq!(
        edge_omission["enumeration"],
        "sum_of_complete_match_counts_under_native_queries"
    );
    let nodes = expected["nodes"].as_array().ok_or("retained nodes")?;
    let edges = expected["edges"].as_array().ok_or("retained edges")?;
    let supports = core["supports"]
        .as_array()
        .ok_or("deduplicated mandatory supports")?;
    assert_eq!(supports.len(), selected_records.len());
    let mut seen = BTreeSet::new();
    for receipt in supports {
        let explanation = &receipt["explanation"];
        assert_eq!(
            explanation["source_context_id"],
            expected["source_context_id"]
        );
        assert_eq!(explanation["support_complete"], true);
        let record = &explanation["record"];
        let (kind, id) = match record["kind"].as_str().ok_or("native record kind")? {
            "entity" => {
                assert!(nodes.contains(&record["node"]));
                (
                    "entity",
                    record["node"]["node_id"].as_str().ok_or("native node ID")?,
                )
            }
            "relation" => {
                assert!(edges.contains(&record["edge"]));
                (
                    "relation",
                    record["edge"]["edge_id"].as_str().ok_or("native edge ID")?,
                )
            }
            _ => return Err("non-native map record".into()),
        };
        assert!(
            seen.insert((kind, id)),
            "support dedup is by actual node/edge identity"
        );
        assert!(
            selected_records.remove(&(kind.to_string(), id.to_string())),
            "exact selected support union"
        );
        let original = explanation["supports"]
            .as_array()
            .ok_or("original native support")?;
        assert!(!original.is_empty());
        for support in original {
            assert_eq!(support["producer"]["partition_id"], SOURCE_GRAPH_PARTITION);
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("fixture has no synthetic foundation support".into()),
            };
            assert!(
                expected[proposals]
                    .as_array()
                    .ok_or("native proposals")?
                    .contains(&support["proposal"])
            );
            assert!(
                expected[accepted]
                    .as_array()
                    .ok_or("native acceptance")?
                    .contains(&support["accepted"])
            );
        }
        let resolution = &receipt["evidence_resolution"];
        assert_eq!(resolution["closure_complete"], true);
        assert!(
            resolution["unresolved"]
                .as_array()
                .ok_or("unresolved")?
                .is_empty()
        );
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            let rows = resolution[actual]
                .as_array()
                .ok_or("native evidence rows")?;
            assert!(!rows.is_empty());
            for row in rows {
                assert!(
                    expected[retained]
                        .as_array()
                        .ok_or("retained evidence")?
                        .contains(row)
                );
            }
        }
    }
    assert!(
        selected_records.is_empty(),
        "every selected actual node/edge has mandatory support"
    );
    let text = std::str::from_utf8(result.canonical_bytes())?;
    for body in [LUA, PROJECT_OTHER_LUA] {
        assert!(
            !text.contains(&serde_json::to_string(body)?),
            "raw Lua body is not map metadata"
        );
        assert!(
            !text.contains(body.lines().next().ok_or("fixture Lua declaration")?),
            "no Lua body prefix"
        );
    }
    assert!(
        !text.contains(&serde_json::to_string(&host_root.join("registered-store"))?),
        "no absolute store path"
    );
    assert!(!text.contains("project_store_root"));
    assert!(
        !text.contains(
            host_root
                .file_name()
                .ok_or("scratch name")?
                .to_str()
                .ok_or("UTF-8 scratch")?
        )
    );
    Ok(map.clone())
}

#[test]
fn retained_project_root_enumerates_both_files_and_refuses_required_aggregate_budget() -> TestResult
{
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let store_root = scratch.root.join("registered-store");
    let (publisher, graph, reference, profile) = owners_from_bundle(
        input_bundle_files(vec![
            ProjectInputFile::new("main.lua", LUA)?,
            ProjectInputFile::new("other.lua", PROJECT_OTHER_LUA)?,
        ])?,
        &stop,
    )?;
    let project_id = publisher
        .current_snapshot()
        .ok_or("published native snapshot")?
        .open_view()
        .configuration()
        .project_id()
        .clone();
    let mut store = LiveProjectStore::create(&store_root, graph.snapshot().universe().as_str())?;
    let published = store.publish(
        &publisher,
        &graph,
        "fixture-context-project-root",
        None,
        &stop,
    )?;
    store.close()?;
    drop(graph);
    drop(publisher);
    let expected = retained_project_expectation(&store_root, &published.generation_id, &stop)?;
    assert_eq!(expected["project_id"], serde_json::to_value(&project_id)?);
    let mut host_json = host_value(&store_root, &profile, &reference);
    host_json["limits"]["max_direct_edges"] = json!(2);
    let host = decode_host(&host_json, &stop)?;
    let request = ContextMapRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(published.generation_id.as_str())?,
        PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        Some(ContextMapRoot::project(project_id.as_str())?),
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let first = execute_context_map(&host, &request, &stop)?;
    let explicit = assert_project_map(&first, &expected, &scratch.root, ContextMapStatus::Partial)?;
    let repeated = execute_context_map(&host, &request, &stop)?;
    assert_eq!(first.canonical_bytes(), repeated.canonical_bytes());
    let derived_request = ContextMapRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(published.generation_id.as_str())?,
        PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        None,
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    let derived = execute_context_map(&host, &derived_request, &stop)?;
    let implicit = assert_project_map(
        &derived,
        &expected,
        &scratch.root,
        ContextMapStatus::Partial,
    )?;
    assert_eq!(
        explicit, implicit,
        "same exact acquired project root yields same map artifact"
    );
    let mut truncated_host_json = host_json.clone();
    truncated_host_json["limits"]["max_direct_edges"] = json!(1);
    let truncated =
        execute_context_map(&decode_host(&truncated_host_json, &stop)?, &request, &stop)?;
    assert_project_map(
        &truncated,
        &expected,
        &scratch.root,
        ContextMapStatus::Truncated,
    )?;

    let mismatch = ContextMapRequest::primary(
        STORE_ID,
        ContextMapPublication::store_generation(published.generation_id.as_str())?,
        PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        Some(ContextMapRoot::project("fixture-context-unselected")?),
        None,
        Some(reference_guard(&profile, &reference)?),
    )?;
    assert_refused(
        &host,
        &mismatch,
        ServiceErrorCode::InvalidRequest,
        "closed",
        &stop,
    )?;
    let mut low_work = host_json.clone();
    low_work["limits"]["max_context_work"] = json!(1);
    assert_refused(
        &decode_host(&low_work, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    let mut low_output = host_json.clone();
    // One native explanation allowance cannot fund the mandatory project support batch.
    low_output["limits"]["max_metadata_bytes"] =
        host_json["limits"]["explain"]["max_output_bytes"].clone();
    assert_refused(
        &decode_host(&low_output, &stop)?,
        &request,
        ServiceErrorCode::BudgetExceeded,
        "closed",
        &stop,
    )?;
    let prepared_current: CurrentPublication = serde_json::from_value(expected["current"].clone())?;
    let (expected_envelope, expected_request) = frozen_physical::runtime_bound_fixture(
        include_bytes!("../../e3/native-primary-project-map/envelope.json"),
        include_bytes!("../../e3/native-primary-project-map/request.json"),
        &published.generation_id,
        &prepared_current,
        &host_json,
    )?;
    assert_eq!(first.canonical_bytes(), expected_envelope);
    assert_eq!(wow_core::canonical_json_bytes(&request)?, expected_request);
    assert_eq!(
        wow_context::PRIMARY_PROJECT_MAP_PROFILE_BYTES,
        include_bytes!("../../e3/native-primary-project-map/profile.json")
    );
    export_fixture(&scratch, &host_json, &request, &first, &profile, &reference)?;
    Ok(())
}
