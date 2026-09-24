//! Compose the XML script partition and resolve final publication identities.
use super::*;
use std::collections::BTreeMap;
use wow_recognizers::source_scripts::{
    SOURCE_SCRIPT_PARTITION, SourceScriptFact, SourceScriptInput, SourceScriptRecognition,
    recognize_source_scripts,
};

#[derive(Debug, Serialize)]
pub(super) struct HandlerNode {
    script_id: String,
    unit_id: String,
    document: String,
    node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct ScriptEdge {
    binding_id: String,
    site_id: String,
    receiver_node_id: wow_graph::GraphNodeId,
    handler_node_id: wow_graph::GraphNodeId,
    edge_id: wow_graph::GraphEdgeId,
    confidence: wow_graph::GraphConfidence,
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceScriptRecognition)> {
    let result = recognize_source_scripts(
        SourceScriptInput {
            owner: source,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            context: provenance.context(),
            facts: provenance
                .script_bindings()
                .iter()
                .map(|b| SourceScriptFact {
                    fact_id: &b.binding_id,
                    receiver_proposal_id: &b.receiver_proposal_id,
                    handler_proposal_id: &b.handler_proposal_id,
                    confidence: b.confidence,
                    source_handle_ids: &b.source_handle_ids,
                    evidence_ids: &b.evidence_ids,
                })
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(|e| {
        error(match e.code() {
            wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                ServiceErrorCode::BudgetExceeded
            }
            _ => ServiceErrorCode::InternalContractViolation,
        })
    })?;
    checkpoint(stop)?;
    let prepared = source
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
    Ok((prepared.candidate().clone(), result.recognition))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    recognition: &SourceScriptRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<HandlerNode>, Vec<ScriptEdge>)> {
    let limits = snapshot.snapshot().limits();
    let mut handlers = Vec::new();
    for handler in provenance.inline_handlers() {
        checkpoint(stop)?;
        handlers.push(HandlerNode {
            script_id: handler.script_id.clone(),
            unit_id: handler.unit_id.clone(),
            document: handler.document.clone(),
            node_id: materialized_node_id(snapshot, &handler.proposal_id, limits)?,
        });
    }
    let bindings = provenance
        .script_bindings()
        .iter()
        .map(|b| (b.binding_id.as_str(), b))
        .collect::<BTreeMap<_, _>>();
    let accepted = snapshot
        .partition(SOURCE_SCRIPT_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
        .report()
        .accepted_relations();
    let mut edges = Vec::new();
    let mut nodes = BTreeMap::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let binding = bindings
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let index = accepted
            .binary_search_by(|e| e.proposal_id().cmp(&receipt.proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let original = accepted[index].edge();
        for proposal in [&binding.receiver_proposal_id, &binding.handler_proposal_id] {
            if !nodes.contains_key(proposal) {
                nodes.insert(
                    proposal.clone(),
                    materialized_node_id(snapshot, proposal, limits)?,
                );
            }
        }
        let from = nodes
            .get(&binding.receiver_proposal_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let to = nodes
            .get(&binding.handler_proposal_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let edge = wow_graph::GraphEdge::new(
            from.clone(),
            to.clone(),
            original.relation(),
            original.confidence(),
            original.evidence_ids().to_vec(),
            limits,
        )
        .map_err(graph_error)?;
        if snapshot.snapshot().edge(edge.edge_id()).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        edges.push(ScriptEdge {
            binding_id: binding.binding_id.clone(),
            site_id: binding.site_id.clone(),
            receiver_node_id: from.clone(),
            handler_node_id: to.clone(),
            edge_id: edge.edge_id().clone(),
            confidence: edge.confidence(),
        });
    }
    Ok((handlers, edges))
}
