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

struct BodyShapeObservation {
    events: Vec<wow_emmy::function_calls::FunctionShapeEvent>,
    shape_digest:
        ContentDigest<wow_project::graph::fingerprints::FunctionDeclarationShapeFingerprint>,
    source_digest: ContentDigest<wow_project::graph::fingerprints::FunctionSourceSpanFingerprint>,
    source_bytes: Vec<u8>,
    function_id: Box<str>,
    project_snapshot_id: Box<str>,
}

fn selected_fixture_body_shape(
    root: &FixtureRoot,
    config: &mut Value,
    lua: &str,
    stop: &AtomicBool,
) -> TestResult<BodyShapeObservation> {
    use wow_emmy::function_calls::{
        PhysicalFunctionDeclarationRecipe, SourceFunctionDeclaration,
        SourceFunctionDeclarationForm, SourceFunctionShape,
    };
    use wow_project::graph::fingerprints::{
        FunctionDeclarationShapeComponent, FunctionSourceSpanComponent,
    };

    rewrite_declaration_lua(root, config, lua)?;
    let path = root.0.join("project.json");
    let (project, request, _, _) = declaration_query(&path, stop)?;
    let recipe = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1;
    assert_eq!(
        project.configuration().physical_function_declarations(),
        Some(recipe)
    );
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("selected native body-shape report")?;
    assert_eq!(report.declaration_recipe(), Some(recipe));
    let mut closures = report.functions().iter().filter(|fact| {
        matches!(
            fact.declaration(),
            Some(SourceFunctionDeclaration::Observed {
                form: SourceFunctionDeclarationForm::LocalFunction,
                ..
            })
        )
    });
    let fact = closures
        .next()
        .ok_or("fixture local-function declaration")?;
    assert!(
        closures.next().is_none(),
        "one genuine local-function target"
    );
    assert_eq!(fact.kind(), SourceFunctionKind::Closure);
    let SourceFunctionShape::Observed { events } =
        fact.shape().ok_or("selected native body shape")?
    else {
        return Err("fixture native body shape was not observed".into());
    };
    let source_bytes = span_bytes(&project, fact)?.to_vec();
    let function_id = Box::<str>::from(fact.fact_id());
    let request = request.with_function_fingerprints(FunctionFingerprintSelection::new(
        vec![function_id.clone()],
        1,
        source_bytes.len(),
        METADATA_BYTES,
    )?)?;
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, stop)?,
        &request,
        stop,
    )?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{:?}",
        result.failure_message()
    );
    let manifest = result
        .function_fingerprints()
        .ok_or("selected service body-shape manifest")?;
    assert_eq!(
        manifest.profile(),
        "wow-project/function-fingerprint/declaration-body-shape/1"
    );
    assert!(manifest.is_partial());
    assert_eq!(manifest.records().len(), 1);
    assert_eq!(
        manifest.binding().project_snapshot_id(),
        project.snapshot_id()
    );
    assert_eq!(
        manifest.binding().function_analysis_id(),
        report.analysis_id()
    );
    let record = manifest.records().first().ok_or("selected shape record")?;
    assert_eq!(record.function_id(), function_id.as_ref());
    let header = record.declared_header().ok_or("retained declared header")?;
    assert_eq!(header.native_recipe(), Some(recipe));
    assert_eq!(header.native_observation(), fact.declaration());
    let FunctionDeclarationShapeComponent::Available {
        digest: shape_digest,
    } = record.declaration_shape()
    else {
        return Err("fixture service body-shape component was not available".into());
    };
    let FunctionSourceSpanComponent::Available {
        digest: source_digest,
        byte_length,
        ..
    } = record.source_span()
    else {
        return Err("fixture source-span component was not available".into());
    };
    assert_eq!(usize::try_from(*byte_length)?, source_bytes.len());
    let wire = serde_json::to_value(record)?;
    assert_eq!(wire["declaration_shape"]["state"], "available");
    assert_eq!(wire["signature"], "missing_signature");

    Ok(BodyShapeObservation {
        events: events.clone(),
        shape_digest: *shape_digest,
        source_digest: *source_digest,
        source_bytes,
        function_id,
        project_snapshot_id: project.snapshot_id().into(),
    })
}

#[test]
fn selected_body_shape_ignores_trivia_and_payloads_but_retains_operator() -> TestResult {
    use wow_emmy::function_calls::{
        FunctionShapeEvent, FunctionShapeNode, FunctionShapeToken,
        PhysicalFunctionDeclarationRecipe,
    };
    use wow_project::graph::fingerprints::FunctionDeclarationShapeComponent;

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
        ])?,
    ))?;
    let recipe = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1;
    assert_eq!(
        serde_json::to_value(recipe)?,
        "wow-emmy/physical-function-declaration-syntax-and-body-shape/1"
    );
    config["physical_function_declarations"] = serde_json::to_value(recipe)?;
    // This pins the legacy field encoding only, not predecessor manifest/result bytes.
    assert_eq!(
        serde_json::to_vec(&FunctionDeclarationShapeComponent::MissingShape)?,
        br#""missing_shape""#
    );

    let base = selected_fixture_body_shape(
        &root,
        &mut config,
        "local function ShapeFixture(value)\n    local adjusted = value + 1\n    return adjusted\nend\n",
        &stop,
    )?;
    assert_eq!(
        base.events.first(),
        Some(&FunctionShapeEvent::Enter(FunctionShapeNode::Block))
    );
    assert_eq!(
        base.events.last(),
        Some(&FunctionShapeEvent::Leave(FunctionShapeNode::Block))
    );
    assert!(base.events.contains(&FunctionShapeEvent::Enter(
        FunctionShapeNode::LocalStatement
    )));
    assert!(base.events.contains(&FunctionShapeEvent::Enter(
        FunctionShapeNode::BinaryExpression
    )));
    assert!(base.events.contains(&FunctionShapeEvent::Enter(
        FunctionShapeNode::ReturnStatement
    )));
    assert!(
        base.events
            .contains(&FunctionShapeEvent::Token(FunctionShapeToken::Name))
    );
    assert!(
        base.events
            .contains(&FunctionShapeEvent::Token(FunctionShapeToken::Number))
    );
    assert!(
        base.events
            .contains(&FunctionShapeEvent::Token(FunctionShapeToken::Plus))
    );

    // One neutral mutation changes trivia, local spelling, and a number's spelling/value.
    let neutral = selected_fixture_body_shape(
        &root,
        &mut config,
        "-- fixture trivia\nlocal function ShapeFixture(value)\n    -- body trivia\n    local renamed = value    +    2.0\n    return renamed\nend\n",
        &stop,
    )?;
    assert_eq!(neutral.events, base.events);
    assert_eq!(neutral.shape_digest, base.shape_digest);
    assert_ne!(neutral.source_bytes, base.source_bytes);
    assert_ne!(neutral.source_digest, base.source_digest);
    assert_ne!(neutral.function_id, base.function_id);
    assert_ne!(neutral.project_snapshot_id, base.project_snapshot_id);

    // The only change from the baseline is the retained binary operator.
    let operator = selected_fixture_body_shape(
        &root,
        &mut config,
        "local function ShapeFixture(value)\n    local adjusted = value - 1\n    return adjusted\nend\n",
        &stop,
    )?;
    assert!(
        operator
            .events
            .contains(&FunctionShapeEvent::Token(FunctionShapeToken::Minus))
    );
    assert!(
        !operator
            .events
            .contains(&FunctionShapeEvent::Token(FunctionShapeToken::Plus))
    );
    assert_ne!(operator.events, base.events);
    assert_ne!(operator.shape_digest, base.shape_digest);
    assert_ne!(operator.source_bytes, base.source_bytes);
    assert_ne!(operator.source_digest, base.source_digest);
    assert_ne!(operator.function_id, base.function_id);
    assert_ne!(operator.project_snapshot_id, base.project_snapshot_id);
    Ok(())
}

