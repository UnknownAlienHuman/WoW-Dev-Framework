use super::support::*;
use serde_json::{Value, json};
use wow_core::{
    BlockingPartitionRef, ConflictId, CoreErrorCode, CoverageRecord, CoverageStatus,
    GenerationContext, NegativeAuthorityOutcome, NotEvaluatedRecord, canonical_json_bytes,
};

fn rebuild(value: &Value) -> TestResult<wow_core::CoreResult<NotEvaluatedRecord>> {
    Ok(NotEvaluatedRecord::new(
        field(value, "context_id")?,
        field(value, "producer_id")?,
        field(value, "producer_version")?,
        field::<String>(value, "subject_kind")?,
        field::<String>(value, "subject_id")?,
        field(value, "reason_code")?,
        field(value, "blocking_capability_ids")?,
        field(value, "blocking_partitions")?,
        field(value, "conflict_ids")?,
    ))
}

#[test]
fn not_evaluated_join_001_compact_golden_conflicts_are_admitted_without_rewriting() -> TestResult {
    for index in [2, 3] {
        let value = fixture(index)?;
        let raw = &value["not_evaluated"][0];
        let evaluation: NotEvaluatedRecord = serde_json::from_value(raw.clone())?;
        evaluation.validate()?;
        assert_eq!(
            canonical_json_bytes(&evaluation)?,
            canonical_json_bytes(raw)?
        );
        assert_eq!(
            canonical_json_bytes(&rebuild(raw)??)?,
            canonical_json_bytes(raw)?
        );
        let decision = negative(&value, &evaluation)??;
        assert_eq!(
            decision.outcome(),
            NegativeAuthorityOutcome::NotAuthoritative
        );
        assert_eq!(
            serde_json::to_value(decision)?["conflict_ids"],
            raw["conflict_ids"]
        );
    }
    Ok(())
}

#[test]
fn not_evaluated_join_002_explicit_partition_conflicts_match_owner_truth() -> TestResult {
    let mut value = fixture(3)?;
    let coverage: CoverageRecord = serde_json::from_value(value["coverage_records"][3].clone())?;
    value["not_evaluated"][0]["blocking_partitions"][0] =
        serde_json::to_value(BlockingPartitionRef::from_record(&coverage))?;
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_eq!(
        evaluation.blocking_partitions()[0].conflict_ids(),
        coverage.conflict_ids()
    );
    assert_eq!(
        negative(&value, &evaluation)??.outcome(),
        NegativeAuthorityOutcome::NotAuthoritative
    );
    draft(&value)?.finalize()?.validate()?;
    Ok(())
}

#[test]
fn not_evaluated_join_003_compact_form_cannot_omit_the_enclosing_conflicts() -> TestResult {
    let mut value = fixture(3)?;
    value["not_evaluated"][0]["conflict_ids"] = json!([]);
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_error(
        negative(&value, &evaluation)?,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )?;
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )
}

#[test]
fn not_evaluated_join_004_nested_conflicts_must_be_retained_by_the_parent() -> TestResult {
    let mut value = fixture(3)?;
    value["not_evaluated"][0]["blocking_partitions"][0]["conflict_ids"] =
        json!([ConflictId::derive(&"not-retained")?]);
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_error(
        evaluation.validate(),
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.conflict_ids",
    )?;
    assert_error(
        rebuild(&value["not_evaluated"][0])?,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.conflict_ids",
    )?;
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.conflict_ids",
    )
}

#[test]
fn not_evaluated_join_005_one_coverage_id_cannot_describe_two_different_blockers() -> TestResult {
    let mut value = fixture(2)?;
    let mut extra = value["not_evaluated"][0]["blocking_partitions"][0].clone();
    extra["partition_id"] = json!("partition:restriction.fixture:another");
    value["not_evaluated"][0]["blocking_partitions"]
        .as_array_mut()
        .ok_or("blockers")?
        .push(extra);
    let mut sorted: Vec<BlockingPartitionRef> =
        field(&value["not_evaluated"][0], "blocking_partitions")?;
    sorted.sort();
    value["not_evaluated"][0]["blocking_partitions"] = serde_json::to_value(sorted)?;
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_error(
        evaluation.validate(),
        CoreErrorCode::DuplicateCoverageRecord,
        "blocking_partitions",
    )?;
    assert_error(
        rebuild(&value["not_evaluated"][0])?,
        CoreErrorCode::DuplicateCoverageRecord,
        "blocking_partitions",
    )?;
    reject_envelope(
        value,
        CoreErrorCode::DuplicateCoverageRecord,
        "blocking_partitions",
    )
}

#[test]
fn not_evaluated_join_006_blocker_capability_cannot_be_outside_declared_set() -> TestResult {
    let mut value = fixture(2)?;
    value["not_evaluated"][0]["blocking_capability_ids"] = json!(["fixture.unrelated"]);
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_error(
        evaluation.validate(),
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.capability_id",
    )?;
    assert_error(
        rebuild(&value["not_evaluated"][0])?,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.capability_id",
    )?;
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "blocking_partitions.capability_id",
    )
}

