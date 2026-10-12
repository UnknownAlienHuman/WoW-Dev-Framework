//! Exact captured callable facts -> existing recognizer engine -> call proposals.
//! The source producer owns functions; this producer owns semantic call relations.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_emmy::bindings::SymbolTarget;
use wow_emmy::function_calls::{FunctionCallReport, SourceCallTarget};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphLocalAssertion, GraphNodeId, GraphPartitionSnapshot,
    GraphProducerLookup, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation, StructuredObservationInput, run_recognizers,
};

pub const SOURCE_CALL_PARTITION: &str = "wow-recognizers.lua-direct-calls";
pub const SOURCE_CALL_PROFILE: &str = "wow-recognizers/source-function-calls/2";
/// Selected API-use adapter; the ordinary direct-call profile remains unchanged.
pub const SOURCE_API_CALL_PROFILE: &str = "wow-recognizers/source-function-api-calls/1";
pub const SOURCE_API_ENTITY_KIND: &str = "native_api_callable";
pub const SOURCE_API_RELATION: &str = "lua_uses_native_api";
const MAX_FUNCTIONS: usize = 8192;
const MAX_CALLS: usize = 8192;

/// An upstream native owner associates the exact Library tuple with an admitted
/// API producer. This adapter validates acceptance/support, never discovers the
/// association from a query name or reconstructs native/Reference authority.
/// The minimal endpoint contract carries one original platform source support.
#[derive(Debug)]
pub struct SourceApiTargetBinding<'a> {
    pub assertion: GraphAssertionRef,
    pub source_handle: &'a SourceHandle,
    pub evidence: &'a EvidenceRecord,
}

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
    /// None preserves LibraryTarget omissions and the ordinary profile/coverage.
    /// Some selects exact API use; absent keys remain LibraryTarget omissions.
    /// Rows must be used Library targets in this report (at most MAX_CALLS).
    pub api_targets: Option<&'a BTreeMap<SymbolTarget, SourceApiTargetBinding<'a>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceCallOutcome {
    Projected {
        observation_id: String,
        assertion_id: String,
        proposal_id: String,
    },
    /// Selected API use never participates in the Main-only Calls receipt set.
    ApiUseProjected {
        proposal_id: String,
        rule_id: String,
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
    if input.report.functions().len() > MAX_FUNCTIONS
        || input.report.calls().len() > MAX_CALLS
        || input
            .api_targets
            .is_some_and(|targets| targets.len() > MAX_CALLS)
    {
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
    // One validated lookup resolves exact producer receipts in the input generation.
    // Assertions must never use node IDs from a materialized generation as inputs.
    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    let graph = lookup.input_view();
    let api_selected = input.api_targets.is_some();
    let api_bindings = resolve_api_targets(&input, &lookup, stop)?;
    let partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
        let reference = GraphAssertionRef::Producer {
            partition_id: partition.partition_id().into(),
            batch_id: partition.batch().batch_id().into(),
            assertion: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: proposal_id.into(),
            },
        };
        let resolved = lookup
            .entity(lookup.scope(), &reference, stop)
            .map_err(|error| {
                if error.code() == wow_graph::GraphErrorCode::PartitionInvalid {
                    failure(RecognizerErrorCode::AdapterBindingMissing)
                } else {
                    graph_error(error)
                }
            })?;
        let proposal = resolved.proposal();
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
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "source callable proposal differs from its analyzer fact",
            ));
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
        let node = resolved.accepted().node();
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
    let mut used_api_targets = BTreeSet::new();
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
                    let source_evidence =
                        BTreeSet::from([evidence, caller.evidence, target.evidence])
                            .into_iter()
                            .collect::<Vec<_>>();
                    let observation = StructuredObservation::new(
                        StructuredObservationInput {
                            source_snapshot_id: graph.snapshot_id().clone(),
                            family: ObservationFamily::DirectCall,
                            from: caller.node.clone(),
                            to: target.node.clone(),
                            origin: ObservationOrigin::AnalyzerFact,
                            confidence: GraphConfidence::Derived,
                            evidence_ids: source_evidence
                                .iter()
                                .map(|id| id.to_string().into_boxed_str())
                                .collect(),
                        },
                        limits,
                    )?;
                    let handles = BTreeSet::from([handle, caller.handle, target.handle])
                        .into_iter()
                        .collect::<Vec<_>>();
                    if pending
                        .insert(
                            observation.observation_id().to_string(),
                            (
                                call.fact_id(),
                                handles,
                                source_evidence,
                                GraphRelationKind::Calls,
                                "lua_direct_calls",
                            ),
                        )
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    observations.push(observation);
                    None
                }
            }
            SourceCallTarget::LibraryFunction { target } => {
                if let Some(binding) = api_bindings.get(target) {
                    used_api_targets.insert(target);
                    let source_evidence =
                        BTreeSet::from([evidence, caller.evidence, binding.evidence])
                            .into_iter()
                            .collect::<Vec<_>>();
                    let observation = StructuredObservation::new(
                        StructuredObservationInput {
                            source_snapshot_id: graph.snapshot_id().clone(),
                            family: ObservationFamily::ApiUse,
                            from: caller.node.clone(),
                            to: binding.node.clone(),
                            origin: ObservationOrigin::AnalyzerFact,
                            confidence: GraphConfidence::Derived,
                            evidence_ids: source_evidence
                                .iter()
                                .map(|id| id.to_string().into_boxed_str())
                                .collect(),
                        },
                        limits,
                    )?;
                    let handles = BTreeSet::from([handle, caller.handle, binding.handle])
                        .into_iter()
                        .collect::<Vec<_>>();
                    if pending
                        .insert(
                            observation.observation_id().to_string(),
                            (
                                call.fact_id(),
                                handles,
                                source_evidence,
                                GraphRelationKind::UsesApi,
                                SOURCE_API_RELATION,
                            ),
                        )
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    observations.push(observation);
                    None
                } else {
                    Some(SourceCallOutcome::LibraryTarget)
                }
            }
            SourceCallTarget::SignatureNotCaptured => Some(SourceCallOutcome::SignatureNotCaptured),
            SourceCallTarget::Unresolved => Some(SourceCallOutcome::Unresolved),
            SourceCallTarget::Indeterminate => Some(SourceCallOutcome::Indeterminate),
        };
        if let Some(outcome) = outcome {
            receipts.insert(call.fact_id(), outcome);
        }
    }
    if used_api_targets.len() != api_bindings.len() {
        return Err(failure(RecognizerErrorCode::AdapterBindingUnknown));
    }
    let coverage = ObservationFamily::ALL
        .into_iter()
        .map(|family| {
            let (state, blocker) = if family == ObservationFamily::DirectCall {
                (
                    RecognitionCoverageState::Partial,
                    "source_calls.captured_single_signature_main_targets_only",
                )
            } else if api_selected && family == ObservationFamily::ApiUse {
                (
                    RecognitionCoverageState::Partial,
                    "source_calls.selected_exact_library_api_targets_no_absence_authority",
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
        graph,
        observations,
        coverage,
        limits,
        stop,
    )?;
    let mut relations = Vec::new();
    for assertion in recognition.assertions() {
        checkpoint(stop)?;
        let (call_id, handles, evidence_ids, relation, relation_id) = pending
            .remove(assertion.observation_id().as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingUnknown))?;
        if assertion.relation() != relation || assertion.confidence() != GraphConfidence::Derived {
            return Err(RecognizerError::new(
                RecognizerErrorCode::AdapterFactMismatch,
                "source call assertion has incompatible relation or confidence",
            ));
        }
        let proposal_id = assertion.assertion_id().to_string();
        // The generic assertion also names its structured observation. That
        // receipt is retained in RecognitionReport, not a source EvidenceRecord.
        // Check the exact engine handoff before forwarding the original typed
        // source supports to the graph owner.
        let mut expected_evidence = evidence_ids
            .iter()
            .map(|id| id.to_string().into_boxed_str())
            .collect::<Vec<_>>();
        expected_evidence.push(assertion.observation_id().as_str().into());
        expected_evidence.sort();
        expected_evidence.dedup();
        if assertion.evidence_ids() != expected_evidence {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        relations.push(
            GraphRelationProposal::new(
                proposal_id.as_str(),
                relation_id,
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
        let outcome = match relation {
            GraphRelationKind::Calls => SourceCallOutcome::Projected {
                observation_id: assertion.observation_id().to_string(),
                assertion_id: assertion.assertion_id().to_string(),
                proposal_id,
            },
            GraphRelationKind::UsesApi => SourceCallOutcome::ApiUseProjected {
                proposal_id,
                rule_id: assertion.recognizer_id().to_string(),
            },
            _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
        };
        if receipts.insert(call_id, outcome).is_some() {
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
                if relation == GraphRelationKind::Calls
                    || api_selected && relation == GraphRelationKind::UsesApi
                {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if relation == GraphRelationKind::Calls
                    || api_selected && relation == GraphRelationKind::UsesApi
                {
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
            profile: if api_selected {
                SOURCE_API_CALL_PROFILE
            } else {
                SOURCE_CALL_PROFILE
            },
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

fn resolve_api_targets<'a>(
    input: &SourceCallInput<'a>,
    lookup: &GraphProducerLookup<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<BTreeMap<&'a SymbolTarget, FunctionBinding>> {
    let mut bindings = BTreeMap::new();
    let Some(targets) = input.api_targets else {
        return Ok(bindings);
    };
    // Selection requires the reviewed API definitions even for an empty map.
    let registry = input.owner.registry();
    let entity = registry
        .entity_kind(SOURCE_API_ENTITY_KIND)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let relation = registry
        .relation_kind(SOURCE_API_RELATION)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if !entity.allows_universe(lookup.input_view().universe())
        || !entity.allows_confidence(GraphConfidence::Derived)
        || relation.relation() != GraphRelationKind::UsesApi
        || !relation.allows_source_kind("lua_source_function")
        || !relation.allows_target_kind(SOURCE_API_ENTITY_KIND)
        || !relation.allows_confidence(GraphConfidence::Derived)
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    for (target, binding) in targets {
        checkpoint(stop)?;
        target
            .span
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        target
            .content_digest
            .parse::<ContentDigest<SourceContent>>()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if target.role != "library"
            || input
                .report
                .library_snapshot_ids()
                .binary_search(&target.workspace_id)
                .is_err()
            || target.path.is_empty()
            || target.span.byte_start() == target.span.byte_end()
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let resolved = lookup
            .entity(lookup.scope(), &binding.assertion, stop)
            .map_err(graph_error)?;
        let proposal = resolved.proposal();
        let handle = binding.source_handle;
        let evidence = binding.evidence;
        handle
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        evidence
            .validate()
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        if proposal.entity_kind_id() != SOURCE_API_ENTITY_KIND
            || proposal.confidence() != GraphConfidence::Derived
            || proposal.source_handle_ids() != [handle.handle_id()]
            || proposal.evidence_ids() != [evidence.evidence_id()]
            || evidence.context_id() != input.context.context_id()
            || evidence.source_handle_ids() != [handle.handle_id()]
            || evidence.provenance() != ProvenanceClass::PlatformSource
            || evidence.confidence() != EvidenceConfidence::Proven
            || evidence.claim_scope() != ClaimScope::SourceObservation
            || !evidence.derivation_input_ids().is_empty()
            || !evidence.coverage_refs().is_empty()
            || handle.reference_generation() != Some(input.context.reference_generation())
            || handle
                .project_generation()
                .is_some_and(|generation| Some(generation) != input.context.project_generation())
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        // Original API support is not the generated Library closure span. Their
        // exact native association is a caller-owned admission prerequisite.
        bindings.insert(
            target,
            FunctionBinding {
                node: resolved.accepted().node().node_id().clone(),
                handle: handle.handle_id(),
                evidence: evidence.evidence_id(),
            },
        );
    }
    checkpoint(stop)?;
    Ok(bindings)
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
        return Err(RecognizerError::new(
            RecognizerErrorCode::AdapterFactMismatch,
            "source callable support differs from its exact source evidence",
        ));
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
