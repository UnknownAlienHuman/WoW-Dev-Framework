//! Exact CreateFromMixins facts -> declarative core recognizer -> graph proposals.
//! This producer consumes only generation-bound owner facts. It never reparses Lua,
//! executes mixins, infers from local variable names, or claims runtime instances.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, SourceSpan, StableHandleId, canonical_json_bytes,
};
use wow_emmy::bindings::SymbolTarget;
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

pub const SOURCE_MIXIN_PARTITION: &str = "wow-recognizers.lua-mixins";
pub const SOURCE_MIXIN_PROFILE: &str = "wow-recognizers/lua-mixins/1";
const FACT_PARTITION: &str = "wow-recognizers.lua-mixin-facts";
const FACT_PROFILE: &str = "wow-recognizers-lua-mixin-facts-1";
const CREATE_FROM_MIXINS_RULE: &str = "core.lua.create_from_mixins";
const CREATE_FROM_MIXINS_CALLABLE: &str = "CreateFromMixins";
pub const SOURCE_MIXIN_ASSIGNMENT_PARTITION: &str = "wow-recognizers.lua-mixin-assignments";
pub const SOURCE_MIXIN_ASSIGNMENT_PROFILE: &str = "wow-recognizers/lua-mixin-assignments/1";
const MIXIN_ASSIGNMENT_FACT_PARTITION: &str = "wow-recognizers.lua-mixin-assignment-facts";
const MIXIN_ASSIGNMENT_FACT_PROFILE: &str = "wow-recognizers-lua-mixin-assignment-facts-1";
const MIXIN_ASSIGNMENT_RULE: &str = "core.lua.mixin_assignment";
const MIXIN_CALLABLE: &str = "Mixin";
const MAX_CALLS: usize = 8192;
const MAX_MIXIN_ARGUMENTS: usize = 16;
const MAX_MIXIN_PAIRS: usize = 65_536;

pub struct SourceMixinInput<'a> {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMixinRelationMatch {
    pub declaration_proposal_id: String,
    pub relation_proposal_id: String,
    pub argument_ordinals: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceCreateFromMixinsMatch {
    pub call_id: String,
    pub instance_proposal_id: String,
    pub instantiates_proposal_id: String,
    pub mixins: Vec<SourceMixinRelationMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMixinRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matches: Vec<SourceCreateFromMixinsMatch>,
}
impl SourceMixinRecognition {
    pub fn matches(&self) -> &[SourceCreateFromMixinsMatch] {
        &self.matches
    }
}

pub struct SourceMixinProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceMixinRecognition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMixinAssignmentMatch {
    pub call_id: String,
    pub target_declaration_proposal_id: String,
    pub mixins: Vec<SourceMixinRelationMatch>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceSelfMixinSkip {
    pub call_id: String,
    pub declaration_proposal_id: String,
    pub argument_ordinals: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceMixinAssignmentRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matches: Vec<SourceMixinAssignmentMatch>,
    self_mixins_skipped: Vec<SourceSelfMixinSkip>,
}
impl SourceMixinAssignmentRecognition {
    pub fn matches(&self) -> &[SourceMixinAssignmentMatch] {
        &self.matches
    }

    pub fn self_mixins_skipped(&self) -> &[SourceSelfMixinSkip] {
        &self.self_mixins_skipped
    }
}

pub struct SourceMixinAssignmentProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceMixinAssignmentRecognition,
}

struct SourceBinding {
    node: GraphNodeId,
    handle: StableHandleId,
    evidence: EvidenceId,
}

struct PendingInstance {
    entity_proposal_id: String,
    confidence: GraphConfidence,
    coverage_ids: Vec<wow_core::CoverageId>,
}

