use super::*;
use serde_json::json;
fn report() -> Value {
    let hash = format!("sha256:{}", "a".repeat(64));
    json!({"schema":"wow-native-annotation-library/6","projection":"projected_with_sidecars","literal_execution":{"schema":"wow-literal-execution/1","module":{"sha256":hash,"epoch":2},"calls":[{"ordinal":0,"operation":"events","request_sha256":hash,"result":{"Ok":hash}}],"artifacts":[{"path":"events-0001.lua","call_ordinal":0}]},"files":[{"path":"events-0001.lua","sha256":hash}]})
}
#[test]
fn verifies_bound_literal_outputs() -> Result<()> {
    verify(&report(), Some(&format!("sha256:{}", "a".repeat(64))))
}
#[test]
fn rejects_missing_trace_and_downgrade() {
    let mut value = report();
    value["schema"] = json!("wow-native-annotation-library/5");
    assert!(verify(&value, None).is_err());
    value = report();
    value.as_object_mut().map(|v| v.remove("literal_execution"));
    assert!(verify(&value, None).is_err());
    value["schema"] = json!("wow-native-annotation-library/3");
    assert!(verify(&value, None).is_ok());
    assert!(verify(&value, Some(&format!("sha256:{}", "a".repeat(64)))).is_err());
}
#[test]
fn rejects_wrong_module_identity_ordinal_and_digest() {
    for (pointer, replacement) in [
        ("/literal_execution/module/sha256", json!("bad")),
        ("/literal_execution/module/epoch", json!(-1)),
        ("/literal_execution/calls/0/ordinal", json!(2)),
        ("/literal_execution/calls/0/request_sha256", json!("bad")),
        (
            "/literal_execution/calls/0/result/Ok",
            json!(format!("sha256:{}", "b".repeat(64))),
        ),
        ("/literal_execution/artifacts/0/call_ordinal", json!(3)),
    ] {
        let mut value = report();
        if let Some(slot) = value.pointer_mut(pointer) {
            *slot = replacement;
        }
        assert!(verify(&value, None).is_err(), "{pointer}");
    }
    assert!(verify(&report(), Some(&format!("sha256:{}", "b".repeat(64)))).is_err());
}
#[test]
fn rejects_missing_duplicate_or_nonliteral_bindings() {
    let mut value = report();
    value["literal_execution"]["artifacts"] = json!([]);
    assert!(verify(&value, None).is_err());
    value = report();
    let binding = value["literal_execution"]["artifacts"][0].clone();
    value["literal_execution"]["artifacts"] = json!([binding, binding]);
    assert!(verify(&value, None).is_err());
    value = report();
    value["literal_execution"]["artifacts"][0]["path"] = json!("api-0000.lua");
    assert!(verify(&value, None).is_err());
}
#[test]
fn rejects_fatal_wire_failures_even_when_projection_is_partial() {
    for error in ["BridgeFailure", "InvalidWire", "IncompatibleSchema"] {
        let mut value = report();
        value["projection"] = json!("partial");
        value["literal_execution"]["calls"][0]["result"] = json!({"Err":error});
        assert!(verify(&value, None).is_err());
    }
}
#[test]
fn requires_last_result_not_an_earlier_validation_call() {
    let mut value = report();
    let mut second = value["literal_execution"]["calls"][0].clone();
    second["ordinal"] = json!(1);
    if let Some(calls) = value["literal_execution"]["calls"].as_array_mut() {
        calls.push(second);
    }
    assert!(verify(&value, None).is_err());
}
#[test]
fn rejects_unknown_fields_operations_and_ambiguous_result_tags() {
    for (pointer, replacement) in [
        ("/literal_execution/schema", json!("other")),
        ("/literal_execution/calls/0/operation", json!("shell")),
        (
            "/literal_execution/calls/0/result",
            json!({"Ok":"x","Err":"InputLimit"}),
        ),
        (
            "/literal_execution/module",
            json!({"sha256":format!("sha256:{}","a".repeat(64)),"epoch":2,"extra":1}),
        ),
    ] {
        let mut value = report();
        if let Some(slot) = value.pointer_mut(pointer) {
            *slot = replacement;
        }
        assert!(verify(&value, None).is_err());
    }
}
