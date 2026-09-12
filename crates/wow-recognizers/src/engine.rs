use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use wow_graph::{
    GraphCoverageRecord, GraphCoverageState, GraphEdge, GraphError, GraphLimits, GraphSnapshot,
};

use crate::{
    ObservationFamily, RecognitionAssertion, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation,
};

const ASSERTION_BUDGET_BLOCKER: &str = "recognizer-budget:max-assertions";
const TRUNCATED_COVERAGE_BLOCKER: &str = "recognizer-output:truncated";

/// Applies the closed registry to exact structured observations from one immutable graph snapshot.
///
/// This function never parses source, discovers facts, or creates graph nodes. Every observation
/// must bind the exact input snapshot and two existing endpoints. Equivalent input order produces
/// the same report. Output pressure truncates in canonical assertion-id order and downgrades the
/// affected family coverage instead of silently dropping matches.
pub fn run_recognizers(
    registry: &RecognizerRegistry,
    source: &GraphSnapshot,
    mut observations: Vec<StructuredObservation>,
    coverage: Vec<RecognitionCoverage>,
    limits: RecognizerLimits,
    cancelled: &AtomicBool,
) -> RecognizerResult<RecognitionReport> {
    check_cancelled(cancelled)?;
    registry.validate()?;
    source.validate().map_err(graph_projection_error)?;
    limits.validate()?;
    if observations.len() > limits.max_observations as usize
        || coverage.len() > limits.max_coverage_records as usize
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::BudgetExceeded,
            "recognizer input exceeds configured limits",
        ));
    }

    let node_ids = source
        .nodes()
        .iter()
        .map(|node| node.node_id().clone())
        .collect::<BTreeSet<_>>();
    observations.sort_by(|left, right| left.observation_id().cmp(right.observation_id()));
    if observations
        .windows(2)
        .any(|pair| pair[0].observation_id() == pair[1].observation_id())
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::ObservationDuplicate,
            "recognizer input contains a duplicate structured observation",
        ));
    }

    let mut coverage_by_family = BTreeMap::new();
    for record in coverage {
        check_cancelled(cancelled)?;
        record.validate(limits)?;
        let family = record.family();
        if coverage_by_family.insert(family, record).is_some() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::CoverageDuplicate,
                "recognizer input contains duplicate family coverage",
            ));
        }
    }
    if coverage_by_family.len() != ObservationFamily::ALL.len()
        || ObservationFamily::ALL
            .into_iter()
            .any(|family| !coverage_by_family.contains_key(&family))
    {
        return Err(RecognizerError::new(
            RecognizerErrorCode::CoverageInvalid,
            "recognizer input must describe every E2-B family exactly once",
        ));
    }

    let mut matched = Vec::with_capacity(observations.len());
    for observation in &observations {
        check_cancelled(cancelled)?;
        observation.validate(limits)?;
        if observation.source_snapshot_id() != source.snapshot_id() {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationSnapshotMismatch,
                "structured observation belongs to another graph snapshot",
            ));
        }
        if !node_ids.contains(observation.from()) || !node_ids.contains(observation.to()) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationEndpointMissing,
                "structured observation references a missing graph endpoint",
            ));
        }
        let family_coverage = coverage_by_family
            .get(&observation.family())
            .ok_or_else(|| {
                RecognizerError::new(
                    RecognizerErrorCode::CoverageInvalid,
                    "observation family has no coverage record",
                )
            })?;
        if matches!(
            family_coverage.state(),
            RecognitionCoverageState::NotEvaluated | RecognitionCoverageState::Failed
        ) {
            return Err(RecognizerError::new(
                RecognizerErrorCode::ObservationInvalid,
                "not-evaluated or failed family coverage cannot carry recognized observations",
            ));
        }
        let descriptor = registry.descriptor(observation.family()).ok_or_else(|| {
            RecognizerError::new(
                RecognizerErrorCode::RegistryIncomplete,
                "registry has no recognizer for an observation family",
            )
        })?;
        matched.push((
            observation.family(),
            RecognitionAssertion::from_observation(descriptor, observation)?,
        ));
    }

    matched.sort_by(|left, right| left.1.assertion_id().cmp(right.1.assertion_id()));
    let mut truncated_families = BTreeSet::new();
    if matched.len() > limits.max_assertions as usize {
        for (family, _) in matched.drain(limits.max_assertions as usize..) {
            truncated_families.insert(family);
        }
    }
    for family in truncated_families {
        let existing = coverage_by_family.remove(&family).ok_or_else(|| {
            RecognizerError::new(
                RecognizerErrorCode::CoverageInvalid,
                "truncated family has no coverage record",
            )
        })?;
        let mut blockers = existing.blocker_ids().to_vec();
        blockers.push(ASSERTION_BUDGET_BLOCKER.into());
        blockers.sort();
        blockers.dedup();
        coverage_by_family.insert(
            family,
            RecognitionCoverage::new(
                family,
                RecognitionCoverageState::Truncated,
                blockers,
                limits,
            )?,
        );
    }

    let assertions = matched
        .into_iter()
        .map(|(_, assertion)| assertion)
        .collect();
    let coverage = coverage_by_family.into_values().collect();
    check_cancelled(cancelled)?;
    RecognitionReport::build(
        registry.registry_id().clone(),
        source.snapshot_id().clone(),
        source.universe().clone(),
        source.generation().clone(),
        limits,
        assertions,
        coverage,
    )
}

