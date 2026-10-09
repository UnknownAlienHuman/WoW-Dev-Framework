//! Exact producer assertion chains and reported unresolved conflicts. Coverage
//! and automatic conflict assessment remain separate from these retained rows.
use super::*;
use crate::assertion_validation::{self, Address};
use crate::{GraphAssertionKind, GraphConfidence, GraphLocalAssertion, GraphProducerPartition};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn record_id(
    kind: &str,
    partition: &GraphProducerPartition,
    record: &impl Serialize,
) -> GraphResult<Box<str>> {
    let bytes = encoded(&(
        kind,
        partition.partition_id(),
        partition.partition_digest(),
        record,
    ))?;
    let digest = Sha256::digest(bytes);
    let mut text = format!("graph-{kind}:sha256:");
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        text.push(char::from(HEX[usize::from(byte >> 4)]));
        text.push(char::from(HEX[usize::from(byte & 15)]));
    }
    Ok(text.into())
}

fn address(support: &GraphAssertionSupport<'_>) -> Option<Address> {
    let (producer, kind, id) = match support {
        GraphAssertionSupport::ProducerEntity {
            producer, proposal, ..
        } => (producer, GraphAssertionKind::Entity, proposal.proposal_id()),
        GraphAssertionSupport::ProducerRelation {
            producer, proposal, ..
        } => (
            producer,
            GraphAssertionKind::Relation,
            proposal.proposal_id(),
        ),
        _ => return None,
    };
    Some((
        producer.partition_id.into(),
        GraphLocalAssertion {
            kind,
            proposal_id: id.into(),
        },
    ))
}

fn support<'a>(
    owner: &'a GraphPartitionSnapshot,
    partition: &'a GraphProducerPartition,
    assertion: &GraphLocalAssertion,
) -> GraphResult<GraphAssertionSupport<'a>> {
    let producer = super::collect::origin(partition);
    match assertion.kind {
        GraphAssertionKind::Entity => {
            let proposal = partition
                .batch()
                .entity_proposal(&assertion.proposal_id)
                .ok_or_else(|| invalid("retained entity assertion missing"))?;
            let index = partition
                .report()
                .accepted_entities()
                .binary_search_by(|entry| entry.proposal_id().cmp(&assertion.proposal_id))
                .map_err(|_| invalid("entity assertion not admitted"))?;
            let definition = owner
                .registry()
                .entity_kind(proposal.entity_kind_id())
                .ok_or_else(|| invalid("entity assertion kind missing"))?;
            Ok(GraphAssertionSupport::ProducerEntity {
                producer,
                proposal,
                accepted: &partition.report().accepted_entities()[index],
                definition,
            })
        }
        GraphAssertionKind::Relation => {
            let proposal = partition
                .batch()
                .relation_proposal(&assertion.proposal_id)
                .ok_or_else(|| invalid("retained relation assertion missing"))?;
            let index = partition
                .report()
                .accepted_relations()
                .binary_search_by(|entry| entry.proposal_id().cmp(&assertion.proposal_id))
                .map_err(|_| invalid("relation assertion not admitted"))?;
            let definition = owner
                .registry()
                .relation_kind(proposal.relation_kind_id())
                .ok_or_else(|| invalid("relation assertion kind missing"))?;
            Ok(GraphAssertionSupport::ProducerRelation {
                producer,
                proposal,
                accepted: &partition.report().accepted_relations()[index],
                definition,
            })
        }
    }
}

fn confidence(support: &GraphAssertionSupport<'_>) -> GraphConfidence {
    match support {
        GraphAssertionSupport::ProducerEntity { proposal, .. } => proposal.confidence(),
        GraphAssertionSupport::ProducerRelation { accepted, .. } => accepted.edge().confidence(),
        GraphAssertionSupport::FoundationRelation { edge } => edge.confidence(),
        GraphAssertionSupport::FoundationEntity { .. } => GraphConfidence::Proven,
    }
}

fn tick(result: &mut GraphExplanation<'_>) -> bool {
    if result.scanned_assertions >= result.query.limits.max_scanned_assertions {
        truncate(result, GraphExplanationTruncation::AssertionWork);
        return false;
    }
    result.scanned_assertions += 1;
    true
}
fn truncate(result: &mut GraphExplanation<'_>, reason: GraphExplanationTruncation) {
    if !result.truncations.contains(&reason) {
        result.truncations.push(reason);
    }
    result.derivation_complete = false;
}
fn fits(
    result: &mut GraphExplanation<'_>,
    bytes: &mut usize,
    item: &impl Serialize,
) -> GraphResult<bool> {
    let next = bytes.saturating_add(encoded(item)?.len()).saturating_add(1);
    if next > result.query.limits.max_output_bytes as usize {
        truncate(result, GraphExplanationTruncation::OutputBytes);
        return Ok(false);
    }
    *bytes = next;
    Ok(true)
}

