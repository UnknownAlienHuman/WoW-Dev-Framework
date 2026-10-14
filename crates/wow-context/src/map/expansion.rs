//! One explicit native frontier batch. Initial detail stays immutable; all
//! aggregate origins are original assertions, never inferred transitive edges.
use super::*;
use crate::skeleton::{
    PrimaryL0Profile, PrimarySkeletons, SkeletonLevel, build_primary_skeletons_with_budget,
};
use wow_graph::GraphPartitionRead;

pub const PRIMARY_FRONTIER_EXPANSION_PROFILE: &str = "wow-context/primary-native-frontier/1";
const MAX_FRONTIER: usize = 32;
mod progressive;
pub use progressive::{
    PRIMARY_PROGRESSIVE_EXPANSION_PROFILE, plan_context_progression, plan_next_context_expansion,
};
use progressive::{PROGRESSIVE_PROFILE, PreviousNavigation, Progression};

#[derive(Serialize)]
struct ExpansionProfile {
    profile_id: &'static str,
    rounds: u8,
    max_depth: u8,
    max_frontier: usize,
    relations: &'static [wow_graph::GraphRelationKind],
    direction: &'static str,
    confidence: &'static str,
    ordering: &'static str,
    selection: &'static str,
    stopping: &'static str,
    source_bodies: &'static str,
}
const PROFILE: ExpansionProfile = ExpansionProfile {
    profile_id: PRIMARY_FRONTIER_EXPANSION_PROFILE,
    rounds: 1,
    max_depth: 2,
    max_frontier: MAX_FRONTIER,
    relations: &DETAIL_RELATIONS,
    direction: "both",
    confidence: "include_candidate",
    ordering: "frontier_node_id_then_native_node_id_then_native_edge_id",
    selection: "explicit_initial_detail_members_excluding_initial_root",
    stopping: "one_round_profile_bound",
    source_bodies: "excluded",
};

#[derive(Serialize)]
pub(super) struct ExpansionNavigation<'a> {
    schema: &'static str,
    initial_map_id: &'a str,
    pub(super) root: &'a GraphNode,
    frontier: Vec<&'a GraphNode>,
    neighborhoods: Vec<GraphNeighborView<'a>>,
    #[serde(skip)]
    initial: &'a ProjectMap<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    previous: Option<PreviousNavigation>,
    #[serde(skip)]
    progressive: bool,
}
impl ExpansionNavigation<'_> {
    pub(super) fn truncated(&self) -> bool {
        self.previous
            .as_ref()
            .map_or(self.initial.state(), |previous| previous.state)
            == ProjectMapState::Truncated
            || self
                .neighborhoods
                .iter()
                .any(|view| view.state() == GraphQueryState::Truncated)
    }
    pub(super) fn omitted_edges(&self) -> ContextResult<u64> {
        self.neighborhoods.iter().try_fold(
            self.previous.as_ref().map_or_else(
                || self.initial.core.navigation.omitted_edges(),
                |previous| Ok(previous.omitted_edges),
            )?,
            |n, view| {
                n.checked_add(u64::from(view.omitted_edges()))
                    .ok_or_else(exhausted)
            },
        )
    }
    pub(super) fn is_progressive(&self) -> bool {
        self.progressive
    }
}

/// Sealed plan from one actual initial detail view and graph owner. Queries are
/// already cumulatively reserved before the application executes the batch.
pub struct ContextExpansionPlan<'a> {
    universe: &'a ContextUniverseSet<'a>,
    initial: &'a ProjectMap<'a>,
    skeletons: PrimarySkeletons<'a, 'a>,
    root: &'a GraphNode,
    frontier: Vec<&'a GraphNode>,
    queries: Vec<GraphNeighborReadQuery>,
    neighborhoods: Vec<GraphNeighborView<'a>>,
    budget: Budget,
    progression: Option<Progression<'a>>,
}
impl<'a> ContextExpansionPlan<'a> {
    pub fn navigation_queries(&self) -> &[GraphNeighborReadQuery] {
        &self.queries
    }

