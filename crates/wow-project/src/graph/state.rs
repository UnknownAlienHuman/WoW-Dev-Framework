//! TOC-defined state namespaces joined to exact analyzer global-slot accesses.
//! State paths are symbolic source paths, not existing runtime fields or values.
use super::*;
use crate::load::{TocSavedVariableScope, TocSavedVariableState};
use std::collections::BTreeSet;
use wow_emmy::global_access::{AliasAccessBlocker, GlobalAccessKind, GlobalAccessResolution};

pub(super) const MAX_ROOTS: usize = 1024;
pub(super) const MAX_PATHS: usize = 8192;
pub(super) const MAX_ACCESSES: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphStateDeclaration {
    pub record_ordinal: u64,
    pub entry_ordinal: u32,
    pub name: String,
    pub scope: TocSavedVariableScope,
    pub state: TocSavedVariableState,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphStateRoot {
    pub root_id: String,
    pub name: String,
    pub scope: TocSavedVariableScope,
    pub document: String,
    pub proposal_id: String,
    pub ambiguous: bool,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphStatePath {
    pub path_id: String,
    pub root_id: String,
    pub keys: Vec<GlobalAccessKey>,
    pub proposal_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectGraphStateOutcome {
    Projected,
    AmbiguousDeclaration,
    UnresolvedTocSelection,
    UnresolvedGlobal,
    LibraryGlobal,
    DeclarationSourceIncomplete,
    DynamicOrUnsupportedKey,
    ReassignedAlias,
    LocalAliasRebinding,
    UnsupportedAssignment,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphStateSite {
    pub access_id: String,
    pub outcome: ProjectGraphStateOutcome,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphStateBinding {
    pub binding_id: String,
    pub access_id: String,
    pub root_id: String,
    pub caller_proposal_id: String,
    pub target_proposal_id: String,
    pub kind: GlobalAccessKind,
    pub confidence: GraphConfidence,
    pub source_handle_ids: Vec<StableHandleId>,
    pub evidence_ids: Vec<EvidenceId>,
}

type Output = functions::FunctionProposals;
type RootGroups =
    BTreeMap<(String, TocSavedVariableScope), (BTreeSet<StableHandleId>, BTreeSet<EvidenceId>)>;

pub(super) fn project(
    project: &ProjectView,
    file_ids: &BTreeMap<&str, String>,
    provenance: &mut ProjectGraphProvenance,
    text_bytes: &mut usize,
    stop: &AtomicBool,
) -> ProjectResult<Output> {
    let mut output = Output {
        entities: Vec::new(),
        relations: Vec::new(),
    };
    let Some(plan) = project.configuration().load_plan() else {
        return Ok(output);
    };
    let toc = plan
        .sources()
        .iter()
        .find(|s| s.path == plan.selected_toc())
        .ok_or_else(invalid)?;
    let mut groups = RootGroups::new();
    let mut counts = BTreeMap::<String, usize>::new();
    for record in plan.records() {
        crate::analyzer::checkpoint(stop)?;
        if record.document != toc.path || record.selection != LoadSelection::Included {
            continue;
        }
        for entry in &record.saved_variables {
            if provenance.state_declarations.len() >= MAX_ROOTS {
                return Err(exhausted());
            }
            charge(text_bytes, entry.name.len() + 256)?;
            let text = plan.document_text(&toc.path).ok_or_else(invalid)?;
            let (start, end) = (
                usize::try_from(record.byte_start).map_err(|_| invalid())?,
                usize::try_from(record.byte_end).map_err(|_| invalid())?,
            );
            if text.get(start..end).is_none() {
                return Err(invalid());
            }
            let span = SourceSpan::byte_range(record.byte_start, record.byte_end)
                .map_err(|_| invalid())?;
            let (handle, evidence) = support(project, toc, span, provenance)?;
            provenance
                .state_declarations
                .push(ProjectGraphStateDeclaration {
                    record_ordinal: record.ordinal,
                    entry_ordinal: entry.ordinal,
                    name: entry.name.clone(),
                    scope: entry.scope,
                    state: entry.state,
                    source_handle_id: handle,
                    evidence_id: evidence,
                });
            if entry.state != TocSavedVariableState::Declared {
                continue;
            }
            *counts.entry(entry.name.clone()).or_default() += 1;
            let group = groups.entry((entry.name.clone(), entry.scope)).or_default();
            group.0.insert(handle);
            group.1.insert(evidence);
            if group.0.len() > 32 || group.1.len() > 32 {
                return Err(exhausted());
            }
        }
    }
    let mut by_name = BTreeMap::<String, Vec<ProjectGraphStateRoot>>::new();
    for ((name, scope), (handles, evidence)) in groups {
        crate::analyzer::checkpoint(stop)?;
        let root_id = id("saved-root", &(&toc.path, &name, scope))?;
        let root = ProjectGraphStateRoot {
            proposal_id: root_id.clone(),
            root_id,
            name: name.clone(),
            scope,
            document: toc.path.clone(),
            ambiguous: counts.get(&name) != Some(&1),
            source_handle_ids: handles.into_iter().collect(),
            evidence_ids: evidence.into_iter().collect(),
        };
        output.entities.push(
            GraphEntityProposal::new(
                root.proposal_id.as_str(),
                "state_root",
                BTreeMap::from([
                    (
                        "document".into(),
                        GraphProposalValue::String(toc.path.clone().into()),
                    ),
                    (
                        "name".into(),
                        GraphProposalValue::String(name.clone().into()),
                    ),
                    (
                        "scope".into(),
                        GraphProposalValue::String(
                            match scope {
                                TocSavedVariableScope::Account => "account",
                                TocSavedVariableScope::Character => "character",
                            }
                            .into(),
                        ),
                    ),
                ]),
                GraphConfidence::Proven,
                root.source_handle_ids.clone(),
                root.evidence_ids.clone(),
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        output.relations.push(ownership(
            &format!("owner:{}", root.root_id),
            file_ids.get(toc.path.as_str()).ok_or_else(invalid)?,
            &root.proposal_id,
            &root.source_handle_ids,
            &root.evidence_ids,
        )?);
        by_name.entry(name).or_default().push(root.clone());
        provenance.state_roots.push(root);
    }
    let Some(report) = project.snapshot().analyzer_binding().function_call_report() else {
        return Ok(output);
    };
    report.validate().map_err(|_| invalid())?;
    let functions = provenance
        .functions
        .iter()
        .map(|f| (f.function_id.clone(), f.clone()))
        .collect::<BTreeMap<_, _>>();
    let source_health = report
        .files()
        .iter()
        .map(|f| (f.path.as_str(), f.parse_error_count == 0))
        .collect::<BTreeMap<_, _>>();
    let unresolved_selection = plan.records().iter().any(|r| {
        r.document == toc.path
            && r.kind == LoadRecordKind::Metadata
            && r.selection == LoadSelection::Unresolved
    });
    let mut paths = BTreeMap::<String, ProjectGraphStatePath>::new();
    for access in report.global_accesses() {
        crate::analyzer::checkpoint(stop)?;
        let Some(roots) = by_name.get(access.root_name()) else {
            continue;
        };
        if provenance.state_sites.len() >= MAX_ACCESSES {
            return Err(exhausted());
        }
        charge(
            text_bytes,
            access.root_name().len()
                + access
                    .keys()
                    .iter()
                    .map(GlobalAccessKey::text_bytes)
                    .sum::<usize>()
                + 512,
        )?;
        let outcome = if unresolved_selection {
            Some(ProjectGraphStateOutcome::UnresolvedTocSelection)
        } else if roots.len() != 1 || roots[0].ambiguous {
            Some(ProjectGraphStateOutcome::AmbiguousDeclaration)
        } else if let Some(blocker) = access.alias_blocker() {
            Some(match blocker {
                AliasAccessBlocker::ReassignedBinding => ProjectGraphStateOutcome::ReassignedAlias,
                AliasAccessBlocker::LocalBindingWrite => {
                    ProjectGraphStateOutcome::LocalAliasRebinding
                }
            })
        } else if !access.path_complete() {
            Some(ProjectGraphStateOutcome::DynamicOrUnsupportedKey)
        } else if access.kind() == GlobalAccessKind::UnsupportedAssignment {
            Some(ProjectGraphStateOutcome::UnsupportedAssignment)
        } else {
            match access.resolution() {
                GlobalAccessResolution::Unresolved => {
                    Some(ProjectGraphStateOutcome::UnresolvedGlobal)
                }
                GlobalAccessResolution::LibraryGlobal => {
                    Some(ProjectGraphStateOutcome::LibraryGlobal)
                }
                GlobalAccessResolution::MainGlobal => {
                    let target = access.declaration().ok_or_else(invalid)?;
                    if source_health.get(target.path.as_str()) != Some(&true) {
                        Some(ProjectGraphStateOutcome::DeclarationSourceIncomplete)
                    } else {
                        None
                    }
                }
            }
        };
        if let Some(outcome) = outcome {
            provenance.state_sites.push(ProjectGraphStateSite {
                access_id: access.fact_id().into(),
                outcome,
            });
            continue;
        }
        let root = &roots[0];
        let function = functions.get(access.function_id()).ok_or_else(invalid)?;
        let declaration = access.declaration().ok_or_else(invalid)?;
        if declaration.role != "main" || declaration.workspace_id != report.main_snapshot_id() {
            return Err(invalid());
        }
        let (handle, evidence) = main_support(
            project,
            access.path(),
            access.content_digest(),
            access.span(),
            provenance,
        )?;
        let (decl_handle, decl_evidence) = main_support(
            project,
            &declaration.path,
            &declaration.content_digest,
            declaration.span,
            provenance,
        )?;
        let mut handles = root
            .source_handle_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        handles.extend([handle, decl_handle, function.source_handle_id]);
        let mut evidences = root.evidence_ids.iter().copied().collect::<BTreeSet<_>>();
        evidences.extend([evidence, decl_evidence, function.evidence_id]);
        // Every alias hop is supported by its actual initializer statement,
        // not just a spelling or a digest-shaped reference in the report.
        for hop in access.aliases() {
            crate::analyzer::checkpoint(stop)?;
            let (alias_handle, alias_evidence) = main_support(
                project,
                access.path(),
                access.content_digest(),
                hop.statement_span,
                provenance,
            )?;
            handles.insert(alias_handle);
            evidences.insert(alias_evidence);
        }
        let handles = handles.into_iter().collect::<Vec<_>>();
        let evidences = evidences.into_iter().collect::<Vec<_>>();
        if handles.len() > 32 || evidences.len() > 32 {
            return Err(exhausted());
        }
        let target_proposal_id = if access.keys().is_empty() {
            root.proposal_id.clone()
        } else {
            let path_id = id("saved-path", &(&root.root_id, access.keys()))?;
            if !paths.contains_key(&path_id) {
                if paths.len() >= MAX_PATHS {
                    return Err(exhausted());
                }
                let keys = wow_core::canonical_json_bytes(&access.keys()).map_err(|_| invalid())?;
                if keys.len() > 4096 {
                    return Err(exhausted());
                }
                let keys = String::from_utf8(keys).map_err(|_| invalid())?;
                output.entities.push(
                    GraphEntityProposal::new(
                        path_id.as_str(),
                        "state_path",
                        BTreeMap::from([
                            (
                                "root".into(),
                                GraphProposalValue::Identifier(root.root_id.clone().into()),
                            ),
                            ("path".into(), GraphProposalValue::String(keys.into())),
                        ]),
                        GraphConfidence::Derived,
                        handles.clone(),
                        evidences.clone(),
                        Vec::new(),
                    )
                    .map_err(|_| invalid())?,
                );
                output.relations.push(ownership(
                    &format!("owner:{path_id}"),
                    &root.proposal_id,
                    &path_id,
                    &handles,
                    &evidences,
                )?);
                paths.insert(
                    path_id.clone(),
                    ProjectGraphStatePath {
                        path_id: path_id.clone(),
                        root_id: root.root_id.clone(),
                        keys: access.keys().to_vec(),
                        proposal_id: path_id.clone(),
                    },
                );
            }
            path_id
        };
        let confidence = if access.is_alias() {
            GraphConfidence::Possible
        } else {
            GraphConfidence::Derived
        };
        let digest = crate::identity::canonical_digest(
            "wow-project/saved-access/2",
            &(
                access.fact_id(),
                &root.root_id,
                access.kind(),
                confidence,
                &target_proposal_id,
                &handles,
                &evidences,
            ),
            ProjectPhase::View,
        )?;
        let binding_id = format!("saved-access:{digest}");
        provenance.state_bindings.push(ProjectGraphStateBinding {
            binding_id,
            access_id: access.fact_id().into(),
            root_id: root.root_id.clone(),
            caller_proposal_id: function.proposal_id.clone(),
            target_proposal_id,
            kind: access.kind(),
            confidence,
            source_handle_ids: handles,
            evidence_ids: evidences,
        });
        provenance.state_sites.push(ProjectGraphStateSite {
            access_id: access.fact_id().into(),
            outcome: ProjectGraphStateOutcome::Projected,
        });
    }
    provenance.state_paths = paths.into_values().collect();
    provenance
        .state_bindings
        .sort_by(|a, b| a.binding_id.cmp(&b.binding_id));
    Ok(output)
}

fn id(prefix: &str, value: &impl Serialize) -> ProjectResult<String> {
    let digest = crate::identity::canonical_digest(
        &format!("wow-project/{prefix}/1"),
        value,
        ProjectPhase::View,
    )?;
    Ok(format!("{prefix}:{digest}"))
}
fn ownership(
    id: &str,
    from: &str,
    to: &str,
    handles: &[StableHandleId],
    evidence: &[EvidenceId],
) -> ProjectResult<GraphRelationProposal> {
    GraphRelationProposal::new(
        id,
        "source_declaration_owns",
        GraphRelationProposalInput {
            source: GraphProposalEndpoint::Proposed(from.into()),
            target: GraphProposalEndpoint::Proposed(to.into()),
            confidence: GraphConfidence::Derived,
            source_handle_ids: handles.to_vec(),
            evidence_ids: evidence.to_vec(),
            coverage_ids: Vec::new(),
        },
    )
    .map_err(|_| invalid())
}
fn main_support(
    project: &ProjectView,
    path: &str,
    digest: &str,
    span: SourceSpan,
    provenance: &mut ProjectGraphProvenance,
) -> ProjectResult<(StableHandleId, EvidenceId)> {
    let files = project
        .snapshot()
        .analyzer_binding()
        .main_workspace()
        .files();
    let index = files
        .binary_search_by(|file| file.path().cmp(path))
        .map_err(|_| invalid())?;
    let file = &files[index];
    let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
        return Err(invalid());
    };
    let (start, end) = (
        usize::try_from(start).map_err(|_| invalid())?,
        usize::try_from(end).map_err(|_| invalid())?,
    );
    if file.content_sha256() != digest || file.text().get(start..end).is_none() {
        return Err(invalid());
    }
    let captured = project.file_by_path(path)?.ok_or_else(invalid)?;
    let source = LoadSource {
        path: path.into(),
        content_digest: captured.content_digest(),
        byte_length: captured.byte_length(),
    };
    support(project, &source, span, provenance)
}
