//! Immutable producer ownership and all-or-nothing graph replacement plans.
//!
//! Proposal endpoints use the exact input-generation view, not the materialized
//! publication IDs. A new input generation requires a new foundation.
pub(crate) mod materialize;
mod storage;
mod transaction;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{GenerationContextId, canonical_json_bytes};

use crate::{
    GraphCoverageRecord, GraphError, GraphErrorCode, GraphGenerationId, GraphNode,
    GraphProposalBatch, GraphProposalValidationReport, GraphRegistryBundle, GraphResult,
    GraphSnapshot, GraphSnapshotId, validate_graph_proposal_batch,
};

pub const GRAPH_PARTITION_SNAPSHOT_SCHEMA: &str = "wow-graph/partition-snapshot/e2-a/1";
pub const GRAPH_PARTITION_SNAPSHOT_SCHEMA_V2: &str = "wow-graph/partition-snapshot/e2-a/2";
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
        digest(
            "graph-partition",
            &(
                &self.producer_version,
                &self.batch,
                &self.report,
                &self.coverage,
            ),
        )
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

/// Graph-owned proof that one exact immutable partition passed full validation.
/// The proof and its results borrow the owner. It has no decoded/caller-supplied
/// constructor, serialization, cache flag or authority over project acquisition.
#[derive(Debug)]
pub struct GraphPartitionRead<'a> {
    owner: &'a GraphPartitionSnapshot,
}

impl<'a> GraphPartitionRead<'a> {
    /// The exact immutable owner whose partition closure was validated.
    #[must_use]
    pub const fn owner(&self) -> &'a GraphPartitionSnapshot {
        self.owner
    }

    /// Resolve one accepted producer entity into its exact published native node.
    /// This read reuses the sealed owner and rebinds only the selected record.
    pub fn published_entity(
        &self,
        partition_id: &str,
        proposal_id: &str,
        stop: &AtomicBool,
    ) -> GraphResult<&'a GraphNode> {
        check_cancelled(stop)?;
        let partition = self
            .owner
            .partition(partition_id)
            .ok_or_else(|| invalid("published entity producer partition is absent"))?;
        partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| invalid("published entity proposal is absent"))?;
        let entries = partition.report().accepted_entities();
        let index = entries
            .binary_search_by(|entry| entry.proposal_id().cmp(proposal_id))
            .map_err(|_| invalid("published entity proposal was not accepted"))?;
        let accepted = entries[index].node();
        let snapshot = self.owner.snapshot();
        let rebound = GraphNode::new(
            snapshot.universe().clone(),
            snapshot.generation().clone(),
            accepted.kind(),
            accepted.owner_key(),
            accepted.evidence_ids().to_vec(),
            snapshot.limits(),
        )?;
        let node = snapshot
            .node(rebound.node_id())
            .ok_or_else(|| invalid("published entity is absent from the final snapshot"))?;
        if node.kind() != accepted.kind() || node.owner_key() != accepted.owner_key() {
            return Err(invalid(
                "published entity differs from its accepted native node",
            ));
        }
        // Materialization legitimately unions support for the same identity.
        // Retain that exact final record, requiring every original receipt item.
        for evidence in accepted.evidence_ids() {
            check_cancelled(stop)?;
            if node.evidence_ids().binary_search(evidence).is_err() {
                return Err(invalid("published entity lost accepted evidence"));
            }
        }
        check_cancelled(stop)?;
        Ok(node)
    }

    /// Reuse admission while retaining every exact-ID/work/output query guard.
    pub fn neighbors(
        &self,
        query: &crate::GraphNeighborReadQuery,
        stop: &AtomicBool,
    ) -> GraphResult<crate::GraphNeighborView<'a>> {
        query.execute_on_read(self, stop)
    }

    /// Original retained support and admitted evidence; no source is reopened.
    pub fn explain_with_evidence(
        &self,
        query: &crate::GraphExplainQuery,
        catalog: &'a crate::GraphEvidenceCatalog,
        limits: crate::GraphEvidenceResolveLimits,
        stop: &AtomicBool,
    ) -> GraphResult<crate::GraphResolvedExplanation<'a>> {
        query.execute_with_evidence_on_read(self, catalog, limits, stop)
    }
}

