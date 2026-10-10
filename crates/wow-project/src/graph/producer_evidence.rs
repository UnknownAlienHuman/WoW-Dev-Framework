//! Source support is checked against held native admission and accepted owners.
use super::*;
use platform_producers::graph_error;
use producer_budget::ProducerBudget;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphEvidenceCatalog,
    GraphLocalAssertion, GraphPartitionSnapshot,
};

pub(super) fn catalog(
    project: &ProjectView,
    owner: &GraphPartitionSnapshot,
    provenance: &ProjectGraphProvenance,
    scope: &GraphAssertionRecordScope,
    addresses: &BTreeMap<GraphLocalAssertion, GraphAssertionRef>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<GraphEvidenceCatalog> {
    crate::analyzer::checkpoint(stop)?;
    let lookup = owner.producer_lookup(stop).map_err(graph_error)?;
    if lookup.scope() != scope || &provenance.context != project.snapshot().generation_context() {
        return Err(invalid());
    }
    let inventory = owner
        .partition(PlatformGraphProducer::Inventory.partition_id())
        .ok_or_else(invalid)?;
    let mut files = BTreeMap::new();
    for file in &provenance.files {
        crate::analyzer::checkpoint(stop)?;
        if files.insert(file.path.as_str(), file).is_some() {
            return Err(invalid());
        }
        let source = project.source_artifact(&file.path)?.ok_or_else(invalid)?;
        if source.content_digest() != file.content_digest
            || source.byte_length() != file.byte_length
        {
            return Err(invalid());
        }
        let key = GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: file.proposal_id.clone().into(),
        };
        let reference = addresses.get(&key).ok_or_else(invalid)?;
        let resolved = lookup.entity(scope, reference, stop).map_err(graph_error)?;
        if resolved.partition().partition_id() != inventory.partition_id()
            || resolved.proposal().entity_kind_id() != "source_file"
            || resolved.proposal().semantic_key().get("path")
                != Some(&GraphProposalValue::String(file.path.clone().into()))
            || !resolved
                .proposal()
                .source_handle_ids()
                .contains(&file.source_handle_id)
            || !resolved
                .proposal()
                .evidence_ids()
                .contains(&file.evidence_id)
        {
            return Err(invalid());
        }
        let handle = provenance
            .source_handles
            .get(&file.source_handle_id)
            .ok_or_else(invalid)?;
        let evidence = provenance
            .evidence
            .get(&file.evidence_id)
            .ok_or_else(invalid)?;
        if handle.span().kind() != wow_core::SourceSpanKind::WholeFile
            || handle.path().as_str() != file.path
            || *handle.content_digest() != file.content_digest
            || !evidence
                .source_handle_ids()
                .contains(&file.source_handle_id)
        {
            return Err(invalid());
        }
    }
    if inventory
        .batch()
        .entity_proposals()
        .iter()
        .filter(|proposal| proposal.entity_kind_id() == "source_file")
        .count()
        != files.len()
    {
        return Err(invalid());
    }
    for handle in provenance.source_handles.values() {
        crate::analyzer::checkpoint(stop)?;
        if !files.contains_key(handle.path().as_str())
            || project.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned(),
            )? != *handle
        {
            return Err(invalid());
        }
    }
    // Assemble borrowed entries first; preflight the complete catalog envelope
    // before copying any record, handle or context into the retained catalog.
    let mut evidence = provenance
        .evidence
        .iter()
        .map(|(id, value)| (*id, value))
        .collect::<BTreeMap<_, _>>();
    let mut handles = provenance
        .source_handles
        .iter()
        .map(|(id, value)| (*id, value))
        .collect::<BTreeMap<_, _>>();
    if let Some(manifest) = &provenance.raw_inventory {
        let source = project
            .configuration()
            .platform_packages()
            .ok_or_else(invalid)?
            .source();
        let receipt = source.receipt();
        if manifest.source_snapshot_id() != receipt.source_snapshot_id()
            || manifest.inventory() != receipt.inventory()
            || manifest.admission_digest() != receipt.admission_digest()
            || manifest.profile_digest() != receipt.profile_digest()
            || manifest.content_manifest_digest() != receipt.content_manifest_digest()
            || manifest.coverage() != receipt.coverage()
            || manifest.members().len() != receipt.coverage().verified_files()
        {
            return Err(invalid());
        }
        let (origin, revision) = crate::registry::source_handle_identity(
            project.configuration(),
            project.project_generation(),
        )?;
        for record in manifest.members() {
            crate::analyzer::checkpoint(stop)?;
            let member = source.raw_member(&record.path, stop)?;
            let handle = SourceHandleBuilder::new(
                origin,
                project.configuration().source_origin_id().as_str(),
                revision.as_ref(),
                member.path(),
                SourceSpan::whole_file(),
                member.content_digest(),
            )
            .reference_generation(project.configuration().reference_generation())
            .project_generation(project.project_generation())
            .build()
            .map_err(|_| invalid())?;
            if record.kind != member.kind()
                || record.content_digest != member.content_digest()
                || record.byte_length != member.byte_length()
                || record.source_handle != handle
                || record.evidence.context_id() != scope.source_context_id
                || !record
                    .evidence
                    .source_handle_ids()
                    .contains(&handle.handle_id())
            {
                return Err(invalid());
            }
            let key = GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: record.proposal_id.clone().into(),
            };
            let reference = addresses.get(&key).ok_or_else(invalid)?;
            let resolved = lookup.entity(scope, reference, stop).map_err(graph_error)?;
            if resolved.partition().partition_id() != PLATFORM_RAW_INVENTORY_PARTITION
                || resolved.proposal().entity_kind_id() != PLATFORM_RAW_MEMBER_KIND
                || !resolved
                    .proposal()
                    .source_handle_ids()
                    .contains(&handle.handle_id())
                || !resolved
                    .proposal()
                    .evidence_ids()
                    .contains(&record.evidence.evidence_id())
            {
                return Err(invalid());
            }
            if let Some(previous) = handles.get(&handle.handle_id()) {
                if *previous != &handle {
                    return Err(invalid());
                }
            } else {
                handles.insert(handle.handle_id(), &record.source_handle);
            }
            if let Some(previous) = evidence.get(&record.evidence.evidence_id()) {
                if *previous != &record.evidence {
                    return Err(invalid());
                }
            } else {
                evidence.insert(record.evidence.evidence_id(), &record.evidence);
            }
        }
    }
    for (key, reference) in addresses {
        crate::analyzer::checkpoint(stop)?;
        let (handle_ids, evidence_ids) = match key.kind {
            GraphAssertionKind::Entity => {
                let resolved = lookup.entity(scope, reference, stop).map_err(graph_error)?;
                (
                    resolved.proposal().source_handle_ids(),
                    resolved.proposal().evidence_ids(),
                )
            }
            GraphAssertionKind::Relation => {
                let resolved = lookup
                    .relation(scope, reference, stop)
                    .map_err(graph_error)?;
                (
                    resolved.proposal().source_handle_ids(),
                    resolved.proposal().evidence_ids(),
                )
            }
        };
        if handle_ids.iter().any(|id| !handles.contains_key(id))
            || evidence_ids.iter().any(|id| !evidence.contains_key(id))
        {
            return Err(invalid());
        }
    }
    budget.charge_serialized(
        &(
            "graph-evidence-catalog/1",
            (&provenance.context, &evidence, &handles),
        ),
        stop,
    )?;
    let evidence = evidence
        .into_iter()
        .map(|(id, value)| (id, value.clone()))
        .collect();
    let handles = handles
        .into_iter()
        .map(|(id, value)| (id, value.clone()))
        .collect();
    crate::analyzer::checkpoint(stop)?;
    let catalog = GraphEvidenceCatalog::new(provenance.context.clone(), evidence, handles, stop)
        .map_err(graph_error)?;
    crate::analyzer::checkpoint(stop)?;
    Ok(catalog)
}
