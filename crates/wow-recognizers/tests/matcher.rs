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
use wow_graph::GraphConfidence;
use wow_recognizers::{
    RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerCapture, RecognizerCaptureCardinality,
    RecognizerClause, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind, RecognizerFactValue,
    RecognizerOutput, RecognizerOutputConfidence, RecognizerPack, RecognizerPackBudgets,
    RecognizerPackDocument, RecognizerPackLiteral, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerRule, RecognizerRuleOutcomeState, compile_recognizer_plan, execute_recognizer_plan,
    parse_recognizer_pack,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn context() -> TestResult<GenerationContext> {
    let schema = SchemaVersionEntry::new(
        SchemaId::from_str("schema:wow:recognizer-matcher-fixture")?,
        ToolVersion::from_str("1.0.0")?,
    );
    let profile = ProfileIdentityBuilder::new(
        ProfileId::from_str("profile:fixture:recognizer-matcher")?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:recognizer-matcher-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([5_u8; 32]),
    )
    .client_version(ToolVersion::from_str("12.1.0")?)
    .schema_versions(vec![schema.clone()])
    .fixture_scope("recognizer matcher tests")
    .build()?;
    Ok(GenerationContextBuilder::new(
        profile,
        ReferenceGenerationId::derive(&"recognizer-matcher-reference")?,
    )
    .project_generation(ProjectGenerationId::derive(&"recognizer-matcher-project")?)
    .schema_versions(vec![schema])
    .producer_versions(vec![ProducerVersionEntry::new(
        ProducerId::from_str("wow.recognizers")?,
        ToolVersion::from_str("0.1.0")?,
    )])
    .build()?)
}

fn budgets(max_matches: u32, max_proposals: u32) -> RecognizerPackBudgets {
    RecognizerPackBudgets {
        max_rules: 8,
        max_clauses_per_rule: 64,
        max_clause_depth: 8,
        max_join_expansions_per_rule: 10_000,
        max_matches_per_rule_partition: max_matches,
        max_proposals_per_rule_partition: max_proposals,
        max_explanation_bytes: 256 * 1024,
    }
}

fn direct_rule(extra: Vec<RecognizerClause>) -> RecognizerRule {
    let mut clauses = vec![
        RecognizerClause::Fact {
            alias: "call".into(),
            kind: "emmy.member_call".into(),
        },
        RecognizerClause::FieldEq {
            field: "call.call_kind".into(),
            value: RecognizerPackLiteral::String("direct".into()),
        },
    ];
    clauses.extend(extra);
    let mut semantic_key = BTreeMap::new();
    semantic_key.insert("member".into(), "member".into());
    semantic_key.insert("receiver".into(), "call.receiver".into());
    RecognizerRule {
        rule_id: "core.direct-call".into(),
        version: 1,
        required_capabilities: vec!["emmy.member_call.complete".into()],
        scope: "function".into(),
        clauses,
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
        positive_fixture_ids: vec!["fixture.matcher.positive".into()],
        near_negative_fixture_ids: vec!["fixture.matcher.near-negative".into()],
        partial_fixture_ids: vec!["fixture.matcher.partial".into()],
        mutation_fixture_ids: vec!["fixture.matcher.mutation".into()],
    }
}

fn compiled(
    rule: RecognizerRule,
    max_matches: u32,
    max_proposals: u32,
) -> TestResult<(
    wow_recognizers::CompiledRecognizerPack,
    wow_recognizers::CompiledRecognizerPlan,
)> {
    let document = RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "core.matcher".into(),
            version: "1.0.0".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: "recognizer-facts:e2-b:1".into(),
            graph_registry_bundle_id: "graph-registry:e2-a:1".into(),
            evaluation_profile_id: "recognizer-evaluation:e2-b:1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: budgets(max_matches, max_proposals),
            rules: vec![rule],
        },
    };
    let pack = parse_recognizer_pack(&canonical_json_bytes(&document)?)?;
    let plan = compile_recognizer_plan(&pack)?;
    Ok((pack, plan))
}

fn fields(member: &str, call_kind: &str, ordinal: i64) -> BTreeMap<Box<str>, RecognizerFactValue> {
    BTreeMap::from([
        (
            Box::<str>::from("receiver"),
            RecognizerFactValue::Identifier("C_Test".into()),
        ),
        (
            Box::<str>::from("member"),
            RecognizerFactValue::Identifier(member.into()),
        ),
        (
            Box::<str>::from("call_kind"),
            RecognizerFactValue::String(call_kind.into()),
        ),
        (
            Box::<str>::from("ordinal"),
            RecognizerFactValue::Integer(ordinal),
        ),
    ])
}

fn fact(
    context: &GenerationContext,
    kind: &str,
    member: &str,
    call_kind: &str,
    ordinal: i64,
    confidence: GraphConfidence,
) -> TestResult<RecognizerFact> {
    Ok(RecognizerFact::new(
        context.context_id(),
        RecognizerFactInput {
            kind: kind.into(),
            partition_id: "project.main".into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                "main.lua::function:1",
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            confidence,
            fields: fields(member, call_kind, ordinal),
            source_handle_ids: vec![StableHandleId::derive(&(kind, member, ordinal))?],
            evidence_ids: vec![EvidenceId::derive(&(kind, member, ordinal))?],
        },
        RecognizerFactLimits::default(),
    )?)
}