fn rewrite_load_role_toc(root: &FixtureRoot, config: &mut Value, toc: &str) -> TestResult {
    // Refresh the existing synthetic fixture; this does not attest Git membership.
    let mut manifest: Value =
        serde_json::from_slice(&std::fs::read(root.0.join("manifest.json"))?)?;
    let files = manifest["files"].as_array_mut().ok_or("manifest files")?;
    let member = files
        .iter_mut()
        .find(|file| file["path"] == "UI/Fixture.toc")
        .ok_or("fixture TOC member")?;
    member["bytes"] = json!(toc.len());
    member["content_sha256"] = json!(format!("{:x}", Sha256::digest(toc.as_bytes())));
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
    std::fs::write(root.0.join("UI/Fixture.toc"), toc)?;
    config["source"]["manifest"] =
        serde_json::to_value(write_json(root, "manifest.json", &manifest)?)?;
    config["source"]["origin"]["revision"] =
        serde_json::to_value(PlatformSourceRevision::Fixture {
            digest: digest(&serde_json::to_vec(&manifest)?),
        })?;
    write_json(root, "project.json", config)?;
    Ok(())
}

struct PackageLoadRoleObservation {
    role_digest:
        ContentDigest<wow_project::graph::fingerprints::FunctionPackageLoadRoleFingerprint>,
    source_digest: ContentDigest<wow_project::graph::fingerprints::FunctionSourceSpanFingerprint>,
    source_bytes: Vec<u8>,
    load_on_demand: wow_project::load::TocLoadOnDemandState,
    phase: wow_project::load::ProjectPackageLoadPhase,
    load_plan_digest: ContentDigest<CanonicalResult>,
    main_plan_digest: ContentDigest<CanonicalResult>,
    selected_toc_digest: ContentDigest<SourceContent>,
    project_snapshot_id: Box<str>,
}

fn observe_fixture_package_load_role(
    root: &FixtureRoot,
    config: &mut Value,
    toc: &str,
    stop: &AtomicBool,
) -> TestResult<PackageLoadRoleObservation> {
    use crate::graph::FunctionPackageLoadRoleRecipe;
    use wow_project::{
        graph::fingerprints::{
            FunctionPackageLoadRoleComponent, FunctionPackageLoadRoleCoverage,
            FunctionPackageLoadRoleSelection, FunctionSourceSpanComponent,
        },
        load::{LoadRecordKind, ProjectPackageReachability},
    };

    rewrite_load_role_toc(root, config, toc)?;
    let path = root.0.join("project.json");
    let (project, request, _, _) = declaration_query(&path, stop)?;
    assert_eq!(
        project.configuration().physical_function_declarations(),
        None
    );
    let packages = project
        .configuration()
        .platform_packages()
        .ok_or("native package owners")?;
    let load = packages.load_plan();
    let main = packages.main_plan();
    assert!(load.coverage().complete(), "{:?}", load.issues());
    assert_eq!(load.packages().len(), 1);
    assert_eq!(main.files().len(), 1);
    let node = load.packages().first().ok_or("native package node")?;
    let file = main.files().first().ok_or("native physical Main file")?;
    assert!(node.selected_root);
    assert_eq!(node.reachability, ProjectPackageReachability::Reachable);
    assert_eq!(file.package, node.package);
    assert_eq!(file.source_path, "main.lua");
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("native physical function report")?;
    assert_eq!(report.declaration_recipe(), None);
    let mut closures = report.functions().iter().filter(|fact| {
        fact.kind() == SourceFunctionKind::Closure && fact.path() == file.project_path
    });
    let fact = closures.next().ok_or("physical fixture closure")?;
    assert!(closures.next().is_none(), "one physical fixture closure");
    let source_bytes = span_bytes(&project, fact)?.to_vec();
    let recipe = FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1;
    let selection = FunctionFingerprintSelection::new(
        vec![fact.fact_id().into()],
        1,
        source_bytes.len(),
        METADATA_BYTES,
    )?
    .with_package_load_roles(recipe)?;
    assert_eq!(selection.package_load_roles(), Some(recipe));
    let request = request.with_function_fingerprints(selection)?;
    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, stop)?,
        &request,
        stop,
    )?;
    assert_eq!(
        result.status(),
        GraphReadStatus::Partial,
        "{:?}",
        result.failure_message()
    );
    let result_wire = serde_json::to_value(&result)?;
    assert_eq!(
        result_wire["schema"],
        "wow-service/graph-build-result/selected-function-components/1"
    );
    assert_eq!(
        result_wire["request"]["schema"],
        "wow-service/graph-build-request/selected-function-components/1"
    );
    let manifest = result
        .function_fingerprints()
        .ok_or("selected service load-role manifest")?;
    assert_eq!(
        manifest.schema(),
        "wow-project/function-fingerprint-records/selected-components/1"
    );
    assert!(manifest.is_partial());
    assert!(manifest.scope_conflicts().is_empty());
    assert_eq!(manifest.records().len(), 1);
    assert_eq!(
        manifest.binding().project_snapshot_id(),
        project.snapshot_id()
    );
    assert_eq!(
        manifest.binding().function_analysis_id(),
        report.analysis_id()
    );
    let selected = manifest
        .selected_components()
        .ok_or("component selection receipt")?;
    assert_eq!(selected.physical_declarations(), None);
    assert_eq!(selected.package_load_role(), Some(recipe));
    let selected_wire = serde_json::to_value(selected)?;
    assert!(selected_wire.get("physical_declarations").is_none());
    assert_eq!(
        selected_wire["package_load_role"],
        serde_json::to_value(recipe)?
    );
    let record = manifest
        .records()
        .first()
        .ok_or("selected physical record")?;
    assert_eq!(record.function_id(), fact.fact_id());
    assert!(record.declared_header().is_none());
    assert_eq!(
        serde_json::to_value(record)?["declaration_shape"],
        "missing_shape"
    );
    let FunctionPackageLoadRoleSelection::Selected(role) = record.package_load_role() else {
        return Err("fixture package/load role was not selected".into());
    };
    assert_eq!(role.coverage(), FunctionPackageLoadRoleCoverage::Complete);
    assert_eq!(role.native_load_coverage(), load.coverage());
    assert_eq!(serde_json::to_value(recipe)?, role.recipe());
    let FunctionPackageLoadRoleComponent::Comparable {
        digest: role_digest,
        value,
    } = role.component()
    else {
        return Err("complete fixture package/load role was not comparable".into());
    };
    assert_eq!(value.selected_root(), node.selected_root);
    assert_eq!(value.load_on_demand(), node.load_on_demand);
    assert_eq!(value.package_phase(), node.phase);
    assert_eq!(value.package_reachability(), node.reachability);
    let witness = role.witness().ok_or("native package/load witness")?;
    assert_eq!(witness.package_load_plan_digest(), load.digest());
    assert_eq!(witness.package_main_plan_digest(), main.digest());
    assert_eq!(witness.selected_plan_digest(), node.selected_plan_digest);
    assert_eq!(witness.main_file(), file);
    let selected_toc = node
        .variants
        .iter()
        .find(|variant| variant.selected)
        .ok_or("native selected TOC")?;
    assert_eq!(witness.selected_toc(), selected_toc);
    let mut units = load.units().iter().filter(|unit| {
        unit.kind == LoadRecordKind::LuaFile
            && unit.package == file.package
            && unit.target == file.source_path
    });
    let unit = units.next().ok_or("native physical Lua load unit")?;
    assert!(units.next().is_none(), "one physical Lua load occurrence");
    assert_eq!(witness.load_units().len(), 1);
    assert_eq!(witness.load_units().first(), Some(unit));
    assert_eq!(value.lua_load_roles().len(), 1);
    let group = value
        .lua_load_roles()
        .first()
        .ok_or("typed Lua role group")?;
    assert_eq!(group.phase(), unit.phase);
    assert_eq!(group.reachability(), unit.reachability);
    assert_eq!(group.bootstrap(), unit.bootstrap);
    assert!(!group.bootstrap());
    assert_eq!(group.occurrences(), 1);
    let FunctionSourceSpanComponent::Available {
        digest: source_digest,
        byte_length,
        ..
    } = record.source_span()
    else {
        return Err("fixture source span was not available".into());
    };
    assert_eq!(usize::try_from(*byte_length)?, source_bytes.len());

    Ok(PackageLoadRoleObservation {
        role_digest: *role_digest,
        source_digest: *source_digest,
        source_bytes,
        load_on_demand: value.load_on_demand(),
        phase: value.package_phase(),
        load_plan_digest: witness.package_load_plan_digest(),
        main_plan_digest: witness.package_main_plan_digest(),
        selected_toc_digest: witness.selected_toc().content_digest,
        project_snapshot_id: project.snapshot_id().into(),
    })
}

