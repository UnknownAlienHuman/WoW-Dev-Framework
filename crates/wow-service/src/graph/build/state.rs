//! State namespace/access composition, with maps bound only after all producers.
use super::source_addresses::SourceGraphAddressCrosswalk;
use super::*;
use std::collections::BTreeMap;
use wow_emmy::global_access::GlobalAccessKind;
use wow_graph::{
    GraphAssertionKind, GraphProducerLookup, GraphProposalEndpoint, GraphRelationProposal,
    GraphRelationProposalInput,
};
use wow_recognizers::source_state::{
    SOURCE_STATE_PARTITION, SourceStateAssertionEndpoints, SourceStateAssertionFact,
    SourceStateAssertionInput, SourceStateAssertionRecognition, SourceStateFact, SourceStateInput,
    SourceStateReceipt, SourceStateRecognition, recognize_source_state,
    recognize_source_state_assertions,
};

#[derive(Debug)]
pub(super) enum StateRecognition {
    Legacy(SourceStateRecognition),
    Assertions(SourceStateAssertionRecognition),
}
impl StateRecognition {
    pub(super) fn into_parts(
        self,
    ) -> (
        Option<SourceStateRecognition>,
        Option<SourceStateAssertionRecognition>,
    ) {
        match self {
            Self::Legacy(recognition) => (Some(recognition), None),
            Self::Assertions(recognition) => (None, Some(recognition)),
        }
    }
}

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

