#![allow(clippy::unwrap_used)]
use super::*;
use crate::native::{ingest_document, source_digest};
use crate::native_view::{NativeCallableCorpus, project_callables};
use wow_core::ReferenceGenerationId;

const BEFORE: &str = "1111111111111111111111111111111111111111";
const AFTER: &str = "2222222222222222222222222222222222222222";
const API: &str = r#"APIDocumentation:AddDocumentationTable({Name="Example",Type="System",Namespace="C_Example",Environment="All",Functions={{Name="Read",Type="Function",Arguments={{Name="value",Type="number",Nilable=false,Default={flag=false,n=0},StrideIndex=1}},Returns={{Name="result",Type="number"}}}}})"#;

fn projection(text: &str, revision: &str, complete: bool) -> NativeViewProjection {
    let stop = AtomicBool::new(false);
    let document = ingest_document(
        revision,
        "API.lua",
        text,
        &source_digest(text.as_bytes()),
        &stop,
    )
    .unwrap();
    project_callables(
        &[document],
        "Mainline",
        ReferenceGenerationId::derive(&("comparison-fixture", revision)).unwrap(),
        if complete {
            NativeCallableCorpus::manifest_toc(1, 1, true)
        } else {
            NativeCallableCorpus::explicit_partial()
        },
        &stop,
    )
    .unwrap()
}
fn limits() -> CallableComparisonLimits {
    CallableComparisonLimits {
        max_keys: 16,
        max_view_bytes: 1024 * 1024,
        max_steps: 1024 * 1024,
    }
}
fn compare(
    before: &NativeViewProjection,
    after: &NativeViewProjection,
    keys: &[&str],
) -> NativeCallableComparison {
    compare_reference_generations(
        before,
        BEFORE,
        after,
        AFTER,
        &keys.iter().map(|key| (*key).to_owned()).collect::<Vec<_>>(),
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap()
}

#[test]
fn source_movement_does_not_change_original_contract_values_or_imply_continuity() {
    let before = projection(API, BEFORE, false);
    let after = projection(
        &format!("-- shifted source coordinates\n{API}"),
        AFTER,
        false,
    );
    let old = serde_json::to_vec(&before).unwrap();
    let new = serde_json::to_vec(&after).unwrap();
    let value =
        serde_json::to_value(compare(&before, &after, &["function:C_Example.Read"])).unwrap();
    assert_eq!(
        value["observations"][0]["comparison"]["state"],
        "selected_fields_equal"
    );
    assert_ne!(
        value["observations"][0]["before"]["record_digest"],
        value["observations"][0]["after"]["record_digest"]
    );
    assert_eq!(value["association_ceiling"], "candidate");
    assert_eq!(value["accepted_lineage_count"], 0);
    assert_eq!(value["lineage_negative_authority"], false);
    assert_eq!(old, serde_json::to_vec(&before).unwrap());
    assert_eq!(new, serde_json::to_vec(&after).unwrap());
}

#[test]
fn native_ordered_fields_distinguish_default_stride_type_nilability_and_multiple_returns() {
    let before = projection(API, BEFORE, false);
    let text = API.replace("Type=\"number\",Nilable=false,Default={flag=false,n=0},StrideIndex=1", "Type=\"bool\",Nilable=true,Default={flag=true,n=0},StrideIndex=2")
        .replace("Returns={{Name=\"result\",Type=\"number\"}}", "Returns={{Name=\"result\",Type=\"number\"},{Name=\"extra\",Type=\"string\",Default=\"must_not_leak_raw_default\"}}");
    let after = projection(&text, AFTER, false);
    let compared = compare(&before, &after, &["function:C_Example.Read"]);
    let value = serde_json::to_value(&compared).unwrap();
    assert_eq!(
        value["observations"][0]["comparison"]["state"],
        "selected_fields_differ"
    );
    assert_eq!(
        value["observations"][0]["comparison"]["differences"][0]["changed"],
        serde_json::json!(["type_name", "nilable", "default", "stride_index"])
    );
    assert_eq!(
        value["observations"][0]["comparison"]["differences"][1],
        serde_json::json!({"lane":"return","ordinal":2,"changed":["added"]})
    );
    assert!(
        !serde_json::to_string(&compared)
            .unwrap()
            .contains("must_not_leak_raw_default")
    );
    let reversed = compare_reference_generations(
        &after,
        AFTER,
        &before,
        BEFORE,
        &["function:C_Example.Read".to_owned()],
        limits(),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(reversed).unwrap()["observations"][0]["comparison"]["differences"][1]
            ["changed"],
        serde_json::json!(["not_present_after"])
    );
}

#[test]
fn native_conflicts_and_partial_or_exact_absence_are_observations_not_removal() {
    let before = projection(API, BEFORE, false);
    let after = projection(
        &format!("{API}\n{}", API.replace("Type=\"number\"", "Type=\"bool\"")),
        AFTER,
        false,
    );
    let value = serde_json::to_value(compare(
        &before,
        &after,
        &["function:C_Example.Read", "function:C_Example.Missing"],
    ))
    .unwrap();
    assert_eq!(
        value["observations"][0]["key"],
        "function:C_Example.Missing"
    );
    assert_eq!(value["observations"][0]["after"]["state"], "unknown");
    assert_eq!(value["observations"][1]["after"]["state"], "conflict");
    assert_eq!(
        value["observations"][1]["comparison"]["state"],
        "not_evaluated"
    );
    let before = projection(API, BEFORE, true);
    let after = projection(API, AFTER, true);
    let result = compare(&before, &after, &["function:C_Example.Missing"]);
    assert!(result.is_partial());
    let value = serde_json::to_value(result).unwrap();
    assert_eq!(
        value["observations"][0]["before"]["state"],
        "authoritative_exact_key_absence"
    );
    assert_eq!(value["lineage_negative_authority"], false);
}

#[test]
fn stale_public_view_missing_original_source_and_wrong_revision_never_pass() {
    let before = projection(API, BEFORE, false);
    let after = projection(API, AFTER, false);
    let stop = AtomicBool::new(false);
    let keys = vec!["function:C_Example.Read".to_owned()];
    let mut missing_source = before.clone();
    missing_source.sources.clear();
    assert_eq!(
        compare_reference_generations(
            &missing_source,
            BEFORE,
            &after,
            AFTER,
            &keys,
            limits(),
            &stop
        )
        .unwrap_err()
        .code,
        NativeErrorCode::InvalidIdentity
    );
    assert_eq!(
        compare_reference_generations(&before, AFTER, &after, AFTER, &keys, limits(), &stop)
            .unwrap_err()
            .code,
        NativeErrorCode::InvalidIdentity
    );
    let mut mixed = before.clone();
    mixed.view = after.view.clone();
    assert_eq!(
        compare_reference_generations(
            &mixed,
            BEFORE,
            &after,
            AFTER,
            &["function:C_Example.Missing".to_owned()],
            limits(),
            &stop
        )
        .unwrap_err()
        .code,
        NativeErrorCode::InvalidIdentity
    );
    let mut altered = before.clone();
    altered.sources[0].path = "Other.lua".to_owned();
    assert_eq!(
        compare_reference_generations(&altered, BEFORE, &after, AFTER, &keys, limits(), &stop)
            .unwrap_err()
            .code,
        NativeErrorCode::InvalidIdentity
    );
}

#[test]
fn one_cumulative_step_budget_and_cancellation_bound_the_real_comparison() {
    let before = projection(API, BEFORE, false);
    let after = projection(API, AFTER, false);
    let keys = vec!["function:C_Example.Read".to_owned()];
    let stop = AtomicBool::new(false);
    let mut tiny = limits();
    tiny.max_steps = 1;
    assert_eq!(
        compare_reference_generations(&before, BEFORE, &after, AFTER, &keys, tiny, &stop)
            .unwrap_err()
            .code,
        NativeErrorCode::Limit
    );
    assert_eq!(
        compare_reference_generations(
            &before,
            BEFORE,
            &after,
            AFTER,
            &keys,
            limits(),
            &AtomicBool::new(true)
        )
        .unwrap_err()
        .code,
        NativeErrorCode::Cancelled
    );
    let duplicates = vec![keys[0].clone(), keys[0].clone()];
    assert_eq!(
        compare_reference_generations(&before, BEFORE, &after, AFTER, &duplicates, limits(), &stop)
            .unwrap_err()
            .code,
        NativeErrorCode::InvalidIdentity
    );
}
