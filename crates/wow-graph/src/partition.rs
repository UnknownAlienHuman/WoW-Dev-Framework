//! Immutable producer ownership and all-or-nothing graph replacement plans.
//!
//! Proposal endpoints use the exact input-generation view, not the materialized
//! publication IDs. A new input generation requires a new foundation.
mod materialize;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{GenerationContextId, canonical_json_bytes};

use crate::{
    GraphCoverageRecord, GraphCoverageState, GraphError, GraphErrorCode, GraphGenerationId,
    GraphProposalBatch, GraphProposalValidationReport, GraphRegistryBundle, GraphResult,
    GraphSnapshot, GraphSnapshotId, validate_graph_proposal_batch,
};

pub const GRAPH_PARTITION_SNAPSHOT_SCHEMA: &str = "wow-graph/partition-snapshot/e2-a/1";
pub const MAX_GRAPH_PRODUCER_PARTITIONS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphProducerPartition {
    partition_digest: Box<str>,
    producer_version: Box<str>,
    batch: GraphProposalBatch,
    report: GraphProposalValidationReport,
    coverage: Vec<GraphCoverageRecord>,
}

impl GraphProducerPartition {
    #[must_use]
    pub fn partition_id(&self) -> &str {
        self.batch.producer_partition_id()
    }

    #[must_use]
    pub fn partition_digest(&self) -> &str {
        &self.partition_digest
    }

    #[must_use]
    pub fn producer_version(&self) -> &str {
        &self.producer_version
    }

    #[must_use]
    pub fn batch(&self) -> &GraphProposalBatch {
        &self.batch
    }

    #[must_use]
    pub fn report(&self) -> &GraphProposalValidationReport {
        &self.report
    }

    #[must_use]
    pub fn coverage(&self) -> &[GraphCoverageRecord] {
        &self.coverage
    }

    fn derive_digest(&self) -> GraphResult<Box<str>> {
        digest("graph-partition", &(&self.producer_version, &self.batch, &self.report, &self.coverage))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphPartitionSnapshot {
    schema: Box<str>,
    registry: GraphRegistryBundle,
    source_context_id: GenerationContextId,
    foundation: GraphSnapshot,
    partitions: Vec<GraphProducerPartition>,
    snapshot: GraphSnapshot,
}

/// One partition is replaced, never merged with its previous output. `None`
/// means the partition must be absent; `Some` binds its exact previous digest,
/// including the previous producer version. Empty batches retain a tombstone.
#[derive(Debug, Clone)]
pub struct GraphPartitionReplacement {
    pub expected_snapshot_id: GraphSnapshotId,
    pub expected_partition_digest: Option<Box<str>>,
    pub producer_version: Box<str>,
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
}

/// Constructible only by successful owner validation. No caller-supplied or
/// deserialized candidate can bypass validation at the store boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPartitionReplacementPlan {
    expected_snapshot_id: GraphSnapshotId,
    candidate: GraphPartitionSnapshot,
}

impl GraphPartitionReplacementPlan {
    #[must_use]
    pub fn expected_snapshot_id(&self) -> &GraphSnapshotId {
        &self.expected_snapshot_id
    }

    #[must_use]
    pub fn candidate(&self) -> &GraphPartitionSnapshot {
        &self.candidate
    }
}

impl GraphPartitionSnapshot {
    /// Starts an ownership stream from one exact, immutable source-owner graph.
    /// Registry and source-context changes require another explicit stream.
    pub fn new(
        registry: GraphRegistryBundle,
        foundation: GraphSnapshot,
        source_context_id: GenerationContextId,
        cancelled: &AtomicBool,
    ) -> GraphResult<Self> {
        rebuild(registry, foundation, source_context_id, Vec::new(), cancelled)
    }

    #[must_use]
    pub fn snapshot(&self) -> &GraphSnapshot {
        &self.snapshot
    }

    #[must_use]
    pub fn foundation(&self) -> &GraphSnapshot {
        &self.foundation
    }

    #[must_use]
    pub fn partitions(&self) -> &[GraphProducerPartition] {
        &self.partitions
    }

    #[must_use]
    pub fn partition(&self, partition_id: &str) -> Option<&GraphProducerPartition> {
        self.partitions
            .binary_search_by(|item| item.partition_id().cmp(partition_id))
            .ok()
            .map(|index| &self.partitions[index])
    }

