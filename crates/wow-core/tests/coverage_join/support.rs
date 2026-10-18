use serde::de::DeserializeOwned;
use serde_json::Value;
use std::error::Error;
use wow_core::{
    CapabilitySummary, CoreErrorCode, CoreResult, CoverageId, CoverageRecord, CoverageStatus,
    GenerationContext, NegativeAuthorityDecision, NotEvaluatedId, NotEvaluatedRecord,
    TruncationState, combine_coverage, evaluate_negative_authority,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;
pub const GOLDENS: [&str; 4] = [
    include_str!("../../examples/e0-clean-result.json"),
    include_str!("../../examples/e0-findings-result.json"),
    include_str!("../../examples/e0-not-evaluated-result.json"),
    include_str!("../../examples/e0-conflict-not-evaluated-result.json"),
];

pub fn fixture(index: usize) -> TestResult<Value> {
    Ok(serde_json::from_str(
        GOLDENS.get(index).ok_or("fixture index")?,
    )?)
}

pub fn field<T: DeserializeOwned>(value: &Value, name: &str) -> TestResult<T> {
    Ok(serde_json::from_value(
        value.get(name).ok_or("fixture field")?.clone(),
    )?)
}

pub fn record(
    value: &Value,
    capability: &str,
    partition: &str,
    status: CoverageStatus,
) -> TestResult<CoverageRecord> {
    Ok(CoverageRecord::new(
        field::<GenerationContext>(value, "context")?.context_id(),
        capability.parse()?,
        format!("partition:fixture:{partition}").parse()?,
        status,
        "fixture.coverage".parse()?,
        "0.1.0".parse()?,
        if matches!(status, CoverageStatus::Partial | CoverageStatus::Unknown) {
            vec!["fixture:missing".into()]
        } else {
            Vec::new()
        },
        if status == CoverageStatus::Failed {
            Some("fixture.failed".parse()?)
        } else {
            None
        },
        Vec::new(),
        Vec::new(),
    )?)
}

pub fn append_record(value: &mut Value, record: &CoverageRecord) -> TestResult {
    value["coverage_records"]
        .as_array_mut()
        .ok_or("coverage array")?
        .push(serde_json::to_value(record)?);
    Ok(())
}

pub fn refresh_summaries(value: &mut Value) -> TestResult {
    let records: Vec<CoverageRecord> = field(value, "coverage_records")?;
    let summaries: Vec<CapabilitySummary> = field(value, "capability_summaries")?;
    let fresh = summaries
        .iter()
        .map(|summary| {
            let selected = records
                .iter()
                .filter(|record| record.capability_id() == summary.capability_id())
                .cloned()
                .collect::<Vec<_>>();
            combine_coverage(
                summary.context_id(),
                summary.capability_id().clone(),
                summary.producer_id().clone(),
                summary.producer_version().clone(),
                &selected,
            )
        })
        .collect::<CoreResult<Vec<_>>>()?;
    value["capability_summaries"] = serde_json::to_value(fresh)?;
    Ok(())
}

pub fn reseal_record(value: &mut Value) -> TestResult {
    let mut material = value.clone();
    material
        .as_object_mut()
        .ok_or("coverage object")?
        .remove("coverage_id");
    value["coverage_id"] = CoverageId::derive(&material)?.to_string().into();
    Ok(())
}

pub fn reseal_evaluation(value: &mut Value) -> TestResult<NotEvaluatedRecord> {
    let mut material = value.clone();
    material
        .as_object_mut()
        .ok_or("evaluation object")?
        .remove("not_evaluated_id");
    value["not_evaluated_id"] = NotEvaluatedId::derive(&material)?.to_string().into();
    Ok(serde_json::from_value(value.clone())?)
}

pub fn assert_error<T>(result: CoreResult<T>, code: CoreErrorCode, path: &str) -> TestResult {
    let Err(error) = result else {
        return Err(format!("expected {code:?} at {path}").into());
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(path));
    error.validate()?;
    Ok(())
}

pub fn negative(
    value: &Value,
    evaluation: &NotEvaluatedRecord,
) -> TestResult<CoreResult<NegativeAuthorityDecision>> {
    Ok(evaluate_negative_authority(
        field::<GenerationContext>(value, "context")?.context_id(),
        true,
        true,
        &field::<Vec<CapabilitySummary>>(value, "capability_summaries")?,
        &field::<Vec<CoverageRecord>>(value, "coverage_records")?,
        &field::<Vec<wow_core::ConflictRecord>>(value, "conflicts")?,
        Vec::new(),
        Some(evaluation),
        &TruncationState::NotTruncated,
    ))
}