    pub fn admit_neighborhood(
        &mut self,
        view: GraphNeighborView<'a>,
        stop: &AtomicBool,
    ) -> ContextResult<()> {
        checkpoint(stop)?;
        let index = self.neighborhoods.len();
        let query = self.queries.get(index).ok_or_else(support_mismatch)?;
        let node = self.frontier.get(index).ok_or_else(support_mismatch)?;
        if view.query() != query || !std::ptr::eq(view.root(), *node) {
            return Err(support_mismatch());
        }
        self.budget.retain(&view, stop)?;
        self.neighborhoods.push(view);
        Ok(())
    }
}

pub fn plan_context_expansion<'a>(
    universe: &'a ContextUniverseSet<'a>,
    initial: &'a ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    frontier: &[GraphNodeId],
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPlan<'a>> {
    plan_initial_expansion(universe, initial, graph, frontier, None, stop)
}

fn plan_initial_expansion<'a>(
    universe: &'a ContextUniverseSet<'a>,
    initial: &'a ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    frontier: &[GraphNodeId],
    progression: Option<Progression<'a>>,
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPlan<'a>> {
    checkpoint(stop)?;
    let view = open_project_map_view(initial, stop)?;
    if !view.bound_to(universe)
        || !std::ptr::eq(graph.owner(), universe.pair.graph())
        || !view.has_detail_vocabulary()
    {
        return Err(support_mismatch());
    }
    if frontier.is_empty() || frontier.len() > MAX_FRONTIER {
        return Err(root_invalid());
    }
    let root = view.root_node().ok_or_else(root_invalid)?;
    let scope = crate::L0Scope::Entity(root.node_id().clone());
    let (skeletons, mut budget) = build_primary_skeletons_with_budget(
        universe,
        initial,
        graph,
        &scope,
        SkeletonLevel::L1,
        PrimaryL0Profile::ContainersV1,
        stop,
    )?;
    budget.retain(&skeletons, stop)?;
    budget.retain(
        if progression.is_some() {
            &PROGRESSIVE_PROFILE
        } else {
            &PROFILE
        },
        stop,
    )?;
    select_frontier(
        ContextExpansionPlan {
            universe,
            initial,
            skeletons,
            root,
            frontier: Vec::new(),
            queries: Vec::new(),
            neighborhoods: Vec::new(),
            budget,
            progression,
        },
        graph,
        frontier,
        stop,
    )
}

fn select_frontier<'a>(
    mut plan: ContextExpansionPlan<'a>,
    graph: &GraphPartitionRead<'a>,
    frontier: &[GraphNodeId],
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPlan<'a>> {
    if frontier.is_empty() || frontier.len() > MAX_FRONTIER {
        return Err(root_invalid());
    }
    let seed = plan
        .progression
        .as_ref()
        .and_then(|progression| progression.previous.as_deref())
        .map_or(plan.initial, |previous| &previous.core.origins);
    let budget = &mut plan.budget;
    budget.work(
        frontier
            .len()
            .checked_mul(seed.nodes.len().max(1).ilog2() as usize + 2)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut selected = Vec::new();
    for id in frontier {
        budget.work(1, stop)?;
        let node = seed
            .nodes
            .binary_search_by(|node| node.node_id().cmp(id))
            .ok()
            .map(|index| seed.nodes[index])
            .filter(|node| node.node_id() != plan.root.node_id())
            .ok_or_else(root_invalid)?;
        if let Some(progression) = &plan.progression {
            budget.work(progression.queried_root_count()?, stop)?;
            if progression.was_queried(id, stop)? {
                return Err(root_invalid());
            }
        }
        if !std::ptr::eq(
            node,
            graph
                .owner()
                .snapshot()
                .node(id)
                .ok_or_else(support_mismatch)?,
        ) {
            return Err(support_mismatch());
        }
        budget.retain(node, stop)?;
        selected.push(node);
    }
    budget.work(
        selected
            .len()
            .checked_mul(selected.len().max(1).ilog2() as usize + 1)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    selected.sort_by(|a, b| a.node_id().cmp(b.node_id()));
    if selected
        .windows(2)
        .any(|pair| pair[0].node_id() == pair[1].node_id())
    {
        return Err(root_invalid());
    }
    // Native neighbors scan the entire edge list for each frontier root.
    // Reserve the complete batch before returning even its first query.
    budget.work(
        selected
            .len()
            .checked_mul(graph.owner().snapshot().edges().len())
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut queries = Vec::new();
    for node in &selected {
        budget.work(1, stop)?;
        let query = navigation_query_for(
            plan.universe,
            node.node_id().clone(),
            &DETAIL_RELATIONS,
            stop,
        )?;
        budget.retain(&query, stop)?;
        queries.push(query);
    }
    plan.frontier = selected;
    plan.queries = queries;
    Ok(plan)
}

pub struct ContextExpansionPreparation<'a> {
    selection: ProjectMapSelection<'a>,
    queries: Vec<GraphExplainQuery>,
    reused: Vec<&'a GraphResolvedExplanation<'a>>,
    initial: &'a ProjectMap<'a>,
    skeletons: PrimarySkeletons<'a, 'a>,
    progression: Option<Progression<'a>>,
}
impl ContextExpansionPreparation<'_> {
    pub fn explanation_queries(&self) -> &[GraphExplainQuery] {
        &self.queries
    }
    pub fn admit_explanation(
        &mut self,
        receipt: &GraphResolvedExplanation<'_>,
        stop: &AtomicBool,
    ) -> ContextResult<()> {
        self.selection.admit_explanation(receipt, stop)
    }
}

