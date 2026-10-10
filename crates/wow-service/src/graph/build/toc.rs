use super::source_addresses::SourceGraphAddressCrosswalk;
use super::*;
use std::collections::{BTreeMap, BTreeSet};
use wow_graph::{GraphAssertionKind, GraphAssertionRef};

use wow_project::graph::{PlatformGraphProducer, ProjectTocFact, ProjectTocFactKind};
use wow_recognizers::source_toc::{
    SourceTocAssertionInput, SourceTocFact, SourceTocFactKind, SourceTocFamily, SourceTocInput,
    SourceTocLoadState, SourceTocRecognition, SourceTocScope, SourceTocSelection,
    recognize_source_toc, recognize_source_toc_assertions,
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
    publish_families(source, provenance, &SourceTocFamily::ALL, None, stop)
}

pub(super) fn publish_bound(
    source: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceTocRecognition>)> {
    if crosswalk.direct_provenance().is_none() {
        return publish(source, crosswalk.source(), stop);
    }
    publish_families(
        source,
        crosswalk.source(),
        &SourceTocFamily::ALL,
        Some(crosswalk),
        stop,
    )
}

pub(super) fn publish_state_root(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceTocRecognition)> {
    let (snapshot, mut recognitions) = publish_families(
        source,
        provenance,
        &[SourceTocFamily::SavedVariableRoot],
        None,
        stop,
    )?;
    let recognition = recognitions
        .pop()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    Ok((snapshot, recognition))
}

pub(super) fn publish_state_root_bound(
    source: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, SourceTocRecognition)> {
    if crosswalk.direct_provenance().is_none() {
        return publish_state_root(source, crosswalk.source(), stop);
    }
    let (snapshot, mut recognitions) = publish_families(
        source,
        crosswalk.source(),
        &[SourceTocFamily::SavedVariableRoot],
        Some(crosswalk),
        stop,
    )?;
    let recognition = recognitions
        .pop()
        .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
    Ok((snapshot, recognition))
}

fn source_files<'a>(
    provenance: &'a ProjectGraphProvenance,
    facts: &[SourceTocFact<'_>],
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    stop: &AtomicBool,
) -> ServiceResult<BTreeMap<&'a str, GraphAssertionRef>> {
    let mut paths = BTreeSet::new();
    for fact in facts {
        checkpoint(stop)?;
        if let SourceTocFactKind::File {
            path: Some(path), ..
        } = &fact.kind
        {
            paths.insert(*path);
        }
    }
    let mut files = BTreeMap::new();
    for file in provenance.files() {
        checkpoint(stop)?;
        if !paths.contains(file.path.as_str()) {
            continue;
        }
        let reference =
            crosswalk.copied_assertion(GraphAssertionKind::Entity, &file.proposal_id, stop)?;
        if !matches!(
            &reference,
            GraphAssertionRef::Producer { partition_id, .. }
                if partition_id.as_ref() == PlatformGraphProducer::Inventory.partition_id()
        ) {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
        if files.insert(file.path.as_str(), reference).is_some() {
            return Err(error(ServiceErrorCode::InternalContractViolation));
        }
    }
    checkpoint(stop)?;
    Ok(files)
}

fn publish_families(
    source: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    families: &[SourceTocFamily],
    crosswalk: Option<&SourceGraphAddressCrosswalk<'_, '_>>,
    stop: &AtomicBool,
) -> ServiceResult<(GraphPartitionSnapshot, Vec<SourceTocRecognition>)> {
    let facts = provenance.toc_facts();
    let converted = convert(facts);
    let files = match crosswalk {
        Some(crosswalk) => source_files(provenance, &converted, crosswalk, stop)?,
        None => BTreeMap::new(),
    };
    let mut snapshot = source.clone();
    let mut recognitions = Vec::new();
    for &family in families {
        checkpoint(stop)?;
        let proposals = match crosswalk {
            Some(crosswalk) => recognize_source_toc_assertions(
                SourceTocAssertionInput {
                    owner: &snapshot,
                    scope: crosswalk.scope(),
                    context: provenance.context(),
                    facts: &converted,
                    source_files: &files,
                    source_handles: provenance.source_handles(),
                    evidence: provenance.evidence(),
                },
                family,
                stop,
            ),
            None => recognize_source_toc(
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
            ),
        }
        .map_err(|failure| recognizer_error(family, failure))?;
        checkpoint(stop)?;
        if let Some(crosswalk) = crosswalk {
            let field = if family == SourceTocFamily::SavedVariableRoot {
                "state_root_recognition"
            } else {
                "toc_recognition"
            };
            crosswalk.reserve(field, &proposals.recognition, stop)?;
        }
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
    checkpoint(stop)?;
    Ok((snapshot, recognitions))
}

pub(super) fn maps(
    snapshot: &GraphPartitionSnapshot,
    crosswalk: &SourceGraphAddressCrosswalk<'_, '_>,
    recognitions: &[SourceTocRecognition],
    stop: &AtomicBool,
) -> ServiceResult<TocTopology> {
    let limits = snapshot.snapshot().limits();
    let nodes = super::materialized::nodes(snapshot, stop)?;
    let mut topology = TocTopology::default();
    for recognition in recognitions {
        let rule_id = recognition.family.rule_id().to_owned();
        let partition_id = recognition.family.partition_id();
        let (nodes_field, edges_field, omissions_field) =
            if recognition.family == SourceTocFamily::SavedVariableRoot {
                (
                    "state_root_topology.nodes",
                    "state_root_topology.edges",
                    "state_root_topology.omissions",
                )
            } else {
                (
                    "toc_topology.nodes",
                    "toc_topology.edges",
                    "toc_topology.omissions",
                )
            };
        for receipt in &recognition.receipts {
            checkpoint(stop)?;
            for proposal_id in &receipt.entity_proposal_ids {
                let node_id =
                    materialized_partition_node_id(snapshot, partition_id, proposal_id, limits)?;
                crosswalk.append(
                    nodes_field,
                    &mut topology.nodes,
                    TocNode {
                        rule_id: rule_id.clone(),
                        fact_ids: receipt.fact_ids.clone(),
                        proposal_id: proposal_id.clone(),
                        node_id,
                    },
                    stop,
                )?;
            }
            for proposal_id in &receipt.relation_proposal_ids {
                let edge_id =
                    super::materialized::edge_id(snapshot, partition_id, proposal_id, &nodes)?;
                let edge = snapshot
                    .snapshot()
                    .edge(&edge_id)
                    .ok_or_else(|| error(ServiceErrorCode::InternalContractViolation))?;
                crosswalk.append(
                    edges_field,
                    &mut topology.edges,
                    TocEdge {
                        rule_id: rule_id.clone(),
                        proposal_id: proposal_id.clone(),
                        edge_id,
                        from_node_id: edge.from().clone(),
                        to_node_id: edge.to().clone(),
                        relation: edge.relation(),
                        confidence: edge.confidence(),
                    },
                    stop,
                )?;
            }
        }
        for omission in &recognition.omissions {
            checkpoint(stop)?;
            crosswalk.append(
                omissions_field,
                &mut topology.omissions,
                TocOmission {
                    rule_id: rule_id.clone(),
                    fact_ids: omission.fact_ids.clone(),
                    blocker: omission.blocker.to_owned(),
                },
                stop,
            )?;
        }
    }
    checkpoint(stop)?;
    Ok(topology)
}