pub(super) fn collect<'a>(
    owner: &'a GraphPartitionSnapshot,
    result: &mut GraphExplanation<'a>,
    stop: &AtomicBool,
) -> GraphResult<()> {
    result.derivation_complete = result.support_complete;
    let roots = result
        .supports
        .iter()
        .filter_map(address)
        .collect::<BTreeSet<_>>();
    let mut queue = roots
        .iter()
        .cloned()
        .map(|address| (address, 0))
        .collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut retained_conflicts = BTreeSet::new();
    let mut conflicts = BTreeMap::<Address, Vec<(&GraphProducerPartition, usize)>>::new();
    // Index conflict participants once under the same scan cap; no O(N*M)
    // rescans for shared support chains.
    for partition in owner.partitions() {
        let Some(records) = partition.batch().assertion_records() else {
            continue;
        };
        for (index, record) in records.conflicts.iter().enumerate() {
            for reference in &record.assertions {
                checkpoint(stop)?;
                if !tick(result) {
                    return Ok(());
                }
                let (producer, assertion) = assertion_validation::resolve(
                    owner.partitions(),
                    partition.partition_id(),
                    reference,
                )?;
                conflicts
                    .entry((producer.partition_id().into(), assertion))
                    .or_default()
                    .push((partition, index));
            }
        }
    }
    let mut bytes = encoded(result)?.len().saturating_add(2048);
    let mut missing_records = result
        .supports
        .iter()
        .any(|support| address(support).is_none());
    if missing_records {
        result.derivation_complete = false;
    }
    while let Some((current, depth)) = queue.pop_front() {
        checkpoint(stop)?;
        if !visited.insert(current.clone()) {
            continue;
        }
        if !tick(result) {
            break;
        }
        let partition = owner
            .partition(&current.0)
            .ok_or_else(|| invalid("support producer missing"))?;
        let item = support(owner, partition, &current.1)?;
        if !roots.contains(&current) {
            if result.supports.len() + result.assertion_supports.len()
                >= result.query.limits.max_supports as usize
            {
                truncate(result, GraphExplanationTruncation::Supports);
                break;
            }
            if !fits(result, &mut bytes, &item)? {
                break;
            }
            result.assertion_supports.push(item.clone());
        }
        if let Some(record) = assertion_validation::derivation(partition, &current.1) {
            let observation = GraphDerivationObservation {
                record_id: record_id("derivation", partition, record)?,
                producer: super::collect::origin(partition),
                record,
            };
            if !fits(result, &mut bytes, &observation)? {
                break;
            }
            result.derivations.push(observation);
            if !record.missing.is_empty() {
                result.derivation_complete = false;
                result
                    .boundaries
                    .push(GraphExplanationBoundary::DerivationSupportMissing);
            }
            for reference in record.inputs.iter().chain(&record.rebuttals) {
                checkpoint(stop)?;
                if !tick(result) {
                    break;
                }
                if depth >= result.query.limits.max_derivation_depth {
                    truncate(result, GraphExplanationTruncation::DerivationDepth);
                    continue;
                }
                let (producer, assertion) = assertion_validation::resolve(
                    owner.partitions(),
                    partition.partition_id(),
                    reference,
                )?;
                queue.push_back(((producer.partition_id().into(), assertion), depth + 1));
            }
        } else if confidence(&item) != GraphConfidence::Proven {
            missing_records = true;
            result.derivation_complete = false;
        }
        if let Some(reported) = conflicts.get(&current) {
            for (producer, index) in reported {
                checkpoint(stop)?;
                if !retained_conflicts.insert((producer.partition_id(), *index)) {
                    continue;
                }
                let record = &producer
                    .batch()
                    .assertion_records()
                    .ok_or_else(|| invalid("conflict records missing"))?
                    .conflicts[*index];
                let observation = GraphConflictObservation {
                    record_id: record_id("conflict", producer, record)?,
                    producer: super::collect::origin(producer),
                    record,
                };
                if !fits(result, &mut bytes, &observation)? {
                    break;
                }
                result.conflicts.push(observation);
                result
                    .boundaries
                    .push(GraphExplanationBoundary::RetainedConflictsUnresolved);
                for reference in &record.assertions {
                    let (peer, assertion) = assertion_validation::resolve(
                        owner.partitions(),
                        producer.partition_id(),
                        reference,
                    )?;
                    queue.push_back(((peer.partition_id().into(), assertion), depth));
                }
            }
        }
        if result
            .truncations
            .contains(&GraphExplanationTruncation::OutputBytes)
            || result
                .truncations
                .contains(&GraphExplanationTruncation::AssertionWork)
        {
            break;
        }
    }
    if !missing_records && result.derivation_complete {
        result
            .boundaries
            .retain(|boundary| *boundary != GraphExplanationBoundary::DerivationRecordsNotRetained);
    }
    result.derivations.sort_by(|a, b| {
        (a.producer.partition_id, &a.record.output)
            .cmp(&(b.producer.partition_id, &b.record.output))
    });
    result.conflicts.sort_by(|a, b| {
        (a.producer.partition_id, a.record).cmp(&(b.producer.partition_id, b.record))
    });
    result.boundaries.sort();
    result.boundaries.dedup();
    Ok(())
}
