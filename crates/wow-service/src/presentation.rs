use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CausalRelation, PresentationGraph, PresentationNode, PresentationNodeKind,
    PresentationRelation, RawFinding, RuleEvaluation, ServiceError, ServiceErrorCode,
    ServiceResult,
};

pub(crate) fn build(
    raw_findings: &[RawFinding],
    evaluations: &[RuleEvaluation],
    relations: &[CausalRelation],
    relation_budget: u32,
) -> ServiceResult<PresentationGraph> {
    if relations.len()
        > usize::try_from(relation_budget).map_err(|_| {
            ServiceError::new(
                ServiceErrorCode::BudgetExceeded,
                "presentation relation budget cannot fit usize",
            )
        })?
    {
        return Err(ServiceError::new(
            ServiceErrorCode::BudgetExceeded,
            "presentation relation budget exceeded",
        ));
    }

    let mut nodes = BTreeMap::<Box<str>, PresentationNodeKind>::new();
    for finding in raw_findings {
        if nodes
            .insert(finding.finding_id().into(), PresentationNodeKind::Finding)
            .is_some()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "duplicate raw finding identity",
            ));
        }
    }
    for evaluation in evaluations {
        if let Some(blocker) = evaluation.blocker()
            && nodes
                .insert(blocker.blocker_id().into(), PresentationNodeKind::Blocker)
                .is_some()
        {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "blocker identity collides with another presentation node",
            ));
        }
    }

    let mut observed = BTreeSet::new();
    let mut children = BTreeSet::new();
    let mut edges = BTreeMap::<&str, Vec<&str>>::new();
    let mut output_relations = Vec::with_capacity(relations.len());
    for relation in relations {
        if !nodes.contains_key(relation.parent_id()) || !nodes.contains_key(relation.child_id()) {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "presentation relation references an unknown node",
            ));
        }
        let key = (relation.parent_id(), relation.child_id(), relation.kind());
        if !observed.insert(key) {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "duplicate presentation relation",
            ));
        }
        children.insert(relation.child_id());
        edges
            .entry(relation.parent_id())
            .or_default()
            .push(relation.child_id());
        output_relations.push(PresentationRelation::from_relation(relation));
    }
    ensure_acyclic(&nodes, &edges)?;

    let output_nodes = nodes
        .iter()
        .map(|(node_id, kind)| PresentationNode::new(node_id.clone(), *kind))
        .collect::<Vec<_>>();
    let display_root_ids = nodes
        .keys()
        .filter(|node_id| !children.contains(node_id.as_ref()))
        .cloned()
        .collect::<Vec<_>>();
    output_relations.sort();
    Ok(PresentationGraph::new(
        output_nodes,
        display_root_ids,
        output_relations,
    ))
}

fn ensure_acyclic(
    nodes: &BTreeMap<Box<str>, PresentationNodeKind>,
    edges: &BTreeMap<&str, Vec<&str>>,
) -> ServiceResult<()> {
    let mut state = BTreeMap::<&str, u8>::new();
    for node in nodes.keys() {
        visit(node, edges, &mut state)?;
    }
    Ok(())
}

fn visit<'a>(
    node: &'a str,
    edges: &BTreeMap<&'a str, Vec<&'a str>>,
    state: &mut BTreeMap<&'a str, u8>,
) -> ServiceResult<()> {
    match state.get(node).copied() {
        Some(1) => {
            return Err(ServiceError::new(
                ServiceErrorCode::InvalidContext,
                "presentation graph contains a cycle",
            ));
        }
        Some(2) => return Ok(()),
        None | Some(_) => {}
    }
    state.insert(node, 1);
    if let Some(children) = edges.get(node) {
        for child in children {
            visit(child, edges, state)?;
        }
    }
    state.insert(node, 2);
    Ok(())
}
