//! Project-owned XML handler facts -> the existing script-assignment recognizer.
//! This adapter verifies the exact graph/evidence crosswalk, not XML or Lua syntax.
#![allow(dead_code)]
mod assertions;
use crate::{
    ObservationFamily, ObservationOrigin, RecognitionCoverage, RecognitionCoverageState,
    RecognitionReport, RecognizerError, RecognizerErrorCode, RecognizerLimits, RecognizerRegistry,
    RecognizerResult, StructuredObservation, StructuredObservationInput, run_recognizers,
};
pub(crate) use assertions::ContextMetadata;
pub use assertions::{
    SOURCE_SCRIPT_ASSERTION_PROFILE, SourceScriptAssertionEndpoints, SourceScriptAssertionFact,
    SourceScriptAssertionInput, SourceScriptAssertionProposals, SourceScriptAssertionRecognition,
    recognize_source_script_assertions,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_core::{
    ClaimScope, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext, ProvenanceClass,
    SourceHandle, StableHandleId,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphCoverageRecord, GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProducerLookup, GraphProducerPartition, GraphProposalBatch,
    GraphProposalEndpoint, GraphProposalValue, GraphRelationKind, GraphRelationProposal,
    GraphRelationProposalInput, GraphSnapshot,
};

pub const SOURCE_SCRIPT_PARTITION: &str = "wow-recognizers.xml-script-bindings";
pub const SOURCE_SCRIPT_PROFILE: &str = "wow-recognizers/source-xml-scripts/2";
const MAX_BINDINGS: usize = 8192;
const XML_CONTEXT_ID_PREFIX: &str = "project-xml-lua-context:sha256:";
const EXACT_XML_SCRIPT_SITE: &str = "exact_xml_script_site";
const IMPLICIT_RECEIVER_NOT_EVALUATED: &str = "not_evaluated_unwrapped_source";
const RUNTIME_DISPATCH_NOT_EVALUATED: &str = "not_evaluated_static_load_evidence_only";

#[derive(Clone, Copy)]
pub struct SourceScriptSemanticContext<'a> {
    pub context_id: &'a str,
    pub script_site: &'a str,
    pub implicit_receiver: &'a str,
    pub runtime_dispatch: &'a str,
}

/// An immutable normalized crosswalk supplied by the source owner via service.
/// The project retains all lookup, inheritance, inline and skipped-site receipts.
pub struct SourceScriptFact<'a> {
    pub fact_id: &'a str,
    pub receiver_proposal_id: &'a str,
    pub handler_proposal_id: &'a str,
    pub semantic_context: Option<SourceScriptSemanticContext<'a>>,
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

struct ScriptData<'a> {
    owner: &'a GraphPartitionSnapshot,
    context: &'a GenerationContext,
    source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

enum ScriptFacts<'a, 'b> {
    Legacy(&'b [SourceScriptFact<'a>]),
    Assertions(&'b [SourceScriptAssertionFact<'a>]),
}
impl<'a> ScriptFacts<'a, '_> {
    fn len(&self) -> usize {
        match self {
            Self::Legacy(facts) => facts.len(),
            Self::Assertions(facts) => facts.len(),
        }
    }
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn fact(&self, index: usize) -> &SourceScriptFact<'a> {
        match self {
            Self::Legacy(facts) => &facts[index],
            Self::Assertions(facts) => &facts[index].fact,
        }
    }
}

enum ScriptEndpoints<'a, 'b> {
    Legacy(&'a GraphProducerPartition),
    Assertions {
        lookup: &'b GraphProducerLookup<'a>,
        scope: &'b GraphAssertionRecordScope,
        facts: &'b [SourceScriptAssertionFact<'a>],
    },
}
struct ScriptEndpoint<'a> {
    proposal: &'a GraphEntityProposal,
    node: Option<&'a GraphNodeId>,
}
impl<'a> ScriptEndpoints<'a, '_> {
    fn proposal(
        &self,
        index: usize,
        receiver: bool,
        proposal_id: &str,
        stop: &AtomicBool,
    ) -> RecognizerResult<ScriptEndpoint<'a>> {
        match self {
            Self::Legacy(partition) => Ok(ScriptEndpoint {
                proposal: partition
                    .batch()
                    .entity_proposal(proposal_id)
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?,
                node: None,
            }),
            Self::Assertions {
                lookup,
                scope,
                facts,
            } => {
                let endpoints = &facts[index].endpoints;
                let reference = if receiver {
                    &endpoints.receiver
                } else {
                    &endpoints.handler
                };
                let resolved =
                    crate::source_assertions::entity(lookup, scope, reference, proposal_id, stop)?;
                Ok(ScriptEndpoint {
                    proposal: resolved.proposal(),
                    node: Some(resolved.accepted().node().node_id()),
                })
            }
        }
    }
    fn node(
        &self,
        endpoint: &ScriptEndpoint<'_>,
        receiver: bool,
        graph: &GraphSnapshot,
    ) -> RecognizerResult<GraphNodeId> {
        let proposal = endpoint.proposal;
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
        let node = match self {
            Self::Legacy(partition) => {
                let accepted = partition.report().accepted_entities();
                let index = accepted
                    .binary_search_by(|p| p.proposal_id().cmp(proposal.proposal_id()))
                    .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                accepted[index].node().node_id()
            }
            Self::Assertions { .. } => endpoint
                .node
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?,
        };
        if graph.node(node).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        Ok(node.clone())
    }
}

struct ScriptOutput {
    batch: GraphProposalBatch,
    coverage: Vec<GraphCoverageRecord>,
    recognition: RecognitionReport,
    receipts: Vec<SourceScriptReceipt>,
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
    let output = recognize_script_facts(
        &ScriptData {
            owner: input.owner,
            context: input.context,
            source_handles: input.source_handles,
            evidence: input.evidence,
        },
        ScriptFacts::Legacy(&input.facts),
        ScriptEndpoints::Legacy(partition),
        &graph,
        stop,
    )?;
    Ok(SourceScriptProposals {
        batch: output.batch,
        coverage: output.coverage,
        recognition: SourceScriptRecognition {
            profile: SOURCE_SCRIPT_PROFILE,
            source_partition: input.source_partition.into(),
            recognition: output.recognition,
            receipts: output.receipts,
        },
    })
}

