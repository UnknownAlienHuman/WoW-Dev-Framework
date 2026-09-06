use serde_json::Value;
type TestResult = Result<(), Box<dyn std::error::Error>>;
pub fn compare(native: &Value, selected: &Value) -> TestResult {
    let mut actual = selected["library"].clone();
    actual
        .as_object_mut()
        .ok_or("invalid library")?
        .remove("literal_execution");
    actual["schema"] = native["library"]["schema"].clone();
    assert_eq!(
        actual, native["library"],
        "selected module changed native facts, bytes or source maps"
    );
    assert_eq!(selected["status"], native["status"]);
    assert_eq!(selected["source_order"], native["source_order"]);
    assert_eq!(selected["input_failures"], native["input_failures"]);
    Ok(())
}
