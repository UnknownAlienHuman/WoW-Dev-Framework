#![allow(dead_code)]
//! Exact resolved EventRegistry-callback bridge facts -> declarative core
//! recognizer -> graph proposals. Source text is never reparsed here. Every key,
//! argument, span and support record comes from the generation-bound Emmy owner
//! report.
//!
//! This module recognizes the BRIDGE: the registry indirection from a native
//! event to a custom callback. An ordinary direct frame registration is a
//! different structural family and never enters this module. No runtime dispatch,
//! taint, combat legality, or platform-availability claim is ever made.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use sha2::{Digest, Sha256};

use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId, canonical_json_bytes,
};
use wow_emmy::function_calls::{FunctionCallReport, SourceCallArgument, SourceCallFact};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRef, GraphConfidence, GraphCoverageRecord,
    GraphCoverageState, GraphEntityProposal, GraphLocalAssertion, GraphNodeId,
    GraphPartitionSnapshot, GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue,
    GraphRelationKind, GraphRelationProposal, GraphRelationProposalInput,
};

use crate::source_signals::SignalEntityAccumulator;
use crate::{
    RecognizerClause, RecognizerError, RecognizerErrorCode, RecognizerFact, RecognizerFactBundle,
    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind,
    RecognizerFactValue, RecognizerOutput, RecognizerOutputConfidence, RecognizerPack,
    RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerResult, RecognizerRule, compile_recognizer_plan, execute_recognizer_plan,
    parse_recognizer_pack,
};

pub const W2_PARTITION: &str = "wow-recognizers.lua-native-event-bridges";
pub const W2_PROFILE: &str = "wow-recognizers/lua-native-event-bridges/5";
const W2_FACT_PARTITION: &str = "wow-recognizers.lua-native-event-bridge-facts";
const W2_FACT_PROFILE: &str = "wow-recognizers-lua-native-event-bridge-facts-5";
const W2_BRIDGE_RULE: &str = "core.signal.native_event_registry_bridge";
const W2_FACT_KIND: &str = "lua_native_event_bridge";
const W2_NATIVE_EVENT_ENTITY: &str = "native_event";
const W2_REGISTERS_RELATION: &str = "lua_registers_native_event";
const W2_BRIDGES_RELATION: &str = "lua_bridges_native_event";
const W2_CAPABILITY: &str = "emmy.fact.calls";
const W2_UNMODELED_FORM_BLOCKER: &str =
    "wow_recognizers.native_event_bridge.unmodeled_registration_or_handle_form";

const MAX_CALLS: usize = 4096;
const MAX_SITES: usize = 65_536;
const MAX_EVENT_KEY_BYTES: usize = 1024;
const MAX_ARGUMENTS: usize = 8;

/// Exact resolved Library registry callables that bind a native event and an
/// exact Main callback. The EventRegistry is the colon receiver; positional
/// arguments are the event key and callback, in that order.
const W2_REGISTER_CALLABLES: [&str; 1] = ["EventRegistry.RegisterFrameEventAndCallback"];
const W2_UNREGISTER_CALLABLES: [&str; 0] = [];
/// Exact resolved Library-owned EventRegistry receivers. Main shadows and bare
/// spellings do not establish Blizzard native-event bridge semantics.
const W2_POSITIVE_FIXTURE_IDS: [&str; 2] = ["RECOG-EVENT-001", "RECOG-EVENT-004"];
const W2_NEAR_NEGATIVE_FIXTURE_IDS: [&str; 2] = ["RECOG-EVENT-007", "RECOG-EVENT-009"];
const W2_PARTIAL_FIXTURE_IDS: [&str; 2] = ["RECOG-EVENT-004", "RECOG-EVENT-005"];
const W2_MUTATION_FIXTURE_IDS: [&str; 2] = ["RECOG-EVENT-001", "RECOG-EVENT-009"];

/// Caller-side crosswalks are checked against the real source proposals, support
/// records and the exact analyzer report. No name, path or repository text is
/// ever consulted, and no source text is reparsed.
pub struct W2Input<'a> {
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

/// One exact resolved bridge site. The registry receiver, the event literal and
/// the span evidence are always exact; the frame/target receiver and the callback
/// are exact or explicitly absent, never fabricated.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W2BridgeMatch {
    pub call_id: String,
    pub event_key: String,
    pub event_ordinal: usize,
    pub registers_native_event: bool,
    pub registry_workspace_id: String,
    pub registry_target_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_declaration_proposal_id: Option<String>,
    pub callback_declaration_proposal_id: String,
    pub event_entity_proposal_id: String,
    pub registers_relation_proposal_id: String,
    pub bridges_relation_proposal_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct W2BridgeRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    bridges: Vec<W2BridgeMatch>,
}