#[test]
fn selected_package_load_role_tracks_lod_with_identical_physical_source() -> TestResult {
    use wow_project::{
        graph::fingerprints::FunctionPackageLoadRoleSelection,
        load::{ProjectPackageLoadPhase, TocLoadOnDemandState},
    };

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    config["graph_profile"] = serde_json::to_value(PlatformGraphProfile::DirectCapabilitiesV1(
        PlatformGraphCapabilities::new(&[
            PlatformGraphCapability::InventorySpansV1,
            PlatformGraphCapability::StructuralRolesV1,
        ])?,
    ))?;
    // Field-byte compatibility only; this is not a predecessor manifest/result freeze.
    assert_eq!(
        serde_json::to_vec(&FunctionPackageLoadRoleSelection::NotSelected)?.as_slice(),
        br#""not_selected""#
    );
    // Interface is the explicit synthetic fixture target; the loaded XML stays out of this case.
    let base = observe_fixture_package_load_role(
        &root,
        &mut config,
        "## Interface: 120100\nmain.lua\n",
        &stop,
    )?;
    assert_eq!(base.load_on_demand, TocLoadOnDemandState::NotDeclared);
    assert_eq!(base.phase, ProjectPackageLoadPhase::Normal);
    let lod = observe_fixture_package_load_role(
        &root,
        &mut config,
        "## Interface: 120100\n## LoadOnDemand: 1\nmain.lua\n",
        &stop,
    )?;
    assert_eq!(lod.load_on_demand, TocLoadOnDemandState::True);
    assert_eq!(lod.phase, ProjectPackageLoadPhase::Deferred);
    assert_ne!(lod.role_digest, base.role_digest);
    assert_eq!(lod.source_bytes, base.source_bytes);
    assert_eq!(lod.source_digest, base.source_digest);
    assert_ne!(lod.load_plan_digest, base.load_plan_digest);
    assert_ne!(lod.main_plan_digest, base.main_plan_digest);
    assert_ne!(lod.selected_toc_digest, base.selected_toc_digest);
    assert_ne!(lod.project_snapshot_id, base.project_snapshot_id);

    Ok(())
}

struct IndexedSignatureObservation {
    signature_digest: ContentDigest<wow_project::graph::fingerprints::FunctionSignatureFingerprint>,
    source_digest: ContentDigest<wow_project::graph::fingerprints::FunctionSourceSpanFingerprint>,
    source_bytes: Vec<u8>,
    function_id: Box<str>,
    project_snapshot_id: Box<str>,
    function_analysis_id: Box<str>,
}

