use super::*;
use crate::{GraphProducerPartition, GraphRelationKind, GraphSnapshot};

/// The projection deliberately rebinds node and edge IDs. Reverse that binding
/// with the very same constructors, never by display names or evidence equality.
fn input_node(node: &GraphNode, owner: &GraphPartitionSnapshot) -> GraphResult<GraphNodeId> {
    let original = GraphNode::new(
        owner.foundation().universe().clone(),
        owner.foundation().generation().clone(),
        node.kind(),
        node.owner_key(),
        Vec::new(),
        owner.foundation().limits(),
    )?;
    Ok(original.node_id().clone())
}

fn select<'a>(
    query: &GraphExplainQuery,
    owner: &'a GraphPartitionSnapshot,
) -> GraphResult<(GraphExplainedRecord<'a>, GraphExplainSubject)> {
    let snapshot = owner.snapshot();
    match &query.subject {
        GraphExplainSubject::Entity(id) => {
            let node = snapshot
                .node(id)
                .ok_or_else(|| invalid("explanation entity is not in the selected snapshot"))?;
            Ok((
                GraphExplainedRecord::Entity { node },
                GraphExplainSubject::Entity(input_node(node, owner)?),
            ))
        }
        GraphExplainSubject::Relation(id) => {
            let edge = snapshot
                .edge(id)
                .ok_or_else(|| invalid("explanation relation is not in the selected snapshot"))?;
            let source = snapshot
                .node(edge.from())
                .ok_or_else(|| invalid("explanation source endpoint is missing"))?;
            let target = snapshot
                .node(edge.to())
                .ok_or_else(|| invalid("explanation target endpoint is missing"))?;
            let original = GraphEdge::new(
                input_node(source, owner)?,
                input_node(target, owner)?,
                edge.relation(),
                edge.confidence(),
                edge.evidence_ids().to_vec(),
                owner.foundation().limits(),
            )?;
            Ok((
                GraphExplainedRecord::Relation {
                    edge,
                    source,
                    target,
                },
                GraphExplainSubject::Relation(original.edge_id().clone()),
            ))
        }
    }
}

fn origin(partition: &GraphProducerPartition) -> GraphProducerSupportOrigin<'_> {
    GraphProducerSupportOrigin {
        partition_id: partition.partition_id(),
        partition_digest: partition.partition_digest(),
        producer_version: partition.producer_version(),
        batch_id: partition.batch().batch_id(),
        report_id: partition.report().report_id(),
    }
}

fn observed(record: Option<&GraphCoverageRecord>) -> GraphCoverageObservation<'_> {
    match record {
        Some(record) => GraphCoverageObservation::Retained(record),
        None => GraphCoverageObservation::Missing,
    }
}
fn snapshot_coverage(
    snapshot: &GraphSnapshot,
    relation: GraphRelationKind,
) -> GraphCoverageObservation<'_> {
    observed(snapshot.coverage_for(relation))
}

fn coverage<'a>(
    owner: &'a GraphPartitionSnapshot,
    relation: GraphRelationKind,
    cancelled: &AtomicBool,
) -> GraphResult<Vec<GraphExplanationCoverage<'a>>> {
    let mut records = vec![
        GraphExplanationCoverage {
            origin: GraphCoverageOrigin::Projection {
                snapshot_id: owner.snapshot().snapshot_id(),
            },
            observation: snapshot_coverage(owner.snapshot(), relation),
        },
        GraphExplanationCoverage {
            origin: GraphCoverageOrigin::Foundation {
                snapshot_id: owner.foundation().snapshot_id(),
            },
            observation: snapshot_coverage(owner.foundation(), relation),
        },
    ];
    for partition in owner.partitions() {
        checkpoint(cancelled)?;
        let record = partition
            .coverage()
            .binary_search_by_key(&relation, GraphCoverageRecord::relation)
            .ok()
            .map(|index| &partition.coverage()[index]);
        records.push(GraphExplanationCoverage {
            origin: GraphCoverageOrigin::Producer {
                partition_id: partition.partition_id(),
                partition_digest: partition.partition_digest(),
            },
            observation: observed(record),
        });
    }
    Ok(records)
}

struct Collector<'a> {
    result: GraphExplanation<'a>,
    bytes: usize,
}
impl<'a> Collector<'a> {
    fn new(result: GraphExplanation<'a>) -> GraphResult<Self> {
        // Leave room for final counters, truncation flags and the foundation
        // provenance boundary. Item encoding includes all payload/support data.
        let bytes = encoded(&result)?
            .len()
            .checked_add(2048)
            .ok_or_else(|| budget("explanation metadata size overflow"))?;
        if bytes > result.query.limits.max_output_bytes as usize {
            return Err(budget(
                "explanation subject and coverage do not fit the output budget",
            ));
        }
        Ok(Self { result, bytes })
    }

