use std::error::Error;
use std::sync::atomic::AtomicBool;

use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyReferenceResolution, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse, analyze_member_calls,
};
use wow_graph::{
    GraphConfidence, GraphGenerationId, GraphLimits, GraphNode, GraphSnapshot, GraphUniverseId,
};
use wow_recognizers::{
    EmmyDirectCallBinding, ObservationFamily, RecognitionCoverageState, RecognizerErrorCode,
    RecognizerLimits, adapt_emmy_direct_calls,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn digest(byte: char) -> String {
    format!("sha256:{}", byte.to_string().repeat(64))
}

fn backend() -> TestResult<EmmyBackendIdentity> {
    Ok(EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        digest('1'),
        digest('2'),
    )?)
}

fn workspace(
    universe: LuaWorkspaceUniverse,
    path: &str,
    text: &str,
) -> TestResult<LuaWorkspaceSnapshot> {
    Ok(LuaWorkspaceSnapshot::build(
        backend()?,
        universe,
        vec![LuaWorkspaceFileInput::new(path, text)],
        LuaWorkspaceLimits::new(16, 1024, 64 * 1024, 256 * 1024)?,
    )?)
}

fn library() -> TestResult<LuaWorkspaceSnapshot> {
    workspace(
        LuaWorkspaceUniverse::Fixture,
        "library/C_Adapter.lua",
        "---@meta _\n---@class C_Adapter\n---@field Known fun(): boolean\nC_Adapter = {}\n",
    )
}

fn graph() -> TestResult<(GraphSnapshot, Vec<GraphNode>)> {
    let limits = GraphLimits::default();
    let universe = GraphUniverseId::new("fixture")?;
    let generation = GraphGenerationId::new("generation-1")?;
    let nodes = [
        ("function", "file:main/calls.lua#chunk"),
        ("function", "api:C_Adapter.Known"),
        ("function", "candidate:C_Adapter.Missing"),
    ]
    .into_iter()
    .map(|(kind, owner)| {
        GraphNode::new(
            universe.clone(),
            generation.clone(),
            kind,
            owner,
            Vec::new(),
            limits,
        )
    })
    .collect::<Result<Vec<_>, _>>()?;
    let snapshot = GraphSnapshot::build(
        universe,
        generation,
        limits,
        nodes.clone(),
        Vec::new(),
        Vec::new(),
    )?;
    Ok((snapshot, nodes))
}

#[test]
fn explicit_bindings_preserve_resolution_without_name_inference() -> TestResult {
    let main = workspace(
        LuaWorkspaceUniverse::Fixture,
        "main/calls.lua",
        "local known = C_Adapter.Known()\nlocal missing = C_Adapter.Missing()\n",
    )?;
    let library = library()?;
    let report = analyze_member_calls(&main, &[&library])?;
    let known_reference = report
        .references()
        .iter()
        .find(|fact| fact.member() == "Known")
        .ok_or("known reference")?;
    let missing_reference = report
        .references()
        .iter()
        .find(|fact| fact.member() == "Missing")
        .ok_or("missing reference")?;
    assert_eq!(
        known_reference.resolution(),
        EmmyReferenceResolution::Resolved
    );
    assert_eq!(
        missing_reference.resolution(),
        EmmyReferenceResolution::Unresolved
    );
    let known_call = report
        .calls()
        .iter()
        .find(|call| call.reference_fact_id() == known_reference.fact_id())
        .ok_or("known call")?;
    let missing_call = report
        .calls()
        .iter()
        .find(|call| call.reference_fact_id() == missing_reference.fact_id())
        .ok_or("missing call")?;
    let (graph, nodes) = graph()?;
    let limits = RecognizerLimits::default();
    let bindings = vec![
        EmmyDirectCallBinding::new(
            missing_call.fact_id(),
            nodes[0].node_id().clone(),
            nodes[2].node_id().clone(),
            vec!["graph-binding:missing".into()],
            limits,
        )?,
        EmmyDirectCallBinding::new(
            known_call.fact_id(),
            nodes[0].node_id().clone(),
            nodes[1].node_id().clone(),
            vec!["graph-binding:known".into()],
            limits,
        )?,
    ];
    let adapted = adapt_emmy_direct_calls(
        &report,
        &graph,
        bindings.clone(),
        limits,
        &AtomicBool::new(false),
    )?;
    let reordered = adapt_emmy_direct_calls(
        &report,
        &graph,
        bindings.into_iter().rev().collect(),
        limits,
        &AtomicBool::new(false),
    )?;
    assert_eq!(adapted, reordered);
    assert_eq!(adapted.source_analysis_id(), report.analysis_id());
    assert_eq!(adapted.source_main_snapshot_id(), main.snapshot_id());
    assert_eq!(adapted.graph_snapshot_id(), graph.snapshot_id());
    assert_eq!(
        adapted.coverage().state(),
        RecognitionCoverageState::Complete
    );
    assert_eq!(adapted.observations().len(), 2);
    let known = adapted
        .observations()
        .iter()
        .find(|observation| observation.to() == nodes[1].node_id())
        .ok_or("known observation")?;
    let missing = adapted
        .observations()
        .iter()
        .find(|observation| observation.to() == nodes[2].node_id())
        .ok_or("missing observation")?;
    assert_eq!(known.family(), ObservationFamily::DirectCall);
    assert_eq!(known.confidence(), GraphConfidence::Derived);
    assert_eq!(missing.confidence(), GraphConfidence::Possible);
    assert!(
        known
            .evidence_ids()
            .iter()
            .any(|id| id.as_ref() == known_call.fact_id())
    );
    assert!(
        missing
            .evidence_ids()
            .iter()
            .any(|id| id.as_ref() == missing_reference.fact_id())
    );
    adapted.validate(limits)?;
    Ok(())
}

