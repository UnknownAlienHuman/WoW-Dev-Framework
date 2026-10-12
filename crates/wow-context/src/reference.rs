//! Exact accepted API-to-Reference records. Inputs are inert borrowed data;
//! only selected native graph and Reference owner validation produces facets.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_core::{
    ClaimScope, EvidenceConfidence, EvidenceRecord, ProvenanceClass, SourceHandle, SourceOriginKind,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphEntityProposal, GraphNode,
    GraphPartitionRead, GraphProposalValue,
};
use wow_reference::{CoverageStatus, LookupResult, ReferenceRecord, ReferenceRecordKind};

use crate::budget::{Budget, checkpoint, exhausted};
use crate::{ContextError, ContextErrorCode, ContextResult, ContextUniverseSet, ProjectMapView};

/// An inert request for one original producer association. These references
/// cannot attest source or graph authority; construction validates every field.
#[derive(Serialize)]
pub struct ReferenceOriginInput<'a> {
    assertion: &'a GraphAssertionRef,
    source: &'a SourceHandle,
    evidence: &'a EvidenceRecord,
}

impl<'a> ReferenceOriginInput<'a> {
    pub const fn new(
        assertion: &'a GraphAssertionRef,
        source: &'a SourceHandle,
        evidence: &'a EvidenceRecord,
    ) -> Self {
        Self {
            assertion,
            source,
            evidence,
        }
    }
}

/// Borrowed original owner records, not reconstructed payload semantics.
#[derive(Serialize)]
pub(crate) struct ReferenceRecordFacet<'a> {
    node: &'a GraphNode,
    assertion: &'a GraphAssertionRef,
    partition_id: &'static str,
    partition_coverage: CoverageStatus,
    record: &'a ReferenceRecord,
    record_digest: &'a str,
    source: &'a SourceHandle,
    evidence: &'a EvidenceRecord,
    source_association: &'static str,
    payload_interpretation: &'static str,
    absence_authoritative: bool,
}