fn observe_fixture_indexed_signature(
    root: &FixtureRoot,
    config: &mut Value,
    lua: &str,
    expected_type: wow_emmy::function_calls::SourceSignatureType,
    stop: &AtomicBool,
) -> TestResult<IndexedSignatureObservation> {
    use wow_emmy::function_calls::{
        PhysicalFunctionSignatureRecipe, SourceSignatureReturnResolution,
        SourceSignatureUnavailable,
    };
    use wow_project::graph::fingerprints::{
        FunctionSignatureComponent, FunctionSourceSpanComponent,
    };

    rewrite_declaration_lua(root, config, lua)?;
    let path = root.0.join("project.json");
    let (project, request, ids, _) = declaration_query(&path, stop)?;
    let recipe = PhysicalFunctionSignatureRecipe::IndexedTypesV1;
    assert_eq!(
        project.configuration().physical_function_signatures(),
        Some(recipe)
    );
    assert_eq!(
        project.configuration().physical_function_declarations(),
        None
    );
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("selected native indexed-signature report")?;
    assert_eq!(report.signature_recipe(), Some(recipe));
    assert_eq!(report.declaration_recipe(), None);
    assert!(report.source_health_complete());
    let main = project
        .configuration()
        .platform_packages()
        .ok_or("native package owners")?
        .main_plan();
    assert_eq!(main.files().len(), 1);
    let file = main.files().first().ok_or("native physical Main file")?;
    let mut closures = report.functions().iter().filter(|fact| {
        fact.kind() == SourceFunctionKind::Closure && fact.path() == file.project_path
    });
    let fact = closures.next().ok_or("native physical fixture closure")?;
    assert!(closures.next().is_none(), "one physical fixture closure");
    assert!(fact.declaration().is_none());
    assert!(fact.shape().is_none());
    let signature = fact.signature().ok_or("selected native signature")?;
    let comparable = signature
        .comparable()
        .ok_or("fixture indexed types were not comparable")?;
    assert_eq!(signature.observed_types(), Some(comparable));
    assert!(signature.unavailable_reasons().is_empty());
    assert!(!comparable.colon_definition());
    assert_eq!(comparable.parameters().len(), 1);
    let parameter = comparable
        .parameters()
        .first()
        .ok_or("ordered native parameter")?;
    assert_eq!(parameter.backend_nullable(), Some(false));
    assert_eq!(parameter.type_shape(), &expected_type);
    assert_eq!(comparable.returns().len(), 1);
    let returns = comparable
        .returns()
        .first()
        .ok_or("primary native return row")?;
    assert_eq!(returns.types(), std::slice::from_ref(&expected_type));
    assert_eq!(
        signature.return_resolution(),
        Some(SourceSignatureReturnResolution::DocumentResolved)
    );
    assert_eq!(signature.vararg(), Some(false));
    let chunk = report
        .functions()
        .iter()
        .find(|fact| fact.kind() == SourceFunctionKind::Chunk && fact.path() == file.project_path)
        .ok_or("native physical Chunk")?;
    let chunk_signature = chunk.signature().ok_or("selected Chunk observation")?;
    assert!(chunk_signature.comparable().is_none());
    assert_eq!(
        chunk_signature.unavailable_reasons(),
        &[SourceSignatureUnavailable::ChunkHasNoSignature]
    );
    let source_bytes = span_bytes(&project, fact)?.to_vec();

    let result = execute_graph_build(
        LocalProjectInput::from_config_path(&path, stop)?,
        &request,
        stop,
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
        "wow-service/graph-build-result/selected-function-components/1"
    );
    assert_eq!(
        wire["request"]["schema"],
        "wow-service/graph-build-request/selected-function-components/1"
    );
    assert_eq!(
        wire["request"]["physical_function_signatures"],
        serde_json::to_value(recipe)?
    );
    let manifest = result
        .function_fingerprints()
        .ok_or("selected service indexed-signature manifest")?;
    assert_eq!(
        manifest.schema(),
        "wow-project/function-fingerprint-records/selected-components/1"
    );
    assert_eq!(
        manifest.profile(),
        "wow-project/physical-function-components/selected/1"
    );
    assert!(manifest.is_partial());
    assert!(manifest.scope_conflicts().is_empty());
    assert_eq!(manifest.records().len(), ids.len());
    assert_eq!(
        manifest.binding().project_snapshot_id(),
        project.snapshot_id()
    );
    assert_eq!(
        manifest.binding().function_analysis_id(),
        report.analysis_id()
    );
    let selected = manifest
        .selected_components()
        .ok_or("native selection receipt")?;
    assert_eq!(selected.physical_signatures(), Some(recipe));
    assert_eq!(selected.physical_declarations(), None);
    assert_eq!(selected.package_load_role(), None);
    let selected_wire = serde_json::to_value(selected)?;
    assert_eq!(
        selected_wire["physical_signatures"],
        serde_json::to_value(recipe)?
    );
    assert!(selected_wire.get("physical_declarations").is_none());
    assert!(selected_wire.get("package_load_role").is_none());
    let record = manifest
        .records()
        .iter()
        .find(|record| record.function_id() == fact.fact_id())
        .ok_or("selected genuine physical closure record")?;
    let FunctionSignatureComponent::Available {
        digest: signature_digest,
    } = record.signature()
    else {
        return Err("fixture service indexed signature was not available".into());
    };
    let signature_wire = serde_json::to_value(record.signature())?;
    assert_eq!(signature_wire["state"], "available");
    assert_eq!(
        signature_wire["recipe"],
        "wow-project/physical-function-indexed-types/1"
    );
    assert!(record.declared_header().is_none());
    let record_wire = serde_json::to_value(record)?;
    assert_eq!(record_wire["declaration_shape"], "missing_shape");
    assert_eq!(record_wire["package_load_role"], "not_selected");
    let chunk_record = manifest
        .records()
        .iter()
        .find(|record| record.function_id() == chunk.fact_id())
        .ok_or("selected genuine Chunk record")?;
    let FunctionSignatureComponent::Unavailable { reasons } = chunk_record.signature() else {
        return Err("selected Chunk incorrectly admitted a signature digest".into());
    };
    assert_eq!(reasons.as_slice(), chunk_signature.unavailable_reasons());
    assert!(chunk_record.signature().digest().is_none());
    let FunctionSourceSpanComponent::Available {
        digest: source_digest,
        byte_length,
        ..
    } = record.source_span()
    else {
        return Err("fixture source-span component was not available".into());
    };
    assert_eq!(usize::try_from(*byte_length)?, source_bytes.len());

    Ok(IndexedSignatureObservation {
        signature_digest: *signature_digest,
        source_digest: *source_digest,
        source_bytes,
        function_id: fact.fact_id().into(),
        project_snapshot_id: project.snapshot_id().into(),
        function_analysis_id: report.analysis_id().into(),
    })
}

#[test]
fn selected_indexed_signature_tracks_annotation_types_with_identical_body() -> TestResult {
    use wow_emmy::function_calls::{PhysicalFunctionSignatureRecipe, SourceSignatureType};
    use wow_project::graph::fingerprints::FunctionSignatureComponent;

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    config["physical_function_signatures"] =
        serde_json::to_value(PhysicalFunctionSignatureRecipe::IndexedTypesV1)?;
    // Legacy field bytes only, not a predecessor result or replay freeze.
    assert_eq!(
        serde_json::to_vec(&FunctionSignatureComponent::MissingSignature)?.as_slice(),
        br#""missing_signature""#
    );
    let number = observe_fixture_indexed_signature(
        &root,
        &mut config,
        "---@param value number\n---@return number\nlocal function TypedSignatureFixture(value)\n    return value\nend\n",
        SourceSignatureType::Number,
        &stop,
    )?;
    let string = observe_fixture_indexed_signature(
        &root,
        &mut config,
        "---@param value string\n---@return string\nlocal function TypedSignatureFixture(value)\n    return value\nend\n",
        SourceSignatureType::String,
        &stop,
    )?;
    assert_ne!(string.signature_digest, number.signature_digest);
    assert_eq!(string.source_bytes, number.source_bytes);
    assert_eq!(string.source_digest, number.source_digest);
    assert_ne!(string.function_id, number.function_id);
    assert_ne!(string.project_snapshot_id, number.project_snapshot_id);
    assert_ne!(string.function_analysis_id, number.function_analysis_id);
    Ok(())
}

