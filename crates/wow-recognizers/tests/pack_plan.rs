use std::collections::BTreeMap;
use std::error::Error;

use wow_core::canonical_json_bytes;
use wow_recognizers::{
    CompiledRecognizerPlan, RECOGNIZER_PACK_SCHEMA_VERSION, RecognizerCapture,
    RecognizerCaptureCardinality, RecognizerClause, RecognizerErrorCode, RecognizerOutput,
    RecognizerOutputConfidence, RecognizerPack, RecognizerPackBudgets, RecognizerPackDocument,
    RecognizerPackLiteral, RecognizerPackRollout, RecognizerPackTrustClass,
    RecognizerPlanCostClass, RecognizerRule, compile_recognizer_plan, parse_recognizer_pack,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn document() -> RecognizerPackDocument {
    let mut semantic_key = BTreeMap::new();
    semantic_key.insert("member".into(), "member".into());
    semantic_key.insert("receiver".into(), "call.receiver".into());
    RecognizerPackDocument {
        schema_version: RECOGNIZER_PACK_SCHEMA_VERSION,
        pack: RecognizerPack {
            pack_id: "core.plan".into(),
            version: "1.0.0".into(),
            trust_class: RecognizerPackTrustClass::Core,
            fact_schema_profile_id: "recognizer-facts:e2-b:1".into(),
            graph_registry_bundle_id: "graph-registry:e2-a:1".into(),
            evaluation_profile_id: "recognizer-evaluation:e2-b:1".into(),
            rollout: RecognizerPackRollout::Shadow,
            budgets: RecognizerPackBudgets {
                max_rules: 8,
                max_clauses_per_rule: 32,
                max_clause_depth: 8,
                max_join_expansions_per_rule: 10_000,
                max_matches_per_rule_partition: 1_000,
                max_proposals_per_rule_partition: 2_000,
                max_explanation_bytes: 64 * 1024,
            },
            rules: vec![RecognizerRule {
                rule_id: "core.direct-call-plan".into(),
                version: 1,
                required_capabilities: vec![
                    "emmy.member_call.complete".into(),
                    "reference.api.complete".into(),
                ],
                scope: "function".into(),
                clauses: vec![
                    RecognizerClause::Fact {
                        alias: "call".into(),
                        kind: "emmy.member_call".into(),
                    },
                    RecognizerClause::Fact {
                        alias: "reference".into(),
                        kind: "reference.api".into(),
                    },
                    RecognizerClause::Join {
                        left: "call.member".into(),
                        right: "reference.member".into(),
                    },
                    RecognizerClause::FieldEq {
                        field: "call.call_kind".into(),
                        value: RecognizerPackLiteral::String("direct".into()),
                    },
                    RecognizerClause::AllOf {
                        clauses: vec![
                            RecognizerClause::SameScope {
                                left: "call.file".into(),
                                right: "reference.file".into(),
                                scope: "project".into(),
                            },
                            RecognizerClause::NotExists {
                                clauses: vec![RecognizerClause::FieldEq {
                                    field: "reference.status".into(),
                                    value: RecognizerPackLiteral::String("removed".into()),
                                }],
                                required_complete_capability: "reference.api.complete".into(),
                            },
                        ],
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
                positive_fixture_ids: vec!["fixture.plan.positive".into()],
                near_negative_fixture_ids: vec!["fixture.plan.near-negative".into()],
                partial_fixture_ids: vec!["fixture.plan.partial".into()],
                mutation_fixture_ids: vec!["fixture.plan.mutation".into()],
            }],
        },
    }
}

fn compiled_pack(document: &RecognizerPackDocument) -> TestResult<wow_recognizers::CompiledRecognizerPack> {
    Ok(parse_recognizer_pack(&canonical_json_bytes(document)?)?)
}

#[test]
fn plan_is_content_addressed_ordered_and_resource_bounded() -> TestResult {
    let pack = compiled_pack(&document())?;
    let plan = compile_recognizer_plan(&pack)?;
    let repeated = compile_recognizer_plan(&pack)?;
    assert_eq!(plan, repeated);
    assert!(plan.plan_id().as_str().starts_with("recognizer-plan:sha256:"));
    assert_eq!(plan.source_pack_digest(), pack.pack_digest());
    assert_eq!(plan.rules().len(), 1);
    let rule = &plan.rules()[0];
    assert_eq!(rule.rule_id(), "core.direct-call-plan");
    assert_eq!(rule.rule_version(), 1);
    assert_eq!(rule.capture_names(), ["member"]);
    assert_eq!(rule.output_ids(), ["called_member"]);
    let costs = rule
        .evaluation_order()
        .iter()
        .map(|step| step.cost_class())
        .collect::<Vec<_>>();
    assert!(costs.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(costs[0], RecognizerPlanCostClass::FactScan);
    assert_eq!(costs[1], RecognizerPlanCostClass::FactScan);
    assert_eq!(rule.bounds().fact_scans, 2);
    assert_eq!(rule.bounds().joins, 1);
    assert_eq!(rule.bounds().captures, 1);
    assert_eq!(rule.bounds().outputs, 1);
    assert_eq!(plan.bounds().rules, 1);
    assert_eq!(plan.bounds().clause_steps, rule.bounds().clause_steps);
    plan.validate()?;
    Ok(())
}

#[test]
fn nested_clause_paths_and_step_ids_are_unique() -> TestResult {
    let plan = compile_recognizer_plan(&compiled_pack(&document())?)?;
    let steps = plan.rules()[0].evaluation_order();
    let paths = steps
        .iter()
        .map(|step| step.clause_path().to_vec())
        .collect::<std::collections::BTreeSet<_>>();
    let ids = steps
        .iter()
        .map(|step| step.step_id())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(paths.len(), steps.len());
    assert_eq!(ids.len(), steps.len());
    assert!(paths.contains(&vec![4, 0]));
    assert!(paths.contains(&vec![4, 1, 0]));
    let negative = steps
        .iter()
        .find(|step| step.required_capabilities() == ["reference.api.complete"])
        .ok_or("negative-existence step")?;
    assert_eq!(negative.cost_class(), RecognizerPlanCostClass::NegativeExistence);
    Ok(())
}

#[test]
fn semantic_clause_change_changes_pack_and_plan_identity() -> TestResult {
    let original_document = document();
    let original_pack = compiled_pack(&original_document)?;
    let original_plan = compile_recognizer_plan(&original_pack)?;
    let mut changed_document = original_document;
    changed_document.pack.rules[0].clauses[3] = RecognizerClause::FieldEq {
        field: "call.call_kind".into(),
        value: RecognizerPackLiteral::String("colon".into()),
    };
    let changed_pack = compiled_pack(&changed_document)?;
    let changed_plan = compile_recognizer_plan(&changed_pack)?;
    assert_ne!(original_pack.pack_digest(), changed_pack.pack_digest());
    assert_ne!(original_plan.plan_id(), changed_plan.plan_id());
    Ok(())
}

#[test]
fn tampered_plan_identity_and_order_fail_read_back() -> TestResult {
    let plan = compile_recognizer_plan(&compiled_pack(&document())?)?;
    let mut value = serde_json::to_value(&plan)?;
    value["plan_id"] = serde_json::json!(format!("recognizer-plan:sha256:{}", "f".repeat(64)));
    let tampered: CompiledRecognizerPlan = serde_json::from_value(value)?;
    assert_eq!(
        tampered
            .validate()
            .err()
            .ok_or("tampered plan ID must fail")?
            .code(),
        RecognizerErrorCode::PlanIdentityMismatch
    );

    let mut value = serde_json::to_value(&plan)?;
    value["rules"][0]["evaluation_order"]
        .as_array_mut()
        .ok_or("evaluation order")?
        .reverse();
    let tampered: CompiledRecognizerPlan = serde_json::from_value(value)?;
    assert_eq!(
        tampered
            .validate()
            .err()
            .ok_or("reordered plan must fail")?
            .code(),
        RecognizerErrorCode::PlanInvalid
    );
    Ok(())
}

#[test]
fn proposal_budget_smaller_than_static_outputs_is_rejected() -> TestResult {
    let mut document = document();
    document.pack.budgets.max_proposals_per_rule_partition = 1;
    let first = document.pack.rules[0].outputs[0].clone();
    let mut second = first;
    match &mut second {
        RecognizerOutput::EntityAssertion { output_id, .. }
        | RecognizerOutput::RelationAssertion { output_id, .. } => {
            *output_id = "second_output".into();
        }
    }
    document.pack.rules[0].outputs.push(second);
    let pack = compiled_pack(&document)?;
    assert_eq!(
        compile_recognizer_plan(&pack)
            .err()
            .ok_or("proposal budget must fail")?
            .code(),
        RecognizerErrorCode::PlanInvalid
    );
    Ok(())
}
