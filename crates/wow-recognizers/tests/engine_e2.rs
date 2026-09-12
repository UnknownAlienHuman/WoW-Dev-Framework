use std::error::Error;
use std::sync::atomic::AtomicBool;

use wow_graph::{
    GraphConfidence, GraphCoverageState, GraphGenerationId, GraphLimits, GraphNode, GraphSnapshot,
    GraphUniverseId,
};
use wow_recognizers::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognizerErrorCode, RecognizerLimits, RecognizerRegistry, StructuredObservation,
    project_graph_coverage, project_graph_edges, run_recognizers,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn source_graph(generation_name: &str) -> TestResult<(GraphSnapshot, Vec<GraphNode>)> {
    let limits = GraphLimits::default();
    let universe = GraphUniverseId::new("fixture")?;
    let generation = GraphGenerationId::new(generation_name)?;
    let nodes = [
        ("function", "file:A.lua#caller"),
        ("function", "api:C_Test.First"),
        ("function", "api:C_Test.Second"),
        ("function", "api:C_Test.Missing"),
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

fn complete_coverage(limits: RecognizerLimits) -> TestResult<Vec<RecognitionCoverage>> {
    Ok(ObservationFamily::ALL
        .into_iter()
        .map(|family| {
            RecognitionCoverage::new(
                family,
                RecognitionCoverageState::Complete,
                Vec::new(),
                limits,
            )
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn observation(
    snapshot: &GraphSnapshot,
    nodes: &[GraphNode],
    family: ObservationFamily,
    target: usize,
    origin: ObservationOrigin,
    confidence: GraphConfidence,
    evidence: &str,
    limits: RecognizerLimits,
) -> TestResult<StructuredObservation> {
    Ok(StructuredObservation::new(
        snapshot.snapshot_id().clone(),
        family,
        nodes[0].node_id().clone(),
        nodes[target].node_id().clone(),
        origin,
        confidence,
        vec![evidence.into()],
        limits,
    )?)
}

#[test]
fn exact_observations_produce_stable_assertions_edges_and_non_authoritative_coverage()
-> TestResult {
    let (snapshot, nodes) = source_graph("generation-1")?;
    let limits = RecognizerLimits::default();
    let registry = RecognizerRegistry::e2_default()?;
    let first = observation(
        &snapshot,
        &nodes,
        ObservationFamily::DirectCall,
        1,
        ObservationOrigin::AnalyzerFact,
        GraphConfidence::Proven,
        "emmy-call:1",
        limits,
    )?;
    let second = observation(
        &snapshot,
        &nodes,
        ObservationFamily::ApiUse,
        2,
        ObservationOrigin::ReferenceFact,
        GraphConfidence::Derived,
        "reference-use:1",
        limits,
    )?;
    let cancelled = AtomicBool::new(false);
    let report = run_recognizers(
        &registry,
        &snapshot,
        vec![second.clone(), first.clone()],
        complete_coverage(limits)?,
        limits,
        &cancelled,
    )?;
    let reordered = run_recognizers(
        &registry,
        &snapshot,
        vec![first, second],
        complete_coverage(limits)?,
        limits,
        &cancelled,
    )?;
    assert_eq!(report, reordered);
    assert_eq!(report.assertions().len(), 2);
    assert!(
        report
            .assertions()
            .iter()
            .all(|assertion| assertion.confidence() == GraphConfidence::Derived)
    );

    let edges = project_graph_edges(&report, GraphLimits::default(), &cancelled)?;
    assert_eq!(edges.len(), 2);
    assert!(edges.iter().all(|edge| {
        edge.evidence_ids()
            .iter()
            .any(|id| id.starts_with("recognizer-assertion:sha256:"))
    }));
    let graph_coverage =
        project_graph_coverage(&report, GraphLimits::default(), &cancelled)?;
    assert_eq!(graph_coverage.len(), ObservationFamily::ALL.len());
    assert!(graph_coverage.iter().all(|record| {
        record.state() == GraphCoverageState::Complete && !record.negative_authority()
    }));
    Ok(())
}

#[test]
fn output_limit_truncates_canonically_and_downgrades_only_affected_coverage() -> TestResult {
    let (snapshot, nodes) = source_graph("generation-1")?;
    let limits = RecognizerLimits::new(16, 1, 64, 64)?;
    let registry = RecognizerRegistry::e2_default()?;
    let observations = vec![
        observation(
            &snapshot,
            &nodes,
            ObservationFamily::DirectCall,
            1,
            ObservationOrigin::AnalyzerFact,
            GraphConfidence::Derived,
            "emmy-call:one",
            limits,
        )?,
        observation(
            &snapshot,
            &nodes,
            ObservationFamily::DirectCall,
            2,
            ObservationOrigin::AnalyzerFact,
            GraphConfidence::Derived,
            "emmy-call:two",
            limits,
        )?,
    ];
    let cancelled = AtomicBool::new(false);
    let report = run_recognizers(
        &registry,
        &snapshot,
        observations,
        complete_coverage(limits)?,
        limits,
        &cancelled,
    )?;
    assert_eq!(report.assertions().len(), 1);
    let direct = report
        .coverage()
        .iter()
        .find(|record| record.family() == ObservationFamily::DirectCall)
        .ok_or("direct-call coverage")?;
    assert_eq!(direct.state(), RecognitionCoverageState::Truncated);
    assert_eq!(direct.blocker_ids().len(), 1);
    assert_eq!(
        direct.blocker_ids()[0].as_ref(),
        "recognizer-budget:max-assertions"
    );
    let projected = project_graph_coverage(&report, GraphLimits::default(), &cancelled)?;
    let direct = projected
        .iter()
        .find(|record| record.relation() == ObservationFamily::DirectCall.relation())
        .ok_or("projected direct-call coverage")?;
    assert_eq!(direct.state(), GraphCoverageState::Partial);
    assert!(!direct.negative_authority());
    Ok(())
}

#[test]
fn stale_endpoints_duplicate_inputs_failed_coverage_and_cancellation_fail_closed() -> TestResult {
    let (snapshot, nodes) = source_graph("generation-1")?;
    let limits = RecognizerLimits::default();
    let registry = RecognizerRegistry::e2_default()?;
    let valid = observation(
        &snapshot,
        &nodes,
        ObservationFamily::DirectCall,
        1,
        ObservationOrigin::AnalyzerFact,
        GraphConfidence::Derived,
        "emmy-call:valid",
        limits,
    )?;
    let cancelled = AtomicBool::new(false);
    let duplicate = run_recognizers(
        &registry,
        &snapshot,
        vec![valid.clone(), valid.clone()],
        complete_coverage(limits)?,
        limits,
        &cancelled,
    )
    .expect_err("duplicate observation must fail");
    assert_eq!(duplicate.code(), RecognizerErrorCode::ObservationDuplicate);

    let (other_snapshot, _) = source_graph("generation-2")?;
    let stale = StructuredObservation::new(
        other_snapshot.snapshot_id().clone(),
        ObservationFamily::DirectCall,
        nodes[0].node_id().clone(),
        nodes[1].node_id().clone(),
        ObservationOrigin::AnalyzerFact,
        GraphConfidence::Derived,
        vec!["emmy-call:stale".into()],
        limits,
    )?;
    let stale = run_recognizers(
        &registry,
        &snapshot,
        vec![stale],
        complete_coverage(limits)?,
        limits,
        &cancelled,
    )
    .expect_err("stale observation must fail");
    assert_eq!(stale.code(), RecognizerErrorCode::ObservationSnapshotMismatch);

    let mut failed_coverage = complete_coverage(limits)?;
    let index = failed_coverage
        .iter()
        .position(|record| record.family() == ObservationFamily::DirectCall)
        .ok_or("direct-call coverage")?;
    failed_coverage[index] = RecognitionCoverage::new(
        ObservationFamily::DirectCall,
        RecognitionCoverageState::Failed,
        vec!["emmy-capability:failed".into()],
        limits,
    )?;
    let failed = run_recognizers(
        &registry,
        &snapshot,
        vec![valid],
        failed_coverage,
        limits,
        &cancelled,
    )
    .expect_err("failed coverage cannot carry observations");
    assert_eq!(failed.code(), RecognizerErrorCode::ObservationInvalid);

    let stopped = AtomicBool::new(true);
    let stopped = run_recognizers(
        &registry,
        &snapshot,
        Vec::new(),
        complete_coverage(limits)?,
        limits,
        &stopped,
    )
    .expect_err("cancelled operation must fail");
    assert_eq!(stopped.code(), RecognizerErrorCode::Cancelled);
    Ok(())
}

#[test]
fn external_candidates_never_become_derived_or_proven() -> TestResult {
    let (snapshot, nodes) = source_graph("generation-1")?;
    let limits = RecognizerLimits::default();
    let report = run_recognizers(
        &RecognizerRegistry::e2_default()?,
        &snapshot,
        vec![observation(
            &snapshot,
            &nodes,
            ObservationFamily::FactoryCreation,
            1,
            ObservationOrigin::ExternalCandidate,
            GraphConfidence::Proven,
            "candidate-provider:1",
            limits,
        )?],
        complete_coverage(limits)?,
        limits,
        &AtomicBool::new(false),
    )?;
    assert_eq!(report.assertions()[0].confidence(), GraphConfidence::Candidate);
    Ok(())
}
