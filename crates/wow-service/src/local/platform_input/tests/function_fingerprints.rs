//! One native fixture proves the selected read-only observation and its refusal seam.
use super::*;
use crate::{
    LocalProjectBackend, ServiceErrorCode,
    graph::{
        FunctionFingerprintSelection, GraphBuildRequest, GraphBuildResult, GraphReadStatus,
        execute_graph_build,
    },
};
use wow_emmy::function_calls::{SourceFunctionFact, SourceFunctionKind};
use wow_graph::{
    GraphAssertionKind, GraphCoverageRecord, GraphLocalAssertion, GraphPartitionReplacement,
    GraphPartitionSnapshot, GraphProposalBatch,
};
use wow_project::{
    ProjectView,
    graph::{
        build_platform_graph_proposal_plan,
        fingerprints::{FunctionFingerprintLimits, build_platform_function_fingerprints},
    },
    replay::{ProjectReplay, publication::ProjectPublicationBundle},
};

const METADATA_BYTES: usize = 256 * 1024;

fn span_bytes<'a>(project: &'a ProjectView, fact: &SourceFunctionFact) -> TestResult<&'a [u8]> {
    let file = project
        .snapshot()
        .analyzer_binding()
        .main_workspace()
        .file(fact.path())
        .ok_or("native Main file")?;
    if fact.kind() == SourceFunctionKind::Chunk && fact.span() == wow_core::SourceSpan::whole_file()
    {
        return Ok(file.text().as_bytes());
    }
    let start = usize::try_from(fact.span().byte_start().ok_or("exact span start")?)?;
    let end = usize::try_from(fact.span().byte_end().ok_or("exact span end")?)?;
    Ok(file
        .text()
        .get(start..end)
        .ok_or("native UTF-8 span")?
        .as_bytes())
}

fn admit(
    owner: &GraphPartitionSnapshot,
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
    version: &str,
    stop: &AtomicBool,
) -> TestResult<GraphPartitionSnapshot> {
    Ok(owner
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: version.into(),
                batch,
                coverage,
            },
            stop,
        )?
        .candidate()
        .clone())
}

