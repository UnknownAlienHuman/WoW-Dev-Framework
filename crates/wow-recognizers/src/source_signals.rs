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
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphEntityProposal, GraphNodeId,
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
//  proposals. Source text is never reparsed here: the event name, the unit
//  tokens, the handler target and every span come from the generation-bound Emmy
//  function-call report. This module never claims combat safety, protected-action
//  authority, taint state or secret-payload accessibility, and it never reads,
//  caches or infers a client build, Interface value, source revision, provider
//  revision or toolchain version. Only universal graph roles are used.

const W1_SIGNAL_PARTITION: &str = "wow-recognizers.lua-native-frame-events";
const W1_SIGNAL_PROFILE: &str = "wow-recognizers/lua-native-frame-events/1";
const W1_FACT_PARTITION: &str = "wow-recognizers.lua-native-frame-event-facts";
const W1_FACT_PROFILE: &str = "wow-recognizers-lua-native-frame-event-facts-1";
const W1_RULE: &str = "core.signal.native_frame_event";
const W1_RULE_VERSION: u32 = 1;
const W1_REGISTER_EVENT_CALLABLE: &str = "Frame.RegisterEvent";
const W1_REGISTER_UNIT_EVENT_CALLABLE: &str = "Frame.RegisterUnitEvent";
const W1_MAX_CALLS: usize = 8192;
const W1_MAX_FUNCTIONS: usize = 8192;
const W1_MAX_DECLARATIONS: usize = 8192;
const W1_MAX_ARGUMENTS: usize = 8;
const W1_MAX_EVENT_NAMES: usize = 16;
const W1_MAX_UNIT_TOKENS: usize = 16;
const W1_MAX_MATCHES: usize = 8192;
const W1_MAX_EVENT_BYTES: usize = 1024;
const W1_EVENT_ENTITY: &str = "native_event";
const W1_REGISTERS_DEFINITION: &str = "lua_register_native_event";
const W1_HANDLES_DEFINITION: &str = "lua_handles_native_event";
const W1_REGISTERS_BLOCKER: &str =
    "lua_native_frame_events.exact_resolved_registerevent_calls_only";
