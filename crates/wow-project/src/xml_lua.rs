//! Generation-bound analysis of retained XML inline Lua units. Emmy owns Lua
//! parsing and semantic resolution; this adapter owns unit identity and exact
//! XML coordinate mapping. Virtual units share the physical Main/Library
//! semantic session and never become physical project files.
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use crate::load::{
    LOAD_PROFILE, PACKAGE_LOAD_PROFILE, ProjectLoadPlan, ProjectPackageLoadPlan,
    ProjectPackageReachability, XmlInlineLua, XmlScriptSource, XmlSourceSpan,
};
use crate::{ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};
use serde::Serialize;
use wow_core::{
    CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent, SourceSpan, SourceSpanKind,
};
use wow_emmy::virtual_syntax::{
    VIRTUAL_SYNTAX_PROFILE, VirtualLuaInput, VirtualSyntaxDiagnostic, VirtualSyntaxError,
    VirtualSyntaxReport, analyze_virtual_syntax,
};
use wow_emmy::{
    EmmyDiagnosticSeverity, EmmyFactFileStatus, EmmyReferenceResolution, EmmySyntaxDiagnosticKind,
    EmmyWorkspaceErrorCode, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse, VIRTUAL_SEMANTIC_PROFILE, VirtualSemanticReport,
};

pub const XML_LUA_ANALYSIS_PROFILE: &str = "wow-project/xml-lua-semantics/4";
pub const XML_LUA_CONTEXT_POLICY_PROFILE: &str = "wow-project/xml-lua-context-policy/1";
pub const XML_LUA_EXACT_SCRIPT_SITE: &str = "exact_xml_script_site";
pub const XML_LUA_IMPLICIT_RECEIVER_NOT_EVALUATED: &str = "not_evaluated_unwrapped_source";
pub const XML_LUA_RUNTIME_DISPATCH_NOT_EVALUATED: &str = "not_evaluated_static_load_evidence_only";
const MAX_MAPPED_PIECES: usize = 262_144;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaDiagnosticMapping {
    ExactPieces,
    CaretBoundaries,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlLuaSemanticState {
    Complete,
    PartialFailedParse,
    NotEvaluatedNoInlineUnits,
}

/// Authority carried by one admitted XML virtual Lua unit. The exact script
/// site is source-backed; implicit receiver construction and runtime dispatch
/// remain explicitly unevaluated for unwrapped static source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaSemanticContext {
    profile: &'static str,
    context_id: Box<str>,
    script_site: &'static str,
    implicit_receiver: &'static str,
    runtime_dispatch: &'static str,
}

impl XmlLuaSemanticContext {
    fn new(unit_id: &str) -> ProjectResult<Self> {
        let script_site = XML_LUA_EXACT_SCRIPT_SITE;
        let implicit_receiver = XML_LUA_IMPLICIT_RECEIVER_NOT_EVALUATED;
        let runtime_dispatch = XML_LUA_RUNTIME_DISPATCH_NOT_EVALUATED;
        let context_id = crate::identity::canonical_id(
            "project-xml-lua-context:sha256:",
            XML_LUA_CONTEXT_POLICY_PROFILE,
            &(unit_id, script_site, implicit_receiver, runtime_dispatch),
            ProjectPhase::Analyzer,
        )?;
        Ok(Self {
            profile: XML_LUA_CONTEXT_POLICY_PROFILE,
            context_id,
            script_site,
            implicit_receiver,
            runtime_dispatch,
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &'static str {
        self.profile
    }

    #[must_use]
    pub fn context_id(&self) -> &str {
        &self.context_id
    }

    #[must_use]
    pub const fn script_site(&self) -> &'static str {
        self.script_site
    }

    #[must_use]
    pub const fn implicit_receiver(&self) -> &'static str {
        self.implicit_receiver
    }

