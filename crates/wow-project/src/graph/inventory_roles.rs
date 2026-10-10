//! Exact captured support spans for the additive native Inventory recipe.
use super::producer_budget::ProducerBudget;
use super::*;
use wow_core::SourceSpanKind;

// Match the existing combined direct-producer assertion ceiling.
const MAX_ASSERTIONS: usize = 200_000;

pub(super) fn append_spans(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<usize> {
    crate::analyzer::checkpoint(stop)?;
    if project.configuration().project_kind() != ProjectKind::BlizzardUiPlatformSource
        || source.context() != project.snapshot().generation_context()
        || source.project_snapshot_id != project.snapshot_id()
        || source.analyzer_snapshot_id != project.analyzer_snapshot_id()
    {
        return Err(invalid());
    }
    let mut rows = entities
        .len()
        .checked_add(relations.len())
        .filter(|count| *count <= MAX_ASSERTIONS)
        .ok_or_else(exhausted)?;
    if source.files().len() > MAX_ASSERTIONS
        || source.source_handles().len() > MAX_ASSERTIONS
        || source.evidence().len() > MAX_ASSERTIONS
    {
        return Err(exhausted());
    }

    // Borrow the existing file addresses; do not recapture or project source.
    let mut files = BTreeMap::new();
    for file in source.files() {
        crate::analyzer::checkpoint(stop)?;
        let artifact = project.source_artifact(&file.path)?.ok_or_else(invalid)?;
        let handle = source
            .source_handles()
            .get(&file.source_handle_id)
            .ok_or_else(invalid)?;
        if artifact.content_digest() != file.content_digest
            || artifact.byte_length() != file.byte_length
            || handle.handle_id() != file.source_handle_id
            || handle.span().kind() != SourceSpanKind::WholeFile
            || handle.path().as_str() != file.path
            || *handle.content_digest() != file.content_digest
        {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "inventory-span-file-index",
                file.path.as_str(),
                file.proposal_id.as_str(),
            ),
            stop,
        )?;
        if files.insert(file.path.as_str(), file).is_some() {
            return Err(invalid());
        }
    }

    // Captured support() records have exactly one handle and one native context.
    let mut evidence_by_handle = BTreeMap::new();
    for (evidence_id, evidence) in source.evidence() {
        crate::analyzer::checkpoint(stop)?;
        evidence.validate().map_err(|_| invalid())?;
        let [handle_id] = evidence.source_handle_ids() else {
            return Err(invalid());
        };
        if evidence.evidence_id() != *evidence_id
            || evidence.context_id() != source.context().context_id()
            || evidence.provenance() != ProvenanceClass::ProjectSource
            || evidence.confidence() != EvidenceConfidence::Proven
            || evidence.claim_scope() != ClaimScope::SourceObservation
            || !source.source_handles().contains_key(handle_id)
        {
            return Err(invalid());
        }
        budget.charge_serialized(
            &("inventory-span-evidence-index", handle_id, evidence_id),
            stop,
        )?;
        if evidence_by_handle
            .insert(*handle_id, *evidence_id)
            .is_some()
        {
            return Err(invalid());
        }
    }
    for file in files.values() {
        crate::analyzer::checkpoint(stop)?;
        if evidence_by_handle.get(&file.source_handle_id) != Some(&file.evidence_id) {
            return Err(invalid());
        }
    }

    let mut omitted = 0_usize;
    for (handle_id, handle) in source.source_handles() {
        crate::analyzer::checkpoint(stop)?;
        let file = files.get(handle.path().as_str()).ok_or_else(invalid)?;
        let evidence_id = *evidence_by_handle.get(handle_id).ok_or_else(invalid)?;
        if handle.handle_id() != *handle_id
            || *handle.content_digest() != file.content_digest
            || project.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned(),
            )? != *handle
        {
            return Err(invalid());
        }
        if handle.span().kind() == SourceSpanKind::Unknown {
            omitted = omitted.checked_add(1).ok_or_else(exhausted)?;
            continue;
        }
        rows = rows
            .checked_add(2)
            .filter(|count| *count <= MAX_ASSERTIONS)
            .ok_or_else(exhausted)?;

        let span_id = crate::identity::canonical_id(
            "span:",
            "wow-project/platform-inventory-source-span/1",
            handle_id,
            ProjectPhase::View,
        )?;
        let entity = EntityDraft::new(
            PlatformGraphProducer::Inventory,
            GraphEntityProposal::new(
                span_id.clone(),
                "source_span",
                BTreeMap::from([(
                    "source_handle".into(),
                    GraphProposalValue::String(handle_id.canonical().into()),
                )]),
                GraphConfidence::Proven,
                vec![*handle_id],
                vec![evidence_id],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        let relation = RelationDraft::new(
            PlatformGraphProducer::Inventory,
            crate::identity::canonical_id(
                "file-span:",
                "wow-project/platform-inventory-file-span-containment/1",
                &(file.proposal_id.as_str(), handle_id),
                ProjectPhase::View,
            )?,
            "source_file_contains_span",
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed(file.proposal_id.as_str().into()),
                target: GraphProposalEndpoint::Proposed(span_id),
                confidence: GraphConfidence::Proven,
                source_handle_ids: vec![*handle_id],
                evidence_ids: vec![evidence_id],
                coverage_ids: Vec::new(),
            },
        )
        .map_err(|_| invalid())?;
        budget.charge_serialized(&entity, stop)?;
        budget.charge_serialized(&relation, stop)?;
        crate::analyzer::checkpoint(stop)?;
        entities.push(entity);
        relations.push(relation);
    }
    budget.charge_serialized(&("inventory-span-omissions", omitted), stop)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(omitted)
}
