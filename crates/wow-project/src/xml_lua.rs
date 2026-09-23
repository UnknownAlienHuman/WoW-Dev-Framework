//! Generation-bound syntax analysis of retained XML inline units. Emmy owns Lua
//! parsing; this adapter owns source identity and exact XML coordinate mapping.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use crate::load::{XmlInlineLua, XmlScriptSource, XmlSourceSpan};
use crate::{ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent};
use wow_emmy::virtual_syntax::{
    VIRTUAL_SYNTAX_PROFILE, VirtualLuaInput, VirtualSyntaxDiagnostic, VirtualSyntaxError,
    VirtualSyntaxReport, analyze_virtual_syntax,
};

pub const XML_LUA_ANALYSIS_PROFILE: &str = "wow-project/xml-lua-syntax/1";
const MAX_MAPPED_PIECES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaDiagnosticMapping {
    ExactPieces,
    CaretBoundaries,
}

/// One parser diagnostic with all its source pieces, not a widened enclosing
/// XML range. Caret boundaries may contain two source positions around a gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaDiagnostic {
    pub diagnostic_id: Box<str>,
    pub parser_diagnostic: VirtualSyntaxDiagnostic,
    pub mapping: XmlLuaDiagnosticMapping,
    pub xml_spans: Vec<XmlSourceSpan>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaUnitAnalysis {
    pub unit_id: Box<str>,
    pub virtual_uri: String,
    pub extracted_unit_id: String,
    pub document: String,
    pub document_digest: ContentDigest<SourceContent>,
    pub script_occurrence_id: String,
    pub script_name: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    pub diagnostics: Vec<XmlLuaDiagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaUnresolvedScript {
    pub document: String,
    pub script_occurrence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlLuaAnalysis {
    profile: &'static str,
    parser_profile: &'static str,
    upstream_revision: &'static str,
    upstream_tree: &'static str,
    project_generation: ProjectGenerationId,
    load_plan_digest: ContentDigest<CanonicalResult>,
    parser_analysis_id: String,
    units: Vec<XmlLuaUnitAnalysis>,
    unresolved_scripts: Vec<XmlLuaUnresolvedScript>,
    semantic_analysis: &'static str,
    analysis_id: Box<str>,
    #[serde(skip)]
    parser_report: VirtualSyntaxReport,
}
impl ProjectXmlLuaAnalysis {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn units(&self) -> &[XmlLuaUnitAnalysis] {
        &self.units
    }
    #[must_use]
    pub fn unresolved_scripts(&self) -> &[XmlLuaUnresolvedScript] {
        &self.unresolved_scripts
    }
    #[must_use]
    pub fn parser_report(&self) -> &VirtualSyntaxReport {
        &self.parser_report
    }
    #[must_use]
    pub fn diagnostic_count(&self) -> usize {
        self.units.iter().map(|unit| unit.diagnostics.len()).sum()
    }
}

struct Source<'a> {
    unit_id: Box<str>,
    document: &'a str,
    document_digest: ContentDigest<SourceContent>,
    script_id: &'a str,
    script_name: &'a str,
    body: &'a XmlInlineLua,
}

pub(crate) fn analyze(
    configuration: &ProjectConfiguration,
    generation: ProjectGenerationId,
    physical_files: usize,
    physical_bytes: u64,
    stop: &AtomicBool,
) -> ProjectResult<Option<ProjectXmlLuaAnalysis>> {
    let Some(plan) = configuration.load_plan() else {
        return Ok(None);
    };
    if plan.xml_documents().is_empty() {
        return Ok(None);
    }
    let budget = configuration.budget_policy();
    let mut sources = Vec::new();
    let mut unresolved = Vec::new();
    let mut source_bytes = physical_bytes;
    for (path, index) in plan.xml_documents() {
        crate::analyzer::checkpoint(stop)?;
        let text = plan.document_text(path).ok_or_else(invalid)?;
        let document_digest = crate::identity::source_digest(text.as_bytes());
        for element in index.scripts() {
            let Some(script) = &element.script else {
                return Err(invalid());
            };
            if script.source_kind == XmlScriptSource::Unresolved {
                unresolved.push(XmlLuaUnresolvedScript {
                    document: path.clone(),
                    script_occurrence_id: element.occurrence_id.clone(),
                });
                continue;
            }
            if script.source_kind != XmlScriptSource::InlineBody {
                continue;
            }
            let body = script.inline_lua.as_ref().ok_or_else(invalid)?;
            if body.byte_length != body.text().len() as u64
                || body.content_digest != crate::identity::source_digest(body.text().as_bytes())
            {
                return Err(invalid());
            }
            source_bytes = source_bytes
                .checked_add(body.byte_length)
                .ok_or_else(exhausted)?;
            if body.byte_length > budget.max_single_file_bytes()
                || source_bytes > budget.max_total_source_bytes()
                || physical_files
                    .saturating_add(sources.len())
                    .saturating_add(1) as u64
                    > budget.max_files()
            {
                return Err(exhausted());
            }
            let unit_id = crate::identity::canonical_id(
                "project-lua-unit:sha256:",
                XML_LUA_ANALYSIS_PROFILE,
                &(
                    generation,
                    plan.digest(),
                    path,
                    document_digest,
                    &element.occurrence_id,
                    &body.unit_id,
                    VIRTUAL_SYNTAX_PROFILE,
                ),
                ProjectPhase::Analyzer,
            )?;
            sources.push(Source {
                unit_id,
                document: path,
                document_digest,
                script_id: &element.occurrence_id,
                script_name: &element.qualified_name,
                body,
            });
        }
    }
    // Source-scoped IDs remain distinct even when bodies have identical text.
    let inputs = sources
        .iter()
        .map(|source| {
            VirtualLuaInput::new(
                &source.unit_id,
                source.body.text(),
                source.body.content_digest,
            )
        })
        .collect::<Vec<_>>();
    let parser_report = analyze_virtual_syntax(
        configuration.analyzer_binding().backend(),
        generation,
        &inputs,
        stop,
    )
    .map_err(|error| match error {
        VirtualSyntaxError::Cancelled => ProjectError::new(
            ProjectErrorCode::AnalysisCancelled,
            ProjectPhase::Analyzer,
            "XML Lua analysis cancelled",
        ),
        VirtualSyntaxError::BudgetExceeded => exhausted(),
        _ => invalid(),
    })?;
    let by_id: BTreeMap<&str, &Source<'_>> = sources
        .iter()
        .map(|source| (source.unit_id.as_ref(), source))
        .collect();
    if by_id.len() != sources.len() || parser_report.units().len() != sources.len() {
        return Err(invalid());
    }
    let mut units = Vec::with_capacity(sources.len());
    let mut mapped_count = 0_usize;
    let mut diagnostic_count = 0_usize;
    for parsed in parser_report.units() {
        crate::analyzer::checkpoint(stop)?;
        let source = by_id.get(parsed.unit_id.as_str()).ok_or_else(invalid)?;
        if parsed.content_digest != source.body.content_digest
            || parsed.byte_length != source.body.byte_length
        {
            return Err(invalid());
        }
        diagnostic_count = diagnostic_count
            .checked_add(parsed.diagnostics.len())
            .ok_or_else(exhausted)?;
        if diagnostic_count as u64 > budget.max_generic_findings() {
            return Err(exhausted());
        }
        let mut diagnostics = Vec::with_capacity(parsed.diagnostics.len());
        for (ordinal, diagnostic) in parsed.diagnostics.iter().enumerate() {
            crate::analyzer::checkpoint(stop)?;
            let start = usize::try_from(diagnostic.byte_start).map_err(|_| invalid())?;
            let end = usize::try_from(diagnostic.byte_end).map_err(|_| invalid())?;
            let (mapping, xml_spans) = if start == end {
                (
                    XmlLuaDiagnosticMapping::CaretBoundaries,
                    source.body.map_position(start)?,
                )
            } else {
                (
                    XmlLuaDiagnosticMapping::ExactPieces,
                    source.body.map_range(start, end)?,
                )
            };
            let xml = plan.document_text(source.document).ok_or_else(invalid)?;
            if xml_spans.is_empty()
                || xml_spans.iter().any(|span| {
                    span.byte_start > span.byte_end
                        || span.byte_end > xml.len() as u64
                        || !xml.is_char_boundary(span.byte_start as usize)
                        || !xml.is_char_boundary(span.byte_end as usize)
                })
            {
                return Err(invalid());
            }
            mapped_count = mapped_count
                .checked_add(xml_spans.len())
                .ok_or_else(exhausted)?;
            if mapped_count > MAX_MAPPED_PIECES {
                return Err(exhausted());
            }
            let diagnostic_id = crate::identity::canonical_id(
                "project-xml-diagnostic:sha256:",
                XML_LUA_ANALYSIS_PROFILE,
                &(
                    source.unit_id.as_ref(),
                    ordinal,
                    diagnostic,
                    mapping,
                    &xml_spans,
                ),
                ProjectPhase::Analyzer,
            )?;
            diagnostics.push(XmlLuaDiagnostic {
                diagnostic_id,
                parser_diagnostic: diagnostic.clone(),
                mapping,
                xml_spans,
            });
        }
        let suffix = source
            .unit_id
            .strip_prefix("project-lua-unit:sha256:")
            .ok_or_else(invalid)?;
        units.push(XmlLuaUnitAnalysis {
            unit_id: source.unit_id.clone(),
            virtual_uri: format!("wow-xml-lua:///{suffix}.lua"),
            extracted_unit_id: source.body.unit_id.clone(),
            document: source.document.to_owned(),
            document_digest: source.document_digest,
            script_occurrence_id: source.script_id.to_owned(),
            script_name: source.script_name.to_owned(),
            content_digest: parsed.content_digest,
            byte_length: parsed.byte_length,
            diagnostics,
        });
    }
    let analysis_id = crate::identity::canonical_id(
        "project-xml-lua-analysis:sha256:",
        XML_LUA_ANALYSIS_PROFILE,
        &(
            generation,
            plan.digest(),
            parser_report.analysis_id(),
            &units,
            &unresolved,
        ),
        ProjectPhase::Analyzer,
    )?;
    Ok(Some(ProjectXmlLuaAnalysis {
        profile: XML_LUA_ANALYSIS_PROFILE,
        parser_profile: VIRTUAL_SYNTAX_PROFILE,
        upstream_revision: wow_emmy::EMMYLUA_REVISION,
        upstream_tree: wow_emmy::EMMYLUA_TREE,
        project_generation: generation,
        load_plan_digest: plan.digest(),
        parser_analysis_id: parser_report.analysis_id().to_owned(),
        units,
        unresolved_scripts: unresolved,
        semantic_analysis: "not_evaluated",
        analysis_id,
        parser_report,
    }))
}
fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::AnalyzerSnapshotMismatch,
        ProjectPhase::Analyzer,
        "XML Lua source, parser report, or mapping is inconsistent",
    )
}
fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Analyzer,
        "XML Lua analysis budget exceeded",
    )
}
