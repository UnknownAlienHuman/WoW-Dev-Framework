use super::*;

use wow_project::graph::{ProjectTocFact, ProjectTocFactKind};
use wow_recognizers::source_toc::{
    SourceTocFact, SourceTocFactKind, SourceTocFamily, SourceTocInput, SourceTocLoadState,
    SourceTocRecognition, SourceTocScope, SourceTocSelection, recognize_source_toc,
};

fn recognizer_error(
    family: SourceTocFamily,
    failure: wow_recognizers::RecognizerError,
) -> ServiceError {
    let code = match failure.code() {
        wow_recognizers::RecognizerErrorCode::Cancelled => ServiceErrorCode::Cancelled,
        wow_recognizers::RecognizerErrorCode::BudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InternalContractViolation,
    };
    ServiceError::new(
        code,
        format!(
            "{} recognition rejected ({:?})",
            family.rule_id(),
            failure.code()
        ),
    )
}

fn selection(value: wow_project::load::LoadSelection) -> SourceTocSelection {
    match value {
        wow_project::load::LoadSelection::Included => SourceTocSelection::Included,
        wow_project::load::LoadSelection::Excluded => SourceTocSelection::Excluded,
        wow_project::load::LoadSelection::Unresolved => SourceTocSelection::Unresolved,
    }
}

fn load_state(value: wow_project::load::TocLoadOnDemandState) -> SourceTocLoadState {
    match value {
        wow_project::load::TocLoadOnDemandState::NotDeclared => SourceTocLoadState::NotDeclared,
        wow_project::load::TocLoadOnDemandState::False => SourceTocLoadState::False,
        wow_project::load::TocLoadOnDemandState::True => SourceTocLoadState::True,
        wow_project::load::TocLoadOnDemandState::Unknown => SourceTocLoadState::Unknown,
    }
}

fn scope(value: wow_project::load::TocSavedVariableScope) -> SourceTocScope {
    match value {
        wow_project::load::TocSavedVariableScope::Account => SourceTocScope::Account,
        wow_project::load::TocSavedVariableScope::Character => SourceTocScope::Character,
    }
}

#[derive(Debug, Default, Serialize)]
pub(super) struct TocTopology {
    pub nodes: Vec<TocNode>,
    pub edges: Vec<TocEdge>,
    pub omissions: Vec<TocOmission>,
}

#[derive(Debug, Serialize)]
pub(super) struct TocNode {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub proposal_id: String,
    pub node_id: wow_graph::GraphNodeId,
}

#[derive(Debug, Serialize)]
pub(super) struct TocEdge {
    pub rule_id: String,
    pub proposal_id: String,
    pub edge_id: wow_graph::GraphEdgeId,
    pub from_node_id: wow_graph::GraphNodeId,
    pub to_node_id: wow_graph::GraphNodeId,
    pub relation: wow_graph::GraphRelationKind,
    pub confidence: wow_graph::GraphConfidence,
}

#[derive(Debug, Serialize)]
pub(super) struct TocOmission {
    pub rule_id: String,
    pub fact_ids: Vec<String>,
    pub blocker: String,
}

fn convert<'a>(facts: &'a [ProjectTocFact]) -> Vec<SourceTocFact<'a>> {
    let mut output = Vec::new();
    for fact in facts {
        let kind = match &fact.kind {
            ProjectTocFactKind::Package {
                source_complete, ..
            } => SourceTocFactKind::Package {
                source_complete: *source_complete,
            },
            ProjectTocFactKind::File { path, repeated, .. } => SourceTocFactKind::File {
                path: path.as_deref(),
                repeated: *repeated,
            },
            ProjectTocFactKind::Dependency {
                name,
                dependency_kind,
                resolved_package,
                ..
            } => SourceTocFactKind::Dependency {
                name,
                optional: matches!(
                    dependency_kind,
                    wow_project::load::TocDependencyKind::Optional
                ),
                resolved_package: resolved_package.as_deref(),
            },
            ProjectTocFactKind::LoadOnDemand {
                effective_state,
                conflicting,
                ..
            } => SourceTocFactKind::LoadOnDemand {
                state: load_state(*effective_state),
                conflicting: *conflicting,
            },
            ProjectTocFactKind::SavedVariable {
                name,
                scope: variable_scope,
                state,
                ..
            } => SourceTocFactKind::SavedVariable {
                name,
                scope: scope(*variable_scope),
                declared: matches!(state, wow_project::load::TocSavedVariableState::Declared),
            },
        };
        output.push(SourceTocFact {
            fact_id: fact.fact_id.as_str(),
            context_id: fact.context_id,
            package: fact.package.as_deref(),
            selected_toc: fact.selected_toc.as_str(),
            flavor: fact.flavor.as_str(),
            ordinal: fact.ordinal,
            selection: selection(fact.selection),
            content_digest: fact.content_digest,
            span: fact.span,
            source_handle_id: fact.source_handle_id,
            evidence_id: fact.evidence_id,
            kind,
        });
    }
    output
}

pub(super) fn publish(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceTocRecognition>)> {
    let facts = provenance.toc_facts();
    let converted = convert(facts);
    let mut snapshot = source.clone();
    let mut recognitions = Vec::new();
    for family in SourceTocFamily::ALL {
        checkpoint(stop)?;
        let proposals = recognize_source_toc(
            SourceTocInput {
                owner: &snapshot,
                source_partition: wow_project::graph::SOURCE_GRAPH_PARTITION,
                context: provenance.context(),
                facts: &converted,
                source_handles: provenance.source_handles(),
                evidence: provenance.evidence(),
            },
            family,
            stop,
        )
        .map_err(|failure| recognizer_error(family, failure))?;
        checkpoint(stop)?;
        let prepared = snapshot
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: snapshot.snapshot().snapshot_id().clone(),
                    expected_partition_digest: None,
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
    recognitions: &[SourceTocRecognition],
    stop: &AtomicBool,
) -> ServiceResult<TocTopology> {
    let limits = snapshot.snapshot().limits();
    let nodes = super::materialized::nodes(snapshot, stop)?;
    let mut topology = TocTopology::default();
    for recognition in recognitions {
        let rule_id = recognition.family.rule_id().to_owned();
        let partition_id = recognition.family.partition_id();
        for receipt in &recognition.receipts {
            checkpoint(stop)?;
            for proposal_id in &receipt.entity_proposal_ids {
                let node_id =
                    materialized_partition_node_id(snapshot, partition_id, proposal_id, limits)?;
                topology.nodes.push(TocNode {
                    rule_id: rule_id.clone(),
                    fact_ids: receipt.fact_ids.clone(),
                    proposal_id: proposal_id.clone(),
                    node_id,
                });
            }
            for proposal_id in &receipt.relation_proposal_ids {
                let edge_id =
                    super::materialized::edge_id(snapshot, partition_id, proposal_id, &nodes)?;
                let edge = snapshot
                    .snapshot()
                    .edge(&edge_id)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                topology.edges.push(TocEdge {
                    rule_id: rule_id.clone(),
                    proposal_id: proposal_id.clone(),
                    edge_id,
                    from_node_id: edge.from().clone(),
                    to_node_id: edge.to().clone(),
                    relation: edge.relation(),
                    confidence: edge.confidence(),
                });
            }
        }
        for omission in &recognition.omissions {
            checkpoint(stop)?;
            topology.omissions.push(TocOmission {
                rule_id: rule_id.clone(),
                fact_ids: omission.fact_ids.clone(),
                blocker: omission.blocker.to_owned(),
            });
        }
    }
    Ok(topology)
}
