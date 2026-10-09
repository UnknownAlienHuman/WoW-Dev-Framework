//! Declarative state access producers consume the already admitted legacy
//! partition. Source parsing and exact analyzer/support validation run once.
use super::*;
use wow_recognizers::source_state_core::{
    SourceStateCoreFamily, SourceStateCoreInput, SourceStateCoreRecognition,
    recognize_source_state_core,
};

#[derive(Debug, Default, Serialize)]
pub(super) struct StateCoreTopology {
    pub nodes: Vec<StateCoreNode>,
    pub edges: Vec<StateCoreEdge>,
}
#[derive(Debug, Serialize)]
pub(super) struct StateCoreNode {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub proposal_id: String,
    pub node_id: wow_graph::GraphNodeId,
}
#[derive(Debug, Serialize)]
pub(super) struct StateCoreEdge {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub proposal_id: String,
    pub edge_id: wow_graph::GraphEdgeId,
    pub from_node_id: wow_graph::GraphNodeId,
    pub to_node_id: wow_graph::GraphNodeId,
    pub relation: wow_graph::GraphRelationKind,
    pub confidence: wow_graph::GraphConfidence,
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    recognition: &SourceStateRecognition,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceStateCoreRecognition>)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    if recognition.analyzer_report_id() != report.analysis_id() {
        return Err(error(ServiceErrorCode::InternalContractViolation));
    }
    let mut snapshot = source.clone();
    let mut recognitions = Vec::new();
    for family in SourceStateCoreFamily::ALL {
        checkpoint(stop)?;
        let proposals = recognize_source_state_core(
            SourceStateCoreInput {
                owner: &snapshot,
                context: provenance.context(),
                recognition,
            },
            family,
            stop,
        )
        .map_err(|failure| {
            ServiceError::new(
                match failure.code() {
                    wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
                    wow_recognizers::RecognizerErrorCode::BudgetExceeded => {
                        ServiceErrorCode::BudgetExceeded
                    }
                    _ => ServiceErrorCode::InternalContractViolation,
                },
                format!(
                    "{} recognition rejected ({:?})",
                    family.rule_id(),
                    failure.code()
                ),
            )
        })?;
        let prepared = snapshot
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: snapshot.snapshot().snapshot_id().clone(),
                    expected_partition_digest: snapshot
                        .partition(family.partition_id())
                        .map(|partition| partition.partition_digest().into()),
                    producer_version: env!("CARGO_PKG_VERSION").into(),
                    batch: proposals.batch,
                    coverage: proposals.coverage,
                },
                stop,
            )
            .map_err(|failure| {
                ServiceError::new(
                    graph_error(failure.clone()).code(),
                    format!(
                        "{} graph publication rejected ({:?})",
                        family.rule_id(),
                        failure.code()
                    ),
                )
            })?;
        snapshot = prepared.candidate().clone();
        recognitions.push(proposals.recognition);
    }
    Ok((snapshot, recognitions))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    recognitions: &[SourceStateCoreRecognition],
    stop: &AtomicBool,
) -> ServiceResult<StateCoreTopology> {
    let limits = snapshot.snapshot().limits();
    let nodes = super::materialized::nodes(snapshot, stop)?;
    let mut topology = StateCoreTopology::default();
    for recognition in recognitions {
        let rule_id = recognition.family.rule_id();
        let partition_id = recognition.family.partition_id();
        for receipt in &recognition.receipts {
            checkpoint(stop)?;
            for proposal_id in &receipt.entity_proposal_ids {
                checkpoint(stop)?;
                topology.nodes.push(StateCoreNode {
                    rule_id: rule_id.into(),
                    fact_ids: receipt.fact_ids.clone(),
                    proposal_id: proposal_id.clone(),
                    node_id: materialized_partition_node_id(
                        snapshot,
                        partition_id,
                        proposal_id,
                        limits,
                    )?,
                });
            }
            for proposal_id in &receipt.relation_proposal_ids {
                checkpoint(stop)?;
                let edge_id =
                    super::materialized::edge_id(snapshot, partition_id, proposal_id, &nodes)?;
                let edge = snapshot
                    .snapshot()
                    .edge(&edge_id)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                topology.edges.push(StateCoreEdge {
                    rule_id: rule_id.into(),
                    fact_ids: receipt.fact_ids.clone(),
                    proposal_id: proposal_id.clone(),
                    edge_id,
                    from_node_id: edge.from().clone(),
                    to_node_id: edge.to().clone(),
                    relation: edge.relation(),
                    confidence: edge.confidence(),
                });
            }
        }
    }
    topology
        .nodes
        .sort_by(|a, b| (&a.rule_id, &a.proposal_id).cmp(&(&b.rule_id, &b.proposal_id)));
    topology
        .edges
        .sort_by(|a, b| (&a.rule_id, &a.proposal_id).cmp(&(&b.rule_id, &b.proposal_id)));
    Ok(topology)
}