fn recognize_script_facts(
    input: &ScriptData<'_>,
    facts: ScriptFacts<'_, '_>,
    endpoints: ScriptEndpoints<'_, '_>,
    graph: &GraphSnapshot,
    stop: &AtomicBool,
) -> RecognizerResult<ScriptOutput> {
    let limits = RecognizerLimits::new(MAX_BINDINGS as u32, MAX_BINDINGS as u32, 19, 32)?;
    let mut observations = Vec::new();
    let mut pending = BTreeMap::new();
    let mut fact_ids = BTreeSet::new();
    for index in 0..facts.len() {
        let fact = facts.fact(index);
        checkpoint(stop)?;
        if !valid_content_id(fact.fact_id, "xml-script-binding:sha256:")
            || !matches!(
                fact.confidence,
                GraphConfidence::Derived | GraphConfidence::Possible
            )
            || !fact_ids.insert(fact.fact_id)
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        }
        let handler = endpoints.proposal(index, false, fact.handler_proposal_id, stop)?;
        validate_semantic_context(
            fact,
            handler.proposal.entity_kind_id(),
            handler.proposal.semantic_key().get("semantic_context_id"),
        )?;
        validate_support(input, fact)?;
        let receiver = endpoints.proposal(index, true, fact.receiver_proposal_id, stop)?;
        // Every endpoint's original source support must be carried by the
        // observation. An ID alone is not evidence for the XML/Lua binding.
        for proposal in [receiver.proposal, handler.proposal] {
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
                from: endpoints.node(&receiver, true, graph)?,
                to: endpoints.node(&handler, false, graph)?,
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
                if family == ObservationFamily::ScriptAssignment && !facts.is_empty() {
                    RecognitionCoverageState::Partial
                } else {
                    RecognitionCoverageState::NotEvaluated
                },
                vec![if family == ObservationFamily::ScriptAssignment {
                    "source_scripts.exact_sites_only_receiver_and_dispatch_not_evaluated".into()
                } else {
                    "source_scripts.family_not_requested".into()
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
    if !pending.is_empty() || receipts.len() != facts.len() {
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
                if relation == GraphRelationKind::SetsScript && !facts.is_empty() {
                    GraphCoverageState::Partial
                } else {
                    GraphCoverageState::NotEvaluated
                },
                false,
                vec![if relation == GraphRelationKind::SetsScript {
                    "source_scripts.partial_exact_site_without_receiver_or_dispatch_authority"
                        .into()
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
    Ok(ScriptOutput {
        batch,
        coverage,
        recognition,
        receipts,
    })
}

fn valid_content_id(value: &str, prefix: &str) -> bool {
    let Some(suffix) = value.strip_prefix(prefix) else {
        return false;
    };
    suffix.len() == 64
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_semantic_context(
    fact: &SourceScriptFact<'_>,
    handler_kind: &str,
    handler_context: Option<&GraphProposalValue>,
) -> RecognizerResult<()> {
    match handler_kind {
        "xml_source_handler" => {
            let context = fact
                .semantic_context
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingInvalid))?;
            let context_matches = matches!(
                handler_context,
                Some(GraphProposalValue::String(value)) if value.as_ref() == context.context_id
            );
            if fact.confidence != GraphConfidence::Possible
                || !valid_content_id(context.context_id, XML_CONTEXT_ID_PREFIX)
                || context.script_site != EXACT_XML_SCRIPT_SITE
                || context.implicit_receiver != IMPLICIT_RECEIVER_NOT_EVALUATED
                || context.runtime_dispatch != RUNTIME_DISPATCH_NOT_EVALUATED
                || !context_matches
            {
                return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
            }
        }
        "lua_source_function" => {
            if fact.semantic_context.is_some() || handler_context.is_some() {
                return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
            }
        }
        _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
    }
    Ok(())
}

fn validate_support(input: &ScriptData<'_>, fact: &SourceScriptFact<'_>) -> RecognizerResult<()> {
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
// ===== BEGIN WORKER 5: script and hook families =====
use crate::{
    RecognizerClause, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind, RecognizerFactValue,
    RecognizerOutput, RecognizerOutputConfidence, RecognizerPack, RecognizerPackBudgets,
    RecognizerPackDocument, RecognizerPackRollout, RecognizerPackTrustClass, RecognizerRule,
    compile_recognizer_plan, execute_recognizer_plan, parse_recognizer_pack,
};
use wow_core::{ContentDigest, SourceContent, SourceSpan, canonical_json_bytes};
use wow_emmy::function_calls::{FunctionCallReport, SourceCallLiteral};
// Exact EmmyLua call facts -> declarative script/hook recognizers -> graph proposals.
//
// This producer copies the architecture of `source_construction.rs` and
// `source_mixins.rs` exactly: partition/profile constants, MAX_* budgets, one input
// struct, match/recognition structs, a `RecognizerFactBundle` built from the
// generation-bound `wow_emmy::function_calls::FunctionCallReport`, declarative packs built
// from `RecognizerRule`/`RecognizerClause`, `compile_recognizer_plan` plus
// `execute_recognizer_plan`, graph coverage records over every registry relation,
// `checkpoint(stop)`, and error mapping through `failure(RecognizerErrorCode::*)`.
//
// Source text is never reparsed. Every callable key, argument, span and support record
// arrives from the immutable analyzer report and the exact source partition. No client
// build, Interface value, source revision, provider revision or toolchain version is
// hard-coded as project truth. No rule here claims taint, combat, protected, forbidden,
// managed-object or Secret legality: these are universal structural roles only.

pub const W5_HOOK_PARTITION: &str = "wow-recognizers.lua-hooks";
pub const W5_HOOK_PROFILE: &str = "wow-recognizers/lua-hooks/5";
const W5_FACT_PARTITION: &str = "wow-recognizers.lua-hook-facts";
const W5_FACT_PROFILE: &str = "wow-recognizers-lua-hook-call-facts-5";
const W5_FACT_KIND: &str = "lua_call";
const W5_SET_SCRIPT_RULE: &str = "core.hook.set_script";
const W5_HOOK_SCRIPT_RULE: &str = "core.hook.hook_script";
const W5_SECURE_POSTHOOK_RULE: &str = "core.hook.secure_posthook";
const W5_SET_SCRIPT_RELATION: &str = "lua_sets_script";
const W5_HOOK_SCRIPT_RELATION: &str = "lua_hooks_script";
const W5_SECURE_POSTHOOK_RELATION: &str = "lua_secure_hooks_function";
const W5_SET_SCRIPT_CALLABLE: &str = "Frame.SetScript";
const W5_HOOK_SCRIPT_CALLABLE: &str = "Frame.HookScript";
const W5_SECURE_HOOK_CALLABLE: &str = "hooksecurefunc";
const W5_SCRIPT_NAME_ORDINAL: usize = 0;
const W5_HANDLER_ORDINAL: usize = 1;

const W5_POSTHOOK_MEMBER_ORDINAL: usize = 1;
const W5_POSTHOOK_CALLBACK_ORDINAL: usize = 2;
const W5_MAX_CALLS: usize = 8192;
const W5_MAX_ARGUMENTS: usize = 8;
const W5_MAX_HOOK_RELATIONS: usize = 65_536;
const W5_HOOK_KIND_SCRIPT_POSTHOOK: &str = "script_posthook";
const W5_HOOK_KIND_SECURE_POSTHOOK: &str = "secure_posthook";
const W5_SECURE_FORM_GLOBAL: &str = "global_target";
const W5_SECURE_FORM_TABLE_MEMBER: &str = "table_member_target";
const W5_SECURE_FORM_DYNAMIC: &str = "dynamic_target";
const W5_RECEIVER_EXACT: &str = "exact_colon_receiver";
const W5_RECEIVER_DYNAMIC: &str = "dynamic_receiver";
const W5_CALLBACK_EXACT: &str = "exact_callback";
const W5_CALLBACK_DYNAMIC: &str = "dynamic_callback";

/// Exact call facts plus the caller/declaration source crosswalk.
///
/// Mirrors `SourceConstructionInput`: the analyzer report owns every call identity and
/// the source partition owns every endpoint proposal. The project owner supplies the
/// crosswalks; this producer only verifies them against the real source records.
pub struct W5HookInput<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub report: &'a FunctionCallReport,
    pub context: &'a GenerationContext,
    pub function_proposals: BTreeMap<&'a str, &'a str>,
    pub declaration_proposals: BTreeMap<(&'a str, SourceSpan), &'a str>,
    pub call_support: BTreeMap<&'a str, (StableHandleId, EvidenceId)>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

/// One exact `object:SetScript(scriptName, handler)` site.
///
/// `object_proposal_id` and `handler_proposal_id` stay `None` when the receiver or the
/// handler resolves to no single Main declaration. The relation then carries only the
/// exact call identity and stays `Possible` instead of claiming an endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W5SetScriptMatch {
    pub call_id: String,
    pub caller_proposal_id: String,
    pub object_proposal_id: Option<String>,
    pub script_name: Option<String>,
    pub handler_proposal_id: Option<String>,
    pub relation_proposal_id: String,
}

/// One exact `object:HookScript(scriptName, handler)` site.
///
/// `hook_kind` is the frozen structural kind `script_posthook`. It is not a
/// protected/forbidden/taint-safety claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W5HookScriptMatch {
    pub call_id: String,
    pub caller_proposal_id: String,
    pub object_proposal_id: Option<String>,
    pub script_name: Option<String>,
    pub hook_kind: &'static str,
    pub handler_proposal_id: Option<String>,
    pub relation_proposal_id: String,
}

/// One exact `hooksecurefunc` site in either admitted form.
///
/// `target_form` records which exact argument form was observed. `target_proposal_id` is
/// the resolved target declaration for `global_target` and the resolved table declaration
/// for `table_member_target`; `member_name` is the exact literal method name for the table
/// form. A dynamic target keeps `dynamic_target` with no target proposal and the relation
/// stays `Possible`. A global assignment or override is never recognized as a safe hook,
/// and no combat/taint guarantee is expressed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W5SecurePosthookMatch {
    pub call_id: String,
    pub caller_proposal_id: String,
    pub target_form: &'static str,
    pub target_proposal_id: Option<String>,
    pub member_name: Option<String>,
    pub callback_form: &'static str,
    pub callback_proposal_id: Option<String>,
    pub hook_kind: &'static str,
    pub relation_proposal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W5HookRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    set_script_matches: Vec<W5SetScriptMatch>,
    hook_script_matches: Vec<W5HookScriptMatch>,
    secure_posthook_matches: Vec<W5SecurePosthookMatch>,
}

impl W5HookRecognition {
    pub fn set_script_matches(&self) -> &[W5SetScriptMatch] {
        &self.set_script_matches
    }
    pub fn hook_script_matches(&self) -> &[W5HookScriptMatch] {
        &self.hook_script_matches
    }
    pub fn secure_posthook_matches(&self) -> &[W5SecurePosthookMatch] {
        &self.secure_posthook_matches
    }
}

pub struct W5HookProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: W5HookRecognition,
}

/// One normalized argument. Only exact literals and exact reference targets are kept;
/// anything else stays dynamic and can never be upgraded to proof.
struct W5Argument {
    ordinal: usize,
    span: SourceSpan,
    kind: &'static str,
    literal: Option<SourceCallLiteral>,
    reference_key: Option<String>,
    reference_proposal: Option<String>,
}

/// One hook call site normalized into exact structural facts before any pack runs.
struct W5Site {
    call_id: String,
    caller_proposal_id: String,
    callable_key: &'static str,
    colon_call: bool,
    receiver_proposal: Option<String>,
    receiver_handle: Option<StableHandleId>,
    receiver_evidence: Option<EvidenceId>,
    argument_count: usize,
    arguments: Vec<W5Argument>,
}

struct W5Endpoints {
    object: Option<String>,
    handler: Option<String>,
}

/// One resolved relation. The caller endpoint is always known; every other endpoint is
/// optional so a dynamic site keeps the call-side relation without a fabricated target.
struct W5RelationSpec {
    proposal_id: String,
    call_id: String,
    caller_function_id: String,
    relation_kind_id: &'static str,
    to_proposal: Option<String>,
    confidence: GraphConfidence,
    handles: Vec<StableHandleId>,
    evidence: Vec<EvidenceId>,
    coverage: Vec<wow_core::CoverageId>,
}

struct W5Binding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

pub fn recognize_source_hooks(
    input: W5HookInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<W5HookProposals> {
    checkpoint(stop)?;
    if input.report.calls().len() > W5_MAX_CALLS || input.report.functions().len() > W5_MAX_CALLS {
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
        || input.call_support.len() != input.report.calls().len()
        || input.function_proposals.len() != input.report.functions().len()
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let lookup = input.owner.producer_lookup(stop).map_err(graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    // Each captured function owns one strict crosswalk. An unknown, duplicate or
    // mistyped proposal is a binding defect, never a silently dropped call.
    let mut caller_nodes = BTreeMap::<String, GraphNodeId>::new();
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
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w5_validate_support_without_digest(
            &input,
            *handle,
            *evidence,
            function.path(),
            function.span(),
        )?;
        let node = resolved.accepted().node().node_id().clone();
        if caller_nodes
            .insert(function.fact_id().to_owned(), node)
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    // Referenced Main declarations own exact (path, span) keys. A library reference or a
    // dynamic argument never fabricates a proposal.
    let mut declarations = BTreeMap::<(String, SourceSpan), W5Binding>::new();
    let mut declaration_proposal_ids = BTreeMap::<(String, SourceSpan), String>::new();
    let mut declaration_ids = BTreeSet::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        if !declaration_ids.insert((*proposal_id).to_owned()) {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        checkpoint(stop)?;
        let reference = GraphAssertionRef::Producer {
            partition_id: source_partition.partition_id().into(),
            batch_id: source_partition.batch().batch_id().into(),
            assertion: GraphLocalAssertion {
                kind: GraphAssertionKind::Entity,
                proposal_id: (*proposal_id).into(),
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
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let expected = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String((*path).into()),
            ),
            (
                "span_start".into(),
                GraphProposalValue::Integer(w5_i64(start)?),
            ),
            ("span_end".into(), GraphProposalValue::Integer(w5_i64(end)?)),
        ]);
        if proposal.entity_kind_id() != "lua_source_declaration"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w5_validate_support_without_digest(&input, *handle, *evidence, path, *span)?;
        let node = resolved.accepted().node().node_id().clone();
        let key = ((*path).to_owned(), *span);
        if declarations
            .insert(
                key.clone(),
                W5Binding {
                    node,
                    handle: *handle,
                    evidence: *evidence,
                },
            )
            .is_some()
            || declaration_proposal_ids
                .insert(key, (*proposal_id).to_owned())
                .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut sites = Vec::new();
    for call in input.report.calls() {
        checkpoint(stop)?;
        // Only exact resolved hook callables enter this partition. An unresolved or
        // indeterminate callable key is not a hook match and never becomes one.
        let Some(callable) = w5_callable_key(call.resolved_callable_key()) else {
            continue;
        };
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w5_validate_support(&input, handle, evidence, call)?;
        let caller_proposal = *input
            .function_proposals
            .get(call.caller_function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if !caller_nodes.contains_key(call.caller_function_id()) {
            return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
        }
        let receiver_binding = if call.is_colon_call() {
            input.report.exact_call_receiver(call).and_then(|receiver| {
                let key = (receiver.target().path.clone(), receiver.target().span);
                let proposal = declaration_proposal_ids.get(&key)?.clone();
                let binding = declarations.get(&key)?;
                Some((proposal, binding.handle, binding.evidence))
            })
        } else {
            None
        };
        let receiver_exact = matches!(callable, W5_SET_SCRIPT_CALLABLE | W5_HOOK_SCRIPT_CALLABLE)
            && receiver_binding.is_some();
        let mut fields = BTreeMap::from([
            (
                "call_id".into(),
                RecognizerFactValue::Reference(call.fact_id().into()),
            ),
            (
                "caller".into(),
                RecognizerFactValue::Reference(caller_proposal.into()),
            ),
            (
                "argument_count".into(),
                RecognizerFactValue::Integer(w5_i64(
                    u64::try_from(call.arguments().len())
                        .map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))?,
                )?),
            ),
            (
                "colon_call".into(),
                RecognizerFactValue::Boolean(call.is_colon_call()),
            ),
            (
                "callable_key".into(),
                RecognizerFactValue::String(callable.into()),
            ),
            (
                "receiver_kind".into(),
                RecognizerFactValue::String(
                    if receiver_exact {
                        W5_RECEIVER_EXACT
                    } else {
                        W5_RECEIVER_DYNAMIC
                    }
                    .into(),
                ),
            ),
        ]);
        if let Some((proposal, _, _)) = &receiver_binding {
            fields.insert(
                "receiver".into(),
                RecognizerFactValue::Reference(proposal.clone().into()),
            );
        }
        let mut exact_arguments = call.arguments().len() <= W5_MAX_ARGUMENTS;
        let mut arguments = Vec::new();
        for (ordinal, argument) in call.arguments().iter().take(W5_MAX_ARGUMENTS).enumerate() {
            let resolved = w5_argument(&input, &declaration_proposal_ids, argument, ordinal)?;
            let (Some(start), Some(end)) = (resolved.span.byte_start(), resolved.span.byte_end())
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            fields.insert(
                format!("argument_{ordinal}_span_start").into_boxed_str(),
                RecognizerFactValue::Integer(w5_i64(start)?),
            );
            fields.insert(
                format!("argument_{ordinal}_span_end").into_boxed_str(),
                RecognizerFactValue::Integer(w5_i64(end)?),
            );
            fields.insert(
                format!("argument_{ordinal}_kind").into_boxed_str(),
                RecognizerFactValue::String(resolved.kind.into()),
            );
            match &resolved.literal {
                Some(SourceCallLiteral::String(value)) => {
                    fields.insert(
                        format!("argument_{ordinal}_value").into_boxed_str(),
                        RecognizerFactValue::String(value.clone().into_boxed_str()),
                    );
                }
                Some(SourceCallLiteral::Boolean(value)) => {
                    fields.insert(
                        format!("argument_{ordinal}_value").into_boxed_str(),
                        RecognizerFactValue::Boolean(*value),
                    );
                }
                Some(SourceCallLiteral::Nil) => {
                    fields.insert(
                        format!("argument_{ordinal}_value").into_boxed_str(),
                        RecognizerFactValue::Nil,
                    );
                }
                None => {}
            }
            if let Some(key) = &resolved.reference_key {
                fields.insert(
                    format!("argument_{ordinal}_reference_key").into_boxed_str(),
                    RecognizerFactValue::String(key.clone().into_boxed_str()),
                );
            }
            exact_arguments &= resolved.kind != "dynamic";
            arguments.push(resolved);
        }
        fields.insert(
            "exact_arguments".into(),
            RecognizerFactValue::Boolean(exact_arguments),
        );
        let mut source_handles = BTreeSet::from([handle]);
        let mut evidence_ids = BTreeSet::from([evidence]);
        if let Some((_, receiver_handle, receiver_evidence)) = &receiver_binding {
            source_handles.insert(*receiver_handle);
            evidence_ids.insert(*receiver_evidence);
        }
        for argument in &arguments {
            if let Some(proposal) = argument.reference_proposal.as_deref() {
                let binding = w5_binding(&declarations, &declaration_proposal_ids, proposal)?;
                source_handles.insert(binding.handle);
                evidence_ids.insert(binding.evidence);
            }
        }
        facts.push(RecognizerFact::new(
            input.context.context_id(),
            RecognizerFactInput {
                kind: W5_FACT_KIND.into(),
                partition_id: W5_FACT_PARTITION.into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    call.caller_function_id(),
                )?,
                producer_id: "wow.emmy".into(),
                producer_version: W5_FACT_PROFILE.into(),
                confidence: if exact_arguments && (receiver_exact || !call.is_colon_call()) {
                    GraphConfidence::Derived
                } else {
                    GraphConfidence::Possible
                },
                fields,
                source_handle_ids: source_handles.into_iter().collect(),
                evidence_ids: evidence_ids.into_iter().collect(),
            },
            fact_limits,
        )?);
        sites.push(W5Site {
            call_id: call.fact_id().to_owned(),
            caller_proposal_id: caller_proposal.to_owned(),
            callable_key: callable,
            colon_call: call.is_colon_call(),
            receiver_proposal: receiver_binding
                .as_ref()
                .map(|(proposal, _, _)| proposal.clone()),
            receiver_handle: receiver_binding.as_ref().map(|(_, handle, _)| *handle),
            receiver_evidence: receiver_binding.as_ref().map(|(_, _, evidence)| *evidence),
            argument_count: call.arguments().len(),
            arguments,
        });
    }

    let coverage_state = if input.report.source_health_complete() {
        RecognizerFactCoverageState::Complete
    } else {
        RecognizerFactCoverageState::NotEvaluated
    };
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W5_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: W5_FACT_PROFILE.into(),
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
        W5_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = w5_hook_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut set_script_matches = Vec::new();
    let mut hook_script_matches = Vec::new();
    let mut secure_posthook_matches = Vec::new();
    let mut relations = Vec::new();
    for outcome in output.outcomes() {
        checkpoint(stop)?;
        if outcome.rule_version() != 1 || !w5_is_expected_rule(outcome.rule_id()) {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let relation_kind_id = w5_relation_id(outcome.rule_id())?;
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Relation {
                proposal_id,
                relation_kind_id: proposed_relation,
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
            let [fact_id] = decisive_fact_ids.as_slice() else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let fact = bundle
                .fact_by_id(fact_id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if fact.kind() != W5_FACT_KIND
                || proposed_relation.as_ref() != relation_kind_id
                || source_handle_ids.as_slice() != fact.source_handle_ids()
                || evidence_ids.as_slice() != fact.evidence_ids()
            {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let site = sites
                .iter()
                .find(|site| site.call_id == call_id.as_ref())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let call = input
                .report
                .calls()
                .iter()
                .find(|call| call.fact_id() == call_id.as_ref())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let (call_handle, call_evidence) = *input
                .call_support
                .get(call_id.as_ref())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let caller_proposal = match fact.field("caller") {
                Some(RecognizerFactValue::Reference(value)) => value.to_string(),
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            };
            let expected_caller = *input
                .function_proposals
                .get(call.caller_function_id())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if caller_proposal != site.caller_proposal_id || caller_proposal != expected_caller {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            // The exact resolved structure decides the target. A dynamic argument form
            // never produces a target proposal and the relation stays Possible.
            let to_proposal = match outcome.rule_id() {
                W5_SET_SCRIPT_RULE | W5_HOOK_SCRIPT_RULE => {
                    w5_object_or_handler(site, W5_HANDLER_ORDINAL).handler
                }
                W5_SECURE_POSTHOOK_RULE => w5_secure_target(site, 0),
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            };
            let mut handles = source_handle_ids.iter().copied().collect::<BTreeSet<_>>();
            let mut evidence = evidence_ids.iter().copied().collect::<BTreeSet<_>>();
            handles.insert(call_handle);
            evidence.insert(call_evidence);
            if let Some(receiver_handle) = site.receiver_handle {
                handles.insert(receiver_handle);
            }
            if let Some(receiver_evidence) = site.receiver_evidence {
                evidence.insert(receiver_evidence);
            }
            if let Some(proposal) = &to_proposal {
                let binding = w5_binding(&declarations, &declaration_proposal_ids, proposal)?;
                handles.insert(binding.handle);
                evidence.insert(binding.evidence);
            }
            let graph_id = proposal_id.to_string();
            if to_proposal.is_some() {
                relations.push(W5RelationSpec {
                    proposal_id: graph_id.clone(),
                    call_id: call_id.to_string(),
                    caller_function_id: call.caller_function_id().to_owned(),
                    relation_kind_id,
                    to_proposal: to_proposal.clone(),
                    confidence: graph_confidence(*confidence),
                    handles: handles.into_iter().collect(),
                    evidence: evidence.into_iter().collect(),
                    coverage: coverage_ids.clone(),
                });
            }
            match outcome.rule_id() {
                W5_SET_SCRIPT_RULE => {
                    let endpoints = w5_object_or_handler(site, W5_HANDLER_ORDINAL);
                    set_script_matches.push(W5SetScriptMatch {
                        call_id: call_id.to_string(),
                        caller_proposal_id: site.caller_proposal_id.to_owned(),
                        object_proposal_id: endpoints.object,
                        script_name: w5_argument_literal(site, W5_SCRIPT_NAME_ORDINAL),
                        handler_proposal_id: endpoints.handler,
                        relation_proposal_id: graph_id,
                    });
                }
                W5_HOOK_SCRIPT_RULE => {
                    let endpoints = w5_object_or_handler(site, W5_HANDLER_ORDINAL);
                    hook_script_matches.push(W5HookScriptMatch {
                        call_id: call_id.to_string(),
                        caller_proposal_id: site.caller_proposal_id.to_owned(),
                        object_proposal_id: endpoints.object,
                        script_name: w5_argument_literal(site, W5_SCRIPT_NAME_ORDINAL),
                        hook_kind: W5_HOOK_KIND_SCRIPT_POSTHOOK,
                        handler_proposal_id: endpoints.handler,
                        relation_proposal_id: graph_id,
                    });
                }
                W5_SECURE_POSTHOOK_RULE => {
                    let callback_ordinal = w5_callback_ordinal(site);
                    let callback_exact = callback_ordinal
                        .and_then(|ordinal| site.arguments.get(ordinal))
                        .is_some_and(|argument| argument.reference_proposal.is_some());
                    secure_posthook_matches.push(W5SecurePosthookMatch {
                        call_id: call_id.to_string(),
                        caller_proposal_id: site.caller_proposal_id.to_owned(),
                        target_form: w5_secure_form(site),
                        target_proposal_id: w5_secure_target(site, 0),
                        member_name: w5_argument_literal(site, W5_POSTHOOK_MEMBER_ORDINAL),
                        callback_form: if callback_exact {
                            W5_CALLBACK_EXACT
                        } else {
                            W5_CALLBACK_DYNAMIC
                        },
                        callback_proposal_id: callback_ordinal
                            .and_then(|ordinal| w5_argument_proposal(site, ordinal)),
                        hook_kind: W5_HOOK_KIND_SECURE_POSTHOOK,
                        relation_proposal_id: graph_id,
                    });
                }
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            }
        }
    }

    // Describe every registry relation. This producer owns only the three hook
    // relations; every other relation remains explicitly unevaluated here.
    let relation_families = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|definition| definition.relation())
        .collect::<BTreeSet<_>>();
    let graph_coverage = relation_families
        .into_iter()
        .map(|relation| {
            let (state, blocker) = match relation {
                GraphRelationKind::SetsScript if !set_script_matches.is_empty() => (
                    GraphCoverageState::Partial,
                    "lua_hooks.set_script_object_name_handler_structure_only",
                ),
                GraphRelationKind::HooksScript
                    if !hook_script_matches.is_empty() || !secure_posthook_matches.is_empty() =>
                {
                    (
                        GraphCoverageState::Partial,
                        "lua_hooks.posthook_structure_only_no_safety_authority",
                    )
                }
                GraphRelationKind::SecureHooksFunction if !secure_posthook_matches.is_empty() => (
                    GraphCoverageState::Partial,
                    "lua_hooks.secure_posthook_structure_only_no_combat_or_taint_authority",
                ),
                _ => (
                    GraphCoverageState::NotEvaluated,
                    "lua_hooks.relation_owned_by_other_producer",
                ),
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;

    if relations.len() > W5_MAX_HOOK_RELATIONS {
        return Err(failure(RecognizerErrorCode::BudgetExceeded));
    }
    let mut graph_relations = Vec::new();
    for relation in &relations {
        checkpoint(stop)?;
        // A dynamic target keeps the call-side match but never publishes an endpoint.
        // This producer therefore only emits relations with an exact resolved target.
        let Some(to_proposal) = relation.to_proposal.clone() else {
            continue;
        };
        let binding = w5_binding(&declarations, &declaration_proposal_ids, &to_proposal)?;
        let caller = caller_nodes
            .get(&relation.caller_function_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        graph_relations.push(
            GraphRelationProposal::new(
                relation.proposal_id.as_str(),
                relation.relation_kind_id,
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(caller.clone()),
                    target: GraphProposalEndpoint::Existing(binding.node.clone()),
                    confidence: relation.confidence,
                    source_handle_ids: relation.handles.clone(),
                    evidence_ids: relation.evidence.clone(),
                    coverage_ids: relation.coverage.clone(),
                },
            )
            .map_err(graph_error)?,
        );
    }
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        W5_HOOK_PARTITION,
        Vec::new(),
        graph_relations,
    )
    .map_err(graph_error)?;

    set_script_matches.sort_by(|left, right| left.call_id.cmp(&right.call_id));
    hook_script_matches.sort_by(|left, right| left.call_id.cmp(&right.call_id));
    secure_posthook_matches.sort_by(|left, right| left.call_id.cmp(&right.call_id));
    checkpoint(stop)?;
    Ok(W5HookProposals {
        batch,
        coverage: graph_coverage,
        recognition: W5HookRecognition {
            profile: W5_HOOK_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            set_script_matches,
            hook_script_matches,
            secure_posthook_matches,
        },
    })
}

/// Only the exact reviewed hook callables of this family enter the partition. The
/// literal spelling is a profile-bound constraint, never a platform-availability claim.
fn w5_callable_key(key: Option<&str>) -> Option<&'static str> {
    match key? {
        W5_SET_SCRIPT_CALLABLE => Some(W5_SET_SCRIPT_CALLABLE),
        W5_HOOK_SCRIPT_CALLABLE => Some(W5_HOOK_SCRIPT_CALLABLE),
        W5_SECURE_HOOK_CALLABLE => Some(W5_SECURE_HOOK_CALLABLE),
        _ => None,
    }
}

fn w5_is_expected_rule(rule_id: &str) -> bool {
    matches!(
        rule_id,
        W5_SET_SCRIPT_RULE | W5_HOOK_SCRIPT_RULE | W5_SECURE_POSTHOOK_RULE
    )
}

fn w5_relation_id(rule_id: &str) -> RecognizerResult<&'static str> {
    match rule_id {
        W5_SET_SCRIPT_RULE => Ok(W5_SET_SCRIPT_RELATION),
        W5_HOOK_SCRIPT_RULE => Ok(W5_HOOK_SCRIPT_RELATION),
        W5_SECURE_POSTHOOK_RULE => Ok(W5_SECURE_POSTHOOK_RELATION),
        _ => Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
    }
}

fn w5_i64(value: u64) -> RecognizerResult<i64> {
    i64::try_from(value).map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))
}

fn graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

/// Normalizes one exact argument into a structural ingredient. A literal keeps
/// its exact form.
fn w5_argument(
    input: &W5HookInput<'_>,
    declaration_proposal_ids: &BTreeMap<(String, SourceSpan), String>,
    argument: &wow_emmy::function_calls::SourceCallArgument,
    ordinal: usize,
) -> RecognizerResult<W5Argument> {
    match (
        argument.literal(),
        argument.reference_key(),
        argument.reference_target(),
    ) {
        (None, Some(key), Some(target)) => {
            if target.role == "main" {
                if target.workspace_id != input.report.main_snapshot_id() {
                    return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                }
                let declaration_key = (target.path.clone(), target.span);
                let proposal_id = declaration_proposal_ids
                    .get(&declaration_key)
                    .cloned()
                    .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                Ok(W5Argument {
                    ordinal,
                    span: argument.span(),
                    kind: "main_reference",
                    literal: None,
                    reference_key: Some((*key).to_owned()),
                    reference_proposal: Some(proposal_id),
                })
            } else if target.role == "library"
                && input
                    .report
                    .library_snapshot_ids()
                    .contains(&target.workspace_id)
            {
                Ok(W5Argument {
                    ordinal,
                    span: argument.span(),
                    kind: "library_reference",
                    literal: None,
                    reference_key: Some((*key).to_owned()),
                    reference_proposal: None,
                })
            } else {
                Err(failure(RecognizerErrorCode::AdapterFactMismatch))
            }
        }
        (Some(literal), None, None) => Ok(W5Argument {
            ordinal,
            span: argument.span(),
            kind: "literal",
            literal: Some(literal.clone()),
            reference_key: None,
            reference_proposal: None,
        }),
        (None, None, None) => Ok(W5Argument {
            ordinal,
            span: argument.span(),
            kind: "dynamic",
            literal: None,
            reference_key: None,
            reference_proposal: None,
        }),
        _ => Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
    }
}

/// Resolves the exact handler declaration for one `SetScript`/`HookScript` site.
///
/// The handler is the resolved Main declaration of the second argument. A dynamic
/// handler keeps `None` instead of a guess.
///
/// The script object is deliberately not resolved here. This producer receives no
/// receiver-declaration crosswalk, and inferring one from a spelling, a file path stem
/// or a local variable name is forbidden. `object_proposal_id` therefore stays `None`
/// and the object axis remains unevaluated for this partition, while the handler axis
/// stays exact.
fn w5_object_or_handler(site: &W5Site, handler_ordinal: usize) -> W5Endpoints {
    W5Endpoints {
        object: site.receiver_proposal.clone(),
        handler: w5_argument_proposal(site, handler_ordinal),
    }
}

/// Resolves the exact `hooksecurefunc` target for one site.
///
/// `hooksecurefunc(globalName, callback)` resolves `globalName` to its Main declaration.
/// `hooksecurefunc(table, methodName, callback)` resolves `table` to its Main declaration
/// and keeps the exact literal member name. Any other shape stays a dynamic target: no
/// target proposal, no exact member, and no safe-hook claim.
fn w5_callback_ordinal(site: &W5Site) -> Option<usize> {
    match site.argument_count {
        2 => Some(1),
        3 => Some(2),
        _ => None,
    }
}

fn w5_secure_target(site: &W5Site, target_ordinal: usize) -> Option<String> {
    match site.argument_count {
        2 | 3 => site
            .arguments
            .get(target_ordinal)
            .and_then(|argument| argument.reference_proposal.clone()),
        _ => None,
    }
}

/// Records the exact `hooksecurefunc` argument form. A two-argument call whose first
/// argument is a literal or resolves to a Main declaration is the global form; a
/// three-argument call whose first argument resolves and whose second argument is an
/// exact literal is the table/member form; everything else stays dynamic. A global
/// assignment or override never becomes a safe hook under any form.
fn w5_secure_form(site: &W5Site) -> &'static str {
    let target = site.arguments.first();
    let member = site.arguments.get(W5_POSTHOOK_MEMBER_ORDINAL);
    match site.argument_count {
        2 => match target {
            Some(argument) if argument.reference_proposal.is_some() => W5_SECURE_FORM_GLOBAL,
            Some(argument) if argument.literal.is_some() => W5_SECURE_FORM_GLOBAL,
            _ => W5_SECURE_FORM_DYNAMIC,
        },
        3 => match (target, member) {
            (Some(argument), Some(member)) if argument.reference_proposal.is_some() => {
                if member.literal.is_some() {
                    W5_SECURE_FORM_TABLE_MEMBER
                } else {
                    W5_SECURE_FORM_DYNAMIC
                }
            }
            _ => W5_SECURE_FORM_DYNAMIC,
        },
        _ => W5_SECURE_FORM_DYNAMIC,
    }
}

fn w5_argument_proposal(site: &W5Site, ordinal: usize) -> Option<String> {
    site.arguments
        .get(ordinal)
        .and_then(|argument| argument.reference_proposal.clone())
}

/// The exact literal script or member name, when that argument is a string literal.
fn w5_argument_literal(site: &W5Site, ordinal: usize) -> Option<String> {
    site.arguments
        .get(ordinal)
        .and_then(|argument| match &argument.literal {
            Some(SourceCallLiteral::String(value)) => Some(value.clone()),
            _ => None,
        })
}

/// Exact reverse lookup from a declaration proposal to its binding. Every endpoint
/// proposal must already exist in the (path, span) crosswalk; a missing entry is an
/// adapter defect, never a silent no-match.
fn w5_binding(
    declarations: &BTreeMap<(String, SourceSpan), W5Binding>,
    declaration_proposal_ids: &BTreeMap<(String, SourceSpan), String>,
    proposal_id: &str,
) -> RecognizerResult<W5Binding> {
    let key = declaration_proposal_ids
        .iter()
        .find(|(_, candidate)| candidate.as_str() == proposal_id)
        .map(|(key, _)| key.clone())
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let binding = declarations
        .get(&key)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    Ok(W5Binding {
        node: binding.node.clone(),
        handle: binding.handle,
        evidence: binding.evidence,
    })
}

fn w5_validate_support(
    input: &W5HookInput<'_>,
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
    let (Some(start), Some(end)) = (call.call_span().byte_start(), call.call_span().byte_end())
    else {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    if handle.handle_id() != handle_id
        || handle.path().as_str() != call.path()
        || handle.span().byte_start() != Some(start)
        || handle.span().byte_end() != Some(end)
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

fn w5_validate_support_without_digest(
    input: &W5HookInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
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
    if handle.handle_id() != handle_id
        || handle.path().as_str() != path
        || handle.span() != span
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

/// One declarative core pack per invocation, holding the three script/hook rule families.
///
/// Each rule selects one exact callable key and asserts one universal relation. The
/// object/name/handler ingredients stay in the structural fact, never in the clause:
/// clauses match exact resolved callable keys only, so a rename of a decisive literal
/// removes exactly one rule and nothing else.
fn w5_hook_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let mut document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-hooks".into(),
            version: "5".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W5_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-hooks-5".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 16,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 10_000,
                max_proposals_per_rule_partition: 20_000,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![
                RecognizerRule {
                    rule_id: W5_SET_SCRIPT_RULE.into(),
                    version: 1,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "call".into(),
                            kind: W5_FACT_KIND.into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W5_SET_SCRIPT_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.argument_0_kind".into(),
                            value: crate::RecognizerPackLiteral::String("literal".into()),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "set_script_relation".into(),
                        relation_kind_id: W5_SET_SCRIPT_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-HOOK-003".into()],
                    near_negative_fixture_ids: vec!["RECOG-EVENT-001".into()],
                    partial_fixture_ids: vec!["RECOG-EVENT-009".into()],
                    mutation_fixture_ids: vec!["RECOG-EVENT-004".into()],
                },
                RecognizerRule {
                    rule_id: W5_HOOK_SCRIPT_RULE.into(),
                    version: 1,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "call".into(),
                            kind: W5_FACT_KIND.into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W5_HOOK_SCRIPT_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.argument_0_kind".into(),
                            value: crate::RecognizerPackLiteral::String("literal".into()),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "hook_script_relation".into(),
                        relation_kind_id: W5_HOOK_SCRIPT_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-EVENT-001".into()],
                    near_negative_fixture_ids: vec!["RECOG-HOOK-003".into()],
                    partial_fixture_ids: vec!["RECOG-EVENT-009".into()],
                    mutation_fixture_ids: vec!["RECOG-STATE-002".into()],
                },
                RecognizerRule {
                    rule_id: W5_SECURE_POSTHOOK_RULE.into(),
                    version: 1,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "call".into(),
                            kind: W5_FACT_KIND.into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W5_SECURE_HOOK_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "call.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(false),
                        },
                        RecognizerClause::AnyOf {
                            clauses: vec![
                                RecognizerClause::AllOf {
                                    clauses: vec![
                                        RecognizerClause::FieldEq {
                                            field: "call.argument_count".into(),
                                            value: crate::RecognizerPackLiteral::Integer(2),
                                        },
                                        RecognizerClause::AnyOf {
                                            clauses: vec![
                                                RecognizerClause::FieldEq {
                                                    field: "call.argument_0_kind".into(),
                                                    value: crate::RecognizerPackLiteral::String(
                                                        "main_reference".into(),
                                                    ),
                                                },
                                                RecognizerClause::FieldEq {
                                                    field: "call.argument_0_kind".into(),
                                                    value: crate::RecognizerPackLiteral::String(
                                                        "literal".into(),
                                                    ),
                                                },
                                            ],
                                        },
                                    ],
                                },
                                RecognizerClause::AllOf {
                                    clauses: vec![
                                        RecognizerClause::FieldEq {
                                            field: "call.argument_count".into(),
                                            value: crate::RecognizerPackLiteral::Integer(3),
                                        },
                                        RecognizerClause::FieldEq {
                                            field: "call.argument_0_kind".into(),
                                            value: crate::RecognizerPackLiteral::String(
                                                "main_reference".into(),
                                            ),
                                        },
                                        RecognizerClause::FieldEq {
                                            field: "call.argument_1_kind".into(),
                                            value: crate::RecognizerPackLiteral::String(
                                                "literal".into(),
                                            ),
                                        },
                                    ],
                                },
                            ],
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "secure_posthook_relation".into(),
                        relation_kind_id: W5_SECURE_POSTHOOK_RELATION.into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: vec!["RECOG-HOOK-003".into()],
                    near_negative_fixture_ids: vec!["RECOG-XML-004".into()],
                    partial_fixture_ids: vec!["RECOG-EVENT-009".into()],
                    mutation_fixture_ids: vec!["RECOG-STATE-002".into()],
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

// ===== END WORKER 5: script and hook families =====
