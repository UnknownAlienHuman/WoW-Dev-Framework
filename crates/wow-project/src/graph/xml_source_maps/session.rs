use super::*;

pub(super) struct Session<'a> {
    parsed: BTreeMap<&'a str, &'a VirtualSyntaxUnit>,
    syntax: BTreeMap<&'a str, &'a EmmySyntaxFileReport>,
    members: BTreeMap<&'a str, &'a EmmyMemberCallFileReport>,
}

pub(super) fn session<'a>(
    project: &ProjectView,
    analysis: Option<&'a ProjectXmlLuaAnalysis>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<Session<'a>> {
    let mut session = Session {
        parsed: BTreeMap::new(),
        syntax: BTreeMap::new(),
        members: BTreeMap::new(),
    };
    let Some(analysis) = analysis else {
        return Ok(session);
    };
    for parsed in analysis.parser_report().units() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("xml-map-parser-index", &parsed.unit_id), stop)?;
        if session.parsed.len() >= MAX_SITES
            || session.parsed.insert(&parsed.unit_id, parsed).is_some()
        {
            return Err(invalid());
        }
    }
    if session.parsed.len() != analysis.units().len() {
        return Err(invalid());
    }
    let Some(report) = analysis.semantic_report() else {
        if !analysis.units().is_empty()
            || analysis.semantic_state() != XmlLuaSemanticState::NotEvaluatedNoInlineUnits
        {
            return Err(invalid());
        }
        return Ok(session);
    };
    let analyzer = project.snapshot().analyzer_binding();
    if analysis.units().is_empty()
        || report.project_generation() != project.project_generation()
        || report.main_snapshot_id() != analyzer.main_workspace().snapshot_id()
        || report
            .library_snapshot_ids()
            .ne(analyzer.library_snapshot_ids())
        || report.wrapper_profile() != "none_exact_unwrapped_source"
        || report.syntax_report().workspace_snapshot_id() != report.virtual_snapshot_id()
        || report.member_call_report().main_snapshot_id() != report.virtual_snapshot_id()
        || report
            .member_call_report()
            .library_snapshot_ids()
            .ne(analyzer.library_snapshot_ids())
    {
        return Err(invalid());
    }
    for file in report.syntax_report().files() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("xml-map-syntax-index", file.path()), stop)?;
        if session.syntax.len() >= MAX_SITES || session.syntax.insert(file.path(), file).is_some() {
            return Err(invalid());
        }
    }
    let mut complete = true;
    for file in report.member_call_report().files() {
        crate::analyzer::checkpoint(stop)?;
        budget.charge_serialized(&("xml-map-member-index", file.path()), stop)?;
        complete &= file.status() == EmmyFactFileStatus::Complete;
        if session.members.len() >= MAX_SITES || session.members.insert(file.path(), file).is_some()
        {
            return Err(invalid());
        }
    }
    if session.syntax.len() != analysis.units().len()
        || session.members.len() != analysis.units().len()
        || analysis.semantic_state()
            != if complete {
                XmlLuaSemanticState::Complete
            } else {
                XmlLuaSemanticState::PartialFailedParse
            }
    {
        return Err(invalid());
    }
    Ok(session)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_unit(
    project: &ProjectView,
    unit: &XmlLuaUnitAnalysis,
    fact: &ProjectXmlFact,
    body: &XmlInlineLua,
    session: &mut Session<'_>,
    budget: &mut ProducerBudget,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let parsed = session
        .parsed
        .remove(unit.unit_id.as_ref())
        .ok_or_else(invalid)?;
    let syntax = session
        .syntax
        .remove(unit.virtual_path.as_str())
        .ok_or_else(invalid)?;
    let members = session
        .members
        .remove(unit.virtual_path.as_str())
        .ok_or_else(invalid)?;
    let suffix = unit
        .unit_id
        .strip_prefix("project-lua-unit:sha256:")
        .ok_or_else(invalid)?;
    budget.charge_serialized(&("xml-map-virtual-address", suffix, &unit.context), stop)?;
    let context_id = crate::identity::canonical_id(
        "project-xml-lua-context:sha256:",
        XML_LUA_CONTEXT_POLICY_PROFILE,
        &(
            unit.unit_id.as_ref(),
            unit.context.script_site(),
            unit.context.implicit_receiver(),
            unit.context.runtime_dispatch(),
        ),
        ProjectPhase::Analyzer,
    )?;
    let expected_state = if members.status() == EmmyFactFileStatus::Complete {
        XmlLuaSemanticState::Complete
    } else {
        XmlLuaSemanticState::PartialFailedParse
    };
    if unit.package != fact.scope.package
        || unit.document != fact.document
        || unit.document_digest != fact.content_digest
        || unit.script_occurrence_id != fact.occurrence_id
        || unit.script_name != fact.element_name
        || unit.extracted_unit_id != body.unit_id
        || unit.content_digest != body.content_digest
        || unit.byte_length != body.byte_length
        || parsed.content_digest != body.content_digest
        || parsed.byte_length != body.byte_length
        || unit.diagnostics.len() != parsed.diagnostics.len()
        || syntax.content_sha256() != body.content_digest.to_string()
        || members.content_sha256() != body.content_digest.to_string()
        || unit.virtual_path != format!("xml-virtual/{suffix}.lua")
        || unit.virtual_uri != format!("wow-xml-lua:///{suffix}.lua")
        || !unit.context.admits_static_source_association()
        || unit.context.context_id() != context_id.as_ref()
        || unit.semantic_state != expected_state
        || project.snapshot().analyzer_binding().project_generation()
            != project.project_generation()
    {
        return Err(invalid());
    }
    for (diagnostic, parsed) in unit.diagnostics.iter().zip(&parsed.diagnostics) {
        crate::analyzer::checkpoint(stop)?;
        if diagnostic.parser_diagnostic != *parsed {
            return Err(invalid());
        }
    }
    crate::analyzer::checkpoint(stop)
}

impl Session<'_> {
    pub(super) fn is_consumed(&self) -> bool {
        self.parsed.is_empty() && self.syntax.is_empty() && self.members.is_empty()
    }
}