impl<'a> crate::GraphProducerLookup<'a> {
    /// This sealed lookup already validated this exact owner in input_view.
    /// Reusing its proof does not retain or clone the lookup's input projection.
    pub fn open_read_view(&self, stop: &AtomicBool) -> GraphResult<GraphPartitionRead<'a>> {
        check_cancelled(stop)?;
        Ok(GraphPartitionRead {
            owner: self.owner(),
        })
    }
}

/// Immutable owned proof emitted only after the retained-storage owner has
/// validated its exact partition closure. No raw/decoded constructor, mutable
/// accessor, serialization or cloning can mint or retarget this proof.
#[derive(Debug)]
pub struct ValidatedGraphPartitionSnapshot {
    owner: GraphPartitionSnapshot,
}

impl ValidatedGraphPartitionSnapshot {
    /// Existing graph readers see the identical immutable records.
    #[must_use]
    pub const fn as_snapshot(&self) -> &GraphPartitionSnapshot {
        &self.owner
    }

    /// Consume the proof without copying its owner. The resulting raw snapshot
    /// still requires independent admission for raw query/read entry points.
    #[must_use]
    pub fn into_raw(self) -> GraphPartitionSnapshot {
        self.owner
    }

    /// Borrow the already validated retained owner; no rebuild or graph clone.
    pub fn borrow_read(&self, stop: &AtomicBool) -> GraphResult<GraphPartitionRead<'_>> {
        check_cancelled(stop)?;
        Ok(GraphPartitionRead { owner: &self.owner })
    }
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
    changes: Vec<GraphPartitionChange>,
}

/// One exact before/after ownership change in a validated replacement plan.
/// Entries are ordered by partition ID, including unchanged requests and empty
/// tombstones. Digests bind producer version, raw batch, report and coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphPartitionChange {
    partition_id: Box<str>,
    previous_partition_digest: Option<Box<str>>,
    target_partition_digest: Box<str>,
}

impl GraphPartitionChange {
    #[must_use]
    pub fn partition_id(&self) -> &str {
        &self.partition_id
    }

    #[must_use]
    pub fn previous_partition_digest(&self) -> Option<&str> {
        self.previous_partition_digest.as_deref()
    }

    #[must_use]
    pub fn target_partition_digest(&self) -> &str {
        &self.target_partition_digest
    }
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

    #[must_use]
    pub fn changes(&self) -> &[GraphPartitionChange] {
        &self.changes
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
        rebuild(
            registry,
            foundation,
            source_context_id,
            Vec::new(),
            cancelled,
        )
    }

    #[must_use]
    pub fn snapshot(&self) -> &GraphSnapshot {
        &self.snapshot
    }

    /// Registry used to admit the retained producer batches.
    #[must_use]
    pub fn registry(&self) -> &GraphRegistryBundle {
        &self.registry
    }

    #[must_use]
    pub const fn source_context_id(&self) -> GenerationContextId {
        self.source_context_id
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

    /// Validate once, then borrow this exact partition for a bounded read batch.
    /// Raw/deserialized snapshots receive the same full independent validation;
    /// ordinary query execute methods continue to validate on every raw entry.
    pub fn open_read_view(&self, stop: &AtomicBool) -> GraphResult<GraphPartitionRead<'_>> {
        self.validate(stop)?;
        check_cancelled(stop)?;
        Ok(GraphPartitionRead { owner: self })
    }

    pub fn validate(&self, cancelled: &AtomicBool) -> GraphResult<()> {
        check_cancelled(cancelled)?;
        if self.schema.as_ref() != record_schema(&self.partitions)
            || self.partitions.len() > MAX_GRAPH_PRODUCER_PARTITIONS
        {
            return Err(invalid(
                "graph partition snapshot schema or count is invalid",
            ));
        }
        let rebuilt = rebuild(
            self.registry.clone(),
            self.foundation.clone(),
            self.source_context_id,
            self.partitions.clone(),
            cancelled,
        )?;
        if rebuilt != *self {
            return Err(invalid(
                "graph partition snapshot order, identity, or projection changed",
            ));
        }
        Ok(())
    }

