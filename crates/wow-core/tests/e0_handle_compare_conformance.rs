//! HANDLE-COMPARE-001..010 through both checked comparison entrypoints.

#[path = "handle/support.rs"]
mod support;

use serde_json::json;
use support::{TestResult, assert_error, fixture, rebuild, reseal};
use wow_core::{CoreErrorCode, SourceHandle, SourceHandleComparison, compare_source_handles};

fn compare_both(
    left: &SourceHandle,
    right: &SourceHandle,
    expected: SourceHandleComparison,
) -> TestResult {
    for (left, right) in [(left, right), (right, left)] {
        assert_eq!(left.compare(right)?, expected);
        assert_eq!(compare_source_handles(left, right)?, expected);
    }
    Ok(())
}

#[test]
fn handle_compare_001_identical_handles_and_round_trips() -> TestResult {
    let left = rebuild(&fixture()?)?;
    let right: SourceHandle = serde_json::from_slice(&serde_json::to_vec(&left)?)?;
    compare_both(&left, &right, SourceHandleComparison::Identical)
}

#[test]
fn handle_compare_002_all_span_states_remain_distinct() -> TestResult {
    let original = fixture()?;
    let mut handles = Vec::new();
    for span in [
        json!({"kind":"unknown"}),
        json!({"kind":"whole_file"}),
        json!({"kind":"byte_range","byte_start":0,"byte_end":0}),
        json!({"kind":"byte_range","byte_start":20,"byte_end":31}),
    ] {
        let mut value = original.clone();
        value["span"] = span;
        handles.push(rebuild(&value)?);
    }
    for (index, left) in handles.iter().enumerate() {
        for right in &handles[index + 1..] {
            compare_both(left, right, SourceHandleComparison::SameFileDifferentSpan)?;
        }
    }
    Ok(())
}

#[test]
fn handle_compare_003_and_handle_017_revision_difference_has_no_lineage_meaning() -> TestResult {
    let mut value = fixture()?;
    let original = rebuild(&value)?;
    value["revision"] = "fixture:another-revision".into();
    compare_both(
        &original,
        &rebuild(&value)?,
        SourceHandleComparison::SameOriginPathDifferentRevision,
    )
}

#[test]
fn handle_compare_004_different_content_is_not_identical() -> TestResult {
    let mut value = fixture()?;
    let original = rebuild(&value)?;
    value["content_digest"] = format!("sha256:{}", "04".repeat(32)).into();
    compare_both(
        &original,
        &rebuild(&value)?,
        SourceHandleComparison::SameOriginRevisionPathDifferentContent,
    )
}

#[test]
fn handle_compare_005_origin_path_and_entity_changes_are_unrelated() -> TestResult {
    let value = fixture()?;
    let original = rebuild(&value)?;
    for (key, changed) in [
        ("origin_kind", "generated_artifact"),
        ("origin_id", "fixture:elsewhere"),
        ("path", "Addon/core.lua"),
        ("entity_key", "entity:api:C_Fixture.Other"),
    ] {
        let mut value = value.clone();
        value[key] = changed.into();
        compare_both(
            &original,
            &rebuild(&value)?,
            SourceHandleComparison::Unrelated,
        )?;
    }
    Ok(())
}

#[test]
fn handle_compare_006_generation_changes_cannot_be_span_only_differences() -> TestResult {
    let value = fixture()?;
    let original = rebuild(&value)?;
    for (key, prefix) in [
        ("reference_generation", "generation:reference"),
        ("project_generation", "generation:project"),
    ] {
        let mut changed = value.clone();
        changed[key] = format!("{prefix}:sha256:{}", "ab".repeat(32)).into();
        changed["span"] = json!({"kind":"unknown"});
        compare_both(
            &original,
            &rebuild(&changed)?,
            SourceHandleComparison::Unrelated,
        )?;
    }
    Ok(())
}

#[test]
fn handle_compare_007_revision_then_content_classification_precedence() -> TestResult {
    let mut value = fixture()?;
    let original = rebuild(&value)?;
    value["content_digest"] = format!("sha256:{}", "09".repeat(32)).into();
    value["span"] = json!({"kind":"unknown"});
    compare_both(
        &original,
        &rebuild(&value)?,
        SourceHandleComparison::SameOriginRevisionPathDifferentContent,
    )?;
    value["revision"] = "fixture:another-revision".into();
    compare_both(
        &original,
        &rebuild(&value)?,
        SourceHandleComparison::SameOriginPathDifferentRevision,
    )
}

#[test]
fn handle_compare_008_009_both_positions_and_self_reject_forged_ids() -> TestResult {
    let mut value = fixture()?;
    let original = rebuild(&value)?;
    value["handle_id"] = format!("handle:sha256:{}", "00".repeat(32)).into();
    let invalid: SourceHandle = serde_json::from_value(value)?;
    for (left, right) in [
        (&invalid, &original),
        (&original, &invalid),
        (&invalid, &invalid),
    ] {
        assert_error(
            left.compare(right),
            CoreErrorCode::CanonicalDigestMismatch,
            "handle_id",
        )?;
        assert_error(
            compare_source_handles(left, right),
            CoreErrorCode::CanonicalDigestMismatch,
            "handle_id",
        )?;
    }
    Ok(())
}

#[test]
fn handle_compare_010_resealed_invalid_fields_cannot_yield_a_comparison() -> TestResult {
    let value = fixture()?;
    let original = rebuild(&value)?;
    for (key, changed, code, path) in [
        (
            "origin_kind",
            json!("repository"),
            CoreErrorCode::InvalidSourceHandle,
            "origin_kind",
        ),
        (
            "revision",
            json!("main"),
            CoreErrorCode::InvalidSourceHandle,
            "revision",
        ),
        (
            "origin_id",
            json!("PRIVATE_MARKER\n"),
            CoreErrorCode::InvalidSourceHandle,
            "origin_id",
        ),
        (
            "span",
            json!({"kind":"unknown","byte_start":0,"byte_end":1}),
            CoreErrorCode::SpanStateConflict,
            "span",
        ),
    ] {
        let mut changed_value = value.clone();
        changed_value[key] = changed;
        let invalid = reseal(changed_value)?;
        for (left, right) in [
            (&invalid, &original),
            (&original, &invalid),
            (&invalid, &invalid),
        ] {
            assert_error(left.compare(right), code, path)?;
            assert_error(compare_source_handles(left, right), code, path)?;
        }
    }
    Ok(())
}
