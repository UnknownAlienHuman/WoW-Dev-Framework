//! TOC state-slot crosswalks -> the existing state-read/state-write recognizers.
//! This adapter joins exact retained owner facts; it does not parse source.
#![allow(dead_code)]
mod assertions;
use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry, RecognizerResult,
    StructuredObservation, StructuredObservationInput, run_recognizers,
};
pub use assertions::{
    SOURCE_STATE_ASSERTION_PROFILE, SourceStateAssertionEndpoints, SourceStateAssertionFact,
    SourceStateAssertionInput, SourceStateAssertionProposals, SourceStateAssertionRecognition,
    recognize_source_state_assertions,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    CanonicalResult, ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord,
    GenerationContext, ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphCoverageRecord, GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProducerLookup, GraphProposalBatch, GraphProposalEndpoint,
    GraphProposalValue, GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use wow_emmy::function_calls::{
    FunctionCallEvidenceProfile, FunctionCallReport, SourceCallConstant, SourceCallTarget,
};
use wow_emmy::global_access::{GlobalAccessKind, GlobalAccessResolution};

pub const SOURCE_STATE_PARTITION: &str = "wow-recognizers.saved-variable-access";
pub const SOURCE_STATE_PROFILE: &str = "wow-recognizers/source-saved-variable-access/3";
const MAX_BINDINGS: usize = 8192;

/// Normalized source facts. The adapter checks the original global-access fact,
/// concrete function occurrence, declared root and literal path, not just names.
#[derive(Clone, Copy, Serialize)]
pub struct SourceStateFact<'a> {
    pub fact_id: &'a str,
    pub access_id: &'a str,
    pub root_proposal_id: &'a str,
    pub caller_proposal_id: &'a str,
    pub target_proposal_id: &'a str,
    pub kind: GlobalAccessKind,
    pub confidence: GraphConfidence,
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
    pub fn analyzer_report_id(&self) -> &str {
        &self.analyzer_report_id
    }
    pub fn source_partition(&self) -> &str {
        &self.source_partition
    }
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
    let source_partition = input.source_partition;
    let input = StateInput {
        owner: input.owner,
        report: input.report,
        context: input.context,
        facts: input.facts,
        source_handles: input.source_handles,
        evidence: input.evidence,
    };
    let result = recognize_state(&input, &StateBindings::Legacy(source_partition), stop)?;
    Ok(SourceStateProposals {
        batch: result.batch,
        coverage: result.coverage,
        recognition: SourceStateRecognition {
            profile: SOURCE_STATE_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            source_partition: source_partition.into(),
            recognition: result.recognition,
            receipts: result.receipts,
        },
    })
}

struct StateInput<'a> {
    owner: &'a GraphPartitionSnapshot,
    report: &'a FunctionCallReport,
    context: &'a GenerationContext,
    facts: Vec<SourceStateFact<'a>>,
    source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}
struct StateOutput {
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
    recognition: RecognitionReport,
    receipts: Vec<SourceStateReceipt>,
}
enum StateBindings<'a> {
    Legacy(&'a str),
    Assertions {
        scope: &'a GraphAssertionRecordScope,
        endpoints: &'a BTreeMap<&'a str, &'a SourceStateAssertionEndpoints>,
    },
}
#[derive(Clone, Copy)]
enum StateRole {
    Root,
    Caller,
    Target,
}
impl StateRole {
    fn id<'a>(self, fact: &SourceStateFact<'a>) -> &'a str {
        match self {
            Self::Root => fact.root_proposal_id,
            Self::Caller => fact.caller_proposal_id,
            Self::Target => fact.target_proposal_id,
        }
    }
    fn reference(self, endpoints: &SourceStateAssertionEndpoints) -> &GraphAssertionRef {
        match self {
            Self::Root => &endpoints.root,
            Self::Caller => &endpoints.caller,
            Self::Target => &endpoints.target,
        }
    }
}
impl StateBindings<'_> {
    fn proposal<'a>(
        &self,
        owner: &'a GraphPartitionSnapshot,
        lookup: &GraphProducerLookup<'a>,
        fact: &SourceStateFact<'_>,
        role: StateRole,
        stop: &AtomicBool,
    ) -> RecognizerResult<&'a GraphEntityProposal> {
        match self {
            Self::Legacy(partition) => owner
                .partition(partition)
                .and_then(|p| p.batch().entity_proposal(role.id(fact)))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing)),
            Self::Assertions { scope, endpoints } => {
                let refs = endpoints
                    .get(fact.fact_id)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                Ok(crate::source_assertions::entity(
                    lookup,
                    scope,
                    role.reference(refs),
                    role.id(fact),
                    stop,
                )?
                .proposal())
            }
        }
    }
    fn node(
        &self,
        owner: &GraphPartitionSnapshot,
        lookup: &GraphProducerLookup<'_>,
        fact: &SourceStateFact<'_>,
        role: StateRole,
        stop: &AtomicBool,
    ) -> RecognizerResult<GraphNodeId> {
        match self {
            Self::Legacy(partition) => {
                let accepted = owner
                    .partition(partition)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
                    .report()
                    .accepted_entities();
                let index = accepted
                    .binary_search_by(|p| p.proposal_id().cmp(role.id(fact)))
                    .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let node = accepted[index].node();
                if lookup.input_view().node(node.node_id()).is_none() {
                    return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
                }
                Ok(node.node_id().clone())
            }
            Self::Assertions { scope, endpoints } => {
                let refs = endpoints
                    .get(fact.fact_id)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let resolved = crate::source_assertions::entity(
                    lookup,
                    scope,
                    role.reference(refs),
                    role.id(fact),
                    stop,
                )?;
                let node = resolved.accepted().node().node_id().clone();
                checkpoint(stop)?;
                Ok(node)
            }
        }
    }
    fn same_root_target(&self, fact: &SourceStateFact<'_>) -> RecognizerResult<bool> {
        match self {
            Self::Legacy(_) => Ok(fact.target_proposal_id == fact.root_proposal_id),
            Self::Assertions { endpoints, .. } => {
                let refs = endpoints
                    .get(fact.fact_id)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                Ok(refs.root == refs.target)
            }
        }
    }
    fn prerequisites(
        &self,
        owner: &GraphPartitionSnapshot,
        fact: &SourceStateFact<'_>,
        stop: &AtomicBool,
    ) -> RecognizerResult<Vec<GraphAssertionRef>> {
        let refs = match self {
            Self::Legacy(partition) => {
                let source = owner
                    .partition(partition)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let mut ids = vec![
                    fact.caller_proposal_id,
                    fact.target_proposal_id,
                    fact.root_proposal_id,
                ];
                ids.sort();
                ids.dedup();
                ids.into_iter()
                    .map(|id| GraphAssertionRef::Producer {
                        partition_id: source.partition_id().into(),
                        batch_id: source.batch().batch_id().into(),
                        assertion: GraphLocalAssertion {
                            kind: GraphAssertionKind::Entity,
                            proposal_id: id.into(),
                        },
                    })
                    .collect()
            }
            Self::Assertions { endpoints, .. } => {
                let refs = endpoints
                    .get(fact.fact_id)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                [refs.caller.clone(), refs.target.clone(), refs.root.clone()]
                    .into_iter()
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            }
        };
        checkpoint(stop)?;
        Ok(refs)
    }
}

