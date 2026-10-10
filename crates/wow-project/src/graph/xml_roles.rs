//! Native XML lexical roles over retained indexes and original source support.
use super::load_inputs::LoadPlanGraphInput;
use super::producer_budget::ProducerBudget;
use super::*;
use crate::load::xml_references::{XmlReferenceKind, XmlReferenceResolution};
use crate::load::{
    LoadIssue, LoadIssueKind, LoadRecord, ProjectPackageLoadPhase, ProjectPackageReachability,
    XmlElementRecord, XmlElementRole, XmlStructureIssue,
};
use std::collections::BTreeSet;
use std::io::Write;
use std::sync::atomic::Ordering;
use wow_core::{CanonicalResult, ContentDigest, SourceContent, SourceSpanKind};

const MAX_ASSERTIONS: usize = 200_000;
const MAX_VALUE_BYTES: usize = 4096;
type OccurrenceKey<'a> = (&'a str, &'a str, Option<&'a str>, &'a str, &'a str);

#[derive(Clone, Copy, Serialize)]
struct Support {
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum LoadOutcome {
    MissingReceipt,
    RepeatedReceipt,
    Excluded,
    SelectionUnresolved,
    Unsupported,
    SourceOnly,
    IncludeCycle,
    RepeatedLoad,
    MissingFile,
    UnreachablePackage,
    TargetNotRegistered,
    SelfTarget,
    Projected,
}

#[derive(Serialize)]
struct LoadState<'a> {
    role: XmlElementRole,
    structure_issues: &'a [XmlStructureIssue],
    records: &'a [&'a LoadRecord],
    issues: &'a [&'a LoadIssue],
    #[serde(skip_serializing_if = "Option::is_none")]
    reachability: Option<ProjectPackageReachability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<ProjectPackageLoadPhase>,
    outcome: LoadOutcome,
}

