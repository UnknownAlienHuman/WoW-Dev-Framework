//! Glue for the source-function and recognizer producer partitions.
use super::native_api::NativeApiProvenance;
use super::source_addresses::SourceGraphAddressCrosswalk;
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
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    api: Option<&NativeApiProvenance>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceCallRecognition)> {
    let provenance = crosswalk.source();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    checkpoint(stop)?;
    let api_targets = api.map(|api| api.api_bindings(stop)).transpose()?;
    checkpoint(stop)?;
    let result = recognize_source_calls(
        SourceCallInput {
            owner: source,
            source_partition: crosswalk.analyzer_partition(source, stop)?,
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
            api_targets: api_targets.as_ref(),
        },
        stop,
    )
    .map_err(|e| {
        let code = match e.code() {
            wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                ServiceErrorCode::BudgetExceeded
            }
            _ => ServiceErrorCode::InternalContractViolation,
        };
        ServiceError::new(
            code,
            format!(
                "source call recognizer rejected ({:?}): {}",
                e.code(),
                e.message()
            ),
        )
    })?;
    crosswalk.reserve("call_recognition", &result.recognition, stop)?;
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
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    recognition: &SourceCallRecognition,
    api: Option<&NativeApiProvenance>,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<FunctionNode>, Vec<CallEdge>)> {
    let provenance = crosswalk.source();
    let lookup = snapshot.producer_lookup(stop).map_err(graph_error)?;
    let mut functions = Vec::new();
    let mut node_ids = BTreeMap::new();
    for function in provenance.functions() {
        checkpoint(stop)?;
        let node_id = crosswalk.node_id(snapshot, &lookup, &function.proposal_id, stop)?;
        node_ids.insert(function.function_id.as_str(), node_id.clone());
        crosswalk.append(
            "function_nodes",
            &mut functions,
            FunctionNode {
                function_id: function.function_id.clone(),
                path: function.path.clone(),
                span: function.span,
                kind: function.kind,
                node_id,
            },
            stop,
        )?;
    }
    let partition = snapshot
        .partition(SOURCE_CALL_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_relations();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    // Resolve each exact Library tuple once against the accepted API producer.
    // None keeps the ordinary Calls path free of API/materialization maps.
    checkpoint(stop)?;
    let api_targets = api.map(|api| api.api_bindings(stop)).transpose()?;
    checkpoint(stop)?;
    let materialized_nodes = if api_targets.is_some() {
        Some(materialized::nodes(snapshot, stop)?)
    } else {
        None
    };
    let mut api_nodes = BTreeMap::new();
    if let Some(targets) = api_targets {
        let nodes = materialized_nodes
            .as_ref()
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        for (target, binding) in targets {
            checkpoint(stop)?;
            let resolved = lookup
                .entity(lookup.scope(), &binding.assertion, stop)
                .map_err(graph_error)?;
            let proposal = resolved.proposal();
            if proposal.entity_kind_id() != wow_recognizers::source_calls::SOURCE_API_ENTITY_KIND
                || proposal.confidence() != wow_graph::GraphConfidence::Derived
                || proposal.source_handle_ids() != [binding.source_handle.handle_id()]
                || proposal.evidence_ids() != [binding.evidence.evidence_id()]
            {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            let input_id = resolved.accepted().node().node_id();
            let final_id = nodes
                .get(input_id)
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
            api_nodes.insert(target, (input_id.clone(), final_id.clone()));
        }
    }
    let mut edges = Vec::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let (proposal_id, api_use) = match &receipt.outcome {
            SourceCallOutcome::Projected { proposal_id, .. } => (proposal_id, false),
            SourceCallOutcome::ApiUseProjected { proposal_id, .. } => (proposal_id, true),
            _ => continue,
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
        let from = node_ids
            .get(call.caller_function_id())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let to = if api_use {
            let wow_emmy::function_calls::SourceCallTarget::LibraryFunction { target } =
                call.target()
            else {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            };
            let (input_id, final_id) = api_nodes
                .get(target)
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
            let nodes = materialized_nodes
                .as_ref()
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
            if edge.relation() != wow_graph::GraphRelationKind::UsesApi
                || edge.confidence() != wow_graph::GraphConfidence::Derived
                || edge.to() != input_id
                || nodes.get(edge.from()) != Some(from)
            {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            final_id
        } else {
            let wow_emmy::function_calls::SourceCallTarget::MainFunction { function_id } =
                call.target()
            else {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            };
            node_ids
                .get(function_id.as_str())
                .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
        };
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
        crosswalk.append(
            "call_edges",
            &mut edges,
            CallEdge {
                call_id: receipt.call_id.clone(),
                edge_id: final_edge.edge_id().clone(),
            },
            stop,
        )?;
    }
    Ok((functions, edges))
}
