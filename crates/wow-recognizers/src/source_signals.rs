#![allow(dead_code)]
//! Exact captured callable facts -> declarative core recognizers -> graph proposals.
//! Source text is never reparsed here. All keys, arguments, spans and support come
//! from the generation-bound Emmy owner report.
//!
//! This module owns the E2-B signal/event family. Each worker block is
//! self-contained and prefixed so it cannot collide with a sibling producer.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId, canonical_json_bytes,
};
use wow_emmy::function_calls::FunctionCallReport;
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use crate::{
    RecognizerClause, RecognizerError, RecognizerErrorCode, RecognizerFact, RecognizerFactBundle,
    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind,
    RecognizerFactValue, RecognizerOutput, RecognizerOutputConfidence, RecognizerPack,
    RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerResult, RecognizerRule, compile_recognizer_plan, execute_recognizer_plan,
    parse_recognizer_pack,
};

// ===== BEGIN WORKER 1: native frame event =====

//  Exact Frame event-registration facts -> declarative core recognizer -> graph
//  proposals. Source text is never reparsed here: the event name, unit tokens and
//  every span come from the generation-bound Emmy function-call report. Handler
//  associations remain owned by independent SetScript/XML producers. This module
//  never claims combat safety, protected-action
//  authority, taint state or secret-payload accessibility, and it never reads,
//  caches or infers a client build, Interface value, source revision, provider
//  revision or toolchain version. Only universal graph roles are used.

pub const W1_SIGNAL_PARTITION: &str = "wow-recognizers.lua-native-frame-events";
const W1_SIGNAL_PROFILE: &str = "wow-recognizers/lua-native-frame-events/4";
const W1_FACT_PARTITION: &str = "wow-recognizers.lua-native-frame-event-facts";
const W1_FACT_PROFILE: &str = "wow-recognizers-lua-native-frame-event-facts-4";
const W1_RULE: &str = "core.signal.native_frame_event";
const W1_RULE_VERSION: u32 = 1;
const W1_REGISTER_EVENT_CALLABLE: &str = "Frame.RegisterEvent";
const W1_REGISTER_UNIT_EVENT_CALLABLE: &str = "Frame.RegisterUnitEvent";
const W1_MAX_CALLS: usize = 8192;
const W1_MAX_FUNCTIONS: usize = 8192;
const W1_MAX_ARGUMENTS: usize = 8;
const W1_MAX_EVENT_NAMES: usize = 16;
const W1_MAX_UNIT_TOKENS: usize = 16;
const W1_MAX_MATCHES: usize = 8192;
const W1_MAX_EVENT_BYTES: usize = 1024;
const W1_EVENT_ENTITY: &str = "native_event";
const W1_REGISTERS_DEFINITION: &str = "lua_register_native_event";
const W1_REGISTERS_BLOCKER: &str =
    "lua_native_frame_events.exact_resolved_registerevent_calls_only";
const W1_HANDLES_BLOCKER: &str =
    "lua_native_frame_events.handler_association_owned_by_other_producers";
const W1_OTHER_RELATION_BLOCKER: &str = "lua_native_frame_events.relation_owned_by_other_producer";
const W1_POSITIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-001"];
const W1_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-005"];
const W1_PARTIAL_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-007"];
const W1_MUTATION_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-004"];

/// One exact registration site, resolved from structured facts only. A
/// `W1RegistrationKind::FrameUnitEvent` site retains its ordered unit token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W1FrameEventMatch {
    pub call_id: String,
    pub registration_kind: W1RegistrationKind,
    pub caller_function_proposal_id: String,
    pub event_proposal_id: String,
    pub event_names: Vec<String>,
    pub exact_event_names: bool,
    pub unit_tokens: Vec<String>,
    pub registers_relation_proposal_id: String,
    pub handles_relation_proposal_ids: Vec<String>,
    pub handler_declaration_proposal_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum W1RegistrationKind {
    FrameEvent,
    FrameUnitEvent,
}

impl W1RegistrationKind {
    const fn callable(self) -> &'static str {
        match self {
            Self::FrameEvent => W1_REGISTER_EVENT_CALLABLE,
            Self::FrameUnitEvent => W1_REGISTER_UNIT_EVENT_CALLABLE,
        }
    }

    /// Both reviewed WoW forms take the native event name first.
    const fn event_ordinal(self) -> usize {
        0
    }

    /// `RegisterUnitEvent(event, unit1, ...)` starts its ordered unit list at 1.
    const fn unit_ordinal(self) -> Option<usize> {
        match self {
            Self::FrameEvent => None,
            Self::FrameUnitEvent => Some(1),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W1Recognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matches: Vec<W1FrameEventMatch>,
}

impl W1Recognition {
    pub fn matches(&self) -> &[W1FrameEventMatch] {
        &self.matches
    }
}

pub struct W1Proposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: W1Recognition,
}

/// Caller-side crosswalks are re-validated against the exact owner graph, the
/// exact source entity proposals and the real support records. No source text,
/// file name or repository path is ever consulted.
pub struct W1Input<'a> {
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

struct W1FunctionBinding {
    node: GraphNodeId,
    proposal_id: String,
    handle: StableHandleId,
    evidence: EvidenceId,
}

struct W1Site {
    call_id: String,
    kind: W1RegistrationKind,
    caller: W1FunctionBinding,
    handle: StableHandleId,
    evidence: EvidenceId,
    event_names: Vec<String>,
    exact_event_names: bool,
    unit_tokens: Vec<String>,
}

pub(crate) struct SignalEntityAccumulator {
    pub(crate) proposal_id: String,
    pub(crate) confidence: GraphConfidence,
    pub(crate) source_handle_ids: BTreeSet<StableHandleId>,
    pub(crate) evidence_ids: BTreeSet<EvidenceId>,
    pub(crate) coverage_ids: BTreeSet<wow_core::CoverageId>,
}

impl SignalEntityAccumulator {
    pub(crate) fn new(proposal_id: String, confidence: GraphConfidence) -> Self {
        Self {
            proposal_id,
            confidence,
            source_handle_ids: BTreeSet::new(),
            evidence_ids: BTreeSet::new(),
            coverage_ids: BTreeSet::new(),
        }
    }

    pub(crate) fn merge(
        &mut self,
        confidence: GraphConfidence,
        source_handle_ids: &[StableHandleId],
        evidence_ids: &[EvidenceId],
        coverage_ids: &[wow_core::CoverageId],
    ) {
        self.confidence = w1_merge_entity_confidence(self.confidence, confidence);
        self.source_handle_ids
            .extend(source_handle_ids.iter().copied());
        self.evidence_ids.extend(evidence_ids.iter().copied());
        self.coverage_ids.extend(coverage_ids.iter().copied());
    }
}

