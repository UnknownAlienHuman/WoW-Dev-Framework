//! Exact retained XML-to-virtual-Lua associations. No extraction or analysis.
mod coordinates;
mod output;
mod session;
mod support;

use coordinates::{line_starts, validate_body, validate_observations, validate_xml_span, visit};
use session::{session, validate_unit};
use support::{admit_piece, fact_support, validate_support};

use super::producer_budget::ProducerBudget;
use super::*;
use crate::load::{
    ProjectPackageReachability, XmlElementRecord, XmlInlineLua, XmlLuaMapKind, XmlLuaMapSegment,
    XmlScriptSource, XmlSourceSpan,
};
use crate::xml_lua::{
    ProjectXmlLuaAnalysis, XML_LUA_CONTEXT_POLICY_PROFILE, XmlLuaDiagnosticMapping,
    XmlLuaMappedSpan, XmlLuaSemanticState, XmlLuaUnitAnalysis,
};
use std::collections::BTreeSet;
use wow_core::{CanonicalResult, ContentDigest, SourceContent, SourceSpanKind};
use wow_emmy::EmmyFactFileStatus;
use wow_emmy::references::EmmyMemberCallFileReport;
use wow_emmy::syntax::EmmySyntaxFileReport;
use wow_emmy::virtual_syntax::VirtualSyntaxUnit;

const MAX_PIECES: usize = 200_000;
const MAX_SITES: usize = 4096;
const MAX_VALUE_BYTES: usize = 4096;
type OwnedSite = (Option<Box<str>>, Box<str>, Box<str>);

/// Observed native mapping coverage; it cannot admit a graph or an analysis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSourceMapSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis_id: Option<Box<str>>,
    semantic_state: XmlLuaSemanticState,
    unit_count: usize,
    piece_count: usize,
    omissions: Vec<XmlSourceMapOmission>,
}

impl XmlSourceMapSummary {
    #[must_use]
    pub fn analysis_id(&self) -> Option<&str> {
        self.analysis_id.as_deref()
    }

    #[must_use]
    pub const fn semantic_state(&self) -> XmlLuaSemanticState {
        self.semantic_state
    }

    #[must_use]
    pub const fn unit_count(&self) -> usize {
        self.unit_count
    }

    #[must_use]
    pub const fn piece_count(&self) -> usize {
        self.piece_count
    }

    #[must_use]
    pub fn omissions(&self) -> &[XmlSourceMapOmission] {
        &self.omissions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum XmlSourceMapOmissionOutcome {
    ExternalFile,
    ReferenceOnly,
    UnresolvedScript,
    UnreachableInline,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct XmlSourceMapOmission {
    scope: ProjectXmlFactScope,
    document: String,
    document_digest: ContentDigest<CanonicalResult>,
    occurrence: String,
    outcome: XmlSourceMapOmissionOutcome,
}

impl XmlSourceMapOmission {
    #[must_use]
    pub const fn scope(&self) -> &ProjectXmlFactScope {
        &self.scope
    }

    #[must_use]
    pub fn document(&self) -> &str {
        &self.document
    }

    #[must_use]
    pub const fn document_digest(&self) -> ContentDigest<CanonicalResult> {
        self.document_digest
    }

    #[must_use]
    pub fn occurrence(&self) -> &str {
        &self.occurrence
    }

    #[must_use]
    pub const fn outcome(&self) -> XmlSourceMapOmissionOutcome {
        self.outcome
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct Support {
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Serialize)]
struct Piece<'a> {
    ordinal: usize,
    segment: &'a XmlLuaMapSegment,
    support: Support,
}

struct Unit<'a> {
    native: &'a XmlLuaUnitAnalysis,
    fact: ProjectXmlFact,
    scope: Box<str>,
    script_state: Box<str>,
    observations: ContentDigest<CanonicalResult>,
    pieces: Vec<Piece<'a>>,
}

/// Borrows only the immutable project, never the mutable graph catalog.
pub(super) struct PreparedXmlSourceMaps<'a> {
    project: &'a ProjectView,
    analysis: Option<&'a ProjectXmlLuaAnalysis>,
    units: Vec<Unit<'a>>,
    piece_count: usize,
    omissions: Vec<XmlSourceMapOmission>,
}

pub(super) fn prepare<'a>(
    project: &'a ProjectView,
    source: &mut ProjectGraphProvenance,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<PreparedXmlSourceMaps<'a>> {
    validate_project(project, source, stop)?;
    let analysis = project.snapshot().analyzer_binding().xml_lua_analysis();
    if source.xml_lua_analysis.as_ref() != analysis {
        return Err(invalid());
    }
    let mut session = session(project, analysis, budget, stop)?;
    let mut native_units = BTreeMap::<OwnedSite, &XmlLuaUnitAnalysis>::new();
    let mut unresolved = BTreeSet::new();
    let mut unit_ids = BTreeSet::new();
    let mut virtual_paths = BTreeSet::new();
    if let Some(analysis) = analysis {
        if analysis.units().len() > MAX_SITES || analysis.unresolved_scripts().len() > MAX_SITES {
            return Err(exhausted());
        }
        for unit in analysis.units() {
            crate::analyzer::checkpoint(stop)?;
            let address = owned_site(
                unit.package.as_deref(),
                &unit.document,
                &unit.script_occurrence_id,
                budget,
                stop,
            )?;
            budget.charge_serialized(
                &(
                    "xml-map-unit-index",
                    &address,
                    &unit.unit_id,
                    &unit.virtual_path,
                ),
                stop,
            )?;
            if native_units.insert(address, unit).is_some()
                || !unit_ids.insert(unit.unit_id.as_ref())
                || !virtual_paths.insert(unit.virtual_path.as_str())
            {
                return Err(invalid());
            }
        }
        for row in analysis.unresolved_scripts() {
            crate::analyzer::checkpoint(stop)?;
            budget.charge_serialized(&("xml-map-unresolved-index", row), stop)?;
            let address = owned_site(
                row.package.as_deref(),
                &row.document,
                &row.script_occurrence_id,
                budget,
                stop,
            )?;
            if !unresolved.insert(address) {
                return Err(invalid());
            }
        }
    }

