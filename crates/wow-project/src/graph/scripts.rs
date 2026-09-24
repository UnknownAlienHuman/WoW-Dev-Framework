//! Captured XML handler sources and callable crosswalks for the recognizer owner.
//! Candidate associations do not select effective handlers or execute inline Lua.
use super::*;
use crate::load::{ProjectLoadPlan, XmlElementRecord, XmlElementRole, XmlScriptSource};
use crate::xml_bindings::{XmlLuaBindingKind, XmlLuaBindingState};
use std::collections::{BTreeMap, BTreeSet};
use wow_emmy::bindings::SymbolLookupState;
use wow_emmy::function_calls::SourceCallTarget;

pub(super) const MAX_HANDLERS: usize = 4096;
pub(super) const MAX_BINDINGS: usize = 8192;
const MAX_SITES: usize = 8192;
const MAX_QUERY_VISITS: usize = 16384;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphScriptSource {
    pub script_id: String,
    pub document: String,
    pub script_name: String,
    pub source_kind: XmlScriptSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaring_owner_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intrinsic_order: Option<String>,
    pub span: SourceSpan,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

/// A source unit, not an Emmy Main function or a generated physical file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphInlineHandler {
    pub script_id: String,
    pub unit_id: String,
    pub document: String,
    pub proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectGraphScriptQueryOutcome {
    MainFunction { function_id: String },
    LibraryTarget,
    NotUnique { lookup_state: SymbolLookupState },
    CallableNotCaptured,
    LoadOrderUnresolved,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphScriptQuery {
    pub query: String,
    pub outcome: ProjectGraphScriptQueryOutcome,
}

/// A direct or inherited source site; all skipped/ambiguous alternatives remain
/// visible even when some independently located callable candidates are retained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphScriptSite {
    pub site_id: String,
    pub script_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consumer_id: Option<String>,
    pub inherited: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_index: Option<usize>,
    pub queries: Vec<ProjectGraphScriptQuery>,
    pub binding_ids: Vec<String>,
    pub blockers: Vec<&'static str>,
}

/// Source-owner fact. Only wow-recognizers turns it into a SetsScript proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphScriptBinding {
    pub binding_id: String,
    pub site_id: String,
    pub receiver_proposal_id: String,
    pub handler_proposal_id: String,
    pub handler_kind: &'static str,
    pub confidence: GraphConfidence,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}

