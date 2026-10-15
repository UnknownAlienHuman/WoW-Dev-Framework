use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;
use std::sync::atomic::AtomicBool;

use wow_core::{
    ContentDigest, EvidenceId, GenerationContext, GenerationContextBuilder, ProducerId,
    ProducerVersionEntry, ProfileId, ProfileIdentityBuilder, ProfileKind, ProjectGenerationId,
    ReferenceGenerationId, SchemaId, SchemaVersionEntry, SourceKind, SourceLogicalSnapshot,
    StableHandleId, ToolVersion, canonical_json_bytes,
};
use wow_graph::{
    GraphAssertionKind, GraphAssertionRecordScope, GraphAssertionRef, GraphConfidence,
    GraphCoverageState, GraphEntityKindDefinition, GraphGenerationId, GraphLimits,
    GraphLocalAssertion, GraphNode, GraphPartitionReplacement, GraphPartitionSession,
    GraphPartitionSnapshot, GraphRegistryBundle, GraphRelationKind, GraphRelationKindDefinition,
    GraphSnapshot, GraphUniverseId,
};
use wow_recognizers::{
    RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerCapture, RecognizerCaptureCardinality,
    RecognizerClause, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind, RecognizerFactValue,
    RecognizerGraphEndpointBinding, RecognizerGraphEndpointSide, RecognizerGraphHandoffDisposition,
    RecognizerGraphHandoffRequest, RecognizerOutput, RecognizerOutputConfidence,
    RecognizerOutputPartition, RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument,
    RecognizerPackLiteral, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerProposedAssertion, RecognizerRule, RecognizerRuleOutcomeState,
    adapt_recognizer_output_to_graph, execute_recognizer_plan,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn context() -> TestResult<GenerationContext> {
    let schema = SchemaVersionEntry::new(
        SchemaId::from_str("schema:wow:recognizer-graph-handoff")?,
        ToolVersion::from_str("1.0.0")?,
    );
    let profile = ProfileIdentityBuilder::new(
        ProfileId::from_str("profile:fixture:recognizer-graph-handoff")?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:recognizer-graph-handoff-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([31_u8; 32]),
    )
    .client_version(ToolVersion::from_str("12.1.0")?)
    .schema_versions(vec![schema.clone()])
    .fixture_scope("native recognizer graph handoff")
    .build()?;
    Ok(GenerationContextBuilder::new(
        profile,
        ReferenceGenerationId::derive(&"recognizer-graph-handoff-reference")?,
    )
    .project_generation(ProjectGenerationId::derive(
        &"recognizer-graph-handoff-project",
    )?)
    .schema_versions(vec![schema])
    .producer_versions(vec![ProducerVersionEntry::new(
        ProducerId::from_str("wow.recognizers")?,
        ToolVersion::from_str("0.1.0")?,
    )])
    .build()?)
}

fn pack() -> TestResult<(
    wow_recognizers::CompiledRecognizerPack,
    wow_recognizers::CompiledRecognizerPlan,
)> {
    let mut semantic_key = BTreeMap::new();
    semantic_key.insert("member".into(), "member".into());
    semantic_key.insert("receiver".into(), "call.receiver".into());
    let rule = RecognizerRule {
        rule_id: "core.direct-call".into(),
        version: 1,
        required_capabilities: vec!["emmy.member_call.complete".into()],
        scope: "function".into(),
        clauses: vec![
            RecognizerClause::Fact {
                alias: "call".into(),
                kind: "emmy.member_call".into(),
            },
            RecognizerClause::FieldEq {
                field: "call.call_kind".into(),
                value: RecognizerPackLiteral::String("direct".into()),
            },
        ],
        captures: vec![RecognizerCapture {
            name: "member".into(),
            value_type: "identifier".into(),
            source: "call.member".into(),
            cardinality: RecognizerCaptureCardinality::One,
        }],
        outputs: vec![
            RecognizerOutput::EntityAssertion {
                output_id: "called_member".into(),
                entity_kind_id: "lua.member".into(),
                semantic_key,
                confidence: RecognizerOutputConfidence::Derived,
            },
            RecognizerOutput::RelationAssertion {
                output_id: "defines_member".into(),
                relation_kind_id: "defines_member".into(),
                source: "call.receiver".into(),
                target: "call.member".into(),
                confidence: RecognizerOutputConfidence::Derived,
            },
        ],
        positive_fixture_ids: vec!["fixture.graph-handoff.positive".into()],
        near_negative_fixture_ids: vec!["fixture.graph-handoff.near-negative".into()],
        partial_fixture_ids: vec!["fixture.graph-handoff.partial".into()],
        mutation_fixture_ids: vec!["fixture.graph-handoff.mutation".into()],
    };
    let document = RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "core.graph-handoff".into(),
            version: "1.0.0".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: "recognizer-facts:e2-b:1".into(),
            graph_registry_bundle_id: "graph-registry:e2-a:1".into(),
            evaluation_profile_id: "recognizer-evaluation:e2-b:1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 64,
                max_clause_depth: 8,
                max_join_expansions_per_rule: 10_000,
                max_matches_per_rule_partition: 100,
                max_proposals_per_rule_partition: 100,
                max_explanation_bytes: 256 * 1024,
            },
            rules: vec![rule],
        },
    };
    let compiled = wow_recognizers::parse_recognizer_pack(&canonical_json_bytes(&document)?)?;
    let plan = wow_recognizers::compile_recognizer_plan(&compiled)?;
    Ok((compiled, plan))
}

