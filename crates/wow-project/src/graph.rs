//! Direct source/load/XML proposals. No recognizer inference or graph publication.
mod xml;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
pub use xml::{
    ProjectGraphXmlDeclaration, ProjectGraphXmlReference, ProjectGraphXmlReferenceOutcome,
};

use serde::Serialize;
use wow_core::{
    ClaimScope, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext, ProducerId,
    ProvenanceClass, SourceHandle, SourceHandleBuilder, SourceOriginKind, SourceSpan,
    StableHandleId, ToolVersion,
};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEntityKindDefinition,
    GraphEntityProposal, GraphGenerationId, GraphLimits, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalValue, GraphRegistryBundle, GraphRelationKind, GraphRelationKindDefinition,
    GraphRelationProposal, GraphRelationProposalInput, GraphUniverseId,
};

use crate::load::{LoadRecordKind, LoadSelection, LoadSource};
use crate::{
    ProjectError, ProjectErrorCode, ProjectKind, ProjectPhase, ProjectResult, ProjectView,
};

pub const SOURCE_GRAPH_PROFILE: &str = "wow-project/source-load-proposals/2";
pub const SOURCE_GRAPH_PARTITION: &str = "wow-project.source-load";
const MAX_FILES: usize = 4096;
const MAX_LOADS: usize = 8192;
const MAX_NODES: usize = MAX_FILES + xml::MAX_DECLARATIONS;
const MAX_EDGES: usize = MAX_LOADS + xml::MAX_DECLARATIONS + xml::MAX_INHERITANCE_REFERENCES;
const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphFile {
    pub path: String,
    pub content_digest: wow_core::ContentDigest<wow_core::SourceContent>,
    pub byte_length: u64,
    pub proposal_id: String,
    pub source_handle_id: StableHandleId,
    pub evidence_id: EvidenceId,
}

/// Graph handles resolve into these exact records, not fabricated ID-shaped strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectGraphProvenance {
    profile: &'static str,
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    context: GenerationContext,
    files: Vec<ProjectGraphFile>,
    xml_declarations: Vec<ProjectGraphXmlDeclaration>,
    xml_inheritance: Vec<ProjectGraphXmlReference>,
    source_handles: BTreeMap<StableHandleId, SourceHandle>,
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_plan: Option<crate::load::ProjectLoadPlan>,
    skipped_missing_targets: usize,
    skipped_self_loads: usize,
}

impl ProjectGraphProvenance {
    #[must_use]
    pub fn files(&self) -> &[ProjectGraphFile] {
        &self.files
    }
    #[must_use]
    pub fn xml_declarations(&self) -> &[ProjectGraphXmlDeclaration] {
        &self.xml_declarations
    }
    #[must_use]
    pub fn xml_inheritance(&self) -> &[ProjectGraphXmlReference] {
        &self.xml_inheritance
    }
}

/// Input-generation proposals only. wow-service asks wow-graph to materialize them.
#[derive(Debug)]
pub struct ProjectSourceGraphProposals {
    registry: GraphRegistryBundle,
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
    provenance: ProjectGraphProvenance,
    limits: GraphLimits,
}
impl ProjectSourceGraphProposals {
    pub fn into_parts(
        self,
    ) -> (
        GraphRegistryBundle,
        GraphProposalBatch,
        Vec<GraphCoverageRecord>,
        ProjectGraphProvenance,
        GraphLimits,
    ) {
        (
            self.registry,
            self.batch,
            self.coverage,
            self.provenance,
            self.limits,
        )
    }
}

fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::View,
        "source graph inputs or proposal identities disagree",
    )
}
fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::View,
        "source graph projection exceeds its bounded profile",
    )
}
fn charge(used: &mut usize, bytes: usize) -> ProjectResult<()> {
    *used = used.checked_add(bytes).ok_or_else(exhausted)?;
    if *used > MAX_TEXT_BYTES {
        return Err(exhausted());
    }
    Ok(())
}