pub(super) struct ScriptProposals {
    pub entities: Vec<GraphEntityProposal>,
    pub relations: Vec<GraphRelationProposal>,
}
struct Endpoint {
    proposal: String,
    kind: &'static str,
    handle: StableHandleId,
    evidence: EvidenceId,
}
struct Site<'a> {
    source: &'a ProjectGraphScriptSource,
    element: &'a XmlElementRecord,
    consumer: Option<&'a str>,
    inherited: bool,
    complete: bool,
}
struct Inputs<'a> {
    project: &'a ProjectView,
    plan: &'a ProjectLoadPlan,
    receivers: BTreeMap<String, Endpoint>,
    functions: BTreeMap<String, Endpoint>,
    function_facts: BTreeMap<&'a str, &'a wow_emmy::function_calls::SourceFunctionFact>,
    inline: BTreeMap<String, Endpoint>,
    bindings: BTreeMap<(&'a str, Option<&'a str>), usize>,
    loads: BTreeMap<&'a str, (u64, usize)>,
}

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<ScriptProposals> {
    let mut output = ScriptProposals {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    let Some(plan) = project.configuration().load_plan() else {
        return Ok(output);
    };
    let analyzer = project.snapshot().analyzer_binding();
    let Some(functions) = analyzer.function_call_report() else {
        return Ok(output);
    };
    let xml_bindings = analyzer.xml_bindings();
    if functions.symbol_lookup_analysis_id()
        != xml_bindings
            .and_then(|b| b.symbol_lookup())
            .map(|r| r.analysis_id())
    {
        return Err(invalid());
    }
    let mut loads = BTreeMap::new();
    for record in plan.records() {
        crate::analyzer::checkpoint(stop)?;
        if record.kind == LoadRecordKind::LuaFile
            && record.selection == LoadSelection::Included
            && let Some(path) = record.target.as_deref()
        {
            let entry = loads.entry(path).or_insert((record.ordinal, 0usize));
            entry.1 += 1;
        }
    }
    let mut bindings = BTreeMap::new();
    if let Some(report) = xml_bindings {
        for (index, binding) in report.bindings().iter().enumerate() {
            crate::analyzer::checkpoint(stop)?;
            if binding.kind != XmlLuaBindingKind::Mixin
                && bindings
                    .insert(
                        (binding.element_id.as_str(), binding.consumer_id.as_deref()),
                        index,
                    )
                    .is_some()
            {
                // Invalid mixed function/method attributes already have an
                // InvalidSource receipt; they must not select an arbitrary row.
                let element = plan
                    .xml_documents()
                    .get(&binding.document)
                    .and_then(|d| d.element(&binding.element_id))
                    .ok_or_else(invalid)?;
                if element
                    .script
                    .as_ref()
                    .is_none_or(|s| s.source_kind != XmlScriptSource::Unresolved)
                {
                    return Err(invalid());
                }
            }
        }
    }
    // These small crosswalks borrow project-owned immutable facts rather than
    // trusting spelling or copying source bodies into graph nodes.
    let receivers = provenance
        .xml_declarations
        .iter()
        .map(|d| {
            (
                d.occurrence_id.clone(),
                Endpoint {
                    proposal: d.proposal_id.clone(),
                    kind: "xml_source_declaration",
                    handle: d.source_handle_id,
                    evidence: d.evidence_id,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let functions_by_id = provenance
        .functions
        .iter()
        .map(|f| {
            (
                f.function_id.clone(),
                Endpoint {
                    proposal: f.proposal_id.clone(),
                    kind: "lua_source_function",
                    handle: f.source_handle_id,
                    evidence: f.evidence_id,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut inline = BTreeMap::new();
    let mut sources = Vec::new();
    let mut elements = BTreeMap::new();
    let parsed = analyzer
        .xml_lua_analysis()
        .map(|r| {
            r.units()
                .iter()
                .map(|u| (u.script_occurrence_id.as_str(), u))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let source_files = plan
        .sources()
        .iter()
        .map(|f| (f.path.as_str(), f))
        .collect::<BTreeMap<_, _>>();
    for (document, index) in plan.xml_documents() {
        for element in index
            .scripts()
            .filter(|e| e.ui_namespace && e.role == XmlElementRole::ScriptBinding)
        {
            crate::analyzer::checkpoint(stop)?;
            if sources.len() >= MAX_HANDLERS {
                return Err(exhausted());
            }
            let script = element.script.as_ref().ok_or_else(invalid)?;
            let file = *source_files.get(document.as_str()).ok_or_else(invalid)?;
            let span = xml::source_span(plan, document, &element.span)?;
            let (handle, evidence) = support(project, file, span, provenance)?;
            charge(
                text_bytes,
                document.len().saturating_mul(6)
                    + element.occurrence_id.len().saturating_mul(6)
                    + element.qualified_name.len().saturating_mul(3)
                    + script
                        .inherit
                        .as_ref()
                        .map_or(0, String::len)
                        .saturating_mul(3)
                    + script
                        .intrinsic_order
                        .as_ref()
                        .map_or(0, String::len)
                        .saturating_mul(3)
                    + 768,
            )?;
            sources.push(ProjectGraphScriptSource {
                script_id: element.occurrence_id.clone(),
                document: document.clone(),
                script_name: element.qualified_name.clone(),
                source_kind: script.source_kind,
                declaring_owner_id: script.owner_occurrence_id.clone(),
                inherit: script.inherit.clone(),
                intrinsic_order: script.intrinsic_order.clone(),
                span,
                source_handle_id: handle,
                evidence_id: evidence,
            });
            if elements
                .insert(element.occurrence_id.as_str(), element)
                .is_some()
            {
                return Err(invalid());
            }
            if script.source_kind == XmlScriptSource::InlineBody && element.issues.is_empty() {
                let unit = parsed
                    .get(element.occurrence_id.as_str())
                    .ok_or_else(invalid)?;
                let body = script.inline_lua.as_ref().ok_or_else(invalid)?;
                if unit.document != *document
                    || unit.document_digest != index.source_digest()
                    || unit.extracted_unit_id != body.unit_id
                    || unit.content_digest != body.content_digest
                    || unit.byte_length != body.byte_length
                {
                    return Err(invalid());
                }
                if !unit.diagnostics.is_empty() {
                    continue;
                }
                let proposal_id = format!("xml-handler:{}", element.occurrence_id);
                output.entities.push(
                    GraphEntityProposal::new(
                        proposal_id.as_str(),
                        "xml_source_handler",
                        BTreeMap::from([
                            (
                                "document".into(),
                                GraphProposalValue::String(document.clone().into()),
                            ),
                            (
                                "occurrence".into(),
                                GraphProposalValue::Identifier(
                                    element.occurrence_id.clone().into(),
                                ),
                            ),
                        ]),
                        GraphConfidence::Derived,
                        vec![handle],
                        vec![evidence],
                        Vec::new(),
                    )
                    .map_err(|_| invalid())?,
                );
                output.relations.push(
                    GraphRelationProposal::new(
                        format!("xml-handler-owner:{}", element.occurrence_id),
                        "source_declaration_owns",
                        GraphRelationProposalInput {
                            source: GraphProposalEndpoint::Proposed(
                                file_ids
                                    .get(document.as_str())
                                    .ok_or_else(invalid)?
                                    .clone()
                                    .into(),
                            ),
                            target: GraphProposalEndpoint::Proposed(proposal_id.clone().into()),
                            confidence: GraphConfidence::Derived,
                            source_handle_ids: vec![handle],
                            evidence_ids: vec![evidence],
                            coverage_ids: Vec::new(),
                        },
                    )
                    .map_err(|_| invalid())?,
                );
                inline.insert(
                    element.occurrence_id.clone(),
                    Endpoint {
                        proposal: proposal_id.clone(),
                        kind: "xml_source_handler",
                        handle,
                        evidence,
                    },
                );
                provenance.inline_handlers.push(ProjectGraphInlineHandler {
                    script_id: element.occurrence_id.clone(),
                    unit_id: unit.unit_id.to_string(),
                    document: document.clone(),
                    proposal_id,
                    source_handle_id: handle,
                    evidence_id: evidence,
                });
            }
        }
    }
    let inputs = Inputs {
        project,
        plan,
        receivers,
        functions: functions_by_id,
        function_facts: functions
            .functions()
            .iter()
            .map(|f| (f.fact_id(), f))
            .collect(),
        inline,
        bindings,
        loads,
    };
    let mut query_visits = 0;
    for source in &sources {
        collect_site(
            &inputs,
            Site {
                source,
                element: elements[source.script_id.as_str()],
                consumer: source.declaring_owner_id.as_deref(),
                inherited: false,
                complete: true,
            },
            provenance,
            text_bytes,
            &mut query_visits,
            stop,
        )?;
    }
    let source_by_id = sources
        .iter()
        .map(|s| (s.script_id.as_str(), s))
        .collect::<BTreeMap<_, _>>();
    if let Some(report) = xml_bindings {
        for inherited in report.inherited_script_sources() {
            crate::analyzer::checkpoint(stop)?;
            let source = *source_by_id
                .get(inherited.script_id.as_str())
                .ok_or_else(invalid)?;
            if source.declaring_owner_id.as_deref() != Some(inherited.declaring_owner_id.as_str()) {
                return Err(invalid());
            }
            collect_site(
                &inputs,
                Site {
                    source,
                    element: elements[source.script_id.as_str()],
                    consumer: Some(&inherited.consumer_id),
                    inherited: true,
                    complete: inherited.source_complete,
                },
                provenance,
                text_bytes,
                &mut query_visits,
                stop,
            )?;
        }
    }
    provenance.script_sources = sources;
    provenance.xml_lua_analysis = analyzer.xml_lua_analysis().cloned();
    // The mixin stage may have no entries; script receipts still need the entire
    // original binding and inherited-source graph, not a synthetic subset.
    provenance.xml_binding_report = xml_bindings.cloned();
    Ok(output)
}

fn collect_site(
    inputs: &Inputs<'_>,
    site: Site<'_>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    query_visits: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    if provenance.script_sites.len() >= MAX_SITES {
        return Err(exhausted());
    }
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-script-site/1",
        &(&site.source.script_id, site.consumer, site.inherited),
        ProjectPhase::View,
    )?;
    let mut receipt = ProjectGraphScriptSite {
        site_id: format!("xml-script-site:{digest}"),
        script_id: site.source.script_id.clone(),
        consumer_id: site.consumer.map(str::to_owned),
        inherited: site.inherited,
        binding_index: None,
        queries: Vec::new(),
        binding_ids: Vec::new(),
        blockers: Vec::new(),
    };
    charge(
        text_bytes,
        site.source.script_id.len().saturating_mul(3)
            + site.consumer.map_or(0, str::len).saturating_mul(3)
            + 512,
    )?;
    let Some(consumer) = site.consumer else {
        receipt.blockers.push("owner_not_captured");
        provenance.script_sites.push(receipt);
        return Ok(());
    };
    let declaration = inputs
        .plan
        .xml_references()
        .declarations()
        .get(consumer)
        .ok_or_else(invalid)?;
    if !site.element.issues.is_empty()
        || !declaration.valid_declaration
        || site.source.source_kind == XmlScriptSource::Unresolved
    {
        receipt.blockers.push("invalid_source");
    } else if !site.complete {
        receipt.blockers.push("inherited_sources_incomplete");
    } else if declaration.load_ordinals.len() != 1 {
        receipt.blockers.push("consumer_load_order_unresolved");
    }
    if !receipt.blockers.is_empty() {
        provenance.script_sites.push(receipt);
        return Ok(());
    }
    let receiver = inputs.receivers.get(consumer).ok_or_else(invalid)?;
    match site.source.source_kind {
        XmlScriptSource::InlineBody => {
            if let Some(handler) = inputs.inline.get(&site.source.script_id) {
                add_binding(
                    &site,
                    receiver,
                    handler,
                    if site.inherited {
                        GraphConfidence::Possible
                    } else {
                        GraphConfidence::Derived
                    },
                    &mut receipt,
                    provenance,
                    text_bytes,
                )?;
            } else {
                receipt.blockers.push("inline_parse_failed");
            }
        }
        XmlScriptSource::ReferenceOnly => {
            let analyzer = inputs.project.snapshot().analyzer_binding();
            let report = analyzer.xml_bindings().ok_or_else(invalid)?;
            let index = *inputs
                .bindings
                .get(&(
                    site.source.script_id.as_str(),
                    site.inherited.then_some(consumer),
                ))
                .ok_or_else(invalid)?;
            receipt.binding_index = Some(index);
            let binding = report.bindings().get(index).ok_or_else(invalid)?;
            let is_method = binding.kind == XmlLuaBindingKind::Method;
            if matches!(
                binding.state,
                XmlLuaBindingState::InvalidSource
                    | XmlLuaBindingState::SourceParseFailed
                    | XmlLuaBindingState::UnsupportedPath
                    | XmlLuaBindingState::ReceiverNotResolved
            ) {
                receipt.blockers.push("binding_not_resolved");
            } else {
                let lookup = report.symbol_lookup().ok_or_else(invalid)?;
                let functions = analyzer.function_call_report().ok_or_else(invalid)?;
                let mut targets = BTreeSet::new();
                for query in &binding.queries {
                    crate::analyzer::checkpoint(stop)?;
                    *query_visits = query_visits.checked_add(1).ok_or_else(exhausted)?;
                    if *query_visits > MAX_QUERY_VISITS {
                        return Err(exhausted());
                    }
                    charge(text_bytes, query.len().saturating_add(256))?;
                    let resolved = lookup.lookups().get(query).ok_or_else(invalid)?;
                    let outcome = if resolved.state != SymbolLookupState::UniqueAnalyzerDeclaration
                    {
                        ProjectGraphScriptQueryOutcome::NotUnique {
                            lookup_state: resolved.state,
                        }
                    } else {
                        match functions.named_targets().get(query) {
                            Some(SourceCallTarget::MainFunction { function_id }) => {
                                let target = *inputs
                                    .function_facts
                                    .get(function_id.as_str())
                                    .ok_or_else(invalid)?;
                                let source_owner = site
                                    .source
                                    .declaring_owner_id
                                    .as_ref()
                                    .and_then(|id| {
                                        inputs.plan.xml_references().declarations().get(id)
                                    })
                                    .ok_or_else(invalid)?;
                                let valid_order = match (
                                    inputs.loads.get(target.path()),
                                    declaration.load_ordinals.as_slice(),
                                ) {
                                    (Some((loaded, 1)), [consumer_load])
                                        if loaded < consumer_load =>
                                    {
                                        is_method
                                            || matches!(source_owner.load_ordinals.as_slice(), [source_load] if loaded < source_load)
                                    }
                                    _ => false,
                                };
                                if valid_order {
                                    targets.insert(function_id.as_str());
                                    ProjectGraphScriptQueryOutcome::MainFunction {
                                        function_id: function_id.clone(),
                                    }
                                } else {
                                    ProjectGraphScriptQueryOutcome::LoadOrderUnresolved
                                }
                            }
                            Some(SourceCallTarget::LibraryFunction { .. }) => {
                                ProjectGraphScriptQueryOutcome::LibraryTarget
                            }
                            _ => ProjectGraphScriptQueryOutcome::CallableNotCaptured,
                        }
                    };
                    receipt.queries.push(ProjectGraphScriptQuery {
                        query: query.clone(),
                        outcome,
                    });
                }
                // Deduplicate aliases only after resolving concrete closure IDs.
                // Method and inherited associations never become effective dispatch.
                for id in targets {
                    let target = inputs.functions.get(id).ok_or_else(invalid)?;
                    add_binding(
                        &site,
                        receiver,
                        target,
                        if is_method || site.inherited {
                            GraphConfidence::Possible
                        } else {
                            GraphConfidence::Derived
                        },
                        &mut receipt,
                        provenance,
                        text_bytes,
                    )?;
                }
                if is_method {
                    receipt.blockers.push("method_dispatch_not_evaluated");
                }
            }
        }
        XmlScriptSource::ExternalFile => {
            receipt.blockers.push("external_script_file_not_a_callable")
        }
        XmlScriptSource::Unresolved => return Err(invalid()),
    }
    if site.inherited {
        receipt
            .blockers
            .push("inherited_dispatch_order_not_evaluated");
    }
    if receipt.binding_ids.is_empty() {
        receipt.blockers.push("no_admitted_handler");
    }
    provenance.script_sites.push(receipt);
    Ok(())
}

fn add_binding(
    site: &Site<'_>,
    receiver: &Endpoint,
    handler: &Endpoint,
    confidence: GraphConfidence,
    receipt: &mut ProjectGraphScriptSite,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
) -> ProjectResult<()> {
    if provenance.script_bindings.len() >= MAX_BINDINGS {
        return Err(exhausted());
    }
    charge(
        text_bytes,
        receiver.proposal.len() + handler.proposal.len() + 768,
    )?;
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-script-binding/1",
        &(
            &receipt.site_id,
            &receiver.proposal,
            &handler.proposal,
            confidence,
        ),
        ProjectPhase::View,
    )?;
    let binding_id = format!("xml-script-binding:{digest}");
    provenance.script_bindings.push(ProjectGraphScriptBinding {
        binding_id: binding_id.clone(),
        site_id: receipt.site_id.clone(),
        receiver_proposal_id: receiver.proposal.clone(),
        handler_proposal_id: handler.proposal.clone(),
        handler_kind: handler.kind,
        confidence,
        source_handle_ids: BTreeSet::from([
            site.source.source_handle_id,
            receiver.handle,
            handler.handle,
        ])
        .into_iter()
        .collect(),
        evidence_ids: BTreeSet::from([
            site.source.evidence_id,
            receiver.evidence,
            handler.evidence,
        ])
        .into_iter()
        .collect(),
    });
    receipt.binding_ids.push(binding_id);
    Ok(())
}