impl W2BridgeRecognition {
    pub fn bridges(&self) -> &[W2BridgeMatch] {
        &self.bridges
    }
}

pub struct W2BridgeProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: W2BridgeRecognition,
}

/// One resolved Main declaration owning its node and support records.
#[derive(Debug, Clone, PartialEq, Eq)]
struct W2Binding {
    proposal_id: String,
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum W2Side {
    Registers,
    Unregisters,
}

impl W2Side {
    const fn callables(self) -> &'static [&'static str] {
        match self {
            Self::Registers => &W2_REGISTER_CALLABLES,
            Self::Unregisters => &W2_UNREGISTER_CALLABLES,
        }
    }

    const fn is_registration(self) -> bool {
        matches!(self, Self::Registers)
    }
}

/// One exact bridge call site after full resolution. Both the registry receiver
/// and the event key are always exact; the frame/target receiver is exact or, for a
/// registering frame-event form, the site is never admitted.
struct W2Site {
    call_id: String,
    caller_proposal_id: String,
    caller_function_id: String,
    caller_node: GraphNodeId,
    registry_workspace_id: String,
    registry_target_identity: String,
    event_key: String,
    event_ordinal: usize,
    frame_binding: Option<W2Binding>,
    frame_node: Option<GraphNodeId>,
    frame_proposal_id: Option<String>,
    callback_binding: W2Binding,
    registration: bool,
    handle: StableHandleId,
    evidence: EvidenceId,
}

struct W2RegistryTarget {
    workspace_id: String,
    identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum W2Relation {
    Registers,
    Bridges,
}

impl W2Relation {
    const fn kind(self) -> GraphRelationKind {
        match self {
            Self::Registers => GraphRelationKind::RegistersNativeEvent,
            Self::Bridges => GraphRelationKind::BridgesNativeEvent,
        }
    }

    const fn relation_id(self) -> &'static str {
        match self {
            Self::Registers => W2_REGISTERS_RELATION,
            Self::Bridges => W2_BRIDGES_RELATION,
        }
    }
}

