use super::support::*;
use serde_json::json;
use wow_core::{
    CapabilityAvailability, CoreErrorCode, CoverageStatus, CoverageTruncationRef, EvidenceId,
    GenerationContextId, NegativeAuthorityOutcome as Outcome, NegativeAuthorityReason as Reason,
    TruncationEntry, TruncationState, canonical_json_bytes, evaluate_negative_authority,
};

#[test]
fn negative_001_004_011_status_truth_table_is_not_vacuous() -> TestResult {
    let cases = [
        (
            CoverageStatus::Complete,
            Outcome::AuthoritativeAbsent,
            vec![],
        ),
        (
            CoverageStatus::Partial,
            Outcome::NotAuthoritative,
            vec![Reason::PartitionPartial],
        ),
        (
            CoverageStatus::Unknown,
            Outcome::NotAuthoritative,
            vec![Reason::PartitionUnknown],
        ),
        (
            CoverageStatus::Failed,
            Outcome::NotAuthoritative,
            vec![Reason::PartitionFailed],
        ),
        (
            CoverageStatus::NotApplicable,
            Outcome::NotApplicable,
            vec![],
        ),
    ];
    for (state, outcome, reasons) in cases {
        let records = [record("fixture.lookup", "a", state)?];
        let summaries = [summary(&records)?];
        let decision = negative(&summaries, &records, &[])?;
        assert_eq!(decision.outcome(), outcome);
        assert_eq!(decision.reasons(), reasons);
        assert_eq!(decision.context_id(), context()?);
        let encoded = serde_json::to_value(decision)?;
        assert_eq!(encoded["coverage_ids"], json!([records[0].coverage_id()]));
        assert_eq!(
            encoded["capability_ids"],
            json!([records[0].capability_id()])
        );
    }
    Ok(())
}

#[test]
fn negative_not_applicable_is_neutral_beside_applicable_coverage() -> TestResult {
    let records = [
        record("fixture.lookup", "a", CoverageStatus::Complete)?,
        record("fixture.other", "b", CoverageStatus::NotApplicable)?,
    ];
    let summaries = [summary(&records[..1])?, summary(&records[1..])?];
    assert_eq!(
        negative(&summaries, &records, &[])?.outcome(),
        Outcome::AuthoritativeAbsent
    );
    Ok(())
}

#[test]
fn negative_008_010_no_lookup_and_unknown_scope_are_distinct_reasons() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let summaries = [summary(&records)?];
    for (scope, lookup, expected) in [
        (false, true, vec![Reason::ScopeUnknown]),
        (true, false, vec![Reason::CapabilityNotEvaluated]),
        (
            false,
            false,
            vec![Reason::CapabilityNotEvaluated, Reason::ScopeUnknown],
        ),
    ] {
        let decision = evaluate_negative_authority(
            context()?,
            scope,
            lookup,
            &summaries,
            &records,
            &[],
            vec![],
            None,
            &TruncationState::NotTruncated,
        )?;
        assert_eq!(decision.outcome(), Outcome::NotAuthoritative);
        assert_eq!(decision.reasons(), expected);
    }
    Ok(())
}

#[test]
fn negative_007_candidates_deny_and_are_canonicalized_without_upgrade() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let summaries = [summary(&records)?];
    let a = EvidenceId::derive(&"candidate-a")?;
    let b = EvidenceId::derive(&"candidate-b")?;
    let decision = evaluate_negative_authority(
        context()?,
        true,
        true,
        &summaries,
        &records,
        &[],
        vec![b, a, b],
        None,
        &TruncationState::NotTruncated,
    )?;
    assert_eq!(decision.outcome(), Outcome::NotAuthoritative);
    assert_eq!(decision.reasons(), &[Reason::CandidateOnlyEvidence]);
    let value = serde_json::to_value(&decision)?;
    assert_eq!(value["candidate_evidence_ids"], json!([a.min(b), a.max(b)]));
    Ok(())
}

#[test]
fn negative_006_011_na_does_not_short_circuit_external_blockers() -> TestResult {
    for state in [CoverageStatus::Complete, CoverageStatus::NotApplicable] {
        let records = [record("fixture.lookup", "a", state)?];
        let summaries = [summary(&records)?];
        let truncation = wow_core::classify_truncation(vec![TruncationEntry::new(
            "findings",
            vec![records[0].capability_id().clone()],
            None,
            true,
            "fixture.truncated".parse()?,
        )?])?;
        let decision = evaluate_negative_authority(
            context()?,
            true,
            true,
            &summaries,
            &records,
            &[],
            vec![],
            None,
            &truncation,
        )?;
        assert_eq!(decision.outcome(), Outcome::NotAuthoritative);
        assert_eq!(decision.reasons(), &[Reason::ResultTruncated]);
        let candidate = evaluate_negative_authority(
            context()?,
            true,
            true,
            &summaries,
            &records,
            &[],
            vec![EvidenceId::derive(&"candidate")?],
            None,
            &TruncationState::NotTruncated,
        )?;
        assert_eq!(candidate.outcome(), Outcome::NotAuthoritative);
        assert_eq!(candidate.reasons(), &[Reason::CandidateOnlyEvidence]);
    }
    Ok(())
}

