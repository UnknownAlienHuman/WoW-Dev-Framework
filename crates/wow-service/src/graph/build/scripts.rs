//! Compose the XML script partition and resolve final publication identities.
use super::source_addresses::SourceGraphAddressCrosswalk;
use super::*;
use std::collections::BTreeMap;
use wow_graph::{GraphAssertionKind, GraphRelationKind};
use wow_recognizers::source_scripts::{
    SOURCE_SCRIPT_PARTITION, SourceScriptAssertionEndpoints, SourceScriptAssertionFact,
    SourceScriptAssertionInput, SourceScriptAssertionRecognition, SourceScriptFact,
    SourceScriptInput, SourceScriptRecognition, SourceScriptSemanticContext,
    recognize_source_script_assertions, recognize_source_scripts,
};

#[derive(Debug)]
pub(super) enum ScriptRecognition {
    Legacy(SourceScriptRecognition),
    Assertions(SourceScriptAssertionRecognition),
}

impl ScriptRecognition {
    pub(super) fn into_parts(
        self,
    ) -> (
        Option<SourceScriptRecognition>,
        Option<SourceScriptAssertionRecognition>,
    ) {
        match self {
            Self::Legacy(recognition) => (Some(recognition), None),
            Self::Assertions(recognition) => (None, Some(recognition)),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct HandlerNode {
    script_id: String,
    unit_id: String,
    document: String,
    semantic_context_id: String,
    script_site: String,
    implicit_receiver: String,
    runtime_dispatch: String,
    node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct ScriptEdge {
    binding_id: String,
    site_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_context_id: Option<String>,
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
                    semantic_context: b.semantic_context.as_ref().map(|context| {
                        SourceScriptSemanticContext {
                            context_id: context.context_id(),
                            script_site: context.script_site(),
                            implicit_receiver: context.implicit_receiver(),
                            runtime_dispatch: context.runtime_dispatch(),
                        }
                    }),
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
            semantic_context_id: handler.semantic_context.context_id().to_owned(),
            script_site: handler.semantic_context.script_site().to_owned(),
            implicit_receiver: handler.semantic_context.implicit_receiver().to_owned(),
            runtime_dispatch: handler.semantic_context.runtime_dispatch().to_owned(),
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
            semantic_context_id: binding
                .semantic_context
                .as_ref()
                .map(|context| context.context_id().to_owned()),
            receiver_node_id: from.clone(),
            handler_node_id: to.clone(),
            edge_id: edge.edge_id().clone(),
            confidence: edge.confidence(),
        });
    }
    Ok((handlers, edges))
}

pub(super) fn publish_bound(
    source: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, ScriptRecognition)> {
    if crosswalk.direct_provenance().is_none() {
        let (snapshot, recognition) = publish(source, crosswalk.source(), stop)?;
        return Ok((snapshot, ScriptRecognition::Legacy(recognition)));
    }
    checkpoint(stop)?;
    let provenance = crosswalk.source();
    if provenance.script_bindings().len() > 8192 {
        return Err(error(ServiceErrorCode::BudgetExceeded));
    }
    let mut facts = Vec::new();
    for binding in provenance.script_bindings() {
        checkpoint(stop)?;
        facts.push(SourceScriptAssertionFact {
            fact: SourceScriptFact {
                fact_id: &binding.binding_id,
                receiver_proposal_id: &binding.receiver_proposal_id,
                handler_proposal_id: &binding.handler_proposal_id,
                semantic_context: binding.semantic_context.as_ref().map(|context| {
                    SourceScriptSemanticContext {
                        context_id: context.context_id(),
                        script_site: context.script_site(),
                        implicit_receiver: context.implicit_receiver(),
                        runtime_dispatch: context.runtime_dispatch(),
                    }
                }),
                confidence: binding.confidence,
                source_handle_ids: &binding.source_handle_ids,
                evidence_ids: &binding.evidence_ids,
            },
            endpoints: SourceScriptAssertionEndpoints {
                receiver: crosswalk.copied_assertion(
                    GraphAssertionKind::Entity,
                    &binding.receiver_proposal_id,
                    stop,
                )?,
                handler: crosswalk.copied_assertion(
                    GraphAssertionKind::Entity,
                    &binding.handler_proposal_id,
                    stop,
                )?,
            },
        });
    }
    let result = recognize_source_script_assertions(
        SourceScriptAssertionInput {
            owner: source,
            scope: crosswalk.scope(),
            context: provenance.context(),
            facts,
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(|failure| {
        error(match failure.code() {
            wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
            wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                ServiceErrorCode::BudgetExceeded
            }
            _ => ServiceErrorCode::InternalContractViolation,
        })
    })?;
    checkpoint(stop)?;
    crosswalk.reserve("script_assertion_recognition", &result.recognition, stop)?;
    let prepared = source
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: source.snapshot().snapshot_id().clone(),
                expected_partition_digest: source
                    .partition(SOURCE_SCRIPT_PARTITION)
                    .map(|partition| partition.partition_digest().into()),
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch: result.batch,
                coverage: result.coverage,
            },
            stop,
        )
        .map_err(graph_error)?;
    let snapshot = prepared.candidate().clone();
    checkpoint(stop)?;
    Ok((snapshot, ScriptRecognition::Assertions(result.recognition)))
}

