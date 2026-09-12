use std::collections::BTreeMap;
use std::error::Error;
use std::str::FromStr;

use wow_core::{
    ContentDigest, EvidenceId, GenerationContext, GenerationContextBuilder, ProducerId,
    ProducerVersionEntry, ProfileId, ProfileIdentityBuilder, ProfileKind, ProjectGenerationId,
    ReferenceGenerationId, SchemaId, SchemaVersionEntry, SourceKind, SourceLogicalSnapshot,
    StableHandleId, ToolVersion,
};
use wow_graph::GraphConfidence;
use wow_recognizers::{
    RecognizerErrorCode, RecognizerFact, RecognizerFactBundle, RecognizerFactCoverage,
    RecognizerFactCoverageInput, RecognizerFactCoverageState, RecognizerFactInput,
    RecognizerFactLimits, RecognizerFactScope, RecognizerFactScopeKind, RecognizerFactValue,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn context(project_seed: &str) -> TestResult<GenerationContext> {
    let profile_schema = SchemaVersionEntry::new(
        SchemaId::from_str("schema:wow:recognizer-fact-fixture")?,
        ToolVersion::from_str("1.0.0")?,
    );
    let profile = ProfileIdentityBuilder::new(
        ProfileId::from_str("profile:fixture:recognizer-facts")?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:recognizer-facts-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7_u8; 32]),
    )
    .client_version(ToolVersion::from_str("12.1.0")?)
    .schema_versions(vec![profile_schema.clone()])
    .fixture_scope("recognizer fact bundle tests")
    .build()?;
    Ok(GenerationContextBuilder::new(
        profile,
        ReferenceGenerationId::derive(&"recognizer-reference-v1")?,
    )
    .project_generation(ProjectGenerationId::derive(&project_seed)?)
    .schema_versions(vec![profile_schema])
    .producer_versions(vec![ProducerVersionEntry::new(
        ProducerId::from_str("wow.recognizers")?,
        ToolVersion::from_str("0.1.0")?,
    )])
    .build()?)
}

fn fields(member: &str) -> BTreeMap<Box<str>, RecognizerFactValue> {
    BTreeMap::from([
        (
            Box::<str>::from("member"),
            RecognizerFactValue::Identifier(member.into()),
        ),
        (
            Box::<str>::from("receiver"),
            RecognizerFactValue::Identifier("Frame".into()),
        ),
        (Box::<str>::from("arity"), RecognizerFactValue::Integer(1)),
    ])
}

fn fact(
    context: &GenerationContext,
    partition: &str,
    member: &str,
    support_seed: &str,
) -> TestResult<RecognizerFact> {
    Ok(RecognizerFact::new(
        context.context_id(),
        RecognizerFactInput {
            kind: "lua.member_call".into(),
            partition_id: partition.into(),
            scope: RecognizerFactScope::new(
                RecognizerFactScopeKind::Function,
                "main.lua::function:1",
            )?,
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            confidence: GraphConfidence::Derived,
            fields: fields(member),
            source_handle_ids: vec![StableHandleId::derive(&(support_seed, "source"))?],
            evidence_ids: vec![EvidenceId::derive(&(support_seed, "evidence"))?],
        },
        RecognizerFactLimits::default(),
    )?)
}

fn coverage(
    context: &GenerationContext,
    partition: &str,
    capability: &str,
    state: RecognizerFactCoverageState,
    blocker: Option<&str>,
) -> TestResult<RecognizerFactCoverage> {
    Ok(RecognizerFactCoverage::new(
        RecognizerFactCoverageInput {
            context_id: context.context_id(),
            partition_id: partition.into(),
            capability_id: capability.into(),
            producer_id: "wow.emmy".into(),
            producer_version: "e0-c.1".into(),
            state,
            blocker_ids: blocker.into_iter().map(Into::into).collect(),
        },
        RecognizerFactLimits::default(),
    )?)
}

