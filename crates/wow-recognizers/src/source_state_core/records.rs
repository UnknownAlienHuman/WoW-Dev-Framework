//! Exact graph prerequisites of the matcher outputs. Missing derivation records
//! in earlier owners remain explanation boundaries rather than invented leaves.
use super::*;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRecords, GraphAssertionRef,
    GraphDerivationRecord, GraphLocalAssertion,
};

pub(super) fn attach(
    family: SourceStateCoreFamily,
    receipts: &BTreeMap<String, SourceStateCoreReceipt>,
    source_inputs: &BTreeMap<String, BTreeSet<GraphAssertionRef>>,
    batch: GraphProposalBatch,
    stop: &AtomicBool,
) -> RecognizerResult<GraphProposalBatch> {
    if receipts.is_empty() {
        return Ok(batch);
    }
    let mut prerequisites = BTreeMap::<GraphLocalAssertion, BTreeSet<GraphAssertionRef>>::new();
    for receipt in receipts.values() {
        checkpoint(stop)?;
        let mut inputs = BTreeSet::new();
        for fact in &receipt.fact_ids {
            let Some(admission) = source_inputs.get(fact.as_str()) else {
                continue;
            };
            inputs.extend(admission.iter().cloned());
            checkpoint(stop)?;
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
