//! Explicit XML mixin source references resolved by the retained Emmy report.
//! No table construction, method precedence, runtime binding or Library merging.
use super::*;
use crate::load::ProjectLoadPlan;
use crate::xml_bindings::{XmlLuaBinding, XmlLuaBindingKind, XmlLuaBindingState};
use wow_emmy::bindings::{SymbolLookupState, SymbolTarget};

pub(super) const MAX_DECLARATIONS: usize = 4096;
pub(super) const MAX_REFERENCES: usize = 4096;

/// One exact Main declaration location. Names/aliases do not establish identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphLuaDeclaration {
    pub declaration_id: String,
    pub path: String,
    pub span: SourceSpan,
    pub proposal_id: String,
    pub ownership_proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProjectGraphMixinOutcome {
    Projected {
        proposal_id: String,
        lua_declaration_id: String,
    },
    Unresolved {
        binding_state: XmlLuaBindingState,
    },
    InvalidDeclaration,
    LibraryTarget,
    UnsupportedTargetSpan,
    LoadOrderUnresolved,
}

/// `binding_index` addresses the complete retained XML binding report, including
/// the original attribute span, ordered spelling and shared lookup candidates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphMixinReference {
    pub binding_index: usize,
    pub element_id: String,
    pub ordinal: usize,
    pub outcome: ProjectGraphMixinOutcome,
}

pub(super) struct MixinProposals {
    pub entities: Vec<GraphEntityProposal>,
    pub relations: Vec<GraphRelationProposal>,
}

/// Exactly one complete Main declaration lookup may cross into the source graph.
/// Mismatch of captured owners aborts; incomplete observations get a receipt.
fn target<'a>(
    project: &'a ProjectView,
    plan: &ProjectLoadPlan,
    binding: &XmlLuaBinding,
    loads: &BTreeMap<&str, (u64, usize)>,
) -> ProjectResult<Result<&'a SymbolTarget, ProjectGraphMixinOutcome>> {
    if binding.consumer_id.is_some() || binding.receiver_source_id.is_some() {
        return Err(invalid());
    }
    let declaration = plan
        .xml_references()
        .declarations()
        .get(&binding.element_id)
        .ok_or_else(invalid)?;
    if declaration.document != binding.document
        || declaration.content_digest != binding.content_digest
    {
        return Err(invalid());
    }
    if !declaration.valid_declaration || binding.state == XmlLuaBindingState::InvalidSource {
        return Ok(Err(ProjectGraphMixinOutcome::InvalidDeclaration));
    }
    if binding.state != XmlLuaBindingState::UniqueAnalyzerDeclaration {
        return Ok(Err(ProjectGraphMixinOutcome::Unresolved {
            binding_state: binding.state,
        }));
    }
    let [query] = binding.queries.as_slice() else {
        return Err(invalid());
    };
    let analyzer = project.snapshot().analyzer_binding();
    let lookup = analyzer
        .xml_bindings()
        .and_then(|r| r.symbol_lookup())
        .ok_or_else(invalid)?;
    let resolved = lookup.lookups().get(query).ok_or_else(invalid)?;
    if !lookup.source_health_complete()
        || resolved.state != SymbolLookupState::UniqueAnalyzerDeclaration
        || resolved.resolved_components != query.split('.').count()
    {
        return Err(invalid());
    }
    let [target] = resolved.targets.as_slice() else {
        return Err(invalid());
    };
    if target.role == "library" {
        if !analyzer
            .library_snapshot_ids()
            .any(|id| id == target.workspace_id)
        {
            return Err(invalid());
        }
        // Matching a Library annotation is not a declaration in the addon, even
        // if its relative path happens to match a first-party file.
        return Ok(Err(ProjectGraphMixinOutcome::LibraryTarget));
    }
    if target.role != "main" || target.workspace_id != analyzer.main_workspace().snapshot_id() {
        return Err(invalid());
    }
    let file = analyzer
        .main_workspace()
        .file(&target.path)
        .ok_or_else(invalid)?;
    if file.content_sha256() != target.content_digest {
        return Err(invalid());
    }
    target.span.validate().map_err(|_| invalid())?;
    let (Some(start), Some(end)) = (target.span.byte_start(), target.span.byte_end()) else {
        return Ok(Err(ProjectGraphMixinOutcome::UnsupportedTargetSpan));
    };
    let start = usize::try_from(start).map_err(|_| invalid())?;
    let end = usize::try_from(end).map_err(|_| invalid())?;
    file.text().get(start..end).ok_or_else(invalid)?;
    if start == end {
        return Ok(Err(ProjectGraphMixinOutcome::UnsupportedTargetSpan));
    }
    // This is only a static source-order guard. It does not attest that an
    // assignment executes, that it yields a table or that the client loads it.
    match (
        loads.get(target.path.as_str()),
        declaration.load_ordinals.as_slice(),
    ) {
        (Some((loaded, 1)), [declared]) if loaded < declared => Ok(Ok(target)),
        _ => Ok(Err(ProjectGraphMixinOutcome::LoadOrderUnresolved)),
    }
}

