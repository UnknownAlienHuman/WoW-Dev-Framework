//! Captured XML handler sources and callable crosswalks for the recognizer owner.
//! Candidate associations do not select effective handlers or execute inline Lua.
use super::*;
use crate::load::{ProjectLoadPlan, XmlElementRecord, XmlElementRole, XmlScriptSource};
use crate::xml_bindings::{XmlLuaBindingKind, XmlLuaBindingState};
use std::collections::{BTreeMap, BTreeSet};
use wow_core::{CanonicalResult, ContentDigest};
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
    pub semantic_context: crate::xml_lua::XmlLuaSemanticContext,
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
    pub semantic_context_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_binding_address: Option<crate::xml_bindings::ProjectPackageXmlLuaBindingAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub semantic_context: Option<crate::xml_lua::XmlLuaSemanticContext>,
    pub confidence: GraphConfidence,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}

pub(super) struct ScriptProposals {
    pub entities: Vec<EntityDraft>,
    pub relations: Vec<RelationDraft>,
}
struct Endpoint {
    proposal: String,
    kind: &'static str,
    semantic_context: Option<crate::xml_lua::XmlLuaSemanticContext>,
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
    scope: &'a load_inputs::LoadPlanGraphInput<'a>,
    xml_bindings: Option<load_inputs::ScopedXmlBindings<'a>>,
    receivers: BTreeMap<&'a str, &'a Endpoint>,
    functions: &'a BTreeMap<String, Endpoint>,
    function_facts: &'a BTreeMap<&'a str, &'a wow_emmy::function_calls::SourceFunctionFact>,
    inline: BTreeMap<String, Endpoint>,
    bindings: BTreeMap<(&'a str, Option<&'a str>), usize>,
    loads: BTreeMap<String, (u64, usize)>,
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
    let scopes = load_inputs::scopes(project, stop)?;
    if scopes.is_empty() {
        return Ok(output);
    }
    let analyzer = project.snapshot().analyzer_binding();
    let Some(functions) = analyzer.function_call_report() else {
        return Ok(output);
    };
    let mut receiver_catalog: BTreeMap<String, BTreeMap<String, Endpoint>> = BTreeMap::new();
    for declaration in &provenance.xml_declarations {
        crate::analyzer::checkpoint(stop)?;
        if receiver_catalog
            .entry(declaration.path.clone())
            .or_default()
            .insert(
                declaration.occurrence_id.clone(),
                Endpoint {
                    proposal: declaration.proposal_id.clone(),
                    kind: "xml_source_declaration",
                    semantic_context: None,
                    handle: declaration.source_handle_id,
                    evidence: declaration.evidence_id,
                },
            )
            .is_some()
        {
            return Err(invalid());
        }
    }
    let functions_by_id = provenance
        .functions
        .iter()
        .map(|function| {
            (
                function.function_id.clone(),
                Endpoint {
                    proposal: function.proposal_id.clone(),
                    kind: "lua_source_function",
                    semantic_context: None,
                    handle: function.source_handle_id,
                    evidence: function.evidence_id,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let function_facts = functions
        .functions()
        .iter()
        .map(|function| (function.fact_id(), function))
        .collect::<BTreeMap<_, _>>();
    let mut parsed = BTreeMap::new();
    if let Some(analysis) = analyzer.xml_lua_analysis() {
        for unit in analysis.units() {
            crate::analyzer::checkpoint(stop)?;
            if parsed
                .insert(
                    (
                        unit.package.as_deref(),
                        unit.document.as_str(),
                        unit.script_occurrence_id.as_str(),
                    ),
                    unit,
                )
                .is_some()
            {
                return Err(invalid());
            }
        }
    }
    let mut all_sources = Vec::new();
    let mut query_visits = 0;
    for scope in &scopes {
        crate::analyzer::checkpoint(stop)?;
        let plan = scope.plan();
        let xml_bindings = scope.bindings()?;
        if xml_bindings
            .as_ref()
            .and_then(|b| b.symbol_lookup)
            .is_some_and(|report| {
                !functions
                    .symbol_lookup_analysis_ids()
                    .iter()
                    .any(|id| id == report.analysis_id())
            })
        {
            return Err(invalid());
        }
        let loads = scope.lua_loads(stop)?;
        let mut bindings = BTreeMap::new();
        if let Some(report) = &xml_bindings {
            for (index, binding) in report.bindings.iter().enumerate() {
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
        let mut receivers = BTreeMap::new();
        for declaration in plan.xml_references().declarations().values() {
            crate::analyzer::checkpoint(stop)?;
            let document = scope.qualified_path(&declaration.document)?;
            if let Some(endpoint) = receiver_catalog
                .get(document.as_str())
                .and_then(|rows| rows.get(declaration.occurrence_id.as_str()))
            {
                receivers.insert(declaration.occurrence_id.as_str(), endpoint);
            }
        }
        let mut inline = BTreeMap::new();
        let mut sources = Vec::new();
        let mut elements = BTreeMap::new();
        for (document, index) in plan.xml_documents() {
            for element in index.scripts().filter(|e| {
                e.ui_namespace
                    && matches!(
                        e.role,
                        XmlElementRole::ScriptBinding | XmlElementRole::Script
                    )
            }) {
                crate::analyzer::checkpoint(stop)?;
                if all_sources.len().saturating_add(sources.len()) >= MAX_HANDLERS {
                    return Err(exhausted());
                }
                let script = element.script.as_ref().ok_or_else(invalid)?;
                let file = scope.mapped_source(document)?;
                let span = xml::source_span(plan, document, &element.span)?;
                let (handle, evidence) = support(project, &file, span, provenance)?;
                charge(
                    text_bytes,
                    file.path.len().saturating_mul(6)
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
                    document: file.path.clone(),
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
                if element.role == XmlElementRole::ScriptBinding
                    && script.source_kind == XmlScriptSource::InlineBody
                    && element.issues.is_empty()
                {
                    if scope.node().is_some_and(|node| {
                        node.reachability == crate::load::ProjectPackageReachability::Unreachable
                    }) {
                        continue;
                    }
                    let unit = parsed
                        .get(&(
                            scope.package(),
                            file.path.as_str(),
                            element.occurrence_id.as_str(),
                        ))
                        .ok_or_else(invalid)?;
                    let body = script.inline_lua.as_ref().ok_or_else(invalid)?;
                    if unit.document != file.path
                        || unit.package.as_deref() != scope.package()
                        || unit.document_digest != index.source_digest()
                        || unit.extracted_unit_id != body.unit_id
                        || unit.content_digest != body.content_digest
                        || unit.byte_length != body.byte_length
                        || !unit.context.admits_static_source_association()
                    {
                        return Err(invalid());
                    }
                    if !unit.diagnostics.is_empty() {
                        continue;
                    }
                    let proposal_id =
                        scope.proposal_id("xml-handler", document, &element.occurrence_id)?;
                    output.entities.push(EntityDraft::new(
                        PlatformGraphProducer::XmlStructure,
                        GraphEntityProposal::new(
                            proposal_id.as_str(),
                            "xml_source_handler",
                            BTreeMap::from([
                                (
                                    "document".into(),
                                    GraphProposalValue::String(file.path.clone().into()),
                                ),
                                (
                                    "occurrence".into(),
                                    GraphProposalValue::Identifier(
                                        element.occurrence_id.clone().into(),
                                    ),
                                ),
                                (
                                    "semantic_context_id".into(),
                                    GraphProposalValue::String(
                                        unit.context.context_id().to_owned().into(),
                                    ),
                                ),
                            ]),
                            GraphConfidence::Derived,
                            vec![handle],
                            vec![evidence],
                            Vec::new(),
                        )
                        .map_err(|_| invalid())?,
                    ));
                    output.relations.push(
                        RelationDraft::new(
                            PlatformGraphProducer::XmlStructure,
                            scope.proposal_id(
                                "xml-handler-owner",
                                document,
                                &element.occurrence_id,
                            )?,
                            "source_declaration_owns",
                            GraphRelationProposalInput {
                                source: GraphProposalEndpoint::Proposed(
                                    file_ids
                                        .get(file.path.as_str())
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
                            semantic_context: Some(unit.context.clone()),
                            handle,
                            evidence,
                        },
                    );
                    provenance.inline_handlers.push(ProjectGraphInlineHandler {
                        script_id: element.occurrence_id.clone(),
                        unit_id: unit.unit_id.to_string(),
                        document: file.path.clone(),
                        semantic_context: unit.context.clone(),
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
            scope,
            xml_bindings,
            receivers,
            functions: &functions_by_id,
            function_facts: &function_facts,
            inline,
            bindings,
            loads,
        };
        for source in &sources {
            collect_site(
                &inputs,
                Site {
                    source,
                    element: elements[source.script_id.as_str()],
                    // Script chunks have lexical owners but no callback receiver.
                    // Keep raw ownership and the parsed unit in their source reports.
                    consumer: source.declaring_owner_id.as_deref().filter(|_| {
                        elements[source.script_id.as_str()].role != XmlElementRole::Script
                    }),
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
        if let Some(report) = &inputs.xml_bindings {
            for inherited in report.inherited_script_sources {
                crate::analyzer::checkpoint(stop)?;
                let source = *source_by_id
                    .get(inherited.script_id.as_str())
                    .ok_or_else(invalid)?;
                if source.declaring_owner_id.as_deref()
                    != Some(inherited.declaring_owner_id.as_str())
                {
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
        all_sources.extend(sources);
    }
    provenance.script_sources = all_sources;
    provenance.xml_lua_analysis = analyzer.xml_lua_analysis().cloned();
    // The mixin stage may have no entries; script receipts still need the entire
    // original binding and inherited-source graph, not a synthetic subset.
    if project.configuration().platform_graph_profile().is_none() {
        provenance.xml_binding_report = analyzer.xml_bindings().cloned();
    }
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
    #[derive(Serialize)]
    struct SiteIdentity<'a> {
        script_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        consumer: Option<&'a str>,
        inherited: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        package: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        document: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        load_plan_digest: Option<ContentDigest<CanonicalResult>>,
    }
    let package = inputs.scope.package();
    let document = package.map(|_| site.source.document.as_str());
    charge(
        text_bytes,
        site.source.script_id.len().saturating_mul(3)
            + site.consumer.map_or(0, str::len).saturating_mul(3)
            + package.map_or(0, str::len)
            + document.map_or(0, str::len)
            + 512,
    )?;
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-script-site/2",
        &SiteIdentity {
            script_id: &site.source.script_id,
            consumer: site.consumer,
            inherited: site.inherited,
            package,
            document,
            load_plan_digest: package.map(|_| inputs.plan.digest()),
        },
        ProjectPhase::View,
    )?;
    let mut receipt = ProjectGraphScriptSite {
        site_id: format!("xml-script-site:{digest}"),
        script_id: site.source.script_id.clone(),
        consumer_id: site.consumer.map(str::to_owned),
        inherited: site.inherited,
        semantic_context_id: None,
        binding_index: None,
        package_binding_address: None,
        package: package.map(str::to_owned),
        document: document.map(str::to_owned),
        queries: Vec::new(),
        binding_ids: Vec::new(),
        blockers: Vec::new(),
    };
    let unreachable = inputs.scope.node().is_some_and(|node| {
        node.reachability == crate::load::ProjectPackageReachability::Unreachable
    });
    if unreachable {
        receipt.blockers.push("package_unreachable");
    }
    let Some(consumer) = site.consumer else {
        receipt.blockers.push("owner_not_captured");
        provenance.script_sites.push(receipt);
        return Ok(());
    };
    if unreachable {
        provenance.script_sites.push(receipt);
        return Ok(());
    }
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
                let context = handler.semantic_context.as_ref().ok_or_else(invalid)?;
                if !context.admits_static_source_association() {
                    return Err(invalid());
                }
                receipt.semantic_context_id = Some(context.context_id().to_owned());
                add_binding(
                    &site,
                    receiver,
                    handler,
                    GraphConfidence::Possible,
                    &mut receipt,
                    provenance,
                    text_bytes,
                )?;
                receipt.blockers.push("implicit_receiver_not_evaluated");
                receipt.blockers.push("runtime_dispatch_not_evaluated");
            } else {
                receipt.blockers.push("inline_parse_failed");
            }
        }
        XmlScriptSource::ReferenceOnly => {
            let analyzer = inputs.project.snapshot().analyzer_binding();
            let report = inputs.xml_bindings.as_ref().ok_or_else(invalid)?;
            let index = *inputs
                .bindings
                .get(&(
                    site.source.script_id.as_str(),
                    site.inherited.then_some(consumer),
                ))
                .ok_or_else(invalid)?;
            receipt.binding_index = Some(index);
            let package_binding_address = report.binding_address(index)?;
            if let Some(address) = &package_binding_address {
                charge(
                    text_bytes,
                    address
                        .analysis_id()
                        .len()
                        .saturating_add(address.package().len())
                        .saturating_add(256),
                )?;
            }
            receipt.package_binding_address = package_binding_address;
            let binding = report.bindings.get(index).ok_or_else(invalid)?;
            let is_method = binding.kind == XmlLuaBindingKind::Method;
            if let Some(id) = &binding.receiver_source_id
                && report
                    .receiver_sources
                    .get(id)
                    .is_none_or(|sources| sources.owner_id != consumer)
            {
                return Err(invalid());
            }
            if matches!(
                binding.state,
                XmlLuaBindingState::InvalidSource
                    | XmlLuaBindingState::SourceParseFailed
                    | XmlLuaBindingState::UnsupportedPath
                    | XmlLuaBindingState::ReceiverNotResolved
            ) {
                receipt.blockers.push("binding_not_resolved");
            } else {
                let lookup = report.symbol_lookup.ok_or_else(invalid)?;
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
    let semantic_context = handler.semantic_context.clone();
    if handler.kind == "xml_source_handler" {
        let context = semantic_context.as_ref().ok_or_else(invalid)?;
        if !context.admits_static_source_association() || confidence != GraphConfidence::Possible {
            return Err(invalid());
        }
    } else if semantic_context.is_some() {
        return Err(invalid());
    }
    charge(
        text_bytes,
        receiver.proposal.len()
            + handler.proposal.len()
            + semantic_context.as_ref().map_or(0, |context| {
                context.profile().len()
                    + context.context_id().len()
                    + context.script_site().len()
                    + context.implicit_receiver().len()
                    + context.runtime_dispatch().len()
            })
            + 768,
    )?;
    #[derive(Serialize)]
    struct BindingIdentity<'a> {
        site_id: &'a str,
        receiver: &'a str,
        handler: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        semantic_context: Option<&'a crate::xml_lua::XmlLuaSemanticContext>,
        confidence: GraphConfidence,
    }
    let digest = crate::identity::canonical_digest(
        "wow-project/xml-script-binding/3",
        &BindingIdentity {
            site_id: &receipt.site_id,
            receiver: &receiver.proposal,
            handler: &handler.proposal,
            semantic_context: semantic_context.as_ref(),
            confidence,
        },
        ProjectPhase::View,
    )?;
    let binding_id = format!("xml-script-binding:{digest}");
    provenance.script_bindings.push(ProjectGraphScriptBinding {
        binding_id: binding_id.clone(),
        site_id: receipt.site_id.clone(),
        receiver_proposal_id: receiver.proposal.clone(),
        handler_proposal_id: handler.proposal.clone(),
        handler_kind: handler.kind,
        semantic_context,
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
