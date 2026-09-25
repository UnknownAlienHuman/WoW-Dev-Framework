//! State namespace/access composition, with maps bound only after all producers.
use super::*;
use std::collections::BTreeMap;
use wow_recognizers::source_state::{
    SOURCE_STATE_PARTITION, SourceStateFact, SourceStateInput, SourceStateRecognition,
    recognize_source_state,
};

#[derive(Debug, Serialize)]
pub(super) struct StateRootNode {
    root_id: String,
    name: String,
    scope: wow_project::load::TocSavedVariableScope,
    document: String,
    ambiguous: bool,
    node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct StatePathNode {
    path_id: String,
    root_id: String,
    keys: Vec<wow_project::graph::GlobalAccessKey>,
    node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct StateNodes {
    roots: Vec<StateRootNode>,
    paths: Vec<StatePathNode>,
}
impl StateNodes {
    pub(super) fn empty() -> Self {
        Self {
            roots: Vec::new(),
            paths: Vec::new(),
        }
    }
}
#[derive(Debug, Serialize)]
pub(super) struct StateEdge {
    binding_id: String,
    access_id: String,
    root_id: String,
    function_node_id: wow_graph::GraphNodeId,
    state_node_id: wow_graph::GraphNodeId,
    edge_id: wow_graph::GraphEdgeId,
    relation: wow_graph::GraphRelationKind,
    confidence: wow_graph::GraphConfidence,
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceStateRecognition)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let result = recognize_source_state(
        SourceStateInput {
            owner: source,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            context: provenance.context(),
            report,
            facts: provenance
                .state_bindings()
                .iter()
                .map(|b| SourceStateFact {
                    fact_id: &b.binding_id,
                    access_id: &b.access_id,
                    root_proposal_id: &b.root_id,
                    caller_proposal_id: &b.caller_proposal_id,
                    target_proposal_id: &b.target_proposal_id,
                    kind: b.kind,
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
    recognition: &SourceStateRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(StateNodes, Vec<StateEdge>)> {
    let limits = snapshot.snapshot().limits();
    let mut result = StateNodes::empty();
    let mut nodes = BTreeMap::new();
    for root in provenance.state_roots() {
        checkpoint(stop)?;
        let node_id = materialized_node_id(snapshot, &root.proposal_id, limits)?;
        nodes.insert(root.proposal_id.clone(), node_id.clone());
        result.roots.push(StateRootNode {
            root_id: root.root_id.clone(),
            name: root.name.clone(),
            scope: root.scope,
            document: root.document.clone(),
            ambiguous: root.ambiguous,
            node_id,
        });
    }
    for path in provenance.state_paths() {
        checkpoint(stop)?;
        let node_id = materialized_node_id(snapshot, &path.proposal_id, limits)?;
        nodes.insert(path.proposal_id.clone(), node_id.clone());
        result.paths.push(StatePathNode {
            path_id: path.path_id.clone(),
            root_id: path.root_id.clone(),
            keys: path.keys.clone(),
            node_id,
        });
    }
    let bindings = provenance
        .state_bindings()
        .iter()
        .map(|b| (b.binding_id.as_str(), b))
        .collect::<BTreeMap<_, _>>();
    let accepted = snapshot
        .partition(SOURCE_STATE_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?
        .report()
        .accepted_relations();
    let mut edges = Vec::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let binding = bindings
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        if receipt.access_id != binding.access_id {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        let index = accepted
            .binary_search_by(|e| e.proposal_id().cmp(&receipt.proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let original = accepted[index].edge();
        if !nodes.contains_key(&binding.caller_proposal_id) {
            nodes.insert(
                binding.caller_proposal_id.clone(),
                materialized_node_id(snapshot, &binding.caller_proposal_id, limits)?,
            );
        }
        let from = nodes
            .get(&binding.caller_proposal_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let to = nodes
            .get(&binding.target_proposal_id)
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
        edges.push(StateEdge {
            binding_id: binding.binding_id.clone(),
            access_id: binding.access_id.clone(),
            root_id: binding.root_id.clone(),
            function_node_id: from.clone(),
            state_node_id: to.clone(),
            edge_id: edge.edge_id().clone(),
            relation: edge.relation(),
            confidence: edge.confidence(),
        });
    }
    Ok((result, edges))
}