pub(super) fn maps_bound(
    snapshot: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    recognition: &ScriptRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(Vec<HandlerNode>, Vec<ScriptEdge>)> {
    let recognition = match (crosswalk.direct_provenance(), recognition) {
        (None, ScriptRecognition::Legacy(recognition)) => {
            return maps(snapshot, crosswalk.source(), recognition, stop);
        }
        (Some(_), ScriptRecognition::Assertions(recognition)) => recognition,
        _ => return Err(error(ServiceErrorCode::IdentityMismatch)),
    };
    let lookup = snapshot.producer_lookup(stop).map_err(graph_error)?;
    if recognition.scope() != crosswalk.scope() || recognition.scope() != lookup.scope() {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let provenance = crosswalk.source();
    let mut handlers = Vec::new();
    for handler in provenance.inline_handlers() {
        checkpoint(stop)?;
        crosswalk.append(
            "handler_nodes",
            &mut handlers,
            HandlerNode {
                script_id: handler.script_id.clone(),
                unit_id: handler.unit_id.clone(),
                document: handler.document.clone(),
                semantic_context_id: handler.semantic_context.context_id().to_owned(),
                script_site: handler.semantic_context.script_site().to_owned(),
                implicit_receiver: handler.semantic_context.implicit_receiver().to_owned(),
                runtime_dispatch: handler.semantic_context.runtime_dispatch().to_owned(),
                node_id: crosswalk.node_id(snapshot, &lookup, &handler.proposal_id, stop)?,
            },
            stop,
        )?;
    }
    let mut bindings = BTreeMap::new();
    for binding in provenance.script_bindings() {
        checkpoint(stop)?;
        if bindings
            .insert(binding.binding_id.as_str(), binding)
            .is_some()
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
    }
    if recognition.endpoints().len() != bindings.len()
        || recognition.receipts().len() != bindings.len()
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let accepted = snapshot
        .partition(SOURCE_SCRIPT_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
        .report()
        .accepted_relations();
    let mut edges = Vec::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let binding = bindings
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let endpoints = recognition
            .endpoints()
            .get(&receipt.binding_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        if endpoints.receiver
            != crosswalk.assertion(
                GraphAssertionKind::Entity,
                &binding.receiver_proposal_id,
                stop,
            )?
            || endpoints.handler
                != crosswalk.assertion(
                    GraphAssertionKind::Entity,
                    &binding.handler_proposal_id,
                    stop,
                )?
        {
            return Err(error(ServiceErrorCode::IdentityMismatch));
        }
        let receiver = lookup
            .entity(recognition.scope(), &endpoints.receiver, stop)
            .map_err(graph_error)?;
        let handler = lookup
            .entity(recognition.scope(), &endpoints.handler, stop)
            .map_err(graph_error)?;
        let index = accepted
            .binary_search_by(|entry| entry.proposal_id().cmp(&receipt.proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let original = accepted[index].edge();
        if receiver.proposal().proposal_id() != binding.receiver_proposal_id
            || handler.proposal().proposal_id() != binding.handler_proposal_id
            || original.from() != receiver.accepted().node().node_id()
            || original.to() != handler.accepted().node().node_id()
            || original.relation() != GraphRelationKind::SetsScript
            || original.confidence() != binding.confidence
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        let from = crosswalk.node_id(snapshot, &lookup, &binding.receiver_proposal_id, stop)?;
        let to = crosswalk.node_id(snapshot, &lookup, &binding.handler_proposal_id, stop)?;
        let nodes = BTreeMap::from([
            (receiver.accepted().node().node_id().clone(), from.clone()),
            (handler.accepted().node().node_id().clone(), to.clone()),
        ]);
        let edge_id = super::materialized::edge_id(
            snapshot,
            SOURCE_SCRIPT_PARTITION,
            &receipt.proposal_id,
            &nodes,
        )?;
        let edge = snapshot
            .snapshot()
            .edge(&edge_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        if edge.from() != &from || edge.to() != &to {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        crosswalk.append(
            "script_edges",
            &mut edges,
            ScriptEdge {
                binding_id: binding.binding_id.clone(),
                site_id: binding.site_id.clone(),
                semantic_context_id: binding
                    .semantic_context
                    .as_ref()
                    .map(|context| context.context_id().to_owned()),
                receiver_node_id: from,
                handler_node_id: to,
                edge_id,
                confidence: edge.confidence(),
            },
            stop,
        )?;
    }
    checkpoint(stop)?;
    Ok((handlers, edges))
}
