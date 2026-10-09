//! Rebind accepted partition identities to the final materialized generation.
use super::*;
use std::collections::BTreeMap;
use wow_graph::{GraphEdge, GraphEdgeId, GraphNode, GraphNodeId};

pub(super) fn nodes(
    snapshot: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ServiceResult<BTreeMap<GraphNodeId, GraphNodeId>> {
    let input = snapshot.input_view(stop).map_err(graph_error)?;
    let mut nodes = BTreeMap::new();
    for original in input.nodes() {
        checkpoint(stop)?;
        let rebound = GraphNode::new(
            snapshot.snapshot().universe().clone(),
            snapshot.snapshot().generation().clone(),
            original.kind(),
            original.owner_key(),
            original.evidence_ids().to_vec(),
            snapshot.snapshot().limits(),
        )
        .map_err(graph_error)?;
        if snapshot.snapshot().node(rebound.node_id()).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        nodes.insert(original.node_id().clone(), rebound.node_id().clone());
    }
    Ok(nodes)
}

pub(super) fn edge_id(
    snapshot: &GraphPartitionSnapshot,
    partition_id: &str,
    proposal_id: &str,
    nodes: &BTreeMap<GraphNodeId, GraphNodeId>,
) -> ServiceResult<GraphEdgeId> {
    let partition = snapshot
        .partition(partition_id)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_relations();
    let index = accepted
        .binary_search_by(|entry| entry.proposal_id().cmp(proposal_id))
        .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
    let original = accepted[index].edge();
    let edge = GraphEdge::new(
        nodes
            .get(original.from())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
            .clone(),
        nodes
            .get(original.to())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
            .clone(),
        original.relation(),
        original.confidence(),
        original.evidence_ids().to_vec(),
        snapshot.snapshot().limits(),
    )
    .map_err(graph_error)?;
    if snapshot.snapshot().edge(edge.edge_id()).is_none() {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    Ok(edge.edge_id().clone())
}