#[test]
fn not_evaluated_join_007_nonblocking_complete_partition_cannot_fake_a_blocker() -> TestResult {
    let mut value = fixture(0)?;
    let coverage: CoverageRecord = serde_json::from_value(value["coverage_records"][3].clone())?;
    let evaluation = NotEvaluatedRecord::new(
        coverage.context_id(),
        "fixture.evaluator".parse()?,
        "0.1.0".parse()?,
        "rule",
        "fixture.rule",
        "capability_unavailable".parse()?,
        vec![coverage.capability_id().clone()],
        vec![BlockingPartitionRef::from_record(&coverage)],
        Vec::new(),
    )?;
    // The constructor knows shape, not the retained owner's status. Both owner
    // consumers must reject this semantically false explanation of a blocker.
    assert_error(
        negative(&value, &evaluation)?,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )?;
    value["not_evaluated"] = json!([evaluation]);
    value["status"] = json!("partial");
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.blocking_partitions",
    )
}

#[test]
fn not_evaluated_join_008_unrelated_conflict_cannot_explain_another_capability() -> TestResult {
    let mut value = fixture(3)?;
    value["not_evaluated"][0]["blocking_capability_ids"] = json!(["restriction.facets.readable"]);
    value["not_evaluated"][0]["blocking_partitions"] = json!([]);
    let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
    assert_error(
        negative(&value, &evaluation)?,
        CoreErrorCode::CoverageConflict,
        "evaluation.conflict_ids",
    )?;
    reject_envelope(
        value,
        CoreErrorCode::CoverageConflict,
        "evaluation.conflict_ids",
    )
}

#[test]
fn not_evaluated_join_009_missing_and_misdescribed_blockers_return_owner_errors() -> TestResult {
    for key in ["coverage_id", "partition_id", "status"] {
        let mut value = fixture(2)?;
        let blocker = &mut value["not_evaluated"][0]["blocking_partitions"][0];
        blocker[key] = match key {
            "coverage_id" => json!(wow_core::CoverageId::derive(&"missing")?),
            "partition_id" => json!("partition:fixture:wrong"),
            _ => json!("failed"),
        };
        let evaluation = reseal_evaluation(&mut value["not_evaluated"][0])?;
        let code = if key == "coverage_id" {
            CoreErrorCode::CoverageRecordMissing
        } else {
            CoreErrorCode::CoverageConflict
        };
        assert_error(
            negative(&value, &evaluation)?,
            code,
            "evaluation.blocking_partitions",
        )?;
        reject_envelope(value, code, "evaluation.blocking_partitions")?;
    }
    Ok(())
}

#[test]
fn not_evaluated_join_010_duplicate_or_unordered_nested_conflict_sets_are_not_repaired()
-> TestResult {
    let mut raw = fixture(3)?["not_evaluated"][0].clone();
    let actual: ConflictId = serde_json::from_value(raw["conflict_ids"][0].clone())?;
    let other = ConflictId::derive(&"other-conflict")?;
    let mut conflicts = vec![actual, other];
    conflicts.sort();
    raw["conflict_ids"] = json!(conflicts);
    for entries in [vec![actual, actual], conflicts.into_iter().rev().collect()] {
        let mut value = raw.clone();
        value["blocking_partitions"][0]["conflict_ids"] = json!(entries);
        let evaluation = reseal_evaluation(&mut value)?;
        assert_error(
            evaluation.validate(),
            CoreErrorCode::DuplicateConflictReference,
            "blocking_partitions.conflict_ids",
        )?;
    }
    Ok(())
}

#[test]
fn not_evaluated_join_011_unknown_partition_and_required_nonapplicable_are_still_blocked()
-> TestResult {
    let mut value = fixture(0)?;
    let unknown = NotEvaluatedRecord::new(
        field::<GenerationContext>(&value, "context")?.context_id(),
        "fixture.evaluator".parse()?,
        "0.1.0".parse()?,
        "lane",
        "unmaterialized",
        "capability_unavailable".parse()?,
        vec!["fixture.missing".parse()?],
        Vec::new(),
        Vec::new(),
    )?;
    assert_eq!(
        negative(&value, &unknown)??.outcome(),
        NegativeAuthorityOutcome::NotAuthoritative
    );
    value["not_evaluated"] = json!([unknown]);
    value["status"] = json!("partial");
    draft(&value)?.finalize()?.validate()?;

    let na = record(
        &value,
        "fixture.nonapplicable",
        "unavailable",
        CoverageStatus::NotApplicable,
    )?;
    append_record(&mut value, &na)?;
    let evaluation = NotEvaluatedRecord::new(
        na.context_id(),
        "fixture.evaluator".parse()?,
        "0.1.0".parse()?,
        "rule",
        "fixture.rule",
        "capability_unavailable".parse()?,
        vec![na.capability_id().clone()],
        vec![BlockingPartitionRef::from_record(&na)],
        Vec::new(),
    )?;
    value["not_evaluated"] = json!([evaluation]);
    draft(&value)?.finalize()?.validate()?;
    Ok(())
}

#[test]
fn not_evaluated_join_012_valid_local_shape_never_waives_identity_checks() -> TestResult {
    let mut value = fixture(2)?;
    value["not_evaluated"][0]["not_evaluated_id"] = wow_core::NotEvaluatedId::derive(&"incorrect")?
        .to_string()
        .into();
    reject_envelope(
        value,
        CoreErrorCode::CanonicalDigestMismatch,
        "not_evaluated_id",
    )
}
