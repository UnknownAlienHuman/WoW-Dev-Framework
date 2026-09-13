use std::collections::BTreeSet;
use std::error::Error;
use std::sync::atomic::AtomicBool;

use serde_json::json;
use wow_core::canonical_json_bytes;
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphDirection, GraphEdge,
    GraphErrorCode, GraphGenerationId, GraphLimits, GraphNode, GraphNodeId, GraphPathConfidence,
    GraphPathCursor, GraphPathLimits, GraphPathQuery, GraphPathTruncation, GraphQueryState,
    GraphRelationKind, GraphSnapshot, GraphUniverseId,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn node(key: &str) -> TestResult<GraphNode> {
    Ok(GraphNode::new(
        GraphUniverseId::new("project:paths")?,
        GraphGenerationId::new("generation:paths:1")?,
        "function",
        key,
        vec![format!("evidence:{key}").into()],
        GraphLimits::default(),
    )?)
}

fn fixture(reverse: bool) -> TestResult<GraphSnapshot> {
    let mut nodes = ["a", "b", "c", "d", "x", "z"]
        .into_iter()
        .map(node)
        .collect::<TestResult<Vec<_>>>()?;
    let mut edges = Vec::new();
    for (a, b, confidence) in [
        (0, 1, GraphConfidence::Proven),
        (1, 3, GraphConfidence::Proven),
        (0, 2, GraphConfidence::Proven),
        (2, 3, GraphConfidence::Derived),
        (1, 0, GraphConfidence::Proven), // A cycle, not a new transitive edge.
        (0, 3, GraphConfidence::Possible),
        (0, 4, GraphConfidence::Candidate),
        (4, 3, GraphConfidence::Proven),
    ] {
        edges.push(GraphEdge::new(
            nodes[a].node_id().clone(),
            nodes[b].node_id().clone(),
            GraphRelationKind::Calls,
            confidence,
            vec![format!("evidence:edge:{a}:{b}").into()],
            GraphLimits::default(),
        )?);
    }
    let mut coverage = vec![
        GraphCoverageRecord::new(
            GraphRelationKind::Calls,
            GraphCoverageState::Complete,
            true,
            Vec::new(),
            GraphLimits::default(),
        )?,
        GraphCoverageRecord::new(
            GraphRelationKind::Owns,
            GraphCoverageState::Complete,
            true,
            Vec::new(),
            GraphLimits::default(),
        )?,
        GraphCoverageRecord::new(
            GraphRelationKind::UsesApi,
            GraphCoverageState::Partial,
            false,
            vec!["fixture:partial".into()],
            GraphLimits::default(),
        )?,
    ];
    if reverse {
        nodes.reverse();
        edges.reverse();
        coverage.reverse();
    }
    Ok(GraphSnapshot::build(
        GraphUniverseId::new("project:paths")?,
        GraphGenerationId::new("generation:paths:1")?,
        GraphLimits::default(),
        nodes,
        edges,
        coverage,
    )?)
}

fn id(snapshot: &GraphSnapshot, key: &str) -> TestResult<GraphNodeId> {
    Ok(snapshot
        .nodes()
        .iter()
        .find(|node| node.owner_key() == key)
        .ok_or("missing fixture node")?
        .node_id()
        .clone())
}

fn query(snapshot: &GraphSnapshot, limits: GraphPathLimits) -> TestResult<GraphPathQuery> {
    Ok(GraphPathQuery::new(
        snapshot.snapshot_id().clone(),
        id(snapshot, "a")?,
        id(snapshot, "d")?,
        GraphDirection::Outgoing,
        vec![GraphRelationKind::Calls],
        limits,
    )?)
}

