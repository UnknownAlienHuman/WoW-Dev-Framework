//! Continue the existing Primary ReferenceFacts ledger through owner acquisition.
use std::sync::atomic::AtomicBool;
use wow_graph::GraphPartitionRead;

use super::*;
use crate::reference::{ReferenceOriginInput, ReferenceOriginPreparation};

/// One exact selected Primary map/read and its original skeleton construction
/// budget. No deserialization, public fields, second map or replacement ledger.
pub struct PrimaryReferencePreparation<'m, 'a> {
    pub(crate) universe: &'m ContextUniverseSet<'a>,
    pub(crate) map: &'m ProjectMap<'a>,
    pub(crate) graph: &'m GraphPartitionRead<'a>,
    budget: Budget,
    map_node_count: usize,
}

pub fn prepare_primary_reference_facts<'m, 'a>(
    universe: &'m ContextUniverseSet<'a>,
    map: &'m ProjectMap<'a>,
    graph: &'m GraphPartitionRead<'a>,
    stop: &AtomicBool,
) -> ContextResult<PrimaryReferencePreparation<'m, 'a>> {
    let (view, budget) = input(universe, map, graph, stop)?;
    Ok(PrimaryReferencePreparation {
        universe,
        map,
        graph,
        budget,
        map_node_count: view.nodes().len(),
    })
}

impl<'m, 'a> PrimaryReferencePreparation<'m, 'a> {
    pub fn reference_origin_preparation(
        &mut self,
    ) -> (&'m ProjectMap<'a>, ReferenceOriginPreparation<'_>) {
        (
            self.map,
            ReferenceOriginPreparation {
                budget: &mut self.budget,
                map_node_count: self.map_node_count,
            },
        )
    }
}

pub fn build_prepared_primary_reference_skeletons<'m, 'a>(
    preparation: PrimaryReferencePreparation<'m, 'a>,
    root: &L0Scope,
    origins: &[ReferenceOriginInput<'a>],
    stop: &AtomicBool,
) -> ContextResult<PrimarySkeletons<'m, 'a>> {
    build_prepared_with_budget(preparation, root, origins, stop).map(|(result, _)| result)
}

pub(crate) fn build_prepared_with_budget<'m, 'a>(
    preparation: PrimaryReferencePreparation<'m, 'a>,
    root: &L0Scope,
    origins: &[ReferenceOriginInput<'a>],
    stop: &AtomicBool,
) -> ContextResult<(PrimarySkeletons<'m, 'a>, Budget)> {
    let PrimaryReferencePreparation {
        universe,
        map,
        graph,
        mut budget,
        ..
    } = preparation;
    let L0Scope::Entity(id) = root else {
        return Err(invalid());
    };
    let view = validate_input(universe, map, graph, stop)?;
    let artifact = l1(
        universe,
        map,
        graph,
        (id, PrimaryL1Profile::ReferenceFactsV1, Some(origins)),
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