    pub fn prepare_replacement(
        &self,
        request: GraphPartitionReplacement,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphPartitionReplacementPlan> {
        self.prepare_replacements(vec![request], cancelled)
    }

    /// Plans a nonempty, bounded set of independent partition replacements.
    /// All requests name this same base snapshot. Endpoints resolve against
    /// unchanged partitions and the batch's own proposals, never an old removed
    /// partition or another replacement's intermediate output.
    pub fn prepare_replacements(
        &self,
        requests: Vec<GraphPartitionReplacement>,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphPartitionReplacementPlan> {
        transaction::prepare(self, requests, cancelled)
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
        return Err(GraphError::new(
            GraphErrorCode::BudgetExceeded,
            "producer partition limit",
        ));
    }
    partitions.sort_by(|left, right| left.partition_id().cmp(right.partition_id()));
    if partitions
        .windows(2)
        .any(|pair| pair[0].partition_id() == pair[1].partition_id())
    {
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
            || item
                .coverage
                .windows(2)
                .any(|pair| pair[0].relation() >= pair[1].relation())
        {
            return Err(invalid(
                "invalid producer partition report, coverage, or identity",
            ));
        }
        for record in &item.coverage {
            record.validate(limits)?;
            if record.negative_authority() {
                return Err(invalid(
                    "producer partition cannot grant graph negative authority",
                ));
            }
        }
        nodes = nodes.saturating_add(item.report.accepted_entities().len());
        edges = edges.saturating_add(item.report.accepted_relations().len());
        if nodes > limits.max_nodes as usize || edges > limits.max_edges as usize {
            return Err(GraphError::new(
                GraphErrorCode::BudgetExceeded,
                "partition assertion budget",
            ));
        }
    }
    let input = materialize::input_view(&foundation, &partitions, false, cancelled)?;
    for item in &partitions {
        check_cancelled(cancelled)?;
        let verified = validate_graph_proposal_batch(&registry, Some(&input), &item.batch, limits)?;
        if verified != item.report {
            return Err(invalid(
                "producer report does not match its exact batch and endpoints",
            ));
        }
    }
    crate::assertion_validation::validate(&partitions, cancelled)?;
    let schema = record_schema(&partitions);
    let generation = GraphGenerationId::new(digest(
        "graph-generation",
        &(
            schema,
            registry.registry_digest(),
            source_context_id,
            foundation.snapshot_id(),
            &partitions,
        ),
    )?)?;
    let snapshot = materialize::rebind(&input, generation, cancelled)?;
    Ok(GraphPartitionSnapshot {
        schema: schema.into(),
        registry,
        source_context_id,
        foundation,
        partitions,
        snapshot,
    })
}

fn record_schema(partitions: &[GraphProducerPartition]) -> &'static str {
    if partitions
        .iter()
        .any(|partition| partition.batch().assertion_records().is_some())
    {
        GRAPH_PARTITION_SNAPSHOT_SCHEMA_V2
    } else {
        GRAPH_PARTITION_SNAPSHOT_SCHEMA
    }
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
        return Err(GraphError::new(
            GraphErrorCode::RegistryIdentityMismatch,
            "partition registry mismatch",
        ));
    }
    if batch.generation() != foundation.generation()
        || batch.source_context_id() != source_context_id
    {
        return Err(GraphError::new(
            GraphErrorCode::GenerationMismatch,
            "partition input generation mismatch",
        ));
    }
    if batch.universe() != foundation.universe() {
        return Err(GraphError::new(
            GraphErrorCode::UniverseMismatch,
            "partition universe mismatch",
        ));
    }
    Ok(())
}

pub(crate) fn check_cancelled(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Relaxed) {
        Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "graph replacement cancelled before publication",
        ))
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
