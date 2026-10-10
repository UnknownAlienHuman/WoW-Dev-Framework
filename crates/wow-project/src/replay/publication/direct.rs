//! Reconstruct the selected direct layout through actual native predecessors.
use super::{graph_error, invalid};
use crate::graph::{
    PlatformGraphProducer, SOURCE_GRAPH_PARTITION, build_platform_graph_proposal_plan,
};
use crate::{ProjectResult, ProjectView};
use std::sync::atomic::AtomicBool;
use wow_graph::{GraphPartitionReplacement, GraphPartitionSnapshot};

pub(super) fn has_direct_partitions(graph: &GraphPartitionSnapshot) -> bool {
    [
        PlatformGraphProducer::Inventory,
        PlatformGraphProducer::TocLoad,
        PlatformGraphProducer::AnalyzerStructure,
        PlatformGraphProducer::XmlStructure,
    ]
    .into_iter()
    .any(|producer| graph.partition(producer.partition_id()).is_some())
}

pub(super) fn validate(
    project: &ProjectView,
    graph: &GraphPartitionSnapshot,
    stop: &AtomicBool,
) -> ProjectResult<()> {
    crate::analyzer::checkpoint(stop)?;
    let mut plan = build_platform_graph_proposal_plan(project, stop)?;
    if graph.registry() != plan.registry()
        || graph.foundation() != plan.foundation()
        || graph.source_context_id() != plan.scope().source_context_id
        || graph.partition(SOURCE_GRAPH_PARTITION).is_some()
    {
        return Err(invalid());
    }
    let raw_batch = plan.raw_inventory_batch().ok_or_else(invalid)?.clone();
    crate::analyzer::checkpoint(stop)?;
    let mut rebuilt = GraphPartitionSnapshot::new(
        plan.registry().clone(),
        plan.foundation().clone(),
        plan.scope().source_context_id,
        stop,
    )
    .map_err(graph_error)?;
    {
        let prepared = rebuilt
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: rebuilt.snapshot().snapshot_id().clone(),
                    expected_partition_digest: None,
                    producer_version: plan.raw_inventory_producer_version().into(),
                    batch: raw_batch,
                    coverage: Vec::new(),
                },
                stop,
            )
            .map_err(graph_error)?;
        rebuilt = prepared.candidate().clone();
    }
    crate::analyzer::checkpoint(stop)?;
    for &producer in plan.producer_order() {
        crate::analyzer::checkpoint(stop)?;
        let stage = plan.build_stage(producer, &rebuilt, stop)?;
        let producer_version = stage.producer_version();
        let (batch, coverage) = stage.into_parts();
        let prepared = rebuilt
            .prepare_replacement(
                GraphPartitionReplacement {
                    expected_snapshot_id: rebuilt.snapshot().snapshot_id().clone(),
                    expected_partition_digest: None,
                    producer_version: producer_version.into(),
                    batch,
                    coverage,
                },
                stop,
            )
            .map_err(graph_error)?;
        rebuilt = prepared.candidate().clone();
        crate::analyzer::checkpoint(stop)?;
    }
    // Finish owns only this rebuilt raw/direct closure. Independent recognizer
    // partitions may be present in the supplied, natively validated graph.
    let provenance = plan.finish(&rebuilt, stop)?;
    for expected in provenance.graph().partitions() {
        crate::analyzer::checkpoint(stop)?;
        if graph.partition(expected.partition_id()) != Some(expected) {
            return Err(invalid());
        }
    }
    crate::analyzer::checkpoint(stop)
}
