use super::support::*;
use serde_json::json;
use wow_core::{
    CapabilityAvailability, CapabilitySummary, CoreErrorCode, CoverageStatus, GenerationContextId,
    NegativeAuthorityOutcome, canonical_json_bytes, combine_coverage,
    evaluate_capability_availability,
};

#[test]
fn capability_001_005_complete_runs_all_other_required_states_block() -> TestResult {
    for state in [
        CoverageStatus::Complete,
        CoverageStatus::Partial,
        CoverageStatus::Unknown,
        CoverageStatus::Failed,
        CoverageStatus::NotApplicable,
    ] {
        let records = [record("fixture.lookup", "a", state)?];
        let summaries = [summary(&records)?];
        match availability(&summaries, &records, &[])? {
            CapabilityAvailability::Runnable => assert_eq!(state, CoverageStatus::Complete),
            CapabilityAvailability::NotEvaluated(blocked) => {
                assert_ne!(state, CoverageStatus::Complete);
                assert_eq!(
                    blocked.blocking_capability_ids(),
                    &[records[0].capability_id().clone()]
                );
                assert_eq!(blocked.blocking_partitions().len(), 1);
                assert_eq!(
                    blocked.blocking_partitions()[0].coverage_id(),
                    records[0].coverage_id()
                );
                assert_eq!(blocked.blocking_partitions()[0].status(), state);
                blocked.validate()?;
            }
        }
    }
    Ok(())
}

#[test]
fn capability_and_negative_015_empty_required_evidence_never_passes() -> TestResult {
    assert_code(
        availability(&[], &[], &[]),
        CoreErrorCode::CoverageRecordMissing,
    )?;
    assert_code(
        negative(&[], &[], &[]),
        CoreErrorCode::CoverageRecordMissing,
    )?;
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    assert_code(
        availability(&[], &records, &[]),
        CoreErrorCode::CoverageRecordMissing,
    )?;
    assert_code(
        negative(&[], &records, &[]),
        CoreErrorCode::CoverageRecordMissing,
    )
}

#[test]
fn capability_012_complete_summary_cannot_run_without_its_records() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let summaries = [summary(&records)?];
    assert_code(
        availability(&summaries, &[], &[]),
        CoreErrorCode::CoverageRecordMissing,
    )?;
    assert_code(
        negative(&summaries, &[], &[]),
        CoreErrorCode::CoverageRecordMissing,
    )
}

#[test]
fn capability_016_forged_complete_status_cannot_hide_partial_record() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Partial)?];
    let mut forged = serde_json::to_value(summary(&records)?)?;
    forged["status"] = json!("complete");
    let summaries: [CapabilitySummary; 1] = [serde_json::from_value(forged)?];
    assert_code(
        availability(&summaries, &records, &[]),
        CoreErrorCode::CoverageConflict,
    )?;
    assert_code(
        negative(&summaries, &records, &[]),
        CoreErrorCode::CoverageConflict,
    )
}

#[test]
fn capability_017_a_self_consistent_subset_cannot_omit_worse_raw_partition() -> TestResult {
    let complete = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let omitted = record("fixture.lookup", "b", CoverageStatus::Failed)?;
    let summaries = [summary(std::slice::from_ref(&complete))?];
    let records = [complete, omitted];
    assert_code(
        availability(&summaries, &records, &[]),
        CoreErrorCode::CoverageConflict,
    )?;
    assert_code(
        negative(&summaries, &records, &[]),
        CoreErrorCode::CoverageConflict,
    )
}

#[test]
fn capability_012_dangling_partition_id_rejects_before_a_decision() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let mut forged = serde_json::to_value(summary(&records)?)?;
    forged["partition_refs"][0]["coverage_id"] =
        serde_json::to_value(wow_core::CoverageId::derive(&"missing")?)?;
    let summaries = [serde_json::from_value::<CapabilitySummary>(forged)?];
    assert_code(
        availability(&summaries, &records, &[]),
        CoreErrorCode::CoverageRecordMissing,
    )?;
    assert_code(
        negative(&summaries, &records, &[]),
        CoreErrorCode::CoverageRecordMissing,
    )
}

#[test]
fn capability_009_negative_009_reject_mixed_contexts_even_for_complete_inputs() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let other_context = GenerationContextId::derive(&"different-context")?;
    let mut forged = serde_json::to_value(summary(&records)?)?;
    forged["context_id"] = serde_json::to_value(other_context)?;
    let summaries = [serde_json::from_value::<CapabilitySummary>(forged)?];
    assert_code(
        availability(&summaries, &records, &[]),
        CoreErrorCode::CoverageContextMismatch,
    )?;
    assert_code(
        negative(&summaries, &records, &[]),
        CoreErrorCode::CoverageContextMismatch,
    )?;
    let summaries = [summary(&records)?];
    let mut foreign = serde_json::to_value(&records[0])?;
    foreign["context_id"] = serde_json::to_value(other_context)?;
    let foreign = [reseal_record(foreign)?];
    assert_code(
        availability(&summaries, &foreign, &[]),
        CoreErrorCode::CoverageContextMismatch,
    )?;
    assert_code(
        negative(&summaries, &foreign, &[]),
        CoreErrorCode::CoverageContextMismatch,
    )
}