#[test]
fn malformed_main_file_produces_partial_coverage_without_invented_calls() -> TestResult {
    let main = workspace(
        LuaWorkspaceUniverse::Fixture,
        "main/broken.lua",
        "local function broken(..., value) end\n",
    )?;
    let report = analyze_member_calls(&main, &[])?;
    let (graph, _) = graph()?;
    let adapted = adapt_emmy_direct_calls(
        &report,
        &graph,
        Vec::new(),
        RecognizerLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert!(adapted.observations().is_empty());
    assert_eq!(
        adapted.coverage().state(),
        RecognitionCoverageState::Partial
    );
    assert_eq!(adapted.coverage().blocker_ids().len(), 1);
    assert!(adapted.coverage().blocker_ids()[0].starts_with("emmy-file-parse-failed:sha256:"));
    Ok(())
}

#[test]
fn missing_unknown_and_duplicate_bindings_fail_closed() -> TestResult {
    let main = workspace(
        LuaWorkspaceUniverse::Fixture,
        "main/calls.lua",
        "return C_Adapter.Known()\n",
    )?;
    let library = library()?;
    let report = analyze_member_calls(&main, &[&library])?;
    let call = report.calls().first().ok_or("call fact")?;
    let (graph, nodes) = graph()?;
    let limits = RecognizerLimits::default();
    let missing =
        adapt_emmy_direct_calls(&report, &graph, Vec::new(), limits, &AtomicBool::new(false))
            .err()
            .ok_or("missing binding must fail")?;
    assert_eq!(missing.code(), RecognizerErrorCode::AdapterBindingMissing);

    let valid = EmmyDirectCallBinding::new(
        call.fact_id(),
        nodes[0].node_id().clone(),
        nodes[1].node_id().clone(),
        Vec::new(),
        limits,
    )?;
    let duplicate = adapt_emmy_direct_calls(
        &report,
        &graph,
        vec![valid.clone(), valid],
        limits,
        &AtomicBool::new(false),
    )
    .err()
    .ok_or("duplicate binding must fail")?;
    assert_eq!(
        duplicate.code(),
        RecognizerErrorCode::AdapterBindingDuplicate
    );

    let unknown_id = format!("emmy-call:sha256:{}", "f".repeat(64));
    let unknown = adapt_emmy_direct_calls(
        &report,
        &graph,
        vec![EmmyDirectCallBinding::new(
            unknown_id,
            nodes[0].node_id().clone(),
            nodes[1].node_id().clone(),
            Vec::new(),
            limits,
        )?],
        limits,
        &AtomicBool::new(false),
    )
    .err()
    .ok_or("unknown binding must fail")?;
    assert_eq!(unknown.code(), RecognizerErrorCode::AdapterBindingUnknown);
    Ok(())
}

#[test]
fn cancellation_prevents_adapter_publication() -> TestResult {
    let main = workspace(
        LuaWorkspaceUniverse::Fixture,
        "main/calls.lua",
        "return C_Adapter.Known()\n",
    )?;
    let library = library()?;
    let report = analyze_member_calls(&main, &[&library])?;
    let (graph, _) = graph()?;
    let cancelled = adapt_emmy_direct_calls(
        &report,
        &graph,
        Vec::new(),
        RecognizerLimits::default(),
        &AtomicBool::new(true),
    )
    .err()
    .ok_or("cancelled adapter must fail")?;
    assert_eq!(cancelled.code(), RecognizerErrorCode::Cancelled);
    Ok(())
}
