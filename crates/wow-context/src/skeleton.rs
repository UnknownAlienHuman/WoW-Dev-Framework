//! Typed container/entity projections of exact retained owners. Source bodies,
//! parser handles and inferred semantic relationships never enter these artifacts.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_graph::{GraphEdge, GraphNode, GraphNodeId, GraphPartitionRead, GraphRelationKind};
use wow_project::{
    ProjectId,
    graph::{
        ProjectCallableMetadata, ProjectGraphInlineHandler, ProjectGraphLuaDeclaration,
        ProjectGraphScriptSource, ProjectGraphStateDeclaration, ProjectGraphStatePath,
        ProjectGraphStateRoot, ProjectGraphXmlDeclaration, SOURCE_GRAPH_PARTITION,
    },
};

use crate::budget::{Budget, canonical, checkpoint, exhausted, identity};
use crate::{
    ContextError, ContextErrorCode, ContextResult, ContextUniverseSet, ProjectMap, ProjectMapState,
    ProjectMapView, open_project_map_view,
};

pub const PRIMARY_L0_PROFILE: &str = "wow-context/primary-l0/1";
pub const PRIMARY_L1_PROFILE: &str = "wow-context/primary-l1/1";
pub const PRIMARY_STATE_ROOT_L0_PROFILE: &str = "wow-context/primary-state-root-l0/1";
pub const PRIMARY_STATE_DETAIL_L1_PROFILE: &str = "wow-context/primary-state-detail-l1/1";
pub const PRIMARY_REFERENCE_FACTS_L1_PROFILE: &str = "wow-context/primary-reference-facts-l1/1";

/// Explicit entity detail recipes over the same retained native neighborhood.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryL1Profile {
    EntityV1,
    StateDetailV1,
    ReferenceFactsV1,
}
impl PrimaryL1Profile {
    pub const fn profile_id(self) -> &'static str {
        self.descriptor().profile_id
    }
    const fn descriptor(self) -> &'static SkeletonProfile {
        match self {
            Self::EntityV1 => &L1_PROFILE,
            Self::StateDetailV1 => &STATE_DETAIL_L1_PROFILE,
            Self::ReferenceFactsV1 => &REFERENCE_FACTS_L1_PROFILE,
        }
    }
    const fn schema(self) -> &'static str {
        match self {
            Self::EntityV1 => "wow-context/l1-skeleton/primary/1",
            Self::StateDetailV1 => "wow-context/l1-skeleton/primary-state-detail/1",
            Self::ReferenceFactsV1 => "wow-context/l1-skeleton/primary-reference-facts/1",
        }
    }
}

/// Closed container selections. The existing descriptor and default bytes are
/// preserved; the state-root descriptor adds only original native state facets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryL0Profile {
    ContainersV1,
    StateRootV1,
}
impl PrimaryL0Profile {
    pub const fn profile_id(self) -> &'static str {
        self.descriptor().profile_id
    }
    const fn descriptor(self) -> &'static SkeletonProfile {
        match self {
            Self::ContainersV1 => &L0_PROFILE,
            Self::StateRootV1 => &STATE_ROOT_L0_PROFILE,
        }
    }
}