    fn retain(
        &mut self,
        support: GraphAssertionSupport<'a>,
        cancelled: &AtomicBool,
    ) -> GraphResult<()> {
        checkpoint(cancelled)?;
        self.result.total_supports += 1;
        // Preserve a canonical prefix, rather than skipping a large assertion
        // and resuming with smaller ones. Continue scanning to count omissions.
        if !self.result.truncations.is_empty() {
            return Ok(());
        }
        if self.result.supports.len() >= self.result.query.limits.max_supports as usize {
            self.result
                .truncations
                .push(GraphExplanationTruncation::Supports);
            return Ok(());
        }
        let bytes = encoded(&support)?
            .len()
            .checked_add(1)
            .and_then(|len| self.bytes.checked_add(len))
            .ok_or_else(|| budget("explanation support size overflow"))?;
        checkpoint(cancelled)?;
        if bytes > self.result.query.limits.max_output_bytes as usize {
            self.result
                .truncations
                .push(GraphExplanationTruncation::OutputBytes);
            return Ok(());
        }
        self.bytes = bytes;
        self.result.supports.push(support);
        Ok(())
    }

    fn foundation(
        &mut self,
        support: GraphAssertionSupport<'a>,
        cancelled: &AtomicBool,
    ) -> GraphResult<()> {
        self.result
            .boundaries
            .push(GraphExplanationBoundary::FoundationProducerNotRetained);
        self.retain(support, cancelled)
    }
}

pub(super) fn execute<'a>(
    query: &GraphExplainQuery,
    owner: &'a GraphPartitionSnapshot,
    cancelled: &AtomicBool,
) -> GraphResult<GraphExplanation<'a>> {
    let (record, input) = select(query, owner)?;
    let mut boundaries = vec![
        GraphExplanationBoundary::EvidenceRecordsNotResolved,
        GraphExplanationBoundary::DerivationRecordsNotRetained,
        GraphExplanationBoundary::ConflictAssessmentNotAvailable,
    ];
    let coverage = match &record {
        GraphExplainedRecord::Entity { .. } => {
            boundaries.push(GraphExplanationBoundary::EntityCoverageNotModeled);
            Vec::new()
        }
        GraphExplainedRecord::Relation { edge, .. } => coverage(owner, edge.relation(), cancelled)?,
    };
    let result = GraphExplanation {
        schema: GRAPH_EXPLANATION_SCHEMA,
        query: query.clone(),
        query_digest: query_digest(query)?,
        universe: owner.snapshot().universe(),
        generation: owner.snapshot().generation(),
        foundation_snapshot_id: owner.foundation().snapshot_id(),
        source_context_id: owner.source_context_id(),
        registry: GraphExplanationRegistry {
            bundle_id: owner.registry().bundle_id(),
            version: owner.registry().version(),
            digest: owner.registry().registry_digest(),
        },
        record,
        supports: Vec::new(),
        coverage,
        scanned_assertions: 0,
        total_supports: 0,
        support_complete: false,
        truncations: Vec::new(),
        boundaries,
        absence_authoritative: false,
    };
    checkpoint(cancelled)?;
    let mut collector = Collector::new(result)?;
    match &input {
        GraphExplainSubject::Entity(id) => {
            if let Some(node) = owner.foundation().node(id) {
                collector
                    .foundation(GraphAssertionSupport::FoundationEntity { node }, cancelled)?;
            }
            for partition in owner.partitions() {
                for accepted in partition.report().accepted_entities() {
                    checkpoint(cancelled)?;
                    collector.result.scanned_assertions += 1;
                    if accepted.node().node_id() != id {
                        continue;
                    }
                    let proposal = partition
                        .batch()
                        .entity_proposal(accepted.proposal_id())
                        .ok_or_else(|| invalid("accepted entity has no retained proposal"))?;
                    let definition = owner
                        .registry()
                        .entity_kind(proposal.entity_kind_id())
                        .ok_or_else(|| invalid("accepted entity has no registry definition"))?;
                    collector.retain(
                        GraphAssertionSupport::ProducerEntity {
                            producer: origin(partition),
                            proposal,
                            accepted,
                            definition,
                        },
                        cancelled,
                    )?;
                }
            }
        }
        GraphExplainSubject::Relation(id) => {
            if let Some(edge) = owner.foundation().edge(id) {
                collector.foundation(
                    GraphAssertionSupport::FoundationRelation { edge },
                    cancelled,
                )?;
            }
            for partition in owner.partitions() {
                for accepted in partition.report().accepted_relations() {
                    checkpoint(cancelled)?;
                    collector.result.scanned_assertions += 1;
                    if accepted.edge().edge_id() != id {
                        continue;
                    }
                    let proposal = partition
                        .batch()
                        .relation_proposal(accepted.proposal_id())
                        .ok_or_else(|| invalid("accepted relation has no retained proposal"))?;
                    let definition = owner
                        .registry()
                        .relation_kind(proposal.relation_kind_id())
                        .ok_or_else(|| invalid("accepted relation has no registry definition"))?;
                    collector.retain(
                        GraphAssertionSupport::ProducerRelation {
                            producer: origin(partition),
                            proposal,
                            accepted,
                            definition,
                        },
                        cancelled,
                    )?;
                }
            }
        }
    }
    let mut result = collector.result;
    if result.total_supports == 0 {
        return Err(GraphError::new(
            GraphErrorCode::PartitionInvalid,
            "materialized graph subject has no retained support",
        ));
    }
    result.support_complete = result.supports.len() == result.total_supports as usize;
    result.boundaries.sort();
    checkpoint(cancelled)?;
    if encoded(&result)?.len() > query.limits.max_output_bytes as usize {
        return Err(budget("canonical explanation exceeds the output limit"));
    }
    checkpoint(cancelled)?;
    Ok(result)
}