pub fn recognize_source_mixins(
    input: SourceMixinInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceMixinProposals> {
    checkpoint(stop)?;
    if input.report.calls().len() > MAX_CALLS
        || input.function_proposals.len() != input.report.functions().len()
        || input.call_support.len() != input.report.calls().len()
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
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = source_partition.report().accepted_entities();

    let mut function_ids = BTreeSet::new();
    let mut proposal_nodes = BTreeMap::<String, GraphNodeId>::new();
    for function in input.report.functions() {
        checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let proposal = source_partition
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
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none()
            || proposal_nodes
                .insert(proposal_id.to_owned(), node.clone())
                .is_some()
            || !function_ids.insert(function.fact_id().to_owned())
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let mut declarations = BTreeMap::<(String, SourceSpan), SourceBinding>::new();
    let mut declaration_proposal_ids = BTreeMap::<(String, SourceSpan), String>::new();
    let mut declaration_ids = BTreeSet::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        if !declaration_ids.insert((*proposal_id).to_owned()) {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        checkpoint(stop)?;
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
                GraphProposalValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
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
        validate_support_without_digest(&input, *handle, *evidence, path, *span)?;
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none() {
            return Err(failure(RecognizerErrorCode::AdapterIdentityMismatch));
        }
        let key = ((*path).to_owned(), *span);
        if declarations
            .insert(
                key.clone(),
                SourceBinding {
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
        let caller_proposal = *input
            .function_proposals
            .get(call.caller_function_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
                RecognizerFactValue::Integer(
                    i64::try_from(call.arguments().len())
                        .map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))?,
                ),
            ),
            (
                "has_arguments".into(),
                RecognizerFactValue::Boolean(!call.arguments().is_empty()),
            ),
            (
                "colon_call".into(),
                RecognizerFactValue::Boolean(call.is_colon_call()),
            ),
        ]);
        if let Some(key) = call.resolved_callable_key() {
            fields.insert(
                "callable_key".into(),
                RecognizerFactValue::String(key.into()),
            );
        }

        let mut exact =
            !call.arguments().is_empty() && call.arguments().len() <= MAX_MIXIN_ARGUMENTS;
        for (ordinal, argument) in call
            .arguments()
            .iter()
            .take(MAX_MIXIN_ARGUMENTS)
            .enumerate()
        {
            let prefix = format!("argument_{ordinal}");
            let (Some(start), Some(end)) =
                (argument.span().byte_start(), argument.span().byte_end())
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            fields.insert(
                format!("{prefix}_span_start").into_boxed_str(),
                RecognizerFactValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            );
            fields.insert(
                format!("{prefix}_span_end").into_boxed_str(),
                RecognizerFactValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            );

            let kind = match (
                argument.reference_key(),
                argument.reference_target(),
                argument.literal(),
            ) {
                (Some(key), Some(target), None) if target.role == "main" => {
                    if target.workspace_id != input.report.main_snapshot_id() {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let declaration_key = (target.path.clone(), target.span);
                    let proposal_id = declaration_proposal_ids
                        .get(&declaration_key)
                        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                    let binding = declarations
                        .get(&declaration_key)
                        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
                    validate_target_binding(&input, target, binding)?;
                    fields.insert(
                        format!("{prefix}_key").into_boxed_str(),
                        RecognizerFactValue::String(key.into()),
                    );
                    fields.insert(
                        format!("{prefix}_declaration").into_boxed_str(),
                        RecognizerFactValue::Reference(proposal_id.clone().into_boxed_str()),
                    );
                    "main_reference"
                }
                (Some(key), Some(target), None) if target.role == "library" => {
                    if !input
                        .report
                        .library_snapshot_ids()
                        .contains(&target.workspace_id)
                    {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    fields.insert(
                        format!("{prefix}_key").into_boxed_str(),
                        RecognizerFactValue::String(key.into()),
                    );
                    exact = false;
                    "library_reference"
                }
                (None, None, Some(_)) => {
                    exact = false;
                    "literal"
                }
                (None, None, None) => {
                    exact = false;
                    "dynamic"
                }
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            };
            fields.insert(
                format!("{prefix}_kind").into_boxed_str(),
                RecognizerFactValue::String(kind.into()),
            );
        }

        facts.push(RecognizerFact::new(
            input.context.context_id(),
            RecognizerFactInput {
                kind: "lua_call".into(),
                partition_id: FACT_PARTITION.into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    call.caller_function_id(),
                )?,
                producer_id: "wow.emmy".into(),
                producer_version: FACT_PROFILE.into(),
                confidence: if exact {
                    GraphConfidence::Derived
                } else {
                    GraphConfidence::Possible
                },
                fields,
                source_handle_ids: vec![handle],
                evidence_ids: vec![evidence],
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
            partition_id: FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: FACT_PROFILE.into(),
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
        FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = create_from_mixins_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut instances = BTreeMap::<String, PendingInstance>::new();
    let mut instantiation_relations = BTreeMap::<String, String>::new();
    let mut instantiation_pending = Vec::new();
    let mut entities = Vec::new();
    let mut relations = Vec::new();

    for outcome in output.outcomes() {
        if outcome.rule_id() != CREATE_FROM_MIXINS_RULE || outcome.rule_version() != 1 {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
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
                    if entity_kind_id.as_ref() != "mixin_instance" || semantic_key.len() != 1 {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let Some(RecognizerFactValue::Reference(call_id)) = semantic_key.get("call")
                    else {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    };
                    let graph_id = proposal_id.to_string();
                    if instances
                        .insert(
                            call_id.to_string(),
                            PendingInstance {
                                entity_proposal_id: graph_id.clone(),
                                confidence: graph_confidence(*confidence),
                                coverage_ids: coverage_ids.clone(),
                            },
                        )
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    entities.push(
                        GraphEntityProposal::new(
                            graph_id,
                            "mixin_instance",
                            BTreeMap::from([(
                                "call".into(),
                                GraphProposalValue::Reference(call_id.clone()),
                            )]),
                            graph_confidence(*confidence),
                            source_handle_ids.clone(),
                            evidence_ids.clone(),
                            coverage_ids.clone(),
                        )
                        .map_err(graph_error)?,
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
                    instantiation_pending.push((
                        proposal_id.to_string(),
                        relation_kind_id.to_string(),
                        source.clone(),
                        target.clone(),
                        *confidence,
                        source_handle_ids.clone(),
                        evidence_ids.clone(),
                        coverage_ids.clone(),
                    ));
                }
            }
        }
    }

    for (
        graph_id,
        relation_kind_id,
        source,
        target,
        confidence,
        source_handle_ids,
        evidence_ids,
        coverage_ids,
    ) in instantiation_pending
    {
        if relation_kind_id != "lua_instantiates" {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let RecognizerFactValue::Reference(source_proposal) = source else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let RecognizerFactValue::Reference(call_id) = target else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let source_node = proposal_nodes
            .get(source_proposal.as_ref())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let target_proposal = instances
            .get(call_id.as_ref())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        if instantiation_relations
            .insert(call_id.to_string(), graph_id.clone())
            .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
        relations.push(
            GraphRelationProposal::new(
                graph_id,
                "lua_instantiates",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(source_node.clone()),
                    target: GraphProposalEndpoint::Proposed(
                        target_proposal.entity_proposal_id.clone().into(),
                    ),
                    confidence: graph_confidence(confidence),
                    source_handle_ids,
                    evidence_ids,
                    coverage_ids,
                },
            )
            .map_err(graph_error)?,
        );
    }

    if instances.len() != instantiation_relations.len()
        || instances
            .keys()
            .any(|call| !instantiation_relations.contains_key(call))
    {
        return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
    }

    let calls = input
        .report
        .calls()
        .iter()
        .map(|call| (call.fact_id(), call))
        .collect::<BTreeMap<_, _>>();
    let mut matches = Vec::new();
    for (call_id, instance) in instances {
        checkpoint(stop)?;
        let call = calls
            .get(call_id.as_str())
            .copied()
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let (call_handle, call_evidence) = *input
            .call_support
            .get(call_id.as_str())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;

        let mut target_ordinals = BTreeMap::<String, Vec<u32>>::new();
        for (ordinal, argument) in call
            .arguments()
            .iter()
            .take(MAX_MIXIN_ARGUMENTS)
            .enumerate()
        {
            let Some(target) = argument.reference_target() else {
                continue;
            };
            if target.role != "main" {
                continue;
            }
            let declaration_id = declaration_proposal_ids
                .get(&(target.path.clone(), target.span))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            target_ordinals
                .entry(declaration_id.clone())
                .or_default()
                .push(
                    u32::try_from(ordinal)
                        .map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))?,
                );
        }

        let mut mixins = Vec::new();
        for (declaration_proposal_id, argument_ordinals) in target_ordinals {
            let key = input
                .declaration_proposals
                .iter()
                .find_map(|(key, proposal)| {
                    (*proposal == declaration_proposal_id.as_str())
                        .then_some((key.0.to_owned(), key.1))
                })
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let binding = declarations
                .get(&key)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let relation_id = mixin_relation_id(&call_id, &declaration_proposal_id)?;
            let source_handle_ids = BTreeSet::from([call_handle, binding.handle])
                .into_iter()
                .collect();
            let evidence_ids = BTreeSet::from([call_evidence, binding.evidence])
                .into_iter()
                .collect();
            relations.push(
                GraphRelationProposal::new(
                    relation_id.as_str(),
                    "source_mixes_in",
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Proposed(
                            instance.entity_proposal_id.clone().into(),
                        ),
                        target: GraphProposalEndpoint::Existing(binding.node.clone()),
                        confidence: instance.confidence,
                        source_handle_ids,
                        evidence_ids,
                        coverage_ids: instance.coverage_ids.clone(),
                    },
                )
                .map_err(graph_error)?,
            );
            mixins.push(SourceMixinRelationMatch {
                declaration_proposal_id,
                relation_proposal_id: relation_id,
                argument_ordinals,
            });
        }
        matches.push(SourceCreateFromMixinsMatch {
            call_id: call_id.clone(),
            instance_proposal_id: instance.entity_proposal_id,
            instantiates_proposal_id: instantiation_relations[&call_id].clone(),
            mixins,
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
                GraphRelationKind::Instantiates => (
                    GraphCoverageState::Partial,
                    "lua_mixins.create_from_mixins_static_calls_only",
                ),
                GraphRelationKind::MixesIn => (
                    GraphCoverageState::Partial,
                    "lua_mixins.exact_main_declarations_only",
                ),
                _ => (
                    GraphCoverageState::NotEvaluated,
                    "lua_mixins.relation_owned_by_other_producer",
                ),
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
        SOURCE_MIXIN_PARTITION,
        entities,
        relations,
    )
    .map_err(graph_error)?;

    Ok(SourceMixinProposals {
        batch,
        coverage: graph_coverage,
        recognition: SourceMixinRecognition {
            profile: SOURCE_MIXIN_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            matches,
        },
    })
}

pub fn recognize_source_mixin_assignments(
    input: SourceMixinInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceMixinAssignmentProposals> {
    checkpoint(stop)?;
    if input.report.calls().len() > MAX_CALLS
        || input.call_support.len() != input.report.calls().len()
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
        return Err(failure(RecognizerErrorCode::AdapterBindingInvalid));
    }

    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = source_partition.report().accepted_entities();

    let mut declarations = BTreeMap::<String, SourceBinding>::new();
    let mut declaration_keys = BTreeMap::<(String, SourceSpan), String>::new();
    for ((path, span), proposal_id) in &input.declaration_proposals {
        checkpoint(stop)?;
        let proposal = source_partition
            .batch()
            .entity_proposal(proposal_id)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
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
                GraphProposalValue::Integer(
                    i64::try_from(start)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
            (
                "span_end".into(),
                GraphProposalValue::Integer(
                    i64::try_from(end)
                        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?,
                ),
            ),
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
        validate_support_without_digest(&input, *handle, *evidence, path, *span)?;
        let index = accepted
            .binary_search_by(|item| item.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none()
            || declarations
                .insert(
                    (*proposal_id).to_owned(),
                    SourceBinding {
                        node,
                        handle: *handle,
                        evidence: *evidence,
                    },
                )
                .is_some()
            || declaration_keys
                .insert(((*path).to_owned(), *span), (*proposal_id).to_owned())
                .is_some()
        {
            return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
        }
    }

    let fact_limits = RecognizerFactLimits::default();
    let mut facts = Vec::new();
    let mut pair_ordinals = BTreeMap::<(String, String, String), Vec<u32>>::new();
    let mut self_mixins = BTreeMap::<(String, String), Vec<u32>>::new();
    let mut pair_count = 0usize;

    for call in input.report.calls() {
        checkpoint(stop)?;
        if call.resolved_callable_key() != Some(MIXIN_CALLABLE) || call.is_colon_call() {
            continue;
        }
        let arguments = call.arguments();
        if arguments.len() < 2 {
            continue;
        }
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

        let Some(target_key) = arguments[0].reference_key() else {
            continue;
        };
        let Some(target) = arguments[0].reference_target() else {
            continue;
        };
        if target.role != "main" {
            continue;
        }
        if target.workspace_id != input.report.main_snapshot_id() {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let target_declaration = declaration_keys
            .get(&(target.path.clone(), target.span))
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
            .clone();
        let target_binding = declarations
            .get(&target_declaration)
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        validate_target_binding(&input, target, target_binding)?;

        let all_exact_main = arguments.len() <= MAX_MIXIN_ARGUMENTS
            && arguments.iter().all(|argument| {
                argument.literal().is_none()
                    && argument.reference_key().is_some()
                    && argument.reference_target().is_some_and(|target| {
                        target.role == "main"
                            && target.workspace_id == input.report.main_snapshot_id()
                            && declaration_keys.contains_key(&(target.path.clone(), target.span))
                    })
            });
        let confidence = if all_exact_main {
            GraphConfidence::Derived
        } else {
            GraphConfidence::Possible
        };

        let mut grouped = BTreeMap::<String, Vec<u32>>::new();
        for (ordinal, argument) in arguments
            .iter()
            .take(MAX_MIXIN_ARGUMENTS)
            .enumerate()
            .skip(1)
        {
            let Some(reference_key) = argument.reference_key() else {
                continue;
            };
            let Some(mixin) = argument.reference_target() else {
                continue;
            };
            if mixin.role != "main" {
                continue;
            }
            if mixin.workspace_id != input.report.main_snapshot_id() {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let mixin_declaration = declaration_keys
                .get(&(mixin.path.clone(), mixin.span))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?
                .clone();
            let mixin_binding = declarations
                .get(&mixin_declaration)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            validate_target_binding(&input, mixin, mixin_binding)?;
            if reference_key.is_empty() || target_key.is_empty() {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let ordinal =
                u32::try_from(ordinal).map_err(|_| failure(RecognizerErrorCode::BudgetExceeded))?;
            if mixin_declaration == target_declaration {
                self_mixins
                    .entry((call.fact_id().to_owned(), target_declaration.clone()))
                    .or_default()
                    .push(ordinal);
                continue;
            }
            grouped.entry(mixin_declaration).or_default().push(ordinal);
        }

        for (mixin_declaration, ordinals) in grouped {
            pair_count = pair_count
                .checked_add(1)
                .ok_or_else(|| failure(RecognizerErrorCode::BudgetExceeded))?;
            if pair_count > MAX_MIXIN_PAIRS {
                return Err(failure(RecognizerErrorCode::BudgetExceeded));
            }
            let fields = BTreeMap::from([
                (
                    "call_id".into(),
                    RecognizerFactValue::Reference(call.fact_id().into()),
                ),
                (
                    "callable_key".into(),
                    RecognizerFactValue::String(MIXIN_CALLABLE.into()),
                ),
                ("colon_call".into(), RecognizerFactValue::Boolean(false)),
                (
                    "target".into(),
                    RecognizerFactValue::Reference(target_declaration.clone().into_boxed_str()),
                ),
                (
                    "mixin".into(),
                    RecognizerFactValue::Reference(mixin_declaration.clone().into_boxed_str()),
                ),
                (
                    "first_ordinal".into(),
                    RecognizerFactValue::Integer(i64::from(ordinals[0])),
                ),
            ]);
            facts.push(RecognizerFact::new(
                input.context.context_id(),
                RecognizerFactInput {
                    kind: "lua_mixin_pair".into(),
                    partition_id: MIXIN_ASSIGNMENT_FACT_PARTITION.into(),
                    scope: RecognizerFactScope::new(
                        RecognizerFactScopeKind::Function,
                        call.caller_function_id(),
                    )?,
                    producer_id: "wow.emmy".into(),
                    producer_version: MIXIN_ASSIGNMENT_FACT_PROFILE.into(),
                    confidence,
                    fields,
                    source_handle_ids: vec![handle],
                    evidence_ids: vec![evidence],
                },
                fact_limits,
            )?);
            if pair_ordinals
                .insert(
                    (
                        call.fact_id().to_owned(),
                        target_declaration.clone(),
                        mixin_declaration,
                    ),
                    ordinals,
                )
                .is_some()
            {
                return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
            }
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
            partition_id: MIXIN_ASSIGNMENT_FACT_PARTITION.into(),
            capability_id: "emmy.fact.calls".into(),
            producer_id: "wow.emmy".into(),
            producer_version: MIXIN_ASSIGNMENT_FACT_PROFILE.into(),
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
        MIXIN_ASSIGNMENT_FACT_PARTITION,
        Vec::new(),
        facts,
        coverage,
        fact_limits,
    )?;
    let pack = mixin_assignment_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(input.context, &pack, &plan, &bundle, fact_limits, stop)?;

    let mut relations = Vec::new();
    let mut matches = BTreeMap::<(String, String), Vec<SourceMixinRelationMatch>>::new();
    for outcome in output.outcomes() {
        if outcome.rule_id() != MIXIN_ASSIGNMENT_RULE || outcome.rule_version() != 1 {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
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
            if relation_kind_id.as_ref() != "source_mixes_in" {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let [fact_id] = decisive_fact_ids.as_slice() else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let fact = bundle
                .fact_by_id(fact_id)
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let Some(RecognizerFactValue::Reference(call_id)) = fact.field("call_id") else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let Some(RecognizerFactValue::Reference(target_declaration)) = fact.field("target")
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let Some(RecognizerFactValue::Reference(mixin_declaration)) = fact.field("mixin")
            else {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            };
            let expected_source = RecognizerFactValue::Reference(target_declaration.clone());
            let expected_target = RecognizerFactValue::Reference(mixin_declaration.clone());
            if source != &expected_source || target != &expected_target {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let target_binding = declarations
                .get(target_declaration.as_ref())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            let mixin_binding = declarations
                .get(mixin_declaration.as_ref())
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
            if target_binding.node == mixin_binding.node {
                return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
            }
            let ordinals = pair_ordinals
                .remove(&(
                    call_id.to_string(),
                    target_declaration.to_string(),
                    mixin_declaration.to_string(),
                ))
                .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;

            let handles = source_handle_ids
                .iter()
                .copied()
                .chain([target_binding.handle, mixin_binding.handle])
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let evidence = evidence_ids
                .iter()
                .copied()
                .chain([target_binding.evidence, mixin_binding.evidence])
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            let graph_id = proposal_id.to_string();
            relations.push(
                GraphRelationProposal::new(
                    graph_id.as_str(),
                    "source_mixes_in",
                    GraphRelationProposalInput {
                        source: GraphProposalEndpoint::Existing(target_binding.node.clone()),
                        target: GraphProposalEndpoint::Existing(mixin_binding.node.clone()),
                        confidence: graph_confidence(*confidence),
                        source_handle_ids: handles,
                        evidence_ids: evidence,
                        coverage_ids: coverage_ids.clone(),
                    },
                )
                .map_err(graph_error)?,
            );
            matches
                .entry((call_id.to_string(), target_declaration.to_string()))
                .or_default()
                .push(SourceMixinRelationMatch {
                    declaration_proposal_id: mixin_declaration.to_string(),
                    relation_proposal_id: graph_id,
                    argument_ordinals: ordinals,
                });
        }
    }
    if !pair_ordinals.is_empty() {
        return Err(failure(RecognizerErrorCode::AdapterBindingMissing));
    }

    let matches = matches
        .into_iter()
        .map(|((call_id, target_declaration_proposal_id), mut mixins)| {
            mixins.sort_by(|left, right| {
                left.declaration_proposal_id
                    .cmp(&right.declaration_proposal_id)
            });
            SourceMixinAssignmentMatch {
                call_id,
                target_declaration_proposal_id,
                mixins,
            }
        })
        .collect();
    let self_mixins_skipped = self_mixins
        .into_iter()
        .map(
            |((call_id, declaration_proposal_id), argument_ordinals)| SourceSelfMixinSkip {
                call_id,
                declaration_proposal_id,
                argument_ordinals,
            },
        )
        .collect();

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
            let (state, blocker) = if relation == GraphRelationKind::MixesIn {
                (
                    GraphCoverageState::Partial,
                    "lua_mixin_assignments.exact_main_target_and_mixins_only",
                )
            } else {
                (
                    GraphCoverageState::NotEvaluated,
                    "lua_mixin_assignments.relation_owned_by_other_producer",
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
        SOURCE_MIXIN_ASSIGNMENT_PARTITION,
        Vec::new(),
        relations,
    )
    .map_err(graph_error)?;

    Ok(SourceMixinAssignmentProposals {
        batch,
        coverage: graph_coverage,
        recognition: SourceMixinAssignmentRecognition {
            profile: SOURCE_MIXIN_ASSIGNMENT_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            matches,
            self_mixins_skipped,
        },
    })
}

fn mixin_assignment_pack(
    registry_bundle_id: &str,
) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-mixin-assignments".into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: MIXIN_ASSIGNMENT_FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-mixin-assignment-1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 4,
                max_clauses_per_rule: 16,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 65_536,
                max_proposals_per_rule_partition: 65_536,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![RecognizerRule {
                rule_id: MIXIN_ASSIGNMENT_RULE.into(),
                version: 1,
                required_capabilities: vec!["emmy.fact.calls".into()],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "pair".into(),
                        kind: "lua_mixin_pair".into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "pair.callable_key".into(),
                        value: crate::RecognizerPackLiteral::String(MIXIN_CALLABLE.into()),
                    },
                    RecognizerClause::FieldEq {
                        field: "pair.colon_call".into(),
                        value: crate::RecognizerPackLiteral::Boolean(false),
                    },
                ],
                captures: Vec::new(),
                outputs: vec![RecognizerOutput::RelationAssertion {
                    output_id: "mixin_assignment_mixes_in".into(),
                    relation_kind_id: "source_mixes_in".into(),
                    source: "pair.target".into(),
                    target: "pair.mixin".into(),
                    confidence: RecognizerOutputConfidence::Derived,
                }],
                positive_fixture_ids: vec!["RECOG-STATE-002".into()],
                near_negative_fixture_ids: vec!["RECOG-XML-004".into()],
                partial_fixture_ids: vec!["RECOG-XML-005".into()],
                mutation_fixture_ids: vec!["RECOG-LIB-004".into()],
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

fn create_from_mixins_pack(
    registry_bundle_id: &str,
) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-mixins".into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-create-from-mixins-1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 32,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 100_000,
                max_matches_per_rule_partition: 10_000,
                max_proposals_per_rule_partition: 20_000,
                max_explanation_bytes: 1_048_576,
            },
            rules: vec![RecognizerRule {
                rule_id: CREATE_FROM_MIXINS_RULE.into(),
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
                            CREATE_FROM_MIXINS_CALLABLE.into(),
                        ),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.colon_call".into(),
                        value: crate::RecognizerPackLiteral::Boolean(false),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.has_arguments".into(),
                        value: crate::RecognizerPackLiteral::Boolean(true),
                    },
                ],
                captures: Vec::new(),
                outputs: vec![
                    RecognizerOutput::EntityAssertion {
                        output_id: "create_from_mixins_instance".into(),
                        entity_kind_id: "mixin_instance".into(),
                        semantic_key: BTreeMap::from([("call".into(), "call.call_id".into())]),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "create_from_mixins_instantiates".into(),
                        relation_kind_id: "lua_instantiates".into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                ],
                positive_fixture_ids: vec!["RECOG-TOC-008".into()],
                near_negative_fixture_ids: vec!["RECOG-XML-004".into()],
                partial_fixture_ids: vec!["RECOG-XML-005".into()],
                mutation_fixture_ids: vec!["RECOG-LIB-004".into()],
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

fn mixin_relation_id(call_id: &str, declaration_id: &str) -> RecognizerResult<String> {
    use sha2::{Digest, Sha256};

    let bytes = canonical_json_bytes(&(
        SOURCE_MIXIN_PROFILE,
        call_id,
        declaration_id,
        "source_mixes_in",
    ))
    .map_err(|_| failure(RecognizerErrorCode::AdapterIdentityMismatch))?;
    Ok(format!(
        "lua-mixin-edge:sha256:{}",
        encode_hex(&Sha256::digest(bytes))
    ))
}

fn encode_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn validate_target_binding(
    input: &SourceMixinInput<'_>,
    target: &SymbolTarget,
    binding: &SourceBinding,
) -> RecognizerResult<()> {
    if target.role != "main" || target.workspace_id != input.report.main_snapshot_id() {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    let digest = target
        .content_digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    let handle = input
        .source_handles
        .get(&binding.handle)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    if handle.path().as_str() != target.path
        || handle.span() != target.span
        || handle.content_digest() != &digest
    {
        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
    }
    Ok(())
}

fn validate_support(
    input: &SourceMixinInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    digest: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    let digest = digest
        .parse::<ContentDigest<SourceContent>>()
        .map_err(|_| failure(RecognizerErrorCode::AdapterFactMismatch))?;
    validate_support_record(input, handle_id, evidence_id, path, span, Some(digest))
}

fn validate_support_without_digest(
    input: &SourceMixinInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
) -> RecognizerResult<()> {
    validate_support_record(input, handle_id, evidence_id, path, span, None)
}

fn validate_support_record(
    input: &SourceMixinInput<'_>,
    handle_id: StableHandleId,
    evidence_id: EvidenceId,
    path: &str,
    span: SourceSpan,
    digest: Option<ContentDigest<SourceContent>>,
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
        "exact Lua mixin facts could not produce a coherent W11 partition",
    )
}

fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}