fn observe(
    project: &ProjectView,
    ids: &[&str],
    source_bytes: usize,
    stop: &AtomicBool,
) -> TestResult<Value> {
    let mut plan = build_platform_graph_proposal_plan(project, stop)?;
    let mut owner = GraphPartitionSnapshot::new(
        plan.registry().clone(),
        plan.foundation().clone(),
        plan.scope().source_context_id,
        stop,
    )?;
    owner = admit(
        &owner,
        plan.raw_inventory_batch()
            .ok_or("native raw inventory")?
            .clone(),
        Vec::new(),
        plan.raw_inventory_producer_version(),
        stop,
    )?;
    for &producer in plan.producer_order() {
        let stage = plan.build_stage(producer, &owner, stop)?;
        let version = stage.producer_version().to_owned();
        let (batch, coverage) = stage.into_parts();
        owner = admit(&owner, batch, coverage, &version, stop)?;
    }
    let direct = plan.finish(&owner, stop)?;
    let manifest = build_platform_function_fingerprints(
        &direct,
        ids,
        FunctionFingerprintLimits::new(ids.len(), source_bytes, METADATA_BYTES)?,
        stop,
    )?;
    let wire = serde_json::to_value(&manifest)?;
    assert_eq!(
        wire["binding"]["project_snapshot_id"],
        project.snapshot_id()
    );
    assert_eq!(
        wire["binding"]["configuration_digest"],
        serde_json::to_value(project.configuration().configuration_digest())?
    );
    assert_eq!(
        wire["binding"]["analyzer_snapshot_id"],
        project.analyzer_snapshot_id()
    );
    assert_eq!(
        wire["binding"]["graph_scope"],
        serde_json::to_value(direct.scope())?
    );
    assert_eq!(
        wire["binding"]["published_graph_snapshot_id"],
        serde_json::to_value(owner.snapshot().snapshot_id())?
    );

    // Resolve assertions on the actual native graph, never on LocalProjectBackend.
    let lookup = direct.graph().producer_lookup(stop)?;
    for record in wire["records"].as_array().ok_or("fingerprint records")? {
        let function = direct
            .source()
            .functions()
            .iter()
            .find(|row| Some(row.function_id.as_str()) == record["function_id"].as_str())
            .ok_or("native selected function")?;
        let entity = direct
            .assertion(&GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: function.proposal_id.as_str().into(),
            })
            .ok_or("native entity assertion")?;
        let ownership = direct
            .assertion(&GraphLocalAssertion {
                kind: GraphAssertionKind::Relation,
                proposal_id: function.ownership_proposal_id.as_str().into(),
            })
            .ok_or("native ownership assertion")?;
        let accepted = lookup.entity(direct.scope(), entity, stop)?;
        let incoming = lookup.relation(direct.scope(), ownership, stop)?;
        assert_eq!(
            incoming.proposal().relation_kind_id(),
            "source_declaration_owns"
        );
        assert_eq!(
            incoming.accepted().edge().to(),
            accepted.accepted().node().node_id()
        );
        let handle = direct
            .source()
            .source_handles()
            .get(&function.source_handle_id)
            .ok_or("native function source handle")?;
        let evidence = direct
            .source()
            .evidence()
            .get(&function.evidence_id)
            .ok_or("native function evidence")?;
        assert_eq!(
            record["exact_owner"]["scope"],
            serde_json::to_value(direct.scope())?
        );
        assert_eq!(
            record["exact_owner"]["node_id"],
            serde_json::to_value(accepted.accepted().node().node_id())?
        );
        assert_eq!(
            record["exact_owner"]["owner_key"],
            accepted.accepted().node().owner_key()
        );
        assert_eq!(
            record["support"]["entity_assertion"],
            serde_json::to_value(entity)?
        );
        assert_eq!(
            record["support"]["incoming_ownership_assertion"],
            serde_json::to_value(ownership)?
        );
        assert_eq!(
            record["support"]["source_handle_id"],
            serde_json::to_value(function.source_handle_id)?
        );
        assert_eq!(
            record["support"]["evidence_id"],
            serde_json::to_value(function.evidence_id)?
        );
        assert_eq!(record["support"]["path"], handle.path().as_str());
        assert_eq!(
            record["support"]["span"],
            serde_json::to_value(handle.span())?
        );
        assert_eq!(
            record["support"]["file_content_digest"],
            serde_json::to_value(handle.content_digest())?
        );
        assert!(
            evidence
                .source_handle_ids()
                .contains(&function.source_handle_id)
        );
        assert_eq!(evidence.context_id(), direct.scope().source_context_id);
    }
    Ok(wire)
}

fn cleared(result: &GraphBuildResult) -> TestResult {
    assert!(result.function_fingerprints().is_none());
    assert!(result.snapshot_bytes()?.is_none());
    let wire = serde_json::to_value(result)?;
    assert!(wire.get("function_fingerprints").is_none());
    assert!(wire.get("provenance").is_none());
    Ok(())
}