fn facts(
    context: &GenerationContext,
    coverage_state: RecognizerFactCoverageState,
) -> TestResult<RecognizerFactBundle> {
    let limits = RecognizerFactLimits::default();
    let fact = RecognizerFact::new(
        context.context_id(),
        RecognizerFactInput {
            kind: "emmy.member_call".into(),
            partition_id: "project.main".into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                "main.lua::function:1",
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            confidence: GraphConfidence::Derived,
            fields: BTreeMap::from([
                (
                    Box::<str>::from("call_kind"),
                    RecognizerFactValue::String("direct".into()),
                ),
                (
                    Box::<str>::from("member"),
                    RecognizerFactValue::Identifier("KnownApi".into()),
                ),
                (
                    Box::<str>::from("receiver"),
                    RecognizerFactValue::Identifier("C_Test".into()),
                ),
            ]),
            source_handle_ids: vec![StableHandleId::derive(&"graph-handoff-source")?],
            evidence_ids: vec![EvidenceId::derive(&"graph-handoff-evidence")?],
        },
        limits,
    )?;
    let coverage = RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: context.context_id(),
            partition_id: "project.main".into(),
            capability_id: "emmy.member_call.complete".into(),
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            state: coverage_state,
            blocker_ids: if coverage_state == RecognizerFactCoverageState::Complete {
                Vec::new()
            } else {
                vec!["fixture.coverage.incomplete".into()]
            },
        },
        limits,
    )?;
    Ok(RecognizerFactBundle::build(
        context,
        "project.main",
        Vec::new(),
        vec![fact],
        vec![coverage],
        limits,
    )?)
}

struct GraphFixture {
    owner: GraphPartitionSnapshot,
    caller: GraphNode,
    callee: GraphNode,
}

