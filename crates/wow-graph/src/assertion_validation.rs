//! Cross-producer record closure is admitted with the immutable owner, never
//! inferred while rendering an explanation.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use crate::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphDerivationRecord, GraphError,
    GraphErrorCode, GraphLocalAssertion, GraphProducerPartition, GraphResult,
};

pub(crate) type Address = (Box<str>, GraphLocalAssertion);

pub(crate) fn resolve<'a>(
    partitions: &'a [GraphProducerPartition],
    local_partition: &str,
    reference: &GraphAssertionRef,
) -> GraphResult<(&'a GraphProducerPartition, GraphLocalAssertion)> {
    let (id, expected_batch, assertion) = match reference {
        GraphAssertionRef::Local { assertion } => (local_partition, None, assertion),
        GraphAssertionRef::Producer {
            partition_id,
            batch_id,
            assertion,
        } => (partition_id.as_ref(), Some(batch_id.as_ref()), assertion),
    };
    let partition = partitions
        .binary_search_by(|p| p.partition_id().cmp(id))
        .ok()
        .map(|index| &partitions[index])
        .ok_or_else(|| invalid("assertion producer is not retained"))?;
    if expected_batch.is_some_and(|batch| batch != partition.batch().batch_id()) {
        return Err(GraphError::new(
            GraphErrorCode::PartitionStale,
            "assertion reference names a stale producer batch",
        ));
    }
    let found = match assertion.kind {
        GraphAssertionKind::Entity => partition
            .batch()
            .entity_proposal(&assertion.proposal_id)
            .is_some(),
        GraphAssertionKind::Relation => partition
            .batch()
            .relation_proposal(&assertion.proposal_id)
            .is_some(),
    };
    if !found {
        return Err(invalid("assertion reference is absent or has another kind"));
    }
    Ok((partition, assertion.clone()))
}

pub(crate) fn derivation<'a>(
    partition: &'a GraphProducerPartition,
    assertion: &GraphLocalAssertion,
) -> Option<&'a GraphDerivationRecord> {
    let records = partition.batch().assertion_records()?;
    records
        .derivations
        .binary_search_by(|record| record.output.cmp(assertion))
        .ok()
        .map(|index| &records.derivations[index])
}

fn confidence(
    partition: &GraphProducerPartition,
    assertion: &GraphLocalAssertion,
) -> GraphResult<GraphConfidence> {
    match assertion.kind {
        GraphAssertionKind::Entity => partition
            .batch()
            .entity_proposal(&assertion.proposal_id)
            .map(|proposal| proposal.confidence())
            .ok_or_else(|| invalid("derivation entity is missing")),
        GraphAssertionKind::Relation => partition
            .report()
            .accepted_relations()
            .binary_search_by(|entry| entry.proposal_id().cmp(&assertion.proposal_id))
            .ok()
            .map(|index| {
                partition.report().accepted_relations()[index]
                    .edge()
                    .confidence()
            })
            .ok_or_else(|| invalid("derivation relation is not accepted")),
    }
}

