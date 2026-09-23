//! Project-owned XML/Lua lookup observations in the ordinary finding stream.
use crate::{ExactSourceLocation, GenericFinding, ServiceResult};
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;
use wow_project::load::ProjectLoadPlan;
use wow_project::xml_bindings::{ProjectXmlLuaBindings, XmlLuaBindingState};

pub(super) fn append_findings(
    report: Option<&ProjectXmlLuaBindings>,
    plan: Option<&ProjectLoadPlan>,
    selected: &BTreeSet<String>,
    findings: &mut Vec<GenericFinding>,
    stop: &AtomicBool,
) -> ServiceResult<()> {
    let Some(report) = report else {
        return Ok(());
    };
    let plan = plan.ok_or_else(|| super::owner_error("XML Lua binding has no load plan"))?;
    for binding in report.bindings() {
        super::cancelled(stop)?;
        if !selected.contains(&binding.document)
            || binding.state == XmlLuaBindingState::UniqueAnalyzerDeclaration
        {
            continue;
        }
        let original = plan
            .document_text(&binding.document)
            .ok_or_else(|| super::owner_error("XML Lua binding document is missing"))?;
        let start = usize::try_from(binding.attribute_span.byte_start)
            .map_err(|_| super::owner_error("XML Lua binding coordinate overflow"))?;
        let end = usize::try_from(binding.attribute_span.byte_end)
            .map_err(|_| super::owner_error("XML Lua binding coordinate overflow"))?;
        if original.get(start..end).is_none() {
            return Err(super::owner_error("XML Lua binding coordinate mismatch"));
        }
        let location = ExactSourceLocation::new(
            binding.document.as_str(),
            binding.content_digest.to_string(),
            binding.attribute_span.byte_start,
            binding.attribute_span.byte_end,
        )?;
        let id = crate::identity::canonical_digest(
            "service-xml-lua-binding:sha256:",
            &(report.analysis_id(), binding),
        )?;
        findings.push(GenericFinding::new(
            id,
            "project.xml.lua_bindings",
            binding.state.code(),
            "information",
            location,
        )?);
    }
    Ok(())
}
