//! Project the retained XML source index, not a runtime object hierarchy.
use super::*;
use crate::load::xml_references::{
    XmlReferenceKind, XmlReferenceOrder, XmlReferenceRecord, XmlReferenceResolution,
};
use crate::load::{ProjectLoadPlan, XmlSourceSpan};

pub(super) const MAX_DECLARATIONS: usize = 4096;
pub(super) const MAX_INHERITANCE_REFERENCES: usize = 8192;

/// Exact source occurrences stay distinct even when their display names coincide.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphXmlDeclaration {
    pub occurrence_id: String,
    pub path: String,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectGraphXmlReferenceOutcome {
    Projected { proposal_id: String },
    Unresolved,
    InvalidDeclaration,
    TargetNotTemplate,
    LoadOrderUnresolved,
    Cycle,
    SelfReference,
}

/// The reference ID addresses the full resolution/order/cycle record in the
/// retained load plan. An omitted graph edge is never an absent source reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphXmlReference {
    pub reference_id: String,
    pub outcome: ProjectGraphXmlReferenceOutcome,
}

pub(super) struct XmlProposals {
    pub entities: Vec<GraphEntityProposal>,
    pub relations: Vec<GraphRelationProposal>,
}

fn source_span(
    plan: &ProjectLoadPlan,
    path: &str,
    span: &XmlSourceSpan,
) -> ProjectResult<SourceSpan> {
    let text = plan.document_text(path).ok_or_else(invalid)?;
    let start = usize::try_from(span.byte_start).map_err(|_| invalid())?;
    let end = usize::try_from(span.byte_end).map_err(|_| invalid())?;
    // Validate UTF-8 boundaries as well as the half-open range before producing
    // evidence against the exact captured bytes. No filesystem access occurs.
    text.get(start..end).ok_or_else(invalid)?;
    SourceSpan::byte_range(span.byte_start, span.byte_end).map_err(|_| invalid())
}

