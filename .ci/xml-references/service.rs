//! Surface static XML link problems through the ordinary diagnostic pipeline.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use crate::{ExactSourceLocation, GenericFinding, ServiceResult};
use wow_project::load::ProjectLoadPlan;
use wow_project::load::xml_references::XmlReferenceIssueKind;

pub(super) fn append_findings(
    plan: Option<&ProjectLoadPlan>,
    selected: &BTreeSet<String>,
    findings: &mut Vec<GenericFinding>,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    let Some(plan) = plan else { return Ok(()); };
    let report = plan.xml_references();
    for issue in report.issues() {
        super::cancelled(stop)?;
        let source = report.declarations().get(&issue.source_id)
            .ok_or_else(|| super::owner_error("XML link issue has no source declaration"))?;
        if !selected.contains(&source.document) { continue; }
        let original = plan.document_text(&source.document)
            .ok_or_else(|| super::owner_error("XML link issue has no captured document"))?;
        let start = usize::try_from(issue.span.byte_start)
            .map_err(|_| super::owner_error("XML link source range overflow"))?;
        let end = usize::try_from(issue.span.byte_end)
            .map_err(|_| super::owner_error("XML link source range overflow"))?;
        if original.get(start..end).is_none() {
            return Err(super::owner_error("XML link source range is invalid"));
        }
        let location = ExactSourceLocation::new(source.document.as_str(),
            source.content_digest.to_string(), issue.span.byte_start, issue.span.byte_end)?;
        let id = crate::identity::canonical_digest("service-xml-reference:sha256:",
            &(plan.digest(), report.digest(), issue))?;
        // Missing from this closure, dynamic names and forward references do not
        // prove broken client code. They describe the limits of static linking.
        let severity = match issue.kind {
            XmlReferenceIssueKind::NotInCapturedScope
            | XmlReferenceIssueKind::DynamicReference
            | XmlReferenceIssueKind::UnsupportedReference
            | XmlReferenceIssueKind::TargetAfterSource
            | XmlReferenceIssueKind::RepeatedLoad
            | XmlReferenceIssueKind::UnrecordedLoad => "information",
            _ => "warning",
        };
        findings.push(GenericFinding::new(id, "project.xml.references",
            issue.kind.code(), severity, location)?);
    }
    Ok(())
}
