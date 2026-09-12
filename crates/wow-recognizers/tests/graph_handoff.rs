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
    GraphConfidence, GraphEntityKindDefinition, GraphGenerationId, GraphLimits,
    GraphRegistryBundle, GraphUniverseId, validate_graph_proposal_batch,
};
use wow_recognizers::{
    RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerCapture, RecognizerCaptureCardinality,
    RecognizerClause, RecognizerErrorCode, RecognizerFact, RecognizerFactBundle,
    RecognizerFactCoverage, RecognizerFactCoverageInput, RecognizerFactCoverageState,
    RecognizerFactInput, RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind,
    RecognizerFactValue, RecognizerOutput, RecognizerOutputConfidence, RecognizerPack,
    RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackLiteral,
    RecognizerPackRollout, RecognizerPackTrustClass, RecognizerRule,
    adapt_recognizer_output_to_graph, compile_recognizer_plan, execute_recognizer_plan,
    parse_recognizer_pack,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn context() -> TestResult<GenerationContext> {
    let schema = SchemaVersionEntry::new(
        SchemaId::from_str("schema:wow:recognizer-graph-fixture")?,
        ToolVersion::from_str("1.0.0")?,
    );
    let profile = ProfileIdentityBuilder::new(
        ProfileId::from_str("profile:fixture:recognizer-graph")?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:recognizer-graph-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([6_u8; 32]),
    )
    .client_version(ToolVersion::from_str("12.1.0")?)
    .schema_versions(vec![schema.clone()])
    .fixture_scope("recognizer graph handoff tests")
    .build()?;
    Ok(GenerationContextBuilder::new(
        profile,
        ReferenceGenerationId::derive(&"recognizer-graph-reference")?,
    )
    .project_generation(ProjectGenerationId::derive(&"recognizer-graph-project")?)
    .schema_versions(vec![schema])
    .producer_versions(vec![ProducerVersionEntry::new(
        ProducerId::from_str("wow.recognizers")?,
        ToolVersion::from_str("0.1.0")?,
    )])
    .build()?)
}

fn compiled() -> TestResult<(
    wow_recognizers::CompiledRecognizerPack,
    wow_recognizers::CompiledRecognizerPlan,
)> {
    let mut semantic_key = BTreeMap::new();
    semantic_key.insert("member".into(), "member".into());
    semantic_key.insert("receiver".into(), "call.receiver".into());
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
                max_rules: 4,
                max_clauses_per_rule: 16,
                max_clause_depth: 4,
                max_join_expansions_per_rule: 1_000,
                max_matches_per_rule_partition: 100,
                max_proposals_per_rule_partition: 100,
                max_explanation_bytes: 64 * 1024,
            },
            rules: vec![RecognizerRule {
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
                outputs: vec![RecognizerOutput::EntityAssertion {
                    output_id: "called_member".into(),
                    entity_kind_id: "lua.member".into(),
                    semantic_key,
                    confidence: RecognizerOutputConfidence::Derived,
                }],
                positive_fixture_ids: vec!["fixture.graph.positive".into()],
                near_negative_fixture_ids: vec!["fixture.graph.near-negative".into()],
                partial_fixture_ids: vec!["fixture.graph.partial".into()],
                mutation_fixture_ids: vec!["fixture.graph.mutation".into()],
            }],
        },
    };
    let pack = parse_recognizer_pack(&canonical_json_bytes(&document)?)?;
    let plan = compile_recognizer_plan(&pack)?;
    Ok((pack, plan))
}

