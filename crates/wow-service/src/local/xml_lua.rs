//! Project-owned XML units enter normal findings without masquerading as Main
//! physical files. Owner reports stay whole-project; presentation obeys scope.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use crate::{
    CheckScope, ExactSourceLocation, GenericFinding, ServiceError, ServiceErrorCode, ServiceResult,
};
use wow_project::xml_lua::ProjectXmlLuaAnalysis;
use wow_project::{ProjectFileId, ProjectFileRecord, ProjectView};

pub(super) struct ResolvedScope<'a> {
    pub physical: Vec<&'a ProjectFileRecord>,
    pub xml_documents: BTreeSet<String>,
}
pub(super) fn resolve_scope<'a>(
    project: &'a ProjectView,
    scope: &CheckScope,
) -> ServiceResult<ResolvedScope<'a>> {
    let plan = project.configuration().load_plan();
    if matches!(scope, CheckScope::WholeProject) {
        return Ok(ResolvedScope {
            physical: project.file_manifest().iter().collect(),
            xml_documents: plan
                .map(|plan| plan.xml_documents().keys().cloned().collect())
                .unwrap_or_default(),
        });
    }
    let paths = match scope {
        CheckScope::ProjectFiles(ids) => ids
            .iter()
            .map(|id| {
                let id = ProjectFileId::parse(id).map_err(|_| invalid_scope())?;
                id.as_str()
                    .strip_prefix("project-file:")
                    .map(str::to_owned)
                    .ok_or_else(invalid_scope)
            })
            .collect::<ServiceResult<Vec<_>>>()?,
        CheckScope::Files(paths) => paths.iter().map(|path| path.to_string()).collect(),
        CheckScope::WholeProject => return Err(invalid_scope()),
    };
    if paths.is_empty() {
        return Err(invalid_scope());
    }
    let mut physical = Vec::new();
    let mut xml_documents = BTreeSet::new();
    for path in paths {
        if let Some(file) = project.file_by_path(&path).map_err(|_| invalid_scope())? {
            physical.push(file);
        } else if plan.is_some_and(|plan| plan.xml_documents().contains_key(&path)) {
            xml_documents.insert(path);
        } else {
            return Err(invalid_scope());
        }
    }
    Ok(ResolvedScope {
        physical,
        xml_documents,
    })
}

pub(super) fn append_findings(
    report: Option<&ProjectXmlLuaAnalysis>,
    selected: &BTreeSet<String>,
    findings: &mut Vec<GenericFinding>,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    let Some(report) = report else {
        return Ok(());
    };
    for unit in report.units() {
        super::cancelled(stop)?;
        if !selected.contains(&unit.document) {
            continue;
        }
        for diagnostic in &unit.diagnostics {
            super::cancelled(stop)?;
            let primary = diagnostic
                .xml_spans
                .first()
                .ok_or_else(|| super::owner_error("XML diagnostic has no mapped source"))?;
            let location = ExactSourceLocation::new(
                unit.document.as_str(),
                unit.document_digest.to_string(),
                primary.byte_start,
                primary.byte_end,
            )?;
            let id = crate::identity::canonical_digest(
                "service-xml-generic:sha256:",
                &(report.analysis_id(), diagnostic),
            )?;
            let code = diagnostic.parser_diagnostic.kind.upstream_code();
            findings.push(
                GenericFinding::new(id, "emmy.xml.inline.syntax", code, "error", location)?
                    .with_xml_source_mapping(unit, diagnostic)?,
            );
        }
    }
    Ok(())
}
fn invalid_scope() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InvalidRequest,
        "selected files are not in the exact project snapshot",
    )
}
