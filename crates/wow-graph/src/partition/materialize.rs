use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphEdge, GraphGenerationId, GraphNode,
    GraphNodeId, GraphResult, GraphSnapshot,
};

use super::{GraphProducerPartition, check_cancelled, invalid};

pub(super) fn input_view(
    foundation: &GraphSnapshot,
    partitions: &[GraphProducerPartition],
    endpoints_only: bool,
    cancelled: &AtomicBool,
) -> GraphResult<GraphSnapshot> {
    let limits = foundation.limits();
    let mut nodes = BTreeMap::<GraphNodeId, GraphNode>::new();
    for node in foundation.nodes().iter().chain(partitions.iter().flat_map(|item| {
        item.report.accepted_entities().iter().map(|entry| entry.node())
    })) {
        check_cancelled(cancelled)?;
        if let Some(previous) = nodes.get(node.node_id()) {
            if previous.kind() != node.kind() || previous.owner_key() != node.owner_key()
                || previous.universe() != node.universe() || previous.generation() != node.generation()
            {
                return Err(invalid("incompatible assertions for the same graph node"));
            }
            let evidence = previous.evidence_ids().iter().chain(node.evidence_ids())
                .cloned().collect::<BTreeSet<_>>().into_iter().collect();
            let combined = GraphNode::new(
                node.universe().clone(), node.generation().clone(), node.kind(),
                node.owner_key(), evidence, limits,
            )?;
            nodes.insert(node.node_id().clone(), combined);
        } else {
            nodes.insert(node.node_id().clone(), node.clone());
        }
        if nodes.len() > limits.max_nodes as usize {
            return Err(invalid("materialized graph node budget exceeded"));
        }
    }
    let mut edges = BTreeMap::new();
    if !endpoints_only {
        for edge in foundation.edges().iter().chain(partitions.iter().flat_map(|item| {
            item.report.accepted_relations().iter().map(|entry| entry.edge())
        })) {
            check_cancelled(cancelled)?;
            if let Some(previous) = edges.insert(edge.edge_id().clone(), edge.clone())
                && previous != *edge
            {
                return Err(invalid("incompatible assertions for the same graph edge"));
            }
            if edges.len() > limits.max_edges as usize {
                return Err(invalid("materialized graph edge budget exceeded"));
            }
        }
    }
    let coverage = if endpoints_only {
        Vec::new()
    } else {
        aggregate_coverage(foundation, partitions, cancelled)?
    };
    GraphSnapshot::build(
        foundation.universe().clone(), foundation.generation().clone(), limits,
        nodes.into_values().collect(), edges.into_values().collect(), coverage,
    )
}

fn aggregate_coverage(
    foundation: &GraphSnapshot,
    partitions: &[GraphProducerPartition],
    cancelled: &AtomicBool,
) -> GraphResult<Vec<GraphCoverageRecord>> {
    let mut relations = BTreeSet::new();
    for record in foundation.coverage().iter().chain(partitions.iter().flat_map(|item| &item.coverage)) {
        relations.insert(record.relation());
    }
    for edge in foundation.edges().iter().chain(partitions.iter().flat_map(|item| {
        item.report.accepted_relations().iter().map(|entry| entry.edge())
    })) {
        relations.insert(edge.relation());
    }
    let mut coverage = Vec::new();
    for relation in relations {
        check_cancelled(cancelled)?;
        let mut state = GraphCoverageState::Complete;
        let mut blockers = BTreeSet::new();
        for records in std::iter::once(foundation.coverage()).chain(partitions.iter().map(|item| item.coverage.as_slice())) {
            if let Some(record) = records.iter().find(|item| item.relation() == relation) {
                state = state.max(record.state());
                blockers.extend(record.blocker_ids().iter().cloned());
            } else {
                state = state.max(GraphCoverageState::NotEvaluated);
                blockers.insert(Box::<str>::from("graph.partition.coverage_missing"));
            }
        }
        // This layer aggregates producer observations, not authoritative platform
        // negatives. A complete empty matcher batch does not change that boundary.
        coverage.push(GraphCoverageRecord::new(
            relation, state, false, blockers.into_iter().collect(), foundation.limits(),
        )?);
    }
    Ok(coverage)
}

pub(super) fn rebind(
    input: &GraphSnapshot,
    generation: GraphGenerationId,
    cancelled: &AtomicBool,
) -> GraphResult<GraphSnapshot> {
    let mut remap = BTreeMap::new();
    let mut nodes = Vec::with_capacity(input.nodes().len());
    for node in input.nodes() {
        check_cancelled(cancelled)?;
        let rebound = GraphNode::new(
            input.universe().clone(), generation.clone(), node.kind(), node.owner_key(),
            node.evidence_ids().to_vec(), input.limits(),
        )?;
        remap.insert(node.node_id().clone(), rebound.node_id().clone());
        nodes.push(rebound);
    }
    let mut edges = Vec::with_capacity(input.edges().len());
    for edge in input.edges() {
        check_cancelled(cancelled)?;
        edges.push(GraphEdge::new(
            remap.get(edge.from()).ok_or_else(|| invalid("missing source during graph rebind"))?.clone(),
            remap.get(edge.to()).ok_or_else(|| invalid("missing target during graph rebind"))?.clone(),
            edge.relation(), edge.confidence(), edge.evidence_ids().to_vec(), input.limits(),
        )?);
    }
    GraphSnapshot::build(
        input.universe().clone(), generation, input.limits(), nodes, edges, input.coverage().to_vec(),
    )
}
