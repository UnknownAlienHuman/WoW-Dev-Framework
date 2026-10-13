//! Receipt-local assertion closure. These checks admit structural wire data;
//! they never establish native producer, catalog, source, or runtime authority.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use sha2::{Digest, Sha256};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphConfidence, GraphNodeId, GraphProposalEndpoint,
};

use super::project_map_wire::{
    AssertionSupportWire, ProducerSupportOriginWire, ProjectMapWire, ProjectMapWireCheckError,
    ResolvedSupportWire, project_edge, project_node_id,
};
use crate::ContextError;

type Address<'a> = (GraphAssertionKind, &'a str);

fn witness(
    support: &AssertionSupportWire,
) -> Option<(&ProducerSupportOriginWire, Address<'_>, GraphConfidence)> {
    match support {
        AssertionSupportWire::ProducerEntity {
            producer,
            proposal,
            accepted,
            ..
        } if proposal.proposal_id() == accepted.proposal_id() => Some((
            producer,
            (GraphAssertionKind::Entity, proposal.proposal_id()),
            proposal.confidence(),
        )),
        AssertionSupportWire::ProducerRelation {
            producer,
            proposal,
            accepted,
            ..
        } if proposal.proposal_id() == accepted.proposal_id() => Some((
            producer,
            (GraphAssertionKind::Relation, proposal.proposal_id()),
            accepted.edge().confidence(),
        )),
        _ => None,
    }
}

fn local_address(reference: &GraphAssertionRef) -> Option<Address<'_>> {
    match reference {
        GraphAssertionRef::Local { assertion } => Some((assertion.kind, &assertion.proposal_id)),
        // Cross-partition authority is deliberately outside this selected route.
        GraphAssertionRef::Producer { .. } => None,
    }
}