#[derive(Serialize, PartialEq, Eq)]
struct SkeletonProfile {
    profile_id: &'static str,
    scope_kinds: &'static [&'static str],
    relations: &'static [GraphRelationKind],
    direction: &'static str,
    depth: u8,
    declarations: &'static str,
    types: &'static str,
    ordering: &'static str,
    privacy: &'static str,
    source_bodies: &'static str,
    tokens: &'static str,
    continuation: &'static str,
}
const L0_PROFILE: SkeletonProfile = SkeletonProfile {
    profile_id: PRIMARY_L0_PROFILE,
    scope_kinds: &["project", "source_file", "source_package"],
    relations: &crate::profile::RELATIONS,
    direction: "both",
    depth: 1,
    declarations: "selected_native_callable_and_lua_xml_occurrence_metadata",
    types: "original_selected_native_signature_state_only",
    ordering: "native_node_id_then_native_edge_id",
    privacy: "local_metadata_only",
    source_bodies: "excluded",
    tokens: "unavailable",
    continuation: "not_evaluated",
};
const L1_PROFILE: SkeletonProfile = SkeletonProfile {
    profile_id: PRIMARY_L1_PROFILE,
    scope_kinds: &["native_graph_entity"],
    relations: &crate::profile::DETAIL_RELATIONS,
    ..L0_PROFILE
};
const STATE_ROOT_L0_PROFILE: SkeletonProfile = SkeletonProfile {
    profile_id: PRIMARY_STATE_ROOT_L0_PROFILE,
    scope_kinds: &["state_root"],
    declarations: "selected_native_saved_variable_root_and_literal_path_metadata",
    types: "symbolic_literal_keys_only_no_runtime_values",
    ..L0_PROFILE
};
const STATE_DETAIL_L1_PROFILE: SkeletonProfile = SkeletonProfile {
    profile_id: PRIMARY_STATE_DETAIL_L1_PROFILE,
    scope_kinds: &["state_root", "state_path"],
    declarations: "selected_native_callable_lua_xml_and_saved_variable_root_literal_path_metadata",
    types: "original_native_signature_state_and_symbolic_literal_keys_no_runtime_values",
    ..L1_PROFILE
};
const STATE_DETAIL_PROFILE_BYTES: &[u8] = include_bytes!("../e3/native-state-detail/profile.json");
const REFERENCE_FACTS_L1_PROFILE: SkeletonProfile = SkeletonProfile {
    profile_id: PRIMARY_REFERENCE_FACTS_L1_PROFILE,
    declarations: "source_normalized_native_callable_and_original_reference_records",
    types: "original_source_type_identities_unresolved",
    ..L1_PROFILE
};
const REFERENCE_FACTS_PROFILE_BYTES: &[u8] =
    include_bytes!("../e3/native-reference-facts/profile.json");

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum L0Scope {
    Project(ProjectId),
    Entity(GraphNodeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SkeletonLevel {
    L0,
    L1,
    Both,
}

#[derive(Serialize)]
pub struct PrimarySkeletons<'m, 'a> {
    schema: &'static str,
    level: SkeletonLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    l0: Option<L0Skeleton<'m, 'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    l1: Option<L1Skeleton<'m, 'a>>,
    work_reserved: u64,
    bytes_reserved: u64,
    #[serde(skip)]
    remaining_metadata_bytes: u64,
}
impl PrimarySkeletons<'_, '_> {
    /// Actual allowance after immutable inspection construction. This snapshot
    /// is only for final delivery; it cannot reconstruct a continuing ledger.
    pub const fn remaining_metadata_bytes(&self) -> u64 {
        self.remaining_metadata_bytes
    }
    pub(crate) fn l0_id(&self) -> Option<&str> {
        self.l0.as_ref().map(L0Skeleton::l0_skeleton_id)
    }
    pub(crate) fn l1_id(&self) -> Option<&str> {
        self.l1.as_ref().map(L1Skeleton::l1_skeleton_id)
    }
    pub(crate) fn matches_l1_selection(
        &self,
        root: &L0Scope,
        level: SkeletonLevel,
        profile: Option<PrimaryL1Profile>,
    ) -> bool {
        if self.level != level
            || self.l0.is_some() != matches!(level, SkeletonLevel::L0 | SkeletonLevel::Both)
        {
            return false;
        }
        match (&self.l1, profile, root) {
            (None, None, _) => level == SkeletonLevel::L0,
            (Some(skeleton), Some(profile), L0Scope::Entity(id)) => {
                skeleton.core.root.node_id() == id
                    && skeleton.core.schema == profile.schema()
                    && skeleton.core.profile == profile.profile_id()
                    && skeleton.core.profile_descriptor == profile.descriptor()
                    && match profile {
                        PrimaryL1Profile::EntityV1 => true,
                        PrimaryL1Profile::StateDetailV1 => {
                            level == SkeletonLevel::L1
                                && matches!(skeleton.core.root.kind(), "state_root" | "state_path")
                        }
                        PrimaryL1Profile::ReferenceFactsV1 => {
                            level == SkeletonLevel::L1 && skeleton.core.reference_records.is_some()
                        }
                    }
            }
            _ => false,
        }
    }
    pub fn state(&self) -> ProjectMapState {
        if self
            .l0
            .as_ref()
            .is_some_and(|v| v.state() == ProjectMapState::Truncated)
            || self
                .l1
                .as_ref()
                .is_some_and(|v| v.state() == ProjectMapState::Truncated)
        {
            ProjectMapState::Truncated
        } else {
            ProjectMapState::Partial
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Scope<'a> {
    Project { project_id: &'a ProjectId },
    Entity { node: &'a GraphNode },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Declaration<'a> {
    Callable {
        node: &'a GraphNode,
        metadata: ProjectCallableMetadata<'a>,
    },
    Lua {
        node: &'a GraphNode,
        origin: &'a ProjectGraphLuaDeclaration,
    },
    Xml {
        node: &'a GraphNode,
        origin: &'a ProjectGraphXmlDeclaration,
    },
    XmlHandler {
        node: &'a GraphNode,
        origin: &'a ProjectGraphInlineHandler,
        source: &'a ProjectGraphScriptSource,
    },
    StateRoot {
        node: &'a GraphNode,
        origin: &'a ProjectGraphStateRoot,
        declarations: Vec<&'a ProjectGraphStateDeclaration>,
    },
    StatePath {
        node: &'a GraphNode,
        origin: &'a ProjectGraphStatePath,
    },
}

#[derive(Serialize)]
struct SkeletonContext<'m, 'a> {
    universe_set_id: &'m str,
    origin_map_id: &'m str,
    /// The immutable origin projection contains the full mandatory native
    /// support/source/load/coverage/conflict closure referenced by these facets.
    origins: &'m ProjectMap<'a>,
    state: ProjectMapState,
    absence_authoritative: bool,
    privacy: &'static str,
    source_bodies: &'static str,
    tokens: &'static str,
    work_reserved: u64,
    facet_bytes_reserved: u64,
}

#[derive(Serialize)]
struct L0Core<'m, 'a> {
    schema: &'static str,
    profile: &'static str,
    profile_descriptor: &'static SkeletonProfile,
    context: SkeletonContext<'m, 'a>,
    scope: Scope<'a>,
    direct_members: Vec<&'a GraphNode>,
    selected_member_count: usize,
    member_enumeration: &'static str,
    declarations: Vec<Declaration<'a>>,
    direct_relations: Vec<&'a GraphEdge>,
    detail_routes: Vec<&'a GraphNodeId>,
    not_evaluated: &'static [&'static str],
}
#[derive(Serialize)]
pub struct L0Skeleton<'m, 'a> {
    l0_skeleton_id: Box<str>,
    core: L0Core<'m, 'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
}
impl L0Skeleton<'_, '_> {
    pub fn l0_skeleton_id(&self) -> &str {
        &self.l0_skeleton_id
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    pub const fn state(&self) -> ProjectMapState {
        self.core.context.state
    }
}

#[derive(Serialize)]
struct L1Core<'m, 'a> {
    schema: &'static str,
    profile: &'static str,
    profile_descriptor: &'static SkeletonProfile,
    context: SkeletonContext<'m, 'a>,
    root: &'a GraphNode,
    local_entities: &'m [&'a GraphNode],
    direct_relations: &'m [&'a GraphEdge],
    declarations: Vec<Declaration<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_records: Option<Vec<crate::reference::ReferenceRecordFacet<'a>>>,
    relation_interpretation: &'static str,
    not_evaluated: &'static [&'static str],
}
#[derive(Serialize)]
pub struct L1Skeleton<'m, 'a> {
    l1_skeleton_id: Box<str>,
    core: L1Core<'m, 'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
}
impl L1Skeleton<'_, '_> {
    pub fn l1_skeleton_id(&self) -> &str {
        &self.l1_skeleton_id
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    pub const fn state(&self) -> ProjectMapState {
        self.core.context.state
    }
}

const NOT_EVALUATED: &[&str] = &[
    "unselected_native_declaration_or_type_recipes",
    "reference_fact_enrichment",
    "control_effect_projection",
    "source_excerpts_and_exact_tokens",
    "deeper_paths_and_retained_continuation",
    "complete_corpus_or_runtime_behavior",
];

const REFERENCE_FACTS_NOT_EVALUATED: &[&str] = &[
    "unselected_native_declaration_or_type_recipes",
    "reference_type_resolution_and_annotation_lowering",
    "control_effect_projection",
    "source_excerpts_and_exact_tokens",
    "deeper_paths_and_retained_continuation",
    "complete_corpus_or_runtime_behavior",
];

fn invalid() -> ContextError {
    ContextError::new(ContextErrorCode::RootInvalid)
}
fn missing() -> ContextError {
    ContextError::new(ContextErrorCode::MissingRequiredCustody)
}

fn validate_input<'m, 'a>(
    universe: &ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    stop: &AtomicBool,
) -> ContextResult<ProjectMapView<'m, 'a>> {
    checkpoint(stop)?;
    let view = open_project_map_view(map, stop)?;
    if !view.is_skeleton_origin() {
        return Err(ContextError::new(ContextErrorCode::SupportMismatch));
    }
    if !view.bound_to(universe) || !std::ptr::eq(graph.owner(), universe.pair.graph()) {
        return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
    }
    Ok(view)
}

fn input<'m, 'a>(
    universe: &ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    stop: &AtomicBool,
) -> ContextResult<(ProjectMapView<'m, 'a>, Budget)> {
    let view = validate_input(universe, map, graph, stop)?;
    let mut budget = Budget::new(universe.limits);
    budget.work(
        usize::try_from(view.work_reserved()).map_err(|_| exhausted())?,
        stop,
    )?;
    budget.retain(map, stop)?;
    Ok((view, budget))
}

fn context<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    budget: &Budget,
) -> SkeletonContext<'m, 'a> {
    SkeletonContext {
        universe_set_id: universe.universe_set_id(),
        origin_map_id: map.project_map_id(),
        origins: map,
        state: map.state(),
        absence_authoritative: false,
        privacy: "local_metadata_only",
        source_bodies: "excluded",
        tokens: "unavailable",
        work_reserved: budget.work,
        facet_bytes_reserved: budget.retained_bytes,
    }
}

fn published_lookup_work(graph: &GraphPartitionRead<'_>) -> ContextResult<usize> {
    let partition = graph
        .owner()
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(missing)?;
    Ok(
        partition.batch().entity_proposals().len().max(1).ilog2() as usize
            + partition.report().accepted_entities().len().max(1).ilog2() as usize
            + graph.owner().snapshot().nodes().len().max(1).ilog2() as usize
            + graph.owner().snapshot().limits().max_evidence_per_record as usize
                * (graph
                    .owner()
                    .snapshot()
                    .limits()
                    .max_evidence_per_record
                    .max(1)
                    .ilog2() as usize
                    + 1)
            + 4,
    )
}

fn declarations<'a>(
    universe: &ContextUniverseSet<'a>,
    view: &ProjectMapView<'_, 'a>,
    graph: &GraphPartitionRead<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Vec<Declaration<'a>>> {
    let provenance = universe.pair.source_provenance().ok_or_else(missing)?;
    let mut rows = Vec::new();
    // Native metadata uses immutable binary-search joins. Charge that actual
    // bounded route rather than an unrelated quadratic whole-report reservation.
    let report_rows = provenance
        .function_call_report()
        .map_or(0, |r| r.functions().len());
    let lookup_work = published_lookup_work(graph)?;
    let row_count = provenance
        .functions()
        .len()
        .checked_add(provenance.lua_declarations().len())
        .and_then(|n| n.checked_add(provenance.xml_declarations().len()))
        .ok_or_else(exhausted)?;
    budget.work(
        row_count.checked_mul(lookup_work).ok_or_else(exhausted)?,
        stop,
    )?;
    let metadata_lookup_work = provenance.functions().len().max(1).ilog2() as usize
        + 2 * (report_rows.max(1).ilog2() as usize + 1)
        + provenance.files().len().max(1).ilog2() as usize
        + provenance.source_handles().len().max(1).ilog2() as usize
        + provenance.evidence().len().max(1).ilog2() as usize
        + graph.owner().snapshot().limits().max_evidence_per_record as usize
        + 8;
    budget.work(
        view.nodes()
            .len()
            .checked_mul(metadata_lookup_work)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    for origin in provenance.functions() {
        budget.work(1, stop)?;
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if view
            .node(node.node_id())
            .is_some_and(|selected| std::ptr::eq(selected, node))
        {
            let metadata = provenance
                .callable_metadata(&origin.function_id)
                .map_err(|_| missing())?
                .ok_or_else(missing)?;
            let row = Declaration::Callable { node, metadata };
            budget.retain(&row, stop)?;
            rows.push(row);
        }
    }
    for origin in provenance.lua_declarations() {
        budget.work(1, stop)?;
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if view
            .node(node.node_id())
            .is_some_and(|selected| std::ptr::eq(selected, node))
        {
            let row = Declaration::Lua { node, origin };
            budget.retain(&row, stop)?;
            rows.push(row);
        }
    }
    for origin in provenance.xml_declarations() {
        budget.work(1, stop)?;
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if view
            .node(node.node_id())
            .is_some_and(|selected| std::ptr::eq(selected, node))
        {
            let row = Declaration::Xml { node, origin };
            budget.retain(&row, stop)?;
            rows.push(row);
        }
    }
    let sort_work = rows
        .len()
        .checked_mul(rows.len().max(1).ilog2() as usize + 1)
        .ok_or_else(exhausted)?;
    budget.work(sort_work, stop)?;
    rows.sort_by(|a, b| {
        declaration_node(a)
            .node_id()
            .cmp(declaration_node(b).node_id())
    });
    if rows
        .windows(2)
        .any(|pair| declaration_node(&pair[0]).node_id() == declaration_node(&pair[1]).node_id())
    {
        return Err(ContextError::new(ContextErrorCode::SupportMismatch));
    }
    Ok(rows)
}
// L1-only metadata from the same admitted source proposal. Source bodies and
// callable signatures are absent from both borrowed native records.
fn xml_handler_declarations<'a>(
    universe: &ContextUniverseSet<'a>,
    view: &ProjectMapView<'_, 'a>,
    graph: &GraphPartitionRead<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Vec<Declaration<'a>>> {
    checkpoint(stop)?;
    let provenance = universe.pair.source_provenance().ok_or_else(missing)?;
    let mut rows = Vec::new();
    if provenance.inline_handlers().is_empty() {
        return Ok(rows);
    }
    budget.work(
        provenance
            .inline_handlers()
            .len()
            .checked_mul(published_lookup_work(graph)?)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    for origin in provenance.inline_handlers() {
        budget.work(1, stop)?;
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if !view
            .node(node.node_id())
            .is_some_and(|selected| std::ptr::eq(selected, node))
        {
            continue;
        }
        if node.kind() != "xml_source_handler"
            || !origin.semantic_context.admits_static_source_association()
        {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
        // A script_id alone is not a cross-scope join. The source constructor
        // supplies this exact document and support pair with the handler.
        budget.work(provenance.script_sources().len(), stop)?;
        let mut source = None;
        for candidate in provenance.script_sources() {
            checkpoint(stop)?;
            if candidate.script_id == origin.script_id
                && candidate.document == origin.document
                && candidate.source_handle_id == origin.source_handle_id
                && candidate.evidence_id == origin.evidence_id
                && source.replace(candidate).is_some()
            {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
        }
        let source = source.ok_or_else(missing)?;
        if source.source_kind != wow_project::load::XmlScriptSource::InlineBody {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
        let row = Declaration::XmlHandler {
            node,
            origin,
            source,
        };
        budget.retain(&row, stop)?;
        rows.push(row);
    }
    Ok(rows)
}

fn declaration_node<'a>(row: &Declaration<'a>) -> &'a GraphNode {
    match row {
        Declaration::Callable { node, .. }
        | Declaration::Lua { node, .. }
        | Declaration::Xml { node, .. }
        | Declaration::XmlHandler { node, .. }
        | Declaration::StateRoot { node, .. }
        | Declaration::StatePath { node, .. } => node,
    }
}

fn state_root_declaration<'a>(
    root_node: &'a GraphNode,
    root: &'a ProjectGraphStateRoot,
    provenance: &'a wow_project::graph::ProjectGraphProvenance,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Declaration<'a>> {
    let declaration_work = root
        .source_handle_ids
        .len()
        .checked_add(root.evidence_ids.len())
        .and_then(|n| n.checked_add(3))
        .ok_or_else(exhausted)?;
    budget.work(
        provenance
            .state_declarations()
            .len()
            .checked_mul(declaration_work)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut declarations = Vec::new();
    for declaration in provenance.state_declarations() {
        budget.work(1, stop)?;
        // Preserve each original declared TOC support pair and multiplicity.
        if declaration.name == root.name
            && declaration.scope == root.scope
            && declaration.state == wow_project::load::TocSavedVariableState::Declared
            && root
                .source_handle_ids
                .contains(&declaration.source_handle_id)
            && root.evidence_ids.contains(&declaration.evidence_id)
        {
            budget.retain(declaration, stop)?;
            declarations.push(declaration);
        }
    }
    if declarations.is_empty() {
        return Err(missing());
    }
    let row = Declaration::StateRoot {
        node: root_node,
        origin: root,
        declarations,
    };
    budget.retain(&row, stop)?;
    Ok(row)
}

fn state_declarations<'a>(
    universe: &ContextUniverseSet<'a>,
    view: &ProjectMapView<'_, 'a>,
    graph: &GraphPartitionRead<'a>,
    members: &[&'a GraphNode],
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Vec<Declaration<'a>>> {
    let provenance = universe.pair.source_provenance().ok_or_else(missing)?;
    let root_node = view.root_node().ok_or_else(invalid)?;
    let lookup_work = published_lookup_work(graph)?;
    budget.work(
        provenance
            .state_roots()
            .len()
            .checked_mul(lookup_work)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut root = None;
    for origin in provenance.state_roots() {
        budget.work(1, stop)?;
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if std::ptr::eq(node, root_node) && root.replace(origin).is_some() {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
    }
    let root = root.ok_or_else(missing)?;
    let row = state_root_declaration(root_node, root, provenance, budget, stop)?;
    let mut rows = vec![row];
    let path_lookup_work = lookup_work
        .checked_add(members.len().max(1).ilog2() as usize + 2)
        .ok_or_else(exhausted)?;
    budget.work(
        provenance
            .state_paths()
            .len()
            .checked_mul(path_lookup_work)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    for origin in provenance.state_paths() {
        budget.work(1, stop)?;
        if origin.root_id != root.root_id {
            continue;
        }
        let node = graph
            .published_entity(SOURCE_GRAPH_PARTITION, &origin.proposal_id, stop)
            .map_err(crate::error::graph_error)?;
        if members
            .binary_search_by(|member| member.node_id().cmp(node.node_id()))
            .is_ok()
        {
            if !view
                .node(node.node_id())
                .is_some_and(|selected| std::ptr::eq(selected, node))
            {
                return Err(missing());
            }
            let row = Declaration::StatePath { node, origin };
            budget.retain(&row, stop)?;
            rows.push(row);
        }
    }
    // Every selected membership row requires its original typed path custody.
    // Omitted/dynamic/unresolved paths remain the original owner's omissions.
    if rows.len().checked_sub(1) != Some(members.len()) {
        return Err(missing());
    }
    budget.work(
        rows.len()
            .checked_mul(rows.len().max(1).ilog2() as usize + 1)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    rows.sort_by(|a, b| {
        declaration_node(a)
            .node_id()
            .cmp(declaration_node(b).node_id())
    });
    if rows
        .windows(2)
        .any(|pair| declaration_node(&pair[0]).node_id() == declaration_node(&pair[1]).node_id())
    {
        return Err(ContextError::new(ContextErrorCode::SupportMismatch));
    }
    Ok(rows)
}

fn local_state_node<'a>(
    graph: &GraphPartitionRead<'a>,
    view: &ProjectMapView<'_, 'a>,
    proposal_id: &str,
    stop: &AtomicBool,
) -> ContextResult<Option<&'a GraphNode>> {
    checkpoint(stop)?;
    let accepted = graph
        .owner()
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(missing)?
        .report()
        .accepted_entities();
    // An unaccepted outside proposal is not a published local facet. A
    // required local state node without original custody is refused below.
    if accepted
        .binary_search_by(|entry| entry.proposal_id().cmp(proposal_id))
        .is_err()
    {
        return Ok(None);
    }
    let node = graph
        .published_entity(SOURCE_GRAPH_PARTITION, proposal_id, stop)
        .map_err(crate::error::graph_error)?;
    match view.node(node.node_id()) {
        Some(selected) if std::ptr::eq(selected, node) => Ok(Some(node)),
        Some(_) => Err(ContextError::new(ContextErrorCode::SupportMismatch)),
        None => Ok(None),
    }
}

fn state_detail_declarations<'a>(
    universe: &ContextUniverseSet<'a>,
    view: &ProjectMapView<'_, 'a>,
    graph: &GraphPartitionRead<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<Vec<Declaration<'a>>> {
    let provenance = universe.pair.source_provenance().ok_or_else(missing)?;
    let accepted_count = graph
        .owner()
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(missing)?
        .report()
        .accepted_entities()
        .len();
    let lookup_work = published_lookup_work(graph)?
        .checked_add(accepted_count.max(1).ilog2() as usize + 1)
        .and_then(|work| work.checked_add(view.nodes().len().max(1).ilog2() as usize + 2))
        .ok_or_else(exhausted)?;
    budget.work(
        provenance
            .state_roots()
            .len()
            .checked_add(provenance.state_paths().len())
            .and_then(|count| count.checked_mul(lookup_work))
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut roots = Vec::new();
    let mut rows = Vec::new();
    for origin in provenance.state_roots() {
        budget.work(1, stop)?;
        if let Some(node) = local_state_node(graph, view, &origin.proposal_id, stop)? {
            if node.kind() != "state_root" {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
            rows.push(state_root_declaration(
                node, origin, provenance, budget, stop,
            )?);
            roots.push(origin);
        }
    }
    budget.work(
        provenance
            .state_paths()
            .len()
            .checked_mul(roots.len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    for origin in provenance.state_paths() {
        budget.work(1, stop)?;
        if let Some(node) = local_state_node(graph, view, &origin.proposal_id, stop)? {
            if node.kind() != "state_path" {
                return Err(ContextError::new(ContextErrorCode::SupportMismatch));
            }
            // Exact producer root identity, never a name/field inference. Its
            // published local root and complete TOC support must be retained.
            if roots
                .iter()
                .filter(|root| root.root_id == origin.root_id)
                .count()
                != 1
            {
                return Err(missing());
            }
            let row = Declaration::StatePath { node, origin };
            budget.retain(&row, stop)?;
            rows.push(row);
        }
    }
    budget.work(view.nodes().len(), stop)?;
    let required = view
        .nodes()
        .iter()
        .filter(|node| matches!(node.kind(), "state_root" | "state_path"))
        .count();
    if rows.len() != required {
        return Err(missing());
    }
    Ok(rows)
}

pub fn build_l0_skeleton<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    scope: &L0Scope,
    stop: &AtomicBool,
) -> ContextResult<L0Skeleton<'m, 'a>> {
    build_l0_skeleton_with_profile(
        universe,
        map,
        graph,
        scope,
        PrimaryL0Profile::ContainersV1,
        stop,
    )
}

pub fn build_l0_skeleton_with_profile<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    scope: &L0Scope,
    profile: PrimaryL0Profile,
    stop: &AtomicBool,
) -> ContextResult<L0Skeleton<'m, 'a>> {
    let (view, mut budget) = input(universe, map, graph, stop)?;
    l0(
        universe,
        map,
        graph,
        (scope, profile),
        &view,
        &mut budget,
        stop,
    )
}
fn l0<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    selection: (&L0Scope, PrimaryL0Profile),
    view: &ProjectMapView<'m, 'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<L0Skeleton<'m, 'a>> {
    let (scope, profile) = selection;
    let descriptor = profile.descriptor();
    let (scope, membership) = match scope {
        L0Scope::Project(id)
            if profile == PrimaryL0Profile::ContainersV1
                && id == universe.pair.project().configuration().project_id()
                && view.root_node().is_none() =>
        {
            (
                Scope::Project {
                    project_id: universe.pair.project().configuration().project_id(),
                },
                "retained_source_file_projection_membership",
            )
        }
        L0Scope::Entity(id) => {
            let node = view
                .root_node()
                .filter(|node| node.node_id() == id)
                .ok_or_else(invalid)?;
            let eligible = match profile {
                PrimaryL0Profile::ContainersV1 => {
                    matches!(node.kind(), "source_file" | "source_package")
                }
                PrimaryL0Profile::StateRootV1 => node.kind() == "state_root",
            };
            if !eligible {
                return Err(invalid());
            }
            (
                Scope::Entity { node },
                "selected_outgoing_native_owns_or_contains",
            )
        }
        _ => return Err(invalid()),
    };
    budget.retain(descriptor, stop)?;
    let mut members = Vec::new();
    for node in view.nodes() {
        budget.work(1, stop)?;
        budget.work(view.edges().len(), stop)?;
        let include = match scope {
            Scope::Project { .. } => node.kind() == "source_file",
            Scope::Entity { node: root } => view.edges().iter().any(|edge| {
                edge.from() == root.node_id()
                    && edge.to() == node.node_id()
                    && matches!(
                        edge.relation(),
                        GraphRelationKind::Owns | GraphRelationKind::Contains
                    )
            }),
        };
        if include {
            budget.retain(node, stop)?;
            members.push(*node);
        }
    }
    let rows = match profile {
        PrimaryL0Profile::ContainersV1 => declarations(universe, view, graph, budget, stop)?,
        PrimaryL0Profile::StateRootV1 => {
            state_declarations(universe, view, graph, &members, budget, stop)?
        }
    };
    let mut relations = Vec::new();
    for edge in view.edges() {
        budget.work(1, stop)?;
        if descriptor.relations.contains(&edge.relation()) {
            budget.retain(edge, stop)?;
            relations.push(*edge);
        }
    }
    let mut routes = Vec::new();
    for node in &members {
        budget.work(1, stop)?;
        budget.retain(node.node_id(), stop)?;
        routes.push(node.node_id());
    }
    let core = L0Core {
        schema: match profile {
            PrimaryL0Profile::ContainersV1 => "wow-context/l0-skeleton/primary/1",
            PrimaryL0Profile::StateRootV1 => "wow-context/l0-skeleton/primary-state-root/1",
        },
        profile: descriptor.profile_id,
        profile_descriptor: descriptor,
        context: context(universe, map, budget),
        scope,
        selected_member_count: members.len(),
        direct_members: members,
        member_enumeration: membership,
        declarations: rows,
        direct_relations: relations,
        detail_routes: routes,
        not_evaluated: NOT_EVALUATED,
    };
    let id = identity(
        core.schema,
        "l0-skeleton:sha256:",
        &core,
        universe.limits.max_metadata_bytes,
        stop,
    )?;
    let mut artifact = L0Skeleton {
        l0_skeleton_id: id,
        core,
        canonical_bytes: Box::new([]),
    };
    budget.retain(&artifact, stop)?;
    artifact.canonical_bytes =
        canonical(&artifact, universe.limits.max_metadata_bytes, stop)?.into_boxed_slice();
    Ok(artifact)
}

pub fn build_l1_skeleton<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &GraphNodeId,
    stop: &AtomicBool,
) -> ContextResult<L1Skeleton<'m, 'a>> {
    let (view, mut budget) = input(universe, map, graph, stop)?;
    l1(
        universe,
        map,
        graph,
        (root, PrimaryL1Profile::EntityV1, None),
        &view,
        &mut budget,
        stop,
    )
}
fn l1<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    selection: (
        &GraphNodeId,
        PrimaryL1Profile,
        Option<&[crate::reference::ReferenceOriginInput<'a>]>,
    ),
    view: &ProjectMapView<'m, 'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<L1Skeleton<'m, 'a>> {
    let (root, profile, reference_origins) = selection;
    if profile == PrimaryL1Profile::ReferenceFactsV1 && reference_origins.is_none() {
        return Err(missing());
    }
    if !view.has_detail_vocabulary() {
        return Err(ContextError::new(ContextErrorCode::SupportMismatch));
    }
    let descriptor = profile.descriptor();
    let frozen = match profile {
        PrimaryL1Profile::EntityV1 => None,
        PrimaryL1Profile::StateDetailV1 => Some(STATE_DETAIL_PROFILE_BYTES),
        PrimaryL1Profile::ReferenceFactsV1 => Some(REFERENCE_FACTS_PROFILE_BYTES),
    };
    if let Some(bytes) = frozen
        && canonical(descriptor, universe.limits.max_metadata_bytes, stop)?
            != bytes.strip_suffix(b"\n").unwrap_or(bytes)
    {
        return Err(ContextError::new(ContextErrorCode::EncodingInvalid));
    }
    budget.retain(descriptor, stop)?;
    let root = view
        .root_node()
        .filter(|node| node.node_id() == root)
        .ok_or_else(invalid)?;
    if profile == PrimaryL1Profile::StateDetailV1
        && !matches!(root.kind(), "state_root" | "state_path")
    {
        return Err(invalid());
    }
    let mut rows = declarations(universe, view, graph, budget, stop)?;
    let original_row_count = rows.len();
    rows.extend(xml_handler_declarations(
        universe, view, graph, budget, stop,
    )?);
    if profile == PrimaryL1Profile::StateDetailV1 {
        rows.extend(state_detail_declarations(
            universe, view, graph, budget, stop,
        )?);
    }
    // Common rows already passed checked order/dedup. Preserve the old
    // no-handler debit path; check the complete set whenever rows were added.
    if profile == PrimaryL1Profile::StateDetailV1 || rows.len() != original_row_count {
        budget.work(
            rows.len()
                .checked_mul(rows.len().max(1).ilog2() as usize + 1)
                .ok_or_else(exhausted)?,
            stop,
        )?;
        rows.sort_by(|a, b| {
            declaration_node(a)
                .node_id()
                .cmp(declaration_node(b).node_id())
        });
        if rows.windows(2).any(|pair| {
            declaration_node(&pair[0]).node_id() == declaration_node(&pair[1]).node_id()
        }) {
            return Err(ContextError::new(ContextErrorCode::SupportMismatch));
        }
    }
    if matches!(
        root.kind(),
        "lua_source_function"
            | "lua_source_declaration"
            | "xml_source_declaration"
            | "xml_source_handler"
    ) && !rows
        .iter()
        .any(|row| declaration_node(row).node_id() == root.node_id())
    {
        return Err(missing());
    }
    let reference_records = if profile == PrimaryL1Profile::ReferenceFactsV1 {
        Some(crate::reference::reference_facets(
            universe,
            view,
            graph,
            reference_origins.ok_or_else(missing)?,
            budget,
            stop,
        )?)
    } else {
        None
    };
    let core = L1Core {
        schema: profile.schema(),
        profile: descriptor.profile_id,
        profile_descriptor: descriptor,
        context: context(universe, map, budget),
        root,
        local_entities: map.native_nodes(),
        direct_relations: map.native_edges(),
        declarations: rows,
        reference_records,
        relation_interpretation: "original_direct_native_assertions_no_transitive_edges",
        not_evaluated: if profile == PrimaryL1Profile::ReferenceFactsV1 {
            REFERENCE_FACTS_NOT_EVALUATED
        } else {
            NOT_EVALUATED
        },
    };
    let id = identity(
        core.schema,
        "l1-skeleton:sha256:",
        &core,
        universe.limits.max_metadata_bytes,
        stop,
    )?;
    let mut artifact = L1Skeleton {
        l1_skeleton_id: id,
        core,
        canonical_bytes: Box::new([]),
    };
    budget.retain(&artifact, stop)?;
    artifact.canonical_bytes =
        canonical(&artifact, universe.limits.max_metadata_bytes, stop)?.into_boxed_slice();
    Ok(artifact)
}

/// An explicit L1 recipe uses the same typed engine and immutable inspection
/// envelope. It never creates an L0 artifact or silently selects an expansion.
pub fn build_primary_l1_skeletons_with_profile<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    profile: PrimaryL1Profile,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    build_primary_l1_skeletons_with_budget(universe, map, graph, root, profile, stop)
        .map(|(result, _)| result)
}

pub(crate) fn build_primary_l1_skeletons_with_budget<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    profile: PrimaryL1Profile,
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    primary_l1_skeletons(universe, map, graph, root, (profile, None), stop)
}

/// Explicit normalized Reference facts over one native entity neighborhood.
/// Origins are inert requests; the shared engine validates their native custody.
/// This recipe produces only L1, with no L0 or frontier reconstruction.
pub fn build_primary_reference_skeletons<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    origins: &[crate::reference::ReferenceOriginInput<'a>],
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    build_primary_reference_skeletons_with_budget(universe, map, graph, root, origins, stop)
        .map(|(result, _)| result)
}

/// Continue the actual input/declaration/Reference/artifact reservation ledger.
pub(crate) fn build_primary_reference_skeletons_with_budget<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    origins: &[crate::reference::ReferenceOriginInput<'a>],
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    primary_l1_skeletons(
        universe,
        map,
        graph,
        root,
        (PrimaryL1Profile::ReferenceFactsV1, Some(origins)),
        stop,
    )
}

fn primary_l1_skeletons<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    selection: (
        PrimaryL1Profile,
        Option<&[crate::reference::ReferenceOriginInput<'a>]>,
    ),
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    let (profile, reference_origins) = selection;
    let L0Scope::Entity(id) = root else {
        return Err(invalid());
    };
    let (view, mut budget) = input(universe, map, graph, stop)?;
    let artifact = l1(
        universe,
        map,
        graph,
        (id, profile, reference_origins),
        &view,
        &mut budget,
        stop,
    )?;
    finish_primary_skeletons(
        universe,
        SkeletonLevel::L1,
        None,
        Some(artifact),
        budget,
        stop,
    )
}

/// One fixed-order inspection with a shared aggregate work/output reservation.
/// Both artifacts preserve their distinct identities; common origins are not
/// re-queried, and a second level cannot reset the cumulative budget.
pub fn build_primary_skeletons<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    level: SkeletonLevel,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    build_primary_skeletons_with_profile(
        universe,
        map,
        graph,
        root,
        level,
        PrimaryL0Profile::ContainersV1,
        stop,
    )
}

pub fn build_primary_skeletons_with_profile<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    level: SkeletonLevel,
    profile: PrimaryL0Profile,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    build_primary_skeletons_with_budget(universe, map, graph, root, level, profile, stop)
        .map(|(result, _)| result)
}

/// Return the existing aggregate reservation ledger for continued pack construction.
pub(crate) fn build_primary_skeletons_with_budget<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &L0Scope,
    level: SkeletonLevel,
    profile: PrimaryL0Profile,
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    if profile == PrimaryL0Profile::StateRootV1
        && (level == SkeletonLevel::L1 || !matches!(root, L0Scope::Entity(_)))
    {
        return Err(invalid());
    }
    let (view, mut budget) = input(universe, map, graph, stop)?;
    let result = primary_skeletons_from_view(
        universe,
        map,
        graph,
        (root, level, profile),
        &view,
        &mut budget,
        stop,
    )?;
    Ok((result, budget))
}

/// Continue an existing construction ledger; the origin map was already charged.
/// Only the default container/entity recipes are selected by combined inspect.
pub(crate) fn build_primary_skeletons_on_budget<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    selection: (&L0Scope, SkeletonLevel),
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    budget.work(1, stop)?;
    let view = validate_input(universe, map, graph, stop)?;
    primary_skeletons_from_view(
        universe,
        map,
        graph,
        (selection.0, selection.1, PrimaryL0Profile::ContainersV1),
        &view,
        budget,
        stop,
    )
}

fn primary_skeletons_from_view<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    selection: (&L0Scope, SkeletonLevel, PrimaryL0Profile),
    view: &ProjectMapView<'m, 'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    let (root, level, profile) = selection;
    let l0 = if matches!(level, SkeletonLevel::L0 | SkeletonLevel::Both) {
        Some(l0(
            universe,
            map,
            graph,
            (root, profile),
            view,
            budget,
            stop,
        )?)
    } else {
        None
    };
    let l1 = if matches!(level, SkeletonLevel::L1 | SkeletonLevel::Both) {
        let L0Scope::Entity(id) = root else {
            return Err(invalid());
        };
        Some(l1(
            universe,
            map,
            graph,
            (id, PrimaryL1Profile::EntityV1, None),
            view,
            budget,
            stop,
        )?)
    } else {
        None
    };
    finish_primary_skeletons_on_budget(universe, level, l0, l1, budget, stop)
}

fn finish_primary_skeletons<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    level: SkeletonLevel,
    l0: Option<L0Skeleton<'m, 'a>>,
    l1: Option<L1Skeleton<'m, 'a>>,
    budget: Budget,
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    let result = finish_primary_skeletons_on_budget(universe, level, l0, l1, &budget, stop)?;
    Ok((result, budget))
}

fn finish_primary_skeletons_on_budget<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    level: SkeletonLevel,
    l0: Option<L0Skeleton<'m, 'a>>,
    l1: Option<L1Skeleton<'m, 'a>>,
    budget: &Budget,
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    let result = PrimarySkeletons {
        schema: "wow-context/primary-skeleton-inspection/1",
        level,
        l0,
        l1,
        work_reserved: budget.work,
        bytes_reserved: budget.retained_bytes,
        remaining_metadata_bytes: universe
            .limits
            .max_metadata_bytes
            .checked_sub(budget.retained_bytes)
            .ok_or_else(exhausted)?,
    };
    crate::budget::encoded_len(&result, universe.limits.max_metadata_bytes, stop)?;
    Ok(result)
}