/// Recognizes exact resolved EventRegistry-callback bridges from the exact Emmy
/// function-call report. The bridge is the registry indirection from a native
/// event to a custom callback, never an ordinary direct frame registration.
pub fn w2_recognize_native_event_bridges(
    input: W2Input<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<W2BridgeProposals> {
    w2_checkpoint(stop)?;
    if input.report.calls().len() > MAX_CALLS
        || input.call_support.len() != input.report.calls().len()
        || input.function_proposals.len() != input.report.functions().len()
        || input.declaration_proposals.len() > MAX_CALLS
    {
        return Err(w2_failure(RecognizerErrorCode::BudgetExceeded));
    }
    input
        .report
        .validate()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    input
        .context
        .validate()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    if input.owner.source_context_id() != input.context.context_id() {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let lookup = input.owner.producer_lookup(stop).map_err(w2_graph_error)?;
    let graph = lookup.input_view();
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;

    let mut function_nodes = BTreeMap::<String, GraphNodeId>::new();
    for function in input.report.functions() {
        w2_checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
            .map_err(w2_lookup_error)?;
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
            return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        w2_validate_support(&input, proposal, function)?;
        let node = resolved.accepted().node().node_id().clone();
        if function_nodes
            .insert(proposal_id.to_owned(), node)
            .is_some()
        {
            return Err(w2_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let mut declarations = BTreeMap::<(String, SourceSpan), W2Binding>::new();

    for ((path, span), proposal_id) in &input.declaration_proposals {
        w2_checkpoint(stop)?;
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
            .map_err(w2_lookup_error)?;
        let proposal = resolved.proposal();
        let (Some(start), Some(end)) = (span.byte_start(), span.byte_end()) else {
            return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
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
                        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
        ]);
        if proposal.entity_kind_id() != "lua_source_declaration"
            || proposal.semantic_key() != &expected
            || proposal.confidence() != GraphConfidence::Derived
        {
            return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
            return Err(w2_failure(RecognizerErrorCode::AdapterBindingInvalid));
        };
        w2_validate_support_without_digest(&input, *handle, *evidence, path, *span)?;
        let node = resolved.accepted().node().node_id().clone();
        let _record = input
            .source_handles
            .get(handle)
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if declarations
            .insert(
                ((*path).clone(), *span),
                W2Binding {
                    proposal_id: (*proposal_id).to_owned(),
                    node,
                    handle: *handle,
                    evidence: *evidence,
                },
            )
            .is_some()
        {
            return Err(w2_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut sites = BTreeMap::<String, W2Site>::new();
    let mut site_count = 0usize;
    let mut unresolved_candidates = false;
    let mut unmodeled_candidates = false;
    for call in input.report.calls() {
        w2_checkpoint(stop)?;
        let Some(side) = w2_side(call) else {
            unmodeled_candidates |= w2_unmodeled_registration(call);
            continue;
        };
        let (handle, evidence) = *input
            .call_support
            .get(call.fact_id())
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w2_validate_support_for_call(&input, handle, evidence, call)?;

        let Some(registry_target) = w2_registry_binding(input.report, call) else {
            unresolved_candidates = true;
            continue;
        };

        let arguments = call.arguments();
        if arguments.len() > MAX_ARGUMENTS {
            return Err(w2_failure(RecognizerErrorCode::BudgetExceeded));
        }
        let shape = w2_argument_shape(call, side);
        let Some(event_argument) = arguments.get(shape.event_ordinal) else {
            return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let Some(wow_emmy::function_calls::SourceCallLiteral::String(literal_key)) =
            event_argument.literal()
        else {
            unresolved_candidates = true;
            continue;
        };
        // String literals are already decoded here; their exact value is the key.
        let event_key = literal_key.to_owned();
        if event_key.is_empty() || event_key.len() > MAX_EVENT_KEY_BYTES {
            if event_key.len() > MAX_EVENT_KEY_BYTES {
                return Err(w2_failure(RecognizerErrorCode::BudgetExceeded));
            }
            unresolved_candidates = true;
            continue;
        }

        // Current Blizzard `RegisterFrameEventAndCallback*` forms take the event
        // key first and the callback second. The EventRegistry object is the Lua
        // colon receiver, not an ordinary positional argument.
        let frame_binding = None;
        let Some(callback_ordinal) = shape.callback_ordinal else {
            unresolved_candidates = true;
            continue;
        };
        let callback_binding = arguments.get(callback_ordinal).and_then(|argument| {
            w2_frame_binding(
                Some(argument),
                &declarations,
                input.report.main_snapshot_id(),
            )
            .resolved()
        });
        let Some(callback_binding) = callback_binding else {
            unresolved_candidates = true;
            continue;
        };

        site_count = site_count
            .checked_add(1)
            .ok_or_else(|| w2_failure(RecognizerErrorCode::BudgetExceeded))?;
        if site_count > MAX_SITES {
            return Err(w2_failure(RecognizerErrorCode::BudgetExceeded));
        }
        let caller_function_id = call.caller_function_id().to_owned();
        let caller_proposal_id = w2_caller_proposal(&input, &caller_function_id)?.to_owned();
        let caller_node = function_nodes
            .get(&caller_proposal_id)
            .cloned()
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let site = W2Site {
            call_id: call.fact_id().into(),
            caller_proposal_id,
            caller_function_id,
            caller_node,
            registry_workspace_id: registry_target.workspace_id,
            registry_target_identity: registry_target.identity,
            event_key,
            event_ordinal: shape.event_ordinal,
            frame_binding: frame_binding.clone(),
            frame_node: frame_binding.as_ref().map(|binding| binding.node.clone()),
            frame_proposal_id: frame_binding
                .as_ref()
                .map(|binding| binding.proposal_id.clone()),
            registration: side.is_registration(),
            callback_binding,
            handle,
            evidence,
        };
        facts.push(w2_signal_fact(&input, &site, fact_limits)?);
        if sites.insert(site.call_id.clone(), site).is_some() {
            return Err(w2_failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let source_complete = input.report.source_health_complete();
    let coverage_state = if !source_complete {
        RecognizerFactCoverageState::NotEvaluated
    } else if unresolved_candidates || unmodeled_candidates {
        RecognizerFactCoverageState::Partial
    } else {
        RecognizerFactCoverageState::Complete
    };
    let mut blocker_ids = Vec::new();
    if !source_complete {
        blocker_ids.push("emmy.call_source_parse_failed".into());
    }
    if unresolved_candidates {
        blocker_ids
            .push("wow_recognizers.native_event_bridge.receiver_or_callback_unresolved".into());
    }
    if unmodeled_candidates {
        blocker_ids.push(W2_UNMODELED_FORM_BLOCKER.into());
    }
    let coverage = vec![RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: input.context.context_id(),
            partition_id: W2_FACT_PARTITION.into(),
            capability_id: W2_CAPABILITY.into(),
            producer_id: "wow.emmy".into(),
            producer_version: W2_FACT_PROFILE.into(),
            state: coverage_state,
            blocker_ids,
        },
        fact_limits,
    )?];
    let bundle = RecognizerFactBundle::build(
        input.context,
        W2_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = w2_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    // The plan asserts exactly one native-event entity and two relations per site:
    // the registration and the bridge. Both relations are rechecked against the
    // declaring fact, the exact receiver nodes and the exact event entity.
    let mut events = Vec::new();
    let mut relations = Vec::new();
    for outcome in output.outcomes() {
        if outcome.rule_id() != W2_BRIDGE_RULE || outcome.rule_version() != 1 {
            return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        for proposal in outcome.proposals() {
            match proposal {
                crate::RecognizerProposedAssertion::Entity {
                    proposal_id: _,
                    entity_kind_id,
                    semantic_key,
                    confidence,
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                    ..
                } => {
                    if entity_kind_id.as_ref() != W2_NATIVE_EVENT_ENTITY {
                        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let Some(RecognizerFactValue::String(event_key)) = semantic_key.get("event")
                    else {
                        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
                    };
                    events.push(w2_event_proposal(
                        event_key,
                        *confidence,
                        source_handle_ids,
                        evidence_ids,
                        coverage_ids,
                    )?);
                }
                crate::RecognizerProposedAssertion::Relation {
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
                } => {
                    let request = W2RelationRequest {
                        proposal_id: proposal_id.to_string(),
                        relation: w2_relation_kind(relation_kind_id.as_ref())?,
                        source: source.clone(),
                        target: target.clone(),
                        confidence: *confidence,
                        source_handle_ids: source_handle_ids.clone(),
                        decisive_fact_ids: decisive_fact_ids.clone(),
                        evidence_ids: evidence_ids.clone(),
                        coverage_ids: coverage_ids.clone(),
                    };
                    relations.push(request);
                }
            }
        }
    }
    if relations.len() != sites.len() * 2 {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingMissing));
    }

    let mut event_entities = BTreeMap::<String, SignalEntityAccumulator>::new();
    for event in events {
        let entity_id = w2_event_entity_id(&event.event_key)?;
        let accumulator = event_entities
            .entry(event.event_key.clone())
            .or_insert_with(|| SignalEntityAccumulator::new(entity_id.clone(), event.confidence));
        if accumulator.proposal_id != entity_id {
            return Err(w2_failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        accumulator.merge(
            event.confidence,
            &event.source_handle_ids,
            &event.evidence_ids,
            &event.coverage_ids,
        );
    }
    let mut event_entities_by_key = BTreeMap::<String, String>::new();
    let mut entity_proposals = Vec::with_capacity(event_entities.len());
    for (event_key, accumulator) in event_entities {
        event_entities_by_key.insert(event_key.clone(), accumulator.proposal_id.clone());
        entity_proposals.push(
            GraphEntityProposal::new(
                accumulator.proposal_id,
                W2_NATIVE_EVENT_ENTITY,
                BTreeMap::from([("event".into(), GraphProposalValue::String(event_key.into()))]),
                accumulator.confidence,
                accumulator.source_handle_ids.into_iter().collect(),
                accumulator.evidence_ids.into_iter().collect(),
                accumulator.coverage_ids.into_iter().collect(),
            )
            .map_err(w2_graph_error)?,
        );
    }

    let mut graph_relations = Vec::new();
    let mut registrations = BTreeMap::<String, String>::new();
    let mut bridge_map = BTreeMap::<String, String>::new();
    for request in relations {
        let call_id = w2_relation_call_id(&request, &bundle)?;
        let site = sites
            .get(&call_id)
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let event_entity_proposal_id = event_entities_by_key
            .get(&site.event_key)
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
        w2_push_relation(
            &request,
            site,
            event_entity_proposal_id,
            &mut graph_relations,
            &mut registrations,
            &mut bridge_map,
        )?;
    }
    if graph_relations.len() != sites.len() * 2
        || registrations.len() != sites.len()
        || bridge_map.len() != sites.len()
    {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingMissing));
    }

    let mut bridges = Vec::new();
    for (call_id, site) in sites {
        let event_entity_proposal_id = event_entities_by_key
            .get(&site.event_key)
            .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?
            .clone();
        bridges.push(W2BridgeMatch {
            call_id: call_id.clone(),
            event_key: site.event_key.clone(),
            event_ordinal: site.event_ordinal,
            registers_native_event: site.registration,
            registry_workspace_id: site.registry_workspace_id.clone(),
            registry_target_identity: site.registry_target_identity.clone(),
            frame_declaration_proposal_id: site.frame_binding.map(|id| id.proposal_id),
            callback_declaration_proposal_id: site.callback_binding.proposal_id,
            event_entity_proposal_id,
            registers_relation_proposal_id: registrations
                .get(&call_id)
                .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone(),
            bridges_relation_proposal_id: bridge_map
                .get(&call_id)
                .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone(),
        });
    }
    if bridges.len() != bridge_map.len() {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingMissing));
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
                GraphRelationKind::RegistersNativeEvent => (
                    GraphCoverageState::Partial,
                    "lua_native_event_bridges.exact_registry_receiver_and_event_key_only",
                ),
                GraphRelationKind::BridgesNativeEvent => (
                    GraphCoverageState::Partial,
                    "lua_native_event_bridges.exact_resolved_callback_only",
                ),
                _ => (
                    GraphCoverageState::NotEvaluated,
                    "lua_native_event_bridges.relation_owned_by_other_producer",
                ),
            };
            GraphCoverageRecord::new(relation, state, false, vec![blocker.into()], graph.limits())
                .map_err(w2_graph_error)
        })
        .collect::<RecognizerResult<Vec<_>>>()?;
    let batch = GraphProposalBatch::build(
        input.owner.registry().bundle_id(),
        input.owner.registry().registry_digest(),
        graph.universe().clone(),
        graph.generation().clone(),
        input.context.context_id(),
        W2_PARTITION,
        entity_proposals,
        graph_relations,
    )
    .map_err(w2_graph_error)?;

    Ok(W2BridgeProposals {
        batch,
        coverage: graph_coverage,
        recognition: W2BridgeRecognition {
            profile: W2_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            bridges,
        },
    })
}

struct W2EventProposal {
    event_key: String,
    confidence: GraphConfidence,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<wow_core::CoverageId>,
}

struct W2RelationRequest {
    proposal_id: String,
    relation: W2Relation,
    source: RecognizerFactValue,
    target: RecognizerFactValue,
    confidence: RecognizerOutputConfidence,
    decisive_fact_ids: Vec<crate::RecognizerFactId>,
    source_handle_ids: Vec<StableHandleId>,
    evidence_ids: Vec<EvidenceId>,
    coverage_ids: Vec<wow_core::CoverageId>,
}

fn w2_event_proposal(
    event_key: &str,
    confidence: RecognizerOutputConfidence,
    source_handle_ids: &[StableHandleId],
    evidence_ids: &[wow_core::EvidenceId],
    coverage_ids: &[wow_core::CoverageId],
) -> RecognizerResult<W2EventProposal> {
    Ok(W2EventProposal {
        event_key: event_key.to_string(),
        confidence: w2_graph_confidence(confidence),
        source_handle_ids: source_handle_ids.to_vec(),
        evidence_ids: evidence_ids.to_vec(),
        coverage_ids: coverage_ids.to_vec(),
    })
}

fn w2_event_entity_id(event_key: &str) -> RecognizerResult<String> {
    let bytes = canonical_json_bytes(&(W2_PROFILE, W2_NATIVE_EVENT_ENTITY, event_key))
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "native-event:sha256:{}",
        w2_hex(&Sha256::digest(bytes))
    ))
}

fn w2_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn w2_relation_kind(relation_kind_id: &str) -> RecognizerResult<W2Relation> {
    match relation_kind_id {
        W2_REGISTERS_RELATION => Ok(W2Relation::Registers),
        W2_BRIDGES_RELATION => Ok(W2Relation::Bridges),
        _ => Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch)),
    }
}

/// The decisive fact must be this module's bridge fact and must declare the exact
/// call, registry receiver, event key and frame receiver.
fn w2_relation_call_id(
    request: &W2RelationRequest,
    bundle: &RecognizerFactBundle,
) -> RecognizerResult<String> {
    let [fact_id] = request.decisive_fact_ids.as_slice() else {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let fact = bundle
        .fact_by_id(fact_id)
        .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if fact.kind() != W2_FACT_KIND {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::String(event_key)) = fact.field("event_key") else {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let Some(RecognizerFactValue::Reference(caller)) = fact.field("caller") else {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    };
    let expected_source = RecognizerFactValue::Reference(caller.clone());
    let expected_target = RecognizerFactValue::String(event_key.clone());
    if request.source != expected_source || request.target != expected_target {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(call_id.to_string())
}

/// Rebinds one plan relation to its exact receiver nodes, target event entity and
/// evidence. The registration relation runs from the registry receiver; the bridge
/// relation runs from the frame/target receiver when it resolved.
#[allow(clippy::too_many_arguments)]
fn w2_push_relation(
    request: &W2RelationRequest,
    site: &W2Site,
    event_entity_proposal_id: &str,
    relations: &mut Vec<GraphRelationProposal>,
    registrations: &mut BTreeMap<String, String>,
    bridge_map: &mut BTreeMap<String, String>,
) -> RecognizerResult<()> {
    if request.source != RecognizerFactValue::Reference(site.caller_proposal_id.clone().into()) {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let expected_target = RecognizerFactValue::String(site.event_key.clone().into());
    if request.target != expected_target {
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let source = GraphProposalEndpoint::Existing(site.caller_node.clone());
    let owned = match request.relation {
        W2Relation::Registers => registrations,
        W2Relation::Bridges => bridge_map,
    };
    if owned
        .insert(site.call_id.clone(), request.proposal_id.clone())
        .is_some()
    {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingDuplicate));
    }
    relations.push(
        GraphRelationProposal::new(
            request.proposal_id.clone(),
            request.relation.relation_id(),
            GraphRelationProposalInput {
                source,
                target: GraphProposalEndpoint::Proposed(event_entity_proposal_id.into()),
                confidence: w2_graph_confidence(request.confidence),
                source_handle_ids: request.source_handle_ids.clone(),
                evidence_ids: request.evidence_ids.clone(),
                coverage_ids: request.coverage_ids.clone(),
            },
        )
        .map_err(w2_graph_error)?,
    );
    Ok(())
}

fn w2_side(call: &SourceCallFact) -> Option<W2Side> {
    if !call.is_colon_call() {
        return None;
    }
    W2_REGISTER_CALLABLES
        .contains(&call.resolved_callable_key()?)
        .then_some(W2Side::Registers)
}

fn w2_unmodeled_registration(call: &SourceCallFact) -> bool {
    call.is_colon_call()
        && matches!(
            call.resolved_callable_key(),
            Some(
                "EventRegistry.RegisterFrameEvent"
                    | "EventRegistry.RegisterFrameEventAndCallbackWithHandle"
            )
        )
}

/// Current EventRegistry bridge forms take the native event key first. The
/// `*AndCallback*` forms take the callback second; `RegisterFrameEvent` has no
/// callback argument. The registry itself is the colon receiver.
fn w2_argument_shape(call: &SourceCallFact, _side: W2Side) -> W2ArgumentShape {
    let callback_ordinal = call
        .resolved_callable_key()
        .is_some_and(|key| key.contains("AndCallback"))
        .then_some(1);
    W2ArgumentShape {
        event_ordinal: 0,
        frame_ordinal: None,
        callback_ordinal,
    }
}

struct W2ArgumentShape {
    event_ordinal: usize,
    frame_ordinal: Option<usize>,
    callback_ordinal: Option<usize>,
}

/// Resolution of a receiver argument: an exact Main declaration, none at all, or
/// anything else (a library target, an unresolved global, a reassigned alias).
enum W2FrameResolution {
    Resolved(W2Binding),
    Absent,
    Unresolved,
}

impl W2FrameResolution {
    fn resolved(self) -> Option<W2Binding> {
        match self {
            Self::Resolved(binding) => Some(binding),
            Self::Absent | Self::Unresolved => None,
        }
    }
}

/// Resolves a receiver argument against the exact Main declaration crosswalk.
fn w2_frame_binding(
    argument: Option<&SourceCallArgument>,
    declarations: &BTreeMap<(String, SourceSpan), W2Binding>,
    main_snapshot_id: &str,
) -> W2FrameResolution {
    let Some(argument) = argument else {
        return W2FrameResolution::Absent;
    };
    let Some(target) = argument.reference_target() else {
        return W2FrameResolution::Unresolved;
    };
    if target.role != "main" || target.workspace_id != main_snapshot_id {
        return W2FrameResolution::Unresolved;
    }
    declarations
        .get(&(target.path.clone(), target.span))
        .cloned()
        .map_or(W2FrameResolution::Unresolved, W2FrameResolution::Resolved)
}

/// Resolves the registry owner argument of a bridge call. The receiver must
/// resolve to an exact Main declaration reached by an exact EventRegistry
/// receiver resolution, never a bare `EventRegistry` name expression.
fn w2_registry_binding(
    report: &FunctionCallReport,
    call: &SourceCallFact,
) -> Option<W2RegistryTarget> {
    let receiver = report.exact_call_receiver(call)?;
    let target = receiver.target();
    if receiver.key() != "EventRegistry"
        || target.role != "library"
        || !report.library_snapshot_ids().contains(&target.workspace_id)
        || target.path.is_empty()
    {
        return None;
    }
    let content_digest = target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .ok()?;
    let (Some(start), Some(end)) = (target.span.byte_start(), target.span.byte_end()) else {
        return None;
    };
    if start >= end {
        return None;
    }
    let bytes = canonical_json_bytes(&(
        target.role,
        target.workspace_id.as_str(),
        target.path.as_str(),
        target.span,
        content_digest,
    ))
    .ok()?;
    Some(W2RegistryTarget {
        workspace_id: target.workspace_id.clone(),
        identity: format!("event-registry:sha256:{}", w2_hex(&Sha256::digest(bytes))),
    })
}

fn w2_caller_proposal<'a>(
    input: &W2Input<'a>,
    caller_function_id: &str,
) -> RecognizerResult<&'a str> {
    input
        .function_proposals
        .get(caller_function_id)
        .copied()
        .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))
}

/// Builds one bridge fact from one resolved site. The fact declares the call, the
/// caller, the registry receiver, the frame/target receiver, the callback and the
/// exact event key. Nothing here re-reads source text.
fn w2_signal_fact(
    input: &W2Input<'_>,
    site: &W2Site,
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
            "registry_receiver".into(),
            RecognizerFactValue::Reference(site.registry_target_identity.clone().into()),
        ),
        (
            "registry_workspace_id".into(),
            RecognizerFactValue::String(site.registry_workspace_id.clone().into()),
        ),
        (
            "event_key".into(),
            RecognizerFactValue::String(site.event_key.clone().into()),
        ),
        (
            "event_ordinal".into(),
            RecognizerFactValue::Integer(
                i64::try_from(site.event_ordinal)
                    .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?,
            ),
        ),
        (
            "registers_native_event".into(),
            RecognizerFactValue::Boolean(site.registration),
        ),
    ]);
    if let Some(frame) = site.frame_binding.as_ref() {
        fields.insert(
            "frame_receiver".into(),
            RecognizerFactValue::Reference(frame.proposal_id.clone().into()),
        );
    }
    fields.insert(
        "callback".into(),
        RecognizerFactValue::Reference(site.callback_binding.proposal_id.clone().into()),
    );
    let mut source_handle_ids = BTreeSet::from([site.handle]);
    let mut evidence_ids = BTreeSet::from([site.evidence]);
    source_handle_ids.insert(site.callback_binding.handle);
    evidence_ids.insert(site.callback_binding.evidence);
    RecognizerFact::new(
        input.context.context_id(),
        RecognizerFactInput {
            kind: W2_FACT_KIND.into(),
            partition_id: W2_FACT_PARTITION.into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                site.caller_function_id.clone(),
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: W2_FACT_PROFILE.into(),
            confidence: GraphConfidence::Derived,
            fields,
            source_handle_ids: source_handle_ids.into_iter().collect(),
            evidence_ids: evidence_ids.into_iter().collect(),
        },
        limits,
    )
}