pub(super) fn check_local_chain<E: From<ContextError>>(
    receipt: &ResolvedSupportWire,
    map: &ProjectMapWire,
    work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, ProjectMapWireCheckError<E>> {
    let explanation = &receipt.explanation;
    work(1).map_err(ProjectMapWireCheckError::Work)?;
    if !explanation.derivation_complete || !explanation.conflicts.is_empty() {
        return Ok(false);
    }
    if explanation.assertion_supports.is_empty()
        && explanation.derivations.is_empty()
        && explanation.conflicts.is_empty()
    {
        for support in &explanation.supports {
            work(1).map_err(ProjectMapWireCheckError::Work)?;
            if witness(support)
                .is_some_and(|(_, _, confidence)| confidence != GraphConfidence::Proven)
            {
                return Ok(false);
            }
        }
        return Ok(true);
    }
    let Some((origin, _, _)) = explanation.supports.first().and_then(witness) else {
        return Ok(false);
    };
    let mut witnesses = BTreeMap::new();
    let mut roots = Vec::new();
    let mut entities = BTreeMap::new();
    let mut projected = BTreeMap::new();
    let mut evidence = BTreeSet::new();
    let mut handles = BTreeSet::new();
    for record in &receipt.evidence_resolution.evidence_records {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        evidence.insert(record.evidence_id().to_string());
    }
    for handle in &receipt.evidence_resolution.source_handles {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        handles.insert(handle.handle_id());
    }
    for support in explanation
        .supports
        .iter()
        .chain(&explanation.assertion_supports)
    {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        let Some((producer, address, confidence)) = witness(support) else {
            return Ok(false);
        };
        if producer != origin || witnesses.insert(address, (support, confidence)).is_some() {
            return Ok(false);
        }
        if !super::support_record_links::validate_producer_record(support, map, work, stop)? {
            return Ok(false);
        }
        match support {
            AssertionSupportWire::ProducerEntity {
                proposal,
                accepted,
                definition,
                ..
            } => {
                work(
                    proposal
                        .evidence_ids()
                        .len()
                        .saturating_add(proposal.source_handle_ids().len())
                        .saturating_add(accepted.node().evidence_ids().len())
                        .saturating_add(1),
                )
                .map_err(ProjectMapWireCheckError::Work)?;
                if proposal.entity_kind_id() != definition.kind_id()
                    || proposal
                        .evidence_ids()
                        .iter()
                        .any(|id| !evidence.contains(&id.to_string()))
                    || proposal
                        .source_handle_ids()
                        .iter()
                        .any(|id| !handles.contains(id))
                    || accepted
                        .node()
                        .evidence_ids()
                        .iter()
                        .any(|id| !evidence.contains(id.as_ref()))
                {
                    return Ok(false);
                }
                let node = accepted.node();
                let id = project_node_id(node, map).map_err(ProjectMapWireCheckError::Invalid)?;
                projected.insert(node.node_id().clone(), id);
                entities.insert(proposal.proposal_id(), node.node_id());
            }
            AssertionSupportWire::ProducerRelation {
                proposal,
                accepted,
                definition,
                ..
            } => {
                work(
                    proposal
                        .evidence_ids()
                        .len()
                        .saturating_add(proposal.source_handle_ids().len())
                        .saturating_add(accepted.edge().evidence_ids().len()),
                )
                .map_err(ProjectMapWireCheckError::Work)?;
                if proposal.relation_kind_id() != definition.relation_id()
                    || accepted.edge().relation() != definition.relation()
                    || proposal
                        .evidence_ids()
                        .iter()
                        .any(|id| !evidence.contains(&id.to_string()))
                    || proposal
                        .source_handle_ids()
                        .iter()
                        .any(|id| !handles.contains(id))
                    || accepted
                        .edge()
                        .evidence_ids()
                        .iter()
                        .any(|id| !evidence.contains(id.as_ref()))
                {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
    }
    for support in &explanation.supports {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        let Some((_, address, _)) = witness(support) else {
            return Ok(false);
        };
        roots.push(address);
    }
    for (support, _) in witnesses.values() {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        if let AssertionSupportWire::ProducerRelation {
            proposal, accepted, ..
        } = support
        {
            let endpoints = proposal.endpoints();
            work(accepted.edge().evidence_ids().len().saturating_add(3))
                .map_err(ProjectMapWireCheckError::Work)?;
            for (endpoint, expected) in endpoints
                .iter()
                .zip([accepted.edge().from(), accepted.edge().to()])
            {
                let matches = match endpoint {
                    GraphProposalEndpoint::Existing(id) => id == expected,
                    GraphProposalEndpoint::Proposed(id) => entities
                        .get(id.as_ref())
                        .is_some_and(|node: &&GraphNodeId| *node == expected),
                };
                if !matches {
                    return Ok(false);
                }
            }
            project_edge(accepted.edge(), &projected).map_err(ProjectMapWireCheckError::Invalid)?;
        }
    }
    let mut outputs = BTreeMap::new();
    for observation in &explanation.derivations {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        let record = &observation.record;
        if observation.producer != *origin
            || !record.missing.is_empty()
            || record.inputs.len().saturating_add(record.rebuttals.len()) > 64
        {
            return Ok(false);
        }
        let output = (record.output.kind, record.output.proposal_id.as_ref());
        let Some((_, confidence)) = witnesses.get(&output) else {
            return Ok(false);
        };
        if *confidence == GraphConfidence::Proven || outputs.insert(output, observation).is_some() {
            return Ok(false);
        }
        let material = (
            "derivation",
            origin.partition_id.as_str(),
            origin.partition_digest.as_str(),
            record,
        );
        let max_bytes = map.core.budget.limits.max_metadata_bytes;
        let length = crate::budget::encoded_len(&material, max_bytes, stop)
            .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
        let spans = usize::try_from(length.div_ceil(4096))
            .map_err(|_| ProjectMapWireCheckError::Invalid("derivation size overflow"))?
            .max(1);
        work(spans.saturating_mul(3)).map_err(ProjectMapWireCheckError::Work)?;
        // Reuse the graph data owner for shape, canonical order and field bounds.
        // A single record has at most 64 references; this creates no partition custody.
        let records = GraphAssertionRecords {
            scope: GraphAssertionRecordScope {
                universe: explanation.universe.clone(),
                generation: explanation.generation.clone(),
                source_context_id: explanation.source_context_id,
            },
            derivations: vec![record.clone()],
            conflicts: Vec::new(),
        };
        if records.validate().is_err() {
            return Ok(false);
        }
        let bytes = crate::budget::canonical(&material, max_bytes, stop)
            .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
        let mut digest = Sha256::new();
        for chunk in bytes.chunks(4096) {
            work(1).map_err(ProjectMapWireCheckError::Work)?;
            digest.update(chunk);
        }
        let expected = format!("graph-derivation:sha256:{:x}", digest.finalize());
        if observation.record_id != expected {
            return Ok(false);
        }
        for reference in record.inputs.iter().chain(&record.rebuttals) {
            work(1).map_err(ProjectMapWireCheckError::Work)?;
            let Some(address) = local_address(reference) else {
                return Ok(false);
            };
            let Some((_, input_confidence)) = witnesses.get(&address) else {
                return Ok(false);
            };
            if record.inputs.binary_search(reference).is_ok() && confidence < input_confidence {
                return Ok(false);
            }
        }
    }
    // Follow exactly the roots emitted by the explanation, rejecting cycles,
    // orphan witnesses and omitted derived outputs with one iterative traversal.
    let mut complete = BTreeMap::new();
    let mut active = BTreeSet::new();
    let mut stack = Vec::new();
    let depth_limit = explanation.query.limits.max_derivation_depth.min(64);
    for root in roots {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        stack.push((root, false, 0_u32));
        while let Some((address, exit, depth)) = stack.pop() {
            work(1).map_err(ProjectMapWireCheckError::Work)?;
            if exit {
                let mut height = 0_u32;
                if let Some(observation) = outputs.get(&address) {
                    for reference in observation
                        .record
                        .inputs
                        .iter()
                        .chain(&observation.record.rebuttals)
                    {
                        work(1).map_err(ProjectMapWireCheckError::Work)?;
                        let Some(next) = local_address(reference) else {
                            return Ok(false);
                        };
                        let Some(child_height) = complete.get(&next) else {
                            return Ok(false);
                        };
                        height = height.max(child_height + 1);
                    }
                }
                if depth.saturating_add(height) > depth_limit {
                    return Ok(false);
                }
                active.remove(&address);
                complete.insert(address, height);
                continue;
            }
            if let Some(height) = complete.get(&address) {
                if depth.saturating_add(*height) > depth_limit {
                    return Ok(false);
                }
                continue;
            }
            if depth > depth_limit || !active.insert(address) {
                return Ok(false);
            }
            let Some((_, confidence)) = witnesses.get(&address) else {
                return Ok(false);
            };
            stack.push((address, true, depth));
            if let Some(observation) = outputs.get(&address) {
                for reference in observation
                    .record
                    .inputs
                    .iter()
                    .chain(&observation.record.rebuttals)
                {
                    work(1).map_err(ProjectMapWireCheckError::Work)?;
                    let Some(next) = local_address(reference) else {
                        return Ok(false);
                    };
                    stack.push((next, false, depth + 1));
                }
            } else if *confidence != GraphConfidence::Proven {
                return Ok(false);
            }
        }
    }
    Ok(complete.len() == witnesses.len())
}