#[test]
fn native_function_source_span_query_is_selected_bounded_and_recomputed_from_replay() -> TestResult
{
    let stop = AtomicBool::new(false);
    let (root, mut config) = native_fixture()?;
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
        ])?,
    ))?;
    write_json(&root, "project.json", &config)?;
    let path = root.0.join("project.json");
    let input = LocalProjectInput::from_config_path(&path, &stop)?;
    let configuration_bytes = serde_json::to_vec(input.project_bundle().configuration())?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let backend =
        LocalProjectBackend::for_graph(LocalProjectInput::from_config_path(&path, &stop)?)?;
    let project = request.acquire_project(&backend, &stop)?;
    assert_eq!(
        serde_json::to_vec(project.configuration())?,
        configuration_bytes
    );
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("genuine physical Main function report")?;
    let ids = report
        .functions()
        .iter()
        .map(|fact| Box::<str>::from(fact.fact_id()))
        .collect::<Vec<_>>();
    assert!(
        ids.len() >= 2,
        "the existing fixture has a chunk and closure"
    );
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    let source_bytes =
        report
            .functions()
            .iter()
            .try_fold(0_usize, |sum, fact| -> TestResult<usize> {
                Ok(sum
                    .checked_add(span_bytes(&project, fact)?.len())
                    .ok_or("source byte sum")?)
            })?;
    assert!(source_bytes > 1);
    let borrowed_ids = ids.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
    let expected = observe(&project, &borrowed_ids, source_bytes, &stop)?;

    let plain = execute_graph_build(input, &request, &stop)?;
    assert_eq!(plain.status(), GraphReadStatus::Partial);
    assert!(plain.function_fingerprints().is_none());
    let plain_wire = serde_json::to_value(&plain)?;
    assert_eq!(plain_wire["schema"], "wow-service/graph-build-result/20");
    assert_eq!(
        plain_wire["request"]["schema"],
        "wow-service/graph-build-request/13"
    );
    assert!(plain_wire.get("function_fingerprints").is_none());
    assert!(plain_wire["request"].get("function_fingerprints").is_none());
    let plain_graph: GraphPartitionSnapshot =
        serde_json::from_slice(&plain.snapshot_bytes()?.ok_or("unselected graph")?)?;
    let (plain_records, _) = backend
        .capture_project_bundle(&plain_graph, &stop)?
        .into_parts();
    let before = plain_records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("unselected replay record")?;

    let selection =
        FunctionFingerprintSelection::new(ids.clone(), ids.len(), source_bytes, METADATA_BYTES)?;
    let selection_wire = serde_json::to_value(&selection)?;
    assert_eq!(
        selection_wire["schema"],
        "wow-service/function-fingerprint-selection/1"
    );
    let decoded = FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&selection)?)?;
    let selected_request = request.clone().with_function_fingerprints(decoded)?;
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &selected_request,
        &stop,
    )?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{:?}",
        result.failure_message()
    );
    let manifest = result
        .function_fingerprints()
        .ok_or("typed selected manifest")?;
    assert_eq!(
        manifest.schema(),
        "wow-project/function-fingerprint-records/1"
    );
    assert_eq!(
        manifest.profile(),
        "wow-project/physical-function-source-span/1"
    );
    let wire = serde_json::to_value(manifest)?;
    assert_eq!(wire, expected);
    assert_eq!(wire["coverage"], "partial");
    assert_eq!(manifest.records().len(), ids.len());
    for (record, fact) in wire["records"]
        .as_array()
        .ok_or("selected records")?
        .iter()
        .zip(report.functions())
    {
        let bytes = span_bytes(&project, fact)?;
        assert_eq!(record["function_id"], fact.fact_id());
        assert_eq!(record["source_span"]["state"], "available");
        assert_eq!(
            record["source_span"]["content_sha256"],
            serde_json::to_value(digest(bytes))?
        );
        assert_eq!(record["source_span"]["byte_length"], bytes.len() as u64);
        assert_eq!(record["signature"], "missing_signature");
        assert_eq!(record["declaration_shape"], "missing_shape");
        assert_eq!(record["direct_neighborhood"], "not_selected");
        assert_eq!(record["package_load_role"], "not_selected");
    }
    let selected_wire = serde_json::to_value(&result)?;
    assert_eq!(
        selected_wire["schema"],
        "wow-service/graph-build-result/function-fingerprints/1"
    );
    assert_eq!(
        selected_wire["request"]["schema"],
        "wow-service/graph-build-request/function-fingerprints/1"
    );

    // Record admission refuses before native work; these are the same genuine IDs.
    let mut too_many = selection_wire;
    too_many["max_records"] = json!(ids.len() - 1);
    assert_eq!(
        FunctionFingerprintSelection::from_json_slice(&serde_json::to_vec(&too_many)?)
            .err()
            .ok_or("record cap ignored")?
            .code(),
        ServiceErrorCode::BudgetExceeded
    );
    for (source_limit, metadata_limit) in [(1, METADATA_BYTES), (source_bytes, 1)] {
        let bounded =
            request
                .clone()
                .with_function_fingerprints(FunctionFingerprintSelection::new(
                    ids.clone(),
                    ids.len(),
                    source_limit,
                    metadata_limit,
                )?)?;
        let refused = execute_graph_build(
            LocalProjectInput::from_config_path(&path, &stop)?,
            &bounded,
            &stop,
        )?;
        assert_eq!(refused.status(), GraphReadStatus::Failed);
        assert_eq!(
            serde_json::to_value(&refused)?["failure"],
            serde_json::to_value(ServiceErrorCode::BudgetExceeded)?
        );
        cleared(&refused)?;
    }
    let pre_cancelled = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &selected_request,
        &AtomicBool::new(true),
    )?;
    assert_eq!(pre_cancelled.status(), GraphReadStatus::Cancelled);
    cleared(&pre_cancelled)?;

    let graph: GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("selected graph")?)?;
    let (records, bindings) = backend.capture_project_bundle(&graph, &stop)?.into_parts();
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("selected query replay record")?;
    assert_eq!(record.version(), before.version());
    assert_eq!(record.decode::<Value>()?, before.decode::<Value>()?);
    let replay: ProjectReplay = record.decode()?;
    let replay_wire = serde_json::to_value(&replay)?;
    assert_eq!(
        replay_wire["schema"],
        "wow-project/native-project-replay/10"
    );
    assert!(replay_wire.get("function_fingerprints").is_none());
    assert!(
        replay_wire["configuration"]
            .get("function_fingerprints")
            .is_none()
    );

    // Archive decoding rebuilds genuine source/project owners, never fingerprint authority.
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_dir_all(root.0.join("Blizzard_APIDocumentationGenerated"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    let archive = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    let reopened = archive.hydrate(&stop)?;
    assert_eq!(reopened.snapshot_id(), project.snapshot_id());
    assert_eq!(
        serde_json::to_vec(reopened.configuration())?,
        configuration_bytes
    );
    assert_eq!(
        observe(&reopened, &borrowed_ids, source_bytes, &stop)?,
        wire
    );
    let (rebuilt_records, rebuilt_bindings) =
        ProjectPublicationBundle::from_replay(&archive, &graph, &stop)?.into_parts();
    assert_eq!(rebuilt_bindings, bindings);
    let rebuilt = rebuilt_records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("recomputed replay record")?;
    assert_eq!(rebuilt.version(), record.version());
    assert_eq!(rebuilt.decode::<Value>()?, record.decode::<Value>()?);

    let cancelled = result.into_cancelled()?;
    assert_eq!(cancelled.status(), GraphReadStatus::Cancelled);
    cleared(&cancelled)?;
    Ok(())
}

const DECLARATION_LUA: &str = "HeaderTable = {}\n\
local function HeaderLocal(first, second, ...) return first end\n\
function HeaderTable:HeaderMethod(first, second, ...) return first end\n\
local HeaderClosure = function(first, second, ...) return first end\n";

fn rewrite_declaration_lua(root: &FixtureRoot, config: &mut Value, lua: &str) -> TestResult {
    // This is the existing synthetic fixture, not evidence of Git membership.
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    let member = files
        .iter_mut()
        .find(|file| file["path"] == "UI/main.lua")
        .ok_or("fixture Lua member")?;
    member["bytes"] = json!(lua.len());
    member["content_sha256"] = json!(format!("{:x}", Sha256::digest(lua.as_bytes())));
    let total = files
        .iter()
        .try_fold(0_u64, |sum, file| -> TestResult<u64> {
            Ok(sum
                .checked_add(file["bytes"].as_u64().ok_or("manifest member bytes")?)
                .ok_or("manifest byte sum")?)
        })?;
    manifest["coverage"]["included_bytes"] = json!(total);
    manifest
        .as_object_mut()
        .ok_or("manifest object")?
        .remove("manifest_sha256");
    manifest["manifest_sha256"] = json!(format!(
        "{:x}",
        Sha256::digest(wow_core::canonical_json_bytes(&manifest)?)
    ));
    std::fs::write(root.0.join("UI/main.lua"), lua)?;
    config["source"]["manifest"] =
        serde_json::to_value(write_json(root, "manifest.json", &manifest)?)?;
    config["source"]["origin"]["revision"] =
        serde_json::to_value(PlatformSourceRevision::Fixture {
            digest: digest(&serde_json::to_vec(&manifest)?),
        })?;
    write_json(root, "project.json", config)?;
    Ok(())
}

pub(super) fn declaration_query(
    path: &std::path::Path,
    stop: &AtomicBool,
) -> TestResult<(ProjectView, GraphBuildRequest, Vec<Box<str>>, usize)> {
    let input = LocalProjectInput::from_config_path(path, stop)?;
    let request = GraphBuildRequest::new("fixture-platform-transport".into(), "current".into())?
        .with_input_profile(&input);
    let backend = LocalProjectBackend::for_graph(input)?;
    let project = request.acquire_project(&backend, stop)?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native physical function report")?;
    let ids = report
        .functions()
        .iter()
        .map(|fact| Box::<str>::from(fact.fact_id()))
        .collect::<Vec<_>>();
    assert!(!ids.is_empty());
    assert!(ids.windows(2).all(|pair| pair[0] < pair[1]));
    let source_bytes =
        report
            .functions()
            .iter()
            .try_fold(0_usize, |sum, fact| -> TestResult<usize> {
                Ok(sum
                    .checked_add(span_bytes(&project, fact)?.len())
                    .ok_or("selected source byte sum")?)
            })?;
    let request = request.with_function_fingerprints(FunctionFingerprintSelection::new(
        ids.clone(),
        ids.len(),
        source_bytes,
        METADATA_BYTES,
    )?)?;
    Ok((project, request, ids, source_bytes))
}

type DeclarationHeaderDigest =
    ContentDigest<wow_project::graph::fingerprints::FunctionDeclaredHeaderFingerprint>;

fn verify_declared_headers(
    project: &ProjectView,
    manifest: &wow_project::graph::fingerprints::FunctionFingerprintManifest,
    method_order: &[&str],
) -> TestResult<[DeclarationHeaderDigest; 3]> {
    use wow_emmy::function_calls::{
        PhysicalFunctionDeclarationRecipe, SourceDeclaredParameter, SourceFunctionDeclaration,
        SourceFunctionDeclarationForm, SourceFunctionDeclarationUnavailable,
    };
    use wow_project::graph::fingerprints::FunctionDeclaredHeaderComponent;

    let recipe = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1;
    assert_eq!(
        project.configuration().physical_function_declarations(),
        Some(recipe)
    );
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("selected native function report")?;
    assert_eq!(report.declaration_recipe(), Some(recipe));
    assert_eq!(
        manifest.schema(),
        "wow-project/function-fingerprint-records/declared-header/1"
    );
    assert_eq!(
        manifest.profile(),
        "wow-project/physical-function-components/declared-header/1"
    );
    assert!(manifest.is_partial());
    assert_eq!(manifest.records().len(), report.functions().len());
    let mut headers = [None; 3];
    let mut chunks = 0;
    for record in manifest.records() {
        let fact = report
            .functions()
            .iter()
            .find(|fact| fact.fact_id() == record.function_id())
            .ok_or("selected native fact")?;
        let observation = fact.declaration().ok_or("native declaration observation")?;
        let header = record.declared_header().ok_or("selected header receipt")?;
        assert_eq!(header.native_recipe(), Some(recipe));
        assert_eq!(header.native_observation(), Some(observation));
        assert_eq!(
            serde_json::to_value(header.profile())?,
            "wow-project/physical-function-declared-header/1"
        );
        let wire = serde_json::to_value(record)?;
        assert_eq!(wire["signature"], "missing_signature");
        assert_eq!(wire["declaration_shape"], "missing_shape");
        assert_eq!(wire["direct_neighborhood"], "not_selected");
        assert_eq!(wire["package_load_role"], "not_selected");
        match observation {
            SourceFunctionDeclaration::Unavailable {
                reason: SourceFunctionDeclarationUnavailable::ChunkHasNoDeclaration,
            } => {
                assert_eq!(fact.kind(), SourceFunctionKind::Chunk);
                assert!(matches!(
                    header.component(),
                    FunctionDeclaredHeaderComponent::Unavailable {
                        reason: SourceFunctionDeclarationUnavailable::ChunkHasNoDeclaration
                    }
                ));
                chunks += 1;
            }
            SourceFunctionDeclaration::Observed {
                form,
                colon_definition,
                declaration_span,
                parameters,
            } => {
                assert_eq!(fact.kind(), SourceFunctionKind::Closure);
                let index = match form {
                    SourceFunctionDeclarationForm::LocalFunction => 0,
                    SourceFunctionDeclarationForm::NamedFunction => 1,
                    SourceFunctionDeclarationForm::ClosureExpression => 2,
                };
                assert_eq!(*colon_definition, index == 1);
                let order = if index == 1 {
                    method_order
                } else {
                    &["first", "second"]
                };
                assert_eq!(parameters.len(), 3, "implicit self must not be inserted");
                let file = project
                    .snapshot()
                    .analyzer_binding()
                    .main_workspace()
                    .file(fact.path())
                    .ok_or("native declaration file")?;
                let start =
                    usize::try_from(declaration_span.byte_start().ok_or("declaration start")?)?;
                let end = usize::try_from(declaration_span.byte_end().ok_or("declaration end")?)?;
                assert!(
                    !file
                        .text()
                        .get(start..end)
                        .ok_or("declaration span")?
                        .is_empty()
                );
                for (position, parameter) in parameters.iter().enumerate() {
                    let expected = match parameter {
                        SourceDeclaredParameter::Name { name, .. } => {
                            assert!(position < 2);
                            assert_eq!(name, order[position]);
                            name.as_str()
                        }
                        SourceDeclaredParameter::Vararg { .. } => {
                            assert_eq!(position, 2);
                            "..."
                        }
                    };
                    let span = parameter.span();
                    let start = usize::try_from(span.byte_start().ok_or("parameter start")?)?;
                    let end = usize::try_from(span.byte_end().ok_or("parameter end")?)?;
                    assert_eq!(
                        file.text().get(start..end).ok_or("parameter token")?,
                        expected
                    );
                }
                let FunctionDeclaredHeaderComponent::Available { digest } = header.component()
                else {
                    return Err("observed fixture header was not available".into());
                };
                assert!(
                    headers[index].replace(*digest).is_none(),
                    "one closure per form"
                );
            }
            _ => return Err("unexpected fixture declaration state".into()),
        }
    }
    assert!(chunks > 0);
    Ok([
        headers[0].ok_or("local-function header")?,
        headers[1].ok_or("named-colon header")?,
        headers[2].ok_or("closure-expression header")?,
    ])
}

#[test]
fn selected_declared_headers_survive_native_replay_and_selected_durable_reopen() -> TestResult {
    use crate::live_project::{
        LiveProjectPublishRequest, LiveProjectStore, PlatformStoreSelection,
        publish_input_in_namespace,
    };
    use wow_emmy::function_calls::PhysicalFunctionDeclarationRecipe;
    use wow_project::ProjectErrorCode;
    use wow_store::project::ReadSelector;

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
        ])?,
    ))?;
    rewrite_declaration_lua(&root, &mut config, DECLARATION_LUA)?;
    let path = root.0.join("project.json");

    // Absence remains the source-span recipe; this is not a predecessor byte freeze.
    let (plain_project, plain_request, _, _) = declaration_query(&path, &stop)?;
    assert_eq!(
        plain_project
            .configuration()
            .physical_function_declarations(),
        None
    );
    let plain_report = plain_project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("source-only native report")?;
    assert_eq!(plain_report.declaration_recipe(), None);
    assert!(
        plain_report
            .functions()
            .iter()
            .all(|fact| fact.declaration().is_none())
    );
    let plain = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &plain_request,
        &stop,
    )?;
    assert_eq!(
        plain.status(),
        GraphReadStatus::Partial,
        "{:?}",
        plain.failure_message()
    );
    let plain_manifest = plain
        .function_fingerprints()
        .ok_or("source-only manifest")?;
    assert_eq!(
        plain_manifest.schema(),
        "wow-project/function-fingerprint-records/1"
    );
    assert_eq!(
        plain_manifest.profile(),
        "wow-project/physical-function-source-span/1"
    );
    assert!(
        plain_manifest
            .records()
            .iter()
            .all(|record| record.declared_header().is_none())
    );
    let plain_wire = serde_json::to_value(&plain)?;
    for record in plain_wire["function_fingerprints"]["records"]
        .as_array()
        .ok_or("source-only records")?
    {
        assert!(record.get("declared_header").is_none());
    }
    assert!(
        plain_wire["request"]
            .get("physical_function_declarations")
            .is_none()
    );

    let recipe = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxV1;
    config["physical_function_declarations"] = serde_json::to_value(recipe)?;
    write_json(&root, "project.json", &config)?;
    let (project, request, ids, source_bytes) = declaration_query(&path, &stop)?;
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &request,
        &stop,
    )?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{:?}",
        result.failure_message()
    );
    let wire = serde_json::to_value(&result)?;
    assert_eq!(
        wire["schema"],
        "wow-service/graph-build-result/physical-function-declarations-and-fingerprints/1"
    );
    assert_eq!(
        wire["request"]["schema"],
        "wow-service/graph-build-request/physical-function-declarations-and-fingerprints/1"
    );
    assert_eq!(
        wire["request"]["physical_function_declarations"],
        serde_json::to_value(recipe)?
    );
    let manifest = result
        .function_fingerprints()
        .ok_or("selected typed manifest")?;
    let headers = verify_declared_headers(&project, manifest, &["first", "second"])?;
    let borrowed_ids = ids.iter().map(AsRef::as_ref).collect::<Vec<&str>>();
    let expected = serde_json::to_value(manifest)?;
    // The existing helper also resolves real accepted ownership/support assertions.
    assert_eq!(
        observe(&project, &borrowed_ids, source_bytes, &stop)?,
        expected
    );

    // Selection equality is checked before the query can claim a different native recipe.
    let mut absent = config.clone();
    absent
        .as_object_mut()
        .ok_or("config object")?
        .remove("physical_function_declarations");
    write_json(&root, "project.json", &absent)?;
    let mismatch = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &request,
        &stop,
    )?;
    assert_eq!(mismatch.status(), GraphReadStatus::Failed);
    assert_eq!(
        serde_json::to_value(&mismatch)?["failure"],
        serde_json::to_value(ServiceErrorCode::IdentityMismatch)?
    );
    cleared(&mismatch)?;
    let mut null = config.clone();
    null["physical_function_declarations"] = Value::Null;
    write_json(&root, "project.json", &null)?;
    assert_eq!(
        LocalProjectInput::from_config_path(&path, &stop)
            .err()
            .ok_or("explicit null recipe admitted")?
            .code(),
        ServiceErrorCode::InvalidConfiguration
    );
    write_json(&root, "project.json", &config)?;

    // Change only the named method's parameter order; use new native IDs, not old ones.
    let selected_config = config.clone();
    assert_eq!(
        DECLARATION_LUA
            .matches("HeaderMethod(first, second, ...)")
            .count(),
        1
    );
    let reordered = DECLARATION_LUA.replace(
        "HeaderMethod(first, second, ...)",
        "HeaderMethod(second, first, ...)",
    );
    rewrite_declaration_lua(&root, &mut config, &reordered)?;
    let (changed_project, changed_request, _, _) = declaration_query(&path, &stop)?;
    assert_ne!(changed_project.snapshot_id(), project.snapshot_id());
    let changed = execute_graph_build(
        LocalProjectInput::from_config_path(&path, &stop)?,
        &changed_request,
        &stop,
    )?;
    assert_eq!(
        changed.status(),
        GraphReadStatus::Partial,
        "{:?}",
        changed.failure_message()
    );
    let changed_headers = verify_declared_headers(
        &changed_project,
        changed.function_fingerprints().ok_or("reordered headers")?,
        &["second", "first"],
    )?;
    assert_eq!(headers[0], changed_headers[0]);
    assert_ne!(headers[1], changed_headers[1]);
    assert_eq!(headers[2], changed_headers[2]);
    rewrite_declaration_lua(&root, &mut config, DECLARATION_LUA)?;
    assert_eq!(config, selected_config);

    let source_profile_id = project
        .configuration()
        .platform_packages()
        .ok_or("native platform packages")?
        .source()
        .profile()
        .profile_id()
        .clone();
    let default_selection = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        source_profile_id,
    )?;
    let selection = default_selection
        .clone()
        .with_physical_function_declarations_catalog();
    let publication_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let store_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let default_store_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let graph: GraphPartitionSnapshot =
        serde_json::from_slice(&result.snapshot_bytes()?.ok_or("selected graph")?)?;

    // Hold admitted inputs, then remove the original Lua/TOC/XML/version bytes before capture.
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    assert!(!root.0.join("UI").exists());
    assert!(!root.0.join("version.txt").exists());
    let (bundle, _) =
        request.live_publication_in_namespace(publication_input, selection.namespace(), &stop)?;
    let (records, bindings) = bundle.into_parts();
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("selected replay record")?;
    let replay: ProjectReplay = record.decode()?;
    let expected_record = wow_store::project::PartitionRecord::new(
        record.key(),
        "wow-project.live-replay.physical-function-declarations.v1",
        &replay,
    )?;
    assert_eq!(record.version(), expected_record.version());
    let replay_wire = serde_json::to_value(&replay)?;
    assert_eq!(
        replay_wire["schema"],
        "wow-project/native-project-replay/physical-function-declarations/1"
    );
    assert_eq!(
        replay_wire["configuration"]["physical_function_declarations"],
        serde_json::to_value(recipe)?
    );
    assert!(replay_wire.get("function_fingerprints").is_none());
    let mut null_replay = replay_wire;
    null_replay["configuration"]["physical_function_declarations"] = Value::Null;
    assert_eq!(
        ProjectReplay::from_json(&serde_json::to_vec(&null_replay)?, &stop)
            .err()
            .ok_or("null replay recipe admitted")?
            .code(),
        ProjectErrorCode::SnapshotInvalid
    );
    let decoded = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    let hydrated = decoded.hydrate(&stop)?;
    assert_eq!(hydrated.snapshot_id(), project.snapshot_id());
    assert_eq!(
        Some(hydrated.snapshot_id()),
        bindings.get("project_snapshot_id").map(String::as_str)
    );
    assert_eq!(
        serde_json::to_value(
            hydrated
                .snapshot()
                .analyzer_binding()
                .function_call_report()
                .ok_or("hydrated report")?
        )?,
        serde_json::to_value(
            project
                .snapshot()
                .analyzer_binding()
                .function_call_report()
                .ok_or("original report")?
        )?
    );
    assert_eq!(
        observe(&hydrated, &borrowed_ids, source_bytes, &stop)?,
        expected
    );
    let (rebuilt, rebuilt_bindings) = ProjectPublicationBundle::from_replay_in_namespace(
        &decoded,
        &graph,
        selection.namespace(),
        &stop,
    )?
    .into_parts();
    assert_eq!(rebuilt_bindings, bindings);
    assert_eq!(rebuilt.len(), records.len());
    for (rebuilt, original) in rebuilt.iter().zip(&records) {
        assert_eq!(rebuilt.key(), original.key());
        assert_eq!(rebuilt.version(), original.version());
        // Semantic record equality; this does not freeze serialization ordering.
        assert_eq!(rebuilt.decode::<Value>()?, original.decode::<Value>()?);
    }

    let store_root = root.0.join("declaration-store");
    let publish = LiveProjectPublishRequest::new("fixture:declared-headers", "absent", true, true)?;
    assert_eq!(
        publish_input_in_namespace(
            default_store_input,
            &request,
            &store_root,
            &default_selection,
            &publish,
            &stop,
        )
        .err()
        .ok_or("default catalog admitted selected declarations")?
        .code(),
        ServiceErrorCode::IdentityMismatch
    );
    assert!(
        !store_root.exists(),
        "catalog/config refusal precedes store creation"
    );
    let published = publish_input_in_namespace(
        store_input,
        &request,
        &store_root,
        &selection,
        &publish,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(&published)?["status"], "activated");
    assert_eq!(
        LiveProjectStore::open_in_namespace(&store_root, &default_selection)
            .err()
            .ok_or("ordinary catalog reopened declaration epoch")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    let current = store.current()?.ok_or("selected Current")?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), project.snapshot_id());
    assert_eq!(
        read.graph().snapshot().snapshot_id(),
        graph.snapshot().snapshot_id()
    );
    assert_eq!(
        read.project()
            .configuration()
            .physical_function_declarations(),
        Some(recipe)
    );
    assert_eq!(
        observe(read.project(), &borrowed_ids, source_bytes, &stop)?,
        expected
    );
    drop(read);
    drop(store);
    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    assert_eq!(store.current()?, Some(current));
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    assert_eq!(read.project().snapshot_id(), hydrated.snapshot_id());
    assert_eq!(
        read.graph().snapshot().snapshot_id(),
        graph.snapshot().snapshot_id()
    );
    assert_eq!(
        read.project()
            .configuration()
            .physical_function_declarations(),
        Some(recipe)
    );
    assert_eq!(
        observe(read.project(), &borrowed_ids, source_bytes, &stop)?,
        expected
    );

    let cancelled = result.into_cancelled()?;
    assert_eq!(cancelled.status(), GraphReadStatus::Cancelled);
    cleared(&cancelled)?;
    Ok(())
}