#[test]
fn paths_preserve_evidence_weakest_confidence_and_canonical_order() -> TestResult {
    let graph = fixture(false)?;
    let original = canonical_json_bytes(&graph)?;
    let q = query(&graph, GraphPathLimits::default())?;
    let result = q.execute(&graph, None, &AtomicBool::new(false))?;
    assert_eq!(result.state(), GraphQueryState::Complete);
    assert_eq!(result.paths().len(), 2);
    assert!(!result.absence_authoritative());
    assert!(!result.prior_truncation());
    assert!(result.continuation().is_none());
    let keys = result
        .paths()
        .iter()
        .map(|path| {
            path.edges()
                .iter()
                .map(|edge| edge.edge_id().clone())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert!(keys.windows(2).all(|pair| pair[0] < pair[1]));
    for path in result.paths() {
        assert_eq!(path.nodes().len(), path.edges().len() + 1);
        assert_eq!(
            path.confidence(),
            path.edges()
                .iter()
                .map(|edge| edge.confidence())
                .max()
                .ok_or("path edge")?
        );
        for edge in path.edges() {
            assert!(graph.edges().contains(edge));
            assert!(!edge.evidence_ids().is_empty());
        }
        for node in path.nodes() {
            assert_eq!(graph.node(node.node_id()), Some(node));
        }
    }
    assert_eq!(
        result,
        q.execute(&fixture(true)?, None, &AtomicBool::new(false))?
    );
    assert_eq!(canonical_json_bytes(&graph)?, original);
    assert_eq!(
        query(&graph, GraphPathLimits::default())?
            .with_confidence(GraphPathConfidence::Proven)
            .execute(&graph, None, &AtomicBool::new(false))?
            .paths()
            .len(),
        1
    );
    let possible = query(&graph, GraphPathLimits::default())?
        .with_confidence(GraphPathConfidence::IncludePossible)
        .execute(&graph, None, &AtomicBool::new(false))?;
    assert_eq!(possible.paths().len(), 3);
    let candidate = q
        .with_confidence(GraphPathConfidence::IncludeCandidate)
        .execute(&graph, None, &AtomicBool::new(false))?;
    assert_eq!(candidate.paths().len(), 4);
    assert!(
        candidate
            .paths()
            .iter()
            .any(|path| path.confidence() == GraphConfidence::Candidate)
    );
    Ok(())
}

#[test]
fn incoming_and_both_traverse_original_edges_without_repeating_nodes() -> TestResult {
    let graph = fixture(false)?;
    for direction in [GraphDirection::Incoming, GraphDirection::Both] {
        let result = GraphPathQuery::new(
            graph.snapshot_id().clone(),
            id(&graph, "d")?,
            id(&graph, "a")?,
            direction,
            vec![GraphRelationKind::Calls],
            GraphPathLimits::default(),
        )?
        .execute(&graph, None, &AtomicBool::new(false))?;
        assert!(!result.paths().is_empty());
        for path in result.paths() {
            assert_eq!(path.nodes()[0].node_id(), &id(&graph, "d")?);
            let unique = path
                .nodes()
                .iter()
                .map(GraphNode::node_id)
                .collect::<BTreeSet<_>>();
            assert_eq!(unique.len(), path.nodes().len());
            assert!(path.edges().iter().all(|edge| graph.edges().contains(edge)));
        }
    }
    Ok(())
}

#[test]
fn pagination_replays_exact_query_without_hiding_previous_truncation() -> TestResult {
    let graph = fixture(false)?;
    let limits = GraphPathLimits {
        max_paths: 1,
        ..GraphPathLimits::default()
    };
    let q = query(&graph, limits)?;
    let first = q.execute(&graph, None, &AtomicBool::new(false))?;
    assert_eq!(first.state(), GraphQueryState::Truncated);
    assert!(first.truncations().contains(&GraphPathTruncation::Paths));
    let encoded = serde_json::to_vec(first.continuation().ok_or("page cursor")?)?;
    let cursor: GraphPathCursor = serde_json::from_slice(&encoded)?;
    let last = q.execute(&graph, Some(&cursor), &AtomicBool::new(false))?;
    assert_eq!(last.state(), GraphQueryState::Complete);
    assert!(last.prior_truncation());
    assert!(!last.absence_authoritative());
    assert!(last.continuation().is_none());
    let collected = first
        .paths()
        .iter()
        .chain(last.paths())
        .cloned()
        .collect::<Vec<_>>();
    let all = query(&graph, GraphPathLimits::default())?.execute(
        &graph,
        None,
        &AtomicBool::new(false),
    )?;
    assert_eq!(collected, all.paths());
    assert!(last.expansions() >= first.expansions());
    Ok(())
}

#[test]
fn changed_cursor_request_snapshot_or_deserialized_limits_fail_closed() -> TestResult {
    let graph = fixture(false)?;
    let q = query(
        &graph,
        GraphPathLimits {
            max_paths: 1,
            ..GraphPathLimits::default()
        },
    )?;
    let first = q.execute(&graph, None, &AtomicBool::new(false))?;
    let cursor = first.continuation().ok_or("page cursor")?;
    let mut value = serde_json::to_value(cursor)?;
    value["integrity_digest"] = json!("changed");
    let changed: GraphPathCursor = serde_json::from_value(value)?;
    assert_eq!(
        q.execute(&graph, Some(&changed), &AtomicBool::new(false))
            .err()
            .ok_or("changed cursor")?
            .code(),
        GraphErrorCode::QueryInvalid
    );
    assert!(
        query(&graph, GraphPathLimits::default())?
            .execute(&graph, Some(cursor), &AtomicBool::new(false))
            .is_err()
    );
    let different = GraphSnapshot::build(
        graph.universe().clone(),
        graph.generation().clone(),
        graph.limits(),
        graph.nodes().to_vec(),
        graph.edges().to_vec(),
        Vec::new(),
    )?;
    assert_eq!(
        q.execute(&different, None, &AtomicBool::new(false))
            .err()
            .ok_or("stale snapshot")?
            .code(),
        GraphErrorCode::SnapshotIdentityMismatch
    );
    for (pointer, replacement) in [
        ("/relations", json!([])),
        ("/limits/max_paths", json!(0)),
        ("/limits/max_expansions", json!(1_000_001)),
        ("/limits/max_depth", json!(65)),
        ("/limits/max_output_bytes", json!(u32::MAX)),
    ] {
        let mut value = serde_json::to_value(&q)?;
        *value.pointer_mut(pointer).ok_or("query field")? = replacement;
        let changed: GraphPathQuery = serde_json::from_value(value)?;
        assert!(
            changed
                .execute(&graph, None, &AtomicBool::new(false))
                .is_err(),
            "{pointer}"
        );
    }
    Ok(())
}

#[test]
fn depth_expansion_and_cancellation_never_authorize_empty_results() -> TestResult {
    let graph = fixture(false)?;
    for (limits, reason) in [
        (
            GraphPathLimits {
                max_depth: 1,
                ..GraphPathLimits::default()
            },
            GraphPathTruncation::Depth,
        ),
        (
            GraphPathLimits {
                max_expansions: 1,
                ..GraphPathLimits::default()
            },
            GraphPathTruncation::Expansions,
        ),
    ] {
        let result = query(&graph, limits)?.execute(&graph, None, &AtomicBool::new(false))?;
        assert_eq!(result.state(), GraphQueryState::Truncated);
        assert!(result.paths().is_empty());
        assert!(result.truncations().contains(&reason));
        assert!(!result.absence_authoritative());
        assert!(result.continuation().is_none());
        assert!(result.expansions() <= limits.max_expansions);
    }
    assert_eq!(
        query(&graph, GraphPathLimits::default())?
            .execute(&graph, None, &AtomicBool::new(true))
            .err()
            .ok_or("cancelled query")?
            .code(),
        GraphErrorCode::Cancelled
    );
    Ok(())
}

#[test]
fn absence_requires_complete_coverage_and_excludes_candidate_queries() -> TestResult {
    let graph = fixture(false)?;
    for (relation, state, authority) in [
        (GraphRelationKind::Owns, GraphQueryState::Complete, true),
        (GraphRelationKind::UsesApi, GraphQueryState::Partial, false),
        (
            GraphRelationKind::Loads,
            GraphQueryState::NotEvaluated,
            false,
        ),
    ] {
        let q = GraphPathQuery::new(
            graph.snapshot_id().clone(),
            id(&graph, "a")?,
            id(&graph, "z")?,
            GraphDirection::Outgoing,
            vec![relation],
            GraphPathLimits::default(),
        )?;
        let result = q.execute(&graph, None, &AtomicBool::new(false))?;
        assert!(result.paths().is_empty());
        assert_eq!(result.state(), state);
        assert_eq!(result.absence_authoritative(), authority);
        assert!(
            !q.with_confidence(GraphPathConfidence::IncludeCandidate)
                .execute(&graph, None, &AtomicBool::new(false))?
                .absence_authoritative()
        );
    }
    Ok(())
}

#[test]
fn output_budget_measures_evidence_and_rejects_nonprogressing_pages() -> TestResult {
    let a = node("a")?;
    let d = node("d")?;
    let evidence = (0..64)
        .map(|index| format!("evidence:{index}:{}", "x".repeat(480)).into())
        .collect();
    let edge = GraphEdge::new(
        a.node_id().clone(),
        d.node_id().clone(),
        GraphRelationKind::Calls,
        GraphConfidence::Proven,
        evidence,
        GraphLimits::default(),
    )?;
    let graph = GraphSnapshot::build(
        a.universe().clone(),
        a.generation().clone(),
        GraphLimits::default(),
        vec![a, d],
        vec![edge],
        Vec::new(),
    )?;
    assert_eq!(
        query(
            &graph,
            GraphPathLimits {
                max_output_bytes: 32_768,
                ..GraphPathLimits::default()
            }
        )?
        .execute(&graph, None, &AtomicBool::new(false))
        .err()
        .ok_or("oversized single path")?
        .code(),
        GraphErrorCode::BudgetExceeded
    );
    let result = query(
        &graph,
        GraphPathLimits {
            max_output_bytes: 65_536,
            ..GraphPathLimits::default()
        },
    )?
    .execute(&graph, None, &AtomicBool::new(false))?;
    assert_eq!(result.paths().len(), 1);
    assert!(canonical_json_bytes(&result)?.len() <= 65_536);
    Ok(())
}
