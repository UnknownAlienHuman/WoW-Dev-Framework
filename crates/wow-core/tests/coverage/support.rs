use std::error::Error;

use serde_json::Value;
use wow_core::{
    CapabilityAvailability, CapabilitySummary, ConflictAffectedRef, ConflictRecord, CoreErrorCode,
    CoreResult, CoverageRecord, CoverageStatus, GenerationContextId, NegativeAuthorityDecision,
    TruncationState, combine_coverage, derive_coverage_id, evaluate_capability_availability,
    evaluate_negative_authority,
};

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub fn context() -> TestResult<GenerationContextId> {
    Ok(GenerationContextId::derive(&"fixture-coverage-admission")?)
}

pub fn record(
    capability: &str,
    partition: &str,
    status: CoverageStatus,
) -> TestResult<CoverageRecord> {
    Ok(CoverageRecord::new(
        context()?,
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

pub fn summary(records: &[CoverageRecord]) -> TestResult<CapabilitySummary> {
    let first = records.first().ok_or("empty summary fixture")?;
    Ok(combine_coverage(
        first.context_id(),
        first.capability_id().clone(),
        "fixture.summary".parse()?,
        "0.1.0".parse()?,
        records,
    )?)
}

pub fn conflict(records: &[CoverageRecord]) -> TestResult<ConflictRecord> {
    Ok(ConflictRecord::new(
        context()?,
        "fixture.conflict".parse()?,
        vec![
            wow_core::EvidenceId::derive(&"left")?,
            wow_core::EvidenceId::derive(&"right")?,
        ],
        records
            .iter()
            .map(|record| {
                ConflictAffectedRef::new(
                    record.capability_id().clone(),
                    Some(record.partition_id().clone()),
                )
            })
            .collect(),
        None,
    )?)
}

pub fn with_conflict(
    record: &CoverageRecord,
    conflict: &ConflictRecord,
) -> TestResult<CoverageRecord> {
    Ok(CoverageRecord::new(
        record.context_id(),
        record.capability_id().clone(),
        record.partition_id().clone(),
        record.status(),
        record.producer_id().clone(),
        record.producer_version().clone(),
        record.missing_input_ids().to_vec(),
        record.failure_code().cloned(),
        vec![conflict.conflict_id()],
        record.truncation_refs().to_vec(),
    )?)
}

pub fn availability(
    summaries: &[CapabilitySummary],
    records: &[CoverageRecord],
    conflicts: &[ConflictRecord],
) -> CoreResult<CapabilityAvailability> {
    evaluate_capability_availability(
        GenerationContextId::derive(&"fixture-coverage-admission")?,
        "fixture.evaluator".parse()?,
        "0.1.0".parse()?,
        "rule",
        "fixture.rule",
        "fixture.not_evaluated".parse()?,
        summaries,
        records,
        conflicts,
    )
}

pub fn negative(
    summaries: &[CapabilitySummary],
    records: &[CoverageRecord],
    conflicts: &[ConflictRecord],
) -> CoreResult<NegativeAuthorityDecision> {
    evaluate_negative_authority(
        GenerationContextId::derive(&"fixture-coverage-admission")?,
        true,
        true,
        summaries,
        records,
        conflicts,
        Vec::new(),
        None,
        &TruncationState::NotTruncated,
    )
}

pub fn assert_code<T>(result: CoreResult<T>, code: CoreErrorCode) -> TestResult {
    let Err(error) = result else {
        return Err("invalid input was accepted".into());
    };
    assert_eq!(error.code(), code);
    Ok(())
}

// Recompute the public record identity after tampering. A digest mismatch must
// not mask a missing semantic check on decoded records.
pub fn reseal_record(value: Value) -> TestResult<CoverageRecord> {
    let decoded: CoverageRecord = serde_json::from_value(value.clone())?;
    let mut value = value;
    value["coverage_id"] = serde_json::to_value(derive_coverage_id(&decoded)?)?;
    Ok(serde_json::from_value(value)?)
}
