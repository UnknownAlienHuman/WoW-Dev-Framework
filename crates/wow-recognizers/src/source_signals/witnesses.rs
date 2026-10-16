//! Original matcher outputs and the existing Graph derivation DAG for W1/W3/W4.
use super::*;
use crate::matcher_witness::{CapturedMatcherWitness, OriginalMatcherWitness, capture, charge};
use wow_graph::{
    GraphAssertionRecordScope, GraphAssertionRecords, GraphDerivationRecord, GraphProducerPartition,
};

pub(super) const W1_PROFILE: &str = "wow-recognizers/lua-native-frame-events/witness-derivation/1";
pub(super) const W3_PROFILE: &str = "wow-recognizers/lua-custom-signals/witness-derivation/1";
pub(super) const W4_PROFILE: &str = "wow-recognizers/lua-cvar-callback/witness-derivation/1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalWitness {
    pub entity_proposal_id: Box<str>,
    pub relation_proposal_id: Box<str>,
    pub matcher_relation_proposal_id: Box<str>,
    pub caller: GraphAssertionRef,
    pub matcher_outputs: BTreeMap<Box<str>, OriginalMatcherWitness>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub corroborating_relations: Vec<GraphAssertionRef>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalMatcherOutcome {
    pub rule_id: Box<str>,
    pub rule_version: u32,
    pub state: crate::RecognizerRuleOutcomeState,
    pub truncated: bool,
    pub blockers: Vec<Box<str>>,
}
pub(super) fn outcomes(
    output: &crate::RecognizerOutputPartition,
    active: bool,
    stop: &AtomicBool,
) -> RecognizerResult<Option<Vec<SignalMatcherOutcome>>> {
    if !active {
        return Ok(None);
    }
    if output.outcomes().len() > 8 {
        return Err(budget());
    }
    let borrowed = output
        .outcomes()
        .iter()
        .map(|outcome| {
            (
                outcome.rule_id(),
                outcome.rule_version(),
                outcome.state(),
                outcome.truncated(),
                outcome.blocker_ids(),
            )
        })
        .collect::<Vec<_>>();
    charge(&borrowed, &mut 0, stop)?;
    Ok(Some(
        output
            .outcomes()
            .iter()
            .map(|outcome| SignalMatcherOutcome {
                rule_id: outcome.rule_id().into(),
                rule_version: outcome.rule_version(),
                state: outcome.state(),
                truncated: outcome.truncated(),
                blockers: outcome.blocker_ids().to_vec(),
            })
            .collect(),
    ))
}

pub(super) struct RelationBinding<'a> {
    pub call_id: &'a str,
    pub entity: &'a str,
    pub relation: &'a str,
    pub caller: &'a str,
    pub receiver: Option<&'a str>,
    pub producer_relations: &'a [String],
    pub missing_producer: bool,
}

pub(super) struct Proof {
    active: bool,
    aggregate_rule: &'static str,
    by_call: BTreeMap<String, BTreeMap<Box<str>, CapturedMatcherWitness>>,
    entities: BTreeMap<String, Vec<GraphAssertionRef>>,
    derivations: Vec<GraphDerivationRecord>,
    final_bytes: u64,
}
impl Proof {
    pub fn new(
        output: &crate::RecognizerOutputPartition,
        bundle: &RecognizerFactBundle,
        active: bool,
        aggregate_rule: &'static str,
        stop: &AtomicBool,
    ) -> RecognizerResult<Self> {
        let mut proof = Self {
            active,
            aggregate_rule,
            by_call: BTreeMap::new(),
            entities: BTreeMap::new(),
            derivations: Vec::new(),
            final_bytes: 0,
        };
        if !active {
            return Ok(proof);
        }
        let mut used = 0;
        for outcome in output.outcomes() {
            for proposal in outcome.proposals() {
                w1_checkpoint(stop)?;
                let facts = match proposal {
                    crate::RecognizerProposedAssertion::Entity {
                        decisive_fact_ids, ..
                    }
                    | crate::RecognizerProposedAssertion::Relation {
                        decisive_fact_ids, ..
                    } => decisive_fact_ids,
                };
                let [fact_id] = facts.as_slice() else {
                    return Err(invalid());
                };
                let fact = bundle.fact_by_id(fact_id).ok_or_else(invalid)?;
                let Some(RecognizerFactValue::Reference(call)) = fact.field("call_id") else {
                    return Err(invalid());
                };
                let original = capture(outcome, proposal, &mut used, stop)?;
                if proof
                    .by_call
                    .entry(call.to_string())
                    .or_default()
                    .insert(proposal.proposal_id().to_string().into(), original)
                    .is_some()
                {
                    return Err(invalid());
                }
            }
        }
        Ok(proof)
    }