#[test]
fn bundle_is_order_invariant_and_queries_exact_partitions() -> TestResult {
    let context = context("project-a")?;
    let call = fact(&context, "project.main", "RegisterEvent", "call")?;
    let dependency = fact(&context, "reference.api", "CreateFrame", "dependency")?;
    let call_coverage = coverage(
        &context,
        "project.main",
        "emmy.fact.calls",
        RecognizerFactCoverageState::Complete,
        None,
    )?;
    let reference_coverage = coverage(
        &context,
        "reference.api",
        "reference.api.lookup",
        RecognizerFactCoverageState::Complete,
        None,
    )?;

    let left = RecognizerFactBundle::build(
        &context,
        "project.main",
        vec!["reference.api".into()],
        vec![call.clone(), dependency.clone()],
        vec![call_coverage.clone(), reference_coverage.clone()],
        RecognizerFactLimits::default(),
    )?;
    let right = RecognizerFactBundle::build(
        &context,
        "project.main",
        vec!["reference.api".into()],
        vec![dependency, call],
        vec![reference_coverage, call_coverage],
        RecognizerFactLimits::default(),
    )?;

    assert_eq!(left, right);
    assert_eq!(left.context_id(), context.context_id());
    assert_eq!(left.visible_partition_ids().len(), 2);
    assert_eq!(left.facts_by_kind("lua.member_call").count(), 2);
    assert!(left.has_complete_capability("project.main", "emmy.fact.calls"));
    assert!(!left.has_complete_capability("reference.api", "emmy.fact.calls"));
    assert!(left.fact_by_id(left.facts()[0].fact_id()).is_some());
    left.validate(&context, RecognizerFactLimits::default())?;
    Ok(())
}

#[test]
fn exact_duplicate_facts_merge_support_without_changing_fact_identity() -> TestResult {
    let context = context("project-a")?;
    let first = fact(&context, "project.main", "RegisterEvent", "support-a")?;
    let second = fact(&context, "project.main", "RegisterEvent", "support-b")?;
    assert_eq!(first.fact_id(), second.fact_id());

    let merged = RecognizerFactBundle::build(
        &context,
        "project.main",
        Vec::new(),
        vec![first, second],
        vec![coverage(
            &context,
            "project.main",
            "emmy.fact.calls",
            RecognizerFactCoverageState::Complete,
            None,
        )?],
        RecognizerFactLimits::default(),
    )?;
    assert_eq!(merged.facts().len(), 1);
    assert_eq!(merged.facts()[0].source_handle_ids().len(), 2);
    assert_eq!(merged.facts()[0].evidence_ids().len(), 2);
    Ok(())
}

#[test]
fn mixed_generation_and_undeclared_partition_fail_before_publication() -> TestResult {
    let context_a = context("project-a")?;
    let context_b = context("project-b")?;
    let foreign = fact(&context_b, "project.main", "RegisterEvent", "foreign")?;
    assert_eq!(
        RecognizerFactBundle::build(
            &context_a,
            "project.main",
            Vec::new(),
            vec![foreign],
            Vec::new(),
            RecognizerFactLimits::default(),
        )
        .err()
        .ok_or("mixed context must fail")?
        .code(),
        RecognizerErrorCode::FactContextMismatch
    );

    let undeclared = fact(&context_a, "project.foreign", "RegisterEvent", "foreign")?;
    assert_eq!(
        RecognizerFactBundle::build(
            &context_a,
            "project.main",
            Vec::new(),
            vec![undeclared],
            Vec::new(),
            RecognizerFactLimits::default(),
        )
        .err()
        .ok_or("undeclared partition must fail")?
        .code(),
        RecognizerErrorCode::FactPartitionUnknown
    );
    Ok(())
}