pub(crate) fn validate(
    partitions: &[GraphProducerPartition],
    stop: &AtomicBool,
) -> GraphResult<()> {
    let mut links = BTreeMap::<Address, Vec<Address>>::new();
    let mut references = 0usize;
    for partition in partitions {
        crate::partition::check_cancelled(stop)?;
        let Some(records) = partition.batch().assertion_records() else {
            continue;
        };
        records.validate()?;
        for record in &records.derivations {
            crate::partition::check_cancelled(stop)?;
            let output_confidence = confidence(partition, &record.output)?;
            if output_confidence == GraphConfidence::Proven {
                return Err(invalid(
                    "a derived assertion cannot claim direct Proven authority",
                ));
            }
            let mut inputs = Vec::new();
            for reference in record.inputs.iter().chain(&record.rebuttals) {
                crate::partition::check_cancelled(stop)?;
                charge(&mut references)?;
                let (producer, assertion) =
                    resolve(partitions, partition.partition_id(), reference)?;
                // Rebuttal evidence does not lend confidence to the conclusion.
                if record.inputs.contains(reference)
                    && output_confidence < confidence(producer, &assertion)?
                {
                    return Err(invalid("derivation promotes a lower-confidence input"));
                }
                inputs.push((producer.partition_id().into(), assertion));
            }
            if !record.missing.is_empty() && output_confidence < GraphConfidence::Possible {
                return Err(invalid(
                    "incomplete derivation cannot publish a confirmed assertion",
                ));
            }
            links.insert(
                (partition.partition_id().into(), record.output.clone()),
                inputs,
            );
        }
        for conflict in &records.conflicts {
            crate::partition::check_cancelled(stop)?;
            let mut participants = BTreeSet::new();
            for reference in &conflict.assertions {
                charge(&mut references)?;
                let (producer, assertion) =
                    resolve(partitions, partition.partition_id(), reference)?;
                if !participants.insert((producer.partition_id(), assertion)) {
                    return Err(invalid("conflict repeats the same resolved assertion"));
                }
            }
            if participants.len() < 2 {
                return Err(invalid("conflict requires distinct retained assertions"));
            }
        }
    }
    // Iterative DFS bounds stack/depth and rejects cycles, including rebuttal
    // dependencies. Completed paths are memoized; fanout is charged above.
    let mut complete = BTreeMap::<Address, usize>::new();
    for root in links.keys() {
        if complete.contains_key(root) {
            continue;
        }
        let mut active = BTreeSet::new();
        let mut stack = vec![(root.clone(), false, 0usize)];
        while let Some((address, exit, depth)) = stack.pop() {
            crate::partition::check_cancelled(stop)?;
            if exit {
                let height = links
                    .get(&address)
                    .into_iter()
                    .flatten()
                    .filter_map(|input| complete.get(input))
                    .copied()
                    .max()
                    .map_or(0, |height| height + 1);
                if height > 64 {
                    return Err(GraphError::new(
                        GraphErrorCode::BudgetExceeded,
                        "assertion derivation depth exceeded",
                    ));
                }
                active.remove(&address);
                complete.insert(address, height);
                continue;
            }
            if complete.contains_key(&address) {
                continue;
            }
            if depth > 64 {
                return Err(GraphError::new(
                    GraphErrorCode::BudgetExceeded,
                    "assertion derivation depth exceeded",
                ));
            }
            if !active.insert(address.clone()) {
                return Err(invalid("cyclic assertion derivation"));
            }
            stack.push((address.clone(), true, depth));
            if let Some(inputs) = links.get(&address) {
                for input in inputs.iter().rev() {
                    stack.push((input.clone(), false, depth + 1));
                }
            }
        }
    }
    Ok(())
}

fn charge(references: &mut usize) -> GraphResult<()> {
    *references += 1;
    if *references > 4_000_000 {
        return Err(GraphError::new(
            GraphErrorCode::BudgetExceeded,
            "assertion record reference budget",
        ));
    }
    Ok(())
}

/// Existing coarse relation coverage is downgraded conservatively where a
/// reported conflict affects a relation or one of its retained endpoint nodes.
/// Exact participants/capabilities/axes remain on the original conflict row.
pub(crate) fn conflicted_relations(
    partitions: &[GraphProducerPartition],
    foundation: &crate::GraphSnapshot,
    stop: &AtomicBool,
) -> GraphResult<BTreeSet<crate::GraphRelationKind>> {
    let mut nodes = BTreeSet::new();
    let mut relations = BTreeSet::new();
    for partition in partitions {
        let Some(records) = partition.batch().assertion_records() else {
            continue;
        };
        for conflict in &records.conflicts {
            for reference in &conflict.assertions {
                crate::partition::check_cancelled(stop)?;
                let (producer, assertion) =
                    resolve(partitions, partition.partition_id(), reference)?;
                match assertion.kind {
                    GraphAssertionKind::Entity => {
                        let index = producer
                            .report()
                            .accepted_entities()
                            .binary_search_by(|entry| {
                                entry.proposal_id().cmp(&assertion.proposal_id)
                            })
                            .map_err(|_| invalid("conflict entity not admitted"))?;
                        nodes.insert(
                            producer.report().accepted_entities()[index]
                                .node()
                                .node_id()
                                .clone(),
                        );
                    }
                    GraphAssertionKind::Relation => {
                        let index = producer
                            .report()
                            .accepted_relations()
                            .binary_search_by(|entry| {
                                entry.proposal_id().cmp(&assertion.proposal_id)
                            })
                            .map_err(|_| invalid("conflict relation not admitted"))?;
                        relations.insert(
                            producer.report().accepted_relations()[index]
                                .edge()
                                .relation(),
                        );
                    }
                }
            }
        }
    }
    for edge in foundation
        .edges()
        .iter()
        .chain(partitions.iter().flat_map(|producer| {
            producer
                .report()
                .accepted_relations()
                .iter()
                .map(|entry| entry.edge())
        }))
    {
        crate::partition::check_cancelled(stop)?;
        if nodes.contains(edge.from()) || nodes.contains(edge.to()) {
            relations.insert(edge.relation());
        }
    }
    Ok(relations)
}
fn invalid(message: &str) -> GraphError {
    GraphError::new(GraphErrorCode::PartitionInvalid, message)
}
