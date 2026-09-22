use super::support::*;
use serde_json::json;
use wow_core::{
    CapabilityAvailability, CapabilitySummary, CoreErrorCode, CoverageRecord, CoverageStatus,
    GenerationContext, TruncationState, canonical_json_bytes, canonical_result_order,
    evaluate_capability_availability, evaluate_negative_authority,
};

#[test]
fn envelope_join_001_all_golden_bytes_and_both_evaluation_representations_are_preserved()
-> TestResult {
    for golden in GOLDENS {
        let value: serde_json::Value = serde_json::from_str(golden)?;
        let envelope: wow_core::E0CheckResultEnvelope = serde_json::from_str(golden)?;
        // Fixtures are pretty-printed; the contract requires compact canonical
        // output. Build the oracle from the raw fixture Value, not from a typed
        // envelope or the finalizer being tested: no field or array is repaired.
        let expected = canonical_json_bytes(&value)?;
        assert_eq!(envelope.canonical_bytes()?.as_slice(), expected.as_slice());
        assert_eq!(
            draft(&value)?.finalize()?.canonical_bytes()?.as_slice(),
            expected.as_slice()
        );
        assert_eq!(
            canonical_result_order(envelope)?
                .canonical_bytes()?
                .as_slice(),
            expected.as_slice()
        );
    }
    Ok(())
}

#[test]
fn envelope_join_002_summary_cannot_select_away_a_retained_worse_partition() -> TestResult {
    for state in [
        CoverageStatus::Partial,
        CoverageStatus::Unknown,
        CoverageStatus::Failed,
    ] {
        let mut value = fixture(0)?;
        let extra = record(&value, "reference.api.exact_lookup", "hidden", state)?;
        append_record(&mut value, &extra)?;
        value["status"] = json!("partial");
        reject_envelope(
            value.clone(),
            CoreErrorCode::CoverageConflict,
            "capability_summary",
        )?;
        refresh_summaries(&mut value)?;
        let result = draft(&value)?.finalize()?;
        result.validate()?;
        let rebuilt = serde_json::to_value(&result)?;
        assert_eq!(
            rebuilt["coverage_records"]
                .as_array()
                .ok_or("coverage")?
                .len(),
            6
        );
        assert_eq!(
            rebuilt["capability_summaries"][3]["partition_refs"]
                .as_array()
                .ok_or("partitions")?
                .len(),
            2
        );
    }
    Ok(())
}

#[test]
fn envelope_join_003_logical_duplicate_without_a_summary_is_still_rejected() -> TestResult {
    let mut value = fixture(0)?;
    let extra = record(&value, "fixture.extra", "a", CoverageStatus::Complete)?;
    append_record(&mut value, &extra)?;
    let mut different = serde_json::to_value(extra)?;
    different["producer_version"] = json!("0.2.0");
    reseal_record(&mut different)?;
    append_record(&mut value, &serde_json::from_value(different)?)?;
    reject_envelope(
        value,
        CoreErrorCode::DuplicateCoverageRecord,
        "coverage_records",
    )
}

#[test]
fn envelope_join_004_omitted_affecting_conflict_cannot_disappear_from_raw_coverage() -> TestResult {
    let mut value = fixture(3)?;
    value["not_evaluated"] = json!([]);
    value["coverage_records"][3]["conflict_ids"] = json!([]);
    reseal_record(&mut value["coverage_records"][3])?;
    refresh_summaries(&mut value)?;
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "coverage_records.conflict_ids",
    )
}

#[test]
fn envelope_join_005_summary_producer_versions_do_not_create_two_logical_owners() -> TestResult {
    let mut value = fixture(0)?;
    let mut summary = value["capability_summaries"][0].clone();
    summary["producer_version"] = json!("0.2.0");
    value["capability_summaries"]
        .as_array_mut()
        .ok_or("summaries")?
        .push(summary);
    reject_envelope(value, CoreErrorCode::DuplicateCoverageRecord, "summaries")
}

