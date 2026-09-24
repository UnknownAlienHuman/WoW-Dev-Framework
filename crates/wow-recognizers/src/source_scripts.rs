//! Project-owned XML handler facts -> the existing script-assignment recognizer.
//! This adapter verifies the exact graph/evidence crosswalk, not XML or Lua syntax.
use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation, StructuredObservationInput, run_recognizers,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    ClaimScope, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext, ProvenanceClass,
    SourceHandle, StableHandleId,
};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphNodeId, GraphPartitionSnapshot,
    GraphProposalBatch, GraphProposalEndpoint, GraphRelationKind, GraphRelationProposal,
    GraphRelationProposalInput,
};

pub const SOURCE_SCRIPT_PARTITION: &str = "wow-recognizers.xml-script-bindings";
pub const SOURCE_SCRIPT_PROFILE: &str = "wow-recognizers/source-xml-scripts/1";
const MAX_BINDINGS: usize = 8192;

/// An immutable normalized crosswalk supplied by the source owner via service.
/// The project retains all lookup, inheritance, inline and skipped-site receipts.
pub struct SourceScriptFact<'a> {
    pub fact_id: &'a str,
    pub receiver_proposal_id: &'a str,
    pub handler_proposal_id: &'a str,
    pub confidence: GraphConfidence,
    pub source_handle_ids: &'a [StableHandleId],
    pub evidence_ids: &'a [EvidenceId],
}
pub struct SourceScriptInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub context: &'a GenerationContext,
    pub facts: Vec<SourceScriptFact<'a>>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceScriptReceipt {
    pub binding_id: String,
    pub observation_id: String,
    pub assertion_id: String,
    pub proposal_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceScriptRecognition {
    profile: &'static str,
    source_partition: String,
    recognition: RecognitionReport,
    receipts: Vec<SourceScriptReceipt>,
}
impl SourceScriptRecognition {
    pub fn recognition(&self) -> &RecognitionReport {
        &self.recognition
    }
    pub fn receipts(&self) -> &[SourceScriptReceipt] {
        &self.receipts
    }
}
pub struct SourceScriptProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceScriptRecognition,
}