    // Owned addresses permit support admission without retaining catalog borrows.
    let mut facts = BTreeMap::new();
    for (position, fact) in source.xml_facts().iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        if !matches!(fact.kind, ProjectXmlFactKind::Script { .. }) {
            continue;
        }
        if facts.len() >= MAX_SITES {
            return Err(exhausted());
        }
        let address = owned_site(
            fact.scope.package.as_deref(),
            &fact.document,
            &fact.occurrence_id,
            budget,
            stop,
        )?;
        if fact.context_id != source.context().context_id()
            || facts.insert(address, position).is_some()
        {
            return Err(invalid());
        }
    }
    let mut evidence_by_handle = BTreeMap::new();
    for (id, evidence) in source.evidence() {
        crate::analyzer::checkpoint(stop)?;
        evidence.validate().map_err(|_| invalid())?;
        let [handle] = evidence.source_handle_ids() else {
            return Err(invalid());
        };
        budget.charge_serialized(&("xml-map-evidence-index", handle, id), stop)?;
        if evidence.evidence_id() != *id
            || !source.source_handles().contains_key(handle)
            || evidence_by_handle.insert(*handle, *id).is_some()
        {
            return Err(invalid());
        }
    }
    let mut units = Vec::new();
    let mut omissions = Vec::new();
    let mut visited_pieces = 0usize;
    let mut piece_count = 0usize;
    budget.charge_serialized(&("xml-map-scopes", MAX_SITES), stop)?;
    for input in load_inputs::scopes(project, stop)? {
        crate::analyzer::checkpoint(stop)?;
        let plan = input.plan();
        plan.validate_profile(project.configuration().selected_profile())?;
        let reachable = !input
            .node()
            .is_some_and(|node| node.reachability == ProjectPackageReachability::Unreachable);
        budget.charge_serialized(
            &("xml-map-scope", input.package(), plan.selected_toc()),
            stop,
        )?;
        let scope = ProjectXmlFactScope {
            selected_toc: input.qualified_path(plan.selected_toc())?,
            flavor: project
                .configuration()
                .selected_profile()
                .flavor_id()
                .to_owned(),
            package: input.package().map(str::to_owned),
        };
        for (local, index) in plan.xml_documents() {
            crate::analyzer::checkpoint(stop)?;
            budget.charge_serialized(&("xml-map-document", input.package(), local), stop)?;
            let mapped = input.mapped_source(local)?;
            let text = plan.document_text(local).ok_or_else(invalid)?;
            let artifact = project.source_artifact(&mapped.path)?.ok_or_else(invalid)?;
            let file = source
                .files()
                .iter()
                .find(|file| file.path == mapped.path)
                .ok_or_else(invalid)?;
            if index.document() != local
                || index.source_digest() != mapped.content_digest
                || mapped.byte_length != text.len() as u64
                || crate::identity::source_digest(text.as_bytes()) != mapped.content_digest
                || artifact.content_digest() != mapped.content_digest
                || artifact.byte_length() != mapped.byte_length
                || file.content_digest != mapped.content_digest
                || file.byte_length != mapped.byte_length
            {
                return Err(invalid());
            }
            if index.scripts().next().is_none() {
                continue;
            }
            let lines = line_starts(text, budget, stop)?;
            for element in index.scripts() {
                crate::analyzer::checkpoint(stop)?;
                let script = element.script.as_ref().ok_or_else(invalid)?;
                let address = owned_site(
                    input.package(),
                    &mapped.path,
                    &element.occurrence_id,
                    budget,
                    stop,
                )?;
                let position = facts.remove(&address).ok_or_else(invalid)?;
                let borrowed = source.xml_facts().get(position).ok_or_else(invalid)?;
                validate_fact(
                    borrowed,
                    &scope,
                    index.digest(),
                    mapped.content_digest,
                    element,
                )?;
                budget.charge_serialized(&("xml-map-script-copy", borrowed), stop)?;
                let fact = borrowed.clone();
                validate_support(
                    project,
                    source,
                    fact_support(&fact),
                    &mapped.path,
                    fact.span,
                    budget,
                    stop,
                )?;
                validate_xml_span(text, &lines, &element.span)?;
                validate_xml_span(text, &lines, &script.body_span)?;
                if script.body_span.byte_start < element.span.byte_start
                    || script.body_span.byte_end > element.span.byte_end
                {
                    return Err(invalid());
                }
                let outcome = match script.source_kind {
                    XmlScriptSource::ExternalFile => XmlSourceMapOmissionOutcome::ExternalFile,
                    XmlScriptSource::ReferenceOnly => XmlSourceMapOmissionOutcome::ReferenceOnly,
                    XmlScriptSource::Unresolved => {
                        if unresolved.remove(&address) != reachable {
                            return Err(invalid());
                        }
                        XmlSourceMapOmissionOutcome::UnresolvedScript
                    }
                    XmlScriptSource::InlineBody => {
                        let body = script.inline_lua.as_ref().ok_or_else(invalid)?;
                        visit(&mut visited_pieces, body.segments().len())?;
                        validate_body(
                            body,
                            text,
                            &lines,
                            &script.body_span,
                            &mut visited_pieces,
                            budget,
                            stop,
                        )?;
                        if reachable {
                            let unit = native_units.remove(&address).ok_or_else(invalid)?;
                            validate_unit(project, unit, &fact, body, &mut session, budget, stop)?;
                            validate_observations(
                                unit,
                                body,
                                text,
                                &lines,
                                &mut visited_pieces,
                                budget,
                                stop,
                            )?;
                            let observations = (
                                &unit.diagnostics,
                                &unit.semantic_diagnostics,
                                &unit.member_references,
                                &unit.member_calls,
                            );
                            budget.charge_serialized(&observations, stop)?;
                            let observations = crate::identity::canonical_digest(
                                "wow-project/platform-xml-unit-observations/1",
                                &observations,
                                ProjectPhase::View,
                            )?;
                            crate::analyzer::checkpoint(stop)?;
                            let scope = xml_roles::canonical_text(&fact.scope, budget, stop)?;
                            let script_state = xml_roles::canonical_text(&fact.kind, budget, stop)?;
                            let mut pieces = Vec::new();
                            for (ordinal, segment) in body.segments().iter().enumerate() {
                                crate::analyzer::checkpoint(stop)?;
                                let support = admit_piece(
                                    project,
                                    source,
                                    &mut evidence_by_handle,
                                    &unit.document,
                                    segment,
                                    budget,
                                    stop,
                                )?;
                                let piece = Piece {
                                    ordinal,
                                    segment,
                                    support,
                                };
                                budget.charge_serialized(&piece, stop)?;
                                crate::analyzer::checkpoint(stop)?;
                                pieces.push(piece);
                            }
                            piece_count = piece_count
                                .checked_add(pieces.len())
                                .ok_or_else(exhausted)?;
                            if piece_count > MAX_PIECES || units.len() >= MAX_SITES {
                                return Err(exhausted());
                            }
                            budget.charge_serialized(
                                &(
                                    "xml-map-prepared-unit",
                                    &unit.unit_id,
                                    &scope,
                                    &script_state,
                                    observations,
                                    pieces.len(),
                                ),
                                stop,
                            )?;
                            crate::analyzer::checkpoint(stop)?;
                            units.push(Unit {
                                native: unit,
                                fact,
                                scope,
                                script_state,
                                observations,
                                pieces,
                            });
                            continue;
                        }
                        XmlSourceMapOmissionOutcome::UnreachableInline
                    }
                };
                if native_units.contains_key(&address)
                    || (script.source_kind != XmlScriptSource::Unresolved
                        && unresolved.contains(&address))
                {
                    return Err(invalid());
                }
                budget.charge_serialized(
                    &(
                        "xml-map-omission",
                        &fact.scope,
                        &fact.document,
                        fact.document_digest,
                        &fact.occurrence_id,
                        outcome,
                    ),
                    stop,
                )?;
                crate::analyzer::checkpoint(stop)?;
                omissions.push(XmlSourceMapOmission {
                    scope: fact.scope,
                    document: fact.document,
                    document_digest: fact.document_digest,
                    occurrence: fact.occurrence_id,
                    outcome,
                });
            }
        }
    }
    if !facts.is_empty()
        || !native_units.is_empty()
        || !unresolved.is_empty()
        || !session.is_consumed()
    {
        return Err(invalid());
    }
    crate::analyzer::checkpoint(stop)?;
    Ok(PreparedXmlSourceMaps {
        project,
        analysis,
        units,
        piece_count,
        omissions,
    })
}

