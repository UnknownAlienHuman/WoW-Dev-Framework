//! W11 exact Lua construction facts -> declarative core recognizer -> graph proposals.
//! Source text is never reparsed here. All keys, arguments, spans and support come
//! from the generation-bound Emmy owner report.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::Serialize;
use wow_core::{
    ClaimScope, ContentDigest, EvidenceConfidence, EvidenceId, EvidenceRecord, GenerationContext,
    ProvenanceClass, SourceContent, SourceHandle, StableHandleId, canonical_json_bytes,
};
use wow_emmy::function_calls::{FunctionCallReport, SourceCallLiteral};
use wow_graph::{
    GraphConfidence, GraphCoverageRecord, GraphCoverageState, GraphNodeId, GraphPartitionSnapshot,
    GraphProposalBatch, GraphProposalEndpoint, GraphProposalValue, GraphRelationKind,
    GraphRelationProposal, GraphRelationProposalInput, GraphEntityProposal,
};

use crate::{
    RecognizerCaptureCardinality, RecognizerClause, RecognizerError, RecognizerErrorCode,
    RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage, RecognizerFactCoverageInput,
    RecognizerFactCoverageState, RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope,
    RecognizerFactScopeKind, RecognizerFactValue, RecognizerOutput, RecognizerOutputConfidence,
    RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackRollout,
    RecognizerPackTrustClass, RecognizerResult, RecognizerRule, compile_recognizer_plan,
    execute_recognizer_plan, parse_recognizer_pack,
};

pub const SOURCE_CONSTRUCTION_PARTITION: &str = "wow-recognizers.lua-construction";
pub const SOURCE_CONSTRUCTION_PROFILE: &str = "wow-recognizers/lua-construction/1";
const FACT_PARTITION: &str = "wow-recognizers.lua-construction-facts";
const FACT_PROFILE: &str = "wow-recognizers-lua-call-facts-1";
const CREATE_FRAME_RULE: &str = "core.lua.create_frame";
const CREATE_FRAME_CALLABLE: &str = "CreateFrame";
const MAX_CALLS: usize = 8192;
const MAX_ARGUMENTS_RETAINED: usize = 4;

pub struct SourceConstructionInput<'a> {
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
pub struct SourceConstructionRecognition {
    profile: &'static str,
    analyzer_report_id: String,
    fact_bundle_id: String,
    pack_digest: String,
    plan_id: String,
    output_partition_id: String,
    matched_create_frame_calls: Vec<String>,
}

pub struct SourceConstructionProposals {
    pub batch: GraphProposalBatch,
    pub coverage: Vec<GraphCoverageRecord>,
    pub recognition: SourceConstructionRecognition,
}

