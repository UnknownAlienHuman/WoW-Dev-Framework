//! W11 signal and hook producer partitions plus their node/edge crosswalks. Each
//! adapter is an independent declarative producer published after every earlier
//! owner, so each one crosswalks against the accepted source graph that precedes
//! it. Nodes and edges are bound only after every producer has published.
use std::collections::BTreeMap;

use super::*;

pub use wow_recognizers::source_bridge::{
    W2BridgeRecognition, W2Input, w2_recognize_native_event_bridges,
};
pub use wow_recognizers::source_signals::{
    W1Input, W1Recognition, W3Input, W3Recognition, W4Recognition, recognize_source_cvar_callbacks,
    w1_recognize_native_frame_events, w3_recognize_signals,
};
fn recognizer_error(error: wow_recognizers::RecognizerError) -> ServiceError {
    super::error(match error.code() {
        wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_recognizers::RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    })
}

#[derive(Debug, Serialize)]
pub(super) struct SignalNode {
    pub call_id: String,
    pub kind: String,
    pub key: String,
    pub node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub(super) struct SignalEdge {
    pub call_id: String,
    pub relation: wow_graph::GraphRelationKind,
    pub confidence: wow_graph::GraphConfidence,
    pub function_node_id: wow_graph::GraphNodeId,
    pub target_node_id: wow_graph::GraphNodeId,
    pub edge_id: wow_graph::GraphEdgeId,
}

#[derive(Debug, Default)]
pub(super) struct SignalTopology {
    pub nodes: Vec<SignalNode>,
    pub edges: Vec<SignalEdge>,
}

fn replacement(
    source: &GraphPartitionSnapshot,
    batch: wow_graph::GraphProposalBatch,
    coverage: Vec<wow_graph::GraphCoverageRecord>,
    stop: &AtomicBool,
) -> ServiceResult<GraphPartitionSnapshot> {
    source
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: source.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch,
                coverage,
            },
            stop,
        )
        .map(|plan| plan.candidate().clone())
        .map_err(graph_error)
}

pub(super) fn publish_signals(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(
    GraphPartitionSnapshot,
    W1Recognition,
    W2BridgeRecognition,
    W3Recognition,
    W4Recognition,
)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    checkpoint(stop)?;

    let function_proposals = provenance
        .functions()
        .iter()
        .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let call_support = provenance
        .call_sites()
        .iter()
        .map(|site| {
            (
                site.call_id.as_str(),
                (site.source_handle_id, site.evidence_id),
            )
        })
        .collect::<BTreeMap<_, _>>();
    // Main-declaration crosswalk. W1 borrows the path, so the owned map is
    // materialized once and reused by the later adapters.
    let declarations = provenance
        .lua_declarations()
        .iter()
        .map(|declaration| {
            (
                declaration.path.clone(),
                declaration.span,
                declaration.proposal_id.clone(),
            )
        })
        .collect::<Vec<_>>();

    // core.signal.native_frame_event@1
    let frame = w1_recognize_native_frame_events(
        W1Input {
            owner: source,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: function_proposals.clone(),
            declaration_proposals: declarations
                .iter()
                .map(|(path, span, proposal)| ((path.as_str(), *span), proposal.as_str()))
                .collect(),
            call_support: call_support.clone(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let frame_snapshot = replacement(source, frame.batch, frame.coverage, stop)?;
    let owner_view = &frame_snapshot;

    // core.signal.native_event_registry_bridge@1
    let string_declarations = declarations
        .iter()
        .map(|(path, span, proposal)| ((path.clone(), *span), proposal.as_str()))
        .collect::<BTreeMap<_, _>>();
    let bridge = w2_recognize_native_event_bridges(
        W2Input {
            owner: owner_view,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: function_proposals.clone(),
            declaration_proposals: string_declarations.clone(),
            call_support: call_support.clone(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let bridge_snapshot = replacement(owner_view, bridge.batch, bridge.coverage, stop)?;
    let owner_view = &bridge_snapshot;

    // core.signal.custom_registry_producer@1 and _subscription@1
    let custom = w3_recognize_signals(
        W3Input {
            owner: owner_view,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: function_proposals.clone(),
            declaration_proposals: string_declarations.clone(),
            call_support: call_support.clone(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let custom_snapshot = replacement(owner_view, custom.batch, custom.coverage, stop)?;
    let owner_view = &custom_snapshot;

    // core.signal.cvar_callback@1
    let cvar = recognize_source_cvar_callbacks(
        W3Input {
            owner: owner_view,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals,
            declaration_proposals: string_declarations,
            call_support,
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let cvar_snapshot = replacement(owner_view, cvar.batch, cvar.coverage, stop)?;

    Ok((
        cvar_snapshot,
        frame.recognition,
        bridge.recognition,
        custom.recognition,
        cvar.recognition,
    ))
}
#[allow(clippy::too_many_lines)]
pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    signal_recognition: &W1Recognition,
    bridge_recognition: &W2BridgeRecognition,
    custom_recognition: &W3Recognition,
    cvar_recognition: &W4Recognition,
    stop: &AtomicBool,
) -> ServiceResult<SignalTopology> {
    let bounds = snapshot.snapshot().limits();
    let mut function_nodes = BTreeMap::new();
    for function in provenance.functions() {
        checkpoint(stop)?;
        let node_id = materialized_node_id(snapshot, &function.proposal_id, bounds)?;
        function_nodes.insert(function.function_id.as_str(), node_id);
    }

    // Each producer publishes exactly one entity per retained site, so node and
    // relation identities are resolved from the published partition rather than
    // re-derived from the recognition payload.
    let entities = [
        (
            "native_frame_event",
            wow_recognizers::source_signals::W1_SIGNAL_PARTITION,
        ),
        (
            "native_event_bridge",
            wow_recognizers::source_bridge::W2_PARTITION,
        ),
        (
            "custom_signal",
            wow_recognizers::source_signals::W3_SIGNAL_PARTITION,
        ),
        ("cvar_key", wow_recognizers::source_signals::W4_PARTITION),
    ];

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (kind, partition_id) in entities {
        checkpoint(stop)?;
        let partition = snapshot
            .partition(partition_id)
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        for accepted in partition.report().accepted_entities() {
            checkpoint(stop)?;
            nodes.push(SignalNode {
                call_id: String::new(),
                kind: kind.to_owned(),
                key: accepted.proposal_id().to_string(),
                node_id: accepted.node().node_id().clone(),
            });
        }
        for accepted in partition.report().accepted_relations() {
            checkpoint(stop)?;
            edges.push(SignalEdge {
                call_id: String::new(),
                relation: accepted.edge().relation(),
                confidence: accepted.edge().confidence(),
                function_node_id: accepted.edge().from().clone(),
                target_node_id: accepted.edge().to().clone(),
                edge_id: accepted.edge().edge_id().clone(),
            });
        }
    }

    let _ = (
        signal_recognition,
        bridge_recognition,
        custom_recognition,
        cvar_recognition,
        &function_nodes,
    );
    Ok(SignalTopology { nodes, edges })
}
