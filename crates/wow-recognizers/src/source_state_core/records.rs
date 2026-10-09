//! Exact graph prerequisites of the matcher outputs. Missing derivation records
//! in earlier owners remain explanation boundaries rather than invented leaves.
use super::*;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphDerivationRecord, GraphLocalAssertion,
};

pub(super) fn attach(
    input: &SourceStateCoreInput<'_>,
    family: SourceStateCoreFamily,
    receipts: &BTreeMap<String, SourceStateCoreReceipt>,
    batch: GraphProposalBatch,
    stop: &AtomicBool,
) -> RecognizerResult<GraphProposalBatch> {
    if receipts.is_empty() {
        return Ok(batch);
    }
    let legacy = input
        .owner
        .partition(SOURCE_STATE_PARTITION)
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let source = input
        .owner
        .partition(input.recognition.source_partition())
        .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
    let legacy_receipts = input
        .recognition
        .receipts()
        .iter()
        .map(|receipt| (receipt.binding_id.as_str(), receipt))
        .collect::<BTreeMap<_, _>>();
    let source_nodes = source
        .report()
        .accepted_entities()
        .iter()
        .map(|accepted| (accepted.node().node_id(), accepted.proposal_id()))
        .collect::<BTreeMap<_, _>>();
    let mut prerequisites = BTreeMap::<GraphLocalAssertion, BTreeSet<GraphAssertionRef>>::new();
    for receipt in receipts.values() {
        checkpoint(stop)?;
        let mut inputs = BTreeSet::new();
        for fact in &receipt.fact_ids {
            let Some(admission) = legacy_receipts.get(fact.as_str()) else {
                continue;
            };
            let index = legacy
                .report()
                .accepted_relations()
                .binary_search_by(|accepted| accepted.proposal_id().cmp(&admission.proposal_id))
                .map_err(|_| fail(RecognizerErrorCode::AdapterBindingMissing))?;
            let edge = legacy.report().accepted_relations()[index].edge();
            inputs.insert(GraphAssertionRef::Producer {
                partition_id: legacy.partition_id().into(),
                batch_id: legacy.batch().batch_id().into(),
                assertion: GraphLocalAssertion {
                    kind: GraphAssertionKind::Relation,
                    proposal_id: admission.proposal_id.clone().into(),
                },
            });
            for node in [edge.from(), edge.to()] {
                let id = source_nodes
                    .get(node)
                    .ok_or_else(|| fail(RecognizerErrorCode::AdapterBindingMissing))?;
                inputs.insert(GraphAssertionRef::Producer {
                    partition_id: source.partition_id().into(),
                    batch_id: source.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: (*id).into(),
                    },
                });
            }
        }
        if inputs.is_empty() {
            return Err(fail(RecognizerErrorCode::AdapterBindingMissing));
        }
        for (kind, ids) in [
            (GraphAssertionKind::Entity, &receipt.entity_proposal_ids),
            (GraphAssertionKind::Relation, &receipt.relation_proposal_ids),
        ] {
            for id in ids {
                let merged = prerequisites
                    .entry(GraphLocalAssertion {
                        kind,
                        proposal_id: id.clone().into(),
                    })
                    .or_default();
                merged.extend(inputs.iter().cloned());
                if merged.len() > 64 {
                    return Err(fail(RecognizerErrorCode::BudgetExceeded));
                }
            }
        }
    }
    let derivations = prerequisites
        .into_iter()
        .map(|(output, inputs)| GraphDerivationRecord {
            output,
            rule_id: family.rule_id().into(),
            rule_version: 1,
            inputs: inputs.into_iter().collect(),
            rebuttals: Vec::new(),
            missing: Vec::new(),
        })
        .collect();
    let records = GraphAssertionRecords::build(
        GraphAssertionRecordScope {
            universe: batch.universe().clone(),
            generation: batch.generation().clone(),
            source_context_id: batch.source_context_id(),
        },
        derivations,
        Vec::new(),
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    batch.with_assertion_records(records).map_err(graph_error)
}