fn w2_graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn w2_checkpoint(stop: &AtomicBool) -> RecognizerResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(w2_failure(RecognizerErrorCode::Cancelled))
    } else {
        Ok(())
    }
}

fn w2_failure(code: RecognizerErrorCode) -> RecognizerError {
    RecognizerError::new(
        code,
        "exact EventRegistry bridge facts could not produce a coherent native-event partition",
    )
}

fn w2_graph_error(error: wow_graph::GraphError) -> RecognizerError {
    w2_failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}

fn w2_lookup_error(error: wow_graph::GraphError) -> RecognizerError {
    if error.code() == wow_graph::GraphErrorCode::PartitionInvalid {
        w2_failure(RecognizerErrorCode::AdapterBindingMissing)
    } else {
        w2_graph_error(error)
    }
}

fn w2_validate_support(
    input: &W2Input<'_>,
    proposal: &wow_graph::GraphEntityProposal,
    function: &wow_emmy::function_calls::SourceFunctionFact,
) -> RecognizerResult<()> {
    let ([handle], [evidence]) = (proposal.source_handle_ids(), proposal.evidence_ids()) else {
        return Err(w2_failure(RecognizerErrorCode::AdapterBindingInvalid));
    };
    let digest = function
        .content_digest()
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    w2_validate_support_record(
        input,
        *handle,
        *evidence,
        function.path(),
        function.span(),
        Some(digest),
    )
}