pub fn prepare_context_expansion<'a>(
    mut plan: ContextExpansionPlan<'a>,
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPreparation<'a>> {
    checkpoint(stop)?;
    if plan.neighborhoods.len() != plan.queries.len() {
        return Err(support_mismatch());
    }
    let snapshot = plan.universe.pair.graph().snapshot();
    let seed = plan
        .progression
        .as_ref()
        .and_then(|progression| progression.previous.as_deref())
        .map_or(plan.initial, |previous| &previous.core.origins);
    let mut nodes = BTreeMap::new();
    let mut edges = BTreeMap::new();
    let lookup_work = snapshot.nodes().len().max(1).ilog2() as usize
        + snapshot.edges().len().max(1).ilog2() as usize
        + 4;
    let row_count = plan.neighborhoods.iter().try_fold(
        seed.nodes
            .len()
            .checked_add(seed.edges.len())
            .ok_or_else(exhausted)?,
        |count, view| {
            count
                .checked_add(view.adjacent_nodes().len())
                .and_then(|n| n.checked_add(view.edges().len()))
                .and_then(|n| n.checked_add(1))
                .ok_or_else(exhausted)
        },
    )?;
    plan.budget.work(
        row_count.checked_mul(lookup_work).ok_or_else(exhausted)?,
        stop,
    )?;
    for node in &seed.nodes {
        plan.budget.work(1, stop)?;
        nodes.insert(node.node_id(), *node);
    }
    for edge in &seed.edges {
        plan.budget.work(1, stop)?;
        edges.insert(edge.edge_id(), *edge);
    }
    for ((view, query), frontier) in plan
        .neighborhoods
        .iter()
        .zip(&plan.queries)
        .zip(&plan.frontier)
    {
        plan.budget.work(1, stop)?;
        if view.query() != query || !std::ptr::eq(view.root(), *frontier) {
            return Err(support_mismatch());
        }
        for node in std::iter::once(view.root()).chain(view.adjacent_nodes().iter().copied()) {
            plan.budget.work(1, stop)?;
            if !std::ptr::eq(
                node,
                snapshot.node(node.node_id()).ok_or_else(support_mismatch)?,
            ) {
                return Err(support_mismatch());
            }
            plan.budget.retain(node, stop)?;
            nodes.insert(node.node_id(), node);
        }
        for edge in view.edges() {
            plan.budget.work(1, stop)?;
            if !std::ptr::eq(
                *edge,
                snapshot.edge(edge.edge_id()).ok_or_else(support_mismatch)?,
            ) {
                return Err(support_mismatch());
            }
            plan.budget.retain(edge, stop)?;
            edges.insert(edge.edge_id(), *edge);
        }
    }
    let previous = if plan
        .progression
        .as_ref()
        .is_some_and(|progression| progression.previous.is_some())
    {
        plan.budget.retain(seed.project_map_id(), stop)?;
        Some(PreviousNavigation {
            map_id: seed.project_map_id().into(),
            state: seed.state(),
            omitted_edges: seed.core.navigation.omitted_edges()?,
        })
    } else {
        None
    };
    let navigation = MapNavigation::Expansion(ExpansionNavigation {
        schema: if plan.progression.is_some() {
            "wow-context/primary-progressive-navigation/1"
        } else {
            "wow-context/primary-frontier-navigation/1"
        },
        initial_map_id: plan.initial.project_map_id(),
        root: plan.root,
        frontier: plan.frontier,
        neighborhoods: plan.neighborhoods,
        initial: plan.initial,
        previous,
        progressive: plan.progression.is_some(),
    });
    let mut selection = finish_selection(
        plan.universe,
        navigation,
        nodes.into_values().collect(),
        edges.into_values().collect(),
        plan.budget,
        stop,
    )?;
    // Initial custody is immutable and already validated. Reuse only an exact
    // native query match; newly discovered subjects get new owner receipts.
    let lookup_work = seed.core.supports.len().max(1).ilog2() as usize + 2;
    selection.budget.work(
        selection
            .queries
            .len()
            .checked_mul(lookup_work)
            .ok_or_else(exhausted)?,
        stop,
    )?;
    let mut queries = Vec::new();
    let mut reused = Vec::new();
    for index in 0..selection.queries.len() {
        selection.budget.work(1, stop)?;
        let query = &selection.queries[index];
        let existing = seed
            .core
            .supports
            .binary_search_by(|receipt| {
                subject_key(receipt.explanation().query().subject())
                    .cmp(&subject_key(query.subject()))
            })
            .ok()
            .map(|index| seed.core.supports[index]);
        if let Some(receipt) = existing.filter(|receipt| receipt.explanation().query() == query) {
            selection.admit_explanation(receipt, stop)?;
            reused.push(receipt);
        } else {
            selection.budget.retain(query, stop)?;
            queries.push(query.clone());
        }
    }
    Ok(ContextExpansionPreparation {
        selection,
        queries,
        reused,
        initial: plan.initial,
        skeletons: plan.skeletons,
        progression: plan.progression,
    })
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub(crate) struct InclusionReason<'a> {
    kind: &'static str,
    root_id: &'a GraphNodeId,
}
type ReasonIndex<'a> = BTreeMap<(u8, &'a str), Vec<InclusionReason<'a>>>;
fn reason<'a>(
    index: &mut ReasonIndex<'a>,
    key: (u8, &'a str),
    value: InclusionReason<'a>,
    budget: &mut Budget,
    stop: &AtomicBool,
    frontier_bound: usize,
) -> ContextResult<()> {
    budget.work(
        index.len().max(1).ilog2() as usize + frontier_bound + 2,
        stop,
    )?;
    let rows = index.entry(key).or_default();
    if !rows.contains(&value) {
        budget.retain(&value, stop)?;
        rows.push(value);
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum ExpansionItem<'a> {
    Entity {
        node: &'a GraphNode,
        previously_selected: bool,
        reasons: Vec<InclusionReason<'a>>,
    },
    Relation {
        edge: &'a GraphEdge,
        previously_selected: bool,
        reasons: Vec<InclusionReason<'a>>,
    },
}
#[derive(Serialize)]
struct ExpansionBudget {
    max_context_work: u64,
    max_metadata_bytes: u64,
    work_reserved: u64,
    bytes_reserved: u64,
    reservation_basis: &'static str,
    tokens: &'static str,
}
#[derive(Serialize)]
struct ExpansionCore<'a> {
    schema: &'static str,
    profile: &'static ExpansionProfile,
    universe_set_id: &'a str,
    initial_map_id: &'a str,
    initial_l1_id: Box<str>,
    root: &'a GraphNode,
    frontier: Vec<&'a GraphNode>,
    origins: ProjectMap<'a>,
    items: Vec<ExpansionItem<'a>>,
    added_entity_count: usize,
    added_relation_count: usize,
    unselected_initial_frontier_count: usize,
    state: ProjectMapState,
    absence_authoritative: bool,
    stop_reason: &'static str,
    continuation: &'static str,
    not_evaluated: &'static [&'static str],
    budget: ExpansionBudget,
    #[serde(skip_serializing_if = "Option::is_none")]
    progression: Option<Progression<'a>>,
}
#[derive(Serialize)]
pub struct ContextExpansionStep<'a> {
    expansion_id: Box<str>,
    core: ExpansionCore<'a>,
    #[serde(skip)]
    skeletons: PrimarySkeletons<'a, 'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
    #[serde(skip)]
    budget: Budget,
}
impl<'a> ContextExpansionStep<'a> {
    pub fn skeletons(&self) -> &PrimarySkeletons<'a, 'a> {
        &self.skeletons
    }
    pub fn state(&self) -> ProjectMapState {
        self.core.state
    }
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    pub fn expansion_id(&self) -> &str {
        &self.expansion_id
    }
    pub fn expansion_profile_id(&self) -> &'static str {
        self.core.profile.profile_id
    }
    /// Remaining cumulative allowance for the consumer envelope, after the
    /// original artifacts, aggregate rows and canonical expansion are retained.
    pub fn remaining_metadata_bytes(&self) -> u64 {
        self.core
            .budget
            .max_metadata_bytes
            .saturating_sub(self.budget.retained_bytes)
    }
    pub(crate) fn into_pack_parts(
        self,
        universe: &ContextUniverseSet<'a>,
        root: &crate::L0Scope,
        stop: &AtomicBool,
    ) -> ContextResult<(
        ContextExpansionArtifact<'a>,
        PrimarySkeletons<'a, 'a>,
        Budget,
    )> {
        checkpoint(stop)?;
        if !open_project_map_view(&self.core.origins, stop)?.bound_to(universe) {
            return Err(ContextError::new(ContextErrorCode::UniverseMismatch));
        }
        if !matches!(root, crate::L0Scope::Entity(id) if id == self.core.root.node_id()) {
            return Err(ContextError::new(ContextErrorCode::RootInvalid));
        }
        Ok((
            ContextExpansionArtifact {
                expansion_id: self.expansion_id,
                core: self.core,
                canonical_bytes: self.canonical_bytes,
            },
            self.skeletons,
            self.budget,
        ))
    }
}