fn add_declaration(
    project: &ProjectView,
    target: &SymbolTarget,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    output: &mut MixinProposals,
) -> ProjectResult<usize> {
    if provenance.lua_declarations.len() >= MAX_DECLARATIONS {
        return Err(exhausted());
    }
    let file = project.file_by_path(&target.path)?.ok_or_else(invalid)?;
    let file_id = file_ids.get(target.path.as_str()).ok_or_else(invalid)?;
    let source = LoadSource {
        path: target.path.clone(),
        content_digest: file.content_digest(),
        byte_length: file.byte_length(),
    };
    let digest = crate::identity::canonical_digest(
        "wow-project/lua-source-declaration/1",
        &(&target.workspace_id, &target.path, target.span),
        ProjectPhase::View,
    )?;
    let declaration_id = format!("lua-declaration:{digest}");
    let proposal_id = format!("lua:{digest}");
    let ownership_proposal_id = format!("lua-owner:{digest}");
    charge(text_bytes, target.path.len().saturating_mul(10) + 1024)?;
    let (handle, evidence) = support(project, &source, target.span, provenance)?;
    let start =
        i64::try_from(target.span.byte_start().ok_or_else(invalid)?).map_err(|_| invalid())?;
    let end = i64::try_from(target.span.byte_end().ok_or_else(invalid)?).map_err(|_| invalid())?;
    output.entities.push(
        GraphEntityProposal::new(
            proposal_id.as_str(),
            "lua_source_declaration",
            BTreeMap::from([
                (
                    "document".into(),
                    GraphProposalValue::String(target.path.clone().into()),
                ),
                ("span_start".into(), GraphProposalValue::Integer(start)),
                ("span_end".into(), GraphProposalValue::Integer(end)),
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
            ownership_proposal_id.as_str(),
            "source_declaration_owns",
            GraphRelationProposalInput {
                source: GraphProposalEndpoint::Proposed(file_id.clone().into()),
                target: GraphProposalEndpoint::Proposed(proposal_id.clone().into()),
                confidence: GraphConfidence::Derived,
                source_handle_ids: vec![handle],
                evidence_ids: vec![evidence],
                coverage_ids: Vec::new(),
            },
        )
        .map_err(|_| invalid())?,
    );
    let index = provenance.lua_declarations.len();
    provenance
        .lua_declarations
        .push(ProjectGraphLuaDeclaration {
            declaration_id,
            path: target.path.clone(),
            span: target.span,
            proposal_id,
            ownership_proposal_id,
            source_handle_id: handle,
            evidence_id: evidence,
        });
    Ok(index)
}

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<MixinProposals> {
    let mut output = MixinProposals {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    let Some(plan) = project.configuration().load_plan() else {
        return Ok(output);
    };
    let report = project
        .snapshot()
        .analyzer_binding()
        .xml_bindings()
        .ok_or_else(invalid)?;
    let count = report
        .bindings()
        .iter()
        .filter(|b| b.kind == XmlLuaBindingKind::Mixin)
        .count();
    if count == 0 {
        return Ok(output);
    }
    if count > MAX_REFERENCES {
        return Err(exhausted());
    }
    let mut loads = BTreeMap::new();
    for record in plan.records() {
        crate::analyzer::checkpoint(stop)?;
        if record.kind == LoadRecordKind::LuaFile
            && record.selection == LoadSelection::Included
            && let Some(path) = record.target.as_deref()
        {
            let load = loads.entry(path).or_insert((record.ordinal, 0usize));
            load.1 += 1;
        }
    }
    let sources: BTreeMap<_, _> = plan
        .sources()
        .iter()
        .map(|s| (s.path.as_str(), s))
        .collect();
    let xml_ids: BTreeMap<_, _> = provenance
        .xml_declarations
        .iter()
        .map(|d| (d.occurrence_id.clone(), d.proposal_id.clone()))
        .collect();
    let mut lua_ids = BTreeMap::new();
    for (binding_index, binding) in report.bindings().iter().enumerate() {
        crate::analyzer::checkpoint(stop)?;
        if binding.kind != XmlLuaBindingKind::Mixin {
            continue;
        }
        charge(
            text_bytes,
            binding.document.len().saturating_mul(4)
                + binding.element_id.len().saturating_mul(6)
                + 512,
        )?;
        let outcome = match target(project, plan, binding, &loads)? {
            Err(outcome) => outcome,
            Ok(target) => {
                let key = (target.path.clone(), target.span);
                let target_index = match lua_ids.get(&key) {
                    Some(index) => *index,
                    None => {
                        let index = add_declaration(
                            project,
                            target,
                            file_ids,
                            provenance,
                            text_bytes,
                            &mut output,
                        )?;
                        lua_ids.insert(key, index);
                        index
                    }
                };
                let source = sources.get(binding.document.as_str()).ok_or_else(invalid)?;
                let span = xml::source_span(plan, &binding.document, &binding.attribute_span)?;
                let (handle, evidence) = support(project, source, span, provenance)?;
                let lua = provenance
                    .lua_declarations
                    .get(target_index)
                    .ok_or_else(invalid)?;
                let source_id = xml_ids.get(&binding.element_id).ok_or_else(invalid)?;
                let proposal_id = format!("xml-mixin:{}:{}", binding.element_id, binding.ordinal);
                output.relations.push(
                    GraphRelationProposal::new(
                        proposal_id.as_str(),
                        "source_xml_mixes_in",
                        GraphRelationProposalInput {
                            source: GraphProposalEndpoint::Proposed(source_id.clone().into()),
                            target: GraphProposalEndpoint::Proposed(lua.proposal_id.clone().into()),
                            confidence: GraphConfidence::Derived,
                            source_handle_ids: vec![handle, lua.source_handle_id],
                            evidence_ids: vec![evidence, lua.evidence_id],
                            coverage_ids: Vec::new(),
                        },
                    )
                    .map_err(|_| invalid())?,
                );
                ProjectGraphMixinOutcome::Projected {
                    proposal_id,
                    lua_declaration_id: lua.declaration_id.clone(),
                }
            }
        };
        provenance.xml_mixins.push(ProjectGraphMixinReference {
            binding_index,
            element_id: binding.element_id.clone(),
            ordinal: binding.ordinal,
            outcome,
        });
    }
    // Keep the original owner receipt, not a fabricated "complete" subset of it.
    // It includes shared lookup candidates and exact analyzer/source identities.
    crate::analyzer::checkpoint(stop)?;
    provenance.xml_binding_report = Some(report.clone());
    crate::analyzer::checkpoint(stop)?;
    Ok(output)
}