fn recognize_state(
    input: &StateInput<'_>,
    bindings: &StateBindings<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<StateOutput> {
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
    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    let graph = lookup.input_view();
    match bindings {
        StateBindings::Legacy(partition) if input.owner.partition(partition).is_none() => {
            return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
        }
        StateBindings::Assertions { scope, .. } if *scope != lookup.scope() => {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        _ => {}
    }
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
        let confidence = if access.is_alias() {
            GraphConfidence::Possible
        } else {
            GraphConfidence::Derived
        };
        if fact.kind != access.kind()
            || fact.confidence != confidence
            || access.alias_blocker().is_some()
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
            "wow-project/saved-access/2",
            &(
                access.fact_id(),
                fact.root_proposal_id,
                access.kind(),
                confidence,
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
        validate_support(input, fact)?;
        let function = functions
            .get(access.function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let caller = bindings.proposal(input.owner, &lookup, fact, StateRole::Caller, stop)?;
        let root = bindings.proposal(input.owner, &lookup, fact, StateRole::Root, stop)?;
        let target = bindings.proposal(input.owner, &lookup, fact, StateRole::Target, stop)?;
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
            if fact.target_proposal_id != fact.root_proposal_id
                || !bindings.same_root_target(fact)?
            {
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
            input,
            fact,
            access.path(),
            access.content_digest(),
            access.span(),
        )?;
        located_support(
            input,
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
            input,
            fact,
            &declaration.path,
            &declaration.content_digest,
            declaration.span,
        )?;
        for hop in access.aliases() {
            checkpoint(stop)?;
            located_support(
                input,
                fact,
                access.path(),
                access.content_digest(),
                hop.statement_span,
            )?;
        }
        let observation = StructuredObservation::new(
            StructuredObservationInput {
                source_snapshot_id: graph.snapshot_id().clone(),
                family: family(fact.kind)?,
                from: bindings.node(input.owner, &lookup, fact, StateRole::Caller, stop)?,
                to: bindings.node(input.owner, &lookup, fact, StateRole::Target, stop)?,
                origin: ObservationOrigin::ProjectFact,
                confidence,
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
        graph,
        observations,
        coverage,
        limits,
        stop,
    )?;
    let mut relations = Vec::new();
    let mut receipts = Vec::new();
    let mut derivations = Vec::new();
    for assertion in recognition.assertions() {
        checkpoint(stop)?;
        let fact = pending
            .remove(assertion.observation_id().as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingUnknown))?;
        if assertion.relation() != relation(fact.kind)? || assertion.confidence() != fact.confidence
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let proposal_id = assertion.assertion_id().to_string();
        let prerequisites = bindings.prerequisites(input.owner, fact, stop)?;
        derivations.push(wow_graph::GraphDerivationRecord {
            output: wow_graph::GraphLocalAssertion {
                kind: wow_graph::GraphAssertionKind::Relation,
                proposal_id: proposal_id.clone().into(),
            },
            rule_id: match fact.kind {
                GlobalAccessKind::Read => "wow-recognizers.state-read-admission",
                GlobalAccessKind::Write => "wow-recognizers.state-write-admission",
                GlobalAccessKind::UnsupportedAssignment => {
                    return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                }
            }
            .into(),
            rule_version: 1,
            inputs: prerequisites,
            rebuttals: Vec::new(),
            missing: Vec::new(),
        });
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
    let batch = if derivations.is_empty() {
        batch
    } else {
        let records = wow_graph::GraphAssertionRecords::build(
            wow_graph::GraphAssertionRecordScope {
                universe: graph.universe().clone(),
                generation: graph.generation().clone(),
                source_context_id: input.context.context_id(),
            },
            derivations,
            Vec::new(),
        )
        .map_err(graph_error)?;
        batch.with_assertion_records(records).map_err(graph_error)?
    };
    Ok(StateOutput {
        batch,
        coverage,
        recognition,
        receipts,
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
    input: &StateInput<'_>,
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

fn validate_support(input: &StateInput<'_>, fact: &SourceStateFact<'_>) -> RecognizerResult<()> {
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

// ===== BEGIN WORKER 6: library family =====

// Exact LibStub/GetLibrary/NewLibrary structural call facts -> the declarative
// core library recognizer -> graph proposals. The opt-in AceAddon profile consumes
// only typed result-slot and selected-call evidence. This producer consumes only
// generation-bound Emmy owner facts. It never reparses Lua, never executes Lua or
// repository scripts, and never infers an upstream repository, a license, a loaded
// revision, or an embed relation from a path or folder name such as Libs/.

use wow_core::canonical_json_bytes;
use wow_emmy::function_calls::SourceCallLiteral;

use crate::{
    RecognizerClause, RecognizerError, RecognizerFact, RecognizerFactBundle,
    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind,
    RecognizerFactValue, RecognizerOutput, RecognizerOutputConfidence, RecognizerPack,
    RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerRule, compile_recognizer_plan, execute_recognizer_plan, parse_recognizer_pack,
};

pub const SOURCE_STATE_LIBRARY_PARTITION: &str = "wow-recognizers.lua-library";
pub const SOURCE_STATE_LIBRARY_PROFILE: &str = "wow-recognizers/source-library/4";
const W6_FACT_PARTITION: &str = "wow-recognizers.lua-library-facts";
const W6_FACT_PROFILE: &str = "wow-recognizers-lua-library-facts-4";
const W6_LOCAL_RESULT_FACT_PROFILE: &str = "wow-recognizers-lua-library-facts-5";
const W6_MAX_CALLS: usize = 8192;
const W6_MAX_FUNCTIONS: usize = 8192;
const W6_MAX_ARGUMENTS_RETAINED: usize = 3;
const W6_MAX_LIBRARY_KEY_BYTES: usize = 256;
const W6_MAX_PROPOSALS: usize = 65_536;
// Reuse the existing bounded source-state entity-merge support ceiling.
const W6_MAX_ENTITY_SUPPORT: usize = 64;
const W6_RULE_VERSION: u32 = 2;
const W6_ACEADDON_EMBED_RULE_VERSION: u32 = 1;
const W6_REQUIRE_RULE: &str = "core.library.libstub_require";
const W6_NEW_RULE: &str = "core.library.libstub_new";
const W6_ACEADDON_EMBED_RULE: &str = "calibration.library.aceaddon_embed";
const W6_LIBSTUB_CALLABLE: &str = "LibStub";
const W6_GET_LIBRARY_CALLABLE: &str = "LibStub.GetLibrary";
const W6_NEW_LIBRARY_CALLABLE: &str = "LibStub.NewLibrary";
const W6_ACEADDON_EMBED_CALLABLE: &str = "AceAddon.EmbedLibrary";
const W6_ACEADDON_MAJOR: &str = "AceAddon-3.0";

// Normative relation ids from RULE_FAMILIES.md section 6.
const W6_REQUIRE_RELATION: &str = "lua_requires_library";
const W6_NEW_RELATION: &str = "lua_declares_library";
const W6_EMBED_RELATION: &str = "lua_embeds_library";
const W6_LIBRARY_ENTITY: &str = "library";
const W6_LIBRARY_RELATIONS: [&str; 2] = [W6_REQUIRE_RELATION, W6_NEW_RELATION];
const W6_DEFERRED_LIBRARY_RELATIONS: [&str; 1] = [W6_EMBED_RELATION];
const SOURCE_STATE_LIBRARY_ACEADDON_PROFILE: &str =
    "wow-recognizers/source-library/aceaddon-local-result/1";

// Caller supplied crosswalks, checked against the owner source partition and the
// real source/evidence records before any fact is built.
pub struct SourceLibraryInput<'a> {
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
pub struct SourceLibraryMatch {
    pub call_id: String,
    /// Complete LibStub major key, including any hyphens and suffix text.
    pub library_name: String,
    pub caller_proposal_id: String,
    /// LibStub minor is independent of the major key and absent unless the
    /// normalized call facts provide an exact value.
    // Canonical Service graph results reject JSON null, so absence is omission.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_binding_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initializer_call_fact_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initializer_source_handle_id: Option<StableHandleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initializer_evidence_id: Option<EvidenceId>,
    pub relation_proposal_id: String,
    pub rule_id: &'static str,
    pub relation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLibraryRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matches: Vec<SourceLibraryMatch>,
}

impl SourceLibraryRecognition {
    pub fn matches(&self) -> &[SourceLibraryMatch] {
        &self.matches
    }
}

pub struct SourceLibraryProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceLibraryRecognition,
}

// One retained call, normalized from the owner report. Exact argument values and
// kinds become facts; nothing here reparses or re-evaluates source text.
struct W6Call {
    call_id: String,
    caller_proposal_id: String,
    callable_key: Option<String>,
    colon_call: bool,
    argument_count: usize,
    exact: bool,
    callable_target_selected: Option<bool>,
    receiver_proof: Option<W6LocalResultReceiverProof>,
    handle: StableHandleId,
    evidence: EvidenceId,
    argument_0_kind: Option<&'static str>,
    argument_0_key: Option<String>,
    argument_0_value: Option<String>,
    argument_0_local_constant_kind: Option<&'static str>,
    argument_1_kind: Option<&'static str>,
    argument_1_key: Option<String>,
    argument_1_value: Option<String>,
    argument_1_local_constant_kind: Option<&'static str>,
    argument_1_integer: Option<i64>,
    argument_2_kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct W6LocalResultReceiverProof {
    binding_id: String,
    result_slot: u8,
    initializer_call_fact_id: String,
    initializer_callable_key: Option<String>,
    initializer_major: Option<String>,
    initializer_target_selected: bool,
    initializer_source_handle_id: StableHandleId,
    initializer_evidence_id: EvidenceId,
}

// One exact LibStub major key and optional separately resolved minor observed at
// one exact call site. Neither axis proves a loaded revision or ownership.
struct W6LibraryKey {
    major: String,
    minor: Option<i64>,
}

// One pending relation from a matched rule, before the projection.
struct W6Relation {
    proposal_id: String,
    rule_id: String,
    relation: &'static str,
    call_id: String,
    confidence: GraphConfidence,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<wow_core::CoverageId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct W6LibraryEntitySupport {
    source_handle_ids: BTreeSet<StableHandleId>,
    evidence_ids: BTreeSet<EvidenceId>,
    coverage_ids: BTreeSet<wow_core::CoverageId>,
    confidence: GraphConfidence,
}

impl Default for W6LibraryEntitySupport {
    fn default() -> Self {
        Self {
            source_handle_ids: BTreeSet::new(),
            evidence_ids: BTreeSet::new(),
            coverage_ids: BTreeSet::new(),
            confidence: GraphConfidence::Derived,
        }
    }
}

impl W6LibraryEntitySupport {
    fn include(&mut self, relation: &W6Relation) -> RecognizerResult<()> {
        self.source_handle_ids
            .extend(relation.source_handle_ids.iter().copied());
        self.evidence_ids
            .extend(relation.evidence_ids.iter().copied());
        self.coverage_ids
            .extend(relation.coverage_ids.iter().copied());
        if self.source_handle_ids.len() > W6_MAX_ENTITY_SUPPORT
            || self.evidence_ids.len() > W6_MAX_ENTITY_SUPPORT
            || self.coverage_ids.len() > W6_MAX_PROPOSALS
        {
            return Err(failure(RecognizerErrorCode::BudgetExceeded));
        }
        if relation.confidence == GraphConfidence::Possible {
            self.confidence = GraphConfidence::Possible;
        }
        Ok(())
    }

    fn proposal(self, proposal_id: String, major: &str) -> RecognizerResult<GraphEntityProposal> {
        GraphEntityProposal::new(
            proposal_id,
            W6_LIBRARY_ENTITY,
            BTreeMap::from([(
                "library".into(),
                GraphProposalValue::String(major.to_owned().into_boxed_str()),
            )]),
            self.confidence,
            self.source_handle_ids.into_iter().collect(),
            self.evidence_ids.into_iter().collect(),
            self.coverage_ids.into_iter().collect(),
        )
        .map_err(graph_error)
    }
}

fn library_entity_proposal_id(
    context_id: wow_core::GenerationContextId,
    major: &str,
) -> RecognizerResult<String> {
    library_entity_proposal_id_for_profile(context_id, major, SOURCE_STATE_LIBRARY_PROFILE)
}

fn library_entity_proposal_id_for_profile(
    context_id: wow_core::GenerationContextId,
    major: &str,
    profile: &str,
) -> RecognizerResult<String> {
    let digest = wow_core::domain_separated_digest(
        "wow-recognizers/source-library-entity/4",
        &(profile, context_id, major),
    )
    .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "w6-library-{}",
        ContentDigest::<CanonicalResult>::from_bytes(digest)
    ))
}

fn include_library_support(
    entities: &mut BTreeMap<String, W6LibraryEntitySupport>,
    major: &str,
    relation: &W6Relation,
) -> RecognizerResult<()> {
    entities
        .entry(major.to_owned())
        .or_default()
        .include(relation)
}

fn source_call_target_selected(
    report: &FunctionCallReport,
    call: &wow_emmy::function_calls::SourceCallFact,
) -> bool {
    let Some(selected) = call
        .resolved_callable_key()
        .and_then(|key| report.named_targets().get(key))
    else {
        return false;
    };
    selected_source_call_target_matches(selected, call.target())
}

fn selected_source_call_target_matches(
    selected: &SourceCallTarget,
    call_target: &SourceCallTarget,
) -> bool {
    matches!(
        selected,
        SourceCallTarget::MainFunction { .. } | SourceCallTarget::LibraryFunction { .. }
    ) && selected == call_target
}

fn exact_source_call_string(
    call: &wow_emmy::function_calls::SourceCallFact,
    ordinal: usize,
) -> Option<String> {
    let argument = call.arguments().get(ordinal)?;
    match (argument.literal(), argument.local_constant()) {
        (Some(SourceCallLiteral::String(value)), None) => Some(value.clone()),
        (None, Some(value)) => match value.value() {
            SourceCallConstant::String(value) => Some(value.clone()),
            SourceCallConstant::Integer(_) => None,
        },
        _ => None,
    }
}

fn local_result_receiver_proof(
    input: &SourceLibraryInput<'_>,
    call: &wow_emmy::function_calls::SourceCallFact,
) -> RecognizerResult<Option<W6LocalResultReceiverProof>> {
    let report = input.report;
    if report.local_call_result_profile()
        != Some(FunctionCallEvidenceProfile::AceAddonLocalResultV1)
        || call.resolved_callable_key() != Some(W6_ACEADDON_EMBED_CALLABLE)
    {
        return Ok(None);
    }
    let Some(binding_id) = call.receiver_binding_id() else {
        return Ok(None);
    };
    let binding = report
        .call_result_binding(binding_id)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if binding.binding_id() != binding_id
        || binding.path() != call.path()
        || binding.content_digest() != call.content_digest()
        || binding.scope_function_id() != call.caller_function_id()
        || binding.initializer_call_fact_id() == call.fact_id()
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let initializer = report
        .calls()
        .iter()
        .find(|candidate| candidate.fact_id() == binding.initializer_call_fact_id())
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if initializer.path() != binding.path()
        || initializer.content_digest() != binding.content_digest()
        || initializer.caller_function_id() != binding.scope_function_id()
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let (initializer_source_handle_id, initializer_evidence_id) = input
        .call_support
        .get(initializer.fact_id())
        .copied()
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    validate_call_support(
        input,
        initializer_source_handle_id,
        initializer_evidence_id,
        initializer,
    )?;
    let initializer_key = initializer.resolved_callable_key().map(str::to_owned);
    let initializer_major = (initializer_key.as_deref() == Some(W6_NEW_LIBRARY_CALLABLE))
        .then(|| exact_source_call_string(initializer, 0))
        .flatten();
    let initializer_target_selected = source_call_target_selected(report, initializer);
    Ok(Some(W6LocalResultReceiverProof {
        binding_id: binding.binding_id().into(),
        result_slot: binding.result_slot(),
        initializer_call_fact_id: initializer.fact_id().into(),
        initializer_callable_key: initializer_key,
        initializer_major,
        initializer_target_selected,
        initializer_source_handle_id,
        initializer_evidence_id,
    }))
}

pub fn recognize_source_library(
    input: SourceLibraryInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceLibraryProposals> {
    checkpoint(stop)?;
    if input.report.calls().len() > W6_MAX_CALLS
        || input.report.functions().len() > W6_MAX_FUNCTIONS
        || input.call_support.len() != input.report.calls().len()
        || input.function_proposals.len() != input.report.functions().len()
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
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
    }
    let aceaddon_local_result_profile = input.report.local_call_result_profile()
        == Some(FunctionCallEvidenceProfile::AceAddonLocalResultV1);
    let recognition_profile = if aceaddon_local_result_profile {
        SOURCE_STATE_LIBRARY_ACEADDON_PROFILE
    } else {
        SOURCE_STATE_LIBRARY_PROFILE
    };

    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let mut caller_nodes = BTreeMap::<String, GraphNodeId>::new();
    let mut proposal_nodes = BTreeMap::<String, GraphNodeId>::new();
    for function in input.report.functions() {
        checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let reference = GraphAssertionRef::Producer {
            partition_id: source_partition.partition_id().into(),
            batch_id: source_partition.batch().batch_id().into(),
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
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let node = resolved.accepted().node().node_id().clone();
        if proposal_nodes
            .insert(proposal_id.to_owned(), node.clone())
            .is_some()
            || caller_nodes.insert(proposal_id.to_owned(), node).is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut calls = Vec::new();
    for call in input.report.calls() {
        checkpoint(stop)?;
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        validate_call_support(&input, handle, evidence, call)?;
        let caller_proposal_id = *input
            .function_proposals
            .get(call.caller_function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let mut exact = true;
        let mut argument_0_kind = None;
        let mut argument_0_key = None;
        let mut argument_0_value = None;
        let mut argument_0_local_constant_kind = None;
        let mut argument_1_kind = None;
        let mut argument_1_key = None;
        let mut argument_1_value = None;
        let mut argument_1_local_constant_kind = None;
        let mut argument_1_integer = None;
        let mut argument_2_kind = None;
        for (ordinal, argument) in call
            .arguments()
            .iter()
            .take(W6_MAX_ARGUMENTS_RETAINED)
            .enumerate()
        {
            let (Some(start), Some(end)) =
                (argument.span().byte_start(), argument.span().byte_end())
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            if start > end {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let kind = match (
                argument.reference_key(),
                argument.literal(),
                argument.local_constant(),
            ) {
                (Some(key), None, None) => {
                    if ordinal == 0 {
                        argument_0_key = Some(key.to_owned());
                    } else if ordinal == 1 {
                        argument_1_key = Some(key.to_owned());
                    }
                    "reference"
                }
                (None, Some(SourceCallLiteral::String(value)), None) => {
                    // Library-key bounds belong to the reviewed LibStub shape,
                    // not every unrelated string argument in the source graph.
                    if ordinal == 0 {
                        argument_0_value = Some(value.clone());
                    } else if ordinal == 1 {
                        argument_1_value = Some(value.clone());
                    }
                    "string"
                }
                (None, Some(SourceCallLiteral::Boolean(_)), None) => "boolean",
                (None, Some(SourceCallLiteral::Nil), None) => "nil",
                (None, None, Some(local_constant)) => {
                    if ordinal == 0 {
                        argument_0_key = Some(local_constant.name().to_owned());
                        match local_constant.value() {
                            SourceCallConstant::String(value) => {
                                argument_0_value = Some(value.clone());
                                argument_0_local_constant_kind = Some("string");
                            }
                            SourceCallConstant::Integer(_) => {
                                argument_0_local_constant_kind = Some("integer");
                            }
                        }
                    } else if ordinal == 1 {
                        argument_1_key = Some(local_constant.name().to_owned());
                        match local_constant.value() {
                            SourceCallConstant::String(value) => {
                                argument_1_value = Some(value.clone());
                                argument_1_local_constant_kind = Some("string");
                            }
                            SourceCallConstant::Integer(value) => {
                                argument_1_integer = Some(*value);
                                argument_1_local_constant_kind = Some("integer");
                            }
                        }
                    }
                    "reference"
                }
                (None, None, None) => {
                    exact = false;
                    "dynamic"
                }
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            };
            if ordinal == 0 {
                argument_0_kind = Some(kind);
            } else if ordinal == 1 {
                argument_1_kind = Some(kind);
            } else {
                argument_2_kind = Some(kind);
            }
        }
        if call.arguments().len() > W6_MAX_ARGUMENTS_RETAINED {
            exact = false;
        }
        if let Some(key) = call.resolved_callable_key()
            && !wow_emmy::bindings::supported_path(key)
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let receiver_proof = local_result_receiver_proof(&input, call)?;
        let mut normalized = W6Call {
            call_id: call.fact_id().to_owned(),
            caller_proposal_id: caller_proposal_id.to_owned(),
            callable_key: call.resolved_callable_key().map(str::to_owned),
            colon_call: call.is_colon_call(),
            argument_count: call.arguments().len(),
            exact,
            callable_target_selected: aceaddon_local_result_profile
                .then(|| source_call_target_selected(input.report, call)),
            receiver_proof,
            handle,
            evidence,
            argument_0_kind,
            argument_0_key,
            argument_0_value,
            argument_0_local_constant_kind,
            argument_1_kind,
            argument_1_key,
            argument_1_value,
            argument_1_local_constant_kind,
            argument_1_integer,
            argument_2_kind,
        };
        normalized.exact = library_key(&normalized)?.is_some();
        facts.push(build_library_fact(&input, &normalized, fact_limits)?);
        calls.push(normalized);
    }
    if facts.len() > W6_MAX_CALLS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }

    let coverage_state = if input.report.source_health_complete() {
        RecognizerFactCoverageState::Complete
    } else {
        RecognizerFactCoverageState::NotEvaluated
    };
    let fact_profile = if aceaddon_local_result_profile {
        W6_LOCAL_RESULT_FACT_PROFILE
    } else {
        W6_FACT_PROFILE
    };
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W6_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: fact_profile.into(),
            state: coverage_state,
            blocker_ids: if coverage_state == RecognizerFactCoverageState::Complete {
                Vec::new()
            } else {
                vec!["emmy.call_source_parse_failed".into()]
            },
        },
        fact_limits,
    )?];
    let bundle = RecognizerFactBundle::build(
        input.context,
        W6_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = library_pack(
        input.owner.registry().bundle_id(),
        aceaddon_local_result_profile,
    )?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut pending = Vec::<W6Relation>::new();
    for outcome in output.outcomes() {
        checkpoint(stop)?;
        let expected_version = match outcome.rule_id() {
            W6_REQUIRE_RULE | W6_NEW_RULE => Some(W6_RULE_VERSION),
            W6_ACEADDON_EMBED_RULE if aceaddon_local_result_profile => {
                Some(W6_ACEADDON_EMBED_RULE_VERSION)
            }
            _ => None,
        };
        if expected_version != Some(outcome.rule_version()) {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let expected = relation_for_rule(outcome.rule_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Relation {
                proposal_id,
                relation_kind_id,
                source,
                target,
                confidence,
                decisive_fact_ids,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } = proposal
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            if relation_kind_id.as_ref() != expected {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let [fact_id] = decisive_fact_ids.as_slice() else {
                return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
            };
            let fact = bundle
                .fact_by_id(fact_id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let Some(RecognizerFactValue::Reference(fact_caller)) = fact.field("caller") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let Some(RecognizerFactValue::String(fact_library)) = fact.field("library_name") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let RecognizerFactValue::Reference(caller_proposal_id) = source else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let RecognizerFactValue::String(library_name) = target else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            if caller_proposal_id != fact_caller
                || library_name != fact_library
                || !caller_nodes.contains_key(caller_proposal_id.as_ref())
                || !valid_library_name(library_name.as_ref())
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            if source_handle_ids.is_empty()
                || evidence_ids.is_empty()
                || !source_handle_ids.windows(2).all(|pair| pair[0] < pair[1])
                || !evidence_ids.windows(2).all(|pair| pair[0] < pair[1])
                || !coverage_ids.windows(2).all(|pair| pair[0] < pair[1])
                || source_handle_ids.len() > 64
                || evidence_ids.len() > 64
                || !calls.iter().any(|call| call.call_id == call_id.as_ref())
            {
                return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
            }
            pending.push(W6Relation {
                proposal_id: proposal_id.to_string(),
                rule_id: outcome.rule_id().to_owned(),
                relation: expected,
                call_id: call_id.to_string(),
                confidence: match *confidence {
                    RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
                    RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
                },
                source_handle_ids: source_handle_ids.clone(),
                evidence_ids: evidence_ids.clone(),
                coverage_ids: coverage_ids.clone(),
            });
        }
    }
    if pending.len() > W6_MAX_PROPOSALS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }

    // The normative library family is recorded exactly as this pack declared it.
    let mut matches = Vec::<SourceLibraryMatch>::new();
    for relation in &pending {
        checkpoint(stop)?;
        let call = calls
            .iter()
            .find(|call| call.call_id == relation.call_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let key =
            library_key(call)?.ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        let receiver_proof = if relation.relation == W6_EMBED_RELATION {
            Some(
                call.receiver_proof
                    .as_ref()
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?,
            )
        } else {
            None
        };
        matches.push(SourceLibraryMatch {
            call_id: call.call_id.clone(),
            library_name: key.major,
            caller_proposal_id: call.caller_proposal_id.clone(),
            minor: key.minor,
            receiver_binding_id: receiver_proof.map(|proof| proof.binding_id.clone()),
            initializer_call_fact_id: receiver_proof
                .map(|proof| proof.initializer_call_fact_id.clone()),
            initializer_source_handle_id: receiver_proof
                .map(|proof| proof.initializer_source_handle_id),
            initializer_evidence_id: receiver_proof.map(|proof| proof.initializer_evidence_id),
            relation_proposal_id: relation.proposal_id.clone(),
            rule_id: rule_id(relation.rule_id.as_str())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?,
            relation: relation.relation,
        });
    }

    let relation_families = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|definition| definition.relation())
        .collect::<BTreeSet<_>>();
    // Group all matching relations by complete LibStub major before building one
    // semantic entity. Entity identity and support cannot depend on the first call.
    let mut entity_proposals = Vec::<GraphEntityProposal>::new();
    let mut relation_proposals = Vec::<GraphRelationProposal>::new();
    let mut relation_by_proposal = BTreeMap::<String, &W6Relation>::new();
    for relation in &pending {
        if relation_by_proposal
            .insert(relation.proposal_id.clone(), relation)
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }
    let mut entity_support = BTreeMap::<String, W6LibraryEntitySupport>::new();
    for m in &matches {
        checkpoint(stop)?;
        let relation = relation_by_proposal
            .get(&m.relation_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        include_library_support(&mut entity_support, &m.library_name, relation)?;
    }
    let mut entity_ids = BTreeMap::<String, String>::new();
    for (major, support) in entity_support {
        checkpoint(stop)?;
        let entity_id = library_entity_proposal_id_for_profile(
            input.context.context_id(),
            &major,
            recognition_profile,
        )?;
        if entity_ids
            .insert(major.clone(), entity_id.clone())
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        entity_proposals.push(support.proposal(entity_id, &major)?);
    }

    // Relations retain call-specific confidence and evidence even when their
    // semantic target entity aggregates corroborating support from other calls.
    for m in &matches {
        checkpoint(stop)?;
        let relation = relation_by_proposal
            .get(&m.relation_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let caller = caller_nodes
            .get(&m.caller_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let entity_id = entity_ids
            .get(&m.library_name)
            .cloned()
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        relation_proposals.push(
            GraphRelationProposal::new(
                m.relation_proposal_id.as_str(),
                m.relation,
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(caller.clone()),
                    target: GraphProposalEndpoint::Proposed(entity_id.into_boxed_str()),
                    confidence: relation.confidence,
                    source_handle_ids: relation.source_handle_ids.clone(),
                    evidence_ids: relation.evidence_ids.clone(),
                    coverage_ids: relation.coverage_ids.clone(),
                },
            )
            .map_err(graph_error)?,
        );
    }
    let projectable = !matches.is_empty();
    let graph_coverage = relation_families
        .into_iter()
        .map(|relation| {
            let (state, blocker) = if W6_LIBRARY_RELATIONS
                .iter()
                .any(|declared| relation_id_for(declared) == Some(relation))
            {
                (
                    GraphCoverageState::Partial,
                    if projectable {
                        "lua_library.exact_reviewed_libstub_structure_only"
                    } else {
                        "lua_library.no_exact_reviewed_libstub_call_observed"
                    },
                )
            } else if aceaddon_local_result_profile
                && relation_id_for(W6_EMBED_RELATION) == Some(relation)
            {
                (
                    GraphCoverageState::Partial,
                    "lua_library.aceaddon_local_result_profile_only",
                )
            } else if W6_DEFERRED_LIBRARY_RELATIONS
                .iter()
                .any(|declared| relation_id_for(declared) == Some(relation))
            {
                (
                    GraphCoverageState::NotEvaluated,
                    "lua_library.embed_convention_deferred_without_universal_donor",
                )
            } else {
                (
                    GraphCoverageState::NotEvaluated,
                    "lua_library.relation_owned_by_other_producer",
                )
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        SOURCE_STATE_LIBRARY_PARTITION,
        entity_proposals,
        relation_proposals,
    )
    .map_err(graph_error)?;
    checkpoint(stop)?;
    Ok(SourceLibraryProposals {
        batch,
        coverage: graph_coverage,
        recognition: SourceLibraryRecognition {
            profile: recognition_profile,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            matches,
        },
    })
}

fn relation_for_rule(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        W6_REQUIRE_RULE => Some(W6_REQUIRE_RELATION),
        W6_NEW_RULE => Some(W6_NEW_RELATION),
        W6_ACEADDON_EMBED_RULE => Some(W6_EMBED_RELATION),
        _ => None,
    }
}

fn rule_id(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        W6_REQUIRE_RULE => Some(W6_REQUIRE_RULE),
        W6_NEW_RULE => Some(W6_NEW_RULE),
        W6_ACEADDON_EMBED_RULE => Some(W6_ACEADDON_EMBED_RULE),
        _ => None,
    }
}

// A LibStub major is the complete plain string key, including hyphens. It never
// carries a repository, a URL, or a license.
fn valid_library_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= W6_MAX_LIBRARY_KEY_BYTES
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

// Retain the exact LibStub major as one identity. Hyphens are ordinary major-key
// bytes; they do not divide a major from the independent minor argument.
fn library_major_key(major: &str) -> RecognizerResult<W6LibraryKey> {
    if !valid_library_name(major) {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(W6LibraryKey {
        major: major.to_owned(),
        minor: None,
    })
}

// LibStub's pinned NewLibrary implementation extracts the first digit run from
// a string minor before numeric comparison. Keep this optional and independent of
// the major; numeric/reference literals remain absent when Emmy has not typed them.
// Normalizes one exact call into a library key when the reviewed structure is
// exact. An unresolved callee, a dynamic key, an over-long argument list, or a
// colon form yields no key at all rather than a guessed identity.
fn library_key(call: &W6Call) -> RecognizerResult<Option<W6LibraryKey>> {
    let Some(callable_key) = call.callable_key.as_deref() else {
        return Ok(None);
    };
    if call.argument_count == 0 || call.argument_count > W6_MAX_ARGUMENTS_RETAINED {
        return Ok(None);
    }
    let major_is_string = call.argument_0_kind == Some("string")
        || (call.argument_0_kind == Some("reference")
            && call.argument_0_local_constant_kind == Some("string"));
    let aceaddon_library_is_string = call.argument_1_kind == Some("string")
        || (call.argument_1_kind == Some("reference")
            && call.argument_1_local_constant_kind == Some("string"));
    let (major, minor) = match callable_key {
        W6_LIBSTUB_CALLABLE if !call.colon_call && major_is_string => {
            (call.argument_0_value.as_deref(), None)
        }
        W6_GET_LIBRARY_CALLABLE | W6_NEW_LIBRARY_CALLABLE if call.colon_call && major_is_string => {
            let minor = (callable_key == W6_NEW_LIBRARY_CALLABLE
                && call.argument_1_kind == Some("reference")
                && call.argument_1_local_constant_kind == Some("integer"))
            .then_some(call.argument_1_integer)
            .flatten();
            (call.argument_0_value.as_deref(), minor)
        }
        W6_ACEADDON_EMBED_CALLABLE
            if call.colon_call
                && call.argument_0_kind == Some("reference")
                && aceaddon_library_is_string
                && call.callable_target_selected == Some(true)
                && call.receiver_proof.as_ref().is_some_and(|proof| {
                    proof.result_slot == 0
                        && !proof.binding_id.is_empty()
                        && !proof.initializer_call_fact_id.is_empty()
                        && proof.initializer_callable_key.as_deref()
                            == Some(W6_NEW_LIBRARY_CALLABLE)
                        && proof.initializer_major.as_deref() == Some(W6_ACEADDON_MAJOR)
                        && proof.initializer_target_selected
                }) =>
        {
            (call.argument_1_value.as_deref(), None)
        }
        _ => (None, None),
    };
    let Some(major) = major else {
        return Ok(None);
    };
    let mut key = library_major_key(major)?;
    key.minor = minor;
    Ok(Some(key))
}

fn build_library_fact(
    input: &SourceLibraryInput<'_>,
    call: &W6Call,
    fact_limits: RecognizerFactLimits,
) -> RecognizerResult<RecognizerFact> {
    let mut fields = BTreeMap::from([
        (
            "call_id".into(),
            RecognizerFactValue::Reference(call.call_id.clone().into_boxed_str()),
        ),
        (
            "caller".into(),
            RecognizerFactValue::Reference(call.caller_proposal_id.clone().into_boxed_str()),
        ),
        (
            "argument_count".into(),
            RecognizerFactValue::Integer(
                i64::try_from(call.argument_count)
                    .map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))?,
            ),
        ),
        (
            "colon_call".into(),
            RecognizerFactValue::Boolean(call.colon_call),
        ),
        ("exact".into(), RecognizerFactValue::Boolean(call.exact)),
    ]);
    if let Some(key) = &call.callable_key {
        fields.insert(
            "callable_key".into(),
            RecognizerFactValue::String(key.clone().into_boxed_str()),
        );
    }
    if let Some(selected) = call.callable_target_selected {
        fields.insert(
            "callable_target_selected".into(),
            RecognizerFactValue::Boolean(selected),
        );
    }
    if let Some(proof) = &call.receiver_proof {
        fields.insert(
            "receiver_binding_id".into(),
            RecognizerFactValue::Reference(proof.binding_id.clone().into_boxed_str()),
        );
        fields.insert(
            "receiver_result_slot".into(),
            RecognizerFactValue::Integer(i64::from(proof.result_slot)),
        );
        fields.insert(
            "receiver_initializer_call_fact_id".into(),
            RecognizerFactValue::Reference(proof.initializer_call_fact_id.clone().into_boxed_str()),
        );
        if let Some(key) = &proof.initializer_callable_key {
            fields.insert(
                "receiver_initializer_callable_key".into(),
                RecognizerFactValue::String(key.clone().into_boxed_str()),
            );
        }
        if let Some(major) = &proof.initializer_major {
            fields.insert(
                "receiver_initializer_major".into(),
                RecognizerFactValue::String(major.clone().into_boxed_str()),
            );
        }
        fields.insert(
            "receiver_initializer_target_selected".into(),
            RecognizerFactValue::Boolean(proof.initializer_target_selected),
        );
    }
    for (index, kind) in [
        ("0", call.argument_0_kind),
        ("1", call.argument_1_kind),
        ("2", call.argument_2_kind),
    ] {
        if let Some(kind) = kind {
            fields.insert(
                format!("argument_{index}_kind").into_boxed_str(),
                RecognizerFactValue::String(kind.into()),
            );
        }
        let local_constant_kind = match index {
            "0" => call.argument_0_local_constant_kind,
            "1" => call.argument_1_local_constant_kind,
            _ => None,
        };
        if let Some(kind) = local_constant_kind {
            fields.insert(
                format!("argument_{index}_local_constant_kind").into_boxed_str(),
                RecognizerFactValue::String(kind.into()),
            );
        }
        let reference = match index {
            "0" => call.argument_0_key.as_ref(),
            "1" => call.argument_1_key.as_ref(),
            _ => None,
        };
        if let Some(reference) = reference {
            fields.insert(
                format!("argument_{index}_key").into_boxed_str(),
                RecognizerFactValue::String(reference.clone().into_boxed_str()),
            );
        }
        let value = match index {
            "0" => call.argument_0_value.as_ref(),
            "1" => call.argument_1_value.as_ref(),
            _ => None,
        };
        if let Some(value) = value {
            fields.insert(
                format!("argument_{index}_value").into_boxed_str(),
                RecognizerFactValue::String(value.clone().into_boxed_str()),
            );
        }
    }
    if let Some(value) = call.argument_1_integer {
        fields.insert(
            "argument_1_integer".into(),
            RecognizerFactValue::Integer(value),
        );
    }
    // The library identity is the complete exact major. Minor is a separate axis
    // and is retained only when the owner facts provide an exact typed value.
    if let Ok(Some(key)) = library_key(call) {
        fields.insert(
            "library_name".into(),
            RecognizerFactValue::String(key.major.into_boxed_str()),
        );
        if let Some(minor) = key.minor {
            fields.insert("library_minor".into(), RecognizerFactValue::Integer(minor));
        }
    }
    let mut source_handle_ids = vec![call.handle];
    let mut evidence_ids = vec![call.evidence];
    if let Some(proof) = &call.receiver_proof {
        source_handle_ids.push(proof.initializer_source_handle_id);
        evidence_ids.push(proof.initializer_evidence_id);
    }
    source_handle_ids.sort_unstable();
    evidence_ids.sort_unstable();
    if source_handle_ids.windows(2).any(|pair| pair[0] == pair[1])
        || evidence_ids.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
    }
    RecognizerFact::new(
        input.context.context_id(),
        RecognizerFactInput {
            kind: "lua_call".into(),
            partition_id: W6_FACT_PARTITION.into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                call.caller_proposal_id.clone(),
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: if input.report.local_call_result_profile()
                == Some(FunctionCallEvidenceProfile::AceAddonLocalResultV1)
            {
                W6_LOCAL_RESULT_FACT_PROFILE.into()
            } else {
                W6_FACT_PROFILE.into()
            },
            confidence: if call.exact {
                GraphConfidence::Derived
            } else {
                GraphConfidence::Possible
            },
            fields,
            source_handle_ids,
            evidence_ids,
        },
        fact_limits,
    )
}

// The source-load registry declares the library relations as UsesApi. This map is
// the single place that binds a pack relation id to that registry kind, so an
// unregistered id fails loudly instead of being silently aliased.
fn relation_id_for(relation: &str) -> Option<GraphRelationKind> {
    match relation {
        W6_REQUIRE_RELATION | W6_NEW_RELATION | W6_EMBED_RELATION => {
            Some(GraphRelationKind::UsesApi)
        }
        _ => None,
    }
}

fn validate_call_support(
    input: &SourceLibraryInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    call: &wow_emmy::function_calls::SourceCallFact,
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
    let digest = call
        .content_digest()
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if handle.handle_id() != handle_id
        || handle.path().as_str() != call.path()
        || handle.span() != call.call_span()
        || handle.content_digest() != &digest
        || handle.project_generation() != input.context.project_generation()
        || handle.reference_generation() != Some(input.context.reference_generation())
        || evidence.evidence_id() != evidence_id
        || evidence.context_id() != input.context.context_id()
        || evidence.source_handle_ids() != [handle_id]
        || evidence.provenance() != ProvenanceClass::ProjectSource
        || evidence.confidence() != EvidenceConfidence::Proven
        || evidence.claim_scope() != ClaimScope::SourceObservation
        || !evidence.derivation_input_ids().is_empty()
        || !evidence.coverage_refs().is_empty()
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn aceaddon_embed_rule() -> RecognizerRule {
    RecognizerRule {
        rule_id: W6_ACEADDON_EMBED_RULE.into(),
        version: W6_ACEADDON_EMBED_RULE_VERSION,
        required_capabilities: vec!["emmy.fact.calls".into()],
        scope: "function".into(),
        clauses: vec![
            RecognizerClause::Fact {
                alias: "call".into(),
                kind: "lua_call".into(),
            },
            RecognizerClause::FieldEq {
                field: "call.callable_key".into(),
                value: crate::RecognizerPackLiteral::String(W6_ACEADDON_EMBED_CALLABLE.into()),
            },
            RecognizerClause::FieldEq {
                field: "call.colon_call".into(),
                value: crate::RecognizerPackLiteral::Boolean(true),
            },
            RecognizerClause::FieldEq {
                field: "call.callable_target_selected".into(),
                value: crate::RecognizerPackLiteral::Boolean(true),
            },
            RecognizerClause::FieldEq {
                field: "call.receiver_result_slot".into(),
                value: crate::RecognizerPackLiteral::Integer(0),
            },
            RecognizerClause::FieldEq {
                field: "call.receiver_initializer_callable_key".into(),
                value: crate::RecognizerPackLiteral::String(W6_NEW_LIBRARY_CALLABLE.into()),
            },
            RecognizerClause::FieldEq {
                field: "call.receiver_initializer_major".into(),
                value: crate::RecognizerPackLiteral::String(W6_ACEADDON_MAJOR.into()),
            },
            RecognizerClause::FieldEq {
                field: "call.receiver_initializer_target_selected".into(),
                value: crate::RecognizerPackLiteral::Boolean(true),
            },
            RecognizerClause::FieldEq {
                field: "call.argument_0_kind".into(),
                value: crate::RecognizerPackLiteral::String("reference".into()),
            },
            RecognizerClause::FieldIn {
                field: "call.argument_1_kind".into(),
                values: vec![
                    crate::RecognizerPackLiteral::String("string".into()),
                    crate::RecognizerPackLiteral::String("reference".into()),
                ],
            },
            RecognizerClause::FieldEq {
                field: "call.exact".into(),
                value: crate::RecognizerPackLiteral::Boolean(true),
            },
        ],
        captures: Vec::new(),
        outputs: vec![RecognizerOutput::RelationAssertion {
            output_id: "aceaddon_embed_library".into(),
            relation_kind_id: W6_EMBED_RELATION.into(),
            source: "call.caller".into(),
            target: "call.library_name".into(),
            confidence: RecognizerOutputConfidence::Derived,
        }],
        positive_fixture_ids: vec!["RECOG-LIB-ACEADDON-001".into()],
        near_negative_fixture_ids: vec![
            "RECOG-LIB-ACEADDON-002".into(),
            "RECOG-LIB-ACEADDON-003".into(),
        ],
        partial_fixture_ids: vec!["RECOG-LIB-ACEADDON-004".into()],
        mutation_fixture_ids: vec!["RECOG-LIB-ACEADDON-005".into()],
    }
}

fn library_pack(
    registry_bundle_id: &str,
    aceaddon_local_result_profile: bool,
) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let mut document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: if aceaddon_local_result_profile {
                "wow-calibration-aceaddon-local-result".into()
            } else {
                "wow-core-lua-library".into()
            },
            version: if aceaddon_local_result_profile {
                "1".into()
            } else {
                "4".into()
            },
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: if aceaddon_local_result_profile {
                W6_LOCAL_RESULT_FACT_PROFILE.into()
            } else {
                W6_FACT_PROFILE.into()
            },
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: if aceaddon_local_result_profile {
                "wow-recognizers-aceaddon-local-result-1".into()
            } else {
                "wow-recognizers-w11-library-4".into()
            },
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 4,
                max_clauses_per_rule: 12,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 65_536,
                max_proposals_per_rule_partition: 65_536,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![
                RecognizerRule {
                    rule_id: W6_REQUIRE_RULE.into(),
                    version: W6_RULE_VERSION,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "call".into(),
                            kind: "lua_call".into(),
                        },
                        RecognizerClause::AnyOf {
                            clauses: vec![
                                RecognizerClause::AllOf {
                                    clauses: vec![
                                        RecognizerClause::FieldEq {
                                            field: "call.callable_key".into(),
                                            value: crate::RecognizerPackLiteral::String(
                                                W6_LIBSTUB_CALLABLE.into(),
                                            ),
                                        },
                                        RecognizerClause::FieldEq {
                                            field: "call.colon_call".into(),
                                            value: crate::RecognizerPackLiteral::Boolean(false),
                                        },
                                    ],
                                },
                                RecognizerClause::AllOf {
                                    clauses: vec![
                                        RecognizerClause::FieldEq {
                                            field: "call.callable_key".into(),
                                            value: crate::RecognizerPackLiteral::String(
                                                W6_GET_LIBRARY_CALLABLE.into(),
                                            ),
                                        },
                                        RecognizerClause::FieldEq {
                                            field: "call.colon_call".into(),
                                            value: crate::RecognizerPackLiteral::Boolean(true),
                                        },
                                    ],
                                },
                            ],
                        },
                        RecognizerClause::FieldIn {
                            field: "call.argument_0_kind".into(),
                            values: vec![
                                crate::RecognizerPackLiteral::String("string".into()),
                                crate::RecognizerPackLiteral::String("reference".into()),
                            ],
                        },
                        RecognizerClause::FieldEq {
                            field: "call.exact".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "libstub_require".into(),
                        relation_kind_id: W6_REQUIRE_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.library_name".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-LIB-001".into()],
                    near_negative_fixture_ids: vec!["RECOG-LIB-004".into()],
                    partial_fixture_ids: vec!["RECOG-LIB-009".into()],
                    mutation_fixture_ids: vec!["RECOG-LIB-007".into()],
                },
                RecognizerRule {
                    rule_id: W6_NEW_RULE.into(),
                    version: W6_RULE_VERSION,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "call".into(),
                            kind: "lua_call".into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W6_NEW_LIBRARY_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                        RecognizerClause::FieldIn {
                            field: "call.argument_0_kind".into(),
                            values: vec![
                                crate::RecognizerPackLiteral::String("string".into()),
                                crate::RecognizerPackLiteral::String("reference".into()),
                            ],
                        },
                        RecognizerClause::FieldEq {
                            field: "call.exact".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "libstub_new".into(),
                        relation_kind_id: W6_NEW_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.library_name".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-LIB-002".into()],
                    near_negative_fixture_ids: vec!["RECOG-LIB-005".into()],
                    partial_fixture_ids: vec!["RECOG-LIB-009".into()],
                    mutation_fixture_ids: vec!["RECOG-LIB-007".into()],
                },
            ],
        },
    };
    if aceaddon_local_result_profile {
        document.pack.rules.push(aceaddon_embed_rule());
    }
    document
        .pack
        .rules
        .sort_by(|a, b| (&a.rule_id, a.version).cmp(&(&b.rule_id, b.version)));
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}
// ===== END WORKER 6: library family =====

#[cfg(test)]
mod w6_library_tests {
    use super::*;

    fn call(
        callable_key: &str,
        colon_call: bool,
        argument_0_value: &str,
        argument_1_kind: Option<&'static str>,
        argument_1_value: Option<&str>,
    ) -> W6Call {
        W6Call {
            call_id: "call-id".into(),
            caller_proposal_id: "caller-id".into(),
            callable_key: Some(callable_key.into()),
            colon_call,
            argument_count: if argument_1_kind.is_some() { 2 } else { 1 },
            exact: true,
            callable_target_selected: None,
            receiver_proof: None,
            handle: StableHandleId::from_hash([1; 32]),
            evidence: EvidenceId::from_hash([2; 32]),
            argument_0_kind: Some("string"),
            argument_0_key: None,
            argument_0_value: Some(argument_0_value.into()),
            argument_0_local_constant_kind: None,
            argument_1_kind,
            argument_1_key: None,
            argument_1_value: argument_1_value.map(str::to_owned),
            argument_1_local_constant_kind: None,
            argument_1_integer: None,
            argument_2_kind: None,
        }
    }

    fn relation(
        proposal_id: &str,
        call_id: &str,
        confidence: GraphConfidence,
        handle: u8,
        evidence: u8,
        coverage: u8,
    ) -> W6Relation {
        W6Relation {
            proposal_id: proposal_id.into(),
            rule_id: W6_REQUIRE_RULE.into(),
            relation: W6_REQUIRE_RELATION,
            call_id: call_id.into(),
            confidence,
            source_handle_ids: vec![StableHandleId::from_hash([handle; 32])],
            evidence_ids: vec![EvidenceId::from_hash([evidence; 32])],
            coverage_ids: vec![wow_core::CoverageId::from_hash([coverage; 32])],
        }
    }

    #[test]
    fn aceaddon_target_selection_rejects_matching_non_executable_sentinels() {
        let signature_not_captured = SourceCallTarget::SignatureNotCaptured;
        assert!(!selected_source_call_target_matches(
            &signature_not_captured,
            &signature_not_captured,
        ));

        let indeterminate = SourceCallTarget::Indeterminate;
        assert!(!selected_source_call_target_matches(
            &indeterminate,
            &indeterminate,
        ));
    }

    #[test]
    fn libstub_major_keeps_hyphens_and_minor_stays_separate() -> RecognizerResult<()> {
        let require = call(W6_LIBSTUB_CALLABLE, false, "AceAddon-3.0", None, None);
        let key = library_key(&require)?
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        assert_eq!(key.major, "AceAddon-3.0");
        assert_eq!(key.minor, None);

        let new_with_untyped_numeric = call(
            W6_NEW_LIBRARY_CALLABLE,
            true,
            "AceAddon-3.0",
            Some("dynamic"),
            None,
        );
        let key = library_key(&new_with_untyped_numeric)?
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        assert_eq!(key.major, "AceAddon-3.0");
        assert_eq!(key.minor, None);

        let new_with_string_minor = call(
            W6_NEW_LIBRARY_CALLABLE,
            true,
            "AceAddon-3.0",
            Some("string"),
            Some("13"),
        );
        let key = library_key(&new_with_string_minor)?
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterFactMismatch))?;
        assert_eq!(key.major, "AceAddon-3.0");
        assert_eq!(key.minor, None);
        Ok(())
    }

    #[test]
    fn one_library_entity_merges_bounded_support_conservatively() -> RecognizerResult<()> {
        let exact = relation("relation-a", "call-a", GraphConfidence::Derived, 1, 2, 3);
        let possible = relation("relation-b", "call-b", GraphConfidence::Possible, 4, 5, 3);
        let another_major = relation("relation-c", "call-c", GraphConfidence::Derived, 6, 7, 8);
        let mut forward = BTreeMap::<String, W6LibraryEntitySupport>::new();
        include_library_support(&mut forward, "AceAddon-3.0", &exact)?;
        include_library_support(&mut forward, "AceAddon-3.0", &possible)?;
        include_library_support(&mut forward, "AceAddon-2.0", &another_major)?;
        let mut reverse = BTreeMap::<String, W6LibraryEntitySupport>::new();
        include_library_support(&mut reverse, "AceAddon-2.0", &another_major)?;
        include_library_support(&mut reverse, "AceAddon-3.0", &possible)?;
        include_library_support(&mut reverse, "AceAddon-3.0", &exact)?;
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 2);
        let support = forward
            .get("AceAddon-3.0")
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        assert_eq!(support.source_handle_ids.len(), 2);
        assert_eq!(support.evidence_ids.len(), 2);
        assert_eq!(support.coverage_ids.len(), 1);
        assert_eq!(support.confidence, GraphConfidence::Possible);
        let proposal_id = library_entity_proposal_id(
            wow_core::GenerationContextId::from_hash([1; 32]),
            "AceAddon-3.0",
        )?;
        let proposal = support.clone().proposal(proposal_id, "AceAddon-3.0")?;
        assert_eq!(
            proposal.semantic_key().get("library"),
            Some(&GraphProposalValue::String("AceAddon-3.0".into()))
        );
        assert_eq!(proposal.confidence(), GraphConfidence::Possible);
        assert_eq!(proposal.source_handle_ids().len(), 2);
        assert_eq!(proposal.evidence_ids().len(), 2);
        Ok(())
    }

    #[test]
    fn library_entity_identity_uses_context_and_complete_major_not_first_relation()
    -> RecognizerResult<()> {
        let first_context = wow_core::GenerationContextId::from_hash([1; 32]);
        let second_context = wow_core::GenerationContextId::from_hash([2; 32]);
        let first_witness = library_entity_proposal_id(first_context, "AceAddon-3.0")?;
        let surviving_witness = library_entity_proposal_id(first_context, "AceAddon-3.0")?;
        let different_major = library_entity_proposal_id(first_context, "AceAddon-2.0")?;
        let different_context = library_entity_proposal_id(second_context, "AceAddon-3.0")?;

        assert_eq!(first_witness, surviving_witness);
        assert_ne!(first_witness, different_major);
        assert_ne!(first_witness, different_context);
        Ok(())
    }

    #[test]
    fn donor_local_major_and_minor_reach_the_w6_consumer() -> Result<(), Box<dyn std::error::Error>>
    {
        use std::str::FromStr;
        use wow_core::{
            EvidenceConfidence, GenerationContextBuilder, ProducerId, ProfileId,
            ProfileIdentityBuilder, ProfileKind, ProjectGenerationId, ReferenceGenerationId,
            SourceKind, SourceOriginKind, ToolVersion,
        };
        use wow_emmy::{
            LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
            backend_identity_from_report,
            function_calls::FunctionCallBudget,
            global_access::GlobalAccessBudget,
            references::{
                MemberCallSessionQueryProfile, analyze_member_call_session_with_callable_queries,
            },
        };

        let stop = AtomicBool::new(false);
        let source = concat!(
            "LibStub = {}\n",
            "function LibStub.NewLibrary(major, minor) return {}, nil end\n",
            "function RegisterLibrary()\n",
            "  local MAJOR, MINOR = \"AceAddon-3.0\", 13\n",
            "  local AceAddon, oldminor = LibStub:NewLibrary(MAJOR, MINOR)\n",
            "end\n",
        );
        let compatibility_report = br#"{"analysis_crate":{"manifest_path":"crates/emmylua_code_analysis/Cargo.toml","name":"emmylua_code_analysis","version":"0.25.1"},"compatibility":{"missing_symbols":[],"required_symbols":[],"status":"compatible"},"producer_version":1,"report_sha256":"sha256:e56096cd332cd4d35105772f6084c0c4116f885dff237097c38706b249f8f343","schema":"wow-dev-framework/emmylua-compatibility-report","schema_version":1,"source":{"network_checked":false,"relation":"unverified_current","revision":"aaaca68425d9362876228649b0b8d92f07654daa","tree":"9175c01384e650b9a5bd64da69c36f47dbeaaf67"},"surface":{"files":[],"surface_sha256":"sha256:3333333333333333333333333333333333333333333333333333333333333333","symbols":[]}}"#;
        let backend = backend_identity_from_report(compatibility_report)?;
        let main = LuaWorkspaceSnapshot::build(
            backend,
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new("main/library.lua", source)],
            LuaWorkspaceLimits::new(4, 4096, 64 * 1024, 64 * 1024)?,
        )?;
        let call_budget = FunctionCallBudget::default()
            .with_global_access_budget(GlobalAccessBudget::new(2_000_000, 8 * 1024 * 1024)?);
        let callable_queries = vec!["LibStub.NewLibrary".to_owned()];
        let report = analyze_member_call_session_with_callable_queries(
            &main,
            &[],
            MemberCallSessionQueryProfile::new(&[], &callable_queries)
                .with_function_call_budget(call_budget),
            true,
            &stop,
        )?
        .function_calls
        .ok_or("profile10 function-call report")?;
        assert_eq!(
            serde_json::to_value(&report)?["profile"],
            "wow-emmy/function-call-facts/10"
        );
        let call = report
            .calls()
            .iter()
            .find(|call| call.resolved_callable_key() == Some(W6_NEW_LIBRARY_CALLABLE))
            .ok_or("resolved NewLibrary donor-shaped call")?;
        assert_eq!(call.arguments().len(), 2);
        assert_eq!(
            call.arguments()[0]
                .local_constant()
                .map(|value| value.name()),
            Some("MAJOR")
        );
        assert_eq!(
            call.arguments()[1]
                .local_constant()
                .map(|value| value.name()),
            Some("MINOR")
        );

        let schema = wow_core::SchemaVersionEntry::new(
            wow_core::SchemaId::from_str("schema:wow:w6-local-constant-fixture")?,
            ToolVersion::from_str("1.0.0")?,
        );
        let profile = ProfileIdentityBuilder::new(
            ProfileId::from_str("profile:fixture:w6-local-constants")?,
            ProfileKind::Fixture,
            "retail",
            120_100,
            SourceKind::SyntheticFixture,
            "fixture:w6-local-constants-v1",
            ContentDigest::<wow_core::SourceLogicalSnapshot>::from_bytes([9; 32]),
        )
        .schema_versions(vec![schema.clone()])
        .fixture_scope("w6 local constant consumer")
        .build()?;
        let project_generation = ProjectGenerationId::derive(&"w6-local-constant-project")?;
        let reference_generation = ReferenceGenerationId::derive(&"w6-local-constant-reference")?;
        let context = GenerationContextBuilder::new(profile, reference_generation)
            .project_generation(project_generation)
            .schema_versions(vec![schema])
            .build()?;
        let call_digest = call
            .content_digest()
            .parse::<ContentDigest<SourceContent>>()?;
        let call_handle = wow_core::SourceHandleBuilder::new(
            SourceOriginKind::Fixture,
            "fixture:w6-local-constants",
            "fixture:w6-local-constants-v1",
            call.path(),
            call.call_span(),
            call_digest,
        )
        .reference_generation(reference_generation)
        .project_generation(project_generation)
        .build()?;
        let evidence = EvidenceRecord::new(
            context.context_id(),
            ProvenanceClass::ProjectSource,
            EvidenceConfidence::Proven,
            ClaimScope::SourceObservation,
            ProducerId::from_str("wow.project")?,
            ToolVersion::from_str("1.0.0")?,
            vec![call_handle.handle_id()],
            Vec::new(),
            Vec::new(),
        )?;
        let call_handle_id = call_handle.handle_id();
        let evidence_id = evidence.evidence_id();
        let source_handles = BTreeMap::from([(call_handle_id, call_handle)]);
        let evidence_records = BTreeMap::from([(evidence_id, evidence)]);

        const SOURCE_PARTITION: &str = "wow-project.source-graph";
        let registry = wow_graph::GraphRegistryBundle::build(
            "graph-registry:w6-local-constants",
            "1.0.0",
            vec![
                wow_graph::GraphEntityKindDefinition::new(
                    "lua_source_function",
                    vec!["project".into()],
                    vec!["document".into(), "function".into()],
                    vec![GraphConfidence::Derived, GraphConfidence::Possible],
                )?,
                wow_graph::GraphEntityKindDefinition::new(
                    W6_LIBRARY_ENTITY,
                    vec!["project".into()],
                    vec!["library".into()],
                    vec![GraphConfidence::Derived, GraphConfidence::Possible],
                )?,
            ],
            [W6_REQUIRE_RELATION, W6_NEW_RELATION, W6_EMBED_RELATION]
                .into_iter()
                .map(|relation| {
                    wow_graph::GraphRelationKindDefinition::new(
                        relation,
                        GraphRelationKind::UsesApi,
                        vec!["lua_source_function".into()],
                        vec![W6_LIBRARY_ENTITY.into()],
                        vec![GraphConfidence::Derived, GraphConfidence::Possible],
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
        )?;
        let universe = wow_graph::GraphUniverseId::new("project:w6-local-constants")?;
        let generation =
            wow_graph::GraphGenerationId::new("project-generation:w6-local-constants:1")?;
        let limits = wow_graph::GraphLimits::default();
        let foundation = wow_graph::GraphSnapshot::build(
            universe.clone(),
            generation.clone(),
            limits,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )?;
        let owner =
            GraphPartitionSnapshot::new(registry.clone(), foundation, context.context_id(), &stop)?;
        let mut source_entities = Vec::new();
        let mut function_records = Vec::new();
        for (index, function) in report.functions().iter().enumerate() {
            let proposal_id = format!("source-function-{index}");
            source_entities.push(GraphEntityProposal::new(
                proposal_id.as_str(),
                "lua_source_function",
                BTreeMap::from([
                    (
                        Box::<str>::from("document"),
                        GraphProposalValue::String(function.path().into()),
                    ),
                    (
                        Box::<str>::from("function"),
                        GraphProposalValue::String(function.fact_id().into()),
                    ),
                ]),
                GraphConfidence::Derived,
                vec![call_handle_id],
                vec![evidence_id],
                Vec::new(),
            )?);
            function_records.push((function.fact_id().to_owned(), proposal_id));
        }
        let function_proposals = function_records
            .iter()
            .map(|(function_id, proposal_id)| (function_id.as_str(), proposal_id.as_str()))
            .collect::<BTreeMap<_, _>>();
        let source_batch = GraphProposalBatch::build(
            registry.bundle_id(),
            registry.registry_digest(),
            universe,
            generation,
            context.context_id(),
            SOURCE_PARTITION,
            source_entities,
            Vec::new(),
        )?;
        let owner = owner
            .prepare_replacement(
                wow_graph::GraphPartitionReplacement {
                    expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
                    expected_partition_digest: None,
                    producer_version: "1.0.0".into(),
                    batch: source_batch,
                    coverage: Vec::new(),
                },
                &stop,
            )?
            .into_candidate();
        let function_call_report = &report;
        let function_call_support = report
            .calls()
            .iter()
            .map(|call| (call.fact_id(), (call_handle_id, evidence_id)))
            .collect::<BTreeMap<_, _>>();
        let recognized = recognize_source_library(
            SourceLibraryInput {
                owner: &owner,
                source_partition: SOURCE_PARTITION,
                report: function_call_report,
                context: &context,
                function_proposals,
                call_support: function_call_support,
                source_handles: &source_handles,
                evidence: &evidence_records,
            },
            &stop,
        )?;
        assert_eq!(recognized.recognition.matches().len(), 1);
        let library = &recognized.recognition.matches()[0];
        assert_eq!(library.rule_id, W6_NEW_RULE);
        assert_eq!(library.library_name, "AceAddon-3.0");
        assert_eq!(library.minor, Some(13));
        Ok(())
    }
}