fn graph_fixture(context: &GenerationContext) -> TestResult<GraphFixture> {
    let limits = GraphLimits::default();
    let universe = GraphUniverseId::new("project:recognizer-handoff")?;
    let generation = GraphGenerationId::new("project-generation:recognizer-handoff:1")?;
    let caller = GraphNode::new(
        universe.clone(),
        generation.clone(),
        "function",
        "function:caller",
        vec!["fixture:caller-evidence".into()],
        limits,
    )?;
    let callee = GraphNode::new(
        universe.clone(),
        generation.clone(),
        "function",
        "function:known-api",
        vec!["fixture:callee-evidence".into()],
        limits,
    )?;
    let registry = GraphRegistryBundle::build(
        "graph-registry:e2-a:1",
        "1.0.0",
        vec![
            GraphEntityKindDefinition::new(
                "function",
                vec!["project".into()],
                vec!["symbol".into()],
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )?,
            GraphEntityKindDefinition::new(
                "lua.member",
                vec!["project".into()],
                vec!["member".into(), "receiver".into()],
                vec![GraphConfidence::Derived, GraphConfidence::Possible],
            )?,
        ],
        vec![GraphRelationKindDefinition::new(
            "defines_member",
            GraphRelationKind::Defines,
            vec!["function".into()],
            vec!["lua.member".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
    )?;
    let foundation = GraphSnapshot::build(
        universe,
        generation,
        limits,
        vec![caller.clone(), callee.clone()],
        Vec::new(),
        Vec::new(),
    )?;
    let owner = GraphPartitionSnapshot::new(
        registry,
        foundation,
        context.context_id(),
        &AtomicBool::new(false),
    )?;
    Ok(GraphFixture {
        owner,
        caller,
        callee,
    })
}

fn binding_inputs(
    graph: &GraphFixture,
    output: &RecognizerOutputPartition,
) -> TestResult<Vec<RecognizerGraphEndpointBinding>> {
    let proposals = output
        .outcomes()
        .first()
        .ok_or("matcher output outcome missing")?
        .proposals();
    let (entity_proposal_id, entity_match_id) = proposals
        .iter()
        .find_map(|proposal| match proposal {
            RecognizerProposedAssertion::Entity {
                proposal_id,
                match_id,
                ..
            } => Some((proposal_id, match_id)),
            RecognizerProposedAssertion::Relation { .. } => None,
        })
        .ok_or("matcher entity proposal missing")?;
    let (relation_kind_id, target, relation_match_id) = proposals
        .iter()
        .find_map(|proposal| match proposal {
            RecognizerProposedAssertion::Relation {
                relation_kind_id,
                target,
                match_id,
                ..
            } => Some((relation_kind_id, target, match_id)),
            RecognizerProposedAssertion::Entity { .. } => None,
        })
        .ok_or("matcher relation proposal missing")?;
    if entity_match_id != relation_match_id {
        return Err("entity and relation proposals have different match identities".into());
    }
    Ok(vec![
        RecognizerGraphEndpointBinding::existing(
            relation_kind_id.clone(),
            RecognizerGraphEndpointSide::Source,
            RecognizerFactValue::Identifier("C_Test".into()),
            graph.caller.node_id().clone(),
        )?,
        RecognizerGraphEndpointBinding::proposed(
            relation_kind_id.clone(),
            RecognizerGraphEndpointSide::Target,
            target.clone(),
            relation_match_id.clone(),
            entity_proposal_id.clone(),
        )?,
    ])
}

fn proposal_reference(
    owner: &GraphPartitionSnapshot,
    partition_id: &str,
    kind: GraphAssertionKind,
    proposal_id: &str,
) -> TestResult<GraphAssertionRef> {
    Ok(GraphAssertionRef::Producer {
        partition_id: partition_id.into(),
        batch_id: owner
            .partition(partition_id)
            .ok_or("recognizer producer partition missing")?
            .batch()
            .batch_id()
            .into(),
        assertion: GraphLocalAssertion {
            kind,
            proposal_id: proposal_id.into(),
        },
    })
}

#[test]
#[allow(clippy::too_many_lines)]
fn exact_matcher_output_validates_replaces_and_reads_back_from_graph_owner() -> TestResult {
    let context = context()?;
    let (pack, plan) = pack()?;
    let facts = facts(&context, RecognizerFactCoverageState::Complete)?;
    let graph = graph_fixture(&context)?;
    let cancelled = AtomicBool::new(false);
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &cancelled,
    )?;
    let endpoint_bindings = binding_inputs(&graph, &output)?;
    let handoff = adapt_recognizer_output_to_graph(RecognizerGraphHandoffRequest {
        context: &context,
        pack: &pack,
        plan: &plan,
        facts: &facts,
        output: &output,
        fact_limits: RecognizerFactLimits::default(),
        graph: &graph.owner,
        graph_limits: GraphLimits::default(),
        endpoint_bindings: &endpoint_bindings,
        cancelled: &cancelled,
    })?;

    assert_eq!(
        handoff.disposition(),
        RecognizerGraphHandoffDisposition::Ready
    );
    assert!(handoff.publication_ready());
    assert_eq!(handoff.output().outcomes().len(), 1);
    assert_eq!(
        handoff.output().outcomes()[0].state(),
        RecognizerRuleOutcomeState::Matched
    );
    let entity_proposal_id = handoff.output().outcomes()[0]
        .proposals()
        .iter()
        .find_map(|proposal| match proposal {
            RecognizerProposedAssertion::Entity { proposal_id, .. } => Some(proposal_id.as_str()),
            RecognizerProposedAssertion::Relation { .. } => None,
        })
        .ok_or("matcher entity proposal missing")?;
    let relation_proposal_id = handoff.output().outcomes()[0]
        .proposals()
        .iter()
        .find_map(|proposal| match proposal {
            RecognizerProposedAssertion::Relation { proposal_id, .. } => Some(proposal_id.as_str()),
            RecognizerProposedAssertion::Entity { .. } => None,
        })
        .ok_or("matcher relation proposal missing")?;
    let batch = handoff.batch().ok_or("ready handoff has no graph batch")?;
    let batch_json = serde_json::to_value(batch)?;
    assert_eq!(batch.entity_proposals().len(), 1);
    assert_eq!(
        batch_json["relation_proposals"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        batch.entity_proposals()[0].proposal_id(),
        entity_proposal_id
    );
    assert_eq!(
        batch_json["entity_proposals"][0]["source_handle_ids"],
        batch_json["relation_proposals"][0]["source_handle_ids"]
    );
    assert_eq!(
        batch_json["entity_proposals"][0]["evidence_ids"],
        batch_json["relation_proposals"][0]["evidence_ids"]
    );
    assert_eq!(
        batch_json["entity_proposals"][0]["coverage_ids"],
        batch_json["relation_proposals"][0]["coverage_ids"]
    );
    assert_eq!(
        batch
            .entity_proposal(entity_proposal_id)
            .ok_or("graph entity proposal missing")?
            .confidence(),
        GraphConfidence::Derived
    );
    assert_eq!(
        batch
            .relation_proposal(relation_proposal_id)
            .ok_or("graph relation proposal missing")?
            .confidence(),
        GraphConfidence::Derived
    );
    let graph_coverage = handoff
        .coverage_records()
        .ok_or("ready handoff has no graph coverage")?;
    assert_eq!(graph_coverage.len(), 1);
    assert_eq!(graph_coverage[0].relation(), GraphRelationKind::Defines);
    assert_eq!(graph_coverage[0].state(), GraphCoverageState::Complete);
    assert!(!graph_coverage[0].negative_authority());

    let mut session = GraphPartitionSession::new(graph.owner.clone(), &cancelled)?;
    let before = session.view();
    let producer_partition_id = handoff.producer_partition_id();
    let request = GraphPartitionReplacement {
        expected_snapshot_id: before.snapshot().snapshot_id().clone(),
        expected_partition_digest: before
            .partition(producer_partition_id)
            .map(|partition| partition.partition_digest().into()),
        producer_version: handoff.producer_version().into(),
        batch: batch.clone(),
        coverage: graph_coverage.to_vec(),
    };
    let plan = before.prepare_replacement(request, &cancelled)?;
    let after = session.publish(plan, &cancelled)?;

    let lookup = after.producer_lookup(&cancelled)?;
    let scope = GraphAssertionRecordScope {
        universe: after.foundation().universe().clone(),
        generation: after.foundation().generation().clone(),
        source_context_id: after.source_context_id(),
    };
    let entity_proposal_id = batch.entity_proposals()[0].proposal_id();
    let entity_ref = proposal_reference(
        &after,
        producer_partition_id,
        GraphAssertionKind::Entity,
        entity_proposal_id,
    )?;
    let entity = lookup.entity(&scope, &entity_ref, &cancelled)?;
    assert_eq!(entity.proposal().proposal_id(), entity_proposal_id);
    assert_eq!(entity.accepted().node().kind(), "lua.member");
    let published = after.open_read_view(&cancelled)?.published_entity(
        producer_partition_id,
        entity_proposal_id,
        &cancelled,
    )?;
    assert_eq!(published.kind(), "lua.member");
    assert_eq!(published.owner_key(), entity.accepted().node().owner_key());

    let relation_ref = proposal_reference(
        &after,
        producer_partition_id,
        GraphAssertionKind::Relation,
        relation_proposal_id,
    )?;
    let relation = lookup.relation(&scope, &relation_ref, &cancelled)?;
    assert_eq!(relation.proposal().proposal_id(), relation_proposal_id);
    assert_eq!(
        relation.accepted().edge().relation(),
        GraphRelationKind::Defines
    );
    assert_eq!(relation.accepted().edge().from(), graph.caller.node_id());
    assert_eq!(
        relation.accepted().edge().to(),
        entity.accepted().node().node_id()
    );
    assert_ne!(relation.accepted().edge().to(), graph.callee.node_id());
    assert!(after.snapshot().edges().iter().any(|edge| {
        edge.relation() == GraphRelationKind::Defines
            && edge.confidence() == GraphConfidence::Derived
    }));
    Ok(())
}

#[test]
fn incomplete_and_cancelled_outputs_remain_attached_without_complete_batches() -> TestResult {
    let context = context()?;
    let (pack, plan) = pack()?;
    let graph = graph_fixture(&context)?;

    let partial_facts = facts(&context, RecognizerFactCoverageState::Partial)?;
    let not_cancelled = AtomicBool::new(false);
    let partial_output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &partial_facts,
        RecognizerFactLimits::default(),
        &not_cancelled,
    )?;
    let partial_bindings = binding_inputs(&graph, &partial_output)?;
    let partial_cancelled = AtomicBool::new(false);
    let partial = adapt_recognizer_output_to_graph(RecognizerGraphHandoffRequest {
        context: &context,
        pack: &pack,
        plan: &plan,
        facts: &partial_facts,
        output: &partial_output,
        fact_limits: RecognizerFactLimits::default(),
        graph: &graph.owner,
        graph_limits: GraphLimits::default(),
        endpoint_bindings: &partial_bindings,
        cancelled: &partial_cancelled,
    })?;
    assert_eq!(
        partial.disposition(),
        RecognizerGraphHandoffDisposition::Deferred
    );
    assert!(partial.batch().is_none());
    assert_eq!(
        partial.output().outcomes()[0].state(),
        RecognizerRuleOutcomeState::Partial
    );
    assert!(!partial.output().outcomes()[0].blocker_ids().is_empty());
    assert!(!partial.output().outcomes()[0].proposals().is_empty());

    let complete_facts = facts(&context, RecognizerFactCoverageState::Complete)?;
    let cancelled = AtomicBool::new(true);
    let cancelled_output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &complete_facts,
        RecognizerFactLimits::default(),
        &cancelled,
    )?;
    let stopped = adapt_recognizer_output_to_graph(RecognizerGraphHandoffRequest {
        context: &context,
        pack: &pack,
        plan: &plan,
        facts: &complete_facts,
        output: &cancelled_output,
        fact_limits: RecognizerFactLimits::default(),
        graph: &graph.owner,
        graph_limits: GraphLimits::default(),
        endpoint_bindings: &[],
        cancelled: &cancelled,
    })?;
    assert_eq!(
        stopped.disposition(),
        RecognizerGraphHandoffDisposition::Cancelled
    );
    assert!(stopped.batch().is_none());
    assert_eq!(
        stopped.output().outcomes()[0].state(),
        RecognizerRuleOutcomeState::Cancelled
    );
    assert!(!stopped.output().outcomes()[0].blocker_ids().is_empty());
    Ok(())
}
