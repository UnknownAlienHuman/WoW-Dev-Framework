use super::support::*;
use serde_json::json;
use wow_core::{CoreErrorCode, CoverageStatus, GenerationContextId};

#[test]
fn coverage_validate_006_capability_013_missing_conflict_rejects() -> TestResult {
    let original = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let conflict = conflict(std::slice::from_ref(&original))?;
    let records = [with_conflict(&original, &conflict)?];
    let summaries = [summary(&records)?];
    assert_code(
        availability(&summaries, &records, &[]),
        CoreErrorCode::MissingConflictReference,
    )?;
    assert_code(
        negative(&summaries, &records, &[]),
        CoreErrorCode::MissingConflictReference,
    )
}

#[test]
fn coverage_validate_005_unrelated_conflict_cannot_be_assigned_to_partition() -> TestResult {
    let original = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let unrelated = record("fixture.other", "b", CoverageStatus::Complete)?;
    let conflict = conflict(&[unrelated])?;
    let records = [with_conflict(&original, &conflict)?];
    let summaries = [summary(&records)?];
    assert_code(
        availability(&summaries, &records, std::slice::from_ref(&conflict)),
        CoreErrorCode::CoverageConflict,
    )?;
    assert_code(
        negative(&summaries, &records, &[conflict]),
        CoreErrorCode::CoverageConflict,
    )
}

#[test]
fn coverage_affecting_conflict_cannot_disappear_from_raw_records_and_summary() -> TestResult {
    let records = [record("fixture.lookup", "a", CoverageStatus::Complete)?];
    let conflicts = [conflict(&records)?];
    let summaries = [summary(&records)?];
    assert_code(
        availability(&summaries, &records, &conflicts),
        CoreErrorCode::CoverageConflict,
    )?;
    assert_code(
        negative(&summaries, &records, &conflicts),
        CoreErrorCode::CoverageConflict,
    )
}

#[test]
fn conflict_context_and_duplicate_conflict_admission_are_checked() -> TestResult {
    let plain = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let original = conflict(std::slice::from_ref(&plain))?;
    let records = [with_conflict(&plain, &original)?];
    let summaries = [summary(&records)?];
    let duplicates = [original.clone(), original.clone()];
    assert_code(
        availability(&summaries, &records, &duplicates),
        CoreErrorCode::DuplicateConflictReference,
    )?;
    assert_code(
        negative(&summaries, &records, &duplicates),
        CoreErrorCode::DuplicateConflictReference,
    )?;
    let mut value = serde_json::to_value(&original)?;
    value["context_id"] = json!(GenerationContextId::derive(&"foreign-conflict-context")?);
    let foreign: wow_core::ConflictRecord = serde_json::from_value(value.clone())?;
    value["conflict_id"] = json!(wow_core::derive_conflict_id(&foreign)?);
    let foreign: wow_core::ConflictRecord = serde_json::from_value(value)?;
    assert_code(
        availability(&summaries, &records, std::slice::from_ref(&foreign)),
        CoreErrorCode::ConflictContextMismatch,
    )?;
    assert_code(
        negative(&summaries, &records, &[foreign]),
        CoreErrorCode::ConflictContextMismatch,
    )
}

#[test]
fn optional_conflicts_do_not_disable_unrelated_required_capabilities() -> TestResult {
    let required = record("fixture.lookup", "a", CoverageStatus::Complete)?;
    let optional = record("fixture.optional", "b", CoverageStatus::Complete)?;
    let conflict = conflict(std::slice::from_ref(&optional))?;
    let summaries = [summary(std::slice::from_ref(&required))?];
    let records = [required, with_conflict(&optional, &conflict)?];
    assert!(matches!(
        availability(&summaries, &records, std::slice::from_ref(&conflict))?,
        wow_core::CapabilityAvailability::Runnable
    ));
    assert_eq!(
        negative(&summaries, &records, &[conflict])?.outcome(),
        wow_core::NegativeAuthorityOutcome::AuthoritativeAbsent
    );
    Ok(())
}
