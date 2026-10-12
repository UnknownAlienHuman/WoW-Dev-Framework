//! Explicit finite next batches over genuine retained native endpoints.
//! Prior artifacts remain immutable; the operation moves its actual ledger.
use super::*;

pub const PRIMARY_PROGRESSIVE_EXPANSION_PROFILE: &str = "wow-context/primary-native-progressive/1";
pub(super) const MAX_ROUNDS: u8 = 4;
pub(super) const PROGRESSIVE_PROFILE: ExpansionProfile = ExpansionProfile {
    profile_id: PRIMARY_PROGRESSIVE_EXPANSION_PROFILE,
    rounds: MAX_ROUNDS,
    max_depth: MAX_ROUNDS + 1,
    selection: "explicit_retained_native_endpoints_excluding_initial_and_queried_roots",
    stopping: "explicit_batch_or_progressive_round_profile_bound",
    ..PROFILE
};

#[derive(Serialize)]
pub(super) struct PreviousNavigation {
    pub(super) map_id: Box<str>,
    pub(super) state: ProjectMapState,
    pub(super) omitted_edges: u64,
}

#[derive(Serialize)]
pub(super) struct Progression<'a> {
    pub(super) round: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) previous: Option<Box<ContextExpansionArtifact<'a>>>,
}
impl Progression<'_> {
    pub(super) fn queried_root_count(&self) -> ContextResult<usize> {
        let mut count = 0usize;
        let mut previous = self.previous.as_deref();
        while let Some(artifact) = previous {
            count = count
                .checked_add(artifact.core.frontier.len())
                .ok_or_else(exhausted)?;
            previous = artifact
                .core
                .progression
                .as_ref()
                .and_then(|progression| progression.previous.as_deref());
        }
        Ok(count)
    }
    pub(super) fn was_queried(&self, id: &GraphNodeId, stop: &AtomicBool) -> ContextResult<bool> {
        let mut previous = self.previous.as_deref();
        while let Some(artifact) = previous {
            checkpoint(stop)?;
            if artifact
                .core
                .frontier
                .iter()
                .any(|node| node.node_id() == id)
            {
                return Ok(true);
            }
            previous = artifact
                .core
                .progression
                .as_ref()
                .and_then(|progression| progression.previous.as_deref());
        }
        Ok(false)
    }
}

/// Start a progressive operation from the genuine immutable detail map.
pub fn plan_context_progression<'a>(
    universe: &'a ContextUniverseSet<'a>,
    initial: &'a ProjectMap<'a>,
    graph: &GraphPartitionRead<'a>,
    frontier: &[GraphNodeId],
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPlan<'a>> {
    plan_initial_expansion(
        universe,
        initial,
        graph,
        frontier,
        Some(Progression {
            round: 1,
            previous: None,
        }),
        stop,
    )
}

/// Consume a genuine prior step once, preserving its original artifacts and
/// continuing Budget. No decoded view or scalar allowance can create this plan.
pub fn plan_next_context_expansion<'a>(
    universe: &'a ContextUniverseSet<'a>,
    previous: ContextExpansionStep<'a>,
    graph: &GraphPartitionRead<'a>,
    frontier: &[GraphNodeId],
    stop: &AtomicBool,
) -> ContextResult<ContextExpansionPlan<'a>> {
    checkpoint(stop)?;
    let view = open_project_map_view(&previous.core.origins, stop)?;
    if previous.expansion_profile_id() != PRIMARY_PROGRESSIVE_EXPANSION_PROFILE
        || !view.bound_to(universe)
        || !std::ptr::eq(graph.owner(), universe.pair.graph())
        || previous.core.universe_set_id != universe.universe_set_id()
        || previous.core.budget.max_context_work != universe.limits.max_context_work
        || previous.core.budget.max_metadata_bytes != universe.limits.max_metadata_bytes
    {
        return Err(support_mismatch());
    }
    let round = previous
        .core
        .progression
        .as_ref()
        .ok_or_else(support_mismatch)?
        .round
        .checked_add(1)
        .filter(|round| *round <= MAX_ROUNDS)
        .ok_or_else(root_invalid)?;
    let MapNavigation::Expansion(navigation) = &previous.core.origins.core.navigation else {
        return Err(support_mismatch());
    };
    let initial = navigation.initial;
    let root = navigation.root;
    if !navigation.is_progressive() || initial.project_map_id() != previous.core.initial_map_id {
        return Err(support_mismatch());
    }
    let ContextExpansionStep {
        expansion_id,
        core,
        skeletons,
        canonical_bytes,
        budget,
    } = previous;
    let artifact = ContextExpansionArtifact {
        expansion_id,
        core,
        canonical_bytes,
    };
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
            progression: Some(Progression {
                round,
                previous: Some(Box::new(artifact)),
            }),
        },
        graph,
        frontier,
        stop,
    )
}
