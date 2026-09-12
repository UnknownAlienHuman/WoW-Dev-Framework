use std::collections::BTreeMap;
use std::error::Error;

use wow_core::canonical_json_bytes;
use wow_recognizers::{
    RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerCapture, RecognizerCaptureCardinality,
    RecognizerClause, RecognizerErrorCode, RecognizerOutput, RecognizerOutputConfidence,
    RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument, RecognizerPackLiteral,
    RecognizerPackRollout, RecognizerPackTrustClass, RecognizerRule, parse_recognizer_pack,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn budgets() -> RecognizerPackBudgets {
    RecognizerPackBudgets {
        max_rules: 8,
        max_clauses_per_rule: 32,
        max_clause_depth: 8,
        max_join_expansions_per_rule: 10_000,
        max_matches_per_rule_partition: 1_000,
        max_proposals_per_rule_partition: 2_000,
        max_explanation_bytes: 64 * 1024,
    }
}

fn valid_rule() -> RecognizerRule {
    let mut semantic_key = BTreeMap::new();
    semantic_key.insert("member".into(), "member".into());
    semantic_key.insert("receiver".into(), "call.receiver".into());
    RecognizerRule {
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
            value_type: "string".into(),
            source: "call.member".into(),
            cardinality: RecognizerCaptureCardinality::One,
        }],
        outputs: vec![RecognizerOutput::EntityAssertion {
            output_id: "called_member".into(),
            entity_kind_id: "lua.member".into(),
            semantic_key,
            confidence: RecognizerOutputConfidence::Derived,
        }],
        positive_fixture_ids: vec!["fixture.direct-call.positive".into()],
        near_negative_fixture_ids: vec!["fixture.direct-call.near-negative".into()],
        partial_fixture_ids: vec!["fixture.direct-call.partial".into()],
        mutation_fixture_ids: vec!["fixture.direct-call.mutation".into()],
    }
}

fn valid_document() -> RecognizerPackDocument {
    RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "core.e2-b".into(),
            version: "1.0.0".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: "recognizer-facts:e2-b:1".into(),
            graph_registry_bundle_id: "graph-registry:e2-a:1".into(),
            evaluation_profile_id: "recognizer-evaluation:e2-b:1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: budgets(),
            rules: vec![valid_rule()],
        },
    }
}

fn canonical(document: &RecognizerPackDocument) -> TestResult<Vec<u8>> {
    Ok(canonical_json_bytes(document)?)
}

#[test]
fn canonical_pack_compiles_and_round_trips_identity() -> TestResult {
    let document = valid_document();
    let bytes = canonical(&document)?;
    let compiled = parse_recognizer_pack(&bytes)?;
    assert!(compiled.pack_digest().starts_with("recognizer-pack:sha256:"));
    assert_eq!(compiled.document(), &document);
    compiled.validate()?;
    let encoded = serde_json::to_vec(&compiled)?;
    let decoded = serde_json::from_slice(&encoded)?;
    assert_eq!(compiled, decoded);
    Ok(())
}

#[test]
fn noncanonical_json_unknown_fields_null_and_floats_fail_closed() -> TestResult {
    let document = valid_document();
    let canonical = canonical(&document)?;
    let mut padded = canonical.clone();
    padded.push(b'\n');
    assert_eq!(
        parse_recognizer_pack(&padded).err().ok_or("padded pack must fail")?.code(),
        RecognizerErrorCode::PackNonCanonical
    );

    let mut value = serde_json::to_value(&document)?;
    value["extra"] = serde_json::json!(true);
    assert_eq!(
        parse_recognizer_pack(&serde_json::to_vec(&value)?)
            .err()
            .ok_or("unknown field must fail")?
            .code(),
        RecognizerErrorCode::PackSyntaxInvalid
    );
    let mut value = serde_json::to_value(&document)?;
    value["pack"]["version"] = serde_json::Value::Null;
    assert_eq!(
        parse_recognizer_pack(&serde_json::to_vec(&value)?)
            .err()
            .ok_or("null field must fail")?
            .code(),
        RecognizerErrorCode::PackSyntaxInvalid
    );
    let mut value = serde_json::to_value(&document)?;
    value["pack"]["rules"][0]["clauses"][1]["value"] = serde_json::json!(0.5);
    assert_eq!(
        parse_recognizer_pack(&serde_json::to_vec(&value)?)
            .err()
            .ok_or("floating literal must fail")?
            .code(),
        RecognizerErrorCode::PackSyntaxInvalid
    );
    Ok(())
}

