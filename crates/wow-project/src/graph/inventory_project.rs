//! Snapshot-bound Inventory roles over already retained native source receipts.
use super::producer_budget::ProducerBudget;
use super::*;
use crate::platform_source::PlatformEntryDisposition;
use wow_core::SourceSpanKind;

const MAX_ASSERTIONS: usize = 200_000;
const MAX_RAW_MEMBERS: usize = 4096;

pub(super) fn append_project(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    if config.project_kind() != ProjectKind::BlizzardUiPlatformSource
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
    let packages = config.platform_packages().ok_or_else(invalid)?;
    let admitted = packages.source();
    let receipt = admitted.receipt();
    let manifest = source.raw_inventory().ok_or_else(invalid)?;
    let load = packages.load_plan();
    let main = packages.main_plan();
    packages.binding().validate(admitted, load, main)?;
    crate::analyzer::checkpoint(stop)?;
    if source.package_load_plan.as_ref() != Some(load)
        || source.package_main_plan.as_ref() != Some(main)
        || manifest.source_snapshot_id() != receipt.source_snapshot_id()
        || manifest.profile_digest() != receipt.profile_digest()
        || manifest.content_manifest_digest() != receipt.content_manifest_digest()
        || manifest.admission_digest() != receipt.admission_digest()
        || manifest.inventory() != receipt.inventory()
        || manifest.coverage() != receipt.coverage()
        || manifest.members().len() != receipt.coverage().verified_files()
        || source.packages().len() != load.packages().len()
        || packages.package_inputs().len() != load.packages().len()
    {
        return Err(invalid());
    }
    if manifest.members().len() > MAX_RAW_MEMBERS
        || source.files().len() > MAX_FILES
        || source.packages().len() > packages::MAX_PACKAGE_NODES
    {
        return Err(exhausted());
    }

    // Original raw paths never become package-qualified Main coordinates.
    let mut raw_by_path = BTreeMap::new();
    let mut verified_bytes = 0_u64;
    for member in manifest.members() {
        crate::analyzer::checkpoint(stop)?;
        let entry = receipt
            .inventory()
            .entries
            .binary_search_by(|entry| entry.path.as_str().cmp(member.path.as_str()))
            .ok()
            .and_then(|index| receipt.inventory().entries.get(index))
            .ok_or_else(invalid)?;
        let PlatformEntryDisposition::Included {
            digest,
            byte_length,
            ..
        } = &entry.disposition
        else {
            return Err(invalid());
        };
        validate_support(project, source, &member.source_handle, &member.evidence)?;
        if entry.kind != member.kind
            || *digest != member.content_digest
            || *byte_length != member.byte_length
            || member.source_handle.path().as_str() != member.path
            || *member.source_handle.content_digest() != member.content_digest
            || member.source_handle.span().kind() != SourceSpanKind::WholeFile
            || member.source_handle.entity_key().is_some()
        {
            return Err(invalid());
        }
        let expected = crate::identity::canonical_digest(
            "wow-project/platform-raw-member/1",
            &(receipt.source_snapshot_id(), member.path.as_str()),
            ProjectPhase::View,
        )?;
        if member.proposal_id != format!("raw-member:{expected}") {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "inventory-project-raw-index",
                member.path.as_str(),
                member.proposal_id.as_str(),
                member.source_handle.handle_id(),
                member.evidence.evidence_id(),
            ),
            stop,
        )?;
        if raw_by_path.insert(member.path.as_str(), member).is_some() {
            return Err(invalid());
        }
        verified_bytes = verified_bytes
            .checked_add(member.byte_length)
            .ok_or_else(exhausted)?;
    }
    if verified_bytes != receipt.coverage().verified_bytes() {
        return Err(invalid());
    }
    // A real byte witness anchors support, not inventory completeness.
    let witness = raw_by_path.first_key_value().ok_or_else(invalid)?.1;
    let project_support = (
        witness.source_handle.handle_id(),
        witness.evidence.evidence_id(),
    );

    let mut files = BTreeMap::new();
    for file in source.files() {
        crate::analyzer::checkpoint(stop)?;
        captured_support(project, source, file)?;
        budget.charge_serialized(
            &(
                "inventory-project-file-index",
                file.path.as_str(),
                file.proposal_id.as_str(),
            ),
            stop,
        )?;
        if files.insert(file.path.as_str(), file).is_some() {
            return Err(invalid());
        }
    }

    let mut package_roots = BTreeMap::new();
    for input in packages.package_inputs() {
        crate::analyzer::checkpoint(stop)?;
        crate::disk::validate_path(input.root())?;
        let package = source
            .packages()
            .iter()
            .find(|package| package.package == input.name())
            .ok_or_else(invalid)?;
        let selected = load.package_plan(input.name()).ok_or_else(invalid)?;
        let path = load
            .source_path(input.name(), selected.selected_toc())
            .ok_or_else(invalid)?;
        let file = files.get(path.as_str()).ok_or_else(invalid)?;
        if package.selected_toc != path
            || package.source_handle_id != file.source_handle_id
            || package.evidence_id != file.evidence_id
            || !load
                .packages()
                .iter()
                .any(|node| node.package == input.name())
        {
            return Err(invalid());
        }
        budget.charge_serialized(
            &(
                "inventory-project-package-index",
                input.name(),
                input.root(),
                package.proposal_id.as_str(),
            ),
            stop,
        )?;
        if package_roots
            .insert(input.name(), (input.root(), package))
            .is_some()
        {
            return Err(invalid());
        }
    }

    // Native load/Main identity does not include physical package roots. Bind
    // the original request here without changing the older owner contracts.
    let package_authority = (packages.binding().binding_digest(), packages.request());
    budget.charge_serialized(&package_authority, stop)?;
    let package_authority_digest = crate::identity::canonical_digest(
        "wow-project/platform-source-project-package-authority/1",
        &package_authority,
        ProjectPhase::View,
    )?;
    crate::analyzer::checkpoint(stop)?;
    budget.charge_serialized(
        &(
            "inventory-project-identity",
            config.project_id(),
            project.snapshot_id(),
            receipt.source_snapshot_id(),
            receipt.profile_digest(),
            receipt.content_manifest_digest(),
            receipt.admission_digest(),
            package_authority_digest,
        ),
        stop,
    )?;
    let semantic_key = BTreeMap::from([
        (
            "project".into(),
            GraphProposalValue::String(config.project_id().as_str().into()),
        ),
        (
            "project_snapshot".into(),
            GraphProposalValue::String(project.snapshot_id().into()),
        ),
        (
            "source_snapshot".into(),
            GraphProposalValue::String(receipt.source_snapshot_id().into()),
        ),
        (
            "profile_digest".into(),
            GraphProposalValue::String(receipt.profile_digest().to_string().into()),
        ),
        (
            "content_manifest_digest".into(),
            GraphProposalValue::String(receipt.content_manifest_digest().to_string().into()),
        ),
        (
            "admission_digest".into(),
            GraphProposalValue::String(receipt.admission_digest().to_string().into()),
        ),
        (
            "package_binding".into(),
            GraphProposalValue::String(package_authority_digest.to_string().into()),
        ),
    ]);
    reserve_row(&mut rows)?;
    let project_id = crate::identity::canonical_id(
        "project:",
        "wow-project/platform-inventory-project/1",
        &semantic_key,
        ProjectPhase::View,
    )?;
    let entity = EntityDraft::new(
        PlatformGraphProducer::Inventory,
        GraphEntityProposal::new(
            project_id.clone(),
            "source_project",
            semantic_key,
            GraphConfidence::Proven,
            vec![project_support.0],
            vec![project_support.1],
            Vec::new(),
        )
        .map_err(|_| invalid())?,
    );
    budget.charge_serialized(&entity, stop)?;
    crate::analyzer::checkpoint(stop)?;
    entities.push(entity);

    for (_, package) in package_roots.values() {
        append_containment(
            "source_project_contains_package",
            (&project_id, package.proposal_id.as_str()),
            (
                project_support,
                (package.source_handle_id, package.evidence_id),
            ),
            relations,
            budget,
            &mut rows,
            stop,
        )?;
    }
    for file in files.values() {
        append_containment(
            "source_project_contains_file",
            (&project_id, file.proposal_id.as_str()),
            (project_support, (file.source_handle_id, file.evidence_id)),
            relations,
            budget,
            &mut rows,
            stop,
        )?;
    }
    for member in raw_by_path.values() {
        crate::analyzer::checkpoint(stop)?;
        let raw_support = (
            member.source_handle.handle_id(),
            member.evidence.evidence_id(),
        );
        append_containment(
            "source_project_contains_raw_member",
            (&project_id, member.proposal_id.as_str()),
            (project_support, raw_support),
            relations,
            budget,
            &mut rows,
            stop,
        )?;
        // Every declared directory match remains a membership. Unmatched raw
        // members still have their project edge; neither case proves coverage.
        for (root, package) in package_roots.values() {
            crate::analyzer::checkpoint(stop)?;
            if !member
                .path
                .strip_prefix(*root)
                .is_some_and(|tail| tail.starts_with('/'))
            {
                continue;
            }
            append_containment(
                "source_package_contains_raw_member",
                (package.proposal_id.as_str(), member.proposal_id.as_str()),
                ((package.source_handle_id, package.evidence_id), raw_support),
                relations,
                budget,
                &mut rows,
                stop,
            )?;
        }
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(())
}