fn validate_project(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    if source.profile != PLATFORM_DIRECT_GRAPH_WITH_XML_SOURCE_MAPS_PROFILE {
        return Err(invalid());
    }
    let analyzer = project.snapshot().analyzer_binding();
    if project.configuration().project_kind() != ProjectKind::BlizzardUiPlatformSource
        || source.context() != project.snapshot().generation_context()
        || source.project_snapshot_id != project.snapshot_id()
        || source.analyzer_snapshot_id != project.analyzer_snapshot_id()
        || analyzer.project_generation() != project.project_generation()
        || analyzer.main_workspace().universe()
            != crate::analyzer::main_workspace_universe(project.configuration())?
        || analyzer.syntax_report().workspace_snapshot_id()
            != analyzer.main_workspace().snapshot_id()
        || analyzer.member_call_report().main_snapshot_id()
            != analyzer.main_workspace().snapshot_id()
        || analyzer
            .member_call_report()
            .library_snapshot_ids()
            .ne(analyzer.library_snapshot_ids())
        || source.package_load_plan.as_ref().map(|plan| plan.digest())
            != project
                .configuration()
                .package_load_plan()
                .map(|plan| plan.digest())
        || source.package_main_plan.as_ref().map(|plan| plan.digest())
            != project
                .configuration()
                .package_main_plan()
                .map(|plan| plan.digest())
    {
        return Err(invalid());
    }
    if source.files().len() > MAX_FILES
        || source.source_handles().len() > MAX_PIECES
        || source.evidence().len() > MAX_PIECES
    {
        return Err(exhausted());
    }
    Ok(())
}

