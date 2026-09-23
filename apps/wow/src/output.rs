use super::args::Format;
use std::fmt::Write as _;
use wow_service::{LocalOperationResult, local::LocalOutcome};

pub fn exit_code(result: &LocalOperationResult) -> u8 {
    match result.outcome_code() {
        LocalOutcome::Available => 0,
        LocalOutcome::Findings => 1,
        LocalOutcome::Partial => 2,
        LocalOutcome::Unavailable => 3,
        LocalOutcome::InternalFailure => 4,
        LocalOutcome::Cancelled => 130,
    }
}

pub fn render(
    result: &LocalOperationResult,
    format: Format,
    capabilities: bool,
) -> Result<Vec<u8>, ()> {
    if format == Format::Json {
        let mut bytes = result.canonical_bytes().map_err(|_| ())?;
        bytes.push(b'\n');
        return Ok(bytes);
    }
    let mut text = String::new();
    match result {
        LocalOperationResult::Status(status) => {
            writeln!(text, "status: {:?}", status.health()).map_err(|_| ())?;
            if let Some(context) = status.current_context() {
                writeln!(
                    text,
                    "project: {:?}\nprofile: {:?}\ngeneration: {:?}",
                    context.project_id(),
                    context.profile_id(),
                    context.project_generation_id()
                )
                .map_err(|_| ())?;
            } else {
                writeln!(
                    text,
                    "project snapshot: not materialized (status does not run analysis)"
                )
                .map_err(|_| ())?;
            }
            for component in status.components() {
                writeln!(
                    text,
                    "component {:?}: {:?}, identity {:?}",
                    component.component_id(),
                    component.health(),
                    component.exact_identity()
                )
                .map_err(|_| ())?;
                if capabilities {
                    for (id, state) in component.capabilities() {
                        writeln!(text, "  {:?}: {:?}", id, state).map_err(|_| ())?;
                    }
                }
            }
            writeln!(text, "deferred: {:?}", status.deferred_operations()).map_err(|_| ())?;
        }
        LocalOperationResult::Check(check) => {
            let context = check.context();
            writeln!(text, "check: {:?}\nproject: {:?}\nprofile: {:?}\ngeneration: {:?}\nreference: {:?}\nanalyzer: {:?}",
                check.semantic_status(), context.project_id(), context.profile_id(), context.project_generation_id(),
                context.reference_generation_id(), context.analyzer_snapshot_id()).map_err(|_| ())?;
            writeln!(
                text,
                "raw findings: {}\ndisplay roots: {}",
                check.raw_findings().len(),
                check.presentation_graph().display_root_ids().len()
            )
            .map_err(|_| ())?;
            for component in check.components() {
                writeln!(
                    text,
                    "component {:?}: {:?}, capabilities {:?}",
                    component.component_id(),
                    component.health(),
                    component.capabilities()
                )
                .map_err(|_| ())?;
            }
            writeln!(text, "deferred: {:?}", check.deferred_operations()).map_err(|_| ())?;
            if let Some(plan) = check
                .owner_analysis()
                .and_then(|analysis| analysis.load_plan())
            {
                writeln!(
                    text,
                    "load: toc={:?}, sources={}, records={}, external_files_complete={}, digest={}",
                    plan.selected_toc(),
                    plan.sources().len(),
                    plan.records().len(),
                    plan.external_files_complete(),
                    plan.digest()
                )
                .map_err(|_| ())?;
                writeln!(
                    text,
                    "load selection: package_gates={}, excluded={}, unresolved={}",
                    plan.package_gate_count(),
                    plan.excluded_records(),
                    plan.unresolved_records()
                )
                .map_err(|_| ())?;
                for (document, index) in plan.xml_documents() {
                    let inline_count = index
                        .scripts()
                        .filter(|element| {
                            element
                                .script
                                .as_ref()
                                .is_some_and(|script| script.inline_lua.is_some())
                        })
                        .count();
                    writeln!(text,
                        "xml: document={:?}, elements={}, declarations={}, scripts={}, extracted_inline={}, digest={}",
                        document, index.elements().len(), index.declarations().count(),
                        index.scripts().count(), inline_count, index.digest()
                    ).map_err(|_| ())?;
                }
                if !plan.xml_documents().is_empty() {
                    let links = plan.xml_references();
                    writeln!(text,
                        "xml references: declarations={}, references={}, unique_local={}, cycles={}, issues={}, local_links_resolved={}, digest={}",
                        links.declarations().len(), links.references().len(), links.unique_link_count(),
                        links.cycles().len(), links.issues().len(), links.local_links_resolved(), links.digest()
                    ).map_err(|_| ())?;
                }
                for issue in plan.issues() {
                    writeln!(
                        text,
                        "load issue: {}",
                        serde_json::to_string(issue).map_err(|_| ())?
                    )
                    .map_err(|_| ())?;
                }
            }
            if let Some(report) = check
                .owner_analysis()
                .and_then(|analysis| analysis.xml_lua_report())
            {
                writeln!(text, "xml inline syntax: units={}, diagnostics={}, unresolved_scripts={}, identity={:?}",
                    report.units().len(), report.diagnostic_count(), report.unresolved_scripts().len(), report.analysis_id()).map_err(|_| ())?;
            }
            if let Some(report) = check.owner_analysis().and_then(|a| a.xml_binding_report()) {
                writeln!(
                    text,
                    "xml Lua bindings: references={}, unresolved_or_candidates={}, receiver_sources={}, partial_receivers={}, inherited_scripts={}, partial_inherited_scripts={}, identity={:?}",
                    report.bindings().len(),
                    report.unresolved_count(),
                    report.receiver_sources().len(),
                    report
                        .receiver_sources()
                        .values()
                        .filter(|r| !r.complete)
                        .count(),
                    report.inherited_script_sources().len(),
                    report.inherited_script_sources().iter().filter(|source| !source.source_complete).count(),
                    report.analysis_id()
                )
                .map_err(|_| ())?;
            }
            for finding in check.raw_findings() {
                // Serialize the service finding unchanged; JSON string escaping also prevents terminal injection.
                let data = serde_json::to_string(finding).map_err(|_| ())?;
                writeln!(text, "finding: {data}").map_err(|_| ())?;
            }
            for evaluation in check.rule_evaluations() {
                let data = serde_json::to_string(evaluation).map_err(|_| ())?;
                writeln!(text, "evaluation: {data}").map_err(|_| ())?;
            }
            for relation in check.presentation_graph().relations() {
                writeln!(
                    text,
                    "relation: {}",
                    serde_json::to_string(relation).map_err(|_| ())?
                )
                .map_err(|_| ())?;
            }
            writeln!(
                text,
                "runtime: NotEvaluated; clean/findings describe only the selected implemented scope"
            )
            .map_err(|_| ())?;
        }
        LocalOperationResult::Failure(_) | LocalOperationResult::Cancelled(_) => {
            text = String::from_utf8(result.canonical_bytes().map_err(|_| ())?).map_err(|_| ())?;
            text.push('\n');
        }
    }
    if text.len() > 32 * 1024 * 1024 {
        return Err(());
    }
    Ok(text.into_bytes())
}