#[test]
fn envelope_join_006_independent_statement_and_summary_producers_remain_distinct() -> TestResult {
    let mut value = fixture(0)?;
    let mut other = value["coverage_records"][3].clone();
    other["producer_id"] = json!("fixture.independent");
    reseal_record(&mut other)?;
    append_record(&mut value, &serde_json::from_value(other)?)?;
    refresh_summaries(&mut value)?;
    let mut summary = value["capability_summaries"][3].clone();
    summary["producer_id"] = json!("fixture.summary");
    value["capability_summaries"]
        .as_array_mut()
        .ok_or("summaries")?
        .push(summary);
    let before = canonical_json_bytes(&value)?;
    let result = draft(&value)?.finalize()?;
    result.validate()?;
    assert_eq!(canonical_json_bytes(&value)?, before);
    let serialized = serde_json::to_value(result)?;
    assert_eq!(
        serialized["coverage_records"]
            .as_array()
            .ok_or("coverage")?
            .len(),
        6
    );
    assert_eq!(
        serialized["capability_summaries"]
            .as_array()
            .ok_or("summaries")?
            .len(),
        6
    );
    Ok(())
}

#[test]
fn envelope_join_007_optional_capability_does_not_widen_required_decision_scope() -> TestResult {
    let mut value = fixture(0)?;
    let optional = record(&value, "fixture.optional", "b", CoverageStatus::Failed)?;
    append_record(&mut value, &optional)?;
    value["status"] = json!("partial");
    draft(&value)?.finalize()?.validate()?;
    let context = field::<GenerationContext>(&value, "context")?.context_id();
    let records: Vec<CoverageRecord> = field(&value, "coverage_records")?;
    let summaries: Vec<CapabilitySummary> = field(&value, "capability_summaries")?;
    assert!(matches!(
        evaluate_capability_availability(
            context,
            "fixture.evaluator".parse()?,
            "0.1.0".parse()?,
            "rule",
            "fixture.rule",
            "capability_unavailable".parse()?,
            &summaries,
            &records,
            &[]
        )?,
        CapabilityAvailability::Runnable
    ));
    Ok(())
}

#[test]
fn envelope_join_008_empty_failed_envelope_is_not_an_empty_required_scope_success() -> TestResult {
    let mut value = fixture(0)?;
    for key in [
        "coverage_records",
        "capability_summaries",
        "source_handles",
        "evidence_records",
        "conflicts",
        "findings",
        "not_evaluated",
        "warnings",
    ] {
        value[key] = json!([]);
    }
    value["status"] = json!("failed");
    draft(&value)?.finalize()?.validate()?;
    let context = field::<GenerationContext>(&value, "context")?.context_id();
    assert_error(
        evaluate_negative_authority(
            context,
            true,
            true,
            &[],
            &[],
            &[],
            Vec::new(),
            None,
            &TruncationState::NotTruncated,
        ),
        CoreErrorCode::CoverageRecordMissing,
        "required_coverage",
    )?;
    assert_error(
        evaluate_capability_availability(
            context,
            "fixture.evaluator".parse()?,
            "0.1.0".parse()?,
            "rule",
            "fixture.rule",
            "capability_unavailable".parse()?,
            &[],
            &[],
            &[],
        ),
        CoreErrorCode::CoverageRecordMissing,
        "required_coverage",
    )
}

#[test]
fn envelope_join_009_missing_summary_record_cannot_pass_through_reordering() -> TestResult {
    let mut value = fixture(0)?;
    value["capability_summaries"][0]["partition_refs"][0]["coverage_id"] =
        wow_core::CoverageId::derive(&"missing")?.to_string().into();
    reject_envelope(
        value,
        CoreErrorCode::CoverageRecordMissing,
        "summaries.partition_refs",
    )
}

#[test]
fn envelope_join_010_shared_admission_keeps_64_permutations_byte_identical() -> TestResult {
    let original = fixture(3)?;
    let expected = canonical_json_bytes(&original)?;
    for seed in 0_u64..64 {
        let mut value = original.clone();
        let mut state = seed;
        for key in [
            "coverage_records",
            "capability_summaries",
            "source_handles",
            "evidence_records",
            "conflicts",
            "not_evaluated",
        ] {
            let items = value[key].as_array_mut().ok_or("collection")?;
            for index in (1..items.len()).rev() {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                let other = usize::try_from(state % u64::try_from(index + 1)?)?;
                items.swap(index, other);
            }
        }
        assert_eq!(
            draft(&value)?.finalize()?.canonical_bytes()?.as_slice(),
            expected.as_slice(),
            "seed {seed}"
        );
    }
    Ok(())
}
