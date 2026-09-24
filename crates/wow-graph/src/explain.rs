//! Exact retained-support explanations over a validated partition snapshot.
//! Results borrow their immutable owner: evidence handles are not dereferenced,
//! and missing conflict/derivation records are never manufactured.
mod collect;

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{GenerationContextId, canonical_json_bytes};

use crate::{
    GraphAcceptedEntityProposal, GraphAcceptedRelationProposal, GraphCoverageRecord, GraphEdge,
    GraphEdgeId, GraphEntityKindDefinition, GraphEntityProposal, GraphError, GraphErrorCode,
    GraphGenerationId, GraphNode, GraphNodeId, GraphPartitionSnapshot, GraphRelationKindDefinition,
    GraphRelationProposal, GraphResult, GraphSnapshotId, GraphUniverseId,
};

pub const GRAPH_EXPLANATION_SCHEMA: &str = "wow-graph/retained-explanation/e2-a/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "id",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum GraphExplainSubject {
    Entity(GraphNodeId),
    Relation(GraphEdgeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphExplainLimits {
    /// Accepted records examined in producer reports. Full owner validation is
    /// separately bounded by the existing snapshot/partition profiles.
    pub max_scanned_assertions: u32,
    pub max_supports: u32,
    pub max_output_bytes: u32,
}
impl Default for GraphExplainLimits {
    fn default() -> Self {
        Self {
            max_scanned_assertions: 500_000,
            max_supports: 128,
            max_output_bytes: 1_048_576,
        }
    }
}
impl GraphExplainLimits {
    pub fn validate(self) -> GraphResult<()> {
        if !(1..=4_000_000).contains(&self.max_scanned_assertions)
            || !(1..=4096).contains(&self.max_supports)
            || !(16_384..=8_388_608).contains(&self.max_output_bytes)
        {
            return Err(invalid(
                "explanation limits are outside the bounded profile",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphExplainQuery {
    snapshot_id: GraphSnapshotId,
    subject: GraphExplainSubject,
    limits: GraphExplainLimits,
}
impl GraphExplainQuery {
    pub fn new(
        snapshot_id: GraphSnapshotId,
        subject: GraphExplainSubject,
        limits: GraphExplainLimits,
    ) -> GraphResult<Self> {
        let query = Self {
            snapshot_id,
            subject,
            limits,
        };
        query.validate()?;
        Ok(query)
    }
    pub fn validate(&self) -> GraphResult<()> {
        self.limits.validate()?;
        GraphSnapshotId::new(self.snapshot_id.as_str())?;
        match &self.subject {
            GraphExplainSubject::Entity(id) => {
                GraphNodeId::new(id.as_str())?;
            }
            GraphExplainSubject::Relation(id) => {
                GraphEdgeId::new(id.as_str())?;
            }
        }
        Ok(())
    }
    #[must_use]
    pub fn snapshot_id(&self) -> &GraphSnapshotId {
        &self.snapshot_id
    }
    #[must_use]
    pub fn subject(&self) -> &GraphExplainSubject {
        &self.subject
    }
    #[must_use]
    pub const fn limits(&self) -> GraphExplainLimits {
        self.limits
    }

    /// Explain the exact materialized node/edge using retained input-generation
    /// proposals. Naming a Candidate record explicitly inspects it without
    /// promoting it or hiding lower-confidence support. No storage is opened.
    pub fn execute<'a>(
        &self,
        owner: &'a GraphPartitionSnapshot,
        cancelled: &AtomicBool,
    ) -> GraphResult<GraphExplanation<'a>> {
        checkpoint(cancelled)?;
        self.validate()?;
        if owner.snapshot().snapshot_id() != &self.snapshot_id {
            return Err(GraphError::new(
                GraphErrorCode::SnapshotIdentityMismatch,
                "explanation query names another snapshot",
            ));
        }
        // Reject an impossible scan before the comparatively expensive owner
        // rebuild. Deserialized owners still receive full independent validation.
        if owner.partitions().len() > crate::MAX_GRAPH_PRODUCER_PARTITIONS {
            return Err(budget(
                "explanation producer count exceeds the snapshot profile",
            ));
        }
        let mut count = 0usize;
        for partition in owner.partitions() {
            checkpoint(cancelled)?;
            let records = match &self.subject {
                GraphExplainSubject::Entity(_) => partition.report().accepted_entities().len(),
                GraphExplainSubject::Relation(_) => partition.report().accepted_relations().len(),
            };
            count = count
                .checked_add(records)
                .ok_or_else(|| budget("explanation scan overflow"))?;
            if count > self.limits.max_scanned_assertions as usize {
                return Err(budget(
                    "explanation exceeds its accepted-assertion scan budget",
                ));
            }
        }
        owner.validate(cancelled)?;
        checkpoint(cancelled)?;
        collect::execute(self, owner, cancelled)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GraphExplainedRecord<'a> {
    Entity {
        node: &'a GraphNode,
    },
    Relation {
        edge: &'a GraphEdge,
        source: &'a GraphNode,
        target: &'a GraphNode,
    },
}

/// Composite address of a retained producer assertion. Proposal IDs are local
/// to their batch; none is presented as a globally unique assertion ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphProducerSupportOrigin<'a> {
    pub partition_id: &'a str,
    pub partition_digest: &'a str,
    pub producer_version: &'a str,
    pub batch_id: &'a str,
    pub report_id: &'a str,
}

/// Every item is complete or omitted as a unit, including evidence, source and
/// coverage handles. Accepted records use the foundation's input generation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "origin", rename_all = "snake_case")]
pub enum GraphAssertionSupport<'a> {
    FoundationEntity {
        node: &'a GraphNode,
    },
    FoundationRelation {
        edge: &'a GraphEdge,
    },
    ProducerEntity {
        producer: GraphProducerSupportOrigin<'a>,
        proposal: &'a GraphEntityProposal,
        accepted: &'a GraphAcceptedEntityProposal,
        definition: &'a GraphEntityKindDefinition,
    },
    ProducerRelation {
        producer: GraphProducerSupportOrigin<'a>,
        proposal: &'a GraphRelationProposal,
        accepted: &'a GraphAcceptedRelationProposal,
        definition: &'a GraphRelationKindDefinition,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphExplanationRegistry<'a> {
    pub bundle_id: &'a str,
    pub version: &'a str,
    pub digest: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "layer", rename_all = "snake_case")]
pub enum GraphCoverageOrigin<'a> {
    Projection {
        snapshot_id: &'a GraphSnapshotId,
    },
    Foundation {
        snapshot_id: &'a GraphSnapshotId,
    },
    Producer {
        partition_id: &'a str,
        partition_digest: &'a str,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", content = "record", rename_all = "snake_case")]
pub enum GraphCoverageObservation<'a> {
    Retained(&'a GraphCoverageRecord),
    Missing,
}

/// For a relation, includes every partition, even one with no matching assertion:
/// its missing or failed coverage must not disappear from the explanation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphExplanationCoverage<'a> {
    pub origin: GraphCoverageOrigin<'a>,
    pub observation: GraphCoverageObservation<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphExplanationBoundary {
    EvidenceRecordsNotResolved,
    DerivationRecordsNotRetained,
    ConflictAssessmentNotAvailable,
    FoundationProducerNotRetained,
    EntityCoverageNotModeled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphExplanationTruncation {
    Supports,
    OutputBytes,
}

/// A read view, not a publishable assertion or graph snapshot. `support_complete`
/// concerns retained contributor enumeration only; inspect `boundaries` as well.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GraphExplanation<'a> {
    schema: &'static str,
    query: GraphExplainQuery,
    query_digest: Box<str>,
    universe: &'a GraphUniverseId,
    generation: &'a GraphGenerationId,
    foundation_snapshot_id: &'a GraphSnapshotId,
    source_context_id: GenerationContextId,
    registry: GraphExplanationRegistry<'a>,
    record: GraphExplainedRecord<'a>,
    supports: Vec<GraphAssertionSupport<'a>>,
    coverage: Vec<GraphExplanationCoverage<'a>>,
    scanned_assertions: u32,
    total_supports: u32,
    support_complete: bool,
    truncations: Vec<GraphExplanationTruncation>,
    boundaries: Vec<GraphExplanationBoundary>,
    absence_authoritative: bool,
}
impl<'a> GraphExplanation<'a> {
    #[must_use]
    pub fn query(&self) -> &GraphExplainQuery {
        &self.query
    }
    #[must_use]
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }
    #[must_use]
    pub fn record(&self) -> &GraphExplainedRecord<'a> {
        &self.record
    }
    #[must_use]
    pub fn registry(&self) -> &GraphExplanationRegistry<'a> {
        &self.registry
    }
    #[must_use]
    pub fn supports(&self) -> &[GraphAssertionSupport<'a>] {
        &self.supports
    }
    #[must_use]
    pub fn coverage(&self) -> &[GraphExplanationCoverage<'a>] {
        &self.coverage
    }
    #[must_use]
    pub const fn scanned_assertions(&self) -> u32 {
        self.scanned_assertions
    }
    #[must_use]
    pub const fn total_supports(&self) -> u32 {
        self.total_supports
    }
    #[must_use]
    pub const fn support_complete(&self) -> bool {
        self.support_complete
    }
    #[must_use]
    pub fn truncations(&self) -> &[GraphExplanationTruncation] {
        &self.truncations
    }
    #[must_use]
    pub fn boundaries(&self) -> &[GraphExplanationBoundary] {
        &self.boundaries
    }
    #[must_use]
    pub const fn absence_authoritative(&self) -> bool {
        self.absence_authoritative
    }
}

fn checkpoint(cancelled: &AtomicBool) -> GraphResult<()> {
    if cancelled.load(Ordering::Acquire) {
        return Err(GraphError::new(
            GraphErrorCode::Cancelled,
            "graph explanation cancelled",
        ));
    }
    Ok(())
}
fn invalid(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::QueryInvalid, message)
}
fn budget(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::BudgetExceeded, message)
}
fn encoded<T: Serialize>(value: &T) -> GraphResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(|_| invalid("graph explanation serialization failed"))
}
fn query_digest(query: &GraphExplainQuery) -> GraphResult<Box<str>> {
    let hash = Sha256::digest(encoded(&(GRAPH_EXPLANATION_SCHEMA, query))?);
    let mut text = String::from("graph-explanation-query:sha256:");
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in hash {
        text.push(char::from(HEX[usize::from(byte >> 4)]));
        text.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(text.into_boxed_str())
}