/// The unchanged immutable native artifact moved from its sealed step. It has
/// no public constructor and never stores references into its own fields.
#[derive(Serialize)]
pub(crate) struct ContextExpansionArtifact<'a> {
    expansion_id: Box<str>,
    core: ExpansionCore<'a>,
    #[serde(skip)]
    canonical_bytes: Box<[u8]>,
}
impl<'a> ContextExpansionArtifact<'a> {
    pub(crate) fn into_step<'m>(
        self,
        skeletons: PrimarySkeletons<'m, 'a>,
        budget: Budget,
    ) -> ContextExpansionStep<'m>
    where
        'a: 'm,
    {
        ContextExpansionStep {
            expansion_id: self.expansion_id,
            core: self.core,
            skeletons,
            canonical_bytes: self.canonical_bytes,
            budget,
        }
    }

    pub(crate) fn expansion_profile_id(&self) -> &'static str {
        self.core.profile.profile_id
    }
    pub(crate) fn expansion_id(&self) -> &str {
        &self.expansion_id
    }
    pub(crate) fn origins(&self) -> &ProjectMap<'a> {
        &self.core.origins
    }
    pub(crate) fn items(&self) -> &[ExpansionItem<'a>] {
        &self.core.items
    }
    pub(crate) fn state(&self) -> ProjectMapState {
        self.core.state
    }
    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    pub(crate) fn progression_round(&self) -> Option<u8> {
        self.core
            .progression
            .as_ref()
            .map(|progression| progression.round)
    }
    pub(crate) fn previous_expansion_ids(&self) -> Vec<Box<str>> {
        let mut result = Vec::new();
        let mut previous = self
            .core
            .progression
            .as_ref()
            .and_then(|progression| progression.previous.as_deref());
        while let Some(artifact) = previous {
            result.push(artifact.expansion_id.clone());
            previous = artifact
                .core
                .progression
                .as_ref()
                .and_then(|progression| progression.previous.as_deref());
        }
        result
    }
}

