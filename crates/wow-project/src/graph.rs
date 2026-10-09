//! Direct source/load/XML proposals. No recognizer inference or graph publication.
mod functions;
mod packages;
pub mod persistence;
mod retained_evidence;
mod source_read;
pub use retained_evidence::RetainedProjectGraphEvidence;
pub use source_read::{
    ProjectSourceExcerpt, ProjectSourceExcerptStatus, ProjectSourceFileRead,
    ProjectSourceFileStatus, ProjectSourceReadLimits, ProjectSourceReadReport,
    ProjectSourceReadTruncation, RetainedProjectSourceManifest,
};
mod state;
pub use state::{
    ProjectGraphStateBinding, ProjectGraphStateDeclaration, ProjectGraphStateOutcome,
    ProjectGraphStatePath, ProjectGraphStateRoot, ProjectGraphStateSite,
};
pub use wow_emmy::global_access::GlobalAccessKey;
mod scripts;
pub use scripts::{
    ProjectGraphInlineHandler, ProjectGraphScriptBinding, ProjectGraphScriptQuery,
    ProjectGraphScriptQueryOutcome, ProjectGraphScriptSite, ProjectGraphScriptSource,
};
mod mixins;
pub use functions::{ProjectGraphCallSite, ProjectGraphFunction};
pub use packages::{
    ProjectGraphPackage, ProjectGraphPackageDependency, ProjectGraphPackageDependencyOutcome,
    ProjectGraphPackageFile, ProjectGraphPackageLoad, ProjectGraphPackageLoadOutcome,
};
mod xml;
pub use mixins::{
    ProjectGraphLuaDeclaration, ProjectGraphMixinOutcome, ProjectGraphMixinReference,
};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
pub use xml::{
    ProjectGraphXmlDeclaration, ProjectGraphXmlReference, ProjectGraphXmlReferenceOutcome,
};

use serde::{Deserialize, Serialize};
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

pub const SOURCE_GRAPH_PROFILE: &str = "wow-project/source-load-proposals/14";
pub const SOURCE_GRAPH_PARTITION: &str = "wow-project.source-load";
const MAX_FILES: usize = 4096;
const MAX_LOADS: usize = 8192;
const MAX_RECOGNIZER_NODES: usize = functions::MAX_CALLS * 2;

const MAX_SIGNAL_RELATION_EDGES: usize = 131072;
const MAX_RECOGNIZER_EDGES: usize = functions::MAX_CALLS * 19 + MAX_SIGNAL_RELATION_EDGES;
const MAX_MIXIN_ASSIGNMENT_EDGES: usize = 65_536;
const MAX_NODES: usize = MAX_FILES
    + packages::MAX_PACKAGE_NODES
    + xml::MAX_DECLARATIONS
    + mixins::MAX_DECLARATIONS
    + functions::MAX_FUNCTIONS
    + scripts::MAX_HANDLERS
    + state::MAX_ROOTS
    + state::MAX_PATHS
    + MAX_RECOGNIZER_NODES;

const MAX_EDGES: usize = MAX_LOADS
    + packages::MAX_PACKAGE_RELATIONS
    + xml::MAX_DECLARATIONS
    + xml::MAX_INHERITANCE_REFERENCES
    + mixins::MAX_DECLARATIONS
    + mixins::MAX_REFERENCES
    + functions::MAX_FUNCTIONS
    + MAX_RECOGNIZER_EDGES
    + MAX_MIXIN_ASSIGNMENT_EDGES
    + scripts::MAX_HANDLERS
    + scripts::MAX_BINDINGS
    + state::MAX_ROOTS
    + state::MAX_PATHS
    + state::MAX_ACCESSES;