fn registry() -> ProjectResult<GraphRegistryBundle> {
    let file = GraphEntityKindDefinition::new(
        "source_file",
        vec!["project".into()],
        vec!["path".into()],
        vec![GraphConfidence::Proven],
    )
    .map_err(|_| invalid())?;
    // The load axis also requires DependsOn; it remains explicitly unevaluated.
    let mut relations = [
        ("source_loads", GraphRelationKind::Loads),
        ("source_depends_on", GraphRelationKind::DependsOn),
    ]
    .into_iter()
    .map(|(id, kind)| {
        GraphRelationKindDefinition::new(
            id,
            kind,
            vec!["source_file".into()],
            vec!["source_file".into()],
            vec![GraphConfidence::Proven],
        )
        .map_err(|_| invalid())
    })
    .collect::<ProjectResult<Vec<_>>>()?;
    let declaration = GraphEntityKindDefinition::new(
        "xml_source_declaration",
        vec!["project".into()],
        vec!["document".into(), "occurrence".into()],
        vec![GraphConfidence::Proven],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "source_xml_owns",
            GraphRelationKind::Owns,
            vec!["source_file".into()],
            vec!["xml_source_declaration".into()],
            vec![GraphConfidence::Proven],
        )
        .map_err(|_| invalid())?,
    );
    relations.push(
        GraphRelationKindDefinition::new(
            "source_xml_inherits",
            GraphRelationKind::Inherits,
            vec!["xml_source_declaration".into()],
            vec!["xml_source_declaration".into()],
            vec![GraphConfidence::Derived],
        )
        .map_err(|_| invalid())?,
    );
    GraphRegistryBundle::build(
        "wow-project.source-load",
        "2",
        vec![file, declaration],
        relations,
    )
    .map_err(|_| invalid())
}

fn support(
    project: &ProjectView,
    source: &LoadSource,
    span: SourceSpan,
    provenance: &mut ProjectGraphProvenance,
) -> ProjectResult<(StableHandleId, EvidenceId)> {
    let config = project.configuration();
    let origin = match config.project_kind() {
        ProjectKind::Fixture => SourceOriginKind::Fixture,
        ProjectKind::Repository => SourceOriginKind::GeneratedArtifact,
    };
    let handle = if span == SourceSpan::whole_file()
        && let Some(file) = project.file_by_path(&source.path)?
    {
        file.source_handle_base().clone()
    } else {
        SourceHandleBuilder::new(
            origin,
            config.source_origin_id().as_str(),
            project.project_generation().canonical(),
            &source.path,
            span,
            source.content_digest,
        )
        .reference_generation(config.reference_generation())
        .project_generation(project.project_generation())
        .build()
        .map_err(|_| invalid())?
    };
    let handle_id = handle.handle_id();
    let evidence = EvidenceRecord::new(
        provenance.context.context_id(),
        ProvenanceClass::ProjectSource,
        EvidenceConfidence::Proven,
        ClaimScope::SourceObservation,
        "wow.project".parse::<ProducerId>().map_err(|_| invalid())?,
        ToolVersion::parse(env!("CARGO_PKG_VERSION")).map_err(|_| invalid())?,
        vec![handle_id],
        Vec::new(),
        Vec::new(),
    )
    .map_err(|_| invalid())?;
    let evidence_id = evidence.evidence_id();
    provenance.source_handles.insert(handle_id, handle);
    provenance.evidence.insert(evidence_id, evidence);
    Ok((handle_id, evidence_id))
}