fn bundle(
    context: &GenerationContext,
    state: RecognizerFactCoverageState,
) -> TestResult<RecognizerFactBundle> {
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
                    Box::<str>::from("receiver"),
                    RecognizerFactValue::Identifier("C_Test".into()),
                ),
                (
                    Box::<str>::from("member"),
                    RecognizerFactValue::Identifier("KnownApi".into()),
                ),
                (
                    Box::<str>::from("call_kind"),
                    RecognizerFactValue::String("direct".into()),
                ),
            ]),
            source_handle_ids: vec![StableHandleId::derive(&"recognizer-graph-handle")?],
            evidence_ids: vec![EvidenceId::derive(&"recognizer-graph-evidence")?],
        },
        RecognizerFactLimits::default(),
    )?;
    let coverage = RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: context.context_id(),
            partition_id: "project.main".into(),
            capability_id: "emmy.member_call.complete".into(),
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            state,
            blocker_ids: if state == RecognizerFactCoverageState::Complete {
                Vec::new()
            } else {
                vec!["fixture.incomplete".into()]
            },
        },
        RecognizerFactLimits::default(),
    )?;
    Ok(RecognizerFactBundle::build(
        context,
        "project.main",
        Vec::new(),
        vec![fact],
        vec![coverage],
        RecognizerFactLimits::default(),
    )?)
}

fn registry(bundle_id: &str) -> TestResult<GraphRegistryBundle> {
    Ok(GraphRegistryBundle::build(
        bundle_id,
        "1.0.0",
        vec![GraphEntityKindDefinition::new(
            "lua.member",
            vec!["project".into()],
            vec!["member".into(), "receiver".into()],
            vec![GraphConfidence::Derived, GraphConfidence::Possible],
        )?],
        Vec::new(),
    )?)
}

fn output(
    state: RecognizerFactCoverageState,
) -> TestResult<(
    GenerationContext,
    wow_recognizers::CompiledRecognizerPack,
    wow_recognizers::CompiledRecognizerPlan,
    wow_recognizers::RecognizerOutputPartition,
)> {
    let context = context()?;
    let (pack, plan) = compiled()?;
    let facts = bundle(&context, state)?;
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    Ok((context, pack, plan, output))
}

#[test]
fn complete_output_becomes_one_exact_graph_proposal_batch() -> TestResult {
    let (context, pack, plan, output) = output(RecognizerFactCoverageState::Complete)?;
    let registry = registry("graph-registry:e2-a:1")?;
    let batch = adapt_recognizer_output_to_graph(
        &output,
        &pack,
        &plan,
        &registry,
        GraphUniverseId::new("project:recognizer-graph")?,
        GraphGenerationId::new("project-generation:recognizer-graph:1")?,
    )?;
    assert_eq!(batch.source_context_id(), context.context_id());
    assert_eq!(batch.producer_partition_id(), output.producer_partition_id());
    let report = validate_graph_proposal_batch(
        &registry,
        None,
        &batch,
        GraphLimits::default(),
    )?;
    assert!(report.ready_for_publication());
    assert_eq!(report.accepted_entities().len(), 1);
    assert!(report.accepted_relations().is_empty());
    assert!(report.rejections().is_empty());
    Ok(())
}

#[test]
fn partial_output_never_becomes_a_replacement_batch() -> TestResult {
    let (_, pack, plan, output) = output(RecognizerFactCoverageState::Partial)?;
    let error = adapt_recognizer_output_to_graph(
        &output,
        &pack,
        &plan,
        &registry("graph-registry:e2-a:1")?,
        GraphUniverseId::new("project:recognizer-graph")?,
        GraphGenerationId::new("project-generation:recognizer-graph:1")?,
    )
    .err()
    .ok_or("partial output must fail")?;
    assert_eq!(error.code(), RecognizerErrorCode::GraphHandoffIncomplete);
    Ok(())
}

#[test]
fn registry_profile_mismatch_fails_before_graph_validation() -> TestResult {
    let (_, pack, plan, output) = output(RecognizerFactCoverageState::Complete)?;
    let error = adapt_recognizer_output_to_graph(
        &output,
        &pack,
        &plan,
        &registry("graph-registry:e2-a:other")?,
        GraphUniverseId::new("project:recognizer-graph")?,
        GraphGenerationId::new("project-generation:recognizer-graph:1")?,
    )
    .err()
    .ok_or("registry mismatch must fail")?;
    assert_eq!(error.code(), RecognizerErrorCode::GraphHandoffMismatch);
    Ok(())
}