#[test]
fn capability_006_optional_partial_records_do_not_block_required_complete_coverage() -> TestResult {
    let required = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let optional = record("fixture.optional", "b", CoverageStatus::Partial)?;
    let summaries = [summary(std::slice::from_ref(&required))?];
    let records = [required, optional];
    assert!(matches!(
        availability(&summaries, &records, &[])?,
        CapabilityAvailability::Runnable
    ));
    assert_eq!(
        negative(&summaries, &records, &[])?.outcome(),
        NegativeAuthorityOutcome::AuthoritativeAbsent
    );
    Ok(())
}

#[test]
fn capability_008_shared_conflict_is_preserved_once_across_blocking_capabilities() -> TestResult {
    let a = record("fixture.alpha", "a", CoverageStatus::Complete)?;
    let b = record("fixture.beta", "b", CoverageStatus::Complete)?;
    let conflict = conflict(&[a.clone(), b.clone()])?;
    let records = [with_conflict(&a, &conflict)?, with_conflict(&b, &conflict)?];
    let summaries = [summary(&records[..1])?, summary(&records[1..])?];
    let conflicts = [conflict];
    let CapabilityAvailability::NotEvaluated(blocked) =
        availability(&summaries, &records, &conflicts)?
    else {
        return Err("affecting conflict produced Runnable".into());
    };
    assert_eq!(blocked.blocking_capability_ids().len(), 2);
    assert_eq!(blocked.blocking_partitions().len(), 2);
    assert_eq!(blocked.conflict_ids(), &[conflicts[0].conflict_id()]);
    for partition in blocked.blocking_partitions() {
        assert_eq!(partition.status(), CoverageStatus::Complete);
        assert_eq!(partition.conflict_ids(), &[conflicts[0].conflict_id()]);
    }
    blocked.validate()?;
    Ok(())
}

#[test]
fn capability_007_010_011_all_blockers_deterministic_and_producer_bound() -> TestResult {
    let records = [
        record("fixture.alpha", "a", CoverageStatus::Unknown)?,
        record("fixture.beta", "b", CoverageStatus::Partial)?,
    ];
    let summaries = [summary(&records[..1])?, summary(&records[1..])?];
    let CapabilityAvailability::NotEvaluated(blocked) = availability(&summaries, &records, &[])?
    else {
        return Err("missing blockers".into());
    };
    let reversed_summaries = [summaries[1].clone(), summaries[0].clone()];
    let reversed_records = [records[1].clone(), records[0].clone()];
    let CapabilityAvailability::NotEvaluated(reordered) =
        availability(&reversed_summaries, &reversed_records, &[])?
    else {
        return Err("missing reordered blockers".into());
    };
    assert_eq!(
        canonical_json_bytes(&blocked)?,
        canonical_json_bytes(&reordered)?
    );
    assert_eq!(blocked.blocking_capability_ids().len(), 2);
    let CapabilityAvailability::NotEvaluated(upgraded) = evaluate_capability_availability(
        context()?,
        "fixture.evaluator".parse()?,
        "0.2.0".parse()?,
        "rule",
        "fixture.rule",
        "fixture.not_evaluated".parse()?,
        &summaries,
        &records,
        &[],
    )?
    else {
        return Err("missing versioned blockers".into());
    };
    assert_ne!(blocked.not_evaluated_id(), upgraded.not_evaluated_id());
    Ok(())
}

#[test]
fn capability_duplicate_summaries_reject_but_distinct_summary_producers_aggregate() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Partial)?];
    let first = summary(&records)?;
    let duplicates = [first.clone(), first.clone()];
    assert_code(
        availability(&duplicates, &records, &[]),
        CoreErrorCode::DuplicateCoverageRecord,
    )?;
    assert_code(
        negative(&duplicates, &records, &[]),
        CoreErrorCode::DuplicateCoverageRecord,
    )?;
    let second = combine_coverage(
        context()?,
        records[0].capability_id().clone(),
        "fixture.another_summary".parse()?,
        "0.1.0".parse()?,
        &records,
    )?;
    let CapabilityAvailability::NotEvaluated(blocked) =
        availability(&[first, second], &records, &[])?
    else {
        return Err("missing shared blocker".into());
    };
    assert_eq!(blocked.blocking_capability_ids().len(), 1);
    assert_eq!(blocked.blocking_partitions().len(), 1);
    Ok(())
}

#[test]
fn capability_subject_is_validated_even_on_runnable_path() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let summaries = [summary(&records)?];
    for (kind, subject, expected) in [
        ("", "fixture.rule", CoreErrorCode::IdentifierTooLong),
        ("rule", "", CoreErrorCode::InvalidIdentifier),
        ("rule", "bad\nsubject", CoreErrorCode::InvalidIdentifier),
    ] {
        assert_code(
            evaluate_capability_availability(
                context()?,
                "fixture.evaluator".parse()?,
                "0.1.0".parse()?,
                kind,
                subject,
                "fixture.not_evaluated".parse()?,
                &summaries,
                &records,
                &[],
            ),
            expected,
        )?;
    }
    Ok(())
}
