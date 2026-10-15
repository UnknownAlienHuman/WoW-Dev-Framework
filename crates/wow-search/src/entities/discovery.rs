//! Reconstruct only the graph owner's canonical BFS discovery witnesses.
use crate::{SearchError, SearchResult};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_graph::{GraphConfidence, GraphDirection, GraphEdge, GraphNodeId, GraphSubgraphResult};

#[derive(Serialize)]
pub(super) struct DiscoveryStep<'a> {
    pub traversal_from: &'a GraphNodeId,
    pub traversal_to: &'a GraphNodeId,
    pub traversal_direction: GraphDirection,
    pub original_edge: &'a GraphEdge,
}
#[derive(Serialize)]
pub(super) struct DiscoveryPath<'a> {
    pub seed: &'a GraphNodeId,
    pub target: &'a GraphNodeId,
    pub steps: Vec<DiscoveryStep<'a>>,
    pub weakest_confidence: GraphConfidence,
}
pub(super) fn discovery_paths<'a>(
    result: &'a GraphSubgraphResult,
    seeds: &[GraphNodeId],
    stop: &AtomicBool,
) -> SearchResult<Vec<DiscoveryPath<'a>>> {
    checkpoint(stop)?;
    let nodes = result
        .nodes()
        .iter()
        .map(|n| (n.node().node_id().as_str(), n))
        .collect::<BTreeMap<_, _>>();
    let edges = result
        .edges()
        .iter()
        .map(|e| (e.edge_id().as_str(), e))
        .collect::<BTreeMap<_, _>>();
    let seeds = seeds
        .iter()
        .map(GraphNodeId::as_str)
        .collect::<BTreeSet<_>>();
    if nodes.len() != result.nodes().len() || edges.len() != result.edges().len() {
        return Err(SearchError::OwnerEvidenceInvalid);
    }
    let mut paths = Vec::new();
    for original in nodes.values() {
        checkpoint(stop)?;
        if original.depth() == 0 {
            continue;
        }
        if original.depth() > 64 {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        let mut current = *original;
        let mut steps = Vec::new();
        let mut weakest = GraphConfidence::Proven;
        while current.depth() > 0 {
            checkpoint(stop)?;
            let edge = edges
                .get(
                    current
                        .discovery_edge()
                        .ok_or(SearchError::OwnerEvidenceInvalid)?
                        .as_str(),
                )
                .copied()
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            let (parent, direction) = if edge.to() == current.node().node_id() {
                (edge.from(), GraphDirection::Outgoing)
            } else if edge.from() == current.node().node_id() {
                (edge.to(), GraphDirection::Incoming)
            } else {
                return Err(SearchError::OwnerEvidenceInvalid);
            };
            let parent = nodes
                .get(parent.as_str())
                .copied()
                .ok_or(SearchError::OwnerEvidenceInvalid)?;
            if parent.depth().checked_add(1) != Some(current.depth()) {
                return Err(SearchError::OwnerEvidenceInvalid);
            }
            weakest = weakest.max(edge.confidence());
            steps.push(DiscoveryStep {
                traversal_from: parent.node().node_id(),
                traversal_to: current.node().node_id(),
                traversal_direction: direction,
                original_edge: edge,
            });
            current = parent;
        }
        if !seeds.contains(current.node().node_id().as_str())
            || steps.len() != original.depth() as usize
        {
            return Err(SearchError::OwnerEvidenceInvalid);
        }
        steps.reverse();
        paths.push(DiscoveryPath {
            seed: current.node().node_id(),
            target: original.node().node_id(),
            steps,
            weakest_confidence: weakest,
        });
    }
    Ok(paths)
}
fn checkpoint(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_graph::{
        GraphGenerationId, GraphLimits, GraphNode, GraphQueryState, GraphRelationKind,
        GraphSnapshot, GraphSubgraphConfidence, GraphSubgraphLimits, GraphSubgraphQuery,
        GraphUniverseId,
    };
    #[test]
    fn owner_cycle_possible_multihop_and_inverse_witnesses_remain_original()
    -> Result<(), Box<dyn std::error::Error>> {
        let universe = GraphUniverseId::new("project:search-discovery")?;
        let generation = GraphGenerationId::new("generation:search-discovery:1")?;
        let limits = GraphLimits::default();
        let mut nodes = Vec::new();
        for key in ["A", "B", "C", "D"] {
            nodes.push(GraphNode::new(
                universe.clone(),
                generation.clone(),
                "function",
                key,
                vec![format!("evidence:{key}").into()],
                limits,
            )?);
        }
        let a = nodes[0].node_id().clone();
        let b = nodes[1].node_id().clone();
        let c = nodes[2].node_id().clone();
        let d = nodes[3].node_id().clone();
        let ab = GraphEdge::new(
            a.clone(),
            b.clone(),
            GraphRelationKind::Calls,
            GraphConfidence::Proven,
            vec!["evidence:AB".into()],
            limits,
        )?;
        let bc = GraphEdge::new(
            b.clone(),
            c.clone(),
            GraphRelationKind::Calls,
            GraphConfidence::Possible,
            vec!["evidence:BC".into()],
            limits,
        )?;
        let ca = GraphEdge::new(
            c.clone(),
            a.clone(),
            GraphRelationKind::Calls,
            GraphConfidence::Derived,
            vec!["evidence:CA".into()],
            limits,
        )?;
        let dc = GraphEdge::new(
            d,
            c.clone(),
            GraphRelationKind::Calls,
            GraphConfidence::Derived,
            vec!["evidence:DC".into()],
            limits,
        )?;
        let snapshot = GraphSnapshot::build(
            universe,
            generation,
            limits,
            nodes,
            vec![ab.clone(), bc.clone(), ca, dc],
            Vec::new(),
        )?;
        let stop = AtomicBool::new(false);
        let query_limits = GraphSubgraphLimits {
            max_depth: 4,
            max_nodes: 16,
            max_edges: 16,
            max_scanned_edges: 16,
            max_expansions: 64,
            max_output_bytes: 65536,
        };
        let output = GraphSubgraphQuery::new(
            snapshot.snapshot_id().clone(),
            vec![a.clone()],
            GraphDirection::Outgoing,
            vec![GraphRelationKind::Calls],
            query_limits,
        )?
        .with_confidence(GraphSubgraphConfidence::IncludePossible)
        .execute(&snapshot, &stop)?;
        let paths = discovery_paths(&output, std::slice::from_ref(&a), &stop)?;
        let path = paths
            .iter()
            .find(|p| p.target == &c)
            .ok_or("C discovery missing")?;
        assert_eq!(path.steps.len(), 2);
        assert_eq!(path.seed, &a);
        assert_eq!(path.weakest_confidence, GraphConfidence::Possible);
        assert_eq!(path.steps[0].original_edge, &ab);
        assert_eq!(path.steps[1].original_edge, &bc);
        assert!(paths.iter().all(|p| p.target != &a));
        let inverse = GraphSubgraphQuery::new(
            snapshot.snapshot_id().clone(),
            vec![c.clone()],
            GraphDirection::Incoming,
            vec![GraphRelationKind::Calls],
            query_limits,
        )?
        .with_confidence(GraphSubgraphConfidence::IncludePossible)
        .execute(&snapshot, &stop)?;
        for path in discovery_paths(&inverse, std::slice::from_ref(&c), &stop)? {
            for step in path.steps {
                assert_eq!(step.traversal_direction, GraphDirection::Incoming);
                assert_eq!(step.traversal_from, step.original_edge.to());
                assert_eq!(step.traversal_to, step.original_edge.from());
            }
        }
        let tight = GraphSubgraphLimits {
            max_nodes: 1,
            ..query_limits
        };
        let limited = GraphSubgraphQuery::new(
            snapshot.snapshot_id().clone(),
            vec![a.clone()],
            GraphDirection::Outgoing,
            vec![GraphRelationKind::Calls],
            tight,
        )?
        .with_confidence(GraphSubgraphConfidence::IncludePossible)
        .execute(&snapshot, &stop)?;
        assert_eq!(limited.state(), GraphQueryState::Truncated);
        assert!(discovery_paths(&limited, &[a], &stop)?.is_empty());
        assert!(matches!(
            discovery_paths(&output, &[], &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        Ok(())
    }
}