pub(super) fn append(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    if project.configuration().project_kind() != ProjectKind::BlizzardUiPlatformSource
        || source.context() != project.snapshot().generation_context()
        || source.project_snapshot_id != project.snapshot_id()
        || source.analyzer_snapshot_id != project.analyzer_snapshot_id()
    {
        return Err(invalid());
    }
    if source.files().len() > MAX_FILES
        || source.source_handles().len() > MAX_ASSERTIONS
        || source.evidence().len() > MAX_ASSERTIONS
        || source.xml_containment().len() > 32_768
        || source.xml_facts().len() > 24_576
    {
        return Err(exhausted());
    }
    let rows = entities
        .len()
        .checked_add(relations.len())
        .filter(|count| *count <= MAX_ASSERTIONS)
        .ok_or_else(exhausted)?;
    let mut files = BTreeMap::new();
    for file in source.files() {
        crate::analyzer::checkpoint(stop)?;
        let artifact = project.source_artifact(&file.path)?.ok_or_else(invalid)?;
        if artifact.content_digest() != file.content_digest
            || artifact.byte_length() != file.byte_length
        {
            return Err(invalid());
        }
        validate_support(
            project,
            source,
            Support {
                handle: file.source_handle_id,
                evidence: file.evidence_id,
            },
            &file.path,
            SourceSpan::whole_file(),
            file.content_digest,
            stop,
        )?;
        budget.charge_serialized(
            &("xml-role-file-index", &file.path, &file.proposal_id),
            stop,
        )?;
        if files.insert(file.path.as_str(), file).is_some() {
            return Err(invalid());
        }
    }
    let mut spans = BTreeMap::new();
    let mut native_files = BTreeSet::new();
    for draft in entities.iter() {
        crate::analyzer::checkpoint(stop)?;
        let proposal = &draft.proposal;
        if proposal.entity_kind_id() == "source_file" {
            let Some(GraphProposalValue::String(path)) = proposal.semantic_key().get("path") else {
                return Err(invalid());
            };
            let file = files.get(path.as_ref()).ok_or_else(invalid)?;
            if draft.producer != PlatformGraphProducer::Inventory
                || proposal.proposal_id() != file.proposal_id
                || proposal.semantic_key().len() != 1
                || proposal.source_handle_ids() != [file.source_handle_id]
                || proposal.evidence_ids() != [file.evidence_id]
                || proposal.confidence() != GraphConfidence::Proven
            {
                return Err(invalid());
            }
            budget.charge_serialized(&("xml-role-native-file", path), stop)?;
            if !native_files.insert(path.as_ref()) {
                return Err(invalid());
            }
        } else if proposal.entity_kind_id() == "source_span" {
            let [handle_id] = proposal.source_handle_ids() else {
                return Err(invalid());
            };
            let [evidence_id] = proposal.evidence_ids() else {
                return Err(invalid());
            };
            let handle = source.source_handles().get(handle_id).ok_or_else(invalid)?;
            validate_support(
                project,
                source,
                Support {
                    handle: *handle_id,
                    evidence: *evidence_id,
                },
                handle.path().as_str(),
                handle.span(),
                *handle.content_digest(),
                stop,
            )?;
            budget.charge_serialized(
                &("xml-role-span-index", handle_id, proposal.proposal_id()),
                stop,
            )?;
            let canonical = handle_id.canonical();
            if draft.producer != PlatformGraphProducer::Inventory
                || proposal.confidence() != GraphConfidence::Proven
                || handle.span().kind() == SourceSpanKind::Unknown
                || proposal.semantic_key().len() != 1
                || !matches!(proposal.semantic_key().get("source_handle"),
                    Some(GraphProposalValue::String(value)) if value.as_ref() == canonical)
                || spans
                    .insert(*handle_id, Box::<str>::from(proposal.proposal_id()))
                    .is_some()
            {
                return Err(invalid());
            }
        }
    }
    if native_files.len() != files.len() {
        return Err(invalid());
    }
    // No borrowed address into the proposal Vec survives subsequent appends.
    drop(native_files);
    let mut containment = BTreeMap::new();
    for row in source.xml_containment() {
        crate::analyzer::checkpoint(stop)?;
        if row.context_id != source.context().context_id() {
            return Err(invalid());
        }
        budget.charge_serialized(&("xml-role-containment-index", &row.fact_id), stop)?;
        if containment
            .insert(key(&row.scope, &row.document, &row.occurrence_id), row)
            .is_some()
        {
            return Err(invalid());
        }
    }
    let mut facts = BTreeMap::<OccurrenceKey<'_>, Vec<&ProjectXmlFact>>::new();
    let mut fact_ids = BTreeSet::new();
    for fact in source.xml_facts() {
        crate::analyzer::checkpoint(stop)?;
        if fact.context_id != source.context().context_id() {
            return Err(invalid());
        }
        budget.charge_serialized(&("xml-role-fact-index", &fact.fact_id), stop)?;
        if !fact_ids.insert(fact.fact_id.as_str()) {
            return Err(invalid());
        }
        facts
            .entry(key(&fact.scope, &fact.document, &fact.occurrence_id))
            .or_default()
            .push(fact);
    }
    let mut output = Output {
        entities,
        relations,
        budget,
        rows,
        stop,
    };
    let mut visited_containment = 0usize;
    let mut visited_facts = 0usize;
    for input in load_inputs::scopes(project, stop)? {
        crate::analyzer::checkpoint(stop)?;
        let plan = input.plan();
        let toc = input.qualified_path(plan.selected_toc())?;
        let flavor = project.configuration().selected_profile().flavor_id();
        output
            .budget
            .charge_serialized(&("xml-role-scope", &toc, flavor, input.package()), stop)?;
        let scope = ProjectXmlFactScope {
            selected_toc: toc,
            flavor: flavor.to_owned(),
            package: input.package().map(str::to_owned),
        };
        let scope_text = canonical_text(&scope, output.budget, stop)?;
        let mut records = BTreeMap::<(&str, u64), Vec<&LoadRecord>>::new();
        for (ordinal, record) in plan.records().iter().enumerate() {
            crate::analyzer::checkpoint(stop)?;
            if record.ordinal != ordinal as u64 {
                return Err(invalid());
            }
            if plan.xml_documents().contains_key(&record.document)
                && matches!(
                    record.kind,
                    LoadRecordKind::LuaFile
                        | LoadRecordKind::XmlFile
                        | LoadRecordKind::XmlElement
                        | LoadRecordKind::Unknown
                )
            {
                output.budget.charge_serialized(
                    &(
                        "xml-role-load-index",
                        &record.document,
                        record.byte_end,
                        record.ordinal,
                    ),
                    stop,
                )?;
                records
                    .entry((&record.document, record.byte_end))
                    .or_default()
                    .push(record);
            }
        }
        let mut issues = BTreeMap::<(&str, u64), Vec<&LoadIssue>>::new();
        for issue in plan.issues() {
            crate::analyzer::checkpoint(stop)?;
            if plan.xml_documents().contains_key(&issue.document) {
                output.budget.charge_serialized(
                    &(
                        "xml-role-issue-index",
                        &issue.document,
                        issue.byte_start,
                        issue.byte_end,
                    ),
                    stop,
                )?;
                issues
                    .entry((&issue.document, issue.byte_end))
                    .or_default()
                    .push(issue);
            }
        }
        let mut references = BTreeMap::new();
        for reference in plan.xml_references().references() {
            crate::analyzer::checkpoint(stop)?;
            output
                .budget
                .charge_serialized(&("xml-role-reference-index", &reference.reference_id), stop)?;
            if references
                .insert(reference.reference_id.as_str(), reference)
                .is_some()
            {
                return Err(invalid());
            }
        }
        for (local, index) in plan.xml_documents() {
            crate::analyzer::checkpoint(stop)?;
            let mapped = input.mapped_source(local)?;
            let file = *files.get(mapped.path.as_str()).ok_or_else(invalid)?;
            let text = plan.document_text(local).ok_or_else(invalid)?;
            if index.document() != local
                || index.source_digest() != mapped.content_digest
                || file.content_digest != mapped.content_digest
                || file.byte_length != mapped.byte_length
                || text.len() as u64 != mapped.byte_length
                || crate::identity::source_digest(text.as_bytes()) != index.source_digest()
            {
                return Err(invalid());
            }
            let document_support = Support {
                handle: file.source_handle_id,
                evidence: file.evidence_id,
            };
            let mut document_key = output.base(&scope_text, &mapped.path, index.digest())?;
            output.field(
                &mut document_key,
                "content_digest",
                &mapped.content_digest.canonical(),
            )?;
            let document_id =
                output.entity("xml_source_document", document_key, document_support)?;
            output.relation(
                "xml_file_contains_document",
                &file.proposal_id,
                &document_id,
                &[document_support],
            )?;
            let mut occurrence_ids = BTreeMap::new();
            for element in index.elements() {
                crate::analyzer::checkpoint(stop)?;
                let row = *containment
                    .get(&key(&scope, &mapped.path, &element.occurrence_id))
                    .ok_or_else(invalid)?;
                let span = xml::source_span(plan, local, &element.span)?;
                if row.document_digest != index.digest()
                    || row.content_digest != mapped.content_digest
                    || row.element_name != element.qualified_name
                    || row.span != span
                    || row.parent_occurrence_id != element.parent_occurrence_id
                {
                    return Err(invalid());
                }
                let support = Support {
                    handle: row.source_handle_id,
                    evidence: row.evidence_id,
                };
                validate_support(
                    project,
                    source,
                    support,
                    &mapped.path,
                    span,
                    mapped.content_digest,
                    stop,
                )?;
                let mut occurrence_key = output.base(&scope_text, &mapped.path, index.digest())?;
                output.field(&mut occurrence_key, "occurrence", &element.occurrence_id)?;
                output.field(&mut occurrence_key, "role", role_name(element.role))?;
                output.field(
                    &mut occurrence_key,
                    "source_handle",
                    &support.handle.canonical(),
                )?;
                let occurrence_id =
                    output.entity("xml_source_occurrence", occurrence_key, support)?;
                output.relation(
                    "xml_document_contains_occurrence",
                    &document_id,
                    &occurrence_id,
                    &[support],
                )?;
                let span_id = spans.get(&support.handle).ok_or_else(invalid)?;
                output.relation(
                    "xml_occurrence_source_span",
                    &occurrence_id,
                    span_id,
                    &[support],
                )?;
                let mut script_count = 0usize;
                if let Some(element_facts) =
                    facts.get(&key(&scope, &mapped.path, &element.occurrence_id))
                {
                    for fact in element_facts {
                        crate::analyzer::checkpoint(stop)?;
                        if fact.document_digest != index.digest()
                            || fact.content_digest != mapped.content_digest
                            || fact.element_name != element.qualified_name
                        {
                            return Err(invalid());
                        }
                        let fact_support = Support {
                            handle: fact.source_handle_id,
                            evidence: fact.evidence_id,
                        };
                        validate_support(
                            project,
                            source,
                            fact_support,
                            &mapped.path,
                            fact.span,
                            mapped.content_digest,
                            stop,
                        )?;
                        match &fact.kind {
                            ProjectXmlFactKind::Script { .. } => {
                                validate_script(fact, element)?;
                                if fact.span != span {
                                    return Err(invalid());
                                }
                                script_count = script_count.checked_add(1).ok_or_else(exhausted)?;
                                let state = canonical_text(&fact.kind, output.budget, stop)?;
                                let mut semantic =
                                    output.base(&scope_text, &mapped.path, index.digest())?;
                                output.field(
                                    &mut semantic,
                                    "occurrence",
                                    &element.occurrence_id,
                                )?;
                                output.field(&mut semantic, "state", &state)?;
                                let id = output.entity(
                                    "xml_source_script_site",
                                    semantic,
                                    fact_support,
                                )?;
                                output.relation(
                                    "xml_occurrence_owns_script_site",
                                    &occurrence_id,
                                    &id,
                                    &[fact_support],
                                )?;
                            }
                            ProjectXmlFactKind::Parent {
                                reference_id,
                                name,
                                target_occurrence_id,
                                resolution,
                                order,
                                cycle_id,
                            } => {
                                let reference =
                                    references.get(reference_id.as_str()).ok_or_else(invalid)?;
                                let target = match &reference.resolution {
                                    XmlReferenceResolution::UniqueLocalDeclaration {
                                        declaration_id,
                                    } => Some(declaration_id.as_str()),
                                    _ => None,
                                };
                                if reference.kind != XmlReferenceKind::Parent
                                    || reference.source_id != element.occurrence_id
                                    || reference.name != *name
                                    || reference.resolution != *resolution
                                    || reference.order != *order
                                    || reference.cycle_id != *cycle_id
                                    || target != target_occurrence_id.as_deref()
                                    || fact.span
                                        != xml::source_span(plan, local, &reference.attribute_span)?
                                {
                                    return Err(invalid());
                                }
                                let state = canonical_text(&fact.kind, output.budget, stop)?;
                                let mut semantic =
                                    output.base(&scope_text, &mapped.path, index.digest())?;
                                output.field(
                                    &mut semantic,
                                    "occurrence",
                                    &element.occurrence_id,
                                )?;
                                output.field(&mut semantic, "reference_id", reference_id)?;
                                output.field(&mut semantic, "state", &state)?;
                                let id = output.entity(
                                    "xml_source_parent_reference",
                                    semantic,
                                    fact_support,
                                )?;
                                output.relation(
                                    "xml_occurrence_owns_parent_reference",
                                    &occurrence_id,
                                    &id,
                                    &[fact_support],
                                )?;
                            }
                            _ => {}
                        }
                        visited_facts = visited_facts.checked_add(1).ok_or_else(exhausted)?;
                    }
                }
                if script_count != usize::from(element.script.is_some()) {
                    return Err(invalid());
                }
                if matches!(
                    element.role,
                    XmlElementRole::Include | XmlElementRole::Script
                ) {
                    let site_records = records
                        .get(&(local.as_str(), element.start_tag_span.byte_end))
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    let site_issues = issues
                        .get(&(local.as_str(), element.start_tag_span.byte_end))
                        .map(Vec::as_slice)
                        .unwrap_or(&[]);
                    for record in site_records {
                        crate::analyzer::checkpoint(stop)?;
                        let start = usize::try_from(record.byte_start).map_err(|_| invalid())?;
                        let end = usize::try_from(record.byte_end).map_err(|_| invalid())?;
                        if record.byte_start > element.start_tag_span.byte_start
                            || crate::identity::source_digest(
                                text.get(start..end).ok_or_else(invalid)?.as_bytes(),
                            ) != record.raw_digest
                        {
                            return Err(invalid());
                        }
                    }
                    for issue in site_issues {
                        crate::analyzer::checkpoint(stop)?;
                        if issue.byte_start > element.start_tag_span.byte_start {
                            return Err(invalid());
                        }
                    }
                    let (outcome, target) =
                        load_target(&input, element, site_records, site_issues, &files, stop)?;
                    let state = canonical_text(
                        &LoadState {
                            role: element.role,
                            structure_issues: &element.issues,
                            records: site_records,
                            issues: site_issues,
                            reachability: input.node().map(|node| node.reachability),
                            phase: input.node().map(|node| node.phase),
                            outcome,
                        },
                        output.budget,
                        stop,
                    )?;
                    let mut semantic = output.base(&scope_text, &mapped.path, index.digest())?;
                    output.field(&mut semantic, "occurrence", &element.occurrence_id)?;
                    output.field(&mut semantic, "state", &state)?;
                    let id = output.entity("xml_source_load_site", semantic, support)?;
                    output.relation(
                        "xml_occurrence_owns_load_site",
                        &occurrence_id,
                        &id,
                        &[support],
                    )?;
                    if let Some(target) = target {
                        let relation = if element.role == XmlElementRole::Include {
                            "xml_include_target"
                        } else {
                            "xml_external_script_target"
                        };
                        output.relation(relation, &id, &target.proposal_id, &[support])?;
                    }
                }
                output.budget.charge_serialized(
                    &(
                        "xml-role-occurrence-address",
                        &element.occurrence_id,
                        &occurrence_id,
                    ),
                    stop,
                )?;
                if occurrence_ids
                    .insert(element.occurrence_id.as_str(), occurrence_id)
                    .is_some()
                {
                    return Err(invalid());
                }
                visited_containment = visited_containment.checked_add(1).ok_or_else(exhausted)?;
            }
            for element in index.elements() {
                crate::analyzer::checkpoint(stop)?;
                let Some(parent) = element.parent_occurrence_id.as_deref() else {
                    continue;
                };
                let row = containment
                    .get(&key(&scope, &mapped.path, &element.occurrence_id))
                    .ok_or_else(invalid)?;
                let parent_row = containment
                    .get(&key(&scope, &mapped.path, parent))
                    .ok_or_else(invalid)?;
                if parent == element.occurrence_id
                    || parent_row.span.byte_start() > row.span.byte_start()
                    || parent_row.span.byte_end() < row.span.byte_end()
                {
                    return Err(invalid());
                }
                output.relation(
                    "xml_lexical_contains",
                    occurrence_ids.get(parent).ok_or_else(invalid)?,
                    occurrence_ids
                        .get(element.occurrence_id.as_str())
                        .ok_or_else(invalid)?,
                    &[
                        Support {
                            handle: parent_row.source_handle_id,
                            evidence: parent_row.evidence_id,
                        },
                        Support {
                            handle: row.source_handle_id,
                            evidence: row.evidence_id,
                        },
                    ],
                )?;
            }
        }
    }
    if visited_containment != source.xml_containment().len()
        || visited_facts != source.xml_facts().len()
    {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)
}

