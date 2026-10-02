//! Glue for the W11 declarative CreateFromMixins and Mixin producer partitions.
use super::*;
use std::collections::BTreeMap;
use wow_recognizers::source_mixins::{
    SOURCE_MIXIN_ASSIGNMENT_PARTITION, SOURCE_MIXIN_PARTITION, SourceMixinAssignmentRecognition,
    SourceMixinInput, SourceMixinRecognition, recognize_source_mixin_assignments,
    recognize_source_mixins,
};

#[derive(Debug, Serialize)]
pub(super) struct MixinInstanceNode {
    call_id: String,
    node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub(super) struct InstantiationEdge {
    call_id: String,
    edge_id: wow_graph::GraphEdgeId,
}

#[derive(Debug, Serialize)]
pub(super) struct ConstructionMixinEdge {
    call_id: String,
    declaration_proposal_id: String,
    argument_ordinals: Vec<u32>,
    edge_id: wow_graph::GraphEdgeId,
}

#[derive(Debug, Serialize)]
pub(super) struct AssignmentMixinEdge {
    call_id: String,
    target_declaration_proposal_id: String,
    declaration_proposal_id: String,
    argument_ordinals: Vec<u32>,
    edge_id: wow_graph::GraphEdgeId,
}

type MixinMaps = (
    Vec<MixinInstanceNode>,
    Vec<InstantiationEdge>,
    Vec<ConstructionMixinEdge>,
    Vec<AssignmentMixinEdge>,
);

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(
    GraphPartitionSnapshot,
    SourceMixinRecognition,
    SourceMixinAssignmentRecognition,
)> {
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let construction = recognize_source_mixins(
        SourceMixinInput {
            owner: source,
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: provenance
                .functions()
                .iter()
                .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
                .collect(),
            declaration_proposals: provenance
                .lua_declarations()
                .iter()
                .map(|declaration| {
                    (
                        (declaration.path.as_str(), declaration.span),
                        declaration.proposal_id.as_str(),
                    )
                })
                .collect(),
            call_support: provenance
                .call_sites()
                .iter()
                .map(|call| {
                    (
                        call.call_id.as_str(),
                        (call.source_handle_id, call.evidence_id),
                    )
                })
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let construction_candidate = source
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: source.snapshot().snapshot_id().clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch: construction.batch,
                coverage: construction.coverage,
            },
            stop,
        )
        .map_err(graph_error)?;

    let assignment = recognize_source_mixin_assignments(
        SourceMixinInput {
            owner: construction_candidate.candidate(),
            source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
            report,
            context: provenance.context(),
            function_proposals: provenance
                .functions()
                .iter()
                .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
                .collect(),
            declaration_proposals: provenance
                .lua_declarations()
                .iter()
                .map(|declaration| {
                    (
                        (declaration.path.as_str(), declaration.span),
                        declaration.proposal_id.as_str(),
                    )
                })
                .collect(),
            call_support: provenance
                .call_sites()
                .iter()
                .map(|call| {
                    (
                        call.call_id.as_str(),
                        (call.source_handle_id, call.evidence_id),
                    )
                })
                .collect(),
            source_handles: provenance.source_handles(),
            evidence: provenance.evidence(),
        },
        stop,
    )
    .map_err(recognizer_error)?;
    checkpoint(stop)?;
    let assignment_candidate = construction_candidate
        .candidate()
        .prepare_replacement(
            GraphPartitionReplacement {
                expected_snapshot_id: construction_candidate
                    .candidate()
                    .snapshot()
                    .snapshot_id()
                    .clone(),
                expected_partition_digest: None,
                producer_version: env!("CARGO_PKG_VERSION").into(),
                batch: assignment.batch,
                coverage: assignment.coverage,
            },
            stop,
        )
        .map_err(graph_error)?;
    Ok((
        assignment_candidate.candidate().clone(),
        construction.recognition,
        assignment.recognition,
    ))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    recognition: &SourceMixinRecognition,
    assignment_recognition: &SourceMixinAssignmentRecognition,
    stop: &AtomicBool,
) -> ServiceResult<MixinMaps> {
    let partition = snapshot
        .partition(SOURCE_MIXIN_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted_entities = partition.report().accepted_entities();
    let accepted_relations = partition.report().accepted_relations();
    let assignment_partition = snapshot
        .partition(SOURCE_MIXIN_ASSIGNMENT_PARTITION)
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let accepted_assignment_relations = assignment_partition.report().accepted_relations();
    let report = provenance
        .function_call_report()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    let function_proposals = provenance
        .functions()
        .iter()
        .map(|function| (function.function_id.as_str(), function.proposal_id.as_str()))
        .collect::<BTreeMap<_, _>>();

    let mut instances = Vec::new();
    let mut instantiations = Vec::new();
    let mut mixins = Vec::new();
    for receipt in recognition.matches() {
        checkpoint(stop)?;
        let entity_index = accepted_entities
            .binary_search_by(|accepted| accepted.proposal_id().cmp(&receipt.instance_proposal_id))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let entity = accepted_entities[entity_index].node();
        let instance = materialized_partition_node_id(
            snapshot,
            SOURCE_MIXIN_PARTITION,
            &receipt.instance_proposal_id,
            snapshot.snapshot().limits(),
        )?;
        if entity.kind() != "mixin_instance" || snapshot.snapshot().node(&instance).is_none() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }

        let call_index = report
            .calls()
            .binary_search_by(|call| call.fact_id().cmp(receipt.call_id.as_str()))
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let call = &report.calls()[call_index];
        let caller_proposal = function_proposals
            .get(call.caller_function_id())
            .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
        let caller = materialized_node_id(snapshot, caller_proposal, snapshot.snapshot().limits())?;
        let relation_index = accepted_relations
            .binary_search_by(|accepted| {
                accepted
                    .proposal_id()
                    .cmp(&receipt.instantiates_proposal_id)
            })
            .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
        let relation = accepted_relations[relation_index].edge();
        let final_edge = wow_graph::GraphEdge::new(
            caller,
            instance.clone(),
            relation.relation(),
            relation.confidence(),
            relation.evidence_ids().to_vec(),
            snapshot.snapshot().limits(),
        )
        .map_err(graph_error)?;
        if relation.relation() != wow_graph::GraphRelationKind::Instantiates
            || snapshot.snapshot().edge(final_edge.edge_id()).is_none()
        {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }

        instances.push(MixinInstanceNode {
            call_id: receipt.call_id.clone(),
            node_id: instance.clone(),
        });
        instantiations.push(InstantiationEdge {
            call_id: receipt.call_id.clone(),
            edge_id: final_edge.edge_id().clone(),
        });

        for mixin in &receipt.mixins {
            checkpoint(stop)?;
            let relation_index = accepted_relations
                .binary_search_by(|accepted| {
                    accepted.proposal_id().cmp(&mixin.relation_proposal_id)
                })
                .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
            let relation = accepted_relations[relation_index].edge();
            let declaration = materialized_node_id(
                snapshot,
                &mixin.declaration_proposal_id,
                snapshot.snapshot().limits(),
            )?;
            let final_edge = wow_graph::GraphEdge::new(
                instance.clone(),
                declaration,
                relation.relation(),
                relation.confidence(),
                relation.evidence_ids().to_vec(),
                snapshot.snapshot().limits(),
            )
            .map_err(graph_error)?;
            if relation.relation() != wow_graph::GraphRelationKind::MixesIn
                || snapshot.snapshot().edge(final_edge.edge_id()).is_none()
            {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            mixins.push(ConstructionMixinEdge {
                call_id: receipt.call_id.clone(),
                declaration_proposal_id: mixin.declaration_proposal_id.clone(),
                argument_ordinals: mixin.argument_ordinals.clone(),
                edge_id: final_edge.edge_id().clone(),
            });
        }
    }

    let mut assignments = Vec::new();
    for receipt in assignment_recognition.matches() {
        checkpoint(stop)?;
        let target = materialized_node_id(
            snapshot,
            &receipt.target_declaration_proposal_id,
            snapshot.snapshot().limits(),
        )?;
        for mixin in &receipt.mixins {
            checkpoint(stop)?;
            let relation_index = accepted_assignment_relations
                .binary_search_by(|accepted| {
                    accepted.proposal_id().cmp(&mixin.relation_proposal_id)
                })
                .map_err(|_| error(ServiceErrorCode::InternalContractViolation))?;
            let relation = accepted_assignment_relations[relation_index].edge();
            let declaration = materialized_node_id(
                snapshot,
                &mixin.declaration_proposal_id,
                snapshot.snapshot().limits(),
            )?;
            let final_edge = wow_graph::GraphEdge::new(
                target.clone(),
                declaration,
                relation.relation(),
                relation.confidence(),
                relation.evidence_ids().to_vec(),
                snapshot.snapshot().limits(),
            )
            .map_err(graph_error)?;
            if relation.relation() != wow_graph::GraphRelationKind::MixesIn
                || snapshot.snapshot().edge(final_edge.edge_id()).is_none()
            {
                return Err(error(ServiceErrorCode::InternalContractViolation));
            }
            assignments.push(AssignmentMixinEdge {
                call_id: receipt.call_id.clone(),
                target_declaration_proposal_id: receipt.target_declaration_proposal_id.clone(),
                declaration_proposal_id: mixin.declaration_proposal_id.clone(),
                argument_ordinals: mixin.argument_ordinals.clone(),
                edge_id: final_edge.edge_id().clone(),
            });
        }
    }

    Ok((instances, instantiations, mixins, assignments))
}

fn recognizer_error(error: wow_recognizers::RecognizerError) -> ServiceError {
    super::error(match error.code() {
        wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_recognizers::RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    })
}
