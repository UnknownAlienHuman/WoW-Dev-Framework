//! TOC state-slot crosswalks -> the existing state-read/state-write recognizers.
//! This adapter joins exact retained owner facts; it does not parse source.
use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation, StructuredObservationInput, run_recognizers,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    CanonicalResult, ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord,
    GenerationContext, ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEntityProposal, GraphNodeId,
    GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use wow_emmy::function_calls::FunctionCallReport;
use wow_emmy::global_access::{GlobalAccessKind, GlobalAccessResolution};

pub const SOURCE_STATE_PARTITION: &str = "wow-recognizers.saved-variable-access";
pub const SOURCE_STATE_PROFILE: &str = "wow-recognizers/source-saved-variable-access/1";
const MAX_BINDINGS: usize = 8192;

/// Normalized source facts. The adapter checks the original global-access fact,
/// concrete function occurrence, declared root and literal path, not just names.
pub struct SourceStateFact<'a> {
    pub fact_id: &'a str,
    pub access_id: &'a str,
    pub root_proposal_id: &'a str,
    pub caller_proposal_id: &'a str,
    pub target_proposal_id: &'a str,
    pub kind: GlobalAccessKind,
    pub source_handle_ids: &'a [StableHandleId],
    pub evidence_ids: &'a [EvidenceId],
}
pub struct SourceStateInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub report: &'a FunctionCallReport,
    pub context: &'a GenerationContext,
    pub facts: Vec<SourceStateFact<'a>>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateReceipt {
    pub binding_id: String,
    pub access_id: String,
    pub observation_id: String,
    pub assertion_id: String,
    pub proposal_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceStateRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    source_partition: String,
    recognition: RecognitionReport,
    receipts: Vec<SourceStateReceipt>,
}
impl SourceStateRecognition {
    pub fn recognition(&self) -> &RecognitionReport {
        &self.recognition
    }
    pub fn receipts(&self) -> &[SourceStateReceipt] {
        &self.receipts
    }
}
pub struct SourceStateProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceStateRecognition,
}