fn key<'a>(
    scope: &'a ProjectXmlFactScope,
    document: &'a str,
    occurrence: &'a str,
) -> OccurrenceKey<'a> {
    (
        &scope.selected_toc,
        &scope.flavor,
        scope.package.as_deref(),
        document,
        occurrence,
    )
}

fn validate_support(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    support: Support,
    path: &str,
    span: SourceSpan,
    digest: ContentDigest<SourceContent>,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let handle = source
        .source_handles()
        .get(&support.handle)
        .ok_or_else(invalid)?;
    let evidence = source
        .evidence()
        .get(&support.evidence)
        .ok_or_else(invalid)?;
    evidence.validate().map_err(|_| invalid())?;
    if handle.handle_id() != support.handle
        || handle.path().as_str() != path
        || handle.span() != span
        || *handle.content_digest() != digest
        || evidence.evidence_id() != support.evidence
        || evidence.source_handle_ids() != [support.handle]
        || evidence.context_id() != source.context().context_id()
        || evidence.provenance() != ProvenanceClass::ProjectSource
        || evidence.confidence() != EvidenceConfidence::Proven
        || evidence.claim_scope() != ClaimScope::SourceObservation
        || project.source_handle(path, span, handle.entity_key().cloned())? != *handle
    {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)
}

