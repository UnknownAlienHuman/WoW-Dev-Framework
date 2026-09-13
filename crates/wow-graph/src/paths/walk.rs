use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use super::{
    GRAPH_PATH_QUERY_SCHEMA, GraphPath, GraphPathConfidence, GraphPathCursor, GraphPathQuery,
    GraphPathResult, GraphPathTruncation, budget, check_cancelled, encoded, invalid,
};
use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphDirection, GraphEdge, GraphNodeId,
    GraphQueryState, GraphResult, GraphSnapshot,
};

pub(super) fn next_node<'a>(
    edge: &'a GraphEdge,
    current: &GraphNodeId,
    direction: GraphDirection,
) -> Option<&'a GraphNodeId> {
    if direction != GraphDirection::Incoming && edge.from() == current {
        Some(edge.to())
    } else if direction != GraphDirection::Outgoing && edge.to() == current {
        Some(edge.from())
    } else {
        None
    }
}

pub(super) fn execute(
    query: &GraphPathQuery,
    snapshot: &GraphSnapshot,
    cursor: Option<&GraphPathCursor>,
    query_digest: Box<str>,
    cancelled: &AtomicBool,
) -> GraphResult<GraphPathResult> {
    let coverage = query
        .relations
        .iter()
        .filter_map(|relation| snapshot.coverage_for(*relation).cloned())
        .collect::<Vec<_>>();
    // Reserve bounded metadata/cursor space in addition to exact path bytes.
    // The final entire canonical result is measured again before returning.
    let reserve = 16_384 + encoded(&coverage)?.len() + encoded(query)?.len();
    if reserve > query.limits.max_output_bytes as usize {
        return Err(budget(
            "path output budget cannot hold coverage and cursor metadata",
        ));
    }
    let mut adjacency = BTreeMap::<&GraphNodeId, Vec<&GraphEdge>>::new();
    for edge in snapshot.edges() {
        check_cancelled(cancelled)?;
        if !query.confidence.admits(edge.confidence())
            || query.relations.binary_search(&edge.relation()).is_err()
        {
            continue;
        }
        // Snapshot edges are already sorted by exact edge ID. No physical-row ordering.
        if query.direction != GraphDirection::Incoming {
            adjacency.entry(edge.from()).or_default().push(edge);
        }
        if query.direction != GraphDirection::Outgoing {
            adjacency.entry(edge.to()).or_default().push(edge);
        }
    }
    let mut nodes = vec![query.root.clone()];
    let mut edges = Vec::<&GraphEdge>::new();
    let mut positions = vec![0usize];
    let mut visited = BTreeSet::from([query.root.clone()]);
    let mut paths = Vec::<GraphPath>::new();
    let mut truncations = BTreeSet::new();
    let mut expansions = 0u32;
    let mut path_bytes = reserve;
    let mut after_reached = cursor.is_none();
    while let Some(current) = nodes.last() {
        check_cancelled(cancelled)?;
        let incident = adjacency.get(current).map(Vec::as_slice).unwrap_or(&[]);
        let position = positions
            .last_mut()
            .ok_or_else(|| invalid("path stack lost its cursor"))?;
        if *position == incident.len() {
            if let Some(node) = nodes.pop() {
                visited.remove(&node);
            }
            positions.pop();
            edges.pop();
            continue;
        }
        if expansions == query.limits.max_expansions {
            truncations.insert(GraphPathTruncation::Expansions);
            break;
        }
        let edge = incident[*position];
        *position += 1;
        expansions += 1;
        let next = next_node(edge, current, query.direction)
            .ok_or_else(|| invalid("path adjacency changed direction"))?;
        if visited.contains(next) {
            continue;
        }
        if edges.len() == query.limits.max_depth as usize {
            truncations.insert(GraphPathTruncation::Depth);
            continue;
        }
        nodes.push(next.clone());
        edges.push(edge);
        if next != &query.target {
            visited.insert(next.clone());
            positions.push(0);
            continue;
        }
        let key = edges
            .iter()
            .map(|edge| edge.edge_id().clone())
            .collect::<Vec<_>>();
        if !after_reached {
            after_reached = cursor.is_some_and(|cursor| cursor.after == key);
        } else {
            if paths.len() == query.limits.max_paths as usize {
                truncations.insert(GraphPathTruncation::Paths);
                break;
            }
            let path = GraphPath {
                nodes: nodes
                    .iter()
                    .map(|node| {
                        snapshot
                            .node(node)
                            .cloned()
                            .ok_or_else(|| invalid("path node vanished from the snapshot"))
                    })
                    .collect::<GraphResult<Vec<_>>>()?,
                edges: edges.iter().map(|edge| (*edge).clone()).collect(),
                confidence: edges
                    .iter()
                    .map(|edge| edge.confidence())
                    .max()
                    .ok_or_else(|| invalid("empty path is outside this profile"))?,
            };
            check_cancelled(cancelled)?;
            let size = encoded(&path)?.len() + 1;
            if size > query.limits.max_output_bytes as usize - path_bytes {
                if paths.is_empty() {
                    return Err(budget("one path cannot fit the output budget"));
                }
                truncations.insert(GraphPathTruncation::OutputBytes);
                break;
            }
            path_bytes += size;
            paths.push(path);
        }
        nodes.pop();
        edges.pop();
    }
    if !after_reached {
        return Err(budget(
            "path cursor boundary was not reached within the expansion budget",
        ));
    }
    let state = if !truncations.is_empty() {
        GraphQueryState::Truncated
    } else if coverage.iter().any(|record| {
        matches!(
            record.state(),
            GraphCoverageState::Partial | GraphCoverageState::Failed
        )
    }) {
        GraphQueryState::Partial
    } else if coverage.len() != query.relations.len()
        || coverage
            .iter()
            .any(|record| record.state() == GraphCoverageState::NotEvaluated)
    {
        GraphQueryState::NotEvaluated
    } else {
        GraphQueryState::Complete
    };
    let absence_authoritative = paths.is_empty()
        && cursor.is_none()
        && state == GraphQueryState::Complete
        && query.confidence != GraphPathConfidence::IncludeCandidate
        && coverage.iter().all(GraphCoverageRecord::negative_authority);
    let continuation = if truncations.contains(&GraphPathTruncation::Paths)
        || truncations.contains(&GraphPathTruncation::OutputBytes)
    {
        paths
            .last()
            .map(|path| GraphPathCursor::new(&query_digest, path.key()))
            .transpose()?
    } else {
        None
    };
    let result = GraphPathResult {
        schema: GRAPH_PATH_QUERY_SCHEMA.into(),
        snapshot_id: snapshot.snapshot_id().clone(),
        query_digest,
        state,
        paths,
        coverage,
        expansions,
        prior_truncation: cursor.is_some(),
        absence_authoritative,
        truncations: truncations.into_iter().collect(),
        continuation,
    };
    check_cancelled(cancelled)?;
    if encoded(&result)?.len() > query.limits.max_output_bytes as usize {
        return Err(budget("complete path result exceeds the output budget"));
    }
    check_cancelled(cancelled)?;
    Ok(result)
}