    /// Exact endpoint IDs for proposals in this source generation. Publication
    /// snapshots use separate content-derived generation and node identities.
    pub fn input_view(&self, cancelled: &AtomicBool) -> GraphResult<GraphSnapshot> {
        self.validate(cancelled)?;
        materialize::input_view(&self.foundation, &self.partitions, false, cancelled)
    }

    pub fn validate(&self, cancelled: &AtomicBool) -> GraphResult<()> {
        check_cancelled(cancelled)?;
        if self.schema.as_ref() != GRAPH_PARTITION_SNAPSHOT_SCHEMA
            || self.partitions.len() > MAX_GRAPH_PRODUCER_PARTITIONS
        {
            return Err(invalid("graph partition snapshot schema or count is invalid"));
        }
        let rebuilt = rebuild(
            self.registry.clone(),
            self.foundation.clone(),
            self.source_context_id,
            self.partitions.clone(),
            cancelled,
        )?;
        if rebuilt != *self {
            return Err(invalid("graph partition snapshot order, identity, or projection changed"));
        }
        Ok(())
    }

    pub fn prepare_replacement(
        &self,
        request: GraphPartitionReplacement,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphPartitionReplacementPlan> {
        self.validate(cancelled)?;
        let previous = self.partition(request.batch.producer_partition_id());
        if &request.expected_snapshot_id != self.snapshot.snapshot_id()
            || previous.map(GraphProducerPartition::partition_digest)
                != request.expected_partition_digest.as_deref()
        {
            return Err(GraphError::new(
                GraphErrorCode::PartitionStale,
                "graph replacement base or previous producer partition is stale",
            ));
        }
        crate::registry::validate_component(&request.producer_version, "producer version")?;
        check_batch(&self.registry, &self.foundation, self.source_context_id, &request.batch)?;
        let mut partitions = self.partitions
            .iter()
            .filter(|item| item.partition_id() != request.batch.producer_partition_id())
            .cloned()
            .collect::<Vec<_>>();
        if partitions.len() >= MAX_GRAPH_PRODUCER_PARTITIONS {
            return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "producer partition limit"));
        }
        // Do not let removed, exclusively owned nodes satisfy new endpoints.
        // Surviving edges are checked only after the replacement is assembled:
        // the new batch may legitimately restore an endpoint with the same key.
        let endpoints = materialize::input_view(&self.foundation, &partitions, true, cancelled)?;
        let report = validate_graph_proposal_batch(
            &self.registry, Some(&endpoints), &request.batch, self.foundation.limits(),
        )?;
        if !report.ready_for_publication() {
            return Err(GraphError::new(
                GraphErrorCode::PartitionRejected,
                "rejected proposals cannot publish a partial producer partition",
            ));
        }
        let mut coverage = request.coverage;
        // Omitting a previously declared relation is coverage loss, not erasure
        // of the fact that this producer participated in that relation.
        if let Some(previous) = previous {
            for old in &previous.coverage {
                if !coverage.iter().any(|item| item.relation() == old.relation()) {
                    coverage.push(GraphCoverageRecord::new(
                        old.relation(), GraphCoverageState::NotEvaluated, false,
                        vec!["graph.partition.coverage_unreported".into()], self.foundation.limits(),
                    )?);
                }
            }
        }
        coverage.sort_by_key(GraphCoverageRecord::relation);
        let mut partition = GraphProducerPartition {
            partition_digest: "".into(), producer_version: request.producer_version,
            batch: request.batch, report, coverage,
        };
        partition.partition_digest = partition.derive_digest()?;
        partitions.push(partition);
        let candidate = rebuild(
            self.registry.clone(), self.foundation.clone(), self.source_context_id,
            partitions, cancelled,
        )?;
        check_cancelled(cancelled)?;
        Ok(GraphPartitionReplacementPlan {
            expected_snapshot_id: request.expected_snapshot_id, candidate,
        })
    }
}