    #[must_use]
    pub const fn runtime_dispatch(&self) -> &'static str {
        self.runtime_dispatch
    }

    /// Exact static source-site authority is admitted only with the matching
    /// explicit non-authority records for receiver construction and dispatch.
    #[must_use]
    pub fn admits_static_source_association(&self) -> bool {
        self.profile == XML_LUA_CONTEXT_POLICY_PROFILE
            && self.script_site == XML_LUA_EXACT_SCRIPT_SITE
            && self.implicit_receiver == XML_LUA_IMPLICIT_RECEIVER_NOT_EVALUATED
            && self.runtime_dispatch == XML_LUA_RUNTIME_DISPATCH_NOT_EVALUATED
    }
}

/// One virtual UTF-8 byte span and every exact XML source piece that supports it.
/// Empty ranges retain both possible boundaries around removed XML markup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaMappedSpan {
    pub virtual_byte_start: u64,
    pub virtual_byte_end: u64,
    pub mapping: XmlLuaDiagnosticMapping,
    pub xml_spans: Vec<XmlSourceSpan>,
}

/// One syntax-only parser diagnostic retained for compatibility with the
/// existing inline parser route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaDiagnostic {
    pub diagnostic_id: Box<str>,
    pub parser_diagnostic: VirtualSyntaxDiagnostic,
    pub mapping: XmlLuaDiagnosticMapping,
    pub xml_spans: Vec<XmlSourceSpan>,
}