#[test]
fn negative_005_013_014_retains_every_partition_reason_and_exact_blocker() -> TestResult {
    let plain = [
        record("fixture.lookup", "a", CoverageStatus::Partial)?,
        record("fixture.lookup", "b", CoverageStatus::Unknown)?,
        record("fixture.lookup", "c", CoverageStatus::Failed)?,
    ];
    let conflict = conflict(&plain[..1])?;
    let records = [
        with_conflict(&plain[0], &conflict)?,
        plain[1].clone(),
        plain[2].clone(),
    ];
    let summaries = [summary(&records)?];
    let conflicts = [conflict];
    let CapabilityAvailability::NotEvaluated(evaluation) =
        availability(&summaries, &records, &conflicts)?
    else {
        return Err("required inputs unexpectedly runnable".into());
    };
    let truncation = wow_core::classify_truncation(vec![TruncationEntry::new(
        "findings",
        vec![],
        Some(2),
        false,
        "fixture.truncated".parse()?,
    )?])?;
    let decision = evaluate_negative_authority(
        context()?,
        false,
        false,
        &summaries,
        &records,
        &conflicts,
        vec![EvidenceId::derive(&"candidate")?],
        Some(&evaluation),
        &truncation,
    )?;
    assert_eq!(decision.outcome(), Outcome::NotAuthoritative);
    assert_eq!(
        decision.reasons(),
        &[
            Reason::PartitionPartial,
            Reason::PartitionUnknown,
            Reason::PartitionFailed,
            Reason::UnresolvedConflict,
            Reason::CapabilityNotEvaluated,
            Reason::CandidateOnlyEvidence,
            Reason::ScopeUnknown,
            Reason::ResultTruncated
        ]
    );
    let value = serde_json::to_value(&decision)?;
    let mut expected_ids = records
        .iter()
        .map(|record| record.coverage_id())
        .collect::<Vec<_>>();
    expected_ids.sort();
    assert_eq!(value["coverage_ids"], json!(expected_ids));
    assert_eq!(value["conflict_ids"], json!([conflicts[0].conflict_id()]));
    let reversed = [records[2].clone(), records[1].clone(), records[0].clone()];
    let repeated = evaluate_negative_authority(
        context()?,
        false,
        false,
        &summaries,
        &reversed,
        &conflicts,
        vec![EvidenceId::derive(&"candidate")?],
        Some(&evaluation),
        &truncation,
    )?;
    assert_eq!(
        canonical_json_bytes(&decision)?,
        canonical_json_bytes(&repeated)?
    );
    Ok(())
}

#[test]
fn negative_retains_summary_truncation_independently_of_outer_state() -> TestResult {
    let original = record("fixture.lookup", "a", CoverageStatus::Partial)?;
    let mut value = serde_json::to_value(original)?;
    value["truncation_refs"] = json!([CoverageTruncationRef::new(
        "files",
        "fixture.truncated".parse()?
    )?]);
    let records = [reseal_record(value)?];
    let decision = negative(&[summary(&records)?], &records, &[])?;
    assert_eq!(decision.outcome(), Outcome::NotAuthoritative);
    assert_eq!(
        decision.reasons(),
        &[Reason::PartitionPartial, Reason::ResultTruncated]
    );
    Ok(())
}

#[test]
fn negative_evaluation_cannot_bind_another_context_or_missing_coverage() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Partial)?];
    let summaries = [summary(&records)?];
    let CapabilityAvailability::NotEvaluated(evaluation) = availability(&summaries, &records, &[])?
    else {
        return Err("missing blocked evaluation".into());
    };
    let mut value = serde_json::to_value(&evaluation)?;
    value["context_id"] = serde_json::to_value(GenerationContextId::derive(&"foreign")?)?;
    let foreign: wow_core::NotEvaluatedRecord = serde_json::from_value(value.clone())?;
    value["not_evaluated_id"] = serde_json::to_value(wow_core::derive_not_evaluated_id(&foreign)?)?;
    let foreign: wow_core::NotEvaluatedRecord = serde_json::from_value(value)?;
    assert_code(
        evaluate_negative_authority(
            context()?,
            true,
            true,
            &summaries,
            &records,
            &[],
            vec![],
            Some(&foreign),
            &TruncationState::NotTruncated,
        ),
        CoreErrorCode::ResultContextViolation,
    )?;
    let mut value = serde_json::to_value(&evaluation)?;
    value["blocking_partitions"][0]["coverage_id"] =
        serde_json::to_value(wow_core::CoverageId::derive(&"missing")?)?;
    let missing: wow_core::NotEvaluatedRecord = serde_json::from_value(value.clone())?;
    value["not_evaluated_id"] = serde_json::to_value(wow_core::derive_not_evaluated_id(&missing)?)?;
    let missing: wow_core::NotEvaluatedRecord = serde_json::from_value(value)?;
    assert_code(
        evaluate_negative_authority(
            context()?,
            true,
            true,
            &summaries,
            &records,
            &[],
            vec![],
            Some(&missing),
            &TruncationState::NotTruncated,
        ),
        CoreErrorCode::CoverageRecordMissing,
    )
}