#[test]
fn duplicate_rules_unsorted_ids_and_excessive_budgets_are_rejected() -> TestResult {
    let mut document = valid_document();
    document.pack.rules.push(valid_rule());
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("duplicate rules must fail")?
            .code(),
        RecognizerErrorCode::PackRuleDuplicate
    );

    let mut document = valid_document();
    document.pack.rules[0].required_capabilities = vec!["z.capability".into(), "a.capability".into()];
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("unsorted capabilities must fail")?
            .code(),
        RecognizerErrorCode::PackInvalid
    );

    let mut document = valid_document();
    document.pack.budgets.max_rules = u32::MAX;
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("excessive budget must fail")?
            .code(),
        RecognizerErrorCode::PackBudgetInvalid
    );
    Ok(())
}

#[test]
fn negative_clause_requires_declared_complete_coverage() -> TestResult {
    let mut document = valid_document();
    document.pack.rules[0].clauses.push(RecognizerClause::NotExists {
        clauses: vec![RecognizerClause::FieldEq {
            field: "call.member".into(),
            value: RecognizerPackLiteral::String("Removed".into()),
        }],
        required_complete_capability: "reference.api.complete".into(),
    });
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("negative clause without coverage must fail")?
            .code(),
        RecognizerErrorCode::PackNegativeCoverageMissing
    );
    document.pack.rules[0]
        .required_capabilities
        .push("reference.api.complete".into());
    document.pack.rules[0].required_capabilities.sort();
    parse_recognizer_pack(&canonical(&document)?)?;
    Ok(())
}

#[test]
fn undeclared_alias_invalid_output_and_clause_depth_fail_closed() -> TestResult {
    let mut document = valid_document();
    document.pack.rules[0].clauses[1] = RecognizerClause::FieldEq {
        field: "missing.member".into(),
        value: RecognizerPackLiteral::String("Known".into()),
    };
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("unknown alias must fail")?
            .code(),
        RecognizerErrorCode::PackClauseInvalid
    );

    let mut document = valid_document();
    document.pack.rules[0].outputs = vec![RecognizerOutput::RelationAssertion {
        output_id: "call_edge".into(),
        relation_kind_id: "calls".into(),
        source: "unknown_capture".into(),
        target: "call.member".into(),
        confidence: RecognizerOutputConfidence::Possible,
    }];
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("invalid output reference must fail")?
            .code(),
        RecognizerErrorCode::PackOutputInvalid
    );

    let mut document = valid_document();
    document.pack.budgets.max_clause_depth = 2;
    document.pack.rules[0].clauses = vec![RecognizerClause::AllOf {
        clauses: vec![RecognizerClause::AnyOf {
            clauses: vec![RecognizerClause::Exists {
                clauses: vec![RecognizerClause::Fact {
                    alias: "deep".into(),
                    kind: "emmy.member_call".into(),
                }],
            }],
        }],
    }];
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("deep clauses must fail")?
            .code(),
        RecognizerErrorCode::PackClauseInvalid
    );
    Ok(())
}

#[test]
fn noncore_pack_cannot_request_default_rollout() -> TestResult {
    let mut document = valid_document();
    document.pack.trust_class = RecognizerPackTrustClass::Experimental;
    document.pack.rollout = RecognizerPackRollout::Default;
    assert_eq!(
        parse_recognizer_pack(&canonical(&document)?)
            .err()
            .ok_or("experimental default rollout must fail")?
            .code(),
        RecognizerErrorCode::PackInvalid
    );
    Ok(())
}