/// Converts recognizer assertions into graph-owned edges without changing endpoint, relation,
/// confidence, or evidence semantics. Graph validation remains authoritative for graph records.
pub fn project_graph_edges(
    report: &RecognitionReport,
    limits: GraphLimits,
    cancelled: &AtomicBool,
) -> RecognizerResult<Vec<GraphEdge>> {
    check_cancelled(cancelled)?;
    report.validate()?;
    let mut edges = Vec::with_capacity(report.assertions().len());
    for assertion in report.assertions() {
        check_cancelled(cancelled)?;
        let mut evidence = assertion.evidence_ids().to_vec();
        evidence.push(assertion.assertion_id().as_str().into());
        evidence.sort();
        evidence.dedup();
        edges.push(
            GraphEdge::new(
                assertion.from().clone(),
                assertion.to().clone(),
                assertion.relation(),
                assertion.confidence(),
                evidence,
                limits,
            )
            .map_err(graph_projection_error)?,
        );
    }
    edges.sort_by(|left, right| left.edge_id().cmp(right.edge_id()));
    check_cancelled(cancelled)?;
    Ok(edges)
}

/// Converts family coverage to graph relation coverage. Recognizers never create negative
/// authority. Truncated recognizer output becomes partial graph coverage with an explicit blocker.
pub fn project_graph_coverage(
    report: &RecognitionReport,
    limits: GraphLimits,
    cancelled: &AtomicBool,
) -> RecognizerResult<Vec<GraphCoverageRecord>> {
    check_cancelled(cancelled)?;
    report.validate()?;
    let mut projected = Vec::with_capacity(report.coverage().len());
    for record in report.coverage() {
        check_cancelled(cancelled)?;
        let state = match record.state() {
            RecognitionCoverageState::Complete => GraphCoverageState::Complete,
            RecognitionCoverageState::Partial | RecognitionCoverageState::Truncated => {
                GraphCoverageState::Partial
            }
            RecognitionCoverageState::NotEvaluated => GraphCoverageState::NotEvaluated,
            RecognitionCoverageState::Failed => GraphCoverageState::Failed,
        };
        let mut blockers = record.blocker_ids().to_vec();
        if record.state() == RecognitionCoverageState::Truncated {
            blockers.push(TRUNCATED_COVERAGE_BLOCKER.into());
            blockers.sort();
            blockers.dedup();
        }
        projected.push(
            GraphCoverageRecord::new(record.family().relation(), state, false, blockers, limits)
                .map_err(graph_projection_error)?,
        );
    }
    projected.sort_by_key(GraphCoverageRecord::relation);
    check_cancelled(cancelled)?;
    Ok(projected)
}

fn check_cancelled(cancelled: &AtomicBool) -> RecognizerResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(RecognizerError::new(
            RecognizerErrorCode::Cancelled,
            "recognizer operation was cancelled",
        ));
    }
    Ok(())
}

fn graph_projection_error(source: GraphError) -> RecognizerError {
    RecognizerError::new(RecognizerErrorCode::GraphProjectionFailed, source.message())
}