pub fn recognize_source_scripts(
    input: SourceScriptInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceScriptProposals> {
    checkpoint(stop)?;
    if input.facts.len() > MAX_BINDINGS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .context
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = partition.report().accepted_entities();
    let endpoint = |proposal_id: &str, receiver: bool| -> RecognizerResult<GraphNodeId> {
        let proposal = partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let valid = if receiver {
            proposal.entity_kind_id() == "xml_source_declaration"
        } else {
            matches!(
                proposal.entity_kind_id(),
                "lua_source_function" | "xml_source_handler"
            )
        };
        if !valid {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let index = accepted
            .binary_search_by(|p| p.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node();
        if graph.node(node.node_id()).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        Ok(node.node_id().clone())
    };
    let limits = RecognizerLimits::new(MAX_BINDINGS as u32, MAX_BINDINGS as u32, 19, 32)?;
    let mut observations = Vec::new();
    let mut pending = BTreeMap::new();
    let mut fact_ids = BTreeSet::new();
    for fact in &input.facts {
        checkpoint(stop)?;
        let suffix = fact
            .fact_id
            .strip_prefix("xml-script-binding:sha256:")
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingInvalid))?;
        if suffix.len() != 64
            || !suffix
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !matches!(
                fact.confidence,
                GraphConfidence::Derived | GraphConfidence::Possible
            )
            || !fact_ids.insert(fact.fact_id)
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        }
        validate_support(&input, fact)?;
        // Every endpoint's original source support must be carried by the
        // observation. An ID alone is not evidence for the XML/Lua binding.
        for id in [fact.receiver_proposal_id, fact.handler_proposal_id] {
            let proposal = partition
                .batch()
                .entity_proposal(id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if proposal
                .source_handle_ids()
                .iter()
                .any(|id| fact.source_handle_ids.binary_search(id).is_err())
                || proposal
                    .evidence_ids()
                    .iter()
                    .any(|id| fact.evidence_ids.binary_search(id).is_err())
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
        }
        let observation = StructuredObservation::new(
            StructuredObservationInput {
                source_snapshot_id: graph.snapshot_id().clone(),
                family: ObservationFamily::ScriptAssignment,
                from: endpoint(fact.receiver_proposal_id, true)?,
                to: endpoint(fact.handler_proposal_id, false)?,
                origin: ObservationOrigin::ProjectFact,
                confidence: fact.confidence,
                evidence_ids: fact
                    .evidence_ids
                    .iter()
                    .map(|id| id.to_string().into_boxed_str())
                    .collect(),
            },
            limits,
        )?;
        if pending
            .insert(observation.observation_id().to_string(), fact)
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        observations.push(observation);
    }
    let coverage = ObservationFamily::ALL
        .into_iter()
        .map(|family| {
            RecognitionCoverage::new(
                family,
                if family == ObservationFamily::ScriptAssignment && !input.facts.is_empty() {
                    RecognitionCoverageState::Partial
                } else {
                    RecognitionCoverageState::NotEvaluated
                },
                vec![if family == ObservationFamily::ScriptAssignment {
                    "source_scripts.source_associations_not_effective_dispatch".into()
                } else {
                    "source_scripts.family_not_requested".into()
                }],
                limits,
            )
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let recognition = run_recognizers(
        &RecognizerRegistry::e2_default()?,
        &graph,
        observations,
        coverage,
        limits,
        stop,
    )?;
    let mut relations = Vec::new();
    let mut receipts = Vec::new();
    for assertion in recognition.assertions() {
        checkpoint(stop)?;
        let fact = pending
            .remove(assertion.observation_id().as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingUnknown))?;
        if assertion.relation() != GraphRelationKind::SetsScript
            || assertion.confidence() != fact.confidence
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let proposal_id = assertion.assertion_id().to_string();
        relations.push(
            GraphRelationProposal::new(
                proposal_id.as_str(),
                "source_xml_sets_script",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(assertion.from().clone()),
                    target: GraphProposalEndpoint::Existing(assertion.to().clone()),
                    confidence: assertion.confidence(),
                    source_handle_ids: fact.source_handle_ids.to_vec(),
                    evidence_ids: fact.evidence_ids.to_vec(),
                    coverage_ids: Vec::new(),
                },
            )
            .map_err(graph_error)?,
        );
        receipts.push(SourceScriptReceipt {
            binding_id: fact.fact_id.into(),
            observation_id: assertion.observation_id().to_string(),
            assertion_id: assertion.assertion_id().to_string(),
            proposal_id,
        });
    }
    if !pending.is_empty() || receipts.len() != input.facts.len() {
        return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
    }
    receipts.sort_by(|a, b| a.binding_id.cmp(&b.binding_id));
    let coverage = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|r| r.relation())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|relation| {
            GraphCoverageRecord::new(
                relation,
                if relation == GraphRelationKind::SetsScript && !input.facts.is_empty() {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if relation == GraphRelationKind::SetsScript {
                    "source_scripts.partial_no_runtime_dispatch_authority".into()
                } else {
                    "source_scripts.relation_owned_by_other_producer".into()
                }],
                graph.limits(),
            )
            .map_err(graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        SOURCE_SCRIPT_PARTITION,
        Vec::new(),
        relations,
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    Ok(SourceScriptProposals {
        batch,
        coverage,
        recognition: SourceScriptRecognition {
            profile: SOURCE_SCRIPT_PROFILE,
            source_partition: input.source_partition.into(),
            recognition,
            receipts,
        },
    })
}

fn validate_support(
    input: &SourceScriptInput<'_>,
    fact: &SourceScriptFact<'_>,
) -> RecognizerResult<()> {
    if fact.source_handle_ids.is_empty()
        || fact.source_handle_ids.len() > 32
        || fact.evidence_ids.is_empty()
        || fact.evidence_ids.len() > 32
        || fact.source_handle_ids.windows(2).any(|w| w[0] >= w[1])
        || fact.evidence_ids.windows(2).any(|w| w[0] >= w[1])
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }
    let mut witnessed = BTreeSet::new();
    for id in fact.evidence_ids {
        let record = input
            .evidence
            .get(id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        record
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        let [handle_id] = record.source_handle_ids() else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        if record.evidence_id() != *id
            || record.context_id() != input.context.context_id()
            || record.provenance() != ProvenanceClass::ProjectSource
            || record.confidence() != EvidenceConfidence::Proven
            || record.claim_scope() != ClaimScope::SourceObservation
            || !record.derivation_input_ids().is_empty()
            || !record.coverage_refs().is_empty()
            || fact.source_handle_ids.binary_search(handle_id).is_err()
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let handle = input
            .source_handles
            .get(handle_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        handle
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if handle.handle_id() != *handle_id
            || handle.project_generation() != input.context.project_generation()
            || handle.reference_generation() != Some(input.context.reference_generation())
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        witnessed.insert(*handle_id);
    }
    if witnessed.iter().ne(fact.source_handle_ids.iter()) {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}
fn checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}
fn failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(
        code,
        "XML script facts could not produce a coherent source recognizer partition",
    )
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}