pub(crate) fn reference_facets<'a>(
    universe: &ContextUniverseSet<'a>,
    view: &ProjectMapView<'_, 'a>,
    graph: &GraphPartitionRead<'a>,
    origins: &[ReferenceOriginInput<'a>],
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Vec<ReferenceRecordFacet<'a>>> {
    checkpoint(stop)?;
    if !view.bound_to(universe) || !std::ptr::eq(graph.owner(), universe.pair.graph()) {
        return Err(mismatch());
    }
    budget.work(view.nodes().len(), stop)?;
    let selected = view
        .nodes()
        .iter()
        .filter(|node| node.kind() == "native_api_callable")
        .count();
    if origins.len() != selected {
        return Err(missing());
    }
    budget.retain(origins, stop)?;
    const PARTITION: &str = wow_reference::native_view::NATIVE_API_PARTITION;
    budget.work(universe.reference.partitions().len(), stop)?;
    let partition = universe
        .reference
        .partitions()
        .iter()
        .find(|row| row.id() == PARTITION);
    let mut result = Vec::with_capacity(selected);
    for origin in origins {
        budget.work(1, stop)?;
        let GraphAssertionRef::Producer {
            partition_id,
            batch_id,
            assertion,
        } = origin.assertion
        else {
            return Err(mismatch());
        };
        budget.work(graph.owner().partitions().len(), stop)?;
        let producer = graph.owner().partition(partition_id).ok_or_else(missing)?;
        let graph_lookup_work = producer.batch().entity_proposals().len().max(1).ilog2() as usize
            + producer.report().accepted_entities().len().max(1).ilog2() as usize
            + graph.owner().snapshot().nodes().len().max(1).ilog2() as usize
            + graph.owner().snapshot().limits().max_evidence_per_record as usize
                * (graph
                    .owner()
                    .snapshot()
                    .limits()
                    .max_evidence_per_record
                    .max(1)
                    .ilog2() as usize
                    + 1)
            + 8;
        budget.work(graph_lookup_work, stop)?;
        if producer.batch().batch_id() != batch_id.as_ref()
            || assertion.kind != GraphAssertionKind::Entity
            || producer.batch().source_context_id() != universe.binding.source_context_id
        {
            return Err(mismatch());
        }
        let proposal = producer
            .batch()
            .entity_proposal(&assertion.proposal_id)
            .ok_or_else(missing)?;
        let node = graph
            .published_entity(partition_id, &assertion.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if proposal.entity_kind_id() != "native_api_callable"
            || proposal.semantic_key().len() != 5
            || proposal.confidence() != GraphConfidence::Derived
            || node.kind() != "native_api_callable"
            || !view
                .node(node.node_id())
                .is_some_and(|selected| std::ptr::eq(selected, node))
            || proposal.source_handle_ids() != [origin.source.handle_id()]
            || proposal.evidence_ids() != [origin.evidence.evidence_id()]
        {
            return Err(mismatch());
        }
        let profile = universe.pair.project().configuration().selected_profile();
        if text(proposal, "reference_profile_id")? != profile.profile_id().as_str()
            || text(proposal, "reference_generation_id")? != universe.reference.generation_id()
            || text(proposal, "reference_digest")? != universe.reference.self_digest()
        {
            return Err(mismatch());
        }
        let key = text(proposal, "record_key")?;
        let record_digest = text(proposal, "record_digest")?;
        let reference_lookup_work = universe
            .reference
            .conflicts()
            .len()
            .checked_add(universe.reference.partitions().len())
            .and_then(|work| work.checked_add(partition.map_or(0, |row| row.records().len())))
            .and_then(|work| work.checked_add(1))
            .ok_or_else(exhausted)?;
        budget.work(reference_lookup_work, stop)?;
        let record = match universe.reference.lookup(PARTITION, key) {
            LookupResult::Found(record) => record,
            _ => return Err(missing()),
        };
        checkpoint(stop)?;
        if record.kind() != ReferenceRecordKind::Api {
            return Err(mismatch());
        }
        let limits = universe.limits;
        let remaining_bytes = limits
            .max_metadata_bytes
            .checked_sub(budget.retained_bytes)
            .ok_or_else(exhausted)?;
        let remaining_work = limits
            .max_context_work
            .checked_sub(budget.work)
            .ok_or_else(exhausted)?;
        let (actual_digest, _, steps) = record
            .digest_bounded(
                usize::try_from(remaining_bytes).map_err(|_| exhausted())?,
                usize::try_from(remaining_work).map_err(|_| exhausted())?,
                stop,
            )
            .map_err(|error| match error {
                wow_reference::ReferenceViewError::Cancelled => {
                    ContextError::new(ContextErrorCode::Cancelled)
                }
                wow_reference::ReferenceViewError::LimitExceeded => exhausted(),
                _ => mismatch(),
            })?;
        budget.work(steps, stop)?;
        if actual_digest != record_digest {
            return Err(mismatch());
        }
        origin.source.validate().map_err(|_| mismatch())?;
        origin.evidence.validate().map_err(|_| mismatch())?;
        budget.work(record.source_ids().len(), stop)?;
        if origin.source.origin_kind() != SourceOriginKind::ReferencePack
            || origin.source.project_generation().is_some()
            || origin.source.reference_generation() != Some(universe.binding.reference_generation)
            || !record
                .source_ids()
                .iter()
                .any(|id| id.as_ref() == origin.source.origin_id())
            || origin.evidence.context_id() != universe.binding.source_context_id
            || origin.evidence.provenance() != ProvenanceClass::PlatformSource
            || origin.evidence.confidence() != EvidenceConfidence::Proven
            || origin.evidence.claim_scope() != ClaimScope::SourceObservation
            || origin.evidence.source_handle_ids() != [origin.source.handle_id()]
            || !origin.evidence.derivation_input_ids().is_empty()
            || origin.evidence.producer_version().to_string() != producer.producer_version()
        {
            return Err(mismatch());
        }
        let row = ReferenceRecordFacet {
            node,
            assertion: origin.assertion,
            partition_id: PARTITION,
            partition_coverage: partition.ok_or_else(missing)?.coverage(),
            record,
            record_digest,
            source: origin.source,
            evidence: origin.evidence,
            source_association: "exact_record_source_id_and_accepted_original_support",
            payload_interpretation: "original_canonical_reference_payload_no_context_parser",
            absence_authoritative: false,
        };
        budget.retain(&row, stop)?;
        result.push(row);
    }
    budget.work(
        result
            .len()
            .checked_mul(result.len().max(1).ilog2() as usize + 1)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    result.sort_by(|left, right| left.node.node_id().cmp(right.node.node_id()));
    if result
        .windows(2)
        .any(|rows| rows[0].node.node_id() == rows[1].node.node_id())
    {
        return Err(mismatch());
    }
    Ok(result)
}

fn text<'a>(proposal: &'a GraphEntityProposal, field: &str) -> ContextResult<&'a str> {
    match proposal.semantic_key().get(field) {
        Some(GraphProposalValue::String(value)) => Ok(value),
        _ => Err(mismatch()),
    }
}
fn mismatch() -> ContextError {
    ContextError::new(ContextErrorCode::SupportMismatch)
}
fn missing() -> ContextError {
    ContextError::new(ContextErrorCode::MissingRequiredCustody)
}