fn w2_validate_support_for_call(
    input: &W2Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    call: &SourceCallFact,
) -> RecognizerResult<()> {
    let digest = call
        .content_digest()
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    w2_validate_support_record(
        input,
        handle_id,
        evidence_id,
        call.path(),
        call.call_span(),
        Some(digest),
    )
}

fn w2_validate_support_without_digest(
    input: &W2Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    w2_validate_support_record(input, handle_id, evidence_id, path, span, None)
}

fn w2_validate_support_record(
    input: &W2Input<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
    digest: Option<ContentDigest<SourceContent>>,
) -> RecognizerResult<()> {
    let handle = input
        .source_handles
        .get(&handle_id)
        .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let evidence = input
        .evidence
        .get(&evidence_id)
        .ok_or_else(|| w2_failure(RecognizerErrorCode::AdapterBindingMissing))?;
    handle
        .validate()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?;
    evidence
        .validate()
        .map_err(|_| w2_failure(RecognizerErrorCode::AdapterFactMismatch))?;
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
        return Err(w2_failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

/// One declarative rule per bridge relation kind. The rule matches only bridge
/// facts declared by this module and asserts the exact receiver and event entity.
fn w2_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-native-event-bridges".into(),
            version: "5".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: W2_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-native-event-bridge-5".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 16,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 65_536,
                max_proposals_per_rule_partition: 65_536,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![RecognizerRule {
                rule_id: W2_BRIDGE_RULE.into(),
                version: 1,
                required_capabilities: vec![W2_CAPABILITY.into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "bridge".into(),
                        kind: W2_FACT_KIND.into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "bridge.registers_native_event".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                ],
                captures: Vec::new(),
                outputs: vec![
                    RecognizerOutput::EntityAssertion {
                        output_id: "native_event_entity".into(),
                        entity_kind_id: W2_NATIVE_EVENT_ENTITY.into(),
                        semantic_key: BTreeMap::from([("event".into(), "bridge.event_key".into())]),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "registry_registers_native_event".into(),
                        relation_kind_id: W2_REGISTERS_RELATION.into(),
                        source: "bridge.caller".into(),
                        target: "bridge.event_key".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "registry_bridges_native_event".into(),
                        relation_kind_id: W2_BRIDGES_RELATION.into(),
                        source: "bridge.caller".into(),
                        target: "bridge.event_key".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                ],
                positive_fixture_ids: (W2_POSITIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                near_negative_fixture_ids: (W2_NEAR_NEGATIVE_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                partial_fixture_ids: (W2_PARTIAL_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
                mutation_fixture_ids: (W2_MUTATION_FIXTURE_IDS
                    .iter()
                    .map(|value| (*value).into())
                    .collect::<Vec<Box<str>>>()),
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| w2_failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

#[cfg(test)]
mod w2_receiver_tests {
    use super::*;

    #[test]
    fn callback_bridge_receiver_must_be_the_exact_library_registry()
    -> Result<(), Box<dyn std::error::Error>> {
        let (report, library_workspace_id) =
            crate::source_signals::source_receiver_test_support::function_call_report(false)?;
        let callback_call = report
            .calls()
            .iter()
            .find(|call| {
                call.resolved_callable_key() == Some("EventRegistry.RegisterFrameEventAndCallback")
            })
            .ok_or("Library callback bridge call")?;
        let target = w2_registry_binding(&report, callback_call).ok_or("Library registry")?;
        assert_eq!(target.workspace_id, library_workspace_id);
        assert!(target.identity.starts_with("event-registry:sha256:"));

        let plain_registration = report
            .calls()
            .iter()
            .find(|call| call.resolved_callable_key() == Some("EventRegistry.RegisterFrameEvent"))
            .ok_or("plain native registration call")?;
        assert!(w2_side(plain_registration).is_none());
        assert!(w2_unmodeled_registration(plain_registration));

        let handle_registration = report
            .calls()
            .iter()
            .find(|call| {
                call.resolved_callable_key()
                    == Some("EventRegistry.RegisterFrameEventAndCallbackWithHandle")
            })
            .ok_or("handle-returning registration call")?;
        assert!(w2_side(handle_registration).is_none());
        assert!(w2_unmodeled_registration(handle_registration));

        let (shadow_report, _) =
            crate::source_signals::source_receiver_test_support::function_call_report(true)?;
        let shadow_call = shadow_report
            .calls()
            .iter()
            .find(|call| {
                call.resolved_callable_key() == Some("EventRegistry.RegisterFrameEventAndCallback")
            })
            .ok_or("Main-shadow callback bridge call")?;
        let shadow_receiver = shadow_report
            .exact_call_receiver(shadow_call)
            .ok_or("exact Main-shadow target")?;
        assert_eq!(shadow_receiver.target().role, "main");
        assert!(w2_registry_binding(&shadow_report, shadow_call).is_none());
        Ok(())
    }
}
// ===== END WORKER 2: native event registry bridge =====