fn coverage(
    context: &GenerationContext,
    capability: &str,
    state: RecognizerFactCoverageState,
) -> TestResult<RecognizerFactCoverage> {
    Ok(RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: context.context_id(),
            partition_id: "project.main".into(),
            capability_id: capability.into(),
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
    )?)
}

fn bundle(
    context: &GenerationContext,
    facts: Vec<RecognizerFact>,
    coverage: Vec<RecognizerFactCoverage>,
) -> TestResult<RecognizerFactBundle> {
    Ok(RecognizerFactBundle::build(
        context,
        "project.main",
        Vec::new(),
        facts,
        coverage,
        RecognizerFactLimits::default(),
    )?)
}

#[test]
fn exact_selector_predicate_capture_and_output_match() -> TestResult {
    let context = context()?;
    let (pack, plan) = compiled(direct_rule(Vec::new()), 100, 100)?;
    let facts = bundle(
        &context,
        vec![fact(
            &context,
            "emmy.member_call",
            "KnownApi",
            "direct",
            1,
            GraphConfidence::Derived,
        )?],
        vec![coverage(
            &context,
            "emmy.member_call.complete",
            RecognizerFactCoverageState::Complete,
        )?],
    )?;
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(output.outcomes().len(), 1);
    let outcome = &output.outcomes()[0];
    assert_eq!(outcome.state(), RecognizerRuleOutcomeState::Matched);
    assert_eq!(outcome.matches().len(), 1);
    assert_eq!(outcome.proposals().len(), 1);
    assert_eq!(
        outcome.matches()[0].confidence(),
        RecognizerOutputConfidence::Derived
    );
    assert_eq!(
        outcome.proposals()[0].confidence(),
        RecognizerOutputConfidence::Derived
    );
    output.validate()?;
    Ok(())
}

#[test]
fn complete_no_match_is_distinct_from_partial_no_match() -> TestResult {
    let context = context()?;
    let (pack, plan) = compiled(direct_rule(Vec::new()), 100, 100)?;
    let complete = bundle(
        &context,
        Vec::new(),
        vec![coverage(
            &context,
            "emmy.member_call.complete",
            RecognizerFactCoverageState::Complete,
        )?],
    )?;
    let complete_output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &complete,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        complete_output.outcomes()[0].state(),
        RecognizerRuleOutcomeState::EvaluatedNoMatch
    );

    let partial = bundle(
        &context,
        Vec::new(),
        vec![coverage(
            &context,
            "emmy.member_call.complete",
            RecognizerFactCoverageState::Partial,
        )?],
    )?;
    let partial_output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &partial,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        partial_output.outcomes()[0].state(),
        RecognizerRuleOutcomeState::Partial
    );
    assert!(!partial_output.outcomes()[0].blocker_ids().is_empty());
    Ok(())
}

#[test]
fn possible_input_and_ambiguity_cannot_produce_derived_output() -> TestResult {
    let context = context()?;
    let (pack, plan) = compiled(direct_rule(Vec::new()), 100, 100)?;
    let facts = bundle(
        &context,
        vec![
            fact(
                &context,
                "emmy.member_call",
                "First",
                "direct",
                1,
                GraphConfidence::Derived,
            )?,
            fact(
                &context,
                "emmy.member_call",
                "Second",
                "direct",
                2,
                GraphConfidence::Possible,
            )?,
        ],
        vec![coverage(
            &context,
            "emmy.member_call.complete",
            RecognizerFactCoverageState::Complete,
        )?],
    )?;
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(output.outcomes()[0].matches().len(), 2);
    assert!(
        output.outcomes()[0]
            .proposals()
            .iter()
            .all(|proposal| proposal.confidence() == RecognizerOutputConfidence::Possible)
    );
    Ok(())
}

#[test]
fn negative_clause_requires_complete_declared_coverage() -> TestResult {
    let context = context()?;
    let mut rule = direct_rule(vec![RecognizerClause::NotExists {
        clauses: vec![
            RecognizerClause::Fact {
                alias: "removed".into(),
                kind: "reference.api".into(),
            },
            RecognizerClause::FieldEq {
                field: "removed.member".into(),
                value: RecognizerPackLiteral::String("KnownApi".into()),
            },
        ],
        required_complete_capability: "reference.api.complete".into(),
    }]);
    rule.required_capabilities
        .push("reference.api.complete".into());
    rule.required_capabilities.sort();
    let (pack, plan) = compiled(rule, 100, 100)?;
    let call = fact(
        &context,
        "emmy.member_call",
        "KnownApi",
        "direct",
        1,
        GraphConfidence::Derived,
    )?;
    let complete = bundle(
        &context,
        vec![call.clone()],
        vec![
            coverage(
                &context,
                "emmy.member_call.complete",
                RecognizerFactCoverageState::Complete,
            )?,
            coverage(
                &context,
                "reference.api.complete",
                RecognizerFactCoverageState::Complete,
            )?,
        ],
    )?;
    let matched = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &complete,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        matched.outcomes()[0].state(),
        RecognizerRuleOutcomeState::Matched
    );

    let partial = bundle(
        &context,
        vec![call],
        vec![
            coverage(
                &context,
                "emmy.member_call.complete",
                RecognizerFactCoverageState::Complete,
            )?,
            coverage(
                &context,
                "reference.api.complete",
                RecognizerFactCoverageState::Partial,
            )?,
        ],
    )?;
    let blocked = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &partial,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        blocked.outcomes()[0].state(),
        RecognizerRuleOutcomeState::Partial
    );
    assert!(blocked.outcomes()[0].matches().is_empty());
    Ok(())
}