fn validate_script(fact: &ProjectXmlFact, element: &XmlElementRecord) -> ProjectResult<()> {
    let script = element.script.as_ref().ok_or_else(invalid)?;
    let ProjectXmlFactKind::Script {
        script_name,
        source_kind,
        owner_occurrence_id,
        inherit,
        intrinsic_order,
        file_reference,
        function_reference,
        method_reference,
        inline_unit_id,
        inline_content_digest,
    } = &fact.kind
    else {
        return Err(invalid());
    };
    if *script_name != element.qualified_name
        || *source_kind != script.source_kind
        || *owner_occurrence_id != script.owner_occurrence_id
        || *inherit != script.inherit
        || *intrinsic_order != script.intrinsic_order
        || *file_reference != script.file_reference
        || *function_reference != script.function_reference
        || *method_reference != script.method_reference
        || inline_unit_id.as_deref() != script.inline_lua.as_ref().map(|body| body.unit_id.as_str())
        || *inline_content_digest != script.inline_lua.as_ref().map(|body| body.content_digest)
    {
        return Err(invalid());
    }
    Ok(())
}

fn load_target<'a>(
    input: &LoadPlanGraphInput<'_>,
    element: &XmlElementRecord,
    records: &[&LoadRecord],
    issues: &[&LoadIssue],
    files: &BTreeMap<&str, &'a ProjectGraphFile>,
    stop: &AtomicBool,
) -> ProjectResult<(LoadOutcome, Option<&'a ProjectGraphFile>)> {
    crate::analyzer::checkpoint(stop)?;
    let [record] = records else {
        return Ok((
            if records.is_empty() {
                LoadOutcome::MissingReceipt
            } else {
                LoadOutcome::RepeatedReceipt
            },
            None,
        ));
    };
    if record.selection != LoadSelection::Included {
        return Ok((
            if record.selection == LoadSelection::Excluded {
                LoadOutcome::Excluded
            } else {
                LoadOutcome::SelectionUnresolved
            },
            None,
        ));
    }
    for (kind, outcome) in [
        (LoadIssueKind::IncludeCycle, LoadOutcome::IncludeCycle),
        (LoadIssueKind::RepeatedLoad, LoadOutcome::RepeatedLoad),
        (LoadIssueKind::MissingFile, LoadOutcome::MissingFile),
        (
            LoadIssueKind::UnsupportedXmlAttributes,
            LoadOutcome::Unsupported,
        ),
        (LoadIssueKind::UnsupportedFileKind, LoadOutcome::Unsupported),
    ] {
        for issue in issues {
            crate::analyzer::checkpoint(stop)?;
            if issue.kind == kind {
                return Ok((outcome, None));
            }
        }
    }
    if !element.issues.is_empty() {
        return Ok((LoadOutcome::Unsupported, None));
    }
    let expected = if element.role == XmlElementRole::Include {
        LoadRecordKind::XmlFile
    } else {
        LoadRecordKind::LuaFile
    };
    if record.kind != expected {
        return Ok((
            if element.script.is_some() {
                LoadOutcome::SourceOnly
            } else {
                LoadOutcome::Unsupported
            },
            None,
        ));
    }
    if input
        .node()
        .is_some_and(|node| node.reachability == ProjectPackageReachability::Unreachable)
    {
        return Ok((LoadOutcome::UnreachablePackage, None));
    }
    let Some(target) = record.target.as_deref() else {
        return Ok((LoadOutcome::Unsupported, None));
    };
    if target == record.document {
        return Ok((LoadOutcome::SelfTarget, None));
    }
    if !input
        .plan()
        .sources()
        .iter()
        .any(|source| source.path == target)
    {
        return Ok((LoadOutcome::TargetNotRegistered, None));
    }
    let path = input.qualified_path(target)?;
    let Some(file) = files.get(path.as_str()) else {
        return Ok((LoadOutcome::TargetNotRegistered, None));
    };
    crate::analyzer::checkpoint(stop)?;
    Ok((LoadOutcome::Projected, Some(*file)))
}