fn validate_support(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    handle: &SourceHandle,
    evidence: &EvidenceRecord,
) -> ProjectResult<()> {
    handle.validate().map_err(|_| invalid())?;
    evidence.validate().map_err(|_| invalid())?;
    let config = project.configuration();
    let (origin, revision) =
        crate::registry::source_handle_identity(config, project.project_generation())?;
    if handle.origin_kind() != origin
        || handle.origin_id() != config.source_origin_id().as_str()
        || handle.revision() != revision.as_ref()
        || handle.reference_generation() != Some(config.reference_generation())
        || handle.project_generation() != Some(project.project_generation())
        || evidence.context_id() != source.context().context_id()
        || evidence.provenance() != ProvenanceClass::ProjectSource
        || evidence.confidence() != EvidenceConfidence::Proven
        || evidence.claim_scope() != ClaimScope::SourceObservation
        || evidence.source_handle_ids() != [handle.handle_id()]
    {
        return Err(invalid());
    }
    Ok(())
}

fn captured_support(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    file: &ProjectGraphFile,
) -> ProjectResult<()> {
    let handle = source
        .source_handles()
        .get(&file.source_handle_id)
        .ok_or_else(invalid)?;
    let evidence = source
        .evidence()
        .get(&file.evidence_id)
        .ok_or_else(invalid)?;
    validate_support(project, source, handle, evidence)?;
    let artifact = project.source_artifact(&file.path)?.ok_or_else(invalid)?;
    if handle.handle_id() != file.source_handle_id
        || evidence.evidence_id() != file.evidence_id
        || handle.path().as_str() != file.path
        || handle.span().kind() != SourceSpanKind::WholeFile
        || *handle.content_digest() != file.content_digest
        || artifact.content_digest() != file.content_digest
        || artifact.byte_length() != file.byte_length
    {
        return Err(invalid());
    }
    Ok(())
}