pub fn expand_context_frontier<'a>(
    prepared: ContextExpansionPreparation<'a>,
    receipts: &'a [GraphResolvedExplanation<'a>],
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionStep<'a>> {
    checkpoint(stop)?;
    let limits = prepared.selection.universe.limits;
    let universe_set_id = prepared.selection.universe.universe_set_id();
    if receipts.len() != prepared.queries.len() {
        return Err(support_mismatch());
    }
    let mut combined = prepared.reused;
    for receipt in receipts {
        checkpoint(stop)?;
        combined.push(receipt);
    }
    let (origins, mut budget) =
        build_project_map_with_budget(prepared.selection, combined.into_iter(), true, stop)?;
    let MapNavigation::Expansion(navigation) = &origins.core.navigation else {
        return Err(support_mismatch());
    };
    let root = navigation.root;
    let frontier_bound = if prepared.progression.is_some() {
        MAX_FRONTIER * usize::from(progressive::MAX_ROUNDS)
    } else {
        MAX_FRONTIER
    };
    let mut reasons = ReasonIndex::new();
    for node in &prepared.initial.nodes {
        reason(
            &mut reasons,
            (0, node.node_id().as_str()),
            InclusionReason {
                kind: "initial_selection",
                root_id: root.node_id(),
            },
            &mut budget,
            stop,
            frontier_bound,
        )?;
    }
    for edge in &prepared.initial.edges {
        reason(
            &mut reasons,
            (1, edge.edge_id().as_str()),
            InclusionReason {
                kind: "initial_selection",
                root_id: root.node_id(),
            },
            &mut budget,
            stop,
            frontier_bound,
        )?;
    }
    if let Some(previous) = prepared
        .progression
        .as_ref()
        .and_then(|progression| progression.previous.as_deref())
    {
        for item in &previous.core.items {
            let (key, rows) = match item {
                ExpansionItem::Entity { node, reasons, .. } => {
                    ((0, node.node_id().as_str()), reasons)
                }
                ExpansionItem::Relation { edge, reasons, .. } => {
                    ((1, edge.edge_id().as_str()), reasons)
                }
            };
            for row in rows {
                reason(
                    &mut reasons,
                    key,
                    row.clone(),
                    &mut budget,
                    stop,
                    frontier_bound,
                )?;
            }
        }
    }
    for view in &navigation.neighborhoods {
        for node in std::iter::once(view.root()).chain(view.adjacent_nodes().iter().copied()) {
            reason(
                &mut reasons,
                (0, node.node_id().as_str()),
                InclusionReason {
                    kind: "frontier_selection",
                    root_id: view.root().node_id(),
                },
                &mut budget,
                stop,
                frontier_bound,
            )?;
        }
        for edge in view.edges() {
            reason(
                &mut reasons,
                (1, edge.edge_id().as_str()),
                InclusionReason {
                    kind: "frontier_selection",
                    root_id: view.root().node_id(),
                },
                &mut budget,
                stop,
                frontier_bound,
            )?;
        }
    }
    let seed = prepared
        .progression
        .as_ref()
        .and_then(|progression| progression.previous.as_deref())
        .map_or(prepared.initial, |previous| &previous.core.origins);
    let initial = open_project_map_view(seed, stop)?;
    let mut items = Vec::new();
    let mut added_entity_count = 0;
    let mut added_relation_count = 0;
    for node in &origins.nodes {
        budget.work(1, stop)?;
        let previously_selected = initial.node(node.node_id()).is_some();
        added_entity_count += usize::from(!previously_selected);
        let item = ExpansionItem::Entity {
            node,
            previously_selected,
            reasons: reasons
                .remove(&(0, node.node_id().as_str()))
                .ok_or_else(support_mismatch)?,
        };
        budget.retain(&item, stop)?;
        items.push(item);
    }
    for edge in &origins.edges {
        budget.work(1, stop)?;
        let previously_selected = initial.edge(edge.edge_id()).is_some();
        added_relation_count += usize::from(!previously_selected);
        let item = ExpansionItem::Relation {
            edge,
            previously_selected,
            reasons: reasons
                .remove(&(1, edge.edge_id().as_str()))
                .ok_or_else(support_mismatch)?,
        };
        budget.retain(&item, stop)?;
        items.push(item);
    }
    if !reasons.is_empty() {
        return Err(support_mismatch());
    }
    budget.retain(&navigation.frontier, stop)?;
    let frontier = navigation.frontier.clone();
    let unselected_initial_frontier_count = if let Some(progression) = &prepared.progression {
        let mut count = 0usize;
        for node in &prepared.initial.nodes {
            budget.work(
                progression
                    .queried_root_count()?
                    .checked_add(frontier.len() + 1)
                    .ok_or_else(exhausted)?,
                stop,
            )?;
            if node.node_id() != root.node_id()
                && !progression.was_queried(node.node_id(), stop)?
                && !frontier
                    .iter()
                    .any(|selected| selected.node_id() == node.node_id())
            {
                count += 1;
            }
        }
        count
    } else {
        initial
            .nodes()
            .len()
            .checked_sub(1)
            .and_then(|n| n.checked_sub(frontier.len()))
            .ok_or_else(support_mismatch)?
    };
    let initial_l1_id = prepared.skeletons.l1_id().ok_or_else(support_mismatch)?;
    budget.retain(initial_l1_id, stop)?;
    let initial_l1_id = initial_l1_id.into();
    let state = origins.state();
    let core = ExpansionCore {
        schema: if prepared.progression.is_some() {
            "wow-context/primary-progressive-expansion/1"
        } else {
            "wow-context/primary-frontier-expansion/1"
        },
        profile: if prepared.progression.is_some() {
            &PROGRESSIVE_PROFILE
        } else {
            &PROFILE
        },
        universe_set_id,
        initial_map_id: prepared.initial.project_map_id(),
        initial_l1_id,
        root,
        frontier,
        origins,
        items,
        added_entity_count,
        added_relation_count,
        unselected_initial_frontier_count,
        state,
        absence_authoritative: false,
        stop_reason: match &prepared.progression {
            Some(progression) if progression.round == progressive::MAX_ROUNDS => {
                "progressive_round_profile_bound"
            }
            Some(_) => "explicit_batch_complete",
            None => "one_round_profile_bound",
        },
        continuation: "not_evaluated",
        not_evaluated: if prepared.progression.is_some() {
            &[
                "unselected_frontier",
                "automatic_selection_and_retained_continuation",
                "reference_enrichment",
                "source_excerpts_and_exact_tokens",
                "complete_corpus_or_runtime_behavior",
            ]
        } else {
            &[
                "unselected_frontier",
                "further_rounds_and_continuation",
                "reference_enrichment",
                "source_excerpts_and_exact_tokens",
                "complete_corpus_or_runtime_behavior",
            ]
        },
        budget: ExpansionBudget {
            max_context_work: limits.max_context_work,
            max_metadata_bytes: limits.max_metadata_bytes,
            work_reserved: budget.work,
            bytes_reserved: budget.retained_bytes,
            reservation_basis: "before_expansion_finalization",
            tokens: "unavailable",
        },
        progression: prepared.progression,
    };
    let expansion_id = identity(
        core.schema,
        "context-expansion:sha256:",
        &core,
        limits.max_metadata_bytes,
        stop,
    )?;
    #[derive(Serialize)]
    struct ArtifactRef<'p, 'a> {
        expansion_id: &'p str,
        core: &'p ExpansionCore<'a>,
    }
    budget.retain(
        &ArtifactRef {
            expansion_id: &expansion_id,
            core: &core,
        },
        stop,
    )?;
    let mut step = ContextExpansionStep {
        expansion_id,
        core,
        skeletons: prepared.skeletons,
        canonical_bytes: Box::new([]),
        budget,
    };
    step.canonical_bytes = canonical(&step, limits.max_metadata_bytes, stop)?.into_boxed_slice();
    Ok(step)
}
