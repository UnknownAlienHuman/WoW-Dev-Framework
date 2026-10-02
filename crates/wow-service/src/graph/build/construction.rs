//! Glue for the W11 declarative CreateFrame producer partition.
use super::*;
use std::collections::BTreeMap;
use wow_recognizers::source_construction::{
    SOURCE_CONSTRUCTION_PARTITION, SourceConstructionInput, SourceConstructionRecognition,
    recognize_source_construction,
};

#[derive(Debug, Serialize)]
pub(super) struct FrameNode {
    call_id: String,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub(super) struct CreationEdge {
    call_id: String,
    edge_id: wow_graph::GraphEdgeId,
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceConstructionRecognition)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let result = recognize_source_construction(
        SourceConstructionInput {
            owner: source,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: provenance
                .functions()
                .iter()
                .map(|f| (f.function_id.as_str(), f.proposal_id.as_str()))
                .collect(),
            call_support: provenance
                .call_sites()
                .iter()
                .map(|c| (c.call_id.as_str(), (c.source_handle_id, c.evidence_id)))
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(|error| {
        super::error(match error.code() {
            wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                ServiceErrorCode::BudgetExceeded
            }
            _ => ServiceErrorCode::InternalContractViolation,
        })
    })?;
    checkpoint(stop)?;
    let candidate = source
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: source.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch: result.batch,
                coverage: result.coverage,
            },
            stop,
        )
        .map_err(graph_error)?;
    Ok((candidate.candidate().clone(), result.recognition))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    recognition: &SourceConstructionRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<FrameNode>, Vec<CreationEdge>)> {
    let partition = snapshot
        .partition(SOURCE_CONSTRUCTION_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted_entities = partition.report().accepted_entities();
    let accepted_relations = partition.report().accepted_relations();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let function_proposals = provenance
        .functions()
        .iter()
        .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();

    let mut frames = Vec::new();
    let mut edges = Vec::new();
    for receipt in recognition.create_frame_matches() {
        checkpoint(stop)?;
        let entity_index = accepted_entities
            .binary_search_by(|accepted| accepted.proposal_id().cmp(&receipt.entity_proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let entity = accepted_entities[entity_index].node();
        let relation_index = accepted_relations
            .binary_search_by(|accepted| accepted.proposal_id().cmp(&receipt.relation_proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let relation = accepted_relations[relation_index].edge();

        let call_index = report
            .calls()
            .binary_search_by(|call| call.fact_id().cmp(receipt.call_id.as_str()))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let call = &report.calls()[call_index];
        let caller_proposal = function_proposals
            .get(call.caller_function_id())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let caller = materialized_node_id(
            snapshot,
            caller_proposal,
            snapshot.snapshot().limits(),
        )?;
        let frame = materialized_node_id(
            snapshot,
            &receipt.entity_proposal_id,
            snapshot.snapshot().limits(),
        )?;
        if entity.kind() != "frame" || entity.node_id() == &caller {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        let final_edge = wow_graph::GraphEdge::new(
            caller,
            frame.clone(),
            relation.relation(),
            relation.confidence(),
            relation.evidence_ids().to_vec(),
            snapshot.snapshot().limits(),
        )
        .map_err(graph_error)?;
        if relation.relation() != wow_graph::GraphRelationKind::FactoryCreates
            || snapshot.snapshot().node(&frame).is_none()
            || snapshot.snapshot().edge(final_edge.edge_id()).is_none()
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        frames.push(FrameNode {
            call_id: receipt.call_id.clone(),
            node_id: frame,
        });
        edges.push(CreationEdge {
            call_id: receipt.call_id.clone(),
            edge_id: final_edge.edge_id().clone(),
        });
    }
    Ok((frames, edges))
}
