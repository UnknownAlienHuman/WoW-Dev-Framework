use super::support::*;
use serde_json::json;
use wow_core::{
    CoreErrorCode, CoverageRecord, CoverageStatus, CoverageTruncationRef, GenerationContextId,
    canonical_json_bytes, combine_coverage,
};

#[test]
fn coverage_combine_001_009_all_125_status_triples_and_orders() -> TestResult {
    use CoverageStatus::{Complete, Failed, NotApplicable, Partial, Unknown};
    let states = [Complete, Partial, Unknown, Failed, NotApplicable];
    // Explicit truth-table rank, independent of the production precedence helper.
    for (a, left) in states.iter().copied().enumerate() {
        for (b, middle) in states.iter().copied().enumerate() {
            for (c, right) in states.iter().copied().enumerate() {
                let records = vec![
                    record("fixture.lookup", "a", left)?,
                    record("fixture.lookup", "b", middle)?,
                    record("fixture.lookup", "c", right)?,
                ];
                let applicable = [a, b, c].into_iter().filter(|rank| *rank < 4).max();
                let expected = applicable.map_or(NotApplicable, |rank| states[rank]);
                let result = summary(&records)?;
                assert_eq!(result.status(), expected, "case={a},{b},{c}");
                assert_eq!(result.partition_refs().len(), 3);
                let bytes = canonical_json_bytes(&result)?;
                for order in [
                    [0, 1, 2],
                    [0, 2, 1],
                    [1, 0, 2],
                    [1, 2, 0],
                    [2, 0, 1],
                    [2, 1, 0],
                ] {
                    let permuted = order.map(|index| records[index].clone());
                    assert_eq!(
                        canonical_json_bytes(&summary(&permuted)?)?,
                        bytes,
                        "case={a},{b},{c};order={order:?}"
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn coverage_combine_014_empty_is_not_complete() -> TestResult {
    assert_code(
        combine_coverage(
            context()?,
            "fixture.lookup".parse()?,
            "fixture.summary".parse()?,
            "0.1.0".parse()?,
            &[],
        ),
        CoreErrorCode::CoverageRecordMissing,
    )
}

#[test]
fn coverage_combine_011_duplicate_owner_keys_separated_by_digest_order() -> TestResult {
    let a = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let changed = record("fixture.lookup", "a", CoverageStatus::Partial)?;
    let low = a.coverage_id().min(changed.coverage_id());
    let high = a.coverage_id().max(changed.coverage_id());
    // Find an independent partition BETWEEN the two conflicting hashes. The
    // bounded deterministic search makes the old adjacent-window check fail.
    let middle = (0..4096)
        .map(|index| {
            record(
                "fixture.lookup",
                &format!("middle-{index}"),
                CoverageStatus::Complete,
            )
        })
        .collect::<TestResult<Vec<_>>>()?
        .into_iter()
        .find(|item| low < item.coverage_id() && item.coverage_id() < high)
        .ok_or("no deterministic interleaving fixture")?;
    let records = [a, middle, changed];
    assert_code(
        combine_coverage(
            context()?,
            "fixture.lookup".parse()?,
            "fixture.summary".parse()?,
            "0.1.0".parse()?,
            &records,
        ),
        CoreErrorCode::DuplicateCoverageRecord,
    )
}

#[test]
fn coverage_combine_distinct_producers_do_not_replace_each_other() -> TestResult {
    let complete = record("fixture.lookup", "same", CoverageStatus::Complete)?;
    let partial = CoverageRecord::new(
        context()?,
        complete.capability_id().clone(),
        complete.partition_id().clone(),
        CoverageStatus::Partial,
        "fixture.second_owner".parse()?,
        "0.1.0".parse()?,
        vec!["fixture:missing".into()],
        None,
        vec![],
        vec![],
    )?;
    let result = summary(&[complete, partial])?;
    assert_eq!(result.status(), CoverageStatus::Partial);
    assert_eq!(result.partition_refs().len(), 2);
    Ok(())
}

#[test]
fn coverage_validate_002_004_status_fields_cannot_contradict_coverage() -> TestResult {
    let cases = [
        (
            CoverageStatus::Complete,
            vec!["fixture:missing".into()],
            None,
            vec![],
            vec![],
        ),
        (CoverageStatus::Failed, vec![], None, vec![], vec![]),
        (
            CoverageStatus::NotApplicable,
            vec![],
            None,
            vec![wow_core::ConflictId::derive(&"conflict")?],
            vec![],
        ),
        (
            CoverageStatus::NotApplicable,
            vec![],
            None,
            vec![],
            vec![CoverageTruncationRef::new(
                "findings",
                "fixture.truncated".parse()?,
            )?],
        ),
        (CoverageStatus::Partial, vec![], None, vec![], vec![]),
        (CoverageStatus::Unknown, vec![], None, vec![], vec![]),
        (
            CoverageStatus::Complete,
            vec![],
            Some("fixture.failed".parse()?),
            vec![],
            vec![],
        ),
    ];
    for (status, missing, failure, conflicts, truncation) in cases {
        assert_code(
            CoverageRecord::new(
                context()?,
                "fixture.lookup".parse()?,
                "partition:fixture:a".parse()?,
                status,
                "fixture.coverage".parse()?,
                "0.1.0".parse()?,
                missing,
                failure,
                conflicts,
                truncation,
            ),
            CoreErrorCode::CoverageConflict,
        )?;
    }
    Ok(())
}

#[test]
fn coverage_validate_008_constructor_order_does_not_change_ids() -> TestResult {
    let a = CoverageTruncationRef::new("source", "fixture.source_limited".parse()?)?;
    let b = CoverageTruncationRef::new("files", "fixture.files_limited".parse()?)?;
    let ctx = context()?;
    let make = |missing, truncation| {
        CoverageRecord::new(
            ctx,
            "fixture.lookup".parse()?,
            "partition:fixture:a".parse()?,
            CoverageStatus::Partial,
            "fixture.coverage".parse()?,
            "0.1.0".parse()?,
            missing,
            None,
            vec![],
            truncation,
        )
    };
    let left = make(
        vec!["fixture:a".into(), "fixture:z".into()],
        vec![a.clone(), b.clone()],
    )?;
    let right = make(vec!["fixture:z".into(), "fixture:a".into()], vec![b, a])?;
    assert_eq!(left.coverage_id(), right.coverage_id());
    assert_eq!(canonical_json_bytes(&left)?, canonical_json_bytes(&right)?);
    Ok(())
}

#[test]
fn coverage_decoded_missing_inputs_follow_constructor_rules_even_when_resealed() -> TestResult {
    let original = record("fixture.lookup", "a", CoverageStatus::Partial)?;
    for invalid in [
        "".to_owned(),
        " leading".into(),
        "trailing ".into(),
        "bad\ninput".into(),
        "x".repeat(513),
    ] {
        let mut value = serde_json::to_value(&original)?;
        value["missing_input_ids"] = json!([invalid]);
        assert_code(
            reseal_record(value)?.validate(),
            CoreErrorCode::InvalidIdentifier,
        )?;
    }
    Ok(())
}

#[test]
fn coverage_decoded_truncation_collection_cannot_bypass_validation() -> TestResult {
    let original = record("fixture.lookup", "a", CoverageStatus::Partial)?;
    for (invalid, expected) in [
        ("", CoreErrorCode::IdentifierTooLong),
        ("../outside", CoreErrorCode::InvalidIdentifier),
        (" invalid", CoreErrorCode::InvalidIdentifier),
        ("bad\ncollection", CoreErrorCode::InvalidIdentifier),
    ] {
        let mut value = serde_json::to_value(&original)?;
        value["truncation_refs"] =
            json!([{"collection_id":invalid, "reason_code":"fixture.truncated"}]);
        assert_code(reseal_record(value)?.validate(), expected)?;
    }
    Ok(())
}

#[test]
fn coverage_validate_007_mixed_context_or_capability_rejects() -> TestResult {
    let one = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let mut other = serde_json::to_value(&one)?;
    other["context_id"] = serde_json::to_value(GenerationContextId::derive(&"another-context")?)?;
    let other = reseal_record(other)?;
    assert_code(
        combine_coverage(
            context()?,
            one.capability_id().clone(),
            "fixture.summary".parse()?,
            "0.1.0".parse()?,
            &[one.clone(), other],
        ),
        CoreErrorCode::CoverageContextMismatch,
    )?;
    let unrelated = record("fixture.unrelated", "b", CoverageStatus::Complete)?;
    assert_code(
        combine_coverage(
            context()?,
            one.capability_id().clone(),
            "fixture.summary".parse()?,
            "0.1.0".parse()?,
            &[one, unrelated],
        ),
        CoreErrorCode::CoverageConflict,
    )
}
