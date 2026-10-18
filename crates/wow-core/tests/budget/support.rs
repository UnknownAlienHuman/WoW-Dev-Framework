use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wow_core::{
    Budget, BudgetLimits, CoreErrorCode, CoreResult, GenerationContext, NegativeAuthorityDecision,
    TruncationEntry, TruncationState, evaluate_negative_authority,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub const CLEAN: &str = include_str!("../../examples/e0-clean-result.json");
pub const DIMENSIONS: [(&str, &str); 9] = [
    ("max_coverage_records", "coverage_records"),
    ("max_capability_summaries", "capability_summaries"),
    ("max_source_handles", "source_handles"),
    ("max_evidence_records", "evidence_records"),
    ("max_conflicts", "conflicts"),
    ("max_findings", "findings"),
    ("max_not_evaluated", "not_evaluated"),
    ("max_warnings", "warnings"),
    ("max_output_bytes", "output_bytes"),
];

pub fn fixture() -> TestResult<Value> {
    Ok(serde_json::from_str(CLEAN)?)
}

pub fn limits() -> TestResult<BudgetLimits> {
    Ok(serde_json::from_value(
        fixture()?["budget"]["limits"].clone(),
    )?)
}

pub fn entry(collection: &str, capabilities: &[&str]) -> TestResult<TruncationEntry> {
    Ok(TruncationEntry::new(
        collection,
        capabilities
            .iter()
            .map(|id| id.parse())
            .collect::<CoreResult<Vec<_>>>()?,
        Some(1),
        false,
        "fixture.output_limited".parse()?,
    )?)
}

pub fn state(entries: Vec<Value>) -> Value {
    json!({"status": "truncated", "entries": entries})
}

pub fn decoded_budget(truncation: &Value) -> TestResult<Budget> {
    let mut budget = fixture()?["budget"].clone();
    budget["truncation"] = truncation.clone();
    Ok(serde_json::from_value(budget)?)
}

pub fn negative(truncation: &TruncationState) -> TestResult<CoreResult<NegativeAuthorityDecision>> {
    let value = fixture()?;
    let context: GenerationContext = field(&value, "context")?;
    Ok(evaluate_negative_authority(
        context.context_id(),
        true,
        true,
        &field::<Vec<_>>(&value, "capability_summaries")?,
        &field::<Vec<_>>(&value, "coverage_records")?,
        &field::<Vec<_>>(&value, "conflicts")?,
        Vec::new(),
        None,
        truncation,
    ))
}

pub fn assert_error<T>(
    result: CoreResult<T>,
    code: CoreErrorCode,
    field: Option<&str>,
) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err(format!("expected {code:?} at {field:?}").into()),
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), field);
    error.validate()?;
    Ok(())
}

pub fn field<T: DeserializeOwned>(value: &Value, name: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(name).ok_or("missing fixture field")?.clone(),
    )?)
}
