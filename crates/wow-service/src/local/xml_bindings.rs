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
        let consumer = binding
            .consumer_id
            .as_deref()
            .map(|id| {
                plan.xml_references()
                    .declarations()
                    .get(id)
                    .ok_or_else(|| super::owner_error("inherited XML binding consumer is missing"))
            })
            .transpose()?;
        let selected_document =
            consumer.map_or(binding.document.as_str(), |site| site.document.as_str());
        if !selected.contains(selected_document)
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
        // Report the inherited-context finding at the consuming declaration,
        // not as an unrelated diagnostic when only the template file is selected.
        // The original handler's precise attribute anchor stays in the report.
        let (document, digest, span) = consumer.map_or(
            (
                binding.document.as_str(),
                binding.content_digest,
                &binding.attribute_span,
            ),
            |site| (site.document.as_str(), site.content_digest, &site.span),
        );
        let captured = plan
            .document_text(document)
            .ok_or_else(|| super::owner_error("XML binding anchor document is missing"))?;
        let start = usize::try_from(span.byte_start)
            .map_err(|_| super::owner_error("XML binding anchor overflow"))?;
        let end = usize::try_from(span.byte_end)
            .map_err(|_| super::owner_error("XML binding anchor overflow"))?;
        if captured.get(start..end).is_none() {
            return Err(super::owner_error("XML binding anchor is invalid"));
        }
        let location =
            ExactSourceLocation::new(document, digest.to_string(), span.byte_start, span.byte_end)?;
        let id = crate::identity::canonical_digest(
            "service-xml-lua-binding:sha256:",
            &(report.analysis_id(), binding),
        )?;
        findings.push(GenericFinding::new(
            id,
            if consumer.is_some() {
                "project.xml.lua_bindings.inherited"
            } else {
                "project.xml.lua_bindings"
            },
            binding.state.code(),
            "information",
            location,
        )?);
    }
    Ok(())
}
