//! One synthetic native Primary callable lifecycle; no live/Ketho authority.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    error::Error,
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};
use wow_core::{
    ContentDigest, ProfileIdentity, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry,
    SourceContent, SourceKind, SourceLogicalSnapshot,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE,
    function_calls::{
        PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,
        SourceFunctionDeclaration, SourceFunctionKind, SourceSignatureType,
    },
};
use wow_graph::{
    GraphEvidenceResolveLimits, GraphExplainLimits, GraphNeighborReadLimits, GraphNodeId,
    GraphPartitionSnapshot,
};
use wow_project::{
    ProjectKind,
    graph::SOURCE_GRAPH_PARTITION,
    replay::{ProjectReplay, publication},
};
use wow_reference::ReferenceView;
use wow_service::{
    LocalProjectInput, ServiceErrorCode,
    context::{
        CONTEXT_MAP_HOST_SCHEMA, ContextBuildRequest, ContextInspectLevel, ContextInspectRequest,
        ContextMapHostInput, ContextMapPublication, ContextMapReferenceGuard, ContextMapResult,
        ContextMapRoot, ContextMapStatus, ContextRenderSelection, PRIMARY_CONTEXT_PACK_PROFILE_ID,
        PRIMARY_L1_PROFILE_ID, execute_context_build, execute_context_inspect,
    },
    graph::GraphBuildRequest,
    live_project::{
        LiveProjectLibraryMode, LiveProjectPublishRequest, LiveProjectStore,
        LiveProjectUpdateRequest, publish_local_project, update_local_project,
    },
};
use wow_store::project::{CurrentPublication, ProjectStore, ReadSelector, RecordCatalog};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const PROJECT: &str = "fixture-live-pair";
const REGISTRATION: &str = "project-store:fixture-primary-callables";
const INPUT_SCHEMA: &str = "wow-service/local-primary-callables-files/1";
const STORAGE_SCHEMA: &str = "wow-project.live-replay";
const WORKSPACE_ID: &str = "workspace:main:project-pair-regression";
const SOURCE_ORIGIN_ID: &str = "project-origin:project-pair-regression";
const LOGICAL_ROOT: &str = "fixtures/project-pair/main";
const REFERENCE_GENERATION: &str =
    "generation:reference:sha256:97675ba211ab64855e16865ac469cfe32478719078c34f7c34d31a1a07d99918";
const ANALYZER_CONFIGURATION_DIGEST: &str =
    "sha256:0303030303030303030303030303030303030303030303030303030303030303";
const ANALYZER_CONTRACT_ID: &str = "wow-emmy/e0-c/1";
const ANALYZER_LIBRARY_CONTRACT_ID: &str = "wow-emmy-e0-c-library-v1";
// Reuse the native indexed-signature fixture's genuine annotated physical body.
const MAIN: &str = "---@param value number\n---@return number\nlocal function TypedSignatureFixture(value)\n    return value\nend\n";
const REMOVED: &str = "---@param value number\n---@return number\nlocal function RemovedDeclaration(value)\n    return value\nend\n";
const LIBRARY: &str = "---@meta\n---@return number\nfunction External() end\n";