#[test]
fn conflicting_coverage_and_incomplete_negative_authority_fail_closed() -> TestResult {
    let context = context("project-a")?;
    let complete = coverage(
        &context,
        "project.main",
        "emmy.fact.calls",
        RecognizerFactCoverageState::Complete,
        None,
    )?;
    let partial = coverage(
        &context,
        "project.main",
        "emmy.fact.calls",
        RecognizerFactCoverageState::Partial,
        Some("parse.failure"),
    )?;
    assert_eq!(
        RecognizerFactBundle::build(
            &context,
            "project.main",
            Vec::new(),
            vec![fact(&context, "project.main", "RegisterEvent", "call",)?],
            vec![complete, partial],
            RecognizerFactLimits::default(),
        )
        .err()
        .ok_or("conflicting coverage must fail")?
        .code(),
        RecognizerErrorCode::FactCoverageInvalid
    );

    let bundle = RecognizerFactBundle::build(
        &context,
        "project.main",
        Vec::new(),
        vec![fact(&context, "project.main", "RegisterEvent", "call")?],
        vec![coverage(
            &context,
            "project.main",
            "emmy.fact.calls",
            RecognizerFactCoverageState::Partial,
            Some("parse.failure"),
        )?],
        RecognizerFactLimits::default(),
    )?;
    assert!(!bundle.has_complete_capability("project.main", "emmy.fact.calls"));
    Ok(())
}

#[test]
fn tampered_bundle_identity_is_rejected_on_read_back() -> TestResult {
    let context = context("project-a")?;
    let bundle = RecognizerFactBundle::build(
        &context,
        "project.main",
        Vec::new(),
        vec![fact(&context, "project.main", "RegisterEvent", "call")?],
        vec![coverage(
            &context,
            "project.main",
            "emmy.fact.calls",
            RecognizerFactCoverageState::Complete,
            None,
        )?],
        RecognizerFactLimits::default(),
    )?;
    let mut value = serde_json::to_value(&bundle)?;
    value["bundle_id"] =
        serde_json::json!(format!("recognizer-fact-bundle:sha256:{}", "f".repeat(64)));
    let tampered: RecognizerFactBundle = serde_json::from_value(value)?;
    assert_eq!(
        tampered
            .validate(&context, RecognizerFactLimits::default())
            .err()
            .ok_or("tampered bundle must fail")?
            .code(),
        RecognizerErrorCode::FactBundleIdentityMismatch
    );
    Ok(())
}

#[test]
fn fact_limits_and_value_validation_are_enforced() -> TestResult {
    let context = context("project-a")?;
    let mut invalid_fields = fields("RegisterEvent");
    invalid_fields.insert(
        "bad field".into(),
        RecognizerFactValue::String("value".into()),
    );
    assert_eq!(
        RecognizerFact::new(
            context.context_id(),
            RecognizerFactInput {
                kind: "lua.member_call".into(),
                partition_id: "project.main".into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    "main.lua::function:1",
                )?,
                producer_id: "wow.emmy".into(),
                producer_version: "e0-c.1".into(),
                confidence: GraphConfidence::Derived,
                fields: invalid_fields,
                source_handle_ids: vec![StableHandleId::derive(&"source")?],
                evidence_ids: vec![EvidenceId::derive(&"evidence")?],
            },
            RecognizerFactLimits::default(),
        )
        .err()
        .ok_or("invalid field must fail")?
        .code(),
        RecognizerErrorCode::FactInvalid
    );

    let limits = RecognizerFactLimits::new(1, 1, 0, 1, 1, 1, 1)?;
    assert_eq!(
        RecognizerFact::new(
            context.context_id(),
            RecognizerFactInput {
                kind: "lua.member_call".into(),
                partition_id: "project.main".into(),
                scope: RecognizerFactScope::new(
                    RecognizerFactScopeKind::Function,
                    "main.lua::function:1",
                )?,
                producer_id: "wow.emmy".into(),
                producer_version: "e0-c.1".into(),
                confidence: GraphConfidence::Derived,
                fields: fields("RegisterEvent"),
                source_handle_ids: vec![StableHandleId::derive(&"source")?],
                evidence_ids: vec![EvidenceId::derive(&"evidence")?],
            },
            limits,
        )
        .err()
        .ok_or("field budget must fail")?
        .code(),
        RecognizerErrorCode::FactBudgetExceeded
    );
    Ok(())
}
