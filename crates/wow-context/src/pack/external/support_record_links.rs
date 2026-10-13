//! Private semantic joins for one already typed producer support record.
//!
//! This validates the internal proposal/accepted/definition relationship only.
//! It does not establish producer authenticity, source custody, or runtime use.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_graph::{
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphEntityKindDefinition,
    GraphEntityProposal, GraphNode, GraphRelationKindDefinition, GraphRelationProposal,
};

use crate::ContextError;

use super::project_map_wire::{
    AssertionSupportWire, PROJECT_MAP_PROJECTION_LIMITS, ProjectMapWire, ProjectMapWireCheckError,
};

/// Check that a producer proposal, its accepted record, and its typed definition
/// agree under the native graph proposal semantics. Foundation supports have no
/// producer proposal/accepted pair and return `false`.
pub(super) fn validate_producer_record<E: From<ContextError>>(
    support: &AssertionSupportWire,
    map: &ProjectMapWire,
    work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, ProjectMapWireCheckError<E>> {
    work(1).map_err(ProjectMapWireCheckError::Work)?;
    let length =
        crate::budget::encoded_len(support, map.core.budget.limits.max_metadata_bytes, stop)
            .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
    let spans = usize::try_from(length.div_ceil(4096))
        .map_err(|_| ProjectMapWireCheckError::Invalid("support record size overflow"))?
        .max(1);
    work(spans.saturating_mul(3)).map_err(ProjectMapWireCheckError::Work)?;
    match support {
        AssertionSupportWire::ProducerEntity {
            proposal,
            accepted,
            definition,
            ..
        } => validate_entity(proposal, accepted, definition, map, work, stop),
        AssertionSupportWire::ProducerRelation {
            proposal,
            accepted,
            definition,
            ..
        } => validate_relation(proposal, accepted, definition, work),
        AssertionSupportWire::FoundationEntity { .. }
        | AssertionSupportWire::FoundationRelation { .. } => Ok(false),
    }
}

fn validate_entity<E: From<ContextError>>(
    proposal: &GraphEntityProposal,
    accepted: &GraphAcceptedEntityProposal,
    definition: &GraphEntityKindDefinition,
    map: &ProjectMapWire,
    work: &mut impl FnMut(usize) -> Result<(), E>,
    stop: &AtomicBool,
) -> Result<bool, ProjectMapWireCheckError<E>> {
    work(1).map_err(ProjectMapWireCheckError::Work)?;
    if definition.validate().is_err()
        || accepted
            .node()
            .validate(PROJECT_MAP_PROJECTION_LIMITS)
            .is_err()
        || proposal.proposal_id() != accepted.proposal_id()
        || proposal.entity_kind_id() != definition.kind_id()
        || proposal.entity_kind_id() != accepted.node().kind()
        || accepted.node().universe() != &map.core.binding.graph_universe
        || !definition.allows_universe(&map.core.binding.graph_universe)
        || !definition.allows_confidence(proposal.confidence())
        || !proposal
            .semantic_key()
            .keys()
            .map(Box::as_ref)
            .eq(definition.identity_fields().iter().map(Box::as_ref))
    {
        return Ok(false);
    }

    if !proposal_evidence_matches(
        proposal.evidence_ids(),
        accepted.node().evidence_ids(),
        work,
    )? {
        return Ok(false);
    }

    // Native proposal.rs::derive_owner_key hashes this exact struct shape and
    // schema. Bound the canonical encoding, reserve work by 4 KiB chunk, then
    // hash incrementally so the checksum pass remains cancellable and budgeted.
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        kind_id: &'a str,
        semantic_key: &'a std::collections::BTreeMap<Box<str>, wow_graph::GraphProposalValue>,
    }
    let identity = Identity {
        schema: "wow-graph/entity-semantic-key/e2-a/1",
        kind_id: proposal.entity_kind_id(),
        semantic_key: proposal.semantic_key(),
    };
    work(1).map_err(ProjectMapWireCheckError::Work)?;
    let encoded_len =
        crate::budget::encoded_len(&identity, map.core.budget.limits.max_metadata_bytes, stop)
            .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
    let chunks = usize::try_from(encoded_len.div_ceil(4096))
        .map_err(|_| ProjectMapWireCheckError::Invalid("entity semantic-key work overflow"))?
        .max(1);
    work(chunks).map_err(ProjectMapWireCheckError::Work)?;
    let bytes =
        crate::budget::canonical(&identity, map.core.budget.limits.max_metadata_bytes, stop)
            .map_err(|error| ProjectMapWireCheckError::Work(E::from(error)))?;
    let mut hasher = Sha256::new();
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for chunk in bytes.chunks(4096) {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        hasher.update(chunk);
    }
    let mut expected_owner_key = format!("{}:sha256:", proposal.entity_kind_id());
    for byte in hasher.finalize() {
        expected_owner_key.push(char::from(HEX[usize::from(byte >> 4)]));
        expected_owner_key.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(accepted.node().owner_key() == expected_owner_key)
}

fn validate_relation<E: From<ContextError>>(
    proposal: &GraphRelationProposal,
    accepted: &GraphAcceptedRelationProposal,
    definition: &GraphRelationKindDefinition,
    work: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, ProjectMapWireCheckError<E>> {
    work(1).map_err(ProjectMapWireCheckError::Work)?;
    if definition.validate().is_err()
        || accepted
            .edge()
            .validate(PROJECT_MAP_PROJECTION_LIMITS)
            .is_err()
        || proposal.proposal_id() != accepted.proposal_id()
        || proposal.relation_kind_id() != definition.relation_id()
        || definition.relation() != accepted.edge().relation()
        || !definition.allows_confidence(proposal.confidence())
        || accepted.edge().confidence() != proposal.confidence()
    {
        return Ok(false);
    }
    proposal_evidence_matches(
        proposal.evidence_ids(),
        accepted.edge().evidence_ids(),
        work,
    )
}

fn proposal_evidence_matches<E: From<ContextError>>(
    proposal_ids: &[wow_core::EvidenceId],
    accepted_ids: &[Box<str>],
    work: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<bool, ProjectMapWireCheckError<E>> {
    let mut proposed = BTreeSet::new();
    for id in proposal_ids {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        if !proposed.insert(id.to_string()) {
            return Ok(false);
        }
    }
    let mut accepted = BTreeSet::new();
    for id in accepted_ids {
        work(1).map_err(ProjectMapWireCheckError::Work)?;
        if !accepted.insert(id.to_string()) {
            return Ok(false);
        }
    }
    Ok(proposed == accepted)
}

/// Validate definition endpoint-kind policy when the caller has already joined
/// the relation's typed source and target nodes. The helper does not resolve or
/// infer endpoint identity itself.
pub(super) fn relation_endpoint_kinds_match(
    definition: &GraphRelationKindDefinition,
    source: &GraphNode,
    target: &GraphNode,
) -> bool {
    definition.validate().is_ok()
        && definition.allows_source_kind(source.kind())
        && definition.allows_target_kind(target.kind())
}
