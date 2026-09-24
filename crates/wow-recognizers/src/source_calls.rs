//! Exact captured callable facts -> existing recognizer engine -> Calls proposals.
//! The source producer owns functions; this producer owns semantic call relations.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_emmy::function_calls::{FunctionCallReport, SourceCallTarget};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphNodeId, GraphPartitionSnapshot,
    GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue, GraphRelationKind,
    GraphRelationProposal, GraphRelationProposalInput,
};

use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation, StructuredObservationInput, run_recognizers,
};

pub const SOURCE_CALL_PARTITION: &str = "wow-recognizers.lua-direct-calls";
pub const SOURCE_CALL_PROFILE: &str = "wow-recognizers/source-function-calls/1";
const MAX_FUNCTIONS: usize = 8192;
const MAX_CALLS: usize = 8192;

/// Caller supplied crosswalks are checked against source proposals and real records.
pub struct SourceCallInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub report: &'a FunctionCallReport,
    pub context: &'a GenerationContext,
    pub function_proposals: BTreeMap<&'a str, &'a str>,
    pub call_support: BTreeMap<&'a str, (StableHandleId, EvidenceId)>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceCallOutcome {
    Projected {
        observation_id: String,
        assertion_id: String,
        proposal_id: String,
    },
    SelfRecursionUnsupported,
    LibraryTarget,
    SignatureNotCaptured,
    Unresolved,
    Indeterminate,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallReceipt {
    pub call_id: String,
    pub outcome: SourceCallOutcome,
}

/// Original recognizer assertions remain scoped to the input graph, before
/// graph-owned rebinding. The batch and final graph retain the matching proposals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCallRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    source_partition: String,
    recognition: RecognitionReport,
    receipts: Vec<SourceCallReceipt>,
}
impl SourceCallRecognition {
    pub fn recognition(&self) -> &RecognitionReport {
        &self.recognition
    }
    pub fn receipts(&self) -> &[SourceCallReceipt] {
        &self.receipts
    }
}

pub struct SourceCallProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceCallRecognition,
}

struct FunctionBinding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

