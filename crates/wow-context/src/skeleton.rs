//! Typed container/entity projections of exact retained owners. Source bodies,
//! parser handles and inferred semantic relationships never enter these artifacts.
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use wow_graph::{GraphEdge, GraphNode, GraphNodeId, GraphPartitionRead, GraphRelationKind};
use wow_project::{
    ProjectId,
    graph::{
        ProjectCallableMetadata, ProjectGraphLuaDeclaration, ProjectGraphXmlDeclaration,
        SOURCE_GRAPH_PARTITION,
    },
};

use crate::budget::{Budget, canonical, checkpoint, exhausted, identity};
use crate::{
    ContextError, ContextErrorCode, ContextResult, ContextUniverseSet, ProjectMap, ProjectMapState,
    ProjectMapView, open_project_map_view,
};

pub const PRIMARY_L0_PROFILE: &str = "wow-context/primary-l0/1";
pub const PRIMARY_L1_PROFILE: &str = "wow-context/primary-l1/1";

#[derive(Serialize)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
impl PrimarySkeletons<'_, '_> {
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

fn invalid() -> ContextError {
    ContextError::new(ContextErrorCode::RootInvalid)
}
fn missing() -> ContextError {
    ContextError::new(ContextErrorCode::MissingRequiredCustody)
}

fn input<'m, 'a>(
    universe: &ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    stop: &AtomicBool,
) -> ContextResult<(ProjectMapView<'m, 'a>, Budget)> {
    checkpoint(stop)?;
    let view = open_project_map_view(map, stop)?;
    if !view.bound_to(universe) || !std::ptr::eq(graph.owner(), universe.pair.graph()) {
        return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
    }
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
    let partition = graph
        .owner()
        .partition(SOURCE_GRAPH_PARTITION)
        .ok_or_else(missing)?;
    let lookup_work = partition.batch().entity_proposals().len().max(1).ilog2() as usize
        + partition.report().accepted_entities().len().max(1).ilog2() as usize
        + graph.owner().snapshot().nodes().len().max(1).ilog2() as usize
        + (graph.owner().snapshot().limits().max_evidence_per_record as usize)
            * (graph
                .owner()
                .snapshot()
                .limits()
                .max_evidence_per_record
                .max(1)
                .ilog2() as usize
                + 1)
        + 4;
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
fn declaration_node<'a>(row: &Declaration<'a>) -> &'a GraphNode {
    match row {
        Declaration::Callable { node, .. }
        | Declaration::Lua { node, .. }
        | Declaration::Xml { node, .. } => node,
    }
}

pub fn build_l0_skeleton<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    scope: &L0Scope,
    stop: &AtomicBool,
) -> ContextResult<L0Skeleton<'m, 'a>> {
    let (view, mut budget) = input(universe, map, graph, stop)?;
    l0(universe, map, graph, scope, &view, &mut budget, stop)
}
fn l0<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    scope: &L0Scope,
    view: &ProjectMapView<'m, 'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<L0Skeleton<'m, 'a>> {
    let (scope, membership) = match scope {
        L0Scope::Project(id)
            if id == universe.pair.project().configuration().project_id()
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
            if !matches!(node.kind(), "source_file" | "source_package") {
                return Err(invalid());
            }
            (
                Scope::Entity { node },
                "selected_outgoing_native_owns_or_contains",
            )
        }
        _ => return Err(invalid()),
    };
    budget.retain(&L0_PROFILE, stop)?;
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
    let rows = declarations(universe, view, graph, budget, stop)?;
    let mut relations = Vec::new();
    for edge in view.edges() {
        budget.work(1, stop)?;
        if L0_PROFILE.relations.contains(&edge.relation()) {
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
        schema: "wow-context/l0-skeleton/primary/1",
        profile: PRIMARY_L0_PROFILE,
        profile_descriptor: &L0_PROFILE,
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
    l1(universe, map, graph, root, &view, &mut budget, stop)
}
fn l1<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    root: &GraphNodeId,
    view: &ProjectMapView<'m, 'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
) -> ContextResult<L1Skeleton<'m, 'a>> {
    if !view.has_detail_vocabulary() {
        return Err(ContextError::new(ContextErrorCode::SupportMismatch));
    }
    budget.retain(&L1_PROFILE, stop)?;
    let root = view
        .root_node()
        .filter(|node| node.node_id() == root)
        .ok_or_else(invalid)?;
    let rows = declarations(universe, view, graph, budget, stop)?;
    if matches!(
        root.kind(),
        "lua_source_function" | "lua_source_declaration" | "xml_source_declaration"
    ) && !rows
        .iter()
        .any(|row| declaration_node(row).node_id() == root.node_id())
    {
        return Err(missing());
    }
    let core = L1Core {
        schema: "wow-context/l1-skeleton/primary/1",
        profile: PRIMARY_L1_PROFILE,
        profile_descriptor: &L1_PROFILE,
        context: context(universe, map, budget),
        root,
        local_entities: map.native_nodes(),
        direct_relations: map.native_edges(),
        declarations: rows,
        relation_interpretation: "original_direct_native_assertions_no_transitive_edges",
        not_evaluated: NOT_EVALUATED,
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
    let (view, mut budget) = input(universe, map, graph, stop)?;
    let l0 = if matches!(level, SkeletonLevel::L0 | SkeletonLevel::Both) {
        Some(l0(universe, map, graph, root, &view, &mut budget, stop)?)
    } else {
        None
    };
    let l1 = if matches!(level, SkeletonLevel::L1 | SkeletonLevel::Both) {
        let L0Scope::Entity(id) = root else {
            return Err(invalid());
        };
        Some(l1(universe, map, graph, id, &view, &mut budget, stop)?)
    } else {
        None
    };
    let result = PrimarySkeletons {
        schema: "wow-context/primary-skeleton-inspection/1",
        level,
        l0,
        l1,
        work_reserved: budget.work,
        bytes_reserved: budget.retained_bytes,
    };
    crate::budget::encoded_len(&result, universe.limits.max_metadata_bytes, stop)?;
    Ok(result)
}