pub fn recognize_source_state(
    input: SourceStateInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceStateProposals> {
    checkpoint(stop)?;
    if input.facts.len() > MAX_BINDINGS {
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
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = partition.report().accepted_entities();
    let endpoint = |proposal_id: &str| -> RecognizerResult<GraphNodeId> {
        let index = accepted
            .binary_search_by(|p| p.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node();
        if graph.node(node.node_id()).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        Ok(node.node_id().clone())
    };
    let accesses = input
        .report
        .global_accesses()
        .iter()
        .map(|a| (a.fact_id(), a))
        .collect::<BTreeMap<_, _>>();
    let functions = input
        .report
        .functions()
        .iter()
        .map(|f| (f.fact_id(), f))
        .collect::<BTreeMap<_, _>>();
    let limits = RecognizerLimits::new(MAX_BINDINGS as u32, MAX_BINDINGS as u32, 19, 32)?;
    let mut observations = Vec::new();
    let mut pending = BTreeMap::new();
    let mut fact_ids = BTreeSet::new();
    for fact in &input.facts {
        checkpoint(stop)?;
        if !fact_ids.insert(fact.access_id) {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        let access = accesses
            .get(fact.access_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if fact.kind != access.kind()
            || !access.path_complete()
            || access.resolution() != GlobalAccessResolution::MainGlobal
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        family(fact.kind)?;
        let target_source = access
            .declaration()
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let source_index = input
            .report
            .files()
            .binary_search_by(|f| f.path.cmp(&target_source.path))
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        let source = &input.report.files()[source_index];
        if source.parse_error_count != 0 || source.content_digest != target_source.content_digest {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let identity = wow_core::domain_separated_digest(
            "wow-project/saved-access/1",
            &(
                access.fact_id(),
                fact.root_proposal_id,
                access.kind(),
                fact.target_proposal_id,
                fact.source_handle_ids,
                fact.evidence_ids,
            ),
        )
        .map_err(|_| failure(RecognizerErrorCode::AdapterBindingInvalid))?;
        if fact.fact_id
            != format!(
                "saved-access:{}",
                ContentDigest::<CanonicalResult>::from_bytes(identity)
            )
        {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        validate_support(&input, fact)?;
        let function = functions
            .get(access.function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let proposal = |id: &str| {
            partition
                .batch()
                .entity_proposal(id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))
        };
        let caller = proposal(fact.caller_proposal_id)?;
        let root = proposal(fact.root_proposal_id)?;
        let target = proposal(fact.target_proposal_id)?;
        if caller.entity_kind_id() != "lua_source_function"
            || caller.confidence() != GraphConfidence::Derived
            || caller.semantic_key()
                != &BTreeMap::from([
                    (
                        "document".into(),
                        GraphProposalValue::String(function.path().into()),
                    ),
                    (
                        "function".into(),
                        GraphProposalValue::String(function.fact_id().into()),
                    ),
                ])
            || root.entity_kind_id() != "state_root"
            || root.confidence() != GraphConfidence::Proven
            || root.semantic_key().get("name")
                != Some(&GraphProposalValue::String(access.root_name().into()))
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        if access.keys().is_empty() {
            if fact.target_proposal_id != fact.root_proposal_id {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
        } else {
            let path = String::from_utf8(
                wow_core::canonical_json_bytes(&access.keys())
                    .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
            )
            .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
            if target.entity_kind_id() != "state_path"
                || target.confidence() != GraphConfidence::Derived
                || target.semantic_key()
                    != &BTreeMap::from([
                        (
                            "root".into(),
                            GraphProposalValue::Identifier(fact.root_proposal_id.into()),
                        ),
                        ("path".into(), GraphProposalValue::String(path.into())),
                    ])
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
        }
        require_support(fact, caller)?;
        require_support(fact, root)?;
        // A reused symbolic path need not repeat its first observation's evidence.
        // This observation instead proves the identical path at its own exact site.
        located_support(
            &input,
            fact,
            access.path(),
            access.content_digest(),
            access.span(),
        )?;
        located_support(
            &input,
            fact,
            function.path(),
            function.content_digest(),
            function.span(),
        )?;
        let declaration = access
            .declaration()
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if declaration.role != "main" || declaration.workspace_id != input.report.main_snapshot_id()
        {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        located_support(
            &input,
            fact,
            &declaration.path,
            &declaration.content_digest,
            declaration.span,
        )?;
        let observation = StructuredObservation::new(
            StructuredObservationInput {
                source_snapshot_id: graph.snapshot_id().clone(),
                family: family(fact.kind)?,
                from: endpoint(fact.caller_proposal_id)?,
                to: endpoint(fact.target_proposal_id)?,
                origin: ObservationOrigin::ProjectFact,
                confidence: GraphConfidence::Derived,
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
                if input
                    .facts
                    .iter()
                    .any(|f| family_for_kind(f.kind) == Some(family))
                {
                    RecognitionCoverageState::Partial
                } else {
                    RecognitionCoverageState::NotEvaluated
                },
                vec![if matches!(
                    family,
                    ObservationFamily::StateRead | ObservationFamily::StateWrite
                ) {
                    "source_state.source_slots_not_runtime_values".into()
                } else {
                    "source_state.family_not_requested".into()
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
        if assertion.relation() != relation(fact.kind)?
            || assertion.confidence() != GraphConfidence::Derived
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let proposal_id = assertion.assertion_id().to_string();
        relations.push(
            GraphRelationProposal::new(
                proposal_id.as_str(),
                definition(fact.kind)?,
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
        receipts.push(SourceStateReceipt {
            binding_id: fact.fact_id.into(),
            access_id: fact.access_id.into(),
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
                if input
                    .facts
                    .iter()
                    .any(|f| relation_for_kind(f.kind) == Some(relation))
                {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if matches!(
                    relation,
                    GraphRelationKind::ReadsState | GraphRelationKind::WritesState
                ) {
                    "source_state.partial_no_runtime_persistence_authority".into()
                } else {
                    "source_state.relation_owned_by_other_producer".into()
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
        SOURCE_STATE_PARTITION,
        Vec::new(),
        relations,
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    Ok(SourceStateProposals {
        batch,
        coverage,
        recognition: SourceStateRecognition {
            profile: SOURCE_STATE_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            source_partition: input.source_partition.into(),
            recognition,
            receipts,
        },
    })
}

fn family_for_kind(kind: GlobalAccessKind) -> Option<ObservationFamily> {
    match kind {
        GlobalAccessKind::Read => Some(ObservationFamily::StateRead),
        GlobalAccessKind::Write => Some(ObservationFamily::StateWrite),
        GlobalAccessKind::UnsupportedAssignment => None,
    }
}
fn relation_for_kind(kind: GlobalAccessKind) -> Option<GraphRelationKind> {
    match kind {
        GlobalAccessKind::Read => Some(GraphRelationKind::ReadsState),
        GlobalAccessKind::Write => Some(GraphRelationKind::WritesState),
        GlobalAccessKind::UnsupportedAssignment => None,
    }
}
fn family(kind: GlobalAccessKind) -> RecognizerResult<ObservationFamily> {
    family_for_kind(kind).ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))
}
fn relation(kind: GlobalAccessKind) -> RecognizerResult<GraphRelationKind> {
    relation_for_kind(kind).ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))
}
fn definition(kind: GlobalAccessKind) -> RecognizerResult<&'static str> {
    match kind {
        GlobalAccessKind::Read => Ok("source_reads_state"),
        GlobalAccessKind::Write => Ok("source_writes_state"),
        GlobalAccessKind::UnsupportedAssignment => {
            Err(failure(RecognizerErrorCode::AdapterFactMismatch))
        }
    }
}
fn require_support(
    fact: &SourceStateFact<'_>,
    proposal: &GraphEntityProposal,
) -> RecognizerResult<()> {
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
    Ok(())
}
fn located_support(
    input: &SourceStateInput<'_>,
    fact: &SourceStateFact<'_>,
    path: &str,
    digest: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    let digest = digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    // validate_support already establishes each handle's exact witnessed evidence.
    if !fact.source_handle_ids.iter().any(|id| {
        input.source_handles.get(id).is_some_and(|h| {
            h.path().as_str() == path && h.content_digest() == &digest && h.span() == span
        })
    }) {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn validate_support(
    input: &SourceStateInput<'_>,
    fact: &SourceStateFact<'_>,
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
        "SavedVariables source facts could not produce a coherent recognizer partition",
    )
}
fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}