pub fn recognize_source_construction(
    input: SourceConstructionInput<'_>,
    stop: &AtomicBool,
) -> RecognizerResult<SourceConstructionProposals> {
    checkpoint(stop)?;
    if input.report.calls().len() > MAX_CALLS {
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

    let graph = input.owner.input_view(stop).map_err(graph_error)?;
    let source_partition = input
        .owner
        .partition(input.source_partition)
        .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
    let accepted = source_partition.report().accepted_entities();
    let mut caller_nodes = BTreeMap::<String, GraphNodeId>::new();
    let mut proposal_to_node = BTreeMap::<String, GraphNodeId>::new();
    for function in input.report.functions() {
        checkpoint(stop)?;
        let proposal_id = *input
            .function_proposals
            .get(function.fact_id())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let index = accepted
            .binary_search_by(|accepted| accepted.proposal_id().cmp(proposal_id))
            .map_err(|_| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let node = accepted[index].node().node_id().clone();
        if graph.node(&node).is_none()
            || proposal_to_node.insert(proposal_id.to_owned(), node.clone()).is_some()
            || caller_nodes.insert(function.fact_id().to_owned(), node).is_some()
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
        validate_support(&input, handle, evidence, call)?;
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

        let mut exact_arguments = call.arguments().len() <= MAX_ARGUMENTS_RETAINED;
        for (index, argument) in call.arguments().iter().take(MAX_ARGUMENTS_RETAINED).enumerate() {
            let prefix = format!("argument_{index}");
            let (Some(start), Some(end)) = (argument.span().byte_start(), argument.span().byte_end())
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
            let (kind, value) = match (argument.literal(), argument.reference_key()) {
                (Some(SourceCallLiteral::Nil), None) => ("nil", Some(RecognizerFactValue::Nil)),
                (Some(SourceCallLiteral::Boolean(value)), None) => {
                    ("boolean", Some(RecognizerFactValue::Boolean(*value)))
                }
                (Some(SourceCallLiteral::String(value)), None) => (
                    "string",
                    Some(RecognizerFactValue::String(value.clone().into_boxed_str())),
                ),
                (None, Some(reference)) => (
                    "reference",
                    Some(RecognizerFactValue::Reference(reference.into())),
                ),
                (None, None) => {
                    exact_arguments = false;
                    ("dynamic", None)
                }
                _ => return Err(failure(RecognizerErrorCode::AdapterFactMismatch)),
            };
            fields.insert(
                format!("{prefix}_kind").into_boxed_str(),
                RecognizerFactValue::Tag(kind.into()),
            );
            if let Some(value) = value {
                fields.insert(format!("{prefix}_value").into_boxed_str(), value);
            }
        }
        let fact = RecognizerFact::new(
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
                confidence: if exact_arguments {
                    GraphConfidence::Derived
                } else {
                    GraphConfidence::Possible
                },
                fields,
                source_handle_ids: vec![handle],
                evidence_ids: vec![evidence],
            },
            fact_limits,
        )?;
        facts.push(fact);
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
    let pack = create_frame_pack(input.owner.registry().bundle_id())?;
    let plan = compile_recognizer_plan(&pack)?;
    let output = execute_recognizer_plan(
        input.context,
        &pack,
        &plan,
        &bundle,
        fact_limits,
        stop,
    )?;

    let mut frame_by_call = BTreeMap::<String, String>::new();
    let mut entities = Vec::new();
    let mut relations_pending = Vec::new();
    let mut matched = BTreeSet::new();
    for outcome in output.outcomes() {
        if outcome.rule_id() != CREATE_FRAME_RULE || outcome.rule_version() != 1 {
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
                    if entity_kind_id.as_ref() != "frame" || semantic_key.len() != 1 {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    }
                    let Some(RecognizerFactValue::Reference(call_id)) = semantic_key.get("call")
                    else {
                        return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
                    };
                    let graph_id = proposal_id.to_string();
                    if frame_by_call
                        .insert(call_id.to_string(), graph_id.clone())
                        .is_some()
                    {
                        return Err(failure(RecognizerErrorCode::AdapterBindingDuplicate));
                    }
                    matched.insert(call_id.to_string());
                    entities.push(
                        GraphEntityProposal::new(
                            graph_id,
                            "frame",
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
                    relations_pending.push((
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
    let mut relations = Vec::new();
    for (id, kind, source, target, confidence, handles, evidence, coverage) in relations_pending {
        let RecognizerFactValue::Reference(source_proposal) = source else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        let RecognizerFactValue::Reference(call_id) = target else {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        };
        if kind != "lua_factory_creates" {
            return Err(failure(RecognizerErrorCode::AdapterFactMismatch));
        }
        let source_node = proposal_to_node
            .get(source_proposal.as_ref())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        let target_proposal = frame_by_call
            .get(call_id.as_ref())
            .ok_or_else(|| failure(RecognizerErrorCode::AdapterBindingMissing))?;
        relations.push(
            GraphRelationProposal::new(
                id,
                "lua_factory_creates",
                GraphRelationProposalInput {
                    source: GraphProposalEndpoint::Existing(source_node.clone()),
                    target: GraphProposalEndpoint::Proposed(target_proposal.clone().into()),
                    confidence: graph_confidence(confidence),
                    source_handle_ids: handles,
                    evidence_ids: evidence,
                    coverage_ids: coverage,
                },
            )
            .map_err(graph_error)?,
        );
    }

    let graph_coverage = input
        .owner
        .registry()
        .relation_kinds()
        .iter()
        .map(|definition| {
            let relation = definition.relation();
            let state = if relation == GraphRelationKind::FactoryCreates {
                GraphCoverageState::Partial
            } else {
                GraphCoverageState::NotEvaluated
            };
            GraphCoverageRecord::new(
                relation,
                state,
                false,
                vec![if relation == GraphRelationKind::FactoryCreates {
                    "lua_construction.create_frame_only".into()
                } else {
                    "lua_construction.relation_owned_by_other_producer".into()
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
        SOURCE_CONSTRUCTION_PARTITION,
        entities,
        relations,
    )
    .map_err(graph_error)?;

    Ok(SourceConstructionProposals {
        batch,
        coverage: graph_coverage,
        recognition: SourceConstructionRecognition {
            profile: SOURCE_CONSTRUCTION_PROFILE,
            analyzer_report_id: input.report.analysis_id().into(),
            fact_bundle_id: bundle.bundle_id().to_string(),
            pack_digest: pack.pack_digest().into(),
            plan_id: plan.plan_id().to_string(),
            output_partition_id: output.partition_id().to_string(),
            matched_create_frame_calls: matched.into_iter().collect(),
        },
    })
}

fn create_frame_pack(registry_bundle_id: &str) -> RecognizerResult<crate::CompiledRecognizerPack> {
    let document = RecognizerPackDocument {
        schema_version: crate::RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "wow-core-lua-construction".into(),
            version: "1".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: FACT_PROFILE.into(),
            graph_registry_bundle_id: registry_bundle_id.into(),
            evaluation_profile_id: "wow-recognizers-w11-create-frame-1".into(),
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
                rule_id: CREATE_FRAME_RULE.into(),
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
                        value: crate::RecognizerPackLiteral::String(CREATE_FRAME_CALLABLE.into()),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.colon_call".into(),
                        value: crate::RecognizerPackLiteral::Boolean(false),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.argument_0_kind".into(),
                        value: crate::RecognizerPackLiteral::String("string".into()),
                    },
                ],
                captures: Vec::new(),
                outputs: vec![
                    RecognizerOutput::EntityAssertion {
                        output_id: "create_frame_entity".into(),
                        entity_kind_id: "frame".into(),
                        semantic_key: BTreeMap::from([("call".into(), "call.call_id".into())]),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                    RecognizerOutput::RelationAssertion {
                        output_id: "create_frame_created_by".into(),
                        relation_kind_id: "lua_factory_creates".into(),
                        source: "call.caller".into(),
                        target: "call.call_id".into(),
                        confidence: RecognizerOutputConfidence::Derived,
                    },
                ],
                positive_fixture_ids: vec!["RECOG-FRAME-001".into()],
                near_negative_fixture_ids: vec!["RECOG-FRAME-002".into()],
                partial_fixture_ids: vec!["RECOG-FRAME-003".into()],
                mutation_fixture_ids: vec!["RECOG-FRAME-004".into()],
            }],
        },
    };
    let bytes = canonical_json_bytes(&document)
        .map_err(|_| failure(RecognizerErrorCode::PackIdentityMismatch))?;
    parse_recognizer_pack(&bytes)
}

fn graph_confidence(confidence: RecognizerOutputConfidence) -> GraphConfidence {
    match confidence {
        RecognizerOutputConfidence::Derived => GraphConfidence::Derived,
        RecognizerOutputConfidence::Possible => GraphConfidence::Possible,
    }
}

fn validate_support(
    input: &SourceConstructionInput<'_>,
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
        "exact Lua construction facts could not produce a coherent W11 partition",
    )
}

fn graph_error(error: wow_graph::GraphError) -> RecognizerError {
    failure(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => RecognizerErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => RecognizerErrorCode::BudgetExceeded,
        _ => RecognizerErrorCode::GraphProjectionFailed,
    })
}
