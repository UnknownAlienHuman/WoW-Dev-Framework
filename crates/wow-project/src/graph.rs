//! Direct source/load/XML proposals. No recognizer inference or graph publication.
mod derivations;
mod functions;
mod load_inputs;
mod packages;
mod platform_producers;
mod producer_budget;
mod producer_derivations;
mod producer_evidence;
pub use platform_producers::{
    PLATFORM_DIRECT_GRAPH_PROFILE, PlatformGraphProducerProposals, PlatformGraphProposalPlan,
    PlatformGraphProvenance, build_platform_graph_proposal_plan,
};
mod projection;
pub use projection::PlatformGraphProducer;
use projection::{EntityDraft, RelationDraft};
mod raw_inventory;
pub use raw_inventory::{
    PLATFORM_RAW_INVENTORY_PARTITION, PLATFORM_RAW_MEMBER_KIND, ProjectRawInventoryManifest,
    ProjectRawInventoryMember, ProjectRawMemberReadBinding, bind_platform_raw_member,
};
pub mod persistence;
mod retained_evidence;
mod source_read;
mod toc_facts;
mod toc_registry;
mod xml_facts;
mod xml_registry;
pub use retained_evidence::RetainedProjectGraphEvidence;
pub use source_read::{
    ProjectSourceExcerpt, ProjectSourceExcerptStatus, ProjectSourceFileRead,
    ProjectSourceFileStatus, ProjectSourceReadLimits, ProjectSourceReadReport,
    ProjectSourceReadTruncation, RetainedProjectSourceManifest,
};
pub use toc_facts::{PROJECT_TOC_FACT_PROFILE, ProjectTocFact, ProjectTocFactKind};
pub use xml_facts::{
    PROJECT_XML_FACT_PROFILE, ProjectXmlContainment, ProjectXmlFact,
    ProjectXmlFactDeclarationState, ProjectXmlFactKind, ProjectXmlFactScope,
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
    ProvenanceClass, SourceHandle, SourceHandleBuilder, SourceSpan, StableHandleId, ToolVersion,
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

pub const SOURCE_GRAPH_PROFILE: &str = "wow-project/source-load-proposals/19";
pub const PACKAGE_SOURCE_GRAPH_PROFILE: &str = "wow-project/source-load-proposals/20";
pub const PACKAGE_RAW_SOURCE_GRAPH_PROFILE: &str = "wow-project/source-load-proposals/21";
pub const SOURCE_GRAPH_PARTITION: &str = "wow-project.source-load";

/// Select the graph identity from the admitted configuration, never from source
/// class alone. Missing selection retains the published /19 recipe.
#[must_use]
pub fn source_graph_profile(configuration: &crate::ProjectConfiguration) -> &'static str {
    match configuration.platform_graph_profile() {
        None => SOURCE_GRAPH_PROFILE,
        Some(crate::PlatformGraphProfile::PackageProjectionV1) => PACKAGE_SOURCE_GRAPH_PROFILE,
        Some(crate::PlatformGraphProfile::PackageProjectionWithRawInventoryV1) => {
            PACKAGE_RAW_SOURCE_GRAPH_PROFILE
        }
    }
}
const MAX_FILES: usize = 4096;
const MAX_LOADS: usize = 8192;
const MAX_RECOGNIZER_NODES: usize = functions::MAX_CALLS * 2;

const MAX_SIGNAL_RELATION_EDGES: usize = 131072;
const MAX_RECOGNIZER_EDGES: usize = functions::MAX_CALLS * 19 + MAX_SIGNAL_RELATION_EDGES;
const MAX_MIXIN_ASSIGNMENT_EDGES: usize = 65_536;
const MAX_STATE_CORE_EDGES: usize = state::MAX_ROOTS + state::MAX_ACCESSES;
const MAX_XML_RECOGNIZER_NODES: usize = xml::MAX_DECLARATIONS + scripts::MAX_HANDLERS;
const MAX_XML_RECOGNIZER_EDGES: usize = xml::MAX_DECLARATIONS * 3
    + xml::MAX_INHERITANCE_REFERENCES * 2
    + scripts::MAX_HANDLERS * 2
    + scripts::MAX_BINDINGS * 3;
