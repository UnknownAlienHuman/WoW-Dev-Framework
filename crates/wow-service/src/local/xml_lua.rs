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
    let xml_candidates = xml_documents(project)?;
    if matches!(scope, CheckScope::WholeProject) {
        return Ok(ResolvedScope {
            physical: project.file_manifest().iter().collect(),
            xml_documents: xml_candidates,
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
        } else if xml_candidates.contains(&path) {
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

fn xml_documents(project: &ProjectView) -> ServiceResult<BTreeSet<String>> {
    let configuration = project.configuration();
    if let Some(plan) = configuration.load_plan() {
        return Ok(plan.xml_documents().keys().cloned().collect());
    }
    let Some(packages) = configuration.package_load_plan() else {
        return Ok(BTreeSet::new());
    };
    let mut documents = BTreeSet::new();
    for package in packages.packages() {
        let plan = packages
            .package_plan(&package.package)
            .ok_or_else(|| super::owner_error("package XML load receipt is missing"))?;
        for path in plan.xml_documents().keys() {
            let qualified = packages
                .source_path(&package.package, path)
                .ok_or_else(|| super::owner_error("package XML source identity is missing"))?;
            if !documents.insert(qualified) {
                return Err(super::owner_error(
                    "package XML document identities collide",
                ));
            }
        }
    }
    Ok(documents)
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
        for diagnostic in &unit.semantic_diagnostics {
            super::cancelled(stop)?;
            if matches!(
                diagnostic.kind,
                wow_emmy::EmmySyntaxDiagnosticKind::LuaSyntax
                    | wow_emmy::EmmySyntaxDiagnosticKind::DocumentationSyntax
            ) {
                // The existing syntax-only owner already presents these exact
                // classes. Retain the same-session copy in owner evidence without
                // duplicating the user-facing finding.
                continue;
            }
            let primary = diagnostic
                .source
                .xml_spans
                .first()
                .ok_or_else(|| super::owner_error("XML semantic diagnostic has no mapping"))?;
            let location = ExactSourceLocation::new(
                unit.document.as_str(),
                unit.document_digest.to_string(),
                primary.byte_start,
                primary.byte_end,
            )?;
            let id = crate::identity::canonical_digest(
                "service-xml-semantic:sha256:",
                &(report.analysis_id(), diagnostic),
            )?;
            findings.push(
                GenericFinding::new(
                    id,
                    "emmy.xml.inline.semantic",
                    diagnostic.upstream_code.as_str(),
                    semantic_severity(diagnostic.severity),
                    location,
                )?
                .with_xml_semantic_source_mapping(unit, diagnostic)?,
            );
        }
    }
    Ok(())
}

const fn semantic_severity(severity: wow_emmy::EmmyDiagnosticSeverity) -> &'static str {
    match severity {
        wow_emmy::EmmyDiagnosticSeverity::Error => "error",
        wow_emmy::EmmyDiagnosticSeverity::Warning => "warning",
        wow_emmy::EmmyDiagnosticSeverity::Information => "information",
        wow_emmy::EmmyDiagnosticSeverity::Hint => "hint",
        wow_emmy::EmmyDiagnosticSeverity::Unknown => "unknown",
    }
}

fn invalid_scope() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::InvalidRequest,
        "selected files are not in the exact project snapshot",
    )
}
