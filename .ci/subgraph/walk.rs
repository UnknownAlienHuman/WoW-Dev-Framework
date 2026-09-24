use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::atomic::AtomicBool;

use super::{
    GRAPH_SUBGRAPH_QUERY_SCHEMA, GraphSubgraphConfidence, GraphSubgraphNode,
    GraphSubgraphQuery, GraphSubgraphResult, GraphSubgraphTruncation, budget,
    checkpoint, encoded, invalid,
};
use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphDirection, GraphEdge, GraphNodeId,
    GraphQueryState, GraphResult, GraphSnapshot,
};

pub(super) fn execute(
    query: &GraphSubgraphQuery,
    snapshot: &GraphSnapshot,
    query_digest: Box<str>,
    cancelled: &AtomicBool,
) -> GraphResult<GraphSubgraphResult> {
    let mut coverage = Vec::new();
    let mut missing_coverage = Vec::new();
    for relation in &query.relations {
        match snapshot.coverage_for(*relation) {
            Some(record) => coverage.push(record.clone()),
            None => missing_coverage.push(*relation),
        }
    }
    let mut result = GraphSubgraphResult {
        schema: GRAPH_SUBGRAPH_QUERY_SCHEMA, query: query.clone(), query_digest,
        universe: snapshot.universe().clone(), generation: snapshot.generation().clone(),
        state: GraphQueryState::NotEvaluated, nodes: Vec::new(), edges: Vec::new(),
        coverage, missing_coverage, scanned_edges: 0, expansions: 0,
        truncations: Vec::new(), no_new_evidence: true, absence_authoritative: false,
    };
    // Exact item sizes plus a fixed allowance for counters/state/truncation tags.
    // Measure the complete final canonical result as well; never trim evidence.
    let mut bytes = encoded(&result)?.len() + 4096;
    let output_limit = query.limits.max_output_bytes as usize;
    let mut nodes = BTreeMap::new();
    let mut queue = VecDeque::new();
    for root in &query.roots {
        checkpoint(cancelled)?;
        let node = GraphSubgraphNode {
            node: snapshot.node(root).ok_or_else(|| invalid("subgraph root vanished"))?.clone(),
            depth: 0, discovery_edge: None,
        };
        bytes += encoded(&node)?.len() + 1;
        if bytes > output_limit {
            return Err(budget("subgraph output cannot hold all roots and coverage metadata"));
        }
        nodes.insert(root.clone(), node);
        queue.push_back((root.clone(), 0u32));
    }
    let mut adjacency = BTreeMap::<&GraphNodeId, Vec<&GraphEdge>>::new();
    for edge in snapshot.edges() {
        checkpoint(cancelled)?;
        result.scanned_edges += 1;
        if query.relations.binary_search(&edge.relation()).is_err()
            || !query.confidence.admits(edge.confidence())
        { continue; }
        // Edges are already in canonical ID order; direction never rewrites them.
        if query.direction != GraphDirection::Incoming {
            adjacency.entry(edge.from()).or_default().push(edge);
        }
        if query.direction != GraphDirection::Outgoing {
            adjacency.entry(edge.to()).or_default().push(edge);
        }
    }
    let mut edges = BTreeMap::new();
    let mut truncations = BTreeSet::new();
    'traversal: while let Some((current, depth)) = queue.pop_front() {
        checkpoint(cancelled)?;
        for edge in adjacency.get(&current).map(Vec::as_slice).unwrap_or_default() {
            checkpoint(cancelled)?;
            if result.expansions == query.limits.max_expansions {
                truncations.insert(GraphSubgraphTruncation::Expansions);
                break 'traversal;
            }
            result.expansions += 1;
            if edges.contains_key(edge.edge_id()) { continue; }
            let next = if edge.from() == &current { edge.to() } else { edge.from() };
            let new_node = !nodes.contains_key(next);
            if new_node && depth == query.limits.max_depth {
                truncations.insert(GraphSubgraphTruncation::Depth);
                continue;
            }
            if edges.len() == query.limits.max_edges as usize {
                truncations.insert(GraphSubgraphTruncation::Edges);
                break 'traversal;
            }
            if new_node && nodes.len() == query.limits.max_nodes as usize {
                truncations.insert(GraphSubgraphTruncation::Nodes);
                break 'traversal;
            }
            let node = if new_node {
                Some(GraphSubgraphNode {
                    node: snapshot.node(next).ok_or_else(|| invalid("subgraph endpoint vanished"))?.clone(),
                    depth: depth + 1, discovery_edge: Some(edge.edge_id().clone()),
                })
            } else { None };
            let node_bytes = node.as_ref().map(|node| encoded(node).map(|b| b.len() + 1)).transpose()?.unwrap_or(0);
            let item_bytes = encoded(edge)?.len() + 1 + node_bytes;
            if item_bytes > output_limit - bytes {
                truncations.insert(GraphSubgraphTruncation::OutputBytes);
                break 'traversal;
            }
            bytes += item_bytes;
            // Admit a discovery edge and its endpoint atomically. Every retained
            // non-root has a retained witness; no edge ever has a missing node.
            if let Some(node) = node {
                queue.push_back((next.clone(), node.depth));
                nodes.insert(next.clone(), node);
            }
            edges.insert(edge.edge_id().clone(), (*edge).clone());
        }
    }
    checkpoint(cancelled)?;
    result.nodes = nodes.into_values().collect();
    result.edges = edges.into_values().collect();
    result.truncations = truncations.into_iter().collect();
    result.state = if !result.truncations.is_empty() {
        GraphQueryState::Truncated
    } else if result.coverage.iter().any(|record| matches!(record.state(), GraphCoverageState::Partial | GraphCoverageState::Failed)) {
        GraphQueryState::Partial
    } else if !result.missing_coverage.is_empty() || result.coverage.iter().any(|record| record.state() == GraphCoverageState::NotEvaluated) {
        GraphQueryState::NotEvaluated
    } else { GraphQueryState::Complete };
    result.no_new_evidence = result.edges.is_empty();
    result.absence_authoritative = result.no_new_evidence
        && result.state == GraphQueryState::Complete
        && query.confidence != GraphSubgraphConfidence::IncludeCandidate
        && result.coverage.iter().all(GraphCoverageRecord::negative_authority);
    if encoded(&result)?.len() > output_limit {
        return Err(budget("final subgraph result exceeds the canonical output budget"));
    }
    checkpoint(cancelled)?;
    Ok(result)
}