const MAX_NODES: usize = MAX_FILES
    + packages::MAX_PACKAGE_NODES
    + xml::MAX_DECLARATIONS
    + mixins::MAX_DECLARATIONS
    + functions::MAX_FUNCTIONS
    + scripts::MAX_HANDLERS
    + state::MAX_ROOTS
    + state::MAX_PATHS
    + MAX_RECOGNIZER_NODES
    + MAX_XML_RECOGNIZER_NODES
    + (packages::MAX_PACKAGE_NODES + 1) * 4;

const MAX_EDGES: usize = MAX_LOADS
    + packages::MAX_PACKAGE_RELATIONS
    + xml::MAX_DECLARATIONS
    + xml::MAX_INHERITANCE_REFERENCES
    + mixins::MAX_DECLARATIONS
    + mixins::MAX_REFERENCES
    + functions::MAX_FUNCTIONS
    + MAX_RECOGNIZER_EDGES
    + MAX_MIXIN_ASSIGNMENT_EDGES
    + MAX_XML_RECOGNIZER_EDGES
    + MAX_STATE_CORE_EDGES
    + scripts::MAX_HANDLERS
    + scripts::MAX_BINDINGS
    + state::MAX_ROOTS
    + state::MAX_PATHS
    + state::MAX_ACCESSES;
