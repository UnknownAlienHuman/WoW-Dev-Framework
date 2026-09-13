use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphError, GraphErrorCode, GraphResult,
    validate_graph_proposal_batch,
};

use super::{
    GraphPartitionChange, GraphPartitionReplacement, GraphPartitionReplacementPlan,
    GraphPartitionSnapshot, GraphProducerPartition, MAX_GRAPH_PRODUCER_PARTITIONS, check_batch,
    check_cancelled, invalid, materialize, rebuild,
};

pub(super) fn prepare(
    base: &GraphPartitionSnapshot,
    mut requests: Vec<GraphPartitionReplacement>,
    cancelled: &AtomicBool,
) -> GraphResult<GraphPartitionReplacementPlan> {
    check_cancelled(cancelled)?;
    if requests.is_empty() {
        return Err(invalid("graph replacement set must not be empty"));
    }
    if requests.len() > MAX_GRAPH_PRODUCER_PARTITIONS {
        return Err(GraphError::new(
            GraphErrorCode::BudgetExceeded,
            "graph replacement set exceeds the partition limit",
        ));
    }
    requests.sort_by(|left, right| {
        left.batch.producer_partition_id().cmp(right.batch.producer_partition_id())
    });
    let replaced = requests.iter()
        .map(|item| item.batch.producer_partition_id())
        .collect::<BTreeSet<_>>();
    if replaced.len() != requests.len() {
        return Err(invalid("graph replacement set names a partition more than once"));
    }
    base.validate(cancelled)?;
    let limits = base.foundation.limits();
    // Validate every compare-and-swap guard against the SAME base before any
    // batch is admitted. Request order must never become implicit rebasing.
    for request in &requests {
        check_cancelled(cancelled)?;
        let previous = base.partition(request.batch.producer_partition_id());
        if &request.expected_snapshot_id != base.snapshot.snapshot_id()
            || previous.map(GraphProducerPartition::partition_digest)
                != request.expected_partition_digest.as_deref()
        {
            return Err(GraphError::new(
                GraphErrorCode::PartitionStale,
                "graph replacement base or previous producer partition is stale",
            ));
        }
        crate::registry::validate_component(&request.producer_version, "producer version")?;
        check_batch(&base.registry, &base.foundation, base.source_context_id, &request.batch)?;
        if request.coverage.len() > limits.max_coverage_records as usize {
            return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "partition coverage budget"));
        }
        for record in &request.coverage {
            check_cancelled(cancelled)?;
            record.validate(limits)?;
            if record.negative_authority() {
                return Err(invalid("producer partition cannot grant graph negative authority"));
            }
        }
    }
    let mut partitions = base.partitions.iter()
        .filter(|item| !replaced.contains(item.partition_id()))
        .cloned()
        .collect::<Vec<_>>();
    drop(replaced);
    if partitions.len() + requests.len() > MAX_GRAPH_PRODUCER_PARTITIONS {
        return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "producer partition limit"));
    }
    // Drop ALL replaced ownership before endpoint resolution. The fixed view
    // prevents a removed peer from satisfying an Existing endpoint, and avoids
    // accepting/rejecting batches based on their order in this transaction.
    let endpoints = materialize::input_view(&base.foundation, &partitions, true, cancelled)?;
    let mut nodes = base.foundation.nodes().len();
    let mut edges = base.foundation.edges().len();
    for item in &partitions {
        nodes = nodes.saturating_add(item.report.accepted_entities().len());
        edges = edges.saturating_add(item.report.accepted_relations().len());
    }
    let mut changes = Vec::with_capacity(requests.len());
    for request in requests {
        check_cancelled(cancelled)?;
        let report = validate_graph_proposal_batch(
            &base.registry, Some(&endpoints), &request.batch, limits,
        )?;
        if !report.ready_for_publication() {
            return Err(GraphError::new(
                GraphErrorCode::PartitionRejected,
                "rejected proposals cannot publish a partial replacement set",
            ));
        }
        nodes = nodes.saturating_add(report.accepted_entities().len());
        edges = edges.saturating_add(report.accepted_relations().len());
        if nodes > limits.max_nodes as usize || edges > limits.max_edges as usize {
            return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "partition assertion budget"));
        }
        let previous = base.partition(request.batch.producer_partition_id());
        let mut coverage = request.coverage;
        if let Some(previous) = previous {
            for old in &previous.coverage {
                check_cancelled(cancelled)?;
                if !coverage.iter().any(|item| item.relation() == old.relation()) {
                    coverage.push(GraphCoverageRecord::new(
                        old.relation(), GraphCoverageState::NotEvaluated, false,
                        vec!["graph.partition.coverage_unreported".into()], limits,
                    )?);
                }
            }
        }
        coverage.sort_by_key(GraphCoverageRecord::relation);
        if coverage.len() > limits.max_coverage_records as usize {
            return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "partition coverage budget"));
        }
        if coverage.windows(2).any(|pair| pair[0].relation() == pair[1].relation()) {
            return Err(invalid("duplicate relation coverage in producer partition"));
        }
        let mut partition = GraphProducerPartition {
            partition_digest: "".into(),
            producer_version: request.producer_version,
            batch: request.batch,
            report,
            coverage,
        };
        partition.partition_digest = partition.derive_digest()?;
        changes.push(GraphPartitionChange {
            partition_id: partition.partition_id().into(),
            previous_partition_digest: request.expected_partition_digest,
            target_partition_digest: partition.partition_digest.clone(),
        });
        partitions.push(partition);
    }
    // Only the complete final membership is checked for dangling survivor edges.
    // Removing their producer in the same transaction is valid; silently pruning
    // an unchanged producer's edges is not.
    let candidate = rebuild(
        base.registry.clone(), base.foundation.clone(), base.source_context_id,
        partitions, cancelled,
    )?;
    check_cancelled(cancelled)?;
    Ok(GraphPartitionReplacementPlan {
        expected_snapshot_id: base.snapshot.snapshot_id().clone(),
        candidate,
        changes,
    })
}