/// Export first-party files, selected direct references and source XML declarations.
/// Library sources, dependency discovery, calls, XML runtime objects and recognizer roles
/// are not inferred. One immutable ProjectView supplies every source identity.
pub fn build_source_graph_proposals(
    project: &ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<ProjectSourceGraphProposals> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    let plan = config.load_plan();
    let sources = if let Some(plan) = plan {
        plan.validate_profile(config.selected_profile())?;
        plan.sources().to_vec()
    } else {
        project
            .file_manifest()
            .iter()
            .map(|f| LoadSource {
                path: f.relative_path().as_str().to_owned(),
                content_digest: f.content_digest(),
                byte_length: f.byte_length(),
            })
            .collect()
    };
    if sources.is_empty() || sources.len() > MAX_FILES {
        return Err(exhausted());
    }
    let mut source_by_path = BTreeMap::new();
    let mut text_bytes = 0;
    for source in &sources {
        crate::analyzer::checkpoint(stop)?;
        charge(&mut text_bytes, source.path.len().saturating_mul(8))?;
        if source_by_path
            .insert(source.path.as_str(), source)
            .is_some()
        {
            return Err(invalid());
        }
        if let Some(file) = project.file_by_path(&source.path)? {
            if file.content_digest() != source.content_digest
                || file.byte_length() != source.byte_length
            {
                return Err(invalid());
            }
        } else {
            let text = plan
                .and_then(|p| p.document_text(&source.path))
                .ok_or_else(invalid)?;
            if text.len() as u64 != source.byte_length
                || crate::identity::source_digest(text.as_bytes()) != source.content_digest
            {
                return Err(invalid());
            }
        }
    }
    for file in project.file_manifest() {
        crate::analyzer::checkpoint(stop)?;
        if !source_by_path.contains_key(file.relative_path().as_str()) {
            return Err(invalid());
        }
    }
    let registry = registry()?;
    let scope = crate::identity::canonical_digest(
        "wow-project/source-graph-universe/1",
        &(
            config.project_id(),
            config.workspace_id(),
            config.source_origin_id(),
            config.logical_root(),
            config.selected_profile(),
        ),
        ProjectPhase::View,
    )?;
    let universe = GraphUniverseId::new(format!("project:{scope}")).map_err(|_| invalid())?;
    // This seed is not a published GraphGeneration. The graph owner derives the
    // materialized generation from the exact registry and accepted partition.
    let seed = crate::identity::canonical_digest(
        "wow-project/source-graph-input/1",
        &(
            SOURCE_GRAPH_PROFILE,
            registry.registry_digest(),
            project.snapshot_id(),
        ),
        ProjectPhase::View,
    )?;
    let generation =
        GraphGenerationId::new(format!("source-graph-input:{seed}")).map_err(|_| invalid())?;
    let limits = GraphLimits::new(MAX_NODES as u32, MAX_EDGES as u32, 32, 64, MAX_EDGES as u32)
        .map_err(|_| invalid())?;
    let mut provenance = ProjectGraphProvenance {
        profile: SOURCE_GRAPH_PROFILE,
        project_snapshot_id: project.snapshot_id().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        context: project.snapshot().generation_context().clone(),
        files: Vec::new(),
        xml_declarations: Vec::new(),
        xml_inheritance: Vec::new(),
        source_handles: BTreeMap::new(),
        evidence: BTreeMap::new(),
        load_plan: plan.cloned(),
        skipped_missing_targets: 0,
        skipped_self_loads: 0,
    };
    let mut entities = Vec::new();
    let mut ids = BTreeMap::new();
    for source in source_by_path.values() {
        crate::analyzer::checkpoint(stop)?;
        let key = crate::identity::canonical_digest(
            "wow-project/source-file-proposal/1",
            &source.path,
            ProjectPhase::View,
        )?;
        let id = format!("file:{key}");
        let (handle, evidence) =
            support(project, source, SourceSpan::whole_file(), &mut provenance)?;
        entities.push(
            GraphEntityProposal::new(
                id.as_str(),
                "source_file",
                BTreeMap::from([(
                    "path".into(),
                    GraphProposalValue::String(source.path.clone().into()),
                )]),
                GraphConfidence::Proven,
                vec![handle],
                vec![evidence],
                Vec::new(),
            )
            .map_err(|_| invalid())?,
        );
        ids.insert(source.path.as_str(), id.clone());
        provenance.files.push(ProjectGraphFile {
            path: source.path.clone(),
            content_digest: source.content_digest,
            byte_length: source.byte_length,
            proposal_id: id,
            source_handle_id: handle,
            evidence_id: evidence,
        });
    }
    let mut relations = Vec::new();
    if let Some(plan) = plan {
        for record in plan.records() {
            crate::analyzer::checkpoint(stop)?;
            if !matches!(
                record.kind,
                LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
            ) || record.selection != LoadSelection::Included
            {
                continue;
            }
            let Some(target) = record.target.as_deref() else {
                continue;
            };
            if !ids.contains_key(target) {
                provenance.skipped_missing_targets += 1;
                continue;
            }
            // The stored graph profile rejects self-edges. Retain the loader's
            // cycle/occurrence receipt, never erase the limitation from coverage.
            if target == record.document {
                provenance.skipped_self_loads += 1;
                continue;
            }
            if relations.len() >= MAX_LOADS {
                return Err(exhausted());
            }
            let source = source_by_path
                .get(record.document.as_str())
                .ok_or_else(invalid)?;
            let text = plan.document_text(&record.document).ok_or_else(invalid)?;
            let start = usize::try_from(record.byte_start).map_err(|_| invalid())?;
            let end = usize::try_from(record.byte_end).map_err(|_| invalid())?;
            let raw = text.get(start..end).ok_or_else(invalid)?;
            if crate::identity::source_digest(raw.as_bytes()) != record.raw_digest {
                return Err(invalid());
            }
            charge(&mut text_bytes, record.document.len().saturating_mul(4))?;
            let span = SourceSpan::byte_range(record.byte_start, record.byte_end)
                .map_err(|_| invalid())?;
            let (handle, evidence) = support(project, source, span, &mut provenance)?;
            relations.push(
                GraphRelationProposal::new(
                    format!("load:{}", record.ordinal),
                    "source_loads",
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Proposed(
                            ids[record.document.as_str()].clone().into(),
                        ),
                        target: GraphProposalEndpoint::Proposed(ids[target].clone().into()),
                        confidence: GraphConfidence::Proven,
                        source_handle_ids: vec![handle],
                        evidence_ids: vec![evidence],
                        coverage_ids: Vec::new(),
                    },
                )
                .map_err(|_| invalid())?,
            );
        }
    }
    let xml = xml::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(xml.entities);
    relations.extend(xml.relations);
    if entities.len() > MAX_NODES || relations.len() > MAX_EDGES {
        return Err(exhausted());
    }
    let xml_state = if plan.is_some_and(|p| !p.xml_documents().is_empty()) {
        GraphCoverageState::Partial
    } else {
        GraphCoverageState::NotEvaluated
    };
    let coverage = vec![
        GraphCoverageRecord::new(
            GraphRelationKind::Owns,
            xml_state,
            false,
            vec!["source_graph.xml_document_ownership_only".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::Inherits,
            xml_state,
            false,
            vec!["source_graph.admitted_local_xml_inheritance_only".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::Loads,
            if plan.is_some() {
                GraphCoverageState::Partial
            } else {
                GraphCoverageState::NotEvaluated
            },
            false,
            vec!["source_graph.direct_file_references_only".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::DependsOn,
            GraphCoverageState::NotEvaluated,
            false,
            vec!["source_graph.package_dependencies_not_evaluated".into()],
            limits,
        )
        .map_err(|_| invalid())?,
    ];
    let batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe,
        generation,
        provenance.context.context_id(),
        SOURCE_GRAPH_PARTITION,
        entities,
        relations,
    )
    .map_err(|_| invalid())?;
    crate::analyzer::checkpoint(stop)?;
    Ok(ProjectSourceGraphProposals {
        registry,
        batch,
        coverage,
        provenance,
        limits,
    })
}
