//! TOC state-slot crosswalks -> the existing state-read/state-write recognizers.
//! This adapter joins exact retained owner facts; it does not parse source.
#![allow(dead_code)]
use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry, RecognizerResult,
    StructuredObservation, StructuredObservationInput, run_recognizers,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    CanonicalResult, ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord,
    GenerationContext, ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use wow_emmy::function_calls::FunctionCallReport;
use wow_emmy::global_access::{GlobalAccessKind, GlobalAccessResolution};

pub const SOURCE_STATE_PARTITION: &str = "wow-recognizers.saved-variable-access";
pub const SOURCE_STATE_PROFILE: &str = "wow-recognizers/source-saved-variable-access/3";
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
        for hop in access.aliases() {
            checkpoint(stop)?;
            located_support(
                &input,
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
                from: endpoint(fact.caller_proposal_id)?,
                to: endpoint(fact.target_proposal_id)?,
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
        &graph,
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
        let source = input
            .owner
            .partition(input.source_partition)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let mut prerequisites = vec![
            fact.caller_proposal_id,
            fact.target_proposal_id,
            fact.root_proposal_id,
        ];
        prerequisites.sort();
        prerequisites.dedup();
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
            inputs: prerequisites
                .into_iter()
                .map(|id| wow_graph::GraphAssertionRef::Producer {
                    partition_id: source.partition_id().into(),
                    batch_id: source.batch().batch_id().into(),
                    assertion: wow_graph::GraphLocalAssertion {
                        kind: wow_graph::GraphAssertionKind::Entity,
                        proposal_id: id.into(),
                    },
                })
                .collect(),
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

// ===== BEGIN WORKER 6: library family =====

// Exact LibStub/GetLibrary/NewLibrary/embed structural call facts -> the declarative
// core library recognizer -> graph proposals. This producer consumes only
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
pub const SOURCE_STATE_LIBRARY_PROFILE: &str = "wow-recognizers/source-library/3";
const W6_FACT_PARTITION: &str = "wow-recognizers.lua-library-facts";
const W6_FACT_PROFILE: &str = "wow-recognizers-lua-library-facts-3";
const W6_MAX_CALLS: usize = 8192;
const W6_MAX_FUNCTIONS: usize = 8192;
const W6_MAX_ARGUMENTS_RETAINED: usize = 3;
const W6_MAX_LIBRARY_KEY_BYTES: usize = 256;
const W6_MAX_PROPOSALS: usize = 65_536;
const W6_REQUIRE_RULE: &str = "core.library.libstub_require";
const W6_NEW_RULE: &str = "core.library.libstub_new";
const W6_EMBED_RULE: &str = "core.library.embed";
const W6_LIBSTUB_CALLABLE: &str = "LibStub";
const W6_GET_LIBRARY_CALLABLE: &str = "LibStub.GetLibrary";
const W6_NEW_LIBRARY_CALLABLE: &str = "LibStub.NewLibrary";
const W6_EMBED_LIBRARY_CALLABLE: &str = "LibStub.EmbedLibrary";
// The one structural embed convention this pack declares, with the exact member
// order it fixes. Receiver, library and target must all be resolved before an embed
// relation is emitted; otherwise the outcome stays Possible or produces no match.

// Normative relation ids from RULE_FAMILIES.md section 6.
const W6_REQUIRE_RELATION: &str = "lua_requires_library";
const W6_NEW_RELATION: &str = "lua_declares_library";
const W6_EMBED_RELATION: &str = "lua_embeds_library";
const W6_LIBRARY_ENTITY: &str = "library";
const W6_LIBRARY_RELATIONS: [&str; 3] = [W6_REQUIRE_RELATION, W6_NEW_RELATION, W6_EMBED_RELATION];

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
    pub library_name: String,
    pub caller_proposal_id: String,
    pub version: Option<String>,
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
    handle: StableHandleId,
    evidence: EvidenceId,
    argument_0_kind: Option<&'static str>,
    argument_0_key: Option<String>,
    argument_0_value: Option<String>,
    argument_1_kind: Option<&'static str>,
    argument_1_key: Option<String>,
    argument_1_value: Option<String>,
    argument_2_kind: Option<&'static str>,
}

// One exact library key observed at one exact call site. The version string is
// exact retained evidence about the key, never a loaded revision or ownership.
struct W6LibraryKey {
    name: String,
    version: Option<String>,
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
        let mut argument_1_kind = None;
        let mut argument_1_key = None;
        let mut argument_1_value = None;
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
            let kind = match (argument.reference_key(), argument.literal()) {
                (Some(key), None) => {
                    if ordinal == 0 {
                        argument_0_key = Some(key.to_owned());
                    } else if ordinal == 1 {
                        argument_1_key = Some(key.to_owned());
                    }
                    "reference"
                }
                (None, Some(SourceCallLiteral::String(value))) => {
                    if value.len() > W6_MAX_LIBRARY_KEY_BYTES {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    if ordinal == 0 {
                        argument_0_value = Some(value.clone());
                    } else if ordinal == 1 {
                        argument_1_value = Some(value.clone());
                    }
                    "string"
                }
                (None, Some(SourceCallLiteral::Boolean(_))) => "boolean",
                (None, Some(SourceCallLiteral::Nil)) => "nil",
                (None, None) => {
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
        let mut normalized = W6Call {
            call_id: call.fact_id().to_owned(),
            caller_proposal_id: caller_proposal_id.to_owned(),
            callable_key: call.resolved_callable_key().map(str::to_owned),
            colon_call: call.is_colon_call(),
            argument_count: call.arguments().len(),
            exact,
            handle,
            evidence,
            argument_0_kind,
            argument_0_key,
            argument_0_value,
            argument_1_kind,
            argument_1_key,
            argument_1_value,
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
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W6_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: W6_FACT_PROFILE.into(),
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
    let pack = library_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut pending = Vec::<W6Relation>::new();
    for outcome in output.outcomes() {
        checkpoint(stop)?;
        if outcome.rule_version() != 1
            || !matches!(
                outcome.rule_id(),
                W6_REQUIRE_RULE | W6_NEW_RULE | W6_EMBED_RULE
            )
        {
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
        matches.push(SourceLibraryMatch {
            call_id: call.call_id.clone(),
            library_name: key.name,
            caller_proposal_id: call.caller_proposal_id.clone(),
            version: key.version,
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
    // Every retained match projects one library entity and one relation from the
    // exact resolving caller. The version is retained as literal evidence on the
    // match and is never promoted to a loaded revision or ownership claim.
    let mut entity_proposals = Vec::<GraphEntityProposal>::new();
    let mut relation_proposals = Vec::<GraphRelationProposal>::new();
    let mut entity_ids = BTreeMap::<String, String>::new();
    for m in &matches {
        checkpoint(stop)?;
        let relation = pending
            .iter()
            .find(|relation| relation.proposal_id == m.relation_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let caller = caller_nodes
            .get(&m.caller_proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let entity_id = if let Some(existing) = entity_ids.get(&m.library_name) {
            existing.clone()
        } else {
            let entity_id = format!("w6-library-{}", relation.proposal_id);
            entity_ids.insert(m.library_name.clone(), entity_id.clone());
            entity_proposals.push(
                GraphEntityProposal::new(
                    entity_id.clone(),
                    W6_LIBRARY_ENTITY,
                    BTreeMap::from([(
                        "library".into(),
                        GraphProposalValue::String(m.library_name.clone().into_boxed_str()),
                    )]),
                    relation.confidence,
                    relation.source_handle_ids.clone(),
                    relation.evidence_ids.clone(),
                    relation.coverage_ids.clone(),
                )
                .map_err(graph_error)?,
            );
            entity_id
        };
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
            profile: SOURCE_STATE_LIBRARY_PROFILE,
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
        W6_EMBED_RULE => Some(W6_EMBED_RELATION),
        _ => None,
    }
}

fn rule_id(rule_id: &str) -> Option<&'static str> {
    match rule_id {
        W6_REQUIRE_RULE => Some(W6_REQUIRE_RULE),
        W6_NEW_RULE => Some(W6_NEW_RULE),
        W6_EMBED_RULE => Some(W6_EMBED_RULE),
        _ => None,
    }
}

// A library identity is a plain name with no path, separator, or control data. It
// never carries a repository, a URL, or a license.
fn valid_library_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= W6_MAX_LIBRARY_KEY_BYTES
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

// Splits an exact reviewed library key. An exact split requires a nonempty name
// and a nonempty version; the version stays the retained literal string and is
// never treated as a loaded revision or an ownership claim.
fn split_library_key(key: &str) -> RecognizerResult<W6LibraryKey> {
    if key.is_empty() || key.len() > W6_MAX_LIBRARY_KEY_BYTES || key.trim() != key {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let Some((name, version)) = key.split_once('-') else {
        if !valid_library_name(key) {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        return Ok(W6LibraryKey {
            name: key.to_owned(),
            version: None,
        });
    };
    if !valid_library_name(name) || !valid_library_name(version) {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(W6LibraryKey {
        name: name.to_owned(),
        version: Some(version.to_owned()),
    })
}

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
    let literal = match callable_key {
        W6_LIBSTUB_CALLABLE if !call.colon_call && call.argument_0_kind == Some("string") => {
            call.argument_0_value.as_deref()
        }
        W6_GET_LIBRARY_CALLABLE | W6_NEW_LIBRARY_CALLABLE
            if call.colon_call && call.argument_0_kind == Some("string") =>
        {
            call.argument_0_value.as_deref()
        }
        W6_EMBED_LIBRARY_CALLABLE
            if !call.colon_call
                && call.argument_0_kind == Some("reference")
                && call.argument_1_kind == Some("string") =>
        {
            call.argument_1_value.as_deref()
        }
        _ => None,
    };
    literal.map(split_library_key).transpose()
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
    // The library identity is derived only from an exact reviewed structure. A
    // dynamic argument never produces a name or a version.
    if let Ok(Some(key)) = library_key(call) {
        fields.insert(
            "library_name".into(),
            RecognizerFactValue::String(key.name.into_boxed_str()),
        );
        if let Some(version) = key.version {
            fields.insert(
                "library_version".into(),
                RecognizerFactValue::String(version.into_boxed_str()),
            );
        }
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
            producer_version: W6_FACT_PROFILE.into(),
            confidence: if call.exact {
                GraphConfidence::Derived
            } else {
                GraphConfidence::Possible
            },
            fields,
            source_handle_ids: vec![call.handle],
            evidence_ids: vec![call.evidence],
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

fn library_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let mut document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-library".into(),
            version: "3".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W6_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-library-3".into(),
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
                    version: 1,
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
                        RecognizerClause::FieldEq {
                            field: "call.argument_0_kind".into(),
                            value: crate::RecognizerPackLiteral::String("string".into()),
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
                    positive_fixture_ids: vec!["RECOG-LIB-004".into()],
                    near_negative_fixture_ids: vec!["RECOG-LIB-005".into()],
                    partial_fixture_ids: vec!["RECOG-STATE-004".into()],
                    mutation_fixture_ids: vec!["RECOG-XML-004".into()],
                },
                RecognizerRule {
                    rule_id: W6_NEW_RULE.into(),
                    version: 1,
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
                        RecognizerClause::FieldEq {
                            field: "call.argument_0_kind".into(),
                            value: crate::RecognizerPackLiteral::String("string".into()),
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
                    positive_fixture_ids: vec!["RECOG-LIB-005".into()],
                    near_negative_fixture_ids: vec!["RECOG-LIB-005".into()],
                    partial_fixture_ids: vec!["RECOG-STATE-004".into()],
                    mutation_fixture_ids: vec!["RECOG-XML-004".into()],
                },
                RecognizerRule {
                    rule_id: W6_EMBED_RULE.into(),
                    version: 1,
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
                                W6_EMBED_LIBRARY_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(false),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.exact".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.argument_0_kind".into(),
                            value: crate::RecognizerPackLiteral::String("reference".into()),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.argument_1_kind".into(),
                            value: crate::RecognizerPackLiteral::String("string".into()),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "libstub_embed".into(),
                        relation_kind_id: W6_EMBED_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.library_name".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-TOC-003".into()],
                    near_negative_fixture_ids: vec!["RECOG-LIB-004".into()],
                    partial_fixture_ids: vec!["RECOG-STATE-004".into()],
                    mutation_fixture_ids: vec!["RECOG-XML-004".into()],
                },
            ],
        },
    };
    document
        .pack
        .rules
        .sort_by(|a, b| (&a.rule_id, a.version).cmp(&(&b.rule_id, b.version)));
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}
// ===== END WORKER 6: library family =====