pub fn recognize_source_calls(
    input: SourceCallInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceCallProposals> {
    checkpoint(stop)?;
    if input.report.functions().len() > MAX_FUNCTIONS || input.report.calls().len() > MAX_CALLS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .report
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    input
        .context
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id()
        || input.function_proposals.len() != input.report.functions().len()
        || input.call_support.len() != input.report.calls().len()
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }
    // input_view validates the complete owner and reverses publication rebinding.
    // Assertions must never use node IDs from a materialized generation as inputs.
    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = partition.report().accepted_entities();
    let mut bindings = BTreeMap::new();
    let mut used_proposals = BTreeSet::new();
    for function in input.report.functions() {
        checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if !used_proposals.insert(proposal_id) {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        let proposal = partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let expected = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String(function.path().into()),
            ),
            (
                "function".into(),
                GraphProposalValue::String(function.fact_id().into()),
            ),
        ]);
        if proposal.entity_kind_id() != "lua_source_function"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        validate_support(
            &input,
            *handle,
            *evidence,
            function.path(),
            function.content_digest(),
            function.span(),
        )?;
        let index = accepted
            .binary_search_by(|p| p.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node();
        if graph.node(node.node_id()).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        bindings.insert(
            function.fact_id(),
            FunctionBinding {
                node: node.node_id().clone(),
                handle: *handle,
                evidence: *evidence,
            },
        );
    }
    let limits = RecognizerLimits::new(MAX_CALLS as u32, MAX_CALLS as u32, 19, 32)?;
    let mut observations = Vec::new();
    let mut pending = BTreeMap::new();
    let mut receipts = BTreeMap::new();
    for call in input.report.calls() {
        checkpoint(stop)?;
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        validate_support(
            &input,
            handle,
            evidence,
            call.path(),
            call.content_digest(),
            call.call_span(),
        )?;
        let caller = bindings
            .get(call.caller_function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let outcome = match call.target() {
            SourceCallTarget::MainFunction { function_id } => {
                let target = bindings
                    .get(function_id.as_str())
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                if caller.node == target.node {
                    Some(SourceCallOutcome::SelfRecursionUnsupported)
                } else {
                    let evidence_ids = BTreeSet::from([evidence, caller.evidence, target.evidence])
                        .into_iter()
                        .map(|id| id.to_string().into_boxed_str())
                        .collect();
                    let observation = StructuredObservation::new(
                        StructuredObservationInput {
                            source_snapshot_id: graph.snapshot_id().clone(),
                            family: ObservationFamily::DirectCall,
                            from: caller.node.clone(),
                            to: target.node.clone(),
                            origin: ObservationOrigin::AnalyzerFact,
                            confidence: GraphConfidence::Derived,
                            evidence_ids,
                        },
                        limits,
                    )?;
                    let handles = BTreeSet::from([handle, caller.handle, target.handle])
                        .into_iter()
                        .collect::<Vec<_>>();
                    if pending
                        .insert(
                            observation.observation_id().to_string(),
                            (call.fact_id(), handles),
                        )
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    observations.push(observation);
                    None
                }
            }
            SourceCallTarget::LibraryFunction { .. } => Some(SourceCallOutcome::LibraryTarget),
            SourceCallTarget::SignatureNotCaptured => Some(SourceCallOutcome::SignatureNotCaptured),
            SourceCallTarget::Unresolved => Some(SourceCallOutcome::Unresolved),
            SourceCallTarget::Indeterminate => Some(SourceCallOutcome::Indeterminate),
        };
        if let Some(outcome) = outcome {
            receipts.insert(call.fact_id(), outcome);
        }
    }
    let coverage = ObservationFamily::ALL
        .into_iter()
        .map(|family| {
            let (state, blocker) = if family == ObservationFamily::DirectCall {
                (
                    RecognitionCoverageState::Partial,
                    "source_calls.captured_single_signature_main_targets_only",
                )
            } else {
                (
                    RecognitionCoverageState::NotEvaluated,
                    "source_calls.family_not_requested",
                )
            };
            RecognitionCoverage::new(family, state, vec![blocker.into()], limits)
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
    for assertion in recognition.assertions() {
        checkpoint(stop)?;
        if assertion.relation() != GraphRelationKind::Calls
            || assertion.confidence() != GraphConfidence::Derived
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let (call_id, handles) = pending
            .remove(assertion.observation_id().as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingUnknown))?;
        let proposal_id = assertion.assertion_id().to_string();
        let evidence_ids = assertion
            .evidence_ids()
            .iter()
            .map(|id| {
                id.parse::<EvidenceId>()
                    .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))
            })
            .collect::<RecognizerResult<Vec<_>>>()?;
        relations.push(
            GraphRelationProposal::new(
                proposal_id.as_str(),
                "lua_direct_calls",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(assertion.from().clone()),
                    target: GraphProposalEndpoint::Existing(assertion.to().clone()),
                    confidence: assertion.confidence(),
                    source_handle_ids: handles,
                    evidence_ids,
                    coverage_ids: Vec::new(),
                },
            )
            .map_err(graph_error)?,
        );
        if receipts
            .insert(
                call_id,
                SourceCallOutcome::Projected {
                    observation_id: assertion.observation_id().to_string(),
                    assertion_id: assertion.assertion_id().to_string(),
                    proposal_id,
                },
            )
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    if !pending.is_empty() || receipts.len() != input.report.calls().len() {
        return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
    }
    // Describe every stored family. This producer does not certify source-owned
    // relations or unrecognized families, and never grants absence authority.
    let families = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|r| r.relation())
        .collect::<BTreeSet<_>>();
    let coverage = families
        .into_iter()
        .map(|relation| {
            GraphCoverageRecord::new(
                relation,
                if relation == GraphRelationKind::Calls {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if relation == GraphRelationKind::Calls {
                    "source_calls.static_partial_no_runtime_authority".into()
                } else {
                    "source_calls.relation_owned_by_other_producer".into()
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
        SOURCE_CALL_PARTITION,
        Vec::new(),
        relations,
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    Ok(SourceCallProposals {
        batch,
        coverage,
        recognition: SourceCallRecognition {
            profile: SOURCE_CALL_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            source_partition: input.source_partition.into(),
            recognition,
            receipts: receipts
                .into_iter()
                .map(|(id, outcome)| SourceCallReceipt {
                    call_id: id.into(),
                    outcome,
                })
                .collect(),
        },
    })
}

fn validate_support(
    input: &SourceCallInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    digest: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    let handle = input
        .source_handles
        .get(&handle_id)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let evidence = input
        .evidence
        .get(&evidence_id)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    handle
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    evidence
        .validate()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let digest = digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if handle.handle_id() != handle_id
        || handle.path().as_str() != path
        || handle.span() != span
        || handle.content_digest() != &digest
        || evidence.evidence_id() != evidence_id
        || evidence.context_id() != input.context.context_id()
        || evidence.source_handle_ids() != [handle_id]
        || evidence.provenance() != ProvenanceClass::ProjectSource
        || evidence.confidence() != EvidenceConfidence::Proven
        || evidence.claim_scope() != ClaimScope::SourceObservation
        || handle.project_generation() != input.context.project_generation()
        || handle.reference_generation() != Some(input.context.reference_generation())
        || !evidence.derivation_input_ids().is_empty()
        || !evidence.coverage_refs().is_empty()
    {
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
        "captured function-call facts could not produce a coherent recognizer partition",
    )
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}