#[test]
fn typed_join_mismatch_does_not_compare_string_spelling_only() -> TestResult {
    let context = context()?;
    let mut rule = direct_rule(vec![
        RecognizerClause::Fact {
            alias: "reference".into(),
            kind: "reference.api".into(),
        },
        RecognizerClause::Join {
            left: "call.member".into(),
            right: "reference.member".into(),
        },
    ]);
    rule.required_capabilities
        .push("reference.api.complete".into());
    rule.required_capabilities.sort();
    let (pack, plan) = compiled(rule, 100, 100)?;
    let call = fact(
        &context,
        "emmy.member_call",
        "KnownApi",
        "direct",
        1,
        GraphConfidence::Derived,
    )?;
    let mut reference = fact(
        &context,
        "reference.api",
        "KnownApi",
        "direct",
        2,
        GraphConfidence::Proven,
    )?;
    let mut value = serde_json::to_value(&reference)?;
    value["fields"]["member"] = serde_json::json!({"type":"string","value":"KnownApi"});
    value["fact_id"] = serde_json::json!(format!("recognizer-fact:sha256:{}", "f".repeat(64)));
    reference = serde_json::from_value(value)?;
    assert!(reference.validate(RecognizerFactLimits::default()).is_err());

    let typed_reference = RecognizerFact::new(
        context.context_id(),
        RecognizerFactInput {
            kind: "reference.api".into(),
            partition_id: "project.main".into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                "main.lua::function:1",
            )?,
            producer_id: "wow.reference".into(),
            producer_version: "e1-b.1".into(),
            confidence: GraphConfidence::Proven,
            fields: BTreeMap::from([
                (
                    Box::<str>::from("member"),
                    RecognizerFactValue::String("KnownApi".into()),
                ),
                (
                    Box::<str>::from("receiver"),
                    RecognizerFactValue::Identifier("C_Test".into()),
                ),
                (
                    Box::<str>::from("call_kind"),
                    RecognizerFactValue::String("direct".into()),
                ),
                (Box::<str>::from("ordinal"), RecognizerFactValue::Integer(2)),
            ]),
            source_handle_ids: vec![StableHandleId::derive(&"reference-source")?],
            evidence_ids: vec![EvidenceId::derive(&"reference-evidence")?],
        },
        RecognizerFactLimits::default(),
    )?;
    let facts = bundle(
        &context,
        vec![call, typed_reference],
        vec![
            coverage(
                &context,
                "emmy.member_call.complete",
                RecognizerFactCoverageState::Complete,
            )?,
            coverage(
                &context,
                "reference.api.complete",
                RecognizerFactCoverageState::Complete,
            )?,
        ],
    )?;
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        output.outcomes()[0].state(),
        RecognizerRuleOutcomeState::EvaluatedNoMatch
    );
    Ok(())
}

#[test]
fn output_amplification_and_cancellation_are_explicit() -> TestResult {
    let context = context()?;
    let (pack, plan) = compiled(direct_rule(Vec::new()), 1, 1)?;
    let facts = bundle(
        &context,
        vec![
            fact(
                &context,
                "emmy.member_call",
                "First",
                "direct",
                1,
                GraphConfidence::Derived,
            )?,
            fact(
                &context,
                "emmy.member_call",
                "Second",
                "direct",
                2,
                GraphConfidence::Derived,
            )?,
        ],
        vec![coverage(
            &context,
            "emmy.member_call.complete",
            RecognizerFactCoverageState::Complete,
        )?],
    )?;
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        output.outcomes()[0].state(),
        RecognizerRuleOutcomeState::Partial
    );
    assert!(output.outcomes()[0].truncated());
    assert_eq!(output.outcomes()[0].matches().len(), 1);

    let cancelled = AtomicBool::new(true);
    let output = execute_recognizer_plan(
        &context,
        &pack,
        &plan,
        &facts,
        RecognizerFactLimits::default(),
        &cancelled,
    )?;
    assert_eq!(
        output.outcomes()[0].state(),
        RecognizerRuleOutcomeState::Cancelled
    );
    assert!(output.outcomes()[0].matches().is_empty());
    Ok(())
}