/// One accepted diagnostic observed in the same Main/Library semantic session
/// as physical project analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaSemanticDiagnostic {
    pub diagnostic_id: Box<str>,
    pub category: String,
    pub upstream_code: String,
    pub kind: EmmySyntaxDiagnosticKind,
    pub severity: EmmyDiagnosticSeverity,
    pub source: XmlLuaMappedSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaMemberReference {
    pub fact_id: Box<str>,
    pub emmy_fact_id: String,
    pub receiver: String,
    pub member: String,
    pub resolution: EmmyReferenceResolution,
    pub receiver_source: XmlLuaMappedSpan,
    pub member_source: XmlLuaMappedSpan,
    pub reference_source: XmlLuaMappedSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaMemberCall {
    pub fact_id: Box<str>,
    pub emmy_fact_id: String,
    pub reference_fact_id: Box<str>,
    pub emmy_reference_fact_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument_count: Option<u64>,
    pub colon_call: bool,
    pub callee_source: XmlLuaMappedSpan,
    pub call_source: XmlLuaMappedSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaUnitAnalysis {
    pub unit_id: Box<str>,
    pub virtual_uri: String,
    pub virtual_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    pub extracted_unit_id: String,
    pub document: String,
    pub document_digest: ContentDigest<SourceContent>,
    pub script_occurrence_id: String,
    pub script_name: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    pub semantic_state: XmlLuaSemanticState,
    pub context: XmlLuaSemanticContext,
    pub diagnostics: Vec<XmlLuaDiagnostic>,
    pub semantic_diagnostics: Vec<XmlLuaSemanticDiagnostic>,
    pub member_references: Vec<XmlLuaMemberReference>,
    pub member_calls: Vec<XmlLuaMemberCall>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlLuaUnresolvedScript {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    pub document: String,
    pub script_occurrence_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectXmlLuaAnalysis {
    profile: &'static str,
    parser_profile: &'static str,
    semantic_profile: &'static str,
    upstream_revision: &'static str,
    upstream_tree: &'static str,
    project_generation: ProjectGenerationId,
    load_profile: &'static str,
    load_plan_digest: ContentDigest<CanonicalResult>,
    parser_analysis_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    semantic_analysis_id: Option<String>,
    semantic_state: XmlLuaSemanticState,
    units: Vec<XmlLuaUnitAnalysis>,
    unresolved_scripts: Vec<XmlLuaUnresolvedScript>,
    analysis_id: Box<str>,
    #[serde(skip)]
    parser_report: VirtualSyntaxReport,
    #[serde(skip)]
    semantic_report: Option<VirtualSemanticReport>,
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
    pub fn semantic_report(&self) -> Option<&VirtualSemanticReport> {
        self.semantic_report.as_ref()
    }

    #[must_use]
    pub const fn semantic_state(&self) -> XmlLuaSemanticState {
        self.semantic_state
    }

    #[must_use]
    pub fn diagnostic_count(&self) -> usize {
        self.units
            .iter()
            .map(|unit| {
                unit.diagnostics
                    .len()
                    .saturating_add(unit.semantic_diagnostics.len())
            })
            .sum()
    }

    #[must_use]
    pub fn semantic_fact_count(&self) -> usize {
        self.units
            .iter()
            .map(|unit| {
                unit.member_references
                    .len()
                    .saturating_add(unit.member_calls.len())
            })
            .sum()
    }
}

#[derive(Clone)]
struct Source {
    unit_id: Box<str>,
    virtual_path: String,
    package: Option<String>,
    document: String,
    document_digest: ContentDigest<SourceContent>,
    script_id: String,
    script_name: String,
    body: XmlInlineLua,
}

pub(crate) struct PreparedProjectXmlLuaAnalysis {
    generation: ProjectGenerationId,
    load_profile: &'static str,
    load_plan_digest: ContentDigest<CanonicalResult>,
    sources: Vec<Source>,
    unresolved: Vec<XmlLuaUnresolvedScript>,
    documents: BTreeMap<String, String>,
    parser_report: VirtualSyntaxReport,
    virtual_workspace: Option<LuaWorkspaceSnapshot>,
}

impl PreparedProjectXmlLuaAnalysis {
    #[must_use]
    pub(crate) fn virtual_workspace(&self) -> Option<&LuaWorkspaceSnapshot> {
        self.virtual_workspace.as_ref()
    }
}

pub(crate) fn prepare(
    configuration: &ProjectConfiguration,
    generation: ProjectGenerationId,
    main_universe: LuaWorkspaceUniverse,
    physical_files: usize,
    physical_bytes: u64,
    stop: &AtomicBool,
) -> ProjectResult<Option<PreparedProjectXmlLuaAnalysis>> {
    if main_universe != crate::analyzer::main_workspace_universe(configuration)? {
        return Err(ProjectError::new(
            ProjectErrorCode::AnalyzerSnapshotMismatch,
            ProjectPhase::Analyzer,
            "XML virtual Main universe differs from the configured physical Main route",
        )
        .with_candidate_generation(generation));
    }
    let (load_profile, load_plan_digest) = if let Some(plan) = configuration.load_plan() {
        (LOAD_PROFILE, plan.digest())
    } else if let Some(plan) = configuration.package_load_plan() {
        (PACKAGE_LOAD_PROFILE, plan.digest())
    } else {
        return Ok(None);
    };

    let budget = configuration.budget_policy();
    let mut sources = Vec::new();
    let mut unresolved = Vec::new();
    let mut documents = BTreeMap::new();
    let mut source_bytes = physical_bytes;

    if let Some(plan) = configuration.load_plan() {
        collect_plan(
            None,
            plan,
            load_profile,
            load_plan_digest,
            generation,
            physical_files,
            &mut source_bytes,
            &mut sources,
            &mut unresolved,
            &mut documents,
            configuration,
            stop,
        )?;
    } else if let Some(packages) = configuration.package_load_plan() {
        collect_packages(
            packages,
            load_profile,
            load_plan_digest,
            generation,
            physical_files,
            &mut source_bytes,
            &mut sources,
            &mut unresolved,
            &mut documents,
            configuration,
            stop,
        )?;
    }

    if sources.is_empty() && unresolved.is_empty() {
        return Ok(None);
    }

    sources.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    unresolved.sort_by(|left, right| {
        (&left.package, &left.document, &left.script_occurrence_id).cmp(&(
            &right.package,
            &right.document,
            &right.script_occurrence_id,
        ))
    });
    if sources.windows(2).any(|pair| {
        pair[0].unit_id == pair[1].unit_id || pair[0].virtual_path == pair[1].virtual_path
    }) {
        return Err(invalid());
    }

    let parser_inputs = sources
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
        &parser_inputs,
        stop,
    )
    .map_err(map_virtual_syntax_error)?;

    let virtual_workspace = if sources.is_empty() {
        None
    } else {
        let remaining_files = budget
            .max_files()
            .checked_sub(physical_files as u64)
            .ok_or_else(exhausted)?;
        let remaining_bytes = budget
            .max_total_source_bytes()
            .checked_sub(physical_bytes)
            .ok_or_else(exhausted)?;
        let file_limit = budget.max_single_file_bytes().min(remaining_bytes);
        if remaining_files == 0 || remaining_bytes == 0 || file_limit == 0 {
            return Err(exhausted());
        }
        Some(
            LuaWorkspaceSnapshot::build(
                configuration.analyzer_binding().backend().clone(),
                main_universe,
                sources
                    .iter()
                    .map(|source| {
                        LuaWorkspaceFileInput::new(&source.virtual_path, source.body.text())
                    })
                    .collect(),
                LuaWorkspaceLimits::new(remaining_files, 16_384, file_limit, remaining_bytes)
                    .map_err(map_workspace_error)?,
            )
            .map_err(map_workspace_error)?,
        )
    };

    Ok(Some(PreparedProjectXmlLuaAnalysis {
        generation,
        load_profile,
        load_plan_digest,
        sources,
        unresolved,
        documents,
        parser_report,
        virtual_workspace,
    }))
}

#[allow(clippy::too_many_arguments)]
fn collect_packages(
    packages: &ProjectPackageLoadPlan,
    load_profile: &'static str,
    load_plan_digest: ContentDigest<CanonicalResult>,
    generation: ProjectGenerationId,
    physical_files: usize,
    source_bytes: &mut u64,
    sources: &mut Vec<Source>,
    unresolved: &mut Vec<XmlLuaUnresolvedScript>,
    documents: &mut BTreeMap<String, String>,
    configuration: &ProjectConfiguration,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    for package in packages
        .packages()
        .iter()
        .filter(|package| package.reachability != ProjectPackageReachability::Unreachable)
    {
        crate::analyzer::checkpoint(stop)?;
        let plan = packages
            .package_plan(&package.package)
            .ok_or_else(invalid)?;
        collect_plan(
            Some((&package.package, packages)),
            plan,
            load_profile,
            load_plan_digest,
            generation,
            physical_files,
            source_bytes,
            sources,
            unresolved,
            documents,
            configuration,
            stop,
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect_plan(
    package: Option<(&str, &ProjectPackageLoadPlan)>,
    plan: &ProjectLoadPlan,
    load_profile: &'static str,
    load_plan_digest: ContentDigest<CanonicalResult>,
    generation: ProjectGenerationId,
    physical_files: usize,
    source_bytes: &mut u64,
    sources: &mut Vec<Source>,
    unresolved: &mut Vec<XmlLuaUnresolvedScript>,
    documents: &mut BTreeMap<String, String>,
    configuration: &ProjectConfiguration,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    let budget = configuration.budget_policy();
    for (local_path, index) in plan.xml_documents() {
        crate::analyzer::checkpoint(stop)?;
        let text = plan.document_text(local_path).ok_or_else(invalid)?;
        let document = match package {
            Some((package, packages)) => packages
                .source_path(package, local_path)
                .ok_or_else(invalid)?,
            None => local_path.clone(),
        };
        if documents
            .insert(document.clone(), text.to_owned())
            .is_some()
        {
            return Err(invalid());
        }
        let document_digest = crate::identity::source_digest(text.as_bytes());
        for element in index.scripts() {
            let Some(script) = &element.script else {
                return Err(invalid());
            };
            if script.source_kind == XmlScriptSource::Unresolved {
                unresolved.push(XmlLuaUnresolvedScript {
                    package: package.map(|(name, _)| name.to_owned()),
                    document: document.clone(),
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
            *source_bytes = source_bytes
                .checked_add(body.byte_length)
                .ok_or_else(exhausted)?;
            if body.byte_length > budget.max_single_file_bytes()
                || *source_bytes > budget.max_total_source_bytes()
                || physical_files
                    .saturating_add(sources.len())
                    .saturating_add(1) as u64
                    > budget.max_files()
            {
                return Err(exhausted());
            }
            let package_name = package.map(|(name, _)| name.to_owned());
            #[derive(Serialize)]
            struct UnitIdentity<'a> {
                generation: ProjectGenerationId,
                load_profile: &'static str,
                load_plan_digest: ContentDigest<CanonicalResult>,
                #[serde(skip_serializing_if = "Option::is_none")]
                package: Option<&'a str>,
                document: &'a str,
                document_digest: ContentDigest<SourceContent>,
                occurrence: &'a str,
                extracted_unit: &'a str,
                parser_profile: &'static str,
                semantic_profile: &'static str,
            }
            let unit_id = crate::identity::canonical_id(
                "project-lua-unit:sha256:",
                XML_LUA_ANALYSIS_PROFILE,
                &UnitIdentity {
                    generation,
                    load_profile,
                    load_plan_digest,
                    package: package_name.as_deref(),
                    document: &document,
                    document_digest,
                    occurrence: &element.occurrence_id,
                    extracted_unit: &body.unit_id,
                    parser_profile: VIRTUAL_SYNTAX_PROFILE,
                    semantic_profile: VIRTUAL_SEMANTIC_PROFILE,
                },
                ProjectPhase::Analyzer,
            )?;
            let virtual_path = {
                let suffix = unit_id
                    .strip_prefix("project-lua-unit:sha256:")
                    .ok_or_else(invalid)?;
                format!("xml-virtual/{suffix}.lua")
            };
            sources.push(Source {
                unit_id,
                virtual_path,
                package: package_name,
                document: document.clone(),
                document_digest,
                script_id: element.occurrence_id.clone(),
                script_name: element.qualified_name.clone(),
                body: body.clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn finish(
    prepared: PreparedProjectXmlLuaAnalysis,
    semantic_report: Option<VirtualSemanticReport>,
    stop: &AtomicBool,
) -> ProjectResult<ProjectXmlLuaAnalysis> {
    if prepared.virtual_workspace.is_some() != semantic_report.is_some() {
        return Err(invalid());
    }
    let semantic_state = match (&prepared.virtual_workspace, &semantic_report) {
        (None, None) => XmlLuaSemanticState::NotEvaluatedNoInlineUnits,
        (Some(workspace), Some(report)) => {
            if report.project_generation() != prepared.generation
                || report.virtual_snapshot_id() != workspace.snapshot_id()
                || report.wrapper_profile() != "none_exact_unwrapped_source"
            {
                return Err(invalid());
            }
            if report
                .member_call_report()
                .files()
                .iter()
                .all(|file| file.status() == EmmyFactFileStatus::Complete)
            {
                XmlLuaSemanticState::Complete
            } else {
                XmlLuaSemanticState::PartialFailedParse
            }
        }
        _ => return Err(invalid()),
    };

    let parser_by_id = prepared
        .parser_report
        .units()
        .iter()
        .map(|unit| (unit.unit_id.as_str(), unit))
        .collect::<BTreeMap<_, _>>();
    if parser_by_id.len() != prepared.sources.len() {
        return Err(invalid());
    }

    let semantic_syntax_by_path = semantic_report
        .as_ref()
        .map(|report| {
            report
                .syntax_report()
                .files()
                .iter()
                .map(|file| (file.path(), file))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let semantic_member_by_path = semantic_report
        .as_ref()
        .map(|report| {
            report
                .member_call_report()
                .files()
                .iter()
                .map(|file| (file.path(), file))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut mapped_count = 0usize;
    let mut units = Vec::with_capacity(prepared.sources.len());
    for source in &prepared.sources {
        crate::analyzer::checkpoint(stop)?;
        let parsed = parser_by_id
            .get(source.unit_id.as_ref())
            .copied()
            .ok_or_else(invalid)?;
        if parsed.content_digest != source.body.content_digest
            || parsed.byte_length != source.body.byte_length
        {
            return Err(invalid());
        }
        let mut diagnostics = Vec::with_capacity(parsed.diagnostics.len());
        for (ordinal, diagnostic) in parsed.diagnostics.iter().enumerate() {
            let mapped = map_virtual_range(
                source,
                diagnostic.byte_start,
                diagnostic.byte_end,
                &prepared.documents,
                &mut mapped_count,
            )?;
            let diagnostic_id = crate::identity::canonical_id(
                "project-xml-diagnostic:sha256:",
                XML_LUA_ANALYSIS_PROFILE,
                &(
                    source.unit_id.as_ref(),
                    ordinal,
                    diagnostic,
                    mapped.mapping,
                    &mapped.xml_spans,
                ),
                ProjectPhase::Analyzer,
            )?;
            diagnostics.push(XmlLuaDiagnostic {
                diagnostic_id,
                parser_diagnostic: diagnostic.clone(),
                mapping: mapped.mapping,
                xml_spans: mapped.xml_spans,
            });
        }

        let mut semantic_diagnostics = Vec::new();
        let mut member_references = Vec::new();
        let mut member_calls = Vec::new();
        let unit_semantic_state = if let Some(report) = &semantic_report {
            let syntax_file = semantic_syntax_by_path
                .get(source.virtual_path.as_str())
                .copied()
                .ok_or_else(invalid)?;
            let member_file = semantic_member_by_path
                .get(source.virtual_path.as_str())
                .copied()
                .ok_or_else(invalid)?;
            let expected_digest = source.body.content_digest.to_string();
            if syntax_file.content_sha256() != expected_digest
                || member_file.content_sha256() != expected_digest
            {
                return Err(invalid());
            }
            for diagnostic in report
                .syntax_report()
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.path() == source.virtual_path)
            {
                let span = diagnostic.span();
                let (start, end) = byte_range(span)?;
                let source_map =
                    map_virtual_range(source, start, end, &prepared.documents, &mut mapped_count)?;
                let diagnostic_id = crate::identity::canonical_id(
                    "project-xml-semantic-diagnostic:sha256:",
                    XML_LUA_ANALYSIS_PROFILE,
                    &(
                        source.unit_id.as_ref(),
                        diagnostic.category(),
                        diagnostic.upstream_code(),
                        diagnostic.kind(),
                        diagnostic.normalized_severity(),
                        &source_map,
                    ),
                    ProjectPhase::Analyzer,
                )?;
                semantic_diagnostics.push(XmlLuaSemanticDiagnostic {
                    diagnostic_id,
                    category: diagnostic.category().to_owned(),
                    upstream_code: diagnostic.upstream_code().to_owned(),
                    kind: diagnostic.kind(),
                    severity: diagnostic.normalized_severity(),
                    source: source_map,
                });
            }

            let mut mapped_references = BTreeMap::<String, Box<str>>::new();
            for reference in report
                .member_call_report()
                .references()
                .iter()
                .filter(|reference| reference.path() == source.virtual_path)
            {
                let receiver_source = map_span(
                    source,
                    reference.receiver_span(),
                    &prepared.documents,
                    &mut mapped_count,
                )?;
                let member_source = map_span(
                    source,
                    reference.member_span(),
                    &prepared.documents,
                    &mut mapped_count,
                )?;
                let reference_source = map_span(
                    source,
                    reference.reference_span(),
                    &prepared.documents,
                    &mut mapped_count,
                )?;
                let fact_id = crate::identity::canonical_id(
                    "project-xml-member-reference:sha256:",
                    XML_LUA_ANALYSIS_PROFILE,
                    &(
                        source.unit_id.as_ref(),
                        reference.fact_id(),
                        reference.receiver(),
                        reference.member(),
                        reference.resolution(),
                        &receiver_source,
                        &member_source,
                        &reference_source,
                    ),
                    ProjectPhase::Analyzer,
                )?;
                if mapped_references
                    .insert(reference.fact_id().to_owned(), fact_id.clone())
                    .is_some()
                {
                    return Err(invalid());
                }
                member_references.push(XmlLuaMemberReference {
                    fact_id,
                    emmy_fact_id: reference.fact_id().to_owned(),
                    receiver: reference.receiver().to_owned(),
                    member: reference.member().to_owned(),
                    resolution: reference.resolution(),
                    receiver_source,
                    member_source,
                    reference_source,
                });
            }
            for call in report
                .member_call_report()
                .calls()
                .iter()
                .filter(|call| call.path() == source.virtual_path)
            {
                let reference_fact_id = mapped_references
                    .get(call.reference_fact_id())
                    .cloned()
                    .ok_or_else(invalid)?;
                let callee_source = map_span(
                    source,
                    call.callee_span(),
                    &prepared.documents,
                    &mut mapped_count,
                )?;
                let call_source = map_span(
                    source,
                    call.call_span(),
                    &prepared.documents,
                    &mut mapped_count,
                )?;
                let fact_id = crate::identity::canonical_id(
                    "project-xml-member-call:sha256:",
                    XML_LUA_ANALYSIS_PROFILE,
                    &(
                        source.unit_id.as_ref(),
                        call.fact_id(),
                        reference_fact_id.as_ref(),
                        call.argument_count(),
                        call.is_colon_call(),
                        &callee_source,
                        &call_source,
                    ),
                    ProjectPhase::Analyzer,
                )?;
                member_calls.push(XmlLuaMemberCall {
                    fact_id,
                    emmy_fact_id: call.fact_id().to_owned(),
                    reference_fact_id,
                    emmy_reference_fact_id: call.reference_fact_id().to_owned(),
                    argument_count: call.argument_count(),
                    colon_call: call.is_colon_call(),
                    callee_source,
                    call_source,
                });
            }
            if member_file.status() == EmmyFactFileStatus::Complete {
                XmlLuaSemanticState::Complete
            } else {
                XmlLuaSemanticState::PartialFailedParse
            }
        } else {
            XmlLuaSemanticState::NotEvaluatedNoInlineUnits
        };

        semantic_diagnostics.sort_by(|left, right| left.diagnostic_id.cmp(&right.diagnostic_id));
        member_references.sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
        member_calls.sort_by(|left, right| left.fact_id.cmp(&right.fact_id));
        let suffix = source
            .unit_id
            .strip_prefix("project-lua-unit:sha256:")
            .ok_or_else(invalid)?;
        let context = XmlLuaSemanticContext::new(source.unit_id.as_ref())?;
        units.push(XmlLuaUnitAnalysis {
            unit_id: source.unit_id.clone(),
            virtual_uri: format!("wow-xml-lua:///{suffix}.lua"),
            virtual_path: source.virtual_path.clone(),
            package: source.package.clone(),
            extracted_unit_id: source.body.unit_id.clone(),
            document: source.document.clone(),
            document_digest: source.document_digest,
            script_occurrence_id: source.script_id.clone(),
            script_name: source.script_name.clone(),
            content_digest: parsed.content_digest,
            byte_length: parsed.byte_length,
            semantic_state: unit_semantic_state,
            context,
            diagnostics,
            semantic_diagnostics,
            member_references,
            member_calls,
        });
    }

    let semantic_analysis_id = semantic_report
        .as_ref()
        .map(|report| report.analysis_id().to_owned());
    #[derive(Serialize)]
    struct AnalysisIdentity<'a> {
        generation: ProjectGenerationId,
        load_profile: &'static str,
        load_plan_digest: ContentDigest<CanonicalResult>,
        parser_analysis_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        semantic_analysis_id: Option<&'a str>,
        semantic_state: XmlLuaSemanticState,
        units: &'a [XmlLuaUnitAnalysis],
        unresolved: &'a [XmlLuaUnresolvedScript],
    }
    let analysis_id = crate::identity::canonical_id(
        "project-xml-lua-analysis:sha256:",
        XML_LUA_ANALYSIS_PROFILE,
        &AnalysisIdentity {
            generation: prepared.generation,
            load_profile: prepared.load_profile,
            load_plan_digest: prepared.load_plan_digest,
            parser_analysis_id: prepared.parser_report.analysis_id(),
            semantic_analysis_id: semantic_analysis_id.as_deref(),
            semantic_state,
            units: &units,
            unresolved: &prepared.unresolved,
        },
        ProjectPhase::Analyzer,
    )?;
    Ok(ProjectXmlLuaAnalysis {
        profile: XML_LUA_ANALYSIS_PROFILE,
        parser_profile: VIRTUAL_SYNTAX_PROFILE,
        semantic_profile: VIRTUAL_SEMANTIC_PROFILE,
        upstream_revision: wow_emmy::EMMYLUA_REVISION,
        upstream_tree: wow_emmy::EMMYLUA_TREE,
        project_generation: prepared.generation,
        load_profile: prepared.load_profile,
        load_plan_digest: prepared.load_plan_digest,
        parser_analysis_id: prepared.parser_report.analysis_id().to_owned(),
        semantic_analysis_id,
        semantic_state,
        units,
        unresolved_scripts: prepared.unresolved,
        analysis_id,
        parser_report: prepared.parser_report,
        semantic_report,
    })
}

fn map_span(
    source: &Source,
    span: SourceSpan,
    documents: &BTreeMap<String, String>,
    mapped_count: &mut usize,
) -> ProjectResult<XmlLuaMappedSpan> {
    let (start, end) = byte_range(span)?;
    map_virtual_range(source, start, end, documents, mapped_count)
}

fn byte_range(span: SourceSpan) -> ProjectResult<(u64, u64)> {
    if span.kind() != SourceSpanKind::ByteRange {
        return Err(invalid());
    }
    Ok((
        span.byte_start().ok_or_else(invalid)?,
        span.byte_end().ok_or_else(invalid)?,
    ))
}

fn map_virtual_range(
    source: &Source,
    start: u64,
    end: u64,
    documents: &BTreeMap<String, String>,
    mapped_count: &mut usize,
) -> ProjectResult<XmlLuaMappedSpan> {
    let start_usize = usize::try_from(start).map_err(|_| invalid())?;
    let end_usize = usize::try_from(end).map_err(|_| invalid())?;
    let (mapping, xml_spans) = if start == end {
        (
            XmlLuaDiagnosticMapping::CaretBoundaries,
            source.body.map_position(start_usize)?,
        )
    } else {
        (
            XmlLuaDiagnosticMapping::ExactPieces,
            source.body.map_range(start_usize, end_usize)?,
        )
    };
    let xml = documents.get(&source.document).ok_or_else(invalid)?;
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
    *mapped_count = mapped_count
        .checked_add(xml_spans.len())
        .ok_or_else(exhausted)?;
    if *mapped_count > MAX_MAPPED_PIECES {
        return Err(exhausted());
    }
    Ok(XmlLuaMappedSpan {
        virtual_byte_start: start,
        virtual_byte_end: end,
        mapping,
        xml_spans,
    })
}

fn map_virtual_syntax_error(error: VirtualSyntaxError) -> ProjectError {
    match error {
        VirtualSyntaxError::Cancelled => ProjectError::new(
            ProjectErrorCode::AnalysisCancelled,
            ProjectPhase::Analyzer,
            "XML Lua syntax analysis cancelled",
        ),
        VirtualSyntaxError::BudgetExceeded => exhausted(),
        _ => invalid(),
    }
}

fn map_workspace_error(error: wow_emmy::EmmyWorkspaceError) -> ProjectError {
    match error.code() {
        EmmyWorkspaceErrorCode::FileLimitExceeded
        | EmmyWorkspaceErrorCode::FileSizeLimitExceeded
        | EmmyWorkspaceErrorCode::TotalSizeLimitExceeded => exhausted(),
        _ => invalid(),
    }
}

fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::AnalyzerSnapshotMismatch,
        ProjectPhase::Analyzer,
        "XML Lua source, semantic report, or mapping is inconsistent",
    )
}

fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Analyzer,
        "XML Lua analysis budget exceeded",
    )
}