const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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
    packages: Vec<ProjectGraphPackage>,
    package_files: Vec<ProjectGraphPackageFile>,
    package_dependencies: Vec<ProjectGraphPackageDependency>,
    package_loads: Vec<ProjectGraphPackageLoad>,
    xml_declarations: Vec<ProjectGraphXmlDeclaration>,
    xml_inheritance: Vec<ProjectGraphXmlReference>,
    lua_declarations: Vec<ProjectGraphLuaDeclaration>,
    xml_mixins: Vec<ProjectGraphMixinReference>,
    functions: Vec<ProjectGraphFunction>,
    call_sites: Vec<ProjectGraphCallSite>,
    script_sources: Vec<ProjectGraphScriptSource>,
    inline_handlers: Vec<ProjectGraphInlineHandler>,
    script_sites: Vec<ProjectGraphScriptSite>,
    script_bindings: Vec<ProjectGraphScriptBinding>,
    state_declarations: Vec<ProjectGraphStateDeclaration>,
    state_roots: Vec<ProjectGraphStateRoot>,
    state_paths: Vec<ProjectGraphStatePath>,
    state_sites: Vec<ProjectGraphStateSite>,
    state_bindings: Vec<ProjectGraphStateBinding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_lua_analysis: Option<crate::xml_lua::ProjectXmlLuaAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function_call_report: Option<wow_emmy::function_calls::FunctionCallReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    xml_binding_report: Option<crate::xml_bindings::ProjectXmlLuaBindings>,
    source_handles: BTreeMap<StableHandleId, SourceHandle>,
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    load_plan: Option<crate::load::ProjectLoadPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_load_plan: Option<crate::load::ProjectPackageLoadPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    package_main_plan: Option<crate::load::ProjectPackageMainPlan>,
    skipped_missing_targets: usize,
    skipped_self_loads: usize,
}