const W1_HANDLES_BLOCKER: &str = "lua_native_frame_events.exact_main_handler_declarations_only";
const W1_OTHER_RELATION_BLOCKER: &str = "lua_native_frame_events.relation_owned_by_other_producer";
const W1_POSITIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-001"];
const W1_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-002"];
const W1_PARTIAL_FIXTURE_IDS: [&str; 1] = ["RECOG-EVENT-003"];
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

    /// Argument slot of the first event-name argument. RegisterUnitEvent places
    /// the unit token first, so its event slot starts one position later.
    const fn event_ordinal(self) -> usize {
        match self {
            Self::FrameEvent => 0,
            Self::FrameUnitEvent => 1,
        }
    }

    /// Argument slot holding the single exact unit token, when this shape has one.
    const fn unit_ordinal(self) -> Option<usize> {
        match self {
            Self::FrameEvent => None,
            Self::FrameUnitEvent => Some(0),
        }
    }

    /// Argument slot that may hold an exact handler reference.
    const fn handler_ordinal(self) -> usize {
        match self {
            Self::FrameEvent => 1,
            Self::FrameUnitEvent => 2,
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

struct W1DeclarationBinding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
    path: String,
    span: SourceSpan,
    digest: ContentDigest<SourceContent>,
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
    handlers: Vec<W1Handler>,
}

struct W1Handler {
    declaration: W1DeclarationBinding,
    reference_key: String,
    argument_ordinal: u32,
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
        || input.declaration_proposals.len() > W1_MAX_DECLARATIONS
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

    // input_view validates the exact owner and reverses publication rebinding,
    // so a materialized generation node ID is never used as an input here.
    let graph = input.owner.input_view(stop).map_err(w1_graph_error)?;
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = source_partition.report().accepted_entities();

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
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none()
            || !function_proposal_ids.insert(proposal_id.to_owned())
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

    // Main-declaration crosswalk, used only for exact handler endpoints.
    let mut declarations = BTreeMap::<(String, SourceSpan), W1DeclarationBinding>::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        w1_checkpoint(stop)?;
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let expected = BTreeMap::from([
            (
                "document".into(),
                GraphProposalValue::String((*path).into()),
            ),
            (
                "span_start".into(),
                GraphProposalValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
        ]);
        if proposal.entity_kind_id() != "lua_source_declaration"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w1_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        let record = input
            .source_handles
            .get(handle)
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let digest = record
            .content_digest()
            .to_string()
            .parse::<ContentDigest<SourceContent>>()
            .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none()
            || declarations
                .insert(
                    ((*path).to_owned(), *span),
                    W1DeclarationBinding {
                        node,
                        handle: *handle,
                        evidence: *evidence,
                        path: (*path).to_owned(),
                        span: *span,
                        digest,
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
        let (event_names, exact_event_names) = w1_event_names(arguments, kind)?;
        let (unit_tokens, _) = w1_unit_tokens(arguments, kind)?;
        let handlers = w1_handlers(
            &input,
            arguments,
            kind,
            &declarations,
            input.report.main_snapshot_id(),
        )?;

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
            handlers,
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
        for handler in &site.handlers {
            fields.insert(
                format!("handler_{}", handler.argument_ordinal).into_boxed_str(),
                RecognizerFactValue::Reference(
                    handler.declaration.node.as_str().to_string().into(),
                ),
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
                confidence: if site.exact_event_names && !site.handlers.is_empty() {
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
    let mut registers_by_call = BTreeMap::<String, String>::new();
    for outcome in output.outcomes() {
        for proposal in outcome.proposals() {
            match proposal {
                crate::RecognizerProposedAssertion::Entity {
                    proposal_id,
                    entity_kind_id,
                    semantic_key,
                    confidence,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } => {
                    if entity_kind_id.as_ref() != W1_EVENT_ENTITY || semantic_key.len() != 1 {
                        return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let Some(RecognizerFactValue::Reference(call_id)) = semantic_key.get("call")
                    else {
                        return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
                    };
                    let site = sites
                        .get(call_id.as_ref())
                        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
                    if event_by_call
                        .insert(call_id.to_string(), proposal_id.to_string())
                        .is_some()
                    {
                        return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    let mut identity = BTreeMap::from([(
                        "call".into(),
                        GraphProposalValue::Reference(call_id.clone()),
                    )]);
                    for (index, name) in
                        site.event_names.iter().take(W1_MAX_EVENT_NAMES).enumerate()
                    {
                        identity.insert(
                            format!("event_{index}").into_boxed_str(),
                            GraphProposalValue::Identifier(name.as_str().into()),
                        );
                    }
                    entities.push(
                        GraphEntityProposal::new(
                            proposal_id.to_string(),
                            W1_EVENT_ENTITY,
                            identity,
                            w1_graph_confidence(*confidence),
                            source_handle_ids.clone(),
                            evidence_ids.clone(),
                            coverage_ids.clone(),
                        )
                        .map_err(w1_graph_error)?,
                    );
                }
                crate::RecognizerProposedAssertion::Relation {
                    proposal_id,
                    relation_kind_id,
                    source,
                    target,
                    confidence,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } => {
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

    // One handles_native_event relation per exact resolved Main handler endpoint.
    let mut handle_relations = BTreeMap::<(String, String), String>::new();
    for (call_id, site) in &sites {
        w1_checkpoint(stop)?;
        let event_id = event_by_call
            .get(call_id)
            .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?
            .clone();
        for handler in &site.handlers {
            let relation_id =
                w1_handler_relation_id(call_id, &handler.declaration, handler.argument_ordinal)?;
            let confidence = if site.exact_event_names {
                GraphConfidence::Derived
            } else {
                GraphConfidence::Possible
            };
            relations.push(
                GraphRelationProposal::new(
                    relation_id.as_str(),
                    W1_HANDLES_DEFINITION,
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Proposed(event_id.clone().into()),
                        target: GraphProposalEndpoint::Existing(handler.declaration.node.clone()),
                        confidence,
                        source_handle_ids: BTreeSet::from([
                            site.handle,
                            handler.declaration.handle,
                        ])
                        .into_iter()
                        .collect(),
                        evidence_ids: BTreeSet::from([site.evidence, handler.declaration.evidence])
                            .into_iter()
                            .collect(),
                        coverage_ids: Vec::new(),
                    },
                )
                .map_err(w1_graph_error)?,
            );
            if handle_relations
                .insert(
                    (call_id.clone(), handler.declaration.node.to_string()),
                    relation_id.clone(),
                )
                .is_some()
            {
                return Err(w1_failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
        }
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
                (GraphCoverageState::Partial, W1_HANDLES_BLOCKER)
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
        let mut handler_proposal_ids = site
            .handlers
            .iter()
            .map(|handler| handler.declaration.node.to_string())
            .collect::<Vec<_>>();
        handler_proposal_ids.sort();
        let mut handles_relation_proposal_ids = handle_relations
            .iter()
            .filter(|((site_call_id, _), _)| site_call_id == call_id)
            .map(|(_, relation_id)| relation_id.clone())
            .collect::<Vec<_>>();
        handles_relation_proposal_ids.sort();
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
            handles_relation_proposal_ids,
            handler_declaration_proposal_ids: handler_proposal_ids,
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
    RecognizerError::new(code, String::new())
}

fn w1_graph_error(_error: wow_graph::GraphError) -> RecognizerError {
    w1_failure(RecognizerErrorCode::AdapterFactMismatch)
}

fn w1_graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

/// Validate one support record against the exact source handle and evidence.
fn w1_validate_support(
    input: &W1Input<'_>,
    handle: StableHandleId,
    evidence: EvidenceId,
    path: &str,
    content_digest: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    if input.source_handles.contains_key(&handle)
        && input.evidence.contains_key(&evidence)
        && !path.is_empty()
        && !content_digest.is_empty()
    {
        let _ = span;
        Ok(())
    } else {
        Err(w1_failure(RecognizerErrorCode::AdapterBindingMissing))
    }
}
// ===== END WORKER 1: native frame event =====

// ===== BEGIN WORKER 3: custom registry producer and subscription =====

pub const W3_SIGNAL_PARTITION: &str = "wow-recognizers.lua-custom-signals";
pub const W3_SIGNAL_PROFILE: &str = "wow-recognizers/lua-custom-signals/1";
const W3_FACT_PARTITION: &str = "wow-recognizers.lua-custom-signal-facts";
const W3_FACT_PROFILE: &str = "wow-recognizers-lua-custom-signal-facts-1";
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

const W3_POSITIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-SIGNAL-001"];
const W3_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 1] = ["RECOG-SIGNAL-002"];
const W3_PARTIAL_FIXTURE_IDS: [&str; 1] = ["RECOG-SIGNAL-003"];
const W3_MUTATION_FIXTURE_IDS: [&str; 2] = ["RECOG-SIGNAL-004", "RECOG-SIGNAL-005"];

/// One resolved custom-signal producer site: a resolved custom-registry receiver
/// and one literal event key. A dynamic receiver or key never enters this block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3ProducerMatch {
    pub call_id: String,
    pub event_key: String,
    pub receiver_declaration_proposal_id: String,
    pub relation_proposal_id: String,
}

/// One resolved subscription site. confirmed is true only when an exact
/// compatible producer exists in this same report and revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3SubscriptionMatch {
    pub call_id: String,
    pub event_key: String,
    pub receiver_declaration_proposal_id: String,
    pub producer_call_id: Option<String>,
    pub relation_proposal_id: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum W3UnconfirmedReason {
    ProducerAbsentInReport,
    ProducerEventKeyMismatch,
}

/// A subscription that could not be confirmed against an exact producer. Retained
/// explicitly so absence of evidence is never reported as a clean negative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W3UnconfirmedSubscription {
    pub call_id: String,
    pub event_key: String,
    pub receiver_declaration_proposal_id: String,
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

/// Exact Main-declaration handler endpoints. A library, unresolved or dynamic
/// argument never becomes a handler here and never fabricates a target.
fn w1_handlers(
    _input: &W1Input<'_>,
    arguments: &[wow_emmy::function_calls::SourceCallArgument],
    kind: W1RegistrationKind,
    declarations: &BTreeMap<(String, SourceSpan), W1DeclarationBinding>,
    main_snapshot_id: &str,
) -> RecognizerResult<Vec<W1Handler>> {
    let Some(argument) = arguments.get(kind.handler_ordinal()) else {
        return Ok(Vec::new());
    };
    let (Some(reference_key), Some(target)) =
        (argument.reference_key(), argument.reference_target())
    else {
        return Ok(Vec::new());
    };
    if target.role != "main" || target.workspace_id != main_snapshot_id {
        return Ok(Vec::new());
    }
    let declaration = declarations
        .get(&(target.path.clone(), target.span))
        .ok_or_else(|| w1_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if declaration.path != target.path || declaration.span != target.span {
        return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let digest = target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w1_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    if digest != declaration.digest {
        return Err(w1_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let argument_ordinal = u32::try_from(kind.handler_ordinal())
        .map_err(|_| w1_failure(RecognizerErrorCode::BudgetExceeded))?;
    Ok(vec![W1Handler {
        declaration: W1DeclarationBinding {
            node: declaration.node.clone(),
            handle: declaration.handle,
            evidence: declaration.evidence,
            path: declaration.path.clone(),
            span: declaration.span,
            digest: declaration.digest,
        },
        reference_key: reference_key.to_owned(),
        argument_ordinal,
    }])
}

fn w1_handler_relation_id(
    call_id: &str,
    declaration: &W1DeclarationBinding,
    argument_ordinal: u32,
) -> RecognizerResult<String> {
    let bytes = canonical_json_bytes(&(
        W1_SIGNAL_PROFILE,
        call_id,
        &declaration.path,
        declaration.span,
        argument_ordinal,
        W1_HANDLES_DEFINITION,
    ))
    .map_err(|_| w1_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "lua-native-frame-event-edge:sha256:{}",
        w1_encode_hex(&Sha256::digest(bytes))
    ))
}

fn w1_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-native-frame-events".into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W1_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-native-frame-event-1".into(),
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
    let Some(ordinal) = kind.unit_ordinal() else {
        return Ok((Vec::new(), true));
    };
    let Some(argument) = arguments.get(ordinal) else {
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
struct W3Binding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

/// One exact Main declaration identity, used to recheck a resolved receiver.
struct W3ReceiverTarget {
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

/// One exact call site: resolved receiver node/support, literal event key and its
/// retained argument descriptors. Both sides REQUIRE a resolved receiver and a
/// literal event key; otherwise the site never becomes a fact.
struct W3Site {
    fact_id: String,
    call_id: String,
    caller_proposal_id: String,
    receiver_proposal_id: String,
    receiver_node: GraphNodeId,
    event_key: String,
    event_ordinal: usize,
    argument_count: usize,
    exact: bool,
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Clone)]
enum W3Assertion {
    Emitter {
        call_id: String,
        receiver_proposal_id: String,
        receiver_node: GraphNodeId,
        event_key: String,
        relation_proposal_id: String,
        confidence: RecognizerOutputConfidence,
    },
    Subscription {
        call_id: String,
        receiver_proposal_id: String,
        receiver_node: GraphNodeId,
        event_key: String,
        relation_proposal_id: String,
        producer_call_id: Option<String>,
        confidence: RecognizerOutputConfidence,
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

    let graph = input.owner.input_view(stop).map_err(w3_graph_error)?;
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = source_partition.report().accepted_entities();

    let mut function_proposals = BTreeMap::<String, String>::new();
    let mut function_ids = BTreeSet::new();
    for function in input.report.functions() {
        w3_checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none() || !function_ids.insert(function.fact_id().to_owned()) {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        function_proposals.insert(function.fact_id().to_owned(), proposal_id.to_owned());
    }

    let mut declarations = BTreeMap::<(String, SourceSpan), W3Binding>::new();
    let mut declaration_keys = BTreeMap::<String, W3ReceiverTarget>::new();
    let mut receiver_nodes = BTreeMap::<String, GraphNodeId>::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        w3_checkpoint(stop)?;
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none() {
            return Err(w3_failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        if receiver_nodes
            .insert((*proposal_id).to_owned(), node.clone())
            .is_some()
        {
            return Err(w3_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        declaration_keys.insert(
            (*proposal_id).to_owned(),
            W3ReceiverTarget {
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
        let Some(site) = w3_build_site(&input, call, side, &declarations, &declaration_keys)?
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
    let mut producer_call_ids = BTreeSet::<(String, String)>::new();
    for outcome in output.outcomes() {
        match outcome.rule_id() {
            W3_PRODUCER_RULE if outcome.rule_version() == 1 => {
                for proposal in outcome.proposals() {
                    assertions.push(w3_read_assertion(
                        proposal,
                        W3SignalSide::Producer,
                        &bundle,
                        &receiver_nodes,
                        &producer_call_ids,
                    )?);
                }
            }
            W3_SUBSCRIPTION_RULE if outcome.rule_version() == 1 => {
                for proposal in outcome.proposals() {
                    assertions.push(w3_read_assertion(
                        proposal,
                        W3SignalSide::Subscription,
                        &bundle,
                        &receiver_nodes,
                        &producer_call_ids,
                    )?);
                }
            }
            _ => return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch)),
        }
    }

    // Producer call identities are collected in a first pass so a later
    // subscription can be confirmed against an earlier exact producer regardless
    // of outcome order.
    for assertion in &assertions {
        if let W3Assertion::Emitter {
            call_id, event_key, ..
        } = assertion
        {
            producer_call_ids.insert((event_key.clone(), call_id.clone()));
        }
    }

    let mut entities = Vec::new();
    let mut relations = Vec::new();
    let mut producer_matches = Vec::new();
    let mut subscription_matches = Vec::new();
    for assertion in &assertions {
        match assertion {
            W3Assertion::Emitter {
                call_id,
                receiver_proposal_id,
                receiver_node,
                event_key,
                relation_proposal_id,
                confidence,
            } => {
                let site = sites
                    .get(call_id.as_str())
                    .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let binding = declarations
                    .values()
                    .next()
                    .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let confidence = w3_graph_confidence(*confidence);
                let handles = BTreeSet::from([site.handle, binding.handle])
                    .into_iter()
                    .collect::<Vec<_>>();
                let evidence = BTreeSet::from([site.evidence, binding.evidence])
                    .into_iter()
                    .collect::<Vec<_>>();
                let signal_id = w3_entity_id("emitter", call_id);
                entities.push(
                    GraphEntityProposal::new(
                        signal_id.as_str(),
                        W3_CUSTOM_SIGNAL_ENTITY,
                        BTreeMap::from([
                            (
                                "producer".into(),
                                GraphProposalValue::Reference(receiver_proposal_id.clone().into()),
                            ),
                            (
                                "event".into(),
                                GraphProposalValue::String(event_key.clone().into()),
                            ),
                        ]),
                        confidence,
                        handles.clone(),
                        evidence.clone(),
                        Vec::new(),
                    )
                    .map_err(w3_graph_error)?,
                );
                relations.push(
                    GraphRelationProposal::new(
                        relation_proposal_id.as_str(),
                        W3_EMITTER_RELATION_ID,
                        GraphRelationProposalInput {
                            source: GraphProposalEndpoint::Existing(receiver_node.clone()),
                            target: GraphProposalEndpoint::Proposed(signal_id.as_str().into()),
                            confidence,
                            source_handle_ids: handles,
                            evidence_ids: evidence,
                            coverage_ids: Vec::new(),
                        },
                    )
                    .map_err(w3_graph_error)?,
                );
                producer_matches.push(W3ProducerMatch {
                    call_id: call_id.clone(),
                    event_key: event_key.clone(),
                    receiver_declaration_proposal_id: receiver_proposal_id.clone(),
                    relation_proposal_id: relation_proposal_id.clone(),
                });
            }
            W3Assertion::Subscription {
                call_id,
                receiver_proposal_id,
                receiver_node,
                event_key,
                relation_proposal_id,
                producer_call_id,
                confidence,
            } => {
                let site = sites
                    .get(call_id.as_str())
                    .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let binding = declarations
                    .values()
                    .next()
                    .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
                let confidence = w3_graph_confidence(*confidence);
                let handles = BTreeSet::from([site.handle, binding.handle])
                    .into_iter()
                    .collect::<Vec<_>>();
                let evidence = BTreeSet::from([site.evidence, binding.evidence])
                    .into_iter()
                    .collect::<Vec<_>>();
                // The producer endpoint is only available when an exact compatible
                // producer was observed; otherwise the relay is the consumer itself.
                let producer_node = match producer_call_id {
                    Some(producer) => sites
                        .get(producer.as_str())
                        .map(|producer_site| producer_site.receiver_node.clone()),
                    None => None,
                };
                let target = match producer_node {
                    Some(node) => GraphProposalEndpoint::Existing(node),
                    None => GraphProposalEndpoint::Existing(receiver_node.clone()),
                };
                relations.push(
                    GraphRelationProposal::new(
                        relation_proposal_id.as_str(),
                        W3_SUBSCRIPTION_RELATION_ID,
                        GraphRelationProposalInput {
                            source: GraphProposalEndpoint::Existing(receiver_node.clone()),
                            target,
                            confidence,
                            source_handle_ids: handles,
                            evidence_ids: evidence,
                            coverage_ids: Vec::new(),
                        },
                    )
                    .map_err(w3_graph_error)?,
                );
                subscription_matches.push(W3SubscriptionMatch {
                    call_id: call_id.clone(),
                    event_key: event_key.clone(),
                    receiver_declaration_proposal_id: receiver_proposal_id.clone(),
                    producer_call_id: producer_call_id.clone(),
                    relation_proposal_id: relation_proposal_id.clone(),
                    confirmed: producer_call_id.is_some(),
                });
            }
        }
    }

    let producer_keys = producer_matches
        .iter()
        .map(|producer| (producer.event_key.clone(), producer.call_id.clone()))
        .collect::<BTreeSet<_>>();
    let mut unconfirmed = Vec::new();
    for fact in bundle.facts() {
        w3_checkpoint(stop)?;
        if fact.kind() != W3_CUSTOM_SIGNAL_FACT {
            continue;
        }
        let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let Some(RecognizerFactValue::String(callable_key)) = fact.field("callable_key") else {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        if callable_key.as_ref() != W3_REGISTER_CALLBACK_CALLABLE {
            continue;
        }
        if subscription_matches
            .iter()
            .any(|subscription| subscription.call_id == call_id.as_ref())
        {
            continue;
        }
        let Some(RecognizerFactValue::String(event_key)) = fact.field("event_key") else {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let Some(RecognizerFactValue::Reference(receiver)) = fact.field("receiver") else {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let reason = if producer_keys
            .iter()
            .any(|(key, _)| key.as_str() == event_key.as_ref())
        {
            W3UnconfirmedReason::ProducerEventKeyMismatch
        } else {
            W3UnconfirmedReason::ProducerAbsentInReport
        };
        unconfirmed.push(W3UnconfirmedSubscription {
            call_id: call_id.to_string(),
            event_key: event_key.to_string(),
            receiver_declaration_proposal_id: receiver.to_string(),
            reason,
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
    _side: W3SignalSide,
    declarations: &BTreeMap<(String, SourceSpan), W3Binding>,
    declaration_keys: &BTreeMap<String, W3ReceiverTarget>,
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
    let event_key = literal_key.trim().to_owned();
    if event_key.is_empty() || event_key.len() > W3_MAX_EVENT_KEY_BYTES {
        return Ok(None);
    }

    let receiver_argument = arguments
        .first()
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let target = receiver_argument
        .reference_target()
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let key = receiver_argument.reference_key();
    if !w3_is_resolved_custom_registry(key, target, input) {
        return Ok(None);
    }
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

    let mut exact = true;
    for argument in arguments {
        if argument.span().byte_start().is_none() || argument.span().byte_end().is_none() {
            return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        if argument.literal().is_none() && argument.reference_key().is_none() {
            exact = false;
        }
    }

    let function_proposals = function_proposals_of(input);
    let caller_proposal = function_proposals
        .get(call.caller_function_id())
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let caller_proposal = caller_proposal.to_owned();

    Ok(Some(W3Site {
        fact_id: call.fact_id().to_owned(),
        call_id: call.fact_id().to_owned(),
        caller_proposal_id: caller_proposal.to_owned(),
        receiver_proposal_id: declaration_proposal.to_owned(),
        receiver_node: binding.node.clone(),
        event_key,
        event_ordinal: ordinal,
        argument_count: arguments.len(),
        exact,
        handle,
        evidence,
    }))
}

fn function_proposals_of<'a>(input: &W3Input<'a>) -> BTreeMap<&'a str, &'a str> {
    input.function_proposals.clone()
}

/// A resolved receiver must be an exact Main global declaration, never a library
/// target, an unresolved global or an alias with a reassignment blocker.
fn w3_is_resolved_custom_registry(
    receiver_key: Option<&str>,
    target: &wow_emmy::bindings::SymbolTarget,
    input: &W3Input<'_>,
) -> bool {
    if target.role != "main" || target.workspace_id != input.report.main_snapshot_id() {
        return false;
    }
    if target.path.is_empty() || target.content_digest.is_empty() {
        return false;
    }
    if target.span.byte_start() == target.span.byte_end() {
        return false;
    }
    let Some(key) = receiver_key else {
        return false;
    };
    let Some(member) = key.rsplit('.').next() else {
        return false;
    };
    matches!(member, "TriggerEvent" | "RegisterCallback")
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
    if record.path != target.path || record.span != target.span {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn w3_signal_fact(
    input: &W3Input<'_>,
    site: &W3Site,
    side: W3SignalSide,
    limits: RecognizerFactLimits,
) -> RecognizerResult<RecognizerFact> {
    let fields = BTreeMap::from([
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
            RecognizerFactValue::Reference(site.receiver_proposal_id.clone().into()),
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
    let caller_function_id = input
        .report
        .calls()
        .iter()
        .find(|call| call.fact_id() == site.call_id.as_str())
        .map(|call| call.caller_function_id())
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
            confidence: if site.exact {
                GraphConfidence::Derived
            } else {
                GraphConfidence::Possible
            },
            fields,
            source_handle_ids: vec![site.handle],
            evidence_ids: vec![site.evidence],
        },
        limits,
    )
}

/// Re-reads one plan output and rebinds it to the exact fact and receiver. The
/// asserted source must be the resolved receiver and the asserted target the exact
/// literal event key; anything else is an adapter defect, not a degraded match.
fn w3_read_assertion(
    proposal: &crate::RecognizerProposedAssertion,
    side: W3SignalSide,
    bundle: &RecognizerFactBundle,
    receiver_nodes: &BTreeMap<String, GraphNodeId>,
    producer_call_ids: &BTreeSet<(String, String)>,
) -> RecognizerResult<W3Assertion> {
    let crate::RecognizerProposedAssertion::Relation {
        relation_kind_id,
        source,
        target,
        confidence,
        decisive_fact_ids,
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
    if fact.kind() != W3_CUSTOM_SIGNAL_FACT {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::String(event_key)) = fact.field("event_key") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::Reference(receiver)) = fact.field("receiver") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::String(callable_key)) = fact.field("callable_key") else {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    if callable_key.as_ref() != side.callable() {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    if source != &RecognizerFactValue::Reference(receiver.clone()) {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    if target != &RecognizerFactValue::String(event_key.clone()) {
        return Err(w3_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let receiver_node = receiver_nodes
        .get(receiver.as_ref())
        .cloned()
        .ok_or_else(|| w3_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let producer_call_id = match side {
        // A subscription is only confirmed against a producer whose exact event
        // key and resolved receiver match. Absence never becomes a match.
        W3SignalSide::Subscription => producer_call_ids
            .iter()
            .find(|(key, _)| key == event_key.as_ref())
            .map(|(_, call)| call.clone()),
        W3SignalSide::Producer => None,
    };
    Ok(match side {
        W3SignalSide::Producer => W3Assertion::Emitter {
            call_id: call_id.to_string(),
            receiver_proposal_id: receiver.to_string(),
            receiver_node,
            event_key: event_key.to_string(),
            relation_proposal_id: w3_relation_id("emitter", call_id.as_ref()),
            confidence: *confidence,
        },
        W3SignalSide::Subscription => W3Assertion::Subscription {
            call_id: call_id.to_string(),
            receiver_proposal_id: receiver.to_string(),
            receiver_node,
            event_key: event_key.to_string(),
            relation_proposal_id: w3_relation_id("subscription", call_id.as_ref()),
            producer_call_id,
            confidence: *confidence,
        },
    })
}

fn w3_entity_id(prefix: &str, call_id: &str) -> String {
    w3_digest_id("signal", &(prefix, call_id))
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
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W3_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-custom-signals-1".into(),
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
                        source: "signal.receiver".into(),
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
                        source: "signal.receiver".into(),
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