#[test]
fn selected_indexed_signatures_survive_selected_replay_and_catalog_reopen() -> TestResult {
    use crate::live_project::{
        LiveProjectPublishRequest, LiveProjectStore, PlatformStoreSelection,
        publish_input_in_namespace,
    };
    use wow_emmy::function_calls::{PhysicalFunctionSignatureRecipe, SourceFunctionSignature};
    use wow_store::project::{PartitionRecord, ReadSelector};

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    let recipe = PhysicalFunctionSignatureRecipe::IndexedTypesV1;
    config["physical_function_signatures"] = serde_json::to_value(recipe)?;
    rewrite_declaration_lua(
        &root,
        &mut config,
        "---@param value number\n---@return number\nlocal function TypedSignatureFixture(value)\n    return value\nend\n",
    )?;
    let path = root.0.join("project.json");
    let (project, request, _, _) = declaration_query(&path, &stop)?;
    let report = project
        .snapshot()
        .analyzer_binding()
        .function_call_report()
        .ok_or("original native signature report")?;
    assert_eq!(report.signature_recipe(), Some(recipe));
    assert!(report.source_health_complete());
    assert!(report.functions().iter().any(|fact| {
        fact.kind() == SourceFunctionKind::Closure
            && fact
                .signature()
                .and_then(SourceFunctionSignature::comparable)
                .is_some()
    }));
    let native_report_bytes = serde_json::to_vec(report)?;
    let configuration_bytes = serde_json::to_vec(project.configuration())?;
    let verify_native = |restored: &ProjectView| -> TestResult {
        assert_eq!(restored.snapshot_id(), project.snapshot_id());
        assert_eq!(
            restored.configuration().physical_function_signatures(),
            Some(recipe)
        );
        assert_eq!(
            serde_json::to_vec(restored.configuration())?,
            configuration_bytes
        );
        let restored_report = restored
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("restored native signature report")?;
        assert_eq!(restored_report.signature_recipe(), Some(recipe));
        assert_eq!(serde_json::to_vec(restored_report)?, native_report_bytes);
        Ok(())
    };

    let source_profile_id = project
        .configuration()
        .platform_packages()
        .ok_or("native platform packages")?
        .source()
        .profile()
        .profile_id()
        .clone();
    let ordinary_selection = PlatformStoreSelection::new(
        project.configuration().project_id().clone(),
        source_profile_id,
    )?;
    let selection = ordinary_selection
        .clone()
        .with_physical_function_signatures_catalog(recipe);
    let replay_input = LocalProjectInput::from_config_path(&path, &stop)?;
    let publication_input = LocalProjectInput::from_config_path(&path, &stop)?;

    // Retained native inputs remain the proof; decoded rows do not mint signatures.
    std::fs::remove_dir_all(root.0.join("UI"))?;
    std::fs::remove_file(root.0.join("version.txt"))?;
    assert!(!root.0.join("UI").exists());
    assert!(!root.0.join("version.txt").exists());
    let (bundle, _) =
        request.live_publication_in_namespace(replay_input, selection.namespace(), &stop)?;
    let (records, _) = bundle.into_parts();
    let record = records
        .iter()
        .find(|record| record.key() == "live.project.replay")
        .ok_or("selected signature replay record")?;
    let replay: ProjectReplay = record.decode()?;
    let expected_record = PartitionRecord::new(
        record.key(),
        "wow-project.live-replay.selected-components.v1",
        &replay,
    )?;
    assert_eq!(record.version(), expected_record.version());
    let wire = serde_json::to_value(&replay)?;
    assert_eq!(
        wire["schema"],
        "wow-project/native-project-replay/selected-components/1"
    );
    assert_eq!(
        wire["configuration"]["physical_function_signatures"],
        "wow-emmy/physical-function-indexed-types/1"
    );
    assert!(wire.get("function_fingerprints").is_none());
    let decoded = ProjectReplay::from_json(&serde_json::to_vec(&replay)?, &stop)?;
    verify_native(&decoded.hydrate(&stop)?)?;

    let store_root = root.0.join("signature-store");
    let publish =
        LiveProjectPublishRequest::new("fixture:indexed-signature-reopen", "absent", true, true)?;
    let published = publish_input_in_namespace(
        publication_input,
        &request,
        &store_root,
        &selection,
        &publish,
        &stop,
    )?;
    assert_eq!(serde_json::to_value(&published)?["status"], "activated");
    assert_eq!(
        LiveProjectStore::open_in_namespace(&store_root, &ordinary_selection)
            .err()
            .ok_or("ordinary catalog admitted selected signature epoch")?
            .code(),
        ServiceErrorCode::IdentityMismatch
    );

    let store = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    let current = store.current()?.ok_or("selected signature Current")?;
    let read = store.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    verify_native(read.project())?;
    let graph_snapshot_id = read.graph().snapshot().snapshot_id().clone();
    drop(read);
    drop(store);

    let reopened = LiveProjectStore::open_in_namespace(&store_root, &selection)?;
    assert_eq!(reopened.current()?, Some(current));
    let read = reopened.read_in_namespace(&selection, &ReadSelector::Current, &stop)?;
    verify_native(read.project())?;
    assert_eq!(read.graph().snapshot().snapshot_id(), &graph_snapshot_id);
    Ok(())
}