fn owned_site(
    package: Option<&str>,
    document: &str,
    occurrence: &str,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<OwnedSite> {
    budget.charge_serialized(
        &("xml-map-site-address", package, document, occurrence),
        stop,
    )?;
    crate::analyzer::checkpoint(stop)?;
    Ok((package.map(Into::into), document.into(), occurrence.into()))
}

fn validate_fact(
    fact: &ProjectXmlFact,
    scope: &ProjectXmlFactScope,
    document_digest: ContentDigest<CanonicalResult>,
    content_digest: ContentDigest<SourceContent>,
    element: &XmlElementRecord,
) -> ProjectResult<()> {
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
    let span = SourceSpan::byte_range(element.span.byte_start, element.span.byte_end)
        .map_err(|_| invalid())?;
    if fact.scope != *scope
        || fact.document_digest != document_digest
        || fact.content_digest != content_digest
        || fact.occurrence_id != element.occurrence_id
        || fact.element_name != element.qualified_name
        || fact.span != span
        || *script_name != element.qualified_name
        || *source_kind != script.source_kind
        || *owner_occurrence_id != script.owner_occurrence_id
        || *inherit != script.inherit
        || *intrinsic_order != script.intrinsic_order
        || *file_reference != script.file_reference
        || *function_reference != script.function_reference
        || *method_reference != script.method_reference
        || inline_unit_id.as_deref() != script.inline_lua.as_ref().map(|body| body.unit_id.as_str())
        || *inline_content_digest != script.inline_lua.as_ref().map(|body| body.content_digest)
        || (script.source_kind != XmlScriptSource::InlineBody && script.inline_lua.is_some())
    {
        return Err(invalid());
    }
    Ok(())
}

pub(super) fn append(
    project: &ProjectView,
    source: &ProjectGraphProvenance,
    prepared: PreparedXmlSourceMaps<'_>,
    entities: &mut Vec<EntityDraft>,
    relations: &mut Vec<RelationDraft>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<XmlSourceMapSummary> {
    output::append(project, source, prepared, entities, relations, budget, stop)
}
