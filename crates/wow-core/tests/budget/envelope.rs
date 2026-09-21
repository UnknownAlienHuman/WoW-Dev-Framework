use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, E0CheckResultEnvelope, ResultStatus, canonical_json_bytes,
    canonical_result_order,
};

use super::support::*;

#[test]
fn finalize_001_budget_harness_preserves_all_committed_check_bytes() -> TestResult {
    for source in [
        CLEAN,
        include_str!("../../examples/e0-findings-result.json"),
        include_str!("../../examples/e0-not-evaluated-result.json"),
        include_str!("../../examples/e0-conflict-not-evaluated-result.json"),
    ] {
        let mut value: Value = serde_json::from_str(source)?;
        let golden = canonical_json_bytes(&value)?;
        assert_eq!(reseal(&mut value)?.canonical_bytes()?, golden);
        assert_eq!(draft(&value)?.finalize()?.canonical_bytes()?, golden);
    }
    Ok(())
}

#[test]
fn envelope_025_resealed_truncation_cannot_fake_partial_status() -> TestResult {
    let mut value = fixture()?;
    value["status"] = "partial".into();
    value["budget"]["truncation"] = state(vec![]);
    let decoded = reseal(&mut value)?;
    assert_error(
        decoded.validate(),
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    assert_error(
        decoded.canonical_bytes(),
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    assert_error(
        canonical_result_order(decoded),
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    assert_error(
        draft(&value)?.finalize(),
        CoreErrorCode::ContractViolation,
        Some("truncation.entries"),
    )?;
    Ok(())
}

#[test]
fn envelope_026_resealed_nested_omission_records_are_validated() -> TestResult {
    let valid = serde_json::to_value(entry("findings", &[])?)?;
    for (key, invalid, field) in [
        ("count_unknown", json!(true), "entries.omitted_count"),
        (
            "capability_ids",
            json!(["fixture.alpha", "fixture.alpha"]),
            "entries.capability_ids",
        ),
    ] {
        let mut record = valid.clone();
        record[key] = invalid;
        let mut value = fixture()?;
        value["status"] = "partial".into();
        value["budget"]["truncation"] = state(vec![record]);
        let decoded = reseal(&mut value)?;
        assert_error(
            decoded.validate(),
            CoreErrorCode::ContractViolation,
            Some(field),
        )?;
        assert_error(
            draft(&value)?.finalize(),
            CoreErrorCode::ContractViolation,
            Some(field),
        )?;
    }
    Ok(())
}

#[test]
fn truncation_005_valid_omissions_require_partial_and_retain_exact_bytes() -> TestResult {
    let mut value = fixture()?;
    let truncation = state(vec![serde_json::to_value(entry(
        "findings",
        &["wow.api.exists"],
    )?)?]);
    value["budget"]["truncation"] = truncation.clone();
    assert_error(
        draft(&value)?.finalize(),
        CoreErrorCode::ResultStatusViolation,
        Some("status"),
    )?;
    value["status"] = "partial".into();
    let admitted = draft(&value)?.finalize()?;
    assert_eq!(admitted.status(), ResultStatus::Partial);
    let wire = serde_json::to_value(&admitted)?;
    assert_eq!(wire["budget"]["truncation"], truncation);
    assert_eq!(
        wire["budget"]["usage"]["output_bytes"].as_u64(),
        Some(u64::try_from(admitted.canonical_bytes()?.len())?)
    );
    assert_eq!(
        canonical_result_order(admitted.clone())?.canonical_bytes()?,
        admitted.canonical_bytes()?
    );
    Ok(())
}

#[test]
fn truncation_004_collection_usage_cannot_hide_clipped_records() -> TestResult {
    let mut value: Value =
        serde_json::from_str(include_str!("../../examples/e0-findings-result.json"))?;
    value["findings"]
        .as_array_mut()
        .ok_or("findings")?
        .pop()
        .ok_or("finding")?;
    // Keep the retained producer count but independently reseal digest/bytes.
    let decoded = reseal(&mut value)?;
    assert_error(
        decoded.validate(),
        CoreErrorCode::BudgetInvalid,
        Some("budget.usage"),
    )?;
    // If a producer lies about both omissions and counts, core cannot recover
    // the original source inventory. That requires the producer/owner proof.
    Ok(())
}

#[test]
fn finalize_002_final_byte_limit_includes_omission_metadata_and_digest() -> TestResult {
    let mut value = fixture()?;
    value["status"] = "partial".into();
    value["budget"]["truncation"] = state(vec![serde_json::to_value(entry("findings", &[])?)?]);
    let admitted = draft(&value)?.finalize()?;
    let mut wire = serde_json::to_value(admitted)?;
    // Reducing the limit also changes serialized digits; converge on the exact
    // resulting size before testing the boundary. This is not fixture rewriting.
    for _ in 0..6 {
        let size = u64::try_from(reseal(&mut wire)?.canonical_bytes()?.len())?;
        if wire["budget"]["limits"]["max_output_bytes"].as_u64() == Some(size) {
            break;
        }
        wire["budget"]["limits"]["max_output_bytes"] = size.into();
    }
    let exact = draft(&wire)?.finalize()?;
    let count = u64::try_from(exact.canonical_bytes()?.len())?;
    assert_eq!(
        wire["budget"]["limits"]["max_output_bytes"].as_u64(),
        Some(count)
    );
    wire["budget"]["limits"]["max_output_bytes"] = (count - 1).into();
    assert_error(
        draft(&wire)?.finalize(),
        CoreErrorCode::BudgetExceeded,
        None,
    )?;
    let decoded: E0CheckResultEnvelope = reseal(&mut wire)?;
    assert_error(decoded.validate(), CoreErrorCode::BudgetExceeded, None)?;
    Ok(())
}