#[test]
fn selected_composed_neighborhood_tracks_incoming_call_with_unchanged_callee() -> TestResult {
    use crate::graph::FunctionDirectNeighborhoodRecipe;
    use wow_emmy::function_calls::SourceCallTarget;
    use wow_graph::{GraphAssertionRef, GraphConfidence, GraphEdge, GraphNode, GraphRelationKind};
    use wow_project::graph::fingerprints::{
        FunctionDirectNeighborhoodSelection, FunctionNeighborDirection, FunctionNeighborKind,
        FunctionNeighborhoodComponent, FunctionNeighborhoodUnavailable,
        FunctionSourceSpanComponent,
    };

    const CALLEE: &str = "local function Callee()\n    return true\nend\n";
    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let recipe = FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1;
    let mut span_observation = None;
    let mut comparable_digests = Vec::new();
    // Select a closed physical Lua scope. The fixture's unselected XML keeps
    // its unresolved load semantics and supplies no complete handler census.
    rewrite_load_role_toc(&root, &mut config, "## Interface: 120100\nmain.lua\n")?;

    // Add the FIRST distinct Caller -> Callee relation, not a duplicate call.
    // The callback control keeps the same finite selected package scope.
    for (caller, incoming) in [
        (
            "local function Caller()\n    return true\nend\n",
            Some(0_u64),
        ),
        (
            "local function Caller()\n    return Callee()\nend\n",
            Some(1),
        ),
        (
            "local function Caller(callback)\n    return callback()\nend\n",
            None,
        ),
    ] {
        rewrite_declaration_lua(&root, &mut config, &format!("{CALLEE}{caller}"))?;
        let (project, request, ids, source_bytes) = declaration_query(&path, &stop)?;
        assert_eq!(
            project.configuration().physical_function_declarations(),
            None
        );
        assert_eq!(project.configuration().physical_function_signatures(), None);
        let packages = project
            .configuration()
            .platform_packages()
            .ok_or("native package owners")?;
        assert!(
            packages.load_plan().coverage().complete(),
            "{:?}",
            packages.load_plan().issues()
        );
        assert_eq!(packages.main_plan().files().len(), 1);
        let main_file = packages
            .main_plan()
            .files()
            .first()
            .ok_or("physical Main file")?;
        let report = project
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("genuine native function-call report")?;
        report.validate()?;
        assert!(report.source_health_complete());
        let mut closures = report
            .functions()
            .iter()
            .filter(|fact| {
                fact.kind() == SourceFunctionKind::Closure && fact.path() == main_file.project_path
            })
            .collect::<Vec<_>>();
        closures.sort_by_key(|fact| fact.span().byte_start());
        assert_eq!(closures.len(), 2, "Callee then Caller in the physical file");
        let callee_fact = closures[0];
        let caller_fact = closures[1];
        let callee_bytes = span_bytes(&project, callee_fact)?.to_vec();

        // One ordinary observation checks that adding the read-side selection
        // changes neither the graph nor the unselected component bytes.
        let ordinary = if incoming == Some(0) {
            Some(execute_graph_build(
                LocalProjectInput::from_config_path(&path, &stop)?,
                &request,
                &stop,
            )?)
        } else {
            None
        };
        let selection = FunctionFingerprintSelection::new(
            ids.clone(),
            ids.len(),
            source_bytes,
            METADATA_BYTES,
        )?
        .with_direct_neighborhood(recipe)?;
        let request = request.with_function_fingerprints(selection)?;
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
        let manifest = result.function_fingerprints().ok_or("composed manifest")?;
        assert!(manifest.is_partial());
        assert!(manifest.scope_conflicts().is_empty());
        assert_eq!(
            manifest.binding().project_snapshot_id(),
            project.snapshot_id()
        );
        assert_eq!(
            manifest.binding().function_analysis_id(),
            report.analysis_id()
        );
        let selected = manifest
            .selected_components()
            .ok_or("selected component receipt")?;
        assert_eq!(selected.direct_neighborhood(), Some(recipe));
        assert_eq!(selected.physical_declarations(), None);
        assert_eq!(selected.physical_signatures(), None);
        assert_eq!(selected.package_load_role(), None);
        let callee = manifest
            .records()
            .iter()
            .find(|record| record.function_id() == callee_fact.fact_id())
            .ok_or("Callee record")?;
        let caller = manifest
            .records()
            .iter()
            .find(|record| record.function_id() == caller_fact.fact_id())
            .ok_or("Caller record")?;
        for record in manifest.records() {
            assert!(record.declared_header().is_none());
            assert_eq!(
                serde_json::to_value(record.signature())?,
                "missing_signature"
            );
            assert_eq!(
                serde_json::to_value(record.declaration_shape())?,
                "missing_shape"
            );
            assert_eq!(
                serde_json::to_value(record.package_load_role())?,
                "not_selected"
            );
        }
        let FunctionSourceSpanComponent::Available {
            digest,
            content_sha256,
            byte_length,
        } = callee.source_span()
        else {
            return Err("Callee's exact native span was not available".into());
        };
        assert_eq!(usize::try_from(*byte_length)?, callee_bytes.len());
        let observed_span = (*digest, *content_sha256, *byte_length);
        if let Some((bytes, span)) = &span_observation {
            assert_eq!(
                &callee_bytes, bytes,
                "Callee's exact source bytes stay fixed"
            );
            assert_eq!(
                &observed_span, span,
                "including the Callee source-span digest"
            );
        } else {
            span_observation = Some((callee_bytes, observed_span));
        }

        let graph: GraphPartitionSnapshot =
            serde_json::from_slice(&result.snapshot_bytes()?.ok_or("final graph")?)?;
        let lookup = graph.producer_lookup(&stop)?;
        assert_eq!(lookup.scope(), manifest.binding().graph_scope());
        let calls = graph
            .partition("wow-recognizers.lua-direct-calls")
            .ok_or("accepted call partition")?;
        let scripts = graph
            .partition("wow-recognizers.xml-script-bindings")
            .ok_or("accepted script partition")?;
        assert!(calls.report().ready_for_publication());
        assert!(scripts.report().ready_for_publication());
        let binding = manifest
            .neighborhood_binding()
            .ok_or("shared neighborhood binding")?;
        assert_eq!(binding.recipe(), recipe);
        assert_eq!(
            binding.prefix_snapshot_id(),
            manifest.binding().published_graph_snapshot_id()
        );
        assert_eq!(binding.final_snapshot_id(), graph.snapshot().snapshot_id());
        assert_eq!(binding.final_generation(), graph.snapshot().generation());
        assert_eq!(
            binding.registry_digest(),
            graph.registry().registry_digest()
        );
        assert_eq!(binding.call_partition_digest(), calls.partition_digest());
        assert_eq!(
            binding.call_validation_report_id(),
            calls.report().report_id()
        );
        assert_eq!(
            binding.script_partition_digest(),
            scripts.partition_digest()
        );
        assert_eq!(
            binding.script_validation_report_id(),
            scripts.report().report_id()
        );

        // Prefix producer assertions still resolve exactly in the FINAL owner.
        // Native constructors bind their accepted input nodes to final generation.
        let mut published_nodes = Vec::new();
        for record in [callee, caller] {
            let entity = lookup.entity(
                manifest.binding().graph_scope(),
                record.support().entity_assertion(),
                &stop,
            )?;
            let ownership = lookup.relation(
                manifest.binding().graph_scope(),
                record.support().incoming_ownership_assertion(),
                &stop,
            )?;
            let node = entity.accepted().node();
            assert_eq!(node.node_id(), record.exact_owner().node_id());
            assert_eq!(
                ownership.proposal().relation_kind_id(),
                "source_declaration_owns"
            );
            assert_eq!(ownership.accepted().edge().to(), node.node_id());
            let published = GraphNode::new(
                graph.snapshot().universe().clone(),
                graph.snapshot().generation().clone(),
                node.kind(),
                node.owner_key(),
                node.evidence_ids().to_vec(),
                graph.snapshot().limits(),
            )?;
            let final_node = graph
                .snapshot()
                .node(published.node_id())
                .ok_or("materialized physical function")?;
            assert_eq!(final_node.kind(), node.kind());
            assert_eq!(final_node.owner_key(), node.owner_key());
            published_nodes.push(published.node_id().clone());
        }
        if let Some(ordinary) = ordinary {
            assert_eq!(
                ordinary.status(),
                GraphReadStatus::Partial,
                "{:?}",
                ordinary.failure_message()
            );
            assert_eq!(ordinary.snapshot_bytes()?, result.snapshot_bytes()?);
            let old = ordinary
                .function_fingerprints()
                .ok_or("ordinary manifest")?;
            assert!(old.selected_components().is_none());
            assert!(old.neighborhood_binding().is_none());
            assert_eq!(old.binding(), manifest.binding());
            for record in manifest.records() {
                let old_record = old
                    .records()
                    .iter()
                    .find(|old| old.function_id() == record.function_id())
                    .ok_or("same ordinary physical function")?;
                assert!(matches!(
                    old_record.direct_neighborhood(),
                    FunctionDirectNeighborhoodSelection::NotSelected
                ));
                assert_eq!(
                    serde_json::to_value(old_record.source_span())?,
                    serde_json::to_value(record.source_span())?
                );
                assert_eq!(
                    serde_json::to_value(old_record.signature())?,
                    serde_json::to_value(record.signature())?
                );
                assert_eq!(
                    serde_json::to_value(old_record.declaration_shape())?,
                    serde_json::to_value(record.declaration_shape())?
                );
                assert_eq!(
                    serde_json::to_value(old_record.package_load_role())?,
                    serde_json::to_value(record.package_load_role())?
                );
            }
        }

        if let Some(incoming) = incoming {
            assert_eq!(u64::try_from(report.calls().len())?, incoming);
            assert_eq!(
                u64::try_from(calls.report().accepted_relations().len())?,
                incoming
            );
            for (record, expected_incoming, expected_outgoing) in
                [(callee, incoming, 0_u64), (caller, 0, incoming)]
            {
                let FunctionDirectNeighborhoodSelection::Selected(component) =
                    record.direct_neighborhood()
                else {
                    return Err("native neighborhood was not selected".into());
                };
                let FunctionNeighborhoodComponent::Comparable { digest, groups } =
                    component.as_ref()
                else {
                    return Err("closed native fixture neighborhood was not comparable".into());
                };
                assert_eq!(groups[0].relation_definition_id(), "lua_direct_calls");
                assert!(matches!(
                    groups[0].direction(),
                    FunctionNeighborDirection::Incoming
                ));
                assert!(matches!(
                    groups[0].neighbor_kind(),
                    FunctionNeighborKind::LuaSourceFunction
                ));
                assert_eq!(groups[0].multiplicity(), expected_incoming);
                assert_eq!(groups[1].relation_definition_id(), "lua_direct_calls");
                assert!(matches!(
                    groups[1].direction(),
                    FunctionNeighborDirection::Outgoing
                ));
                assert!(matches!(
                    groups[1].neighbor_kind(),
                    FunctionNeighborKind::LuaSourceFunction
                ));
                assert_eq!(groups[1].multiplicity(), expected_outgoing);
                assert_eq!(groups[2].relation_definition_id(), "source_xml_sets_script");
                assert!(matches!(
                    groups[2].direction(),
                    FunctionNeighborDirection::Incoming
                ));
                assert!(matches!(
                    groups[2].neighbor_kind(),
                    FunctionNeighborKind::XmlSourceDeclaration
                ));
                assert_eq!(
                    groups[2].multiplicity(),
                    0,
                    "inline XML is outside physical targets"
                );
                if record.function_id() == callee_fact.fact_id() {
                    comparable_digests.push(*digest);
                }
            }
            if incoming == 1 {
                let call = report.calls().first().ok_or("native direct call")?;
                assert_eq!(call.caller_function_id(), caller_fact.fact_id());
                assert!(
                    matches!(call.target(), SourceCallTarget::MainFunction { function_id }
                    if function_id.as_str() == callee_fact.fact_id())
                );
                let accepted = calls
                    .report()
                    .accepted_relations()
                    .first()
                    .ok_or("accepted distinct Caller -> Callee relation")?;
                let address = GraphAssertionRef::Producer {
                    partition_id: calls.partition_id().into(),
                    batch_id: calls.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Relation,
                        proposal_id: accepted.proposal_id().into(),
                    },
                };
                let resolved = lookup.relation(lookup.scope(), &address, &stop)?;
                assert_eq!(resolved.proposal().relation_kind_id(), "lua_direct_calls");
                assert_eq!(resolved.accepted().edge(), accepted.edge());
                let edge = accepted.edge();
                assert_eq!(edge.relation(), GraphRelationKind::Calls);
                assert_eq!(edge.confidence(), GraphConfidence::Derived);
                assert_eq!(edge.from(), caller.exact_owner().node_id());
                assert_eq!(edge.to(), callee.exact_owner().node_id());
                for record in [callee, caller] {
                    assert!(
                        resolved
                            .proposal()
                            .source_handle_ids()
                            .contains(&record.support().source_handle_id())
                    );
                    assert!(
                        resolved
                            .proposal()
                            .evidence_ids()
                            .contains(&record.support().evidence_id())
                    );
                }
                let wire = serde_json::to_value(&result)?;
                let receipts = wire["call_recognition"]["receipts"]
                    .as_array()
                    .ok_or("real call recognizer receipts")?;
                assert_eq!(receipts.len(), 1);
                assert_eq!(receipts[0]["call_id"], call.fact_id());
                assert_eq!(receipts[0]["outcome"]["state"], "projected");
                assert_eq!(
                    receipts[0]["outcome"]["proposal_id"],
                    accepted.proposal_id()
                );
                let final_edge = GraphEdge::new(
                    published_nodes[1].clone(),
                    published_nodes[0].clone(),
                    edge.relation(),
                    edge.confidence(),
                    edge.evidence_ids().to_vec(),
                    graph.snapshot().limits(),
                )?;
                assert_eq!(
                    graph.snapshot().edge(final_edge.edge_id()),
                    Some(&final_edge)
                );
            }
        } else {
            assert_eq!(report.calls().len(), 1, "one genuine callback call");
            assert!(matches!(
                report.calls()[0].target(),
                SourceCallTarget::SignatureNotCaptured
                    | SourceCallTarget::Unresolved
                    | SourceCallTarget::Indeterminate
            ));
            assert!(calls.report().accepted_relations().is_empty());
            for record in manifest.records() {
                let FunctionDirectNeighborhoodSelection::Selected(component) =
                    record.direct_neighborhood()
                else {
                    return Err("callback control neighborhood was not selected".into());
                };
                assert!(matches!(
                    component.as_ref(),
                    FunctionNeighborhoodComponent::Unavailable {
                        reason: FunctionNeighborhoodUnavailable::UnknownCallTarget,
                    }
                ));
                let wire = serde_json::to_value(component)?;
                assert!(wire.get("digest").is_none());
                assert!(
                    wire.get("groups").is_none(),
                    "no invented empty neighborhood"
                );
            }
        }
    }
    assert_eq!(comparable_digests.len(), 2);
    assert_ne!(comparable_digests[0], comparable_digests[1]);
    Ok(())
}