pub(super) fn publish_bound(
    current: &GraphPartitionSnapshot,
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, StateRecognition)> {
    checkpoint(stop)?;
    let provenance = addresses.source();
    if addresses.direct_provenance().is_none() {
        let (snapshot, recognition) = publish(current, provenance, stop)?;
        return Ok((snapshot, StateRecognition::Legacy(recognition)));
    }
    addresses.analyzer_partition(current, stop)?;
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let facts = provenance
        .state_bindings()
        .iter()
        .map(|binding| {
            checkpoint(stop)?;
            Ok(SourceStateAssertionFact {
                fact: SourceStateFact {
                    fact_id: &binding.binding_id,
                    access_id: &binding.access_id,
                    root_proposal_id: &binding.root_id,
                    caller_proposal_id: &binding.caller_proposal_id,
                    target_proposal_id: &binding.target_proposal_id,
                    kind: binding.kind,
                    confidence: binding.confidence,
                    source_handle_ids: &binding.source_handle_ids,
                    evidence_ids: &binding.evidence_ids,
                },
                endpoints: SourceStateAssertionEndpoints {
                    root: addresses.copied_assertion(
                        GraphAssertionKind::Entity,
                        &binding.root_id,
                        stop,
                    )?,
                    caller: addresses.copied_assertion(
                        GraphAssertionKind::Entity,
                        &binding.caller_proposal_id,
                        stop,
                    )?,
                    target: addresses.copied_assertion(
                        GraphAssertionKind::Entity,
                        &binding.target_proposal_id,
                        stop,
                    )?,
                },
            })
        })
        .collect::<ServiceResult<Vec<_>>>()?;
    checkpoint(stop)?;
    let result = recognize_source_state_assertions(
        SourceStateAssertionInput {
            owner: current,
            scope: addresses.scope(),
            report,
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
    addresses.reserve("state_assertion_recognition", &result.recognition, stop)?;
    let prepared = current
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: current.snapshot().snapshot_id().clone(),
                expected_partition_digest: current
                    .partition(SOURCE_STATE_PARTITION)
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
    Ok((snapshot, StateRecognition::Assertions(result.recognition)))
}

/// Reuse only the opaque recognition produced for these exact native addresses.
pub(super) fn validate_assertion_bindings(
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    recognition: &SourceStateAssertionRecognition,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    checkpoint(stop)?;
    let provenance = addresses.source();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    if addresses.direct_provenance().is_none()
        || recognition.scope() != addresses.scope()
        || recognition.analyzer_report_id() != report.analysis_id()
    {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let bindings = provenance
        .state_bindings()
        .iter()
        .map(|binding| (binding.binding_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    if recognition.receipts().len() != bindings.len()
        || bindings.len() != provenance.state_bindings().len()
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let binding = bindings
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let endpoints = recognition
            .endpoints(&receipt.binding_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        if receipt.access_id != binding.access_id {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        for (reference, id) in [
            (&endpoints.root, binding.root_id.as_str()),
            (&endpoints.caller, binding.caller_proposal_id.as_str()),
            (&endpoints.target, binding.target_proposal_id.as_str()),
        ] {
            if reference != &addresses.assertion(GraphAssertionKind::Entity, id, stop)? {
                return Err(error(ServiceErrorCode::IdentityMismatch));
            }
        }
    }
    checkpoint(stop)
}

pub(super) fn maps_bound(
    snapshot: &GraphPartitionSnapshot,
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    recognition: &StateRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(StateNodes, Vec<StateEdge>)> {
    checkpoint(stop)?;
    let recognition = match (recognition, addresses.direct_provenance()) {
        (StateRecognition::Legacy(recognition), None) => {
            return maps(snapshot, addresses.source(), recognition, stop);
        }
        (StateRecognition::Assertions(recognition), Some(_)) => recognition,
        _ => return Err(error(ServiceErrorCode::IdentityMismatch)),
    };
    validate_assertion_bindings(addresses, recognition, stop)?;
    let lookup = snapshot.producer_lookup(stop).map_err(graph_error)?;
    if lookup.scope() != addresses.scope() {
        return Err(error(ServiceErrorCode::IdentityMismatch));
    }
    let (result, mut nodes) = bound_nodes(snapshot, addresses, &lookup, stop)?;
    let provenance = addresses.source();
    let bindings = provenance
        .state_bindings()
        .iter()
        .map(|binding| (binding.binding_id.as_str(), binding))
        .collect::<BTreeMap<_, _>>();
    let partition = snapshot
        .partition(SOURCE_STATE_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted = partition.report().accepted_relations();
    if accepted.len() != recognition.receipts().len()
        || !partition.batch().entity_proposals().is_empty()
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let assertions = recognition
        .recognition()
        .assertions()
        .iter()
        .map(|assertion| (assertion.assertion_id().as_str(), assertion))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();
    for receipt in recognition.receipts() {
        checkpoint(stop)?;
        let binding = bindings
            .get(receipt.binding_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let endpoints = recognition
            .endpoints(&receipt.binding_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let index = accepted
            .binary_search_by(|entry| entry.proposal_id().cmp(&receipt.proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let original = accepted[index].edge();
        let proposal = partition
            .batch()
            .relation_proposal(&receipt.proposal_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let assertion = assertions
            .get(receipt.assertion_id.as_str())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        validate_admitted_edge(
            &lookup,
            binding,
            receipt,
            endpoints,
            assertion,
            (proposal, original),
            stop,
        )?;
        if !nodes.contains_key(&binding.caller_proposal_id) {
            nodes.insert(
                binding.caller_proposal_id.clone(),
                addresses.node_id(snapshot, &lookup, &binding.caller_proposal_id, stop)?,
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
            snapshot.snapshot().limits(),
        )
        .map_err(graph_error)?;
        if snapshot.snapshot().edge(edge.edge_id()).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        addresses.append(
            "state_edges",
            &mut edges,
            StateEdge {
                binding_id: binding.binding_id.clone(),
                access_id: binding.access_id.clone(),
                root_id: binding.root_id.clone(),
                function_node_id: from.clone(),
                state_node_id: to.clone(),
                edge_id: edge.edge_id().clone(),
                relation: edge.relation(),
                confidence: edge.confidence(),
            },
            stop,
        )?;
        checkpoint(stop)?;
    }
    Ok((result, edges))
}

fn validate_admitted_edge(
    lookup: &GraphProducerLookup<'_>,
    binding: &wow_project::graph::ProjectGraphStateBinding,
    receipt: &SourceStateReceipt,
    endpoints: &SourceStateAssertionEndpoints,
    assertion: &wow_recognizers::RecognitionAssertion,
    (proposal, original): (&wow_graph::GraphRelationProposal, &wow_graph::GraphEdge),
    stop: &AtomicBool,
) -> ServiceResult<()> {
    checkpoint(stop)?;
    let caller = lookup
        .entity(lookup.scope(), &endpoints.caller, stop)
        .map_err(graph_error)?;
    let target = lookup
        .entity(lookup.scope(), &endpoints.target, stop)
        .map_err(graph_error)?;
    let root = lookup
        .entity(lookup.scope(), &endpoints.root, stop)
        .map_err(graph_error)?;
    let (relation, definition) = match binding.kind {
        GlobalAccessKind::Read => (
            wow_graph::GraphRelationKind::ReadsState,
            "source_reads_state",
        ),
        GlobalAccessKind::Write => (
            wow_graph::GraphRelationKind::WritesState,
            "source_writes_state",
        ),
        GlobalAccessKind::UnsupportedAssignment => {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
    };
    let expected_proposal = GraphRelationProposal::new(
        receipt.proposal_id.as_str(),
        definition,
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Existing(caller.accepted().node().node_id().clone()),
            target: GraphProposalEndpoint::Existing(target.accepted().node().node_id().clone()),
            confidence: binding.confidence,
            source_handle_ids: binding.source_handle_ids.clone(),
            evidence_ids: binding.evidence_ids.clone(),
            coverage_ids: Vec::new(),
        },
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    let source_evidence = binding
        .evidence_ids
        .iter()
        .map(|id| id.to_string().into_boxed_str())
        .collect::<Vec<_>>();
    // The engine retains its observation witness in recognition metadata;
    // the admitted graph relation retains only the native source supports.
    let mut expected_evidence = source_evidence.clone();
    expected_evidence.push(receipt.observation_id.clone().into_boxed_str());
    expected_evidence.sort();
    expected_evidence.dedup();
    checkpoint(stop)?;
    if receipt.proposal_id != receipt.assertion_id
        || assertion.assertion_id().as_str() != receipt.assertion_id
        || receipt.observation_id != assertion.observation_id().as_str()
        || proposal != &expected_proposal
        || proposal.proposal_id() != receipt.proposal_id
        || proposal.source_handle_ids() != binding.source_handle_ids.as_slice()
        || proposal.evidence_ids() != binding.evidence_ids.as_slice()
        || original.evidence_ids() != source_evidence.as_slice()
        || original.from() != caller.accepted().node().node_id()
        || original.to() != target.accepted().node().node_id()
        || original.relation() != relation
        || original.confidence() != binding.confidence
        || root.proposal().entity_kind_id() != "state_root"
        || assertion.from() != original.from()
        || assertion.to() != original.to()
        || assertion.relation() != original.relation()
        || assertion.confidence() != original.confidence()
        || assertion.evidence_ids() != expected_evidence.as_slice()
    {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    checkpoint(stop)
}

fn bound_nodes(
    snapshot: &GraphPartitionSnapshot,
    addresses: &SourceGraphAddressCrosswalk<'_, '_>,
    lookup: &GraphProducerLookup<'_>,
    stop: &AtomicBool,
) -> ServiceResult<(StateNodes, BTreeMap<String, wow_graph::GraphNodeId>)> {
    let mut result = StateNodes::empty();
    let mut nodes = BTreeMap::new();
    for root in addresses.source().state_roots() {
        checkpoint(stop)?;
        let node_id = addresses.node_id(snapshot, lookup, &root.proposal_id, stop)?;
        nodes.insert(root.proposal_id.clone(), node_id.clone());
        addresses.append(
            "state_nodes.roots",
            &mut result.roots,
            StateRootNode {
                root_id: root.root_id.clone(),
                name: root.name.clone(),
                scope: root.scope,
                document: root.document.clone(),
                ambiguous: root.ambiguous,
                node_id,
            },
            stop,
        )?;
        checkpoint(stop)?;
    }
    for path in addresses.source().state_paths() {
        checkpoint(stop)?;
        let node_id = addresses.node_id(snapshot, lookup, &path.proposal_id, stop)?;
        nodes.insert(path.proposal_id.clone(), node_id.clone());
        addresses.append(
            "state_nodes.paths",
            &mut result.paths,
            StatePathNode {
                path_id: path.path_id.clone(),
                root_id: path.root_id.clone(),
                keys: path.keys.clone(),
                node_id,
            },
            stop,
        )?;
        checkpoint(stop)?;
    }
    Ok((result, nodes))
}
