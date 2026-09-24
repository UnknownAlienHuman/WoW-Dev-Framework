//! Glue for the source-function and recognizer producer partitions.
use super::*;
use std::collections::BTreeMap;
use wow_recognizers::source_calls::{
    SOURCE_CALL_PARTITION, SourceCallInput, SourceCallOutcome, SourceCallRecognition,
    recognize_source_calls,
};

#[derive(Debug, Serialize)]
pub(super) struct FunctionNode {
    function_id: String,
    path: String,
    span: wow_core::SourceSpan,
    kind: wow_emmy::function_calls::SourceFunctionKind,
    node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct CallEdge {
    call_id: String,
    edge_id: wow_graph::GraphEdgeId,
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceCallRecognition)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let result = recognize_source_calls(
        SourceCallInput {
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
    recognition: &SourceCallRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<FunctionNode>, Vec<CallEdge>)> {
    let mut functions = Vec::new();
    let mut node_ids = BTreeMap::new();
    for function in provenance.functions() {
        checkpoint(stop)?;
        let node_id = materialized_node_id(
            snapshot,
            &function.proposal_id,
            snapshot.snapshot().limits(),
        )?;
        node_ids.insert(function.function_id.as_str(), node_id.clone());
        functions.push(FunctionNode {
            function_id: function.function_id.clone(),
            path: function.path.clone(),
            span: function.span,
            kind: function.kind,
            node_id,
        });
    }
    let partition = snapshot
        .partition(SOURCE_CALL_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_relations();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let mut edges = Vec::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let SourceCallOutcome::Projected { proposal_id, .. } = &receipt.outcome else {
            continue;
        };
        let index = accepted
            .binary_search_by(|p| p.proposal_id().cmp(proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let edge = accepted[index].edge();
        let index = report
            .calls()
            .binary_search_by(|c| c.fact_id().cmp(&receipt.call_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let call = &report.calls()[index];
        let wow_emmy::function_calls::SourceCallTarget::MainFunction { function_id } =
            call.target()
        else {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        };
        let from = node_ids
            .get(call.caller_function_id())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let to = node_ids
            .get(function_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let final_edge = wow_graph::GraphEdge::new(
            from.clone(),
            to.clone(),
            edge.relation(),
            edge.confidence(),
            edge.evidence_ids().to_vec(),
            snapshot.snapshot().limits(),
        )
        .map_err(graph_error)?;
        if snapshot.snapshot().edge(final_edge.edge_id()).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        edges.push(CallEdge {
            call_id: receipt.call_id.clone(),
            edge_id: final_edge.edge_id().clone(),
        });
    }
    Ok((functions, edges))
}