#[test]
fn selected_five_component_composite_tracks_signature_and_refuses_chunk() -> TestResult {
    use crate::graph::{
        FunctionCompositeRecipe, FunctionDirectNeighborhoodRecipe, FunctionPackageLoadRoleRecipe,
    };
    use wow_emmy::function_calls::{
        PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe, SourceSignatureType,
        SourceSignatureUnavailable,
    };
    use wow_project::graph::fingerprints::{
        FunctionCompositeRequiredComponent, FunctionDirectNeighborhoodSelection,
        FunctionNeighborhoodComponent, FunctionPackageLoadRoleComponent,
        FunctionPackageLoadRoleCoverage, FunctionPackageLoadRoleSelection,
        FunctionSignatureComponent, FunctionSourceSpanComponent,
    };

    let stop = AtomicBool::new(false);
    let (root, mut config) = fixture()?;
    let path = root.0.join("project.json");
    let declarations = PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1;
    let signatures = PhysicalFunctionSignatureRecipe::IndexedTypesV1;
    let neighborhood = FunctionDirectNeighborhoodRecipe::ComposedCallsAndXmlHandlersV1;
    let load_role = FunctionPackageLoadRoleRecipe::SelectedPhysicalLuaV1;
    let composite_recipe = FunctionCompositeRecipe::FiveCompleteComponentsV1;
    config["physical_function_declarations"] = serde_json::to_value(declarations)?;
    config["physical_function_signatures"] = serde_json::to_value(signatures)?;
    // Reuse the proven closed Lua-only fixture scope, not the unselected XML.
    rewrite_load_role_toc(&root, &mut config, "## Interface: 120100\nmain.lua\n")?;

    let mut observations = Vec::new();
    for (lua, expected_type) in [
        (
            "---@param value number\n---@return number\nlocal function TypedSignatureFixture(value)\n    return value\nend\n",
            SourceSignatureType::Number,
        ),
        (
            "---@param value string\n---@return string\nlocal function TypedSignatureFixture(value)\n    return value\nend\n",
            SourceSignatureType::String,
        ),
    ] {
        rewrite_declaration_lua(&root, &mut config, lua)?;
        let (project, request, ids, source_bytes) = declaration_query(&path, &stop)?;
        let packages = project
            .configuration()
            .platform_packages()
            .ok_or("native package owners")?;
        assert!(packages.load_plan().coverage().complete());
        assert_eq!(packages.main_plan().files().len(), 1);
        let file = packages
            .main_plan()
            .files()
            .first()
            .ok_or("native Main file")?;
        let report = project
            .snapshot()
            .analyzer_binding()
            .function_call_report()
            .ok_or("selected native report")?;
        report.validate()?;
        assert!(report.source_health_complete());
        assert_eq!(report.declaration_recipe(), Some(declarations));
        assert_eq!(report.signature_recipe(), Some(signatures));
        assert!(report.calls().is_empty());
        let mut closures = report.functions().iter().filter(|fact| {
            fact.kind() == SourceFunctionKind::Closure && fact.path() == file.project_path
        });
        let fact = closures.next().ok_or("native physical closure")?;
        assert!(closures.next().is_none(), "one physical fixture closure");
        let native_signature = fact.signature().ok_or("native indexed signature")?;
        let typed = native_signature
            .comparable()
            .ok_or("genuine supported native signature")?;
        assert!(native_signature.unavailable_reasons().is_empty());
        assert_eq!(typed.parameters().len(), 1);
        assert_eq!(
            typed
                .parameters()
                .first()
                .ok_or("native parameter")?
                .type_shape(),
            &expected_type
        );
        assert_eq!(typed.returns().len(), 1);
        assert_eq!(
            typed.returns().first().ok_or("native return")?.types(),
            std::slice::from_ref(&expected_type)
        );
        let physical_bytes = span_bytes(&project, fact)?.to_vec();
        let chunk = report
            .functions()
            .iter()
            .find(|fact| {
                fact.kind() == SourceFunctionKind::Chunk && fact.path() == file.project_path
            })
            .ok_or("genuine physical Chunk")?;
        let chunk_signature = chunk.signature().ok_or("native Chunk observation")?;
        assert!(chunk_signature.comparable().is_none());
        assert_eq!(
            chunk_signature.unavailable_reasons(),
            &[SourceSignatureUnavailable::ChunkHasNoSignature]
        );

        // Exact native IDs select the real report; no component DTO is supplied.
        let selection = FunctionFingerprintSelection::new(
            ids.clone(),
            ids.len(),
            source_bytes,
            METADATA_BYTES,
        )?
        .with_package_load_roles(load_role)?
        .with_direct_neighborhood(neighborhood)?
        .with_composite(composite_recipe)?;
        let result = execute_graph_build(
            LocalProjectInput::from_config_path(&path, &stop)?,
            &request.with_function_fingerprints(selection)?,
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
            .ok_or("selected service composite manifest")?;
        assert_eq!(
            manifest.schema(),
            "wow-project/function-fingerprint-records/selected-components/1"
        );
        assert!(manifest.is_partial());
        assert!(manifest.scope_conflicts().is_empty());
        assert_eq!(manifest.records().len(), ids.len());
        assert_eq!(
            manifest.binding().project_snapshot_id(),
            project.snapshot_id()
        );
        assert_eq!(
            manifest.binding().function_analysis_id(),
            report.analysis_id()
        );
        assert_eq!(
            manifest
                .neighborhood_binding()
                .ok_or("composed native binding")?
                .recipe(),
            neighborhood
        );
        let selected = manifest
            .selected_components()
            .ok_or("explicit selected recipe receipt")?;
        assert_eq!(selected.physical_declarations(), Some(declarations));
        assert_eq!(selected.physical_signatures(), Some(signatures));
        assert_eq!(selected.direct_neighborhood(), Some(neighborhood));
        assert_eq!(selected.package_load_role(), Some(load_role));
        assert_eq!(selected.composite(), Some(composite_recipe));
        let record = manifest
            .records()
            .iter()
            .find(|record| record.function_id() == fact.fact_id())
            .ok_or("genuine selected physical closure")?;
        let signature_digest = record.signature().digest().ok_or("complete signature")?;
        let shape_digest = record
            .declaration_shape()
            .digest()
            .ok_or("complete native declaration/body shape")?;
        let FunctionSourceSpanComponent::Available {
            digest: source_digest,
            byte_length,
            ..
        } = record.source_span()
        else {
            return Err("exact physical source span unavailable".into());
        };
        assert_eq!(usize::try_from(*byte_length)?, physical_bytes.len());
        let FunctionDirectNeighborhoodSelection::Selected(neighbors) = record.direct_neighborhood()
        else {
            return Err("direct neighborhood not selected".into());
        };
        let FunctionNeighborhoodComponent::Comparable {
            digest: neighborhood_digest,
            groups,
        } = neighbors.as_ref()
        else {
            return Err("closed native neighborhood unavailable".into());
        };
        assert!(groups.iter().all(|group| group.multiplicity() == 0));
        let FunctionPackageLoadRoleSelection::Selected(role) = record.package_load_role() else {
            return Err("package/load role not selected".into());
        };
        assert_eq!(role.coverage(), FunctionPackageLoadRoleCoverage::Complete);
        assert_eq!(role.native_load_coverage(), packages.load_plan().coverage());
        assert_eq!(
            role.witness().ok_or("native role witness")?.main_file(),
            file
        );
        let FunctionPackageLoadRoleComponent::Comparable {
            digest: role_digest,
            ..
        } = role.component()
        else {
            return Err("complete native package/load role unavailable".into());
        };
        let composite = record.composite().ok_or("selected composite")?;
        assert_eq!(composite.recipe(), composite_recipe);
        assert!(composite.incomplete_components().is_empty());
        let value = composite.comparable().ok_or("five complete inputs")?;
        let composite_digest = composite.digest().ok_or("complete composite digest")?;
        // Bind every comparable part to its actual component, never a blessed hash.
        assert_eq!(value.indexed_signature().digest(), signature_digest);
        assert_eq!(value.declaration_body_shape().digest(), shape_digest);
        assert_eq!(value.direct_neighborhood().digest(), *neighborhood_digest);
        assert_eq!(value.exact_source_span().digest(), *source_digest);
        assert_eq!(value.package_load_role().digest(), *role_digest);

        // The same genuine service observation supplies the unavailable control.
        let chunk_record = manifest
            .records()
            .iter()
            .find(|record| record.function_id() == chunk.fact_id())
            .ok_or("selected native Chunk record")?;
        let FunctionSignatureComponent::Unavailable { reasons } = chunk_record.signature() else {
            return Err("Chunk supplied an invented signature digest".into());
        };
        assert_eq!(reasons.as_slice(), chunk_signature.unavailable_reasons());
        assert!(chunk_record.signature().digest().is_none());
        assert!(matches!(
            chunk_record.source_span(),
            FunctionSourceSpanComponent::Available { .. }
        ));
        let refused = chunk_record.composite().ok_or("selected Chunk composite")?;
        assert_eq!(refused.recipe(), composite_recipe);
        assert!(
            refused
                .incomplete_components()
                .contains(&FunctionCompositeRequiredComponent::IndexedSignature)
        );
        assert!(refused.digest().is_none());
        assert!(refused.comparable().is_none());
        let refused_wire = serde_json::to_value(refused)?;
        assert_eq!(refused_wire["state"], "unavailable");
        assert!(refused_wire.get("digest").is_none());
        assert!(refused_wire.get("comparable").is_none());
        observations.push((
            signature_digest,
            shape_digest,
            *neighborhood_digest,
            *source_digest,
            *role_digest,
            composite_digest,
            physical_bytes,
        ));
    }

    let [number, string] = observations.as_slice() else {
        return Err("two native mutation observations required".into());
    };
    assert_ne!(number.0, string.0, "annotation types change signature");
    assert_eq!(number.1, string.1, "native declaration/body shape fixed");
    assert_eq!(number.2, string.2, "closed direct neighborhood fixed");
    assert_eq!(number.3, string.3, "exact physical span component fixed");
    assert_eq!(number.4, string.4, "selected package/load role fixed");
    assert_ne!(
        number.5, string.5,
        "composite follows the changed component"
    );
    assert_eq!(number.6, string.6, "exact closure body bytes fixed");
    Ok(())
}