/// Only unique, valid, previously loaded local templates can become inheritance
/// edges. Preserve every other outcome as a receipt, never pick a name winner.
fn target<'a>(
    plan: &'a ProjectLoadPlan,
    reference: &XmlReferenceRecord,
) -> ProjectResult<Result<&'a str, ProjectGraphXmlReferenceOutcome>> {
    let report = plan.xml_references();
    let source = report
        .declarations()
        .get(&reference.source_id)
        .ok_or_else(invalid)?;
    if !source.valid_declaration {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::InvalidDeclaration));
    }
    let declaration_id = match &reference.resolution {
        XmlReferenceResolution::UniqueLocalDeclaration { declaration_id } => declaration_id,
        XmlReferenceResolution::InvalidSource | XmlReferenceResolution::InvalidTarget { .. } => {
            return Ok(Err(ProjectGraphXmlReferenceOutcome::InvalidDeclaration));
        }
        _ => return Ok(Err(ProjectGraphXmlReferenceOutcome::Unresolved)),
    };
    let target = report
        .declarations()
        .get(declaration_id)
        .ok_or_else(invalid)?;
    if source.occurrence_id == target.occurrence_id {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::SelfReference));
    }
    if !target.valid_declaration {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::InvalidDeclaration));
    }
    if target.virtual_template != Some(true) && target.intrinsic != Some(true) {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::TargetNotTemplate));
    }
    if reference.cycle_id.is_some() {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::Cycle));
    }
    if reference.order != Some(XmlReferenceOrder::TargetBeforeSource) {
        return Ok(Err(ProjectGraphXmlReferenceOutcome::LoadOrderUnresolved));
    }
    Ok(Ok(target.occurrence_id.as_str()))
}

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<XmlProposals> {
    let mut output = XmlProposals {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    let Some(plan) = project.configuration().load_plan() else {
        return Ok(output);
    };
    let report = plan.xml_references();
    if report.declarations().len() > MAX_DECLARATIONS
        || report
            .references()
            .iter()
            .filter(|r| r.kind == XmlReferenceKind::Inherits)
            .count()
            > MAX_INHERITANCE_REFERENCES
    {
        return Err(exhausted());
    }
    let sources: BTreeMap<_, _> = plan
        .sources()
        .iter()
        .map(|s| (s.path.as_str(), s))
        .collect();
    let mut declaration_ids = BTreeMap::new();
    for (id, declaration) in report.declarations() {
        crate::analyzer::checkpoint(stop)?;
        let source = sources
            .get(declaration.document.as_str())
            .ok_or_else(invalid)?;
        if source.content_digest != declaration.content_digest || id != &declaration.occurrence_id {
            return Err(invalid());
        }
        let file_id = file_ids
            .get(declaration.document.as_str())
            .ok_or_else(invalid)?;
        charge(
            text_bytes,
            declaration.document.len().saturating_mul(4) + id.len().saturating_mul(8) + 256,
        )?;
        let span = source_span(plan, &declaration.document, &declaration.span)?;
        let (handle, evidence) = support(project, source, span, provenance)?;
        let proposal_id = format!("xml:{id}");
        let ownership_proposal_id = format!("xml-owner:{id}");
        output.entities.push(
            GraphEntityProposal::new(
                proposal_id.as_str(),
                "xml_source_declaration",
                BTreeMap::from([
                    (
                        "document".into(),
                        GraphProposalValue::String(declaration.document.clone().into()),
                    ),
                    (
                        "occurrence".into(),
                        GraphProposalValue::Identifier(id.clone().into()),
                    ),
                ]),
                GraphConfidence::Proven,
                vec![handle],
                vec![evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        output.relations.push(
            GraphRelationProposal::new(
                ownership_proposal_id.as_str(),
                "source_xml_owns",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Proposed(file_id.clone().into()),
                    target: GraphProposalEndpoint::Proposed(proposal_id.clone().into()),
                    confidence: GraphConfidence::Proven,
                    source_handle_ids: vec![handle],
                    evidence_ids: vec![evidence],
                    coverage_ids: Vec::new(),
                },
            )
            .map_err(|_| invalid())?,
        );
        declaration_ids.insert(id.as_str(), (proposal_id.clone(), handle, evidence));
        provenance
            .xml_declarations
            .push(ProjectGraphXmlDeclaration {
                occurrence_id: id.clone(),
                path: declaration.document.clone(),
                proposal_id,
                ownership_proposal_id,
                source_handle_id: handle,
                evidence_id: evidence,
            });
    }
    for reference in report
        .references()
        .iter()
        .filter(|r| r.kind == XmlReferenceKind::Inherits)
    {
        crate::analyzer::checkpoint(stop)?;
        charge(
            text_bytes,
            reference.reference_id.len().saturating_mul(4) + 128,
        )?;
        let outcome = match target(plan, reference)? {
            Err(outcome) => outcome,
            Ok(target_id) => {
                let declaration = report
                    .declarations()
                    .get(&reference.source_id)
                    .ok_or_else(invalid)?;
                let source = sources
                    .get(declaration.document.as_str())
                    .ok_or_else(invalid)?;
                let span = source_span(plan, &declaration.document, &reference.attribute_span)?;
                let (handle, evidence) = support(project, source, span, provenance)?;
                let proposal_id = format!("xml-inherits:{}", reference.reference_id);
                let source_node = declaration_ids
                    .get(reference.source_id.as_str())
                    .ok_or_else(invalid)?;
                let target_node = declaration_ids.get(target_id).ok_or_else(invalid)?;
                output.relations.push(
                    GraphRelationProposal::new(
                        proposal_id.as_str(),
                        "source_xml_inherits",
                        GraphRelationProposalInput {
                            source: GraphProposalEndpoint::Proposed(source_node.0.clone().into()),
                            target: GraphProposalEndpoint::Proposed(target_node.0.clone().into()),
                            confidence: GraphConfidence::Derived,
                            source_handle_ids: vec![handle, target_node.1],
                            evidence_ids: vec![evidence, target_node.2],
                            coverage_ids: Vec::new(),
                        },
                    )
                    .map_err(|_| invalid())?,
                );
                ProjectGraphXmlReferenceOutcome::Projected { proposal_id }
            }
        };
        provenance.xml_inheritance.push(ProjectGraphXmlReference {
            reference_id: reference.reference_id.clone(),
            outcome,
        });
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(output)
}
