//! Progressive native orchestration over one held map, graph read and catalog.
use std::sync::atomic::AtomicBool;

use wow_context::{ContextExpansionPlan, ContextExpansionStep, ContextUniverseSet};
use wow_graph::{GraphEvidenceCatalog, GraphNodeId, GraphPartitionRead};

use super::{
    ContextMapHostInput, ContextMapRoot, LiveProjectRead, ServiceErrorCode, ServiceResult,
    checkpoint, context_error, failure, graph_error, with_native_origins_with_catalog,
};

/// Each recursive frame retains its receipts until the final callback returns.
pub(in crate::context) fn with_native_progressive<T>(
    universe: &ContextUniverseSet<'_>,
    read: &LiveProjectRead,
    host: &ContextMapHostInput,
    root: ContextMapRoot,
    rounds: &[Vec<GraphNodeId>],
    stop: &AtomicBool,
    build: impl FnOnce(ContextExpansionStep<'_>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    checkpoint(stop)?;
    if rounds.len() > 4 {
        return Err(failure(ServiceErrorCode::InvalidRequest));
    }
    let (first, remaining) = rounds
        .split_first()
        .ok_or_else(|| failure(ServiceErrorCode::InvalidRequest))?;
    with_native_origins_with_catalog(
        universe,
        read,
        host,
        root,
        true,
        stop,
        |map, graph, catalog| {
            let plan = wow_context::plan_context_progression(universe, map, graph, first, stop)
                .map_err(context_error)?;
            run_round(
                universe,
                (graph, catalog),
                host,
                plan,
                remaining,
                stop,
                build,
            )
        },
    )
}

fn run_round<'a, T>(
    universe: &'a ContextUniverseSet<'a>,
    native: (&GraphPartitionRead<'a>, &'a GraphEvidenceCatalog),
    host: &ContextMapHostInput,
    mut plan: ContextExpansionPlan<'a>,
    remaining: &[Vec<GraphNodeId>],
    stop: &AtomicBool,
    build: impl FnOnce(ContextExpansionStep<'_>) -> ServiceResult<T>,
) -> ServiceResult<T> {
    checkpoint(stop)?;
    let (graph, catalog) = native;
    for index in 0..plan.navigation_queries().len() {
        checkpoint(stop)?;
        let view = graph
            .neighbors(&plan.navigation_queries()[index], stop)
            .map_err(graph_error)?;
        plan.admit_neighborhood(view, stop).map_err(context_error)?;
    }
    let mut prepared = wow_context::prepare_context_expansion(plan, stop).map_err(context_error)?;
    let mut receipts = Vec::with_capacity(prepared.explanation_queries().len());
    for index in 0..prepared.explanation_queries().len() {
        checkpoint(stop)?;
        let receipt = graph
            .explain_with_evidence(
                &prepared.explanation_queries()[index],
                catalog,
                host.limits.evidence,
                stop,
            )
            .map_err(graph_error)?;
        prepared
            .admit_explanation(&receipt, stop)
            .map_err(context_error)?;
        receipts.push(receipt);
    }
    // No receipt moves or appends occur after this borrow. Recursion keeps every
    // earlier frame alive while the final consumer encodes the selected result.
    let step =
        wow_context::expand_context_frontier(prepared, &receipts, stop).map_err(context_error)?;
    checkpoint(stop)?;
    if let Some((next, remaining)) = remaining.split_first() {
        let plan = wow_context::plan_next_context_expansion(universe, step, graph, next, stop)
            .map_err(context_error)?;
        run_round(
            universe,
            (graph, catalog),
            host,
            plan,
            remaining,
            stop,
            build,
        )
    } else {
        build(step)
    }
}