struct Scratch {
    root: PathBuf,
    export: bool,
}
impl Scratch {
    fn new() -> TestResult<Self> {
        let selected = std::env::var_os("W21_PRIMARY_EXPORT_DIR");
        let export = selected.is_some();
        let root = selected.map(PathBuf::from).unwrap_or_else(|| {
            std::env::temp_dir().join(format!(
                "wow-primary-callables-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |duration| duration.as_nanos())
            ))
        });
        if !root.is_absolute() || root.as_os_str().len() > 32768 {
            return Err("export requires a bounded absolute fresh directory".into());
        }
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
fn digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}
fn write_new(root: &Path, name: &str, bytes: &[u8], limit: usize) -> TestResult {
    if bytes.len() > limit {
        return Err("bounded fixture write exceeded its allowance".into());
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(root.join(name))?;
    file.write_all(bytes)?;
    Ok(())
}
fn write_json(root: &Path, name: &str, value: &impl serde::Serialize) -> TestResult<Value> {
    let bytes = wow_core::canonical_json_bytes(value)?;
    write_new(root, name, &bytes, 64 * 1024)?;
    Ok(json!({"path":name,"content_digest":digest(&bytes),"byte_length":bytes.len()}))
}
fn report() -> TestResult<Value> {
    // Existing platform transport fixture pattern, imported through the genuine
    // backend validator; synthetic compatibility data is not donor qualification.
    let mut report = json!({
        "schema":"wow-dev-framework/emmylua-compatibility-report",
        "schema_version":1,"producer_version":1,
        "source":{"revision":EMMYLUA_REVISION,"tree":EMMYLUA_TREE,
            "relation":"unverified_current","network_checked":false},
        "analysis_crate":{"manifest_path":"crates/emmylua_code_analysis/Cargo.toml",
            "name":"emmylua_code_analysis","version":EMMYLUA_CODE_ANALYSIS_VERSION},
        "surface":{"files":[],"symbols":[],"surface_sha256":format!("sha256:{}","3".repeat(64))},
        "compatibility":{"required_symbols":[],"missing_symbols":[],"status":"compatible"}
    });
    report["report_sha256"] = json!(format!(
        "sha256:{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&report)?)
    ));
    wow_emmy::backend_identity_from_report(&wow_core::canonical_json_bytes(&report)?)?;
    Ok(report)
}
struct Fixture {
    profile: ProfileIdentity,
    reference: ReferenceView,
}
impl Fixture {
    fn new() -> TestResult<Self> {
        let profile = ProfileIdentityBuilder::new(
            "profile:fixture:project-pair-regression-v1".parse()?,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "fixture:live-pair-v1",
            ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),
        )
        .schema_versions(vec![SchemaVersionEntry::new(
            "schema:wow:fixture-e0".parse()?,
            "1.0.0".parse()?,
        )])
        .fixture_scope("native live pair regression")
        .build()?;
        let reference = ReferenceView::new(REFERENCE_GENERATION, Vec::new(), Vec::new())?;
        reference.validate()?;
        Ok(Self { profile, reference })
    }
    fn input(
        &self,
        root: &Path,
        declarations: bool,
        signatures: bool,
        removed: bool,
    ) -> TestResult<Value> {
        std::fs::create_dir(root)?;
        std::fs::create_dir(root.join("main"))?;
        std::fs::create_dir(root.join("library"))?;
        write_new(root, "main/main.lua", MAIN.as_bytes(), 4096)?;
        if removed {
            write_new(root, "main/removed.lua", REMOVED.as_bytes(), 4096)?;
        }
        write_new(root, "library/Core.lua", LIBRARY.as_bytes(), 4096)?;
        let profile = write_json(root, "profile.json", &self.profile)?;
        let reference = write_json(root, "reference.json", &self.reference)?;
        let report = write_json(root, "compatibility.json", &report()?)?;
        let mut files = vec![json!({"path":"main.lua",
            "content_digest":digest(MAIN.as_bytes()),"byte_length":MAIN.len()})];
        if removed {
            files.push(json!({"path":"removed.lua",
                "content_digest":digest(REMOVED.as_bytes()),"byte_length":REMOVED.len()}));
        }
        let mut input = json!({
            "schema":if declarations || signatures { INPUT_SCHEMA } else {
                "wow-service/local-project-files/1"
            },
            "project_id":PROJECT,"workspace_id":WORKSPACE_ID,
            "source_origin_id":SOURCE_ORIGIN_ID,
            "logical_root":LOGICAL_ROOT,
            "profile":profile,"reference_view":reference,
            "analyzer":{
                "compatibility_report":report,
                "accepted_pin_id":format!("emmy-pin:{EMMYLUA_REVISION}"),
                "configuration_digest":ANALYZER_CONFIGURATION_DIGEST,
                "contract_id":ANALYZER_CONTRACT_ID,
                "fixture_contract_id":ANALYZER_CONTRACT_ID,
                "library_contract_id":ANALYZER_LIBRARY_CONTRACT_ID
            },
            "main":{"root":"main","files":files},
            "library":{"root":"library","files":[{
                "path":"Core.lua","content_digest":digest(LIBRARY.as_bytes()),"byte_length":LIBRARY.len()
            }]}
        });
        if declarations {
            input["physical_function_declarations"] =
                serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?;
        }
        if signatures {
            input["physical_function_signatures"] =
                serde_json::to_value(PhysicalFunctionSignatureRecipe::IndexedTypesV1)?;
        }
        write_json(root, "project.json", &input)?;
        Ok(input)
    }
    fn host(&self, store: &Path) -> Value {
        json!({
            "schema":CONTEXT_MAP_HOST_SCHEMA,"project_store_id":REGISTRATION,
            "project_store_root":store,"project_id":PROJECT,
            "reference_profile":self.profile,"reference":self.reference,
            "limits":{
                "neighbors":GraphNeighborReadLimits::default(),
                "explain":GraphExplainLimits {
                    max_output_bytes:64 * 1024,..GraphExplainLimits::default()
                },
                "evidence":GraphEvidenceResolveLimits::default(),
                "max_direct_edges":16,"max_input_bytes":4 * 1024 * 1024,
                "max_metadata_bytes":4 * 1024 * 1024,"max_context_work":8_000_000
            }
        })
    }
    fn guard(&self) -> TestResult<ContextMapReferenceGuard> {
        Ok(ContextMapReferenceGuard::new(
            self.profile.profile_id().as_str(),
            self.reference.generation_id(),
            self.reference.self_digest(),
        )?)
    }
}
fn catalog(schemas: &[&'static str]) -> TestResult<RecordCatalog> {
    let mut registered = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();
    registered.extend_from_slice(schemas);
    Ok(RecordCatalog::new(
        &registered,
        &[
            GraphPartitionSnapshot::STORAGE_CHECK,
            publication::STORAGE_CHECK,
        ],
    )?)
}
fn raw_replay(root: &Path, selected: bool, stop: &AtomicBool) -> TestResult<(Value, Value)> {
    let store = ProjectStore::open(root, &catalog(publication::STORAGE_SCHEMAS)?)?;
    let epoch = serde_json::to_value(store.epoch())?;
    let read = store.read(&ReadSelector::Current, stop)?;
    let member = read
        .manifest()
        .members
        .iter()
        .find(|member| member.key == "live.project.replay")
        .ok_or("actual retained replay membership")?;
    assert_eq!(
        member.schema,
        if selected {
            STORAGE_SCHEMA
        } else {
            "wow-project.live-replay"
        }
    );
    let replay: ProjectReplay = read
        .record("live.project.replay", stop)?
        .ok_or("retained replay record")?
        .decode()?;
    let wire = serde_json::to_value(&replay)?;
    read.close()?;
    store.close()?;
    Ok((epoch, wire))
}
struct Native {
    current: CurrentPublication,
    root: GraphNodeId,
    metadata: Value,
    original: Value,
}
fn retained(
    root: &Path,
    selector: &ReadSelector,
    declarations: bool,
    signatures: bool,
    main_count: usize,
    stop: &AtomicBool,
) -> TestResult<Native> {
    let store = LiveProjectStore::open(root)?;
    let read = store.read(selector, stop)?;
    assert!(read.namespace().is_none() && read.candidate().is_none());
    let configuration = read.project().configuration();
    assert_eq!(configuration.project_kind(), ProjectKind::Fixture);
    assert_eq!(
        configuration.physical_function_declarations(),
        declarations.then_some(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)
    );
    assert_eq!(
        configuration.physical_function_signatures(),
        signatures.then_some(PhysicalFunctionSignatureRecipe::IndexedTypesV1)
    );
    assert!(configuration.load_plan().is_none());
    assert!(configuration.package_load_plan().is_none());
    assert!(configuration.platform_packages().is_none());
    assert!(configuration.platform_graph_profile().is_none());
    assert_eq!(read.project().snapshot().file_manifest().len(), main_count);
    let provenance = read
        .pair()
        .source_provenance()
        .ok_or("genuine ordinary source provenance")?;
    let report = provenance
        .function_call_report()
        .ok_or("genuine native callable report")?;
    report.validate()?;
    assert_eq!(
        report.declaration_recipe(),
        configuration.physical_function_declarations()
    );
    assert_eq!(
        report.signature_recipe(),
        configuration.physical_function_signatures()
    );
    assert!(report.source_health_complete());
    assert_eq!(provenance.files().len(), main_count);
    assert_eq!(
        provenance
            .functions()
            .iter()
            .filter(|row| row.path == "removed.lua")
            .count(),
        if main_count == 2 { 2 } else { 0 },
        "removed physical Chunk/Closure population"
    );
    let closures = report
        .functions()
        .iter()
        .filter(|fact| fact.path() == "main.lua" && fact.kind() == SourceFunctionKind::Closure)
        .collect::<Vec<_>>();
    assert_eq!(closures.len(), 1, "one explicit fixture Main closure");
    let fact = closures[0];
    if declarations {
        assert!(matches!(
            fact.declaration(),
            Some(SourceFunctionDeclaration::Observed { .. })
        ));
    } else {
        assert!(fact.declaration().is_none());
    }
    if signatures {
        let comparable = fact
            .signature()
            .and_then(|signature| signature.comparable())
            .ok_or("genuine supported number signature")?;
        assert_eq!(comparable.parameters().len(), 1);
        assert_eq!(
            comparable.parameters()[0].type_shape(),
            &SourceSignatureType::Number
        );
    } else {
        assert!(fact.signature().is_none());
    }
    let function = provenance
        .functions()
        .iter()
        .find(|row| row.function_id == fact.fact_id())
        .ok_or("exact native function join")?;
    let proof = read.pair().graph_read(stop)?;
    let node = proof.published_entity(SOURCE_GRAPH_PARTITION, &function.proposal_id, stop)?;
    assert!(read.graph().snapshot().node(node.node_id()).is_some());
    let source = read
        .graph()
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or("native source partition")?;
    let mut original_relations = Vec::new();
    for accepted in source.report().accepted_relations() {
        original_relations.push(
            source
                .batch()
                .relation_proposal(accepted.proposal_id())
                .ok_or("original accepted relation")?,
        );
    }
    let metadata = serde_json::to_value(
        provenance
            .callable_metadata(&function.function_id)?
            .ok_or("sealed native callable metadata")?,
    )?;
    let result = Native {
        current: read
            .current_at_acquisition()
            .ok_or("actual Current")?
            .clone(),
        root: node.node_id().clone(),
        metadata,
        original: json!({
            "project_snapshot":read.project().snapshot_id(),
            "publication_set":read.publication_set_id(),
            "graph":read.graph(),"provenance":provenance,
            "configuration":configuration,
            "entity_proposals":source.batch().entity_proposals(),
            "accepted_entities":source.report().accepted_entities(),
            "relation_proposals":original_relations,
            "accepted_relations":source.report().accepted_relations(),
            "evidence":provenance.evidence().values().collect::<Vec<_>>(),
            "handles":provenance.source_handles().values().collect::<Vec<_>>()
        }),
    };
    read.close()?;
    store.close()?;
    Ok(result)
}
fn assert_context(result: &ContextMapResult, expected: &Native, build: bool) -> TestResult {
    assert_eq!(result.status(), ContextMapStatus::Partial);
    assert_eq!(result.failure_code(), None);
    let envelope: Value = serde_json::from_slice(result.canonical_bytes())?;
    assert_eq!(envelope["resource_closure"], "closed");
    assert_eq!(
        envelope["schema"],
        if build {
            "wow-service/context-build-result/1"
        } else {
            "wow-service/context-inspect-result/1"
        }
    );
    let payload = &envelope["payload"];
    let skeletons = if build {
        &payload["semantic_pack"]["core"]["skeletons"]
    } else {
        &payload["skeletons"]
    };
    assert_eq!(skeletons["level"], "l1");
    let core = &skeletons["l1"]["core"];
    assert_eq!(core["root"]["node_id"], expected.root.as_str());
    let declarations = core["declarations"]
        .as_array()
        .ok_or("genuine native callable declarations")?;
    let exact = declarations
        .iter()
        .filter(|row| row["node"]["node_id"] == expected.root.as_str())
        .collect::<Vec<_>>();
    assert_eq!(exact.len(), 1);
    assert_eq!(exact[0]["kind"], "callable");
    assert_eq!(exact[0]["metadata"], expected.metadata);
    assert_eq!(core["context"]["absence_authoritative"], false);
    let origin = &core["context"]["origins"]["core"];
    let supports = origin["supports"]
        .as_array()
        .ok_or("mandatory original native support")?;
    assert!(!supports.is_empty());
    for receipt in supports {
        assert_eq!(receipt["explanation"]["support_complete"], true);
        for support in receipt["explanation"]["supports"]
            .as_array()
            .ok_or("native supports")?
        {
            let (proposals, accepted) = match support["origin"].as_str() {
                Some("producer_entity") => ("entity_proposals", "accepted_entities"),
                Some("producer_relation") => ("relation_proposals", "accepted_relations"),
                _ => return Err("unexpected synthetic foundation support".into()),
            };
            assert!(
                expected.original[proposals]
                    .as_array()
                    .ok_or("retained proposals")?
                    .contains(&support["proposal"])
            );
            assert!(
                expected.original[accepted]
                    .as_array()
                    .ok_or("retained acceptance")?
                    .contains(&support["accepted"])
            );
        }
        assert_eq!(receipt["evidence_resolution"]["closure_complete"], true);
        for (actual, retained) in [
            ("evidence_records", "evidence"),
            ("source_handles", "handles"),
        ] {
            for row in receipt["evidence_resolution"][actual]
                .as_array()
                .ok_or("resolved evidence")?
            {
                assert!(
                    expected.original[retained]
                        .as_array()
                        .ok_or("retained evidence")?
                        .contains(row)
                );
            }
        }
    }
    if build {
        assert_eq!(
            payload["semantic_pack"]["core"]["absence_authoritative"],
            false
        );
        assert!(
            payload["rendered_artifacts"]
                .as_array()
                .ok_or("render choices")?
                .is_empty()
        );
    }
    let serialized = std::str::from_utf8(result.canonical_bytes())?;
    for excluded in [
        MAIN.lines().next().ok_or("source prefix")?,
        "local function TypedSignatureFixture",
        "function External",
        "project_store_root",
    ] {
        assert!(
            !serialized.contains(excluded),
            "no raw source body or host custody path"
        );
    }
    Ok(())
}
fn context(
    fixture: &Fixture,
    root: &Path,
    expected: &Native,
    stop: &AtomicBool,
) -> TestResult<(Value, ContextBuildRequest, ContextMapResult)> {
    let host_value = fixture.host(root);
    let host =
        ContextMapHostInput::from_json_slice(&wow_core::canonical_json_bytes(&host_value)?, stop)?;
    let inspect = ContextInspectRequest::primary(
        REGISTRATION,
        ContextMapPublication::store_generation(expected.current.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(expected.root.as_str())?,
        None,
        Some(fixture.guard()?),
    )?;
    let inspected = execute_context_inspect(&host, &inspect, stop)?;
    assert_context(&inspected, expected, false)?;
    let build = ContextBuildRequest::primary(
        REGISTRATION,
        ContextMapPublication::store_generation(expected.current.generation_id.as_str())?,
        ContextInspectLevel::L1,
        ContextMapRoot::graph_entity(expected.root.as_str())?,
        ContextRenderSelection::None,
        None,
        Some(fixture.guard()?),
    )?;
    let built = execute_context_build(&host, &build, stop)?;
    assert_context(&built, expected, true)?;
    let cancelled = execute_context_inspect(&host, &inspect, &AtomicBool::new(true))?;
    assert_eq!(cancelled.status(), ContextMapStatus::Cancelled);
    assert_eq!(cancelled.failure_code(), Some(ServiceErrorCode::Cancelled));
    let cancelled: Value = serde_json::from_slice(cancelled.canonical_bytes())?;
    assert!(cancelled.get("payload").is_none());
    Ok((host_value, build, built))
}
fn root_token(id: &GraphNodeId) -> String {
    // Same mechanical base64url transport pattern as context/tests.rs.
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut encoded = String::new();
    for group in id.as_str().as_bytes().chunks(3) {
        encoded.push(ALPHABET[(group[0] >> 2) as usize] as char);
        encoded.push(
            ALPHABET[(((group[0] & 3) << 4) | (group.get(1).copied().unwrap_or(0) >> 4)) as usize]
                as char,
        );
        if group.len() > 1 {
            encoded.push(
                ALPHABET
                    [(((group[1] & 15) << 2) | (group.get(2).copied().unwrap_or(0) >> 6)) as usize]
                    as char,
            );
        }
        if group.len() > 2 {
            encoded.push(ALPHABET[(group[2] & 63) as usize] as char);
        }
    }
    format!("graph-entity@{encoded}")
}
fn export(
    scratch: &Scratch,
    fixture: &Fixture,
    store: &Path,
    expected: &Native,
    host: &Value,
    request: &ContextBuildRequest,
    result: &ContextMapResult,
) -> TestResult {
    if !scratch.export {
        return Ok(());
    }
    // Deliberately written only AFTER independently proving source-absent
    // Current/Exact hydration, update/removal and context acquisition.
    let input = scratch.root.join("cli-input");
    fixture.input(&input, true, true, false)?;
    write_json(&scratch.root, "host.json", host)?;
    write_json(&scratch.root, "build-request.json", request)?;
    write_new(
        &scratch.root,
        "expected-build.json",
        result.canonical_bytes(),
        16 * 1024 * 1024,
    )?;
    let shared = json!([
        "--native-host-config",
        scratch.root.join("host.json"),
        "--project-store",
        REGISTRATION,
        "--publication",
        format!(
            "store-generation:{}",
            expected.current.generation_id.as_str()
        ),
        "--root",
        root_token(&expected.root),
        "--level",
        "l1",
        "--platform",
        "omitted",
        "--expect-profile",
        fixture.profile.profile_id().as_str(),
        "--expect-reference-generation",
        fixture.reference.generation_id(),
        "--expect-reference-view",
        fixture.reference.self_digest(),
        "--output",
        "envelope-json"
    ]);
    let common = shared.as_array().ok_or("CLI tokens")?;
    let mut build = vec![json!("context"), json!("build")];
    build.extend(common.iter().cloned());
    build.extend([
        json!("--context-profile-id"),
        json!(PRIMARY_CONTEXT_PACK_PROFILE_ID),
    ]);
    let mut inspect = vec![json!("context"), json!("inspect")];
    inspect.extend(common.iter().cloned());
    inspect.extend([json!("--l1-profile-id"), json!(PRIMARY_L1_PROFILE_ID)]);
    let arguments = json!({
        "project_publish":["project","publish","--config",input.join("project.json"),
            "--project",PROJECT,"--store-root",scratch.root.join("cli-new-store"),
            "--operation-id","fixture:primary-cli-publish","--expected-current","absent",
            "--initialize","--allow-partial","--format","json"],
        "project_current":["project","read","--store-root",store,"--store-generation","current","--format","json"],
        "project_exact":["project","read","--store-root",store,"--store-generation",
            expected.current.generation_id.as_str(),"--format","json"],
        "context_inspect":inspect,"context_build":build
    });
    write_json(&scratch.root, "cli-arguments.json", &arguments)?;
    write_json(
        &scratch.root,
        "fixture.json",
        &json!({
            "synthetic":true,"authority":"fixture_only_not_live_or_ketho",
            "source_absence":"test input and update directories removed before independent Current/Exact/context reads",
            "cli_input":"separate reproduction written after absence proof; exporting it is not absence evidence",
            "context_cli_exit":2,"context_stdout":"expected-build.json plus one LF",
            "context_stderr":"empty on success","publication":"exact retained combined-selected update",
            "primary_recipes_retained":true,"semantic_host_paths":"excluded"
        }),
    )?;
    Ok(())
}
#[test]
fn primary_callable_files_publish_reopen_context_and_update_without_source_inputs() -> TestResult {
    let stop = AtomicBool::new(false);
    let scratch = Scratch::new()?;
    let fixture = Fixture::new()?;
    let graph = GraphBuildRequest::new(PROJECT.into(), "current".into())?;
    // Exactly the CLI's bare graph request. The public config-loading boundary
    // binds the admitted recipes; no manual with_input_profile bypass in this test.
    let old_input = scratch.root.join("old-input");
    let old_config = fixture.input(&old_input, false, false, true)?;
    let old_store = scratch.root.join("old-store");
    publish_local_project(
        &old_input.join("project.json"),
        &graph,
        &old_store,
        &LiveProjectPublishRequest::new("fixture:primary-old", "absent", true, true)?,
        &stop,
    )?;
    let old = retained(&old_store, &ReadSelector::Current, false, false, 2, &stop)?;
    let (old_epoch, old_replay) = raw_replay(&old_store, false, &stop)?;
    for field in [
        "physical_function_declarations",
        "physical_function_signatures",
    ] {
        assert!(old_replay["configuration"].get(field).is_none());
        assert!(old_config.get(field).is_none());
    }
    for (name, declarations, signatures) in [
        ("declarations", true, false),
        ("signatures", false, true),
        ("both", true, true),
    ] {
        let input = scratch.root.join(format!("{name}-input"));
        let selected_config = fixture.input(&input, declarations, signatures, true)?;
        if declarations && signatures {
            // One closed-schema admission control group; no resource matrix.
            let mut mutations = BTreeMap::new();
            let mut old_schema = selected_config.clone();
            old_schema["schema"] = json!("wow-service/local-project-files/1");
            mutations.insert("old-schema", old_schema);
            for field in [
                "physical_function_declarations",
                "physical_function_signatures",
            ] {
                let mut null = selected_config.clone();
                null[field] = Value::Null;
                mutations.insert(field, null);
            }
            let mut body = selected_config.clone();
            body["physical_function_declarations"] = serde_json::to_value(
                PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1,
            )?;
            mutations.insert("body-shape", body);
            let mut unknown = selected_config.clone();
            unknown["physical_function_signatures"] = json!("unsupported");
            mutations.insert("unsupported-signature", unknown);
            let mut empty = selected_config.clone();
            empty
                .as_object_mut()
                .ok_or("input")?
                .remove("physical_function_declarations");
            empty
                .as_object_mut()
                .ok_or("input")?
                .remove("physical_function_signatures");
            mutations.insert("empty-selection", empty);
            let mut toc = selected_config.clone();
            toc["main"] = json!({"root":"main","toc":{"path":"Main.toc"}});
            mutations.insert("toc", toc);
            for (label, mutation) in mutations {
                // Malformed input is intentionally outside the semantic
                // canonical subset (including null); the decoder must refuse it.
                write_new(
                    &input,
                    &format!("{label}.json"),
                    &serde_json::to_vec(&mutation)?,
                    64 * 1024,
                )?;
                assert_eq!(
                    LocalProjectInput::from_config_path(
                        &input.join(format!("{label}.json")),
                        &stop
                    )
                    .err()
                    .ok_or("closed family admitted invalid selection")?
                    .code(),
                    ServiceErrorCode::InvalidConfiguration
                );
            }
            assert_eq!(
                publish_local_project(
                    &input.join("project.json"),
                    &graph,
                    &old_store,
                    &LiveProjectPublishRequest::new(
                        "fixture:primary-cancel",
                        old.current.record_id.as_str(),
                        false,
                        true
                    )?,
                    &AtomicBool::new(true)
                )
                .err()
                .ok_or("entry cancellation admitted publication")?
                .code(),
                ServiceErrorCode::Cancelled
            );
            let unchanged = retained(&old_store, &ReadSelector::Current, false, false, 2, &stop)?;
            assert_eq!(unchanged.current, old.current);
            assert_eq!(unchanged.original, old.original);
            assert_eq!(
                raw_replay(&old_store, false, &stop)?,
                (old_epoch.clone(), old_replay.clone())
            );
        }
        let store = scratch.root.join(format!("{name}-store"));
        let published = publish_local_project(
            &input.join("project.json"),
            &graph,
            &store,
            &LiveProjectPublishRequest::new(
                &format!("fixture:primary-{name}"),
                "absent",
                true,
                true,
            )?,
            &stop,
        )?;
        assert_eq!(serde_json::to_value(published)?["status"], "activated");
        let initial = retained(
            &store,
            &ReadSelector::Current,
            declarations,
            signatures,
            2,
            &stop,
        )?;
        let (_, wire) = raw_replay(&store, true, &stop)?;
        for (field, selected) in [
            ("physical_function_declarations", declarations),
            ("physical_function_signatures", signatures),
        ] {
            assert_eq!(wire["configuration"].get(field), selected_config.get(field));
            if !selected {
                assert!(wire["configuration"].get(field).is_none());
            }
        }
        // Delete the entire registered input, including Main, exact Library
        // and pinned metadata. Reopen receives only the retained store selector.
        std::fs::remove_dir_all(&input)?;
        assert!(!input.exists());
        let current = retained(
            &store,
            &ReadSelector::Current,
            declarations,
            signatures,
            2,
            &stop,
        )?;
        let exact = retained(
            &store,
            &ReadSelector::Exact(initial.current.generation_id.clone()),
            declarations,
            signatures,
            2,
            &stop,
        )?;
        assert_eq!(current.current, initial.current);
        assert_eq!(current.original, initial.original);
        assert_eq!(exact.original, initial.original);
        context(&fixture, &store, &exact, &stop)?;

        if declarations && signatures {
            let update_input = scratch.root.join("update-input");
            fixture.input(&update_input, true, true, false)?;
            let updated = update_local_project(
                &update_input.join("project.json"),
                &graph,
                &store,
                &LiveProjectUpdateRequest::new(
                    "fixture:primary-remove",
                    initial.current.record_id.as_str(),
                    LiveProjectLibraryMode::Keep,
                    true,
                )?,
                &stop,
            )?;
            assert_eq!(serde_json::to_value(updated)?["status"], "activated");
            std::fs::remove_dir_all(&update_input)?;
            assert!(!update_input.exists() && !input.exists());
            let next = retained(&store, &ReadSelector::Current, true, true, 1, &stop)?;
            assert_ne!(next.current.generation_id, initial.current.generation_id);
            let next_exact = retained(
                &store,
                &ReadSelector::Exact(next.current.generation_id.clone()),
                true,
                true,
                1,
                &stop,
            )?;
            assert_eq!(next.original, next_exact.original);
            let old_exact = retained(
                &store,
                &ReadSelector::Exact(initial.current.generation_id.clone()),
                true,
                true,
                2,
                &stop,
            )?;
            assert_eq!(
                old_exact.original, initial.original,
                "retained prior generation stays exact"
            );
            assert!(
                next.original["provenance"]["functions"]
                    .as_array()
                    .ok_or("native functions")?
                    .iter()
                    .all(|row| row["path"] != "removed.lua")
            );
            let (_, updated_wire) = raw_replay(&store, true, &stop)?;
            assert_eq!(
                updated_wire["configuration"]["physical_function_declarations"],
                serde_json::to_value(PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1)?
            );
            assert_eq!(
                updated_wire["configuration"]["physical_function_signatures"],
                serde_json::to_value(PhysicalFunctionSignatureRecipe::IndexedTypesV1)?
            );
            let (host, request, result) = context(&fixture, &store, &next_exact, &stop)?;
            export(
                &scratch,
                &fixture,
                &store,
                &next_exact,
                &host,
                &request,
                &result,
            )?;
        }
    }
    Ok(())
}