fn role_name(role: XmlElementRole) -> &'static str {
    match role {
        XmlElementRole::Ui => "ui",
        XmlElementRole::Include => "include",
        XmlElementRole::Script => "script",
        XmlElementRole::Scripts => "scripts",
        XmlElementRole::ScriptBinding => "script_binding",
        XmlElementRole::Element => "element",
        XmlElementRole::UnknownNamespace => "unknown_namespace",
    }
}

struct Output<'a> {
    entities: &'a mut Vec<EntityDraft>,
    relations: &'a mut Vec<RelationDraft>,
    budget: &'a mut ProducerBudget,
    rows: usize,
    stop: &'a AtomicBool,
}
impl Output<'_> {
    fn base(
        &mut self,
        scope: &str,
        document: &str,
        digest: ContentDigest<CanonicalResult>,
    ) -> ProjectResult<BTreeMap<Box<str>, GraphProposalValue>> {
        let mut key = BTreeMap::new();
        self.field(&mut key, "scope", scope)?;
        self.field(&mut key, "document", document)?;
        self.field(&mut key, "document_digest", &digest.canonical())?;
        Ok(key)
    }

    fn field(
        &mut self,
        key: &mut BTreeMap<Box<str>, GraphProposalValue>,
        field: &'static str,
        value: &str,
    ) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        if value.len() > MAX_VALUE_BYTES {
            return Err(exhausted());
        }
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(invalid());
        }
        self.budget.charge_serialized(&(field, value), self.stop)?;
        if key
            .insert(field.into(), GraphProposalValue::String(value.into()))
            .is_some()
        {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(self.stop)
    }

    fn entity(
        &mut self,
        kind: &'static str,
        key: BTreeMap<Box<str>, GraphProposalValue>,
        support: Support,
    ) -> ProjectResult<Box<str>> {
        self.next()?;
        self.budget
            .charge_serialized(&(kind, &key, support), self.stop)?;
        let id = crate::identity::canonical_id(
            "xml-role:",
            "wow-project/platform-xml-role-entity/1",
            &(kind, &key),
            ProjectPhase::View,
        )?;
        self.budget
            .charge_serialized(&("xml-role-address", &id), self.stop)?;
        let draft = EntityDraft::new(
            PlatformGraphProducer::XmlStructure,
            GraphEntityProposal::new(
                id.clone(),
                kind,
                key,
                GraphConfidence::Proven,
                vec![support.handle],
                vec![support.evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        self.budget.charge_serialized(&draft, self.stop)?;
        crate::analyzer::checkpoint(self.stop)?;
        self.entities.push(draft);
        Ok(id)
    }

    fn relation(
        &mut self,
        kind: &'static str,
        from: &str,
        to: &str,
        supports: &[Support],
    ) -> ProjectResult<()> {
        self.next()?;
        self.budget
            .charge_serialized(&(kind, from, to, supports), self.stop)?;
        let id = crate::identity::canonical_id(
            "xml-role-edge:",
            "wow-project/platform-xml-role-relation/1",
            &(kind, from, to),
            ProjectPhase::View,
        )?;
        let draft = RelationDraft::new(
            PlatformGraphProducer::XmlStructure,
            id,
            kind,
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed(from.into()),
                target: GraphProposalEndpoint::Proposed(to.into()),
                confidence: GraphConfidence::Proven,
                source_handle_ids: supports.iter().map(|support| support.handle).collect(),
                evidence_ids: supports.iter().map(|support| support.evidence).collect(),
                coverage_ids: Vec::new(),
            },
        )
        .map_err(|_| invalid())?;
        self.budget.charge_serialized(&draft, self.stop)?;
        crate::analyzer::checkpoint(self.stop)?;
        self.relations.push(draft);
        Ok(())
    }

    fn next(&mut self) -> ProjectResult<()> {
        crate::analyzer::checkpoint(self.stop)?;
        self.rows = self
            .rows
            .checked_add(1)
            .filter(|count| *count <= MAX_ASSERTIONS)
            .ok_or_else(exhausted)?;
        Ok(())
    }
}

pub(super) fn canonical_text<T: Serialize>(
    value: &T,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Box<str>> {
    struct Counter<'a> {
        bytes: usize,
        exceeded: bool,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(Ordering::Acquire) {
                return Err(std::io::Error::other("cancelled"));
            }
            let Some(total) = self
                .bytes
                .checked_add(bytes.len())
                .filter(|total| *total <= MAX_VALUE_BYTES)
            else {
                self.exceeded = true;
                return Err(std::io::Error::other("XML role state limit"));
            };
            self.bytes = total;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    crate::analyzer::checkpoint(stop)?;
    let mut counter = Counter {
        bytes: 0,
        exceeded: false,
        stop,
    };
    let encoded = serde_json::to_writer(&mut counter, value);
    crate::analyzer::checkpoint(stop)?;
    if encoded.is_err() {
        return Err(if counter.exceeded {
            exhausted()
        } else {
            invalid()
        });
    }
    budget.charge_serialized(&("xml-role-canonical-state", value), stop)?;
    let text = wow_core::canonical_json_string(value).map_err(|_| invalid())?;
    if text.len() > MAX_VALUE_BYTES {
        return Err(exhausted());
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(text.into_boxed_str())
}
