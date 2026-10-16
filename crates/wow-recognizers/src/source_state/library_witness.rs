//! Opt-in W6 proof projection; legacy packs and their output bytes stay intact.
use super::*;
use crate::matcher_witness::charge;
use wow_graph::{GraphDerivationRecord, GraphProducerPartition};

pub const SOURCE_STATE_LIBRARY_WITNESS_PROFILE: &str =
    "wow-recognizers/source-library/witness-derivation/1";
const MAX_ENTITY_WITNESSES: usize = 64;

/// Original matcher result, not a reconstructed match from proposal strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLibraryWitness {
    pub entity_proposal_id: Box<str>,
    pub caller: GraphAssertionRef,
    pub output_id: Box<str>,
    pub matched: crate::RecognizerMatch,
}

pub(super) struct PendingWitness {
    output_id: Box<str>,
    matched: crate::RecognizerMatch,
    rule_version: u32,
}

/// Select original matcher/fact receipts and Graph-owned derivation records.
/// Upstream source assertions with no derivation remain incomplete in Explain.
pub fn recognize_source_library_witnesses(
    input: SourceLibraryInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceLibraryProposals> {
    recognize_source_library_inner(input, true, stop)
}

pub(super) fn capture(
    outcome: &crate::RecognizerRuleOutcome,
    proposal: &crate::RecognizerProposedAssertion,
    used: &mut u64,
    stop: &AtomicBool,
) -> RecognizerResult<PendingWitness> {
    let captured = crate::matcher_witness::capture(outcome, proposal, used, stop)?;
    Ok(PendingWitness {
        output_id: captured.witness.output_id,
        matched: captured.witness.matched,
        rule_version: captured.rule_version,
    })
}

pub(super) fn attach(
    batch: GraphProposalBatch,
    matches: &mut [SourceLibraryMatch],
    pending: &mut [W6Relation],
    entity_ids: &BTreeMap<String, String>,
    source: &GraphProducerPartition,
    stop: &AtomicBool,
) -> RecognizerResult<GraphProposalBatch> {
    if matches.is_empty() {
        return Ok(batch);
    }
    let indices = pending
        .iter()
        .enumerate()
        .map(|(index, relation)| (relation.proposal_id.clone(), index))
        .collect::<BTreeMap<_, _>>();
    let mut derivations = Vec::new();
    let mut entities = BTreeMap::<String, Vec<GraphAssertionRef>>::new();
    let mut used = 0;
    for receipt in matches {
        checkpoint(stop)?;
        let index = *indices
            .get(&receipt.relation_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let relation = &mut pending[index];
        let entity = entity_ids
            .get(&receipt.library_name)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let local_relation = GraphLocalAssertion {
            kind: GraphAssertionKind::Relation,
            proposal_id: relation.proposal_id.clone().into(),
        };
        let inputs = entities.entry(entity.clone()).or_default();
        if inputs.len() >= MAX_ENTITY_WITNESSES {
            return Err(failure(RecognizerErrorCode::BudgetExceeded));
        }
        inputs.push(GraphAssertionRef::Local {
            assertion: local_relation.clone(),
        });
        let caller = GraphAssertionRef::Producer {
            partition_id: source.partition_id().into(),
            batch_id: source.batch().batch_id().into(),
            assertion: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: receipt.caller_proposal_id.clone().into(),
            },
        };
        let original = relation
            .witness
            .take()
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let witness = SourceLibraryWitness {
            entity_proposal_id: entity.clone().into(),
            caller: caller.clone(),
            output_id: original.output_id,
            matched: original.matched,
        };
        charge(&witness, &mut used, stop)?;
        derivations.push(GraphDerivationRecord {
            output: local_relation,
            rule_id: relation.rule_id.clone().into(),
            rule_version: original.rule_version,
            inputs: vec![caller],
            rebuttals: Vec::new(),
            missing: Vec::new(),
        });
        receipt.witness = Some(witness);
    }
    if pending.iter().any(|relation| relation.witness.is_some()) {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }
    for (entity_id, inputs) in entities {
        checkpoint(stop)?;
        derivations.push(GraphDerivationRecord {
            output: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: entity_id.into(),
            },
            rule_id: "wow-recognizers.library-major-support-union".into(),
            rule_version: 1,
            inputs,
            rebuttals: Vec::new(),
            missing: Vec::new(),
        });
    }
    let records = wow_graph::GraphAssertionRecords::build(
        GraphAssertionRecordScope {
            universe: batch.universe().clone(),
            generation: batch.generation().clone(),
            source_context_id: batch.source_context_id(),
        },
        derivations,
        Vec::new(),
    )
    .map_err(graph_error)?;
    batch.with_assertion_records(records).map_err(graph_error)
}