impl ProjectGraphProvenance {
    #[must_use]
    pub fn packages(&self) -> &[ProjectGraphPackage] {
        &self.packages
    }
    #[must_use]
    pub fn package_files(&self) -> &[ProjectGraphPackageFile] {
        &self.package_files
    }
    #[must_use]
    pub fn package_dependencies(&self) -> &[ProjectGraphPackageDependency] {
        &self.package_dependencies
    }
    #[must_use]
    pub fn package_loads(&self) -> &[ProjectGraphPackageLoad] {
        &self.package_loads
    }
    pub fn state_declarations(&self) -> &[ProjectGraphStateDeclaration] {
        &self.state_declarations
    }
    pub fn state_roots(&self) -> &[ProjectGraphStateRoot] {
        &self.state_roots
    }
    pub fn state_paths(&self) -> &[ProjectGraphStatePath] {
        &self.state_paths
    }
    pub fn state_sites(&self) -> &[ProjectGraphStateSite] {
        &self.state_sites
    }
    pub fn state_bindings(&self) -> &[ProjectGraphStateBinding] {
        &self.state_bindings
    }
    pub fn script_sources(&self) -> &[ProjectGraphScriptSource] {
        &self.script_sources
    }
    pub fn inline_handlers(&self) -> &[ProjectGraphInlineHandler] {
        &self.inline_handlers
    }
    pub fn script_sites(&self) -> &[ProjectGraphScriptSite] {
        &self.script_sites
    }
    pub fn script_bindings(&self) -> &[ProjectGraphScriptBinding] {
        &self.script_bindings
    }
    pub fn functions(&self) -> &[ProjectGraphFunction] {
        &self.functions
    }
    pub fn call_sites(&self) -> &[ProjectGraphCallSite] {
        &self.call_sites
    }
    pub fn function_call_report(&self) -> Option<&wow_emmy::function_calls::FunctionCallReport> {
        self.function_call_report.as_ref()
    }
    pub fn source_handles(&self) -> &BTreeMap<StableHandleId, SourceHandle> {
        &self.source_handles
    }
    pub fn evidence(&self) -> &BTreeMap<EvidenceId, EvidenceRecord> {
        &self.evidence
    }
    pub fn context(&self) -> &GenerationContext {
        &self.context
    }

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
    #[must_use]
    pub fn lua_declarations(&self) -> &[ProjectGraphLuaDeclaration] {
        &self.lua_declarations
    }
    #[must_use]
    pub fn xml_mixins(&self) -> &[ProjectGraphMixinReference] {
        &self.xml_mixins
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
    let package = GraphEntityKindDefinition::new(
        "source_package",
        vec!["project".into()],
        vec!["package".into()],
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
    relations.push(
        GraphRelationKindDefinition::new(
            "source_package_owns",
            GraphRelationKind::Owns,
            vec!["source_package".into()],
            vec!["source_file".into()],
            vec![GraphConfidence::Proven],
        )
        .map_err(|_| invalid())?,
    );
    relations.push(
        GraphRelationKindDefinition::new(
            "source_package_loads",
            GraphRelationKind::Loads,
            vec!["source_package".into()],
            vec!["source_file".into()],
            vec![GraphConfidence::Proven, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    relations.push(
        GraphRelationKindDefinition::new(
            "source_package_depends_on",
            GraphRelationKind::DependsOn,
            vec!["source_package".into()],
            vec!["source_package".into()],
            vec![GraphConfidence::Proven, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    let declaration = GraphEntityKindDefinition::new(
        "xml_source_declaration",
        vec!["project".into()],
        vec!["document".into(), "occurrence".into()],
        vec![GraphConfidence::Proven],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "source_declaration_owns",
            GraphRelationKind::Owns,
            vec!["source_file".into(), "state_root".into()],
            vec![
                "xml_source_declaration".into(),
                "lua_source_declaration".into(),
                "lua_source_function".into(),
                "xml_source_handler".into(),
                "state_root".into(),
                "state_path".into(),
            ],
            vec![GraphConfidence::Proven, GraphConfidence::Derived],
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
    let lua = GraphEntityKindDefinition::new(
        "lua_source_declaration",
        vec!["project".into()],
        vec!["document".into(), "span_start".into(), "span_end".into()],
        vec![GraphConfidence::Derived],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "source_mixes_in",
            GraphRelationKind::MixesIn,
            vec![
                "lua_source_declaration".into(),
                "mixin_instance".into(),
                "xml_source_declaration".into(),
            ],
            vec!["lua_source_declaration".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    let function = GraphEntityKindDefinition::new(
        "lua_source_function",
        vec!["project".into()],
        vec!["document".into(), "function".into()],
        vec![GraphConfidence::Derived],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "lua_direct_calls",
            GraphRelationKind::Calls,
            vec!["lua_source_function".into()],
            vec!["lua_source_function".into()],
            vec![GraphConfidence::Derived],
        )
        .map_err(|_| invalid())?,
    );
    let frame = GraphEntityKindDefinition::new(
        "frame",
        vec!["project".into()],
        vec!["call".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "lua_factory_creates",
            GraphRelationKind::FactoryCreates,
            vec!["lua_source_function".into()],
            vec!["frame".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    let mixin_instance = GraphEntityKindDefinition::new(
        "mixin_instance",
        vec!["project".into()],
        vec!["call".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "lua_instantiates",
            GraphRelationKind::Instantiates,
            vec!["lua_source_function".into()],
            vec!["mixin_instance".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    let handler = GraphEntityKindDefinition::new(
        "xml_source_handler",
        vec!["project".into()],
        vec!["document".into(), "occurrence".into()],
        vec![GraphConfidence::Derived],
    )
    .map_err(|_| invalid())?;
    relations.push(
        GraphRelationKindDefinition::new(
            "source_xml_sets_script",
            GraphRelationKind::SetsScript,
            vec!["xml_source_declaration".into()],
            vec!["lua_source_function".into(), "xml_source_handler".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )
        .map_err(|_| invalid())?,
    );
    let state_root = GraphEntityKindDefinition::new(
        "state_root",
        vec!["project".into()],
        vec!["document".into(), "name".into(), "scope".into()],
        vec![GraphConfidence::Proven],
    )
    .map_err(|_| invalid())?;
    let state_path = GraphEntityKindDefinition::new(
        "state_path",
        vec!["project".into()],
        vec!["root".into(), "path".into()],
        vec![GraphConfidence::Derived],
    )
    .map_err(|_| invalid())?;
    for (id, relation) in [
        ("source_reads_state", GraphRelationKind::ReadsState),
        ("source_writes_state", GraphRelationKind::WritesState),
    ] {
        relations.push(
            GraphRelationKindDefinition::new(
                id,
                relation,
                vec!["lua_source_function".into()],
                vec!["state_root".into(), "state_path".into()],
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )
            .map_err(|_| invalid())?,
        );
    }
    // Signal and event family. A Lua function registers, bridges or handles a
    // native frame event, a custom registry signal or a CVar callback. The
    // event, signal and CVar identities are exact literal evidence, never
    // inferred from a plausible name.
    let native_event = GraphEntityKindDefinition::new(
        "native_event",
        vec!["project".into()],
        vec!["event".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let custom_signal = GraphEntityKindDefinition::new(
        "custom_signal",
        vec!["project".into()],
        vec!["signal".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let cvar_key = GraphEntityKindDefinition::new(
        "cvar_key",
        vec!["project".into()],
        vec!["cvar".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let library = GraphEntityKindDefinition::new(
        "library",
        vec!["project".into()],
        vec!["library".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    // Each tuple is (relation id, relation kind, allowed target entity kinds).
    for (id, relation, targets) in [
        (
            "lua_registers_native_event",
            GraphRelationKind::RegistersNativeEvent,
            vec!["native_event".into()],
        ),
        (
            "lua_handles_native_event",
            GraphRelationKind::HandlesNativeEvent,
            vec!["native_event".into()],
        ),
        (
            "lua_bridges_native_event",
            GraphRelationKind::BridgesNativeEvent,
            vec!["native_event".into()],
        ),
        (
            "lua_emits_custom_signal",
            GraphRelationKind::EmitsCustomSignal,
            vec!["custom_signal".into()],
        ),
        (
            "lua_handles_custom_signal",
            GraphRelationKind::HandlesCustomSignal,
            vec!["custom_signal".into()],
        ),
        (
            "lua_registers_cvar_callback",
            GraphRelationKind::RegistersCvarCallback,
            vec!["cvar_key".into()],
        ),
        // Script hooks and the secure posthook family.
        (
            "lua_sets_script",
            GraphRelationKind::SetsScript,
            vec![
                "lua_source_declaration".into(),
                "lua_source_function".into(),
            ],
        ),
        (
            "lua_hooks_script",
            GraphRelationKind::HooksScript,
            vec![
                "lua_source_declaration".into(),
                "lua_source_function".into(),
            ],
        ),
        (
            "lua_secure_hooks_function",
            GraphRelationKind::SecureHooksFunction,
            vec![
                "lua_source_declaration".into(),
                "lua_source_function".into(),
            ],
        ),
        // Library requirement and structural embedding.
        (
            "lua_declares_library",
            GraphRelationKind::UsesApi,
            vec!["library".into()],
        ),
        (
            "lua_requires_library",
            GraphRelationKind::UsesApi,
            vec!["library".into()],
        ),
        (
            "lua_embeds_library",
            GraphRelationKind::UsesApi,
            vec!["library".into()],
        ),
    ] {
        relations.push(
            GraphRelationKindDefinition::new(
                id,
                relation,
                vec!["lua_source_function".into()],
                targets,
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )
            .map_err(|_| invalid())?,
        );
    }
    GraphRegistryBundle::build(
        "wow-project.source-load",
        "12",
        vec![
            file,
            package,
            declaration,
            lua,
            function,
            frame,
            mixin_instance,
            handler,
            state_root,
            state_path,
            native_event,
            custom_signal,
            cvar_key,
            library,
        ],
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
/// Library sources, dependency discovery, Calls edges, XML runtime objects and recognizer roles
/// are not inferred. Callable source occurrences and call evidence are source-owned. One immutable ProjectView supplies every source identity.
pub fn build_source_graph_proposals(
    project: &ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<ProjectSourceGraphProposals> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    let plan = config.load_plan();
    let package_plan = config.package_load_plan();
    let package_main_plan = config.package_main_plan();
    if package_plan.is_some() != package_main_plan.is_some()
        || (plan.is_some() && package_plan.is_some())
    {
        return Err(invalid());
    }
    let mut projected_sources = BTreeMap::<String, LoadSource>::new();
    let mut retained_documents = BTreeMap::<String, &str>::new();
    if let Some(plan) = plan {
        plan.validate_profile(config.selected_profile())?;
        for source in plan.sources() {
            projected_sources.insert(source.path.clone(), source.clone());
            if let Some(text) = plan.document_text(&source.path) {
                retained_documents.insert(source.path.clone(), text);
            }
        }
    } else if let (Some(package_plan), Some(main_plan)) = (package_plan, package_main_plan) {
        package_plan.validate_profile(config.selected_profile())?;
        main_plan.validate_load_plan(package_plan)?;
        for receipt in main_plan.files() {
            let selected = package_plan
                .package_plan(&receipt.package)
                .ok_or_else(invalid)?;
            let source = selected
                .sources()
                .iter()
                .find(|source| source.path == receipt.source_path)
                .ok_or_else(invalid)?;
            if source.content_digest != receipt.content_digest
                || source.byte_length != receipt.byte_length
            {
                return Err(invalid());
            }
            projected_sources.insert(
                receipt.project_path.clone(),
                LoadSource {
                    path: receipt.project_path.clone(),
                    content_digest: receipt.content_digest,
                    byte_length: receipt.byte_length,
                },
            );
        }
        for package in package_plan.packages() {
            let selected = package_plan
                .package_plan(&package.package)
                .ok_or_else(invalid)?;
            for source in selected.sources() {
                let Some(text) = selected.document_text(&source.path) else {
                    continue;
                };
                let path = format!(
                    "{}/{}/{}",
                    crate::load::PACKAGE_MAIN_NAMESPACE_ROOT,
                    package.package,
                    source.path
                );
                let projected = LoadSource {
                    path: path.clone(),
                    content_digest: source.content_digest,
                    byte_length: source.byte_length,
                };
                if projected_sources
                    .insert(path.clone(), projected.clone())
                    .is_some_and(|prior| prior != projected)
                {
                    return Err(invalid());
                }
                retained_documents.insert(path, text);
            }
        }
    } else {
        for file in project.file_manifest() {
            let source = LoadSource {
                path: file.relative_path().as_str().to_owned(),
                content_digest: file.content_digest(),
                byte_length: file.byte_length(),
            };
            projected_sources.insert(source.path.clone(), source);
        }
    }
    let sources = projected_sources.into_values().collect::<Vec<_>>();
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
            let text = retained_documents
                .get(&source.path)
                .copied()
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
        packages: Vec::new(),
        package_files: Vec::new(),
        package_dependencies: Vec::new(),
        package_loads: Vec::new(),
        xml_declarations: Vec::new(),
        xml_inheritance: Vec::new(),
        lua_declarations: Vec::new(),
        xml_mixins: Vec::new(),
        functions: Vec::new(),
        call_sites: Vec::new(),
        script_sources: Vec::new(),
        inline_handlers: Vec::new(),
        script_sites: Vec::new(),
        script_bindings: Vec::new(),
        state_declarations: Vec::new(),
        state_roots: Vec::new(),
        state_paths: Vec::new(),
        state_sites: Vec::new(),
        state_bindings: Vec::new(),
        xml_lua_analysis: None,
        function_call_report: None,
        xml_binding_report: None,
        source_handles: BTreeMap::new(),
        evidence: BTreeMap::new(),
        load_plan: plan.cloned(),
        package_load_plan: package_plan.cloned(),
        package_main_plan: package_main_plan.cloned(),
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
    let packages = packages::project(
        project,
        &ids,
        &source_by_path,
        &mut provenance,
        &mut text_bytes,
        stop,
    )?;
    entities.extend(packages.entities);
    relations.extend(packages.relations);
    let xml = xml::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(xml.entities);
    relations.extend(xml.relations);
    let mixins = mixins::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(mixins.entities);
    relations.extend(mixins.relations);
    let functions = functions::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(functions.entities);
    relations.extend(functions.relations);
    let scripts = scripts::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(scripts.entities);
    relations.extend(scripts.relations);
    let state = state::project(project, &ids, &mut provenance, &mut text_bytes, stop)?;
    entities.extend(state.entities);
    relations.extend(state.relations);
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
            GraphRelationKind::ReadsState,
            GraphCoverageState::NotEvaluated,
            false,
            vec!["source_graph.state_access_owned_by_recognizers".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::WritesState,
            GraphCoverageState::NotEvaluated,
            false,
            vec!["source_graph.state_access_owned_by_recognizers".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::SetsScript,
            GraphCoverageState::NotEvaluated,
            false,
            vec!["source_graph.script_assignment_owned_by_recognizers".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::Calls,
            GraphCoverageState::NotEvaluated,
            false,
            vec!["source_graph.calls_owned_by_recognizers".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::MixesIn,
            if provenance.xml_mixins.is_empty() {
                GraphCoverageState::NotEvaluated
            } else {
                GraphCoverageState::Partial
            },
            false,
            vec!["source_graph.explicit_xml_main_mixin_references_only".into()],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::Owns,
            if provenance.functions.is_empty()
                && provenance.state_roots.is_empty()
                && provenance.packages.is_empty()
            {
                xml_state
            } else {
                GraphCoverageState::Partial
            },
            false,
            vec![
                "source_graph.package_file_document_declaration_and_state_namespace_ownership_only"
                    .into(),
            ],
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
            if plan.is_some() || package_plan.is_some() {
                GraphCoverageState::Partial
            } else {
                GraphCoverageState::NotEvaluated
            },
            false,
            vec![
                "source_graph.direct_file_and_static_package_load_references_only".into(),
                "source_graph.package_loads_are_not_runtime_load_success".into(),
            ],
            limits,
        )
        .map_err(|_| invalid())?,
        GraphCoverageRecord::new(
            GraphRelationKind::DependsOn,
            if package_plan.is_some() {
                GraphCoverageState::Partial
            } else {
                GraphCoverageState::NotEvaluated
            },
            false,
            vec![
                "source_graph.explicit_selected_toc_package_dependencies_only".into(),
                "source_graph.optional_dependencies_are_possible_not_runtime_presence".into(),
                "source_graph.no_dependency_negative_authority".into(),
            ],
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