/// Recognizes `core.signal.native_frame_event@1` from the exact Emmy
/// function-call report. Unresolved receivers, library receivers and dynamic
/// event arguments keep the match Possible and never fabricate an event name.
pub fn w1_recognize_native_frame_events(
    input: W1Input<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<W1Proposals> {
    w1_checkpoint(stop)?;
    if input.report.calls().len() > W1_MAX_CALLS
        || input.report.functions().len() > W1_MAX_FUNCTIONS
        || input.function_proposals.len() != input.report.functions().len()
        || input.call_support.len() != input.report.calls().len()
    {
        return Err(w1_failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .report
        .validate()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    input
        .context
        .validate()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(w1_failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    // The lookup validates the exact owner once and resolves input-generation
    // receipts, so a materialized generation node ID is never used here.
    let lookup = input.owner.producer_lookup(stop).map_err(w1_graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    // Every enclosing function must cross into an accepted source_function
    // proposal. This producer owns no function identity of its own.
    let mut functions = BTreeMap::<String, W1FunctionBinding>::new();
    let mut function_proposal_ids = BTreeSet::new();
    for function in input.report.functions() {
        w1_checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let resolved = lookup
            .entity(
                lookup.scope(),
                &GraphAssertionRef::Producer {
                    partition_id: source_partition.partition_id().into(),
                    batch_id: source_partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: proposal_id.into(),
                    },
                },
                stop,
            )
            .map_err(w1_entity_binding_error)?;
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
            return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w1_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w1_validate_support(
            &input,
            *handle,
            *evidence,
            function.path(),
            function.content_digest(),
            function.span(),
        )?;
        let node = resolved.accepted().node().node_id().clone();
        if !function_proposal_ids.insert(proposal_id.to_owned())
            || functions
                .insert(
                    function.fact_id().to_owned(),
                    W1FunctionBinding {
                        node,
                        proposal_id: proposal_id.to_owned(),
                        handle: *handle,
                        evidence: *evidence,
                    },
                )
                .is_some()
        {
            return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    // Candidate registration sites, before any identity is derived. Only an exact
    // resolved callable key for the two reviewed shapes enters here.
    let mut sites = BTreeMap::<String, W1Site>::new();
    for call in input.report.calls() {
        w1_checkpoint(stop)?;
        let Some(callable) = call.resolved_callable_key() else {
            continue;
        };
        let Some(kind) = w1_registration_kind(callable) else {
            continue;
        };
        // Both reviewed shapes are frame methods. A dot call carrying such a key
        // is a fact mismatch rather than a silently skipped candidate.
        if !call.is_colon_call() {
            return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w1_validate_support(
            &input,
            handle,
            evidence,
            call.path(),
            call.content_digest(),
            call.call_span(),
        )?;
        let caller = functions
            .get(call.caller_function_id())
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;

        let arguments = call.arguments();
        if arguments.len() > W1_MAX_ARGUMENTS {
            return Err(w1_failure(RecognizerErrorCode::BudgetExceeded));
        }
        let (event_names, event_exact) = w1_event_names(arguments, kind)?;
        let (unit_tokens, units_exact) = w1_unit_tokens(arguments, kind)?;
        let exact_event_names = event_exact && units_exact;
        let site = W1Site {
            call_id: call.fact_id().to_owned(),
            kind,
            caller: W1FunctionBinding {
                node: caller.node.clone(),
                proposal_id: caller.proposal_id.clone(),
                handle: caller.handle,
                evidence: caller.evidence,
            },
            handle,
            evidence,
            event_names,
            exact_event_names,
            unit_tokens,
        };
        if sites.insert(site.call_id.clone(), site).is_some() {
            return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    for (call_id, site) in &sites {
        w1_checkpoint(stop)?;
        let mut fields = BTreeMap::from([
            (
                "call_id".into(),
                RecognizerFactValue::Reference(call_id.as_str().into()),
            ),
            (
                "caller".into(),
                RecognizerFactValue::Reference(site.caller.proposal_id.as_str().into()),
            ),
            (
                "callable_key".into(),
                RecognizerFactValue::String(site.kind.callable().into()),
            ),
            (
                "unit_event".into(),
                RecognizerFactValue::Boolean(site.kind == W1RegistrationKind::FrameUnitEvent),
            ),
            ("colon_call".into(), RecognizerFactValue::Boolean(true)),
            (
                "exact_event_names".into(),
                RecognizerFactValue::Boolean(site.exact_event_names),
            ),
            (
                "has_event_names".into(),
                RecognizerFactValue::Boolean(!site.event_names.is_empty()),
            ),
        ]);
        for (index, name) in site.event_names.iter().take(W1_MAX_EVENT_NAMES).enumerate() {
            fields.insert(
                format!("event_{index}").into_boxed_str(),
                RecognizerFactValue::String(name.as_str().into()),
            );
        }
        for (index, token) in site.unit_tokens.iter().take(W1_MAX_UNIT_TOKENS).enumerate() {
            fields.insert(
                format!("unit_token_{index}").into_boxed_str(),
                RecognizerFactValue::String(token.as_str().into()),
            );
        }
        facts.push(RecognizerFact::new(
            input.context.context_id(),
            RecognizerFactInput {
                kind: "lua_native_frame_event_call".into(),
                partition_id: W1_FACT_PARTITION.into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    site.caller.proposal_id.clone(),
                )?,
                producer_id: "wow.emmy".into(),
                producer_version: W1_FACT_PROFILE.into(),
                confidence: if site.exact_event_names {
                    GraphConfidence::Derived
                } else {
                    GraphConfidence::Possible
                },
                fields,
                source_handle_ids: vec![site.handle],
                evidence_ids: vec![site.evidence],
            },
            fact_limits,
        )?);
    }

    let coverage_state = if input.report.source_health_complete() {
        RecognizerFactCoverageState::Complete
    } else {
        RecognizerFactCoverageState::NotEvaluated
    };
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W1_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: W1_FACT_PROFILE.into(),
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
        W1_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = w1_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    // Only this rule and this rule version may contribute outcomes here.
    for outcome in output.outcomes() {
        if outcome.rule_id() != W1_RULE || outcome.rule_version() != W1_RULE_VERSION {
            return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
    }

    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut event_by_call = BTreeMap::<String, String>::new();
    let mut event_entities = BTreeMap::<String, SignalEntityAccumulator>::new();
    let mut registers_by_call = BTreeMap::<String, String>::new();

    // Proposal order is canonical by proposal ID, not by output kind. Resolve all
    // entities first so relation validation never depends on hash ordering.
    for outcome in output.outcomes() {
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Entity {
                proposal_id: _,
                entity_kind_id,
                semantic_key,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } = proposal
            else {
                continue;
            };
            if entity_kind_id.as_ref() != W1_EVENT_ENTITY || semantic_key.len() != 1 {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let Some(RecognizerFactValue::Reference(call_id)) = semantic_key.get("call") else {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let site = sites
                .get(call_id.as_ref())
                .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let [event_key] = site.event_names.as_slice() else {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let entity_id = w1_event_entity_id(event_key)?;
            let graph_confidence = w1_graph_confidence(*confidence);
            let accumulator = event_entities.entry(event_key.clone()).or_insert_with(|| {
                SignalEntityAccumulator::new(entity_id.clone(), graph_confidence)
            });
            if accumulator.proposal_id != entity_id {
                return Err(w1_failure(RecognizerErrorCode::AdapterIdentityMismatch));
            }
            accumulator.merge(
                graph_confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            );
            if event_by_call
                .insert(call_id.to_string(), entity_id)
                .is_some()
            {
                return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
        }
    }

    for (event_key, accumulator) in event_entities {
        w1_checkpoint(stop)?;
        entities.push(
            GraphEntityProposal::new(
                accumulator.proposal_id,
                W1_EVENT_ENTITY,
                BTreeMap::from([("event".into(), GraphProposalValue::String(event_key.into()))]),
                accumulator.confidence,
                accumulator.source_handle_ids.into_iter().collect(),
                accumulator.evidence_ids.into_iter().collect(),
                accumulator.coverage_ids.into_iter().collect(),
            )
            .map_err(w1_graph_error)?,
        );
    }

    for outcome in output.outcomes() {
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Relation {
                proposal_id,
                relation_kind_id,
                source,
                target,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } = proposal
            else {
                continue;
            };
            if relation_kind_id.as_ref() != W1_REGISTERS_DEFINITION {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let RecognizerFactValue::Reference(caller_proposal) = source else {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let RecognizerFactValue::Reference(call_id) = target else {
                return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let site = sites
                .get(call_id.as_ref())
                .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if site.caller.proposal_id != caller_proposal.as_ref() {
                return Err(w1_failure(RecognizerErrorCode::AdapterBindingMissing));
            }
            let event_id = event_by_call
                .get(call_id.as_ref())
                .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone();
            if registers_by_call
                .insert(call_id.to_string(), proposal_id.to_string())
                .is_some()
            {
                return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
            relations.push(
                GraphRelationProposal::new(
                    proposal_id.to_string(),
                    W1_REGISTERS_DEFINITION,
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Existing(site.caller.node.clone()),
                        target: GraphProposalEndpoint::Proposed(event_id.into()),
                        confidence: w1_graph_confidence(*confidence),
                        source_handle_ids: source_handle_ids.clone(),
                        evidence_ids: evidence_ids.clone(),
                        coverage_ids: coverage_ids.clone(),
                    },
                )
                .map_err(w1_graph_error)?,
            );
        }
    }

    // Every retained site needs exactly one event entity and one registers
    // relation. A missing pair is an adapter mismatch, never a silent drop.
    if sites.len() != event_by_call.len()
        || sites.len() != registers_by_call.len()
        || !sites.keys().all(|call_id| {
            event_by_call.contains_key(call_id) && registers_by_call.contains_key(call_id)
        })
    {
        return Err(w1_failure(RecognizerErrorCode::AdapterBindingMissing));
    }

    // Describe every stored relation family. This producer never certifies
    // source-owned relations and never grants absence authority.
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
            let (state, blocker) = if relation == GraphRelationKind::RegistersNativeEvent {
                (GraphCoverageState::Partial, W1_REGISTERS_BLOCKER)
            } else if relation == GraphRelationKind::HandlesNativeEvent {
                (GraphCoverageState::NotEvaluated, W1_HANDLES_BLOCKER)
            } else {
                (GraphCoverageState::NotEvaluated, W1_OTHER_RELATION_BLOCKER)
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(w1_graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;

    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        W1_SIGNAL_PARTITION,
        entities,
        relations,
    )
    .map_err(w1_graph_error)?;

    let mut matches = Vec::new();
    for (call_id, site) in &sites {
        w1_checkpoint(stop)?;
        if matches.len() >= W1_MAX_MATCHES {
            return Err(w1_failure(RecognizerErrorCode::BudgetExceeded));
        }
        matches.push(W1FrameEventMatch {
            call_id: call_id.clone(),
            registration_kind: site.kind,
            caller_function_proposal_id: site.caller.proposal_id.clone(),
            event_proposal_id: event_by_call
                .get(call_id)
                .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone(),
            event_names: site.event_names.clone(),
            exact_event_names: site.exact_event_names,
            unit_tokens: site.unit_tokens.clone(),
            registers_relation_proposal_id: registers_by_call
                .get(call_id)
                .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone(),
            handles_relation_proposal_ids: Vec::new(),
            handler_declaration_proposal_ids: Vec::new(),
        });
    }

    Ok(W1Proposals {
        batch,
        coverage: graph_coverage,
        recognition: W1Recognition {
            profile: W1_SIGNAL_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            matches,
        },
    })
}

fn w1_checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Relaxed) {
        return Err(w1_failure(RecognizerErrorCode::Cancelled));
    }
    Ok(())
}

fn w1_failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(
        code,
        "exact native frame-event facts could not produce a coherent W11 partition",
    )
}

fn w1_entity_binding_error(error: wow_graph::GraphError) -> RecognizerError {
    if error.code() == wow_graph::GraphErrorCode::PartitionInvalid {
        w1_failure(RecognizerErrorCode::AdapterBindingMissing)
    } else {
        w1_graph_error(error)
    }
}

fn w1_graph_error(error: wow_graph::GraphError) -> RecognizerError {
    w1_failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

fn w1_graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn w1_merge_entity_confidence(left: GraphConfidence, right: GraphConfidence) -> GraphConfidence {
    if left == GraphConfidence::Possible || right == GraphConfidence::Possible {
        GraphConfidence::Possible
    } else {
        GraphConfidence::Derived
    }
}

fn w1_event_entity_id(event_key: &str) -> RecognizerResult<String> {
    let bytes = canonical_json_bytes(&(W1_SIGNAL_PROFILE, W1_EVENT_ENTITY, event_key))
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "lua-native-event:sha256:{}",
        w1_encode_hex(&Sha256::digest(bytes))
    ))
}

/// Validate one support record against the exact source handle and evidence.
fn w1_validate_support(
    input: &W1Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    content_digest: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    let handle = input
        .source_handles
        .get(&handle_id)
        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let evidence = input
        .evidence
        .get(&evidence_id)
        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    handle
        .validate()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    evidence
        .validate()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let digest = content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if handle.handle_id() != handle_id
        || handle.path().as_str() != path
        || handle.span() != span
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
        return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}
// ===== END WORKER 1: native frame event =====

// ===== BEGIN WORKER 3: custom registry producer and subscription =====

pub const W3_SIGNAL_PARTITION: &str = "wow-recognizers.lua-custom-signals";
pub const W3_SIGNAL_PROFILE: &str = "wow-recognizers/lua-custom-signals/6";
const W3_FACT_PARTITION: &str = "wow-recognizers.lua-custom-signal-facts";
const W3_FACT_PROFILE: &str = "wow-recognizers-lua-custom-signal-facts-4";
const W3_PRODUCER_RULE: &str = "core.signal.custom_registry_producer";
const W3_SUBSCRIPTION_RULE: &str = "core.signal.custom_registry_subscription";
const W3_TRIGGER_EVENT_CALLABLE: &str = "EventRegistry.TriggerEvent";
const W3_REGISTER_CALLBACK_CALLABLE: &str = "EventRegistry.RegisterCallback";
const W3_CUSTOM_SIGNAL_FACT: &str = "lua_custom_signal_call";
const W3_EMITTER_RELATION_ID: &str = "source_emits_custom_signal";
const W3_SUBSCRIPTION_RELATION_ID: &str = "source_subscribes_custom_signal";
const W3_CUSTOM_SIGNAL_ENTITY: &str = "custom_signal";
const W3_MAX_CALLS: usize = 4096;
const W3_MAX_ARGUMENTS: usize = 4;
const W3_MAX_SITES: usize = 65_536;
const W3_MAX_EVENT_KEY_BYTES: usize = 1024;

const W3_POSITIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-001"];
const W3_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-009"];
const W3_PARTIAL_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-005"];
const W3_MUTATION_FIXTURE_IDS: [&str; 2] = ["RECOG-EVENT-006", "RECOG-EVENT-007"];

/// One resolved custom-signal producer site: a resolved custom-registry receiver
/// and one literal event key. A dynamic receiver or key never enters this block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3ProducerMatch {
    pub call_id: String,
    pub event_key: String,
    pub receiver_workspace_id: String,
    pub receiver_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_declaration_proposal_id: Option<String>,
    pub relation_proposal_id: String,
}

/// One resolved subscription site. confirmed is true only when an exact
/// compatible producer exists in this same report and revision.
/// One exact EventRegistry registration site and key; callback/owner targets are
/// not represented or inferred by this match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3SubscriptionMatch {
    pub call_id: String,
    pub event_key: String,
    pub receiver_workspace_id: String,
    pub receiver_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_declaration_proposal_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer_call_id: Option<String>,
    pub producer_call_ids: Vec<String>,
    pub relation_proposal_id: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum W3UnconfirmedReason {
    ProducerAbsentInReport,
    ProducerReceiverMismatch,
    ProducerEventKeyMismatch,
    ProducerLookupIncomplete,
}

/// A subscription that could not be confirmed against an exact producer. Retained
/// explicitly so absence of evidence is never reported as a clean negative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3UnconfirmedSubscription {
    pub call_id: String,
    pub event_key: String,
    pub receiver_workspace_id: String,
    pub receiver_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_declaration_proposal_id: Option<String>,
    pub reason: W3UnconfirmedReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3Recognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    producers: Vec<W3ProducerMatch>,
    subscriptions: Vec<W3SubscriptionMatch>,
    unconfirmed_subscriptions: Vec<W3UnconfirmedSubscription>,
}

impl W3Recognition {
    pub fn producers(&self) -> &[W3ProducerMatch] {
        &self.producers
    }

    pub fn subscriptions(&self) -> &[W3SubscriptionMatch] {
        &self.subscriptions
    }

    pub fn unconfirmed_subscriptions(&self) -> &[W3UnconfirmedSubscription] {
        &self.unconfirmed_subscriptions
    }
}

/// Exact reviewed registration shapes. Any other resolved callable key is not a
/// candidate for this rule and never becomes a near match.
fn w1_registration_kind(callable: &str) -> Option<W1RegistrationKind> {
    match callable {
        W1_REGISTER_EVENT_CALLABLE => Some(W1RegistrationKind::FrameEvent),
        W1_REGISTER_UNIT_EVENT_CALLABLE => Some(W1RegistrationKind::FrameUnitEvent),
        _ => None,
    }
}

/// Exact event-name literals from the declared argument slot. A dynamic or
/// non-string argument clears exactness without inventing any name.
fn w1_event_names(
    arguments: &[wow_emmy::function_calls::SourceCallArgument],
    kind: W1RegistrationKind,
) -> RecognizerResult<(Vec<String>, bool)> {
    let Some(argument) = arguments.get(kind.event_ordinal()) else {
        return Ok((Vec::new(), false));
    };
    match argument.literal() {
        Some(wow_emmy::function_calls::SourceCallLiteral::String(value)) => {
            if value.len() > W1_MAX_EVENT_BYTES {
                return Err(w1_failure(RecognizerErrorCode::BudgetExceeded));
            }
            if value.is_empty() {
                return Ok((Vec::new(), false));
            }
            Ok((vec![value.clone()], true))
        }
        _ => Ok((Vec::new(), false)),
    }
}

fn w1_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-native-frame-events".into(),
            version: "4".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W1_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-native-frame-event-4".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 4,
                max_clauses_per_rule: 32,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 10_000,
                max_proposals_per_rule_partition: 20_000,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![RecognizerRule {
                rule_id: W1_RULE.into(),
                version: W1_RULE_VERSION,
                required_capabilities: vec!["emmy.fact.calls".into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "call".into(),
                        kind: "lua_native_frame_event_call".into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.colon_call".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.has_event_names".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.exact_event_names".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                ],
                captures: vec![
                    crate::RecognizerCapture {
                        name: "event_0".into(),
                        value_type: "bounded_string".into(),
                        source: "call.event_0".into(),
                        cardinality: crate::RecognizerCaptureCardinality::Optional,
                    },
                    crate::RecognizerCapture {
                        name: "unit_token_0".into(),
                        value_type: "bounded_string".into(),
                        source: "call.unit_token_0".into(),
                        cardinality: crate::RecognizerCaptureCardinality::Optional,
                    },
                ],
                outputs: vec![
                    RecognizerOutput::EntityAssertion {
                        output_id: "native_frame_event_entity".into(),
                        entity_kind_id: W1_EVENT_ENTITY.into(),
                        semantic_key: BTreeMap::from([("call".into(), "call.call_id".into())]),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "native_frame_event_registers".into(),
                        relation_kind_id: W1_REGISTERS_DEFINITION.into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                ],
                positive_fixture_ids: (W1_POSITIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                near_negative_fixture_ids: (W1_NEAR_NEGATIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                partial_fixture_ids: (W1_PARTIAL_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                mutation_fixture_ids: (W1_MUTATION_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| w1_failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

/// Exact ordered unit token from the declared argument slot. Only an exact
/// string literal becomes a token; a dynamic token drops the whole list.
fn w1_encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}
fn w1_unit_tokens(
    arguments: &[wow_emmy::function_calls::SourceCallArgument],
    kind: W1RegistrationKind,
) -> RecognizerResult<(Vec<String>, bool)> {
    let Some(start) = kind.unit_ordinal() else {
        return Ok((Vec::new(), true));
    };
    if arguments.len() <= start {
        return Ok((Vec::new(), false));
    }
    let mut tokens = Vec::new();
    for argument in &arguments[start..] {
        let Some(wow_emmy::function_calls::SourceCallLiteral::String(value)) = argument.literal()
        else {
            return Ok((Vec::new(), false));
        };
        if value.is_empty() || value.len() > W1_MAX_EVENT_BYTES {
            return if value.len() > W1_MAX_EVENT_BYTES {
                Err(w1_failure(RecognizerErrorCode::BudgetExceeded))
            } else {
                Ok((Vec::new(), false))
            };
        }
        if tokens.len() >= W1_MAX_UNIT_TOKENS {
            return Err(w1_failure(RecognizerErrorCode::BudgetExceeded));
        }
        tokens.push(value.clone());
    }
    Ok((tokens, true))
}

pub struct W3Proposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: W3Recognition,
}

/// Caller-side crosswalks are checked against the real source proposals, support
/// records and the exact analyzer report. No name, path or repository text is
/// ever consulted, and no source is reparsed.
pub struct W3Input<'a> {
    pub owner: &'a GraphPartitionSnapshot,
    pub source_partition: &'a str,
    pub report: &'a FunctionCallReport,
    pub context: &'a GenerationContext,
    pub function_proposals: BTreeMap<&'a str, &'a str>,
    pub declaration_proposals: BTreeMap<(String, SourceSpan), &'a str>,
    pub call_support: BTreeMap<&'a str, (StableHandleId, EvidenceId)>,
    pub source_handles: &'a BTreeMap<StableHandleId, SourceHandle>,
    pub evidence: &'a BTreeMap<EvidenceId, EvidenceRecord>,
}

/// One resolved Main declaration location owning its node and support records.
#[derive(Debug, Clone, PartialEq, Eq)]
struct W3Binding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

/// One exact Main declaration identity, used to recheck a resolved receiver.
struct W3ReceiverTarget {
    workspace_id: String,
    path: String,
    span: SourceSpan,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum W3SignalSide {
    Producer,
    Subscription,
}

impl W3SignalSide {
    const fn callable(self) -> &'static str {
        match self {
            Self::Producer => W3_TRIGGER_EVENT_CALLABLE,
            Self::Subscription => W3_REGISTER_CALLBACK_CALLABLE,
        }
    }

    const fn rule_id(self) -> &'static str {
        match self {
            Self::Producer => W3_PRODUCER_RULE,
            Self::Subscription => W3_SUBSCRIPTION_RULE,
        }
    }

    const fn relation_id(self) -> &'static str {
        match self {
            Self::Producer => W3_EMITTER_RELATION_ID,
            Self::Subscription => W3_SUBSCRIPTION_RELATION_ID,
        }
    }
}

/// One exact EventRegistry call site with resolved receiver support and a literal
/// event key. Callback and owner values are not relation endpoints here.
struct W3Site {
    fact_id: String,
    call_id: String,
    caller_proposal_id: String,
    caller_node: GraphNodeId,
    receiver_proposal_id: Option<String>,
    receiver_workspace_id: String,
    receiver_identity: String,
    receiver_handle: Option<StableHandleId>,
    receiver_evidence: Option<EvidenceId>,
    event_key: String,
    event_ordinal: usize,
    argument_count: usize,
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Clone)]
enum W3Assertion {
    Emitter {
        call_id: String,
        caller_proposal_id: String,
        receiver_proposal_id: Option<String>,
        receiver_workspace_id: String,
        receiver_identity: String,
        event_key: String,
        relation_proposal_id: String,
        confidence: RecognizerOutputConfidence,
        source_handle_ids: Vec<StableHandleId>,
        evidence_ids: Vec<EvidenceId>,
        coverage_ids: Vec<wow_core::CoverageId>,
    },
    Subscription {
        call_id: String,
        caller_proposal_id: String,
        receiver_proposal_id: Option<String>,
        receiver_workspace_id: String,
        receiver_identity: String,
        event_key: String,
        relation_proposal_id: String,
        producer_call_ids: Vec<String>,
        confidence: RecognizerOutputConfidence,
        source_handle_ids: Vec<StableHandleId>,
        evidence_ids: Vec<EvidenceId>,
        coverage_ids: Vec<wow_core::CoverageId>,
    },
}

/// Recognizes resolved custom-registry producers and subscriptions from the exact
/// Emmy function-call report. Nothing about runtime dispatch, delivery or taint is
/// claimed; only the observed structural fact is proposed.
pub fn w3_recognize_signals(
    input: W3Input<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<W3Proposals> {
    w3_checkpoint(stop)?;
    if input.report.calls().len() > W3_MAX_CALLS
        || input.call_support.len() != input.report.calls().len()
        || input.function_proposals.len() != input.report.functions().len()
        || input.declaration_proposals.len() > W3_MAX_CALLS
    {
        return Err(w3_failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .report
        .validate()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    input
        .context
        .validate()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(w3_failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let lookup = input.owner.producer_lookup(stop).map_err(w3_graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let mut function_proposals = BTreeMap::<String, String>::new();
    let mut function_nodes = BTreeMap::<String, GraphNodeId>::new();
    let mut function_ids = BTreeSet::new();
    for function in input.report.functions() {
        w3_checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let resolved = lookup
            .entity(
                lookup.scope(),
                &GraphAssertionRef::Producer {
                    partition_id: source_partition.partition_id().into(),
                    batch_id: source_partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: proposal_id.into(),
                    },
                },
                stop,
            )
            .map_err(w3_entity_binding_error)?;
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
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w3_validate_support_record(
            &input,
            *handle,
            *evidence,
            function.path(),
            function.span(),
            Some(
                function
                    .content_digest()
                    .parse::<ContentDigest<SourceContent>>()
                    .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?,
            ),
        )?;
        let node = resolved.accepted().node().node_id().clone();
        if !function_ids.insert(function.fact_id().to_owned()) {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        function_proposals.insert(function.fact_id().to_owned(), proposal_id.to_owned());
        function_nodes.insert(function.fact_id().to_owned(), node);
    }

    let mut declarations = BTreeMap::<(String, SourceSpan), W3Binding>::new();
    let mut declaration_keys = BTreeMap::<String, W3ReceiverTarget>::new();
    let mut receiver_proposal_ids = BTreeSet::<String>::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        w3_checkpoint(stop)?;
        let resolved = lookup
            .entity(
                lookup.scope(),
                &GraphAssertionRef::Producer {
                    partition_id: source_partition.partition_id().into(),
                    batch_id: source_partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: (*proposal_id).into(),
                    },
                },
                stop,
            )
            .map_err(w3_entity_binding_error)?;
        let proposal = resolved.proposal();
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let expected = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String((*path).clone().into()),
            ),
            (
                "span_start".into(),
                GraphProposalValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
        ]);
        if proposal.entity_kind_id() != "lua_source_declaration"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w3_validate_support_without_digest(&input, *handle, *evidence, path, *span)?;
        let node = resolved.accepted().node().node_id().clone();
        if !receiver_proposal_ids.insert((*proposal_id).to_owned()) {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        declaration_keys.insert(
            (*proposal_id).to_owned(),
            W3ReceiverTarget {
                workspace_id: input.report.main_snapshot_id().to_owned(),
                path: (*path).clone(),
                span: *span,
            },
        );
        if declarations
            .insert(
                ((*path).clone(), *span),
                W3Binding {
                    node,
                    handle: *handle,
                    evidence: *evidence,
                },
            )
            .is_some()
        {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut sites = BTreeMap::<String, W3Site>::new();
    let mut site_count = 0usize;
    for call in input.report.calls() {
        w3_checkpoint(stop)?;
        let Some(side) = w3_side(call) else {
            continue;
        };
        let Some(site) = w3_build_site(
            &input,
            call,
            &declarations,
            &declaration_keys,
            &function_nodes,
        )?
        else {
            continue;
        };
        site_count = site_count
            .checked_add(1)
            .ok_or_else(|| w3_failure(RecognizerErrorCode::BudgetExceeded))?;
        if site_count > W3_MAX_SITES {
            return Err(w3_failure(RecognizerErrorCode::BudgetExceeded));
        }
        let fact = w3_signal_fact(&input, &site, side, fact_limits)?;
        facts.push(fact);
        if sites.insert(site.call_id.clone(), site).is_some() {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let coverage_state = if input.report.source_health_complete() {
        RecognizerFactCoverageState::Complete
    } else {
        RecognizerFactCoverageState::NotEvaluated
    };
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W3_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: W3_FACT_PROFILE.into(),
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
        W3_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = w3_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut assertions = Vec::new();
    let empty_producers = BTreeMap::<(String, String), Vec<String>>::new();
    let mut producer_proof_refs = 0usize;
    for outcome in output.outcomes() {
        if outcome.rule_id() == W3_PRODUCER_RULE && outcome.rule_version() == 1 {
            for proposal in outcome.proposals() {
                assertions.push(w3_read_assertion(
                    proposal,
                    W3SignalSide::Producer,
                    &bundle,
                    &empty_producers,
                    &mut producer_proof_refs,
                )?);
            }
        } else if outcome.rule_id() != W3_SUBSCRIPTION_RULE || outcome.rule_version() != 1 {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
    }

    let mut producer_calls = BTreeMap::<(String, String), Vec<String>>::new();
    for assertion in &assertions {
        if let W3Assertion::Emitter {
            call_id,
            receiver_identity,
            event_key,
            ..
        } = assertion
        {
            producer_calls
                .entry((receiver_identity.clone(), event_key.clone()))
                .or_default()
                .push(call_id.clone());
        }
    }
    for calls in producer_calls.values_mut() {
        calls.sort();
        calls.dedup();
    }
    for outcome in output.outcomes() {
        if outcome.rule_id() == W3_SUBSCRIPTION_RULE && outcome.rule_version() == 1 {
            for proposal in outcome.proposals() {
                assertions.push(w3_read_assertion(
                    proposal,
                    W3SignalSide::Subscription,
                    &bundle,
                    &producer_calls,
                    &mut producer_proof_refs,
                )?);
            }
        }
    }

    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut producer_matches = Vec::new();
    let mut subscription_matches = Vec::new();
    let mut signal_entities = BTreeMap::<String, SignalEntityAccumulator>::new();
    for assertion in &assertions {
        let (
            call_id,
            caller_proposal_id,
            receiver_proposal_id,
            receiver_workspace_id,
            receiver_identity,
            event_key,
            relation_proposal_id,
            confidence,
            producer_call_ids,
            source_handle_ids,
            evidence_ids,
            coverage_ids,
        ) = match assertion {
            W3Assertion::Emitter {
                call_id,
                caller_proposal_id,
                receiver_proposal_id,
                receiver_workspace_id,
                receiver_identity,
                event_key,
                relation_proposal_id,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            } => (
                call_id,
                caller_proposal_id,
                receiver_proposal_id,
                receiver_workspace_id,
                receiver_identity,
                event_key,
                relation_proposal_id,
                *confidence,
                &[][..],
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            ),
            W3Assertion::Subscription {
                call_id,
                caller_proposal_id,
                receiver_proposal_id,
                receiver_workspace_id,
                receiver_identity,
                event_key,
                relation_proposal_id,
                producer_call_ids,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            } => (
                call_id,
                caller_proposal_id,
                receiver_proposal_id,
                receiver_workspace_id,
                receiver_identity,
                event_key,
                relation_proposal_id,
                *confidence,
                producer_call_ids.as_slice(),
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            ),
        };
        let site = sites
            .get(call_id.as_str())
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if site.caller_proposal_id != *caller_proposal_id
            || site.receiver_proposal_id != *receiver_proposal_id
            || site.receiver_workspace_id != *receiver_workspace_id
            || site.receiver_identity != *receiver_identity
            || site.event_key != *event_key
        {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let mut handles = source_handle_ids.iter().copied().collect::<BTreeSet<_>>();
        let mut evidence = evidence_ids.iter().copied().collect::<BTreeSet<_>>();
        handles.insert(site.handle);
        evidence.insert(site.evidence);
        if let Some(handle) = site.receiver_handle {
            handles.insert(handle);
        }
        if let Some(receiver_evidence) = site.receiver_evidence {
            evidence.insert(receiver_evidence);
        }
        for producer_call_id in producer_call_ids {
            let producer = sites
                .get(producer_call_id.as_str())
                .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if producer.receiver_identity != *receiver_identity
                || producer.receiver_workspace_id != *receiver_workspace_id
                || producer.event_key != *event_key
            {
                return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            handles.insert(producer.handle);
            evidence.insert(producer.evidence);
            if let Some(handle) = producer.receiver_handle {
                handles.insert(handle);
            }
            if let Some(receiver_evidence) = producer.receiver_evidence {
                evidence.insert(receiver_evidence);
            }
        }
        let handles = handles.into_iter().collect::<Vec<_>>();
        let evidence = evidence.into_iter().collect::<Vec<_>>();
        let graph_confidence = if matches!(assertion, W3Assertion::Subscription { .. })
            && producer_call_ids.is_empty()
        {
            GraphConfidence::Possible
        } else {
            w3_graph_confidence(confidence)
        };
        let signal_key = w3_signal_entity_key(receiver_identity, event_key);
        let signal_id = w3_entity_id(&signal_key)?;
        let accumulator = signal_entities
            .entry(signal_key)
            .or_insert_with(|| SignalEntityAccumulator::new(signal_id.clone(), graph_confidence));
        if accumulator.proposal_id != signal_id {
            return Err(w3_failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        accumulator.merge(graph_confidence, &handles, &evidence, coverage_ids);
        relations.push(
            GraphRelationProposal::new(
                relation_proposal_id.as_str(),
                match assertion {
                    W3Assertion::Emitter { .. } => W3_EMITTER_RELATION_ID,
                    W3Assertion::Subscription { .. } => W3_SUBSCRIPTION_RELATION_ID,
                },
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(site.caller_node.clone()),
                    target: GraphProposalEndpoint::Proposed(signal_id.into()),
                    confidence: graph_confidence,
                    source_handle_ids: handles,
                    evidence_ids: evidence,
                    coverage_ids: coverage_ids.clone(),
                },
            )
            .map_err(w3_graph_error)?,
        );
        match assertion {
            W3Assertion::Emitter { .. } => producer_matches.push(W3ProducerMatch {
                call_id: call_id.clone(),
                event_key: event_key.clone(),
                receiver_workspace_id: receiver_workspace_id.clone(),
                receiver_identity: receiver_identity.clone(),
                receiver_declaration_proposal_id: receiver_proposal_id.clone(),
                relation_proposal_id: relation_proposal_id.clone(),
            }),
            W3Assertion::Subscription { .. } => {
                subscription_matches.push(W3SubscriptionMatch {
                    call_id: call_id.clone(),
                    event_key: event_key.clone(),
                    receiver_workspace_id: receiver_workspace_id.clone(),
                    receiver_identity: receiver_identity.clone(),
                    receiver_declaration_proposal_id: receiver_proposal_id.clone(),
                    producer_call_id: producer_call_ids.first().cloned(),
                    producer_call_ids: producer_call_ids.to_vec(),
                    relation_proposal_id: relation_proposal_id.clone(),
                    confirmed: !producer_call_ids.is_empty(),
                });
            }
        }
    }

    for (signal_key, accumulator) in signal_entities {
        w3_checkpoint(stop)?;
        entities.push(
            GraphEntityProposal::new(
                accumulator.proposal_id,
                W3_CUSTOM_SIGNAL_ENTITY,
                BTreeMap::from([(
                    "signal".into(),
                    GraphProposalValue::String(signal_key.into()),
                )]),
                accumulator.confidence,
                accumulator.source_handle_ids.into_iter().collect(),
                accumulator.evidence_ids.into_iter().collect(),
                accumulator.coverage_ids.into_iter().collect(),
            )
            .map_err(w3_graph_error)?,
        );
    }

    let source_health_complete = input.report.source_health_complete();
    let mut unconfirmed = Vec::new();
    for subscription in &subscription_matches {
        if subscription.confirmed {
            continue;
        }
        unconfirmed.push(W3UnconfirmedSubscription {
            call_id: subscription.call_id.clone(),
            event_key: subscription.event_key.clone(),
            receiver_workspace_id: subscription.receiver_workspace_id.clone(),
            receiver_identity: subscription.receiver_identity.clone(),
            receiver_declaration_proposal_id: subscription.receiver_declaration_proposal_id.clone(),
            reason: w3_unconfirmed_reason(
                &subscription.receiver_identity,
                &subscription.event_key,
                &producer_matches,
                source_health_complete,
            ),
        });
    }

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
                GraphRelationKind::EmitsCustomSignal => (
                    GraphCoverageState::Partial,
                    "lua_custom_signals.exact_trigger_event_receiver_and_key_only",
                ),
                GraphRelationKind::HandlesCustomSignal => (
                    GraphCoverageState::Partial,
                    "lua_custom_signals.exact_register_callback_receiver_and_key_only",
                ),
                _ => (
                    GraphCoverageState::NotEvaluated,
                    "lua_custom_signals.relation_owned_by_other_producer",
                ),
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(w3_graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        W3_SIGNAL_PARTITION,
        entities,
        relations,
    )
    .map_err(w3_graph_error)?;

    Ok(W3Proposals {
        batch,
        coverage: graph_coverage,
        recognition: W3Recognition {
            profile: W3_SIGNAL_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            producers: producer_matches,
            subscriptions: subscription_matches,
            unconfirmed_subscriptions: unconfirmed,
        },
    })
}

/// Only a colon-form EventRegistry:TriggerEvent or EventRegistry:RegisterCallback
/// whose resolved callable key is exact enters this producer.
fn w3_side(call: &wow_emmy::function_calls::SourceCallFact) -> Option<W3SignalSide> {
    if !call.is_colon_call() {
        return None;
    }
    match call.resolved_callable_key() {
        Some(W3_TRIGGER_EVENT_CALLABLE) => Some(W3SignalSide::Producer),
        Some(W3_REGISTER_CALLBACK_CALLABLE) => Some(W3SignalSide::Subscription),
        _ => None,
    }
}

/// A site exists only with a resolved Main custom-registry receiver and one
/// literal event key. A dynamic receiver, a library target, or a non-literal key
/// yields no fact rather than a degraded one.
fn w3_build_site(
    input: &W3Input<'_>,
    call: &wow_emmy::function_calls::SourceCallFact,
    declarations: &BTreeMap<(String, SourceSpan), W3Binding>,
    declaration_keys: &BTreeMap<String, W3ReceiverTarget>,
    function_nodes: &BTreeMap<String, GraphNodeId>,
) -> RecognizerResult<Option<W3Site>> {
    let (handle, evidence) = *input
        .call_support
        .get(call.fact_id())
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    w3_validate_support(input, handle, evidence, call)?;

    let arguments = call.arguments();
    if arguments.len() > W3_MAX_ARGUMENTS {
        return Ok(None);
    }
    // The event key is the first positional argument on both sides.
    let ordinal = 0;
    let event = arguments.get(ordinal).and_then(|argument| {
        argument.literal().and_then(|literal| match literal {
            wow_emmy::function_calls::SourceCallLiteral::String(value) => Some(value),
            _ => None,
        })
    });
    let Some(literal_key) = event else {
        return Ok(None);
    };
    // String literals are already decoded here; their exact value is the key.
    let event_key = literal_key.to_owned();
    if event_key.is_empty() || event_key.len() > W3_MAX_EVENT_KEY_BYTES {
        return Ok(None);
    }

    let Some(receiver) = input.report.exact_call_receiver(call) else {
        return Ok(None);
    };
    let target = receiver.target();
    if !w3_is_resolved_custom_registry(Some(receiver.key()), target, input.report) {
        return Ok(None);
    }
    let receiver_identity = source_receiver_identity(target)?;
    let (receiver_proposal_id, receiver_handle, receiver_evidence) = if target.role == "main" {
        let declaration_proposal = input
            .declaration_proposals
            .iter()
            .find(|((path, span), _)| *path == target.path && *span == target.span)
            .map(|(_, proposal)| *proposal)
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let binding = declarations
            .get(&(target.path.clone(), target.span))
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w3_validate_receiver(
            input,
            target,
            binding,
            declaration_proposal,
            declaration_keys,
        )?;
        (
            Some(declaration_proposal.to_owned()),
            Some(binding.handle),
            Some(binding.evidence),
        )
    } else {
        w3_validate_library_receiver(input.report, target)?;
        (None, None, None)
    };

    for argument in arguments {
        if argument.span().byte_start().is_none() || argument.span().byte_end().is_none() {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
    }
    // This fact asserts only the exact EventRegistry callsite and decoded key.
    // The emitted graph endpoints are caller and signal key; callback and owner
    // identities are not represented by this producer.
    let function_proposals = function_proposals_of(input);
    let caller_proposal = function_proposals
        .get(call.caller_function_id())
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let caller_node = function_nodes
        .get(call.caller_function_id())
        .cloned()
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;

    Ok(Some(W3Site {
        fact_id: call.fact_id().to_owned(),
        call_id: call.fact_id().to_owned(),
        caller_proposal_id: (*caller_proposal).to_owned(),
        caller_node,
        receiver_proposal_id,
        receiver_workspace_id: target.workspace_id.clone(),
        receiver_identity,
        receiver_handle,
        receiver_evidence,
        event_key,
        event_ordinal: ordinal,
        argument_count: arguments.len(),
        handle,
        evidence,
    }))
}

fn function_proposals_of<'a>(input: &W3Input<'a>) -> BTreeMap<&'a str, &'a str> {
    input.function_proposals.clone()
}

/// Receivers are exact Main project-local registries or exact targets in the
/// selected Library semantic session. Their workspace identity remains distinct.
fn w3_is_resolved_custom_registry(
    receiver_key: Option<&str>,
    target: &wow_emmy::bindings::SymbolTarget,
    report: &FunctionCallReport,
) -> bool {
    let workspace_matches = target.role == "main"
        && target.workspace_id == report.main_snapshot_id()
        || target.role == "library" && report.library_snapshot_ids().contains(&target.workspace_id);
    receiver_key == Some("EventRegistry")
        && workspace_matches
        && !target.path.is_empty()
        && !target.content_digest.is_empty()
        && target
            .span
            .byte_start()
            .zip(target.span.byte_end())
            .is_some_and(|(start, end)| start < end)
}

fn w3_validate_receiver(
    input: &W3Input<'_>,
    target: &wow_emmy::bindings::SymbolTarget,
    binding: &W3Binding,
    proposal_id: &str,
    declaration_keys: &BTreeMap<String, W3ReceiverTarget>,
) -> RecognizerResult<()> {
    if target.role != "main" || target.workspace_id != input.report.main_snapshot_id() {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let digest = target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let handle = input
        .source_handles
        .get(&binding.handle)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if handle.path().as_str() != target.path
        || handle.span() != target.span
        || handle.content_digest() != &digest
    {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let record = declaration_keys
        .get(proposal_id)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if record.workspace_id != target.workspace_id
        || record.path != target.path
        || record.span != target.span
    {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn w3_validate_library_receiver(
    report: &FunctionCallReport,
    target: &wow_emmy::bindings::SymbolTarget,
) -> RecognizerResult<()> {
    if target.role != "library"
        || !report.library_snapshot_ids().contains(&target.workspace_id)
        || target.path.is_empty()
        || target
            .span
            .byte_start()
            .zip(target.span.byte_end())
            .is_none_or(|(start, end)| start >= end)
    {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    Ok(())
}

fn source_receiver_identity(target: &wow_emmy::bindings::SymbolTarget) -> RecognizerResult<String> {
    let digest = target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let bytes = canonical_json_bytes(&(
        "wow-recognizers/resolved-source-receiver/1",
        target.role,
        target.workspace_id.as_str(),
        target.path.as_str(),
        target.span,
        digest,
    ))
    .map_err(|_| w3_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "receiver:sha256:{}",
        w3_hex(&Sha256::digest(bytes))
    ))
}

fn w3_signal_fact(
    input: &W3Input<'_>,
    site: &W3Site,
    side: W3SignalSide,
    limits: RecognizerFactLimits,
) -> RecognizerResult<RecognizerFact> {
    let mut fields = BTreeMap::from([
        (
            "call_id".into(),
            RecognizerFactValue::Reference(site.call_id.clone().into()),
        ),
        (
            "caller".into(),
            RecognizerFactValue::Reference(site.caller_proposal_id.clone().into()),
        ),
        (
            "callable_key".into(),
            RecognizerFactValue::String(side.callable().into()),
        ),
        ("colon_call".into(), RecognizerFactValue::Boolean(true)),
        (
            "receiver".into(),
            RecognizerFactValue::Reference(site.receiver_identity.clone().into()),
        ),
        (
            "receiver_workspace_id".into(),
            RecognizerFactValue::String(site.receiver_workspace_id.clone().into()),
        ),
        (
            "event_key".into(),
            RecognizerFactValue::String(site.event_key.clone().into()),
        ),
        (
            "event_ordinal".into(),
            RecognizerFactValue::Integer(
                i64::try_from(site.event_ordinal)
                    .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?,
            ),
        ),
        (
            "argument_count".into(),
            RecognizerFactValue::Integer(
                i64::try_from(site.argument_count)
                    .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?,
            ),
        ),
    ]);
    if let Some(proposal_id) = &site.receiver_proposal_id {
        fields.insert(
            "receiver_declaration_proposal_id".into(),
            RecognizerFactValue::Reference(proposal_id.clone().into()),
        );
    }
    let caller_function_id = input
        .report
        .calls()
        .iter()
        .find(|call| call.fact_id() == site.call_id.as_str())
        .map(|call| call.caller_function_id())
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let mut source_handle_ids = BTreeSet::from([site.handle]);
    let mut evidence_ids = BTreeSet::from([site.evidence]);
    if let Some(handle) = site.receiver_handle {
        source_handle_ids.insert(handle);
    }
    if let Some(evidence) = site.receiver_evidence {
        evidence_ids.insert(evidence);
    }
    RecognizerFact::new(
        input.context.context_id(),
        RecognizerFactInput {
            kind: W3_CUSTOM_SIGNAL_FACT.into(),
            partition_id: W3_FACT_PARTITION.into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                caller_function_id.to_owned(),
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: W3_FACT_PROFILE.into(),
            confidence: GraphConfidence::Derived,
            fields,
            source_handle_ids: source_handle_ids.into_iter().collect(),
            evidence_ids: evidence_ids.into_iter().collect(),
        },
        limits,
    )
}

/// Re-reads one plan output and rebinds it to the exact fact. Graph relations
/// originate from the enclosing source function; the exact receiver remains
/// retained evidence and producer/subscriber compatibility input.
fn w3_read_assertion(
    proposal: &crate::RecognizerProposedAssertion,
    side: W3SignalSide,
    bundle: &RecognizerFactBundle,
    producer_calls: &BTreeMap<(String, String), Vec<String>>,
    producer_proof_refs: &mut usize,
) -> RecognizerResult<W3Assertion> {
    let crate::RecognizerProposedAssertion::Relation {
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
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    if relation_kind_id.as_ref() != side.relation_id() {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let [fact_id] = decisive_fact_ids.as_slice() else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let fact = bundle
        .fact_by_id(fact_id)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if fact.kind() != W3_CUSTOM_SIGNAL_FACT
        || source_handle_ids.as_slice() != fact.source_handle_ids()
        || evidence_ids.as_slice() != fact.evidence_ids()
    {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::Reference(caller)) = fact.field("caller") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::Reference(receiver)) = fact.field("receiver") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::String(receiver_workspace_id)) =
        fact.field("receiver_workspace_id")
    else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let receiver_proposal_id = match fact.field("receiver_declaration_proposal_id") {
        Some(RecognizerFactValue::Reference(proposal_id)) => Some(proposal_id.to_string()),
        None => None,
        _ => return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch)),
    };
    let Some(RecognizerFactValue::String(event_key)) = fact.field("event_key") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::String(callable_key)) = fact.field("callable_key") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    if callable_key.as_ref() != side.callable()
        || source != &RecognizerFactValue::Reference(caller.clone())
        || target != &RecognizerFactValue::String(event_key.clone())
    {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let producer_call_ids = match side {
        W3SignalSide::Producer => Vec::new(),
        W3SignalSide::Subscription => {
            let calls = producer_calls
                .get(&(receiver.to_string(), event_key.to_string()))
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let next = producer_proof_refs
                .checked_add(calls.len())
                .ok_or_else(|| w3_failure(RecognizerErrorCode::BudgetExceeded))?;
            if next > W3_MAX_SITES {
                return Err(w3_failure(RecognizerErrorCode::BudgetExceeded));
            }
            *producer_proof_refs = next;
            calls.to_vec()
        }
    };
    Ok(match side {
        W3SignalSide::Producer => W3Assertion::Emitter {
            call_id: call_id.to_string(),
            caller_proposal_id: caller.to_string(),
            receiver_proposal_id,
            receiver_workspace_id: receiver_workspace_id.to_string(),
            receiver_identity: receiver.to_string(),
            event_key: event_key.to_string(),
            relation_proposal_id: w3_relation_id("emitter", call_id.as_ref()),
            confidence: *confidence,
            source_handle_ids: source_handle_ids.clone(),
            evidence_ids: evidence_ids.clone(),
            coverage_ids: coverage_ids.clone(),
        },
        W3SignalSide::Subscription => W3Assertion::Subscription {
            call_id: call_id.to_string(),
            caller_proposal_id: caller.to_string(),
            receiver_proposal_id,
            receiver_workspace_id: receiver_workspace_id.to_string(),
            receiver_identity: receiver.to_string(),
            event_key: event_key.to_string(),
            relation_proposal_id: w3_relation_id("subscription", call_id.as_ref()),
            producer_call_ids,
            confidence: *confidence,
            source_handle_ids: source_handle_ids.clone(),
            evidence_ids: evidence_ids.clone(),
            coverage_ids: coverage_ids.clone(),
        },
    })
}

fn w3_signal_entity_key(receiver_identity: &str, event_key: &str) -> String {
    format!("{receiver_identity}|{}|{event_key}", event_key.len())
}

fn w3_entity_id(signal_key: &str) -> RecognizerResult<String> {
    let bytes = canonical_json_bytes(&(W3_SIGNAL_PROFILE, signal_key))
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "custom-signal:sha256:{}",
        w3_hex(&Sha256::digest(bytes))
    ))
}

fn w3_unconfirmed_reason(
    receiver_identity: &str,
    event_key: &str,
    producers: &[W3ProducerMatch],
    source_health_complete: bool,
) -> W3UnconfirmedReason {
    if !source_health_complete {
        return W3UnconfirmedReason::ProducerLookupIncomplete;
    }
    if producers.iter().any(|producer| {
        producer.event_key == event_key && producer.receiver_identity != receiver_identity
    }) {
        return W3UnconfirmedReason::ProducerReceiverMismatch;
    }
    if producers.iter().any(|producer| {
        producer.receiver_identity == receiver_identity && producer.event_key != event_key
    }) {
        return W3UnconfirmedReason::ProducerEventKeyMismatch;
    }
    W3UnconfirmedReason::ProducerAbsentInReport
}

fn w3_relation_id(prefix: &str, call_id: &str) -> String {
    w3_digest_id("relation", &(prefix, call_id))
}

fn w3_digest_id(prefix: &str, parts: &(&str, &str)) -> String {
    let bytes = canonical_json_bytes(parts)
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterIdentityMismatch))
        .unwrap_or_default();
    format!("{prefix}:sha256:{}", w3_hex(&Sha256::digest(bytes)))
}

fn w3_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn w3_graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn w3_validate_support(
    input: &W3Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    call: &wow_emmy::function_calls::SourceCallFact,
) -> RecognizerResult<()> {
    let digest = call
        .content_digest()
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    w3_validate_support_record(
        input,
        handle_id,
        evidence_id,
        call.path(),
        call.call_span(),
        Some(digest),
    )
}

fn w3_validate_support_without_digest(
    input: &W3Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    w3_validate_support_record(input, handle_id, evidence_id, path, span, None)
}

fn w3_validate_support_record(
    input: &W3Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
    digest: Option<ContentDigest<SourceContent>>,
) -> RecognizerResult<()> {
    let handle = input
        .source_handles
        .get(&handle_id)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let evidence = input
        .evidence
        .get(&evidence_id)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    handle
        .validate()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    evidence
        .validate()
        .map_err(|_| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if handle.handle_id() != handle_id
        || handle.path().as_str() != path
        || handle.span() != span
        || digest
            .as_ref()
            .is_some_and(|digest| handle.content_digest() != digest)
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
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn w3_checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(w3_failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}

fn w3_failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(
        code,
        "exact custom-registry signal facts could not produce a coherent producer partition",
    )
}

fn w3_entity_binding_error(error: wow_graph::GraphError) -> RecognizerError {
    if error.code() == wow_graph::GraphErrorCode::PartitionInvalid {
        w3_failure(RecognizerErrorCode::AdapterBindingMissing)
    } else {
        w3_graph_error(error)
    }
}

fn w3_graph_error(error: wow_graph::GraphError) -> RecognizerError {
    w3_failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

fn w3_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-custom-signals".into(),
            version: "3".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W3_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-custom-signals-3".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 32,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 65_536,
                max_proposals_per_rule_partition: 65_536,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![
                RecognizerRule {
                    rule_id: W3_PRODUCER_RULE.into(),
                    version: 1,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "signal".into(),
                            kind: W3_CUSTOM_SIGNAL_FACT.into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "signal.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W3_TRIGGER_EVENT_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "signal.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "custom_signal_producer_emits".into(),
                        relation_kind_id: W3_EMITTER_RELATION_ID.into(),
                        source: "signal.caller".into(),
                        target: "signal.event_key".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: (W3_POSITIVE_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    near_negative_fixture_ids: (W3_NEAR_NEGATIVE_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    partial_fixture_ids: (W3_PARTIAL_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    mutation_fixture_ids: (W3_MUTATION_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                },
                RecognizerRule {
                    rule_id: W3_SUBSCRIPTION_RULE.into(),
                    version: 1,
                    required_capabilities: vec!["emmy.fact.calls".into()],
                    scope: "function".into(),
                    clauses: vec![
                        RecognizerClause::Fact {
                            alias: "signal".into(),
                            kind: W3_CUSTOM_SIGNAL_FACT.into(),
                        },
                        RecognizerClause::FieldEq {
                            field: "signal.callable_key".into(),
                            value: crate::RecognizerPackLiteral::String(
                                W3_REGISTER_CALLBACK_CALLABLE.into(),
                            ),
                        },
                        RecognizerClause::FieldEq {
                            field: "signal.colon_call".into(),
                            value: crate::RecognizerPackLiteral::Boolean(true),
                        },
                    ],
                    captures: Vec::new(),
                    outputs: vec![RecognizerOutput::RelationAssertion {
                        output_id: "custom_signal_subscription_handles".into(),
                        relation_kind_id: W3_SUBSCRIPTION_RELATION_ID.into(),
                        source: "signal.caller".into(),
                        target: "signal.event_key".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    }],
                    positive_fixture_ids: (W3_POSITIVE_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    near_negative_fixture_ids: (W3_NEAR_NEGATIVE_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    partial_fixture_ids: (W3_PARTIAL_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                    mutation_fixture_ids: (W3_MUTATION_FIXTURE_IDS
                        .iter()
                        .map(|value| (*value).into())
                        .collect::<Vec<Box<str>>>()),
                },
            ],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| w3_failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

// ===== END WORKER 3: custom registry producer and subscription =====

// ===== BEGIN WORKER 4: cvar callback =====

//  Exact resolved `CVarCallbackRegistry:RegisterCallback` facts -> declarative core
//  recognizer -> graph proposals. Source text is never reparsed here. The CVar key,
//  the callback target, the owner and every span come from the generation-bound
//  Emmy function-call report. This producer never claims combat safety, taint
//  state or secret-payload accessibility, and it never reads, caches or infers a
//  client build, Interface value, source revision, provider revision or toolchain
//  version. Only universal graph roles are used.

pub const W4_PARTITION: &str = "wow-recognizers.lua-cvar-callbacks";
const W4_PROFILE: &str = "wow-recognizers/lua-cvar-callbacks/4";
const W4_FACT_PARTITION: &str = "wow-recognizers.lua-cvar-callback-facts";
const W4_FACT_PROFILE: &str = "wow-recognizers-lua-cvar-callback-facts-4";
const W4_RULE: &str = "core.signal.cvar_callback";
const W4_RULE_VERSION: u32 = 1;
const W4_REGISTERS_CVAR: &str = "CVarCallbackRegistry.RegisterCallback";
const W4_CVAR_ENTITY: &str = "cvar_key";
const W4_REGISTERS_DEFINITION: &str = "lua_registers_cvar_callback";
const W4_REGISTERS_BLOCKER: &str = "lua_cvar_callbacks.exact_resolved_registercallback_calls_only";
const W4_OTHER_RELATION_BLOCKER: &str = "lua_cvar_callbacks.relation_owned_by_other_producer";
const W4_FACT_KIND: &str = "lua_cvar_callback_call";
const W4_MAX_CALLS: usize = 8192;
const W4_MAX_FUNCTIONS: usize = 8192;
const W4_MAX_DECLARATIONS: usize = 8192;
const W4_MAX_ARGUMENTS: usize = 8;
const W4_MAX_CVAR_KEY_BYTES: usize = 512;

const W4_POSITIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-004"];
const W4_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-001"];
const W4_PARTIAL_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-005"];
const W4_MUTATION_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-009"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W4CvarMatch {
    pub call_id: String,
    pub cvar_key: String,
    pub receiver_workspace_id: String,
    pub receiver_identity: String,
    pub entity_proposal_id: String,
    pub registers_proposal_id: String,
    pub callback_proposal_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W4Recognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matches: Vec<W4CvarMatch>,
}

impl W4Recognition {
    pub fn matches(&self) -> &[W4CvarMatch] {
        &self.matches
    }
}

pub struct W4Proposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: W4Recognition,
}

struct W4FunctionBinding {
    node: GraphNodeId,
    proposal_id: String,
    handle: StableHandleId,
    evidence: EvidenceId,
}
struct W4Site {
    call_id: String,
    cvar_key: String,
    exact_cvar_key: bool,
    colon_call: bool,
    caller: W4FunctionBinding,
    receiver_workspace_id: String,
    receiver_identity: String,
    callback: Option<W4CallbackBinding>,
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Clone)]
struct W4CallbackBinding {
    proposal_id: String,
    handle: StableHandleId,
    evidence: EvidenceId,
}

fn w4_checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Relaxed) {
        return Err(w4_failure(RecognizerErrorCode::Cancelled));
    }
    Ok(())
}

fn w4_failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(code, String::new())
}

fn w4_entity_binding_error(error: wow_graph::GraphError) -> RecognizerError {
    if error.code() == wow_graph::GraphErrorCode::PartitionInvalid {
        w4_failure(RecognizerErrorCode::AdapterBindingMissing)
    } else {
        w4_graph_error(error)
    }
}

fn w4_graph_error(error: wow_graph::GraphError) -> RecognizerError {
    w4_failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::AdapterFactMismatch,
    })
}

fn w4_graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn w4_cvar_entity_key(receiver_identity: &str, cvar_key: &str) -> String {
    format!("{receiver_identity}|{}|{cvar_key}", cvar_key.len())
}

fn w4_cvar_entity_id(entity_key: &str) -> RecognizerResult<String> {
    let bytes = canonical_json_bytes(&(W4_PROFILE, entity_key))
        .map_err(|_| w4_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "cvar-key:sha256:{}",
        w3_hex(&Sha256::digest(bytes))
    ))
}

/// Resolves the exact CVar key literal from the registration argument slot. A
/// dynamic key never fabricates a CVar identity and keeps the site inexact.
fn w4_cvar_key(
    arguments: &[wow_emmy::function_calls::SourceCallArgument],
) -> RecognizerResult<(String, bool)> {
    let Some(argument) = arguments.first() else {
        return Ok((String::new(), false));
    };
    match argument.literal() {
        Some(wow_emmy::function_calls::SourceCallLiteral::String(value)) => {
            if value.len() > W4_MAX_CVAR_KEY_BYTES {
                return Err(w4_failure(RecognizerErrorCode::BudgetExceeded));
            }
            if value.is_empty() {
                return Ok((String::new(), false));
            }
            Ok((value.clone(), true))
        }
        _ => Ok((String::new(), false)),
    }
}

/// Resolves the exact callback declaration for one `RegisterCallback` call. Only a
/// resolved Main declaration becomes a callback; a library target, an unresolved
/// global or an alias keeps the endpoint absent without degrading the CVar key.
fn w4_callback(
    call: &wow_emmy::function_calls::SourceCallFact,
    declarations: &BTreeMap<(String, SourceSpan), W3Binding>,
    declaration_proposals: &BTreeMap<(String, SourceSpan), &str>,
    main_snapshot_id: &str,
) -> Option<W4CallbackBinding> {
    let argument = call.arguments().get(1)?;
    argument.reference_key()?;
    let target = argument.reference_target()?;
    if target.role != "main" || target.workspace_id != main_snapshot_id {
        return None;
    }
    let declaration_key = (target.path.clone(), target.span);
    let binding = declarations.get(&declaration_key)?;
    let proposal_id = declaration_proposals.get(&declaration_key)?;
    Some(W4CallbackBinding {
        proposal_id: (*proposal_id).to_owned(),
        handle: binding.handle,
        evidence: binding.evidence,
    })
}

fn w4_registry_target(
    report: &FunctionCallReport,
    call: &wow_emmy::function_calls::SourceCallFact,
) -> Option<(String, String)> {
    let receiver = report.exact_call_receiver(call)?;
    let target = receiver.target();
    if receiver.key() != "CVarCallbackRegistry"
        || target.role != "library"
        || !report.library_snapshot_ids().contains(&target.workspace_id)
        || target.path.is_empty()
        || target.content_digest.is_empty()
        || target
            .span
            .byte_start()
            .zip(target.span.byte_end())
            .is_none_or(|(start, end)| start >= end)
    {
        return None;
    }
    Some((
        target.workspace_id.clone(),
        source_receiver_identity(target).ok()?,
    ))
}

fn w4_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-cvar-callbacks".into(),
            version: "4".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W4_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-cvar-callback-4".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 4,
                max_clauses_per_rule: 32,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 10_000,
                max_proposals_per_rule_partition: 20_000,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![RecognizerRule {
                rule_id: W4_RULE.into(),
                version: W4_RULE_VERSION,
                required_capabilities: vec!["emmy.fact.calls".into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "call".into(),
                        kind: W4_FACT_KIND.into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.colon_call".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.has_cvar_key".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.exact_cvar_key".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                ],
                captures: vec![crate::RecognizerCapture {
                    name: "cvar_key".into(),
                    value_type: "bounded_string".into(),
                    source: "call.cvar_key".into(),
                    cardinality: crate::RecognizerCaptureCardinality::Optional,
                }],
                outputs: vec![
                    RecognizerOutput::EntityAssertion {
                        output_id: "cvar_key_entity".into(),
                        entity_kind_id: W4_CVAR_ENTITY.into(),
                        semantic_key: BTreeMap::from([("call".into(), "call.call_id".into())]),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "cvar_callback_registers".into(),
                        relation_kind_id: W4_REGISTERS_DEFINITION.into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                ],
                positive_fixture_ids: (W4_POSITIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                near_negative_fixture_ids: (W4_NEAR_NEGATIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                partial_fixture_ids: (W4_PARTIAL_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                mutation_fixture_ids: (W4_MUTATION_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| w4_failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

/// Builds one CVar-callback fact from one resolved site. The fact declares the call,
/// the caller, the resolved callable key and the exact CVar key.
fn w4_signal_fact(
    input: &W3Input<'_>,
    site: &W4Site,
    limits: RecognizerFactLimits,
) -> RecognizerResult<RecognizerFact> {
    let mut fields = BTreeMap::from([
        (
            "call_id".into(),
            RecognizerFactValue::Reference(site.call_id.clone().into()),
        ),
        (
            "caller".into(),
            RecognizerFactValue::Reference(site.caller.proposal_id.clone().into()),
        ),
        (
            "callable_key".into(),
            RecognizerFactValue::String(W4_REGISTERS_CVAR.into()),
        ),
        ("colon_call".into(), RecognizerFactValue::Boolean(true)),
        (
            "has_cvar_key".into(),
            RecognizerFactValue::Boolean(!site.cvar_key.is_empty()),
        ),
        (
            "exact_cvar_key".into(),
            RecognizerFactValue::Boolean(site.exact_cvar_key),
        ),
        (
            "cvar_key".into(),
            RecognizerFactValue::String(site.cvar_key.clone().into()),
        ),
        (
            "receiver_workspace_id".into(),
            RecognizerFactValue::String(site.receiver_workspace_id.clone().into()),
        ),
        (
            "receiver_identity".into(),
            RecognizerFactValue::Reference(site.receiver_identity.clone().into()),
        ),
    ]);
    let mut source_handle_ids = BTreeSet::from([site.handle]);
    let mut evidence_ids = BTreeSet::from([site.evidence]);
    if let Some(callback) = site.callback.as_ref() {
        fields.insert(
            "callback".into(),
            RecognizerFactValue::Reference(callback.proposal_id.clone().into()),
        );
        source_handle_ids.insert(callback.handle);
        evidence_ids.insert(callback.evidence);
    }
    let caller_function_id = input
        .report
        .calls()
        .iter()
        .find(|call| call.fact_id() == site.call_id.as_str())
        .map(|call| call.caller_function_id())
        .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    RecognizerFact::new(
        input.context.context_id(),
        RecognizerFactInput {
            kind: W4_FACT_KIND.into(),
            partition_id: W4_FACT_PARTITION.into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                caller_function_id.to_owned(),
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: W4_FACT_PROFILE.into(),
            confidence: if site.exact_cvar_key {
                GraphConfidence::Derived
            } else {
                GraphConfidence::Possible
            },
            fields,
            source_handle_ids: source_handle_ids.into_iter().collect(),
            evidence_ids: evidence_ids.into_iter().collect(),
        },
        limits,
    )
}

pub fn recognize_source_cvar_callbacks(
    input: W3Input<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<W4Proposals> {
    w4_checkpoint(stop)?;
    if input.report.calls().len() > W4_MAX_CALLS
        || input.report.functions().len() > W4_MAX_FUNCTIONS
        || input.function_proposals.len() != input.report.functions().len()
        || input.call_support.len() != input.report.calls().len()
        || input.declaration_proposals.len() > W4_MAX_DECLARATIONS
    {
        return Err(w4_failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .report
        .validate()
        .map_err(|_| w4_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    input
        .context
        .validate()
        .map_err(|_| w4_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(w4_failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let lookup = input.owner.producer_lookup(stop).map_err(w4_graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    // Every enclosing function must cross into an accepted source_function
    // proposal. This producer owns no function identity of its own.
    let mut functions = BTreeMap::<String, W4FunctionBinding>::new();
    for function in input.report.functions() {
        w4_checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let resolved = lookup
            .entity(
                lookup.scope(),
                &GraphAssertionRef::Producer {
                    partition_id: source_partition.partition_id().into(),
                    batch_id: source_partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: proposal_id.into(),
                    },
                },
                stop,
            )
            .map_err(w4_entity_binding_error)?;
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
            return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w3_validate_support_without_digest(
            &input,
            *handle,
            *evidence,
            function.path(),
            function.span(),
        )?;
        let node = resolved.accepted().node().node_id().clone();
        if functions
            .insert(
                function.fact_id().to_owned(),
                W4FunctionBinding {
                    node,
                    proposal_id: proposal_id.to_owned(),
                    handle: *handle,
                    evidence: *evidence,
                },
            )
            .is_some()
        {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    // Main-declaration crosswalk, used only for the exact callback endpoint.
    let mut declarations = BTreeMap::<(String, SourceSpan), W3Binding>::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        w4_checkpoint(stop)?;
        let resolved = lookup
            .entity(
                lookup.scope(),
                &GraphAssertionRef::Producer {
                    partition_id: source_partition.partition_id().into(),
                    batch_id: source_partition.batch().batch_id().into(),
                    assertion: GraphLocalAssertion {
                        kind: GraphAssertionKind::Entity,
                        proposal_id: (*proposal_id).into(),
                    },
                },
                stop,
            )
            .map_err(w4_entity_binding_error)?;
        let proposal = resolved.proposal();
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let expected = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String((*path).clone().into()),
            ),
            (
                "span_start".into(),
                GraphProposalValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| w4_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| w4_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
        ]);
        if proposal.entity_kind_id() != "lua_source_declaration"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        let node = resolved.accepted().node().node_id().clone();
        if declarations
            .insert(
                ((*path).clone(), *span),
                W3Binding {
                    node,
                    handle: *handle,
                    evidence: *evidence,
                },
            )
            .is_some()
        {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut sites = BTreeMap::<String, W4Site>::new();
    for call in input.report.calls() {
        w4_checkpoint(stop)?;
        if call.resolved_callable_key() != Some(W4_REGISTERS_CVAR) || !call.is_colon_call() {
            continue;
        }
        let Some(caller) = functions.get(call.caller_function_id()) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingMissing));
        };
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w3_validate_support(&input, handle, evidence, call)?;

        let arguments = call.arguments();
        if arguments.len() > W4_MAX_ARGUMENTS {
            return Err(w4_failure(RecognizerErrorCode::BudgetExceeded));
        }
        let (cvar_key, exact_cvar_key) = w4_cvar_key(arguments)?;
        if !exact_cvar_key {
            continue;
        }
        let Some((receiver_workspace_id, receiver_identity)) =
            w4_registry_target(input.report, call)
        else {
            continue;
        };
        let callback = w4_callback(
            call,
            &declarations,
            &input.declaration_proposals,
            input.report.main_snapshot_id(),
        );

        let site = W4Site {
            call_id: call.fact_id().to_owned(),
            cvar_key,

            exact_cvar_key,
            colon_call: call.is_colon_call(),
            caller: W4FunctionBinding {
                node: caller.node.clone(),
                proposal_id: caller.proposal_id.clone(),
                handle: caller.handle,
                evidence: caller.evidence,
            },
            receiver_workspace_id,
            receiver_identity,
            callback,
            handle,
            evidence,
        };
        facts.push(w4_signal_fact(&input, &site, fact_limits)?);
        if sites.insert(site.call_id.clone(), site).is_some() {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W4_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: W4_FACT_PROFILE.into(),
            state: if input.report.source_health_complete() {
                RecognizerFactCoverageState::Complete
            } else {
                RecognizerFactCoverageState::NotEvaluated
            },
            blocker_ids: if input.report.source_health_complete() {
                Vec::new()
            } else {
                vec!["emmy.call_source_parse_failed".into()]
            },
        },
        fact_limits,
    )?];
    let bundle = RecognizerFactBundle::build(
        input.context,
        W4_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = w4_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    w4_checkpoint(stop)?;
    let graph_nodes = graph.nodes().to_vec();
    let _limits = input.owner.snapshot().limits();
    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut matches = Vec::new();
    let mut entity_by_call = BTreeMap::<String, String>::new();
    let mut entity_accumulators = BTreeMap::<String, SignalEntityAccumulator>::new();
    let mut registers_by_call = BTreeMap::<String, String>::new();

    // Matcher proposals are sorted by proposal ID, so materialize entities before
    // relations instead of relying on output declaration order.
    for outcome in output.outcomes() {
        if outcome.rule_id() != W4_RULE || outcome.rule_version() != W4_RULE_VERSION {
            return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Entity {
                proposal_id: _,
                entity_kind_id,
                semantic_key,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } = proposal
            else {
                continue;
            };
            if entity_kind_id.as_ref() != W4_CVAR_ENTITY || semantic_key.len() != 1 {
                return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let Some(RecognizerFactValue::Reference(call_id)) = semantic_key.get("call") else {
                return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let site = sites
                .get(call_id.as_ref())
                .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if !graph_nodes
                .iter()
                .any(|node| node.node_id() == &site.caller.node)
            {
                return Err(w4_failure(RecognizerErrorCode::AdapterBindingMissing));
            }
            let graph_confidence = w4_graph_confidence(*confidence);
            let entity_key = w4_cvar_entity_key(&site.receiver_identity, &site.cvar_key);
            let entity_id = w4_cvar_entity_id(&entity_key)?;
            let accumulator = entity_accumulators.entry(entity_key).or_insert_with(|| {
                SignalEntityAccumulator::new(entity_id.clone(), graph_confidence)
            });
            if accumulator.proposal_id != entity_id {
                return Err(w4_failure(RecognizerErrorCode::AdapterIdentityMismatch));
            }
            accumulator.merge(
                graph_confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
            );
            if entity_by_call
                .insert(call_id.to_string(), entity_id)
                .is_some()
            {
                return Err(w4_failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
        }
    }

    for (entity_key, accumulator) in entity_accumulators {
        w4_checkpoint(stop)?;
        entities.push(
            GraphEntityProposal::new(
                accumulator.proposal_id,
                W4_CVAR_ENTITY,
                BTreeMap::from([("cvar".into(), GraphProposalValue::String(entity_key.into()))]),
                accumulator.confidence,
                accumulator.source_handle_ids.into_iter().collect(),
                accumulator.evidence_ids.into_iter().collect(),
                accumulator.coverage_ids.into_iter().collect(),
            )
            .map_err(w4_graph_error)?,
        );
    }

    for outcome in output.outcomes() {
        for proposal in outcome.proposals() {
            let crate::RecognizerProposedAssertion::Relation {
                proposal_id,
                relation_kind_id,
                source,
                target,
                confidence,
                source_handle_ids,
                evidence_ids,
                coverage_ids,
                ..
            } = proposal
            else {
                continue;
            };
            if relation_kind_id.as_ref() != W4_REGISTERS_DEFINITION {
                return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let RecognizerFactValue::Reference(caller_proposal) = source else {
                return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let RecognizerFactValue::Reference(call_id) = target else {
                return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let site = sites
                .get(call_id.as_ref())
                .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if site.caller.proposal_id != caller_proposal.as_ref() {
                return Err(w4_failure(RecognizerErrorCode::AdapterBindingMissing));
            }
            let cvar_node = entity_by_call
                .get(call_id.as_ref())
                .ok_or_else(|| w4_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone();
            if registers_by_call
                .insert(call_id.to_string(), proposal_id.to_string())
                .is_some()
            {
                return Err(w4_failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
            relations.push(
                GraphRelationProposal::new(
                    proposal_id.to_string(),
                    W4_REGISTERS_DEFINITION,
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Existing(site.caller.node.clone()),
                        target: GraphProposalEndpoint::Proposed(cvar_node.into()),
                        confidence: w4_graph_confidence(*confidence),
                        source_handle_ids: source_handle_ids.clone(),
                        evidence_ids: evidence_ids.clone(),
                        coverage_ids: coverage_ids.clone(),
                    },
                )
                .map_err(w4_graph_error)?,
            );
        }
    }

    // Every retained site needs exactly one CVar entity and one registers
    // relation. A missing pair is an adapter mismatch, never a silent drop.
    if sites.len() != entity_by_call.len() || sites.len() != registers_by_call.len() {
        return Err(w4_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    for (call_id, site) in &sites {
        w4_checkpoint(stop)?;
        let Some(entity_proposal_id) = entity_by_call.get(call_id) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingMissing));
        };
        let Some(registers_proposal_id) = registers_by_call.get(call_id) else {
            return Err(w4_failure(RecognizerErrorCode::AdapterBindingMissing));
        };
        matches.push(W4CvarMatch {
            call_id: call_id.clone(),
            cvar_key: site.cvar_key.clone(),
            receiver_workspace_id: site.receiver_workspace_id.clone(),
            receiver_identity: site.receiver_identity.clone(),
            entity_proposal_id: entity_proposal_id.to_string(),
            registers_proposal_id: registers_proposal_id.clone(),
            callback_proposal_id: site
                .callback
                .as_ref()
                .map(|callback| callback.proposal_id.clone()),
        });
    }

    let recognition = W4Recognition {
        profile: W4_PROFILE,
        analyzer_report_id: input.report.analysis_id().into(),
        fact_bundle_id: bundle.bundle_id().to_string(),
        pack_digest: pack.pack_digest().into(),
        plan_id: plan.plan_id().to_string(),
        output_partition_id: output.partition_id().to_string(),
        matches,
    };
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
            let (state, blocker) = if relation == GraphRelationKind::RegistersCvarCallback {
                (GraphCoverageState::Partial, W4_REGISTERS_BLOCKER)
            } else {
                (GraphCoverageState::NotEvaluated, W4_OTHER_RELATION_BLOCKER)
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(w4_graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        W4_PARTITION,
        entities,
        relations,
    )
    .map_err(w4_graph_error)?;
    Ok(W4Proposals {
        batch,
        coverage: graph_coverage,
        recognition,
    })
}
// ===== END WORKER 4: cvar callback =====

#[cfg(test)]
mod w4_tests {
    use super::*;
    use wow_graph::{
        GraphDirection, GraphErrorCode, GraphGenerationId, GraphLimits, GraphNeighborQuery,
        GraphNeighborReadLimits, GraphNeighborReadQuery, GraphNode, GraphSnapshot, GraphUniverseId,
    };

    #[test]
    fn graph_error_lowering_preserves_cancellation_and_budget()
    -> Result<(), Box<dyn std::error::Error>> {
        let limits = GraphLimits::default();
        let universe = GraphUniverseId::new("project:cvar-error-lowering")?;
        let generation = GraphGenerationId::new("input-generation:cvar-error-lowering")?;
        let node = GraphNode::new(
            universe.clone(),
            generation.clone(),
            "lua_source_function",
            "cvar-error-lowering",
            Vec::new(),
            limits,
        )?;
        let node_id = node.node_id().clone();
        let snapshot = GraphSnapshot::build(
            universe,
            generation,
            limits,
            vec![node],
            Vec::new(),
            Vec::new(),
        )?;
        let query = GraphNeighborReadQuery::new(
            snapshot.snapshot_id().clone(),
            GraphNeighborQuery::new(
                node_id,
                GraphDirection::Outgoing,
                vec![GraphRelationKind::Calls],
                limits.max_query_edges + 1,
            )?,
            GraphNeighborReadLimits::default(),
        )?;
        let stop = AtomicBool::new(true);
        let cancelled = query
            .execute(&snapshot, &stop)
            .err()
            .ok_or("stopped native graph read must reject")?;
        assert_eq!(cancelled.code(), GraphErrorCode::Cancelled);
        assert_eq!(
            w4_graph_error(cancelled).code(),
            RecognizerErrorCode::Cancelled
        );

        stop.store(false, Ordering::Release);
        let budget = query
            .execute(&snapshot, &stop)
            .err()
            .ok_or("native query above the snapshot budget must reject")?;
        assert_eq!(budget.code(), GraphErrorCode::BudgetExceeded);
        assert_eq!(
            w4_graph_error(budget).code(),
            RecognizerErrorCode::BudgetExceeded
        );

        let invalid = GraphCoverageRecord::new(
            GraphRelationKind::Calls,
            GraphCoverageState::Partial,
            true,
            Vec::new(),
            limits,
        )
        .err()
        .ok_or("partial native coverage cannot authorize absence")?;
        assert_eq!(invalid.code(), GraphErrorCode::SnapshotInvalid);
        assert_eq!(
            w4_graph_error(invalid).code(),
            RecognizerErrorCode::AdapterFactMismatch
        );
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod source_receiver_test_support {
    use super::FunctionCallReport;
    use std::error::Error;
    use wow_emmy::{
        EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
        LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
    };

    pub(crate) fn function_call_report(
        main_shadows_registry: bool,
    ) -> Result<(FunctionCallReport, String), Box<dyn Error>> {
        let backend = EmmyBackendIdentity::new(
            "emmylua_code_analysis",
            Some(EMMYLUA_CODE_ANALYSIS_VERSION),
            EMMYLUA_REVISION,
            EMMYLUA_TREE,
            format!("sha256:{}", "1".repeat(64)),
            format!("sha256:{}", "2".repeat(64)),
        )?;
        let main_shadow = if main_shadows_registry {
            "EventRegistry = {}\n\
             function EventRegistry:TriggerEvent(event) end\n\
             function EventRegistry:RegisterCallback(event, callback, owner) end\n\
             function EventRegistry:RegisterFrameEvent(event) end\n\
             function EventRegistry:RegisterFrameEventAndCallback(event, callback, owner) end\n\
             function EventRegistry:RegisterFrameEventAndCallbackWithHandle(event, callback, owner) end\n\
             CVarCallbackRegistry = {}\n\
             function CVarCallbackRegistry:RegisterCallback(cvar, callback, owner) end\n"
        } else {
            ""
        };
        let main_source = format!(
            "{main_shadow}local function onEvent() end\n\
             function Main()\n\
             EventRegistry:TriggerEvent(\"Signal.Ready\")\n\
             EventRegistry:RegisterCallback(\"Signal.Ready\", onEvent)\n\
             EventRegistry:RegisterFrameEvent(\"PLAYER_LOGOUT\")\n\
             EventRegistry:RegisterFrameEventAndCallback(\"PLAYER_LOGIN\", onEvent)\n\
             EventRegistry:RegisterFrameEventAndCallbackWithHandle(\"PLAYER_LOGOUT\", onEvent)\n\
             CVarCallbackRegistry:RegisterCallback(\"test_cvar\", onEvent)\n\
             end\n"
        );
        let library_source = "EventRegistry = {}\n\
            function EventRegistry:TriggerEvent(event) end\n\
            function EventRegistry:RegisterCallback(event, callback, owner) end\n\
            function EventRegistry:RegisterFrameEvent(event) end\n\
            function EventRegistry:RegisterFrameEventAndCallback(event, callback, owner) end\n\
            function EventRegistry:RegisterFrameEventAndCallbackWithHandle(event, callback, owner) end\n\
            CVarCallbackRegistry = {}\n\
            function CVarCallbackRegistry:RegisterCallback(cvar, callback, owner) end\n";
        let limits = LuaWorkspaceLimits::new(16, 16_384, 1024 * 1024, 8 * 1024 * 1024)?;
        let main = LuaWorkspaceSnapshot::build(
            backend.clone(),
            LuaWorkspaceUniverse::Project,
            vec![LuaWorkspaceFileInput::new("main/main.lua", main_source)],
            limits,
        )?;
        let library = LuaWorkspaceSnapshot::build(
            backend,
            LuaWorkspaceUniverse::Fixture,
            vec![LuaWorkspaceFileInput::new(
                "Blizzard_SharedXMLBase/GlobalCallbackRegistry.lua",
                library_source,
            )],
            limits,
        )?;
        let session = wow_emmy::references::analyze_member_call_session(
            &main,
            &[&library],
            &[],
            true,
            &std::sync::atomic::AtomicBool::new(false),
        )?;
        let report = session.function_calls.ok_or("function-call report")?;
        report.validate()?;
        Ok((report, library.snapshot_id().to_owned()))
    }
}

#[cfg(test)]
mod source_receiver_tests {
    use super::source_receiver_test_support::function_call_report;
    use super::*;

    #[test]
    fn library_receiver_gates_preserve_exact_workspace_and_classify_unconfirmed_proofs()
    -> Result<(), Box<dyn std::error::Error>> {
        let (report, library_workspace_id) = function_call_report(false)?;
        let mut saw_custom_producer = false;
        let mut saw_custom_subscription = false;
        let mut saw_cvar = false;
        for call in report.calls() {
            match call.resolved_callable_key() {
                Some("EventRegistry.TriggerEvent" | "EventRegistry.RegisterCallback") => {
                    let receiver = report.exact_call_receiver(call).ok_or("exact receiver")?;
                    let target = receiver.target();
                    assert_eq!(receiver.key(), "EventRegistry");
                    assert_eq!(target.role, "library");
                    assert_eq!(target.workspace_id, library_workspace_id);
                    assert!(w3_is_resolved_custom_registry(
                        Some(receiver.key()),
                        target,
                        &report,
                    ));
                    saw_custom_producer |=
                        call.resolved_callable_key() == Some("EventRegistry.TriggerEvent");
                    saw_custom_subscription |=
                        call.resolved_callable_key() == Some("EventRegistry.RegisterCallback");
                }
                Some("CVarCallbackRegistry.RegisterCallback") => {
                    let (workspace_id, identity) =
                        w4_registry_target(&report, call).ok_or("exact Library CVar receiver")?;
                    assert_eq!(workspace_id, library_workspace_id);
                    assert!(identity.starts_with("receiver:sha256:"));
                    saw_cvar = true;
                }
                _ => {}
            }
        }
        assert!(saw_custom_producer && saw_custom_subscription && saw_cvar);

        let (shadow_report, _) = function_call_report(true)?;
        let shadow = shadow_report
            .calls()
            .iter()
            .find(|call| call.resolved_callable_key() == Some("EventRegistry.TriggerEvent"))
            .ok_or("Main-shadow call")?;
        let shadow_receiver = shadow_report
            .exact_call_receiver(shadow)
            .ok_or("exact Main-shadow receiver")?;
        assert_eq!(shadow_receiver.target().role, "main");
        assert_eq!(
            shadow_receiver.target().workspace_id,
            shadow_report.main_snapshot_id()
        );
        assert!(w3_is_resolved_custom_registry(
            Some(shadow_receiver.key()),
            shadow_receiver.target(),
            &shadow_report,
        ));
        assert!(
            w4_registry_target(
                &shadow_report,
                shadow_report
                    .calls()
                    .iter()
                    .find(|call| {
                        call.resolved_callable_key()
                            == Some("CVarCallbackRegistry.RegisterCallback")
                    })
                    .ok_or("Main-shadow CVar call")?,
            )
            .is_none()
        );

        let producer = |call_id: &str, receiver_identity: &str, event_key: &str| W3ProducerMatch {
            call_id: call_id.into(),
            event_key: event_key.into(),
            receiver_workspace_id: "library-snapshot".into(),
            receiver_identity: receiver_identity.into(),
            receiver_declaration_proposal_id: None,
            relation_proposal_id: format!("relation:{call_id}"),
        };
        assert_eq!(
            w3_unconfirmed_reason("receiver-a", "Signal.Ready", &[], true),
            W3UnconfirmedReason::ProducerAbsentInReport
        );
        assert_eq!(
            w3_unconfirmed_reason(
                "receiver-a",
                "Signal.Ready",
                &[producer("p1", "receiver-a", "Signal.Other")],
                true,
            ),
            W3UnconfirmedReason::ProducerEventKeyMismatch
        );
        assert_eq!(
            w3_unconfirmed_reason(
                "receiver-a",
                "Signal.Ready",
                &[producer("p2", "receiver-b", "Signal.Ready")],
                true,
            ),
            W3UnconfirmedReason::ProducerReceiverMismatch
        );
        assert_eq!(
            w3_unconfirmed_reason("receiver-a", "Signal.Ready", &[], false),
            W3UnconfirmedReason::ProducerLookupIncomplete
        );
        Ok(())
    }
}