const MAX_TEXT_BYTES: usize = 4 * 1024 * 1024;
const MAX_QUERY_EDGES: usize = 100_000;

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
    toc_facts: Vec<ProjectTocFact>,
    xml_facts: Vec<ProjectXmlFact>,
    xml_containment: Vec<ProjectXmlContainment>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    package_xml_binding_report: Option<crate::xml_bindings::ProjectPackageXmlLuaBindings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    raw_inventory: Option<ProjectRawInventoryManifest>,
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
    pub fn raw_inventory(&self) -> Option<&ProjectRawInventoryManifest> {
        self.raw_inventory.as_ref()
    }
    pub fn toc_facts(&self) -> &[ProjectTocFact] {
        &self.toc_facts
    }
    pub fn xml_facts(&self) -> &[ProjectXmlFact] {
        &self.xml_facts
    }
    pub fn xml_containment(&self) -> &[ProjectXmlContainment] {
        &self.xml_containment
    }
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
    inventory_batch: Option<GraphProposalBatch>,
}
impl ProjectSourceGraphProposals {
    /// The separate native inventory producer exists only in the selected raw recipe.
    #[must_use]
    pub fn inventory_batch(&self) -> Option<&GraphProposalBatch> {
        self.inventory_batch.as_ref()
    }
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

fn registry(project_kind: ProjectKind, raw_inventory: bool) -> ProjectResult<GraphRegistryBundle> {
    let universe_class = match project_kind {
        ProjectKind::Fixture | ProjectKind::Repository => "project",
        ProjectKind::BlizzardUiPlatformSource => "blizzard_ui_source",
    };
    let file = GraphEntityKindDefinition::new(
        "source_file",
        vec![universe_class.into()],
        vec!["path".into()],
        vec![GraphConfidence::Proven],
    )
    .map_err(|_| invalid())?;
    let package = GraphEntityKindDefinition::new(
        "source_package",
        vec![universe_class.into()],
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
        vec![universe_class.into()],
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
        vec![universe_class.into()],
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
        vec![universe_class.into()],
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
        vec![universe_class.into()],
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
        vec![universe_class.into()],
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
        vec![universe_class.into()],
        vec![
            "document".into(),
            "occurrence".into(),
            "semantic_context_id".into(),
        ],
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
        vec![universe_class.into()],
        vec!["document".into(), "name".into(), "scope".into()],
        vec![
            GraphConfidence::Proven,
            GraphConfidence::Derived,
            GraphConfidence::Possible,
        ],
    )
    .map_err(|_| invalid())?;
    let state_path = GraphEntityKindDefinition::new(
        "state_path",
        vec![universe_class.into()],
        vec!["root".into(), "path".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
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
        vec![universe_class.into()],
        vec!["event".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let custom_signal = GraphEntityKindDefinition::new(
        "custom_signal",
        vec![universe_class.into()],
        vec!["signal".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let cvar_key = GraphEntityKindDefinition::new(
        "cvar_key",
        vec![universe_class.into()],
        vec!["cvar".into()],
        vec![GraphConfidence::Derived, GraphConfidence::Possible],
    )
    .map_err(|_| invalid())?;
    let library = GraphEntityKindDefinition::new(
        "library",
        vec![universe_class.into()],
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
    let mut entities = vec![
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
    ];
    toc_registry::extend(universe_class, &mut entities, &mut relations)?;
    xml_registry::extend(universe_class, &mut entities, &mut relations)?;
    if raw_inventory {
        raw_inventory::extend_registry(&mut entities)?;
    }
    GraphRegistryBundle::build(
        "wow-project.source-load",
        if raw_inventory { "16" } else { "15" },
        entities,
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
    let (origin, revision) =
        crate::registry::source_handle_identity(config, project.project_generation())?;
    let handle = if span == SourceSpan::whole_file()
        && let Some(file) = project.file_by_path(&source.path)?
    {
        file.source_handle_base().clone()
    } else {
        SourceHandleBuilder::new(
            origin,
            config.source_origin_id().as_str(),
            revision.as_ref(),
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

/// Export selected Main files, native load references and source XML declarations
/// from one immutable ordinary or platform ProjectView. Callable occurrences and
/// direct call evidence remain source-owned. Library sources, dependency discovery,
/// XML runtime objects and recognizer roles are not inferred.
pub fn build_source_graph_proposals(
    project: &ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<ProjectSourceGraphProposals> {
    let CollectedSourceGraph {
        registry,
        universe,
        generation,
        entities,
        relations,
        coverage,
        provenance,
        limits,
        inventory_batch,
        text_bytes: _,
    } = collect_source_graph_proposals(project, stop)?;
    let mut native_entities = Vec::new();
    for entity in entities {
        crate::analyzer::checkpoint(stop)?;
        native_entities.push(entity.proposal);
    }
    let mut native_relations = Vec::new();
    for relation in relations {
        crate::analyzer::checkpoint(stop)?;
        native_relations.push(relation.into_proposal().map_err(|_| invalid())?);
    }
    let derivations = derivations::records(
        &provenance,
        &universe,
        &generation,
        &native_entities,
        &native_relations,
        stop,
    )?;
    let batch = GraphProposalBatch::build(
        registry.bundle_id(),
        registry.registry_digest(),
        universe,
        generation,
        provenance.context.context_id(),
        SOURCE_GRAPH_PARTITION,
        native_entities,
        native_relations,
    )
    .and_then(|batch| batch.with_assertion_records(derivations))
    .map_err(|_| invalid())?;
    crate::analyzer::checkpoint(stop)?;
    Ok(ProjectSourceGraphProposals {
        registry,
        batch,
        coverage,
        provenance,
        limits,
        inventory_batch,
    })
}

#[derive(Serialize)]
struct CollectedSourceGraph {
    registry: GraphRegistryBundle,
    universe: GraphUniverseId,
    generation: GraphGenerationId,
    entities: Vec<EntityDraft>,
    relations: Vec<RelationDraft>,
    coverage: Vec<GraphCoverageRecord>,
    provenance: ProjectGraphProvenance,
    limits: GraphLimits,
    inventory_batch: Option<GraphProposalBatch>,
    #[serde(skip)]
    text_bytes: usize,
}

fn collect_source_graph_proposals(
    project: &ProjectView,
    stop: &AtomicBool,
) -> ProjectResult<CollectedSourceGraph> {
    crate::analyzer::checkpoint(stop)?;
    project.snapshot().validate()?;
    crate::analyzer::checkpoint(stop)?;
    let config = project.configuration();
    let profile = source_graph_profile(config);
    // Fail selected-route owner mismatches before any source graph output.
    let load_scopes = load_inputs::scopes(project, stop)?;
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
                let path = package_plan
                    .source_path(&package.package, &source.path)
                    .ok_or_else(invalid)?;
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
    let raw_selected = raw_inventory::selected(config);
    let registry = registry(config.project_kind(), raw_selected)?;
    let universe = match config.project_kind() {
        ProjectKind::BlizzardUiPlatformSource => {
            let binding = config.platform_package_binding().ok_or_else(invalid)?;
            GraphUniverseId::new(binding.universe_id()).map_err(|_| invalid())?
        }
        ProjectKind::Fixture | ProjectKind::Repository => {
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
            GraphUniverseId::new(format!("project:{scope}")).map_err(|_| invalid())?
        }
    };
    // This seed is not a published GraphGeneration. The graph owner derives the
    // materialized generation from the exact registry and accepted partition.
    let generation = input_generation(project, &registry)?;
    let limits = GraphLimits::new(
        MAX_NODES as u32,
        MAX_EDGES as u32,
        32,
        64,
        MAX_EDGES.min(MAX_QUERY_EDGES) as u32,
    )
    .map_err(|_| invalid())?;
    let mut provenance = ProjectGraphProvenance {
        profile,
        project_snapshot_id: project.snapshot_id().into(),
        analyzer_snapshot_id: project.analyzer_snapshot_id().into(),
        context: project.snapshot().generation_context().clone(),
        files: Vec::new(),
        packages: Vec::new(),
        package_files: Vec::new(),
        package_dependencies: Vec::new(),
        package_loads: Vec::new(),
        toc_facts: Vec::new(),
        xml_facts: Vec::new(),
        xml_containment: Vec::new(),
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
        package_xml_binding_report: None,
        raw_inventory: None,
        source_handles: BTreeMap::new(),
        evidence: BTreeMap::new(),
        load_plan: plan.cloned(),
        package_load_plan: package_plan.cloned(),
        package_main_plan: package_main_plan.cloned(),
        skipped_missing_targets: 0,
        skipped_self_loads: 0,
    };
    if config.platform_graph_profile().is_some() {
        let bindings = project
            .snapshot()
            .analyzer_binding()
            .package_xml_bindings()
            .ok_or_else(invalid)?;
        charge(&mut text_bytes, bindings.serialized_byte_length())?;
        provenance.package_xml_binding_report = Some(bindings.clone());
    }
    let inventory_batch = if raw_selected {
        let (batch, manifest) = raw_inventory::project(
            project,
            &registry,
            &universe,
            &generation,
            &mut text_bytes,
            stop,
        )?;
        provenance.raw_inventory = Some(manifest);
        Some(batch)
    } else {
        None
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
        entities.push(EntityDraft::new(
            PlatformGraphProducer::Inventory,
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
        ));
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
            let producer = if record.document == plan.selected_toc() {
                PlatformGraphProducer::TocLoad
            } else if plan.xml_documents().contains_key(&record.document) {
                PlatformGraphProducer::XmlStructure
            } else {
                return Err(invalid());
            };
            relations.push(
                RelationDraft::new(
                    producer,
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
    provenance.toc_facts = toc_facts::project(
        project,
        &source_by_path,
        &mut provenance,
        &mut text_bytes,
        stop,
    )?;
    let (xml_facts, xml_containment) =
        xml_facts::project(project, &mut provenance, &mut text_bytes, stop)?;
    provenance.xml_facts = xml_facts;
    provenance.xml_containment = xml_containment;
    if entities.len().saturating_add(
        inventory_batch
            .as_ref()
            .map_or(0, |batch| batch.entity_proposals().len()),
    ) > MAX_NODES
        || relations.len() > MAX_EDGES
    {
        return Err(exhausted());
    }
    let xml_state = if load_scopes
        .iter()
        .any(|input| !input.plan().xml_documents().is_empty())
    {
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
    crate::analyzer::checkpoint(stop)?;
    Ok(CollectedSourceGraph {
        registry,
        universe,
        generation,
        entities,
        relations,
        coverage,
        provenance,
        limits,
        inventory_batch,
        text_bytes,
    })
}

fn input_generation(
    project: &ProjectView,
    registry: &GraphRegistryBundle,
) -> ProjectResult<GraphGenerationId> {
    let seed = crate::identity::canonical_digest(
        "wow-project/source-graph-input/1",
        &(
            source_graph_profile(project.configuration()),
            registry.registry_digest(),
            project.snapshot_id(),
        ),
        ProjectPhase::View,
    )?;
    GraphGenerationId::new(format!("source-graph-input:{seed}")).map_err(|_| invalid())
}
