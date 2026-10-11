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