fn rebuild(
    registry: GraphRegistryBundle,
    foundation: GraphSnapshot,
    source_context_id: GenerationContextId,
    mut partitions: Vec<GraphProducerPartition>,
    cancelled: &AtomicBool,
) -> GraphResult<GraphPartitionSnapshot> {
    check_cancelled(cancelled)?;
    registry.validate()?;
    foundation.validate()?;
    if partitions.len() > MAX_GRAPH_PRODUCER_PARTITIONS {
        return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "producer partition limit"));
    }
    partitions.sort_by(|left, right| left.partition_id().cmp(right.partition_id()));
    if partitions.windows(2).any(|pair| pair[0].partition_id() == pair[1].partition_id()) {
        return Err(invalid("duplicate producer partition"));
    }
    let limits = foundation.limits();
    let mut nodes = foundation.nodes().len();
    let mut edges = foundation.edges().len();
    for item in &partitions {
        check_cancelled(cancelled)?;
        check_batch(&registry, &foundation, source_context_id, &item.batch)?;
        crate::registry::validate_component(&item.producer_version, "producer version")?;
        item.report.validate(limits)?;
        if !item.report.ready_for_publication()
            || item.report.batch_id() != item.batch.batch_id()
            || item.partition_digest != item.derive_digest()?
            || item.coverage.len() > limits.max_coverage_records as usize
            || item.coverage.windows(2).any(|pair| pair[0].relation() >= pair[1].relation())
        {
            return Err(invalid("invalid producer partition report, coverage, or identity"));
        }
        for record in &item.coverage {
            record.validate(limits)?;
            if record.negative_authority() {
                return Err(invalid("producer partition cannot grant graph negative authority"));
            }
        }
        nodes = nodes.saturating_add(item.report.accepted_entities().len());
        edges = edges.saturating_add(item.report.accepted_relations().len());
        if nodes > limits.max_nodes as usize || edges > limits.max_edges as usize {
            return Err(GraphError::new(GraphErrorCode::BudgetExceeded, "partition assertion budget"));
        }
    }
    let input = materialize::input_view(&foundation, &partitions, false, cancelled)?;
    for item in &partitions {
        check_cancelled(cancelled)?;
        let verified = validate_graph_proposal_batch(&registry, Some(&input), &item.batch, limits)?;
        if verified != item.report {
            return Err(invalid("producer report does not match its exact batch and endpoints"));
        }
    }
    let generation = GraphGenerationId::new(digest(
        "graph-generation",
        &(GRAPH_PARTITION_SNAPSHOT_SCHEMA, registry.registry_digest(),
            source_context_id, foundation.snapshot_id(), &partitions),
    )?)?;
    let snapshot = materialize::rebind(&input, generation, cancelled)?;
    Ok(GraphPartitionSnapshot {
        schema: GRAPH_PARTITION_SNAPSHOT_SCHEMA.into(), registry, source_context_id,
        foundation, partitions, snapshot,
    })
}

fn check_batch(
    registry: &GraphRegistryBundle,
    foundation: &GraphSnapshot,
    source_context_id: GenerationContextId,
    batch: &GraphProposalBatch,
) -> GraphResult<()> {
    batch.validate()?;
    if batch.registry_digest() != registry.registry_digest()
        || batch.registry_bundle_id() != registry.bundle_id()
    {
        return Err(GraphError::new(GraphErrorCode::RegistryIdentityMismatch, "partition registry mismatch"));
    }
    if batch.generation() != foundation.generation() || batch.source_context_id() != source_context_id {
        return Err(GraphError::new(GraphErrorCode::GenerationMismatch, "partition input generation mismatch"));
    }
    if batch.universe() != foundation.universe() {
        return Err(GraphError::new(GraphErrorCode::UniverseMismatch, "partition universe mismatch"));
    }
    Ok(())
}

pub(crate) fn check_cancelled(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(GraphError::new(GraphErrorCode::Cancelled, "graph replacement cancelled before publication"))
    } else {
        Ok(())
    }
}

fn invalid(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::PartitionInvalid, message)
}

fn digest<T: Serialize>(domain: &str, value: &T) -> GraphResult<Box<str>> {
    let bytes = canonical_json_bytes(&(domain, value))
        .map_err(|_| invalid("graph partition identity cannot be canonicalized"))?;
    let hash = Sha256::digest(bytes);
    let mut output = format!("{domain}:sha256:");
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in hash {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(output.into_boxed_str())
}