fn reserve_row(rows: &mut usize) -> ProjectResult<()> {
    *rows = rows
        .checked_add(1)
        .filter(|count| *count <= MAX_ASSERTIONS)
        .ok_or_else(exhausted)?;
    Ok(())
}

fn append_containment(
    kind: &'static str,
    endpoints: (&str, &str),
    support: ((StableHandleId, EvidenceId), (StableHandleId, EvidenceId)),
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    rows: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    reserve_row(rows)?;
    let domain = match kind {
        "source_project_contains_package" => "wow-project/source_project_contains_package/1",
        "source_project_contains_file" => "wow-project/source_project_contains_file/1",
        "source_project_contains_raw_member" => "wow-project/source_project_contains_raw_member/1",
        "source_package_contains_raw_member" => "wow-project/source_package_contains_raw_member/1",
        _ => return Err(invalid()),
    };
    let mut handles = vec![support.0.0, support.1.0];
    handles.sort_unstable();
    handles.dedup();
    let mut evidence = vec![support.0.1, support.1.1];
    evidence.sort_unstable();
    evidence.dedup();
    let relation = RelationDraft::new(
        PlatformGraphProducer::Inventory,
        crate::identity::canonical_id("contains:", domain, &endpoints, ProjectPhase::View)?,
        kind,
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed(endpoints.0.into()),
            target: GraphProposalEndpoint::Proposed(endpoints.1.into()),
            confidence: GraphConfidence::Proven,
            source_handle_ids: handles,
            evidence_ids: evidence,
            coverage_ids: Vec::new(),
        },
    )
    .map_err(|_| invalid())?;
    budget.charge_serialized(&relation, stop)?;
    crate::analyzer::checkpoint(stop)?;
    relations.push(relation);
    Ok(())
}