    pub fn relation(
        &mut self,
        binding: RelationBinding<'_>,
        source: &GraphProducerPartition,
        stop: &AtomicBool,
    ) -> RecognizerResult<Option<SignalWitness>> {
        if !self.active {
            return Ok(None);
        }
        w1_checkpoint(stop)?;
        let originals = self.by_call.remove(binding.call_id).ok_or_else(invalid)?;
        let mut relations = originals
            .iter()
            .filter(|(_, original)| original.kind == GraphAssertionKind::Relation);
        let (matcher_relation_id, relation) = relations.next().ok_or_else(invalid)?;
        if relations.next().is_some() {
            return Err(invalid());
        }
        let matcher_relation_id = matcher_relation_id.clone();
        let rule_id = relation.rule_id.clone();
        let rule_version = relation.rule_version;
        let caller = reference(source, binding.caller);
        if binding.producer_relations.len() > 62 {
            return Err(budget());
        }
        let mut corroborating = Vec::new();
        for relation in binding.producer_relations {
            w1_checkpoint(stop)?;
            corroborating.push(GraphAssertionRef::Local {
                assertion: GraphLocalAssertion {
                    kind: GraphAssertionKind::Relation,
                    proposal_id: relation.clone().into(),
                },
            });
        }
        let mut inputs = vec![caller.clone()];
        if let Some(receiver) = binding.receiver {
            inputs.push(reference(source, receiver));
        }
        inputs.extend(corroborating.iter().cloned());
        inputs.sort();
        inputs.dedup();
        if inputs.len() > 64 {
            return Err(budget());
        }
        let local = GraphLocalAssertion {
            kind: GraphAssertionKind::Relation,
            proposal_id: binding.relation.into(),
        };
        let entity_inputs = self.entities.entry(binding.entity.to_owned()).or_default();
        if entity_inputs.len() >= 64 {
            return Err(budget());
        }
        entity_inputs.push(GraphAssertionRef::Local {
            assertion: local.clone(),
        });
        self.derivations.push(GraphDerivationRecord {
            output: local,
            rule_id,
            rule_version,
            inputs,
            rebuttals: Vec::new(),
            missing: if binding.missing_producer {
                vec!["compatible_source_producer_unconfirmed".into()]
            } else {
                Vec::new()
            },
        });
        let witness = SignalWitness {
            entity_proposal_id: binding.entity.into(),
            relation_proposal_id: binding.relation.into(),
            matcher_relation_proposal_id: matcher_relation_id,
            caller,
            matcher_outputs: originals
                .into_iter()
                .map(|(id, captured)| (id, captured.witness))
                .collect(),
            corroborating_relations: corroborating,
        };
        charge(&witness, &mut self.final_bytes, stop)?;
        Ok(Some(witness))
    }

    pub fn finish(
        mut self,
        batch: GraphProposalBatch,
        stop: &AtomicBool,
    ) -> RecognizerResult<GraphProposalBatch> {
        if !self.active {
            return Ok(batch);
        }
        if !self.by_call.is_empty() {
            return Err(invalid());
        }
        if self.derivations.is_empty() {
            return Ok(batch);
        }
        for (entity, inputs) in self.entities {
            w1_checkpoint(stop)?;
            if batch.entity_proposal(&entity).is_none() {
                return Err(invalid());
            }
            self.derivations.push(GraphDerivationRecord {
                output: GraphLocalAssertion {
                    kind: GraphAssertionKind::Entity,
                    proposal_id: entity.into(),
                },
                rule_id: self.aggregate_rule.into(),
                rule_version: 1,
                inputs,
                rebuttals: Vec::new(),
                missing: Vec::new(),
            });
        }
        let records = GraphAssertionRecords::build(
            GraphAssertionRecordScope {
                universe: batch.universe().clone(),
                generation: batch.generation().clone(),
                source_context_id: batch.source_context_id(),
            },
            self.derivations,
            Vec::new(),
        )
        .map_err(w1_graph_error)?;
        batch
            .with_assertion_records(records)
            .map_err(w1_graph_error)
    }
}
fn reference(source: &GraphProducerPartition, proposal: &str) -> GraphAssertionRef {
    GraphAssertionRef::Producer {
        partition_id: source.partition_id().into(),
        batch_id: source.batch().batch_id().into(),
        assertion: GraphLocalAssertion {
            kind: GraphAssertionKind::Entity,
            proposal_id: proposal.into(),
        },
    }
}
fn invalid() -> RecognizerError {
    w1_failure(RecognizerErrorCode::AdapterBindingInvalid)
}
fn budget() -> RecognizerError {
    w1_failure(RecognizerErrorCode::BudgetExceeded)
}
