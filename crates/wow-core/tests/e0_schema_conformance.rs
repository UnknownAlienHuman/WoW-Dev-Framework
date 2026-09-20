//! ENVELOPE-016/017: schema admission must not depend on attacker-supplied pins.

use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::Value;
use wow_core::{
    CanonicalResult, ContentDigest, CoreErrorCode, CoreResult, E0CheckResultDraft,
    E0CheckResultEnvelope, E0OperationErrorEnvelope, canonical_json_bytes, domain_separated_digest,
};

type TestResult = Result<(), Box<dyn Error>>;
const CHECK: &str = include_str!("../examples/e0-clean-result.json");
const ERROR: &str = include_str!("../examples/e0-generation-mismatch-error.json");

fn fixture_field<T: DeserializeOwned>(value: &Value, field: &str) -> Result<T, Box<dyn Error>> {
    let field = value
        .get(field)
        .ok_or_else(|| format!("missing fixture field {field}"))?;
    Ok(serde_json::from_value(field.clone())?)
}

fn draft(value: &Value) -> Result<E0CheckResultDraft, Box<dyn Error>> {
    let budget = value.get("budget").ok_or("missing fixture budget")?;
    Ok(E0CheckResultDraft::new(
        fixture_field(value, "schema")?,
        fixture_field(value, "operation_id")?,
        fixture_field(value, "context")?,
        fixture_field(value, "status")?,
        fixture_field(value, "coverage_records")?,
        fixture_field(value, "capability_summaries")?,
        fixture_field(value, "source_handles")?,
        fixture_field(value, "evidence_records")?,
        fixture_field(value, "conflicts")?,
        fixture_field(value, "findings")?,
        fixture_field(value, "not_evaluated")?,
        fixture_field(value, "warnings")?,
        fixture_field(budget, "limits")?,
        fixture_field(budget, "truncation")?,
    ))
}

// Construct a self-consistent wire mutation, not a merely stale digest. No
// production envelope finalizer or validator participates in this resealing.
fn reseal(value: &mut Value) -> TestResult {
    let has_budget = value.get("budget").is_some();
    for _ in 0..6 {
        let mut unsigned = value.clone();
        unsigned
            .as_object_mut()
            .ok_or("fixture is not an object")?
            .remove("canonical_digest");
        let digest = domain_separated_digest("wow-core/result/e0-1", &unsigned)?;
        value["canonical_digest"] = ContentDigest::<CanonicalResult>::from_bytes(digest)
            .to_string()
            .into();
        if !has_budget {
            return Ok(());
        }
        let size = u64::try_from(canonical_json_bytes(value)?.len())?;
        if value["budget"]["usage"]["output_bytes"].as_u64() == Some(size) {
            return Ok(());
        }
        value["budget"]["usage"]["output_bytes"] = size.into();
    }
    Err("fixture resealing did not converge".into())
}

fn assert_schema_error<T>(result: CoreResult<T>, field: &str) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err("unsupported envelope schema was accepted".into()),
    };
    assert_eq!(error.code(), CoreErrorCode::SchemaVersionUnsupported);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    Ok(())
}

#[test]
fn schema_harness_preserves_the_committed_golden_bytes() -> TestResult {
    for source in [CHECK, ERROR] {
        let mut value: Value = serde_json::from_str(source)?;
        let original = canonical_json_bytes(&value)?;
        reseal(&mut value)?;
        assert_eq!(canonical_json_bytes(&value)?, original);
    }
    let value: Value = serde_json::from_str(CHECK)?;
    assert_eq!(
        draft(&value)?.finalize()?.canonical_bytes()?,
        canonical_json_bytes(&value)?
    );
    let value: Value = serde_json::from_str(ERROR)?;
    let envelope = E0OperationErrorEnvelope::finalize(
        fixture_field(&value, "schema")?,
        fixture_field(&value, "operation_id")?,
        fixture_field(&value, "error")?,
    )?;
    assert_eq!(envelope.canonical_bytes()?, canonical_json_bytes(&value)?);
    Ok(())
}

#[test]
fn envelope_016_check_rejects_wrong_family_and_future_versions_even_when_resealed() -> TestResult {
    for (id, version) in [
        ("schema:wow:operation-error", "0.1.0"),
        ("schema:fixture:unrelated", "0.1.0"),
        ("schema:wow:check-result", "1.0.0"),
        ("schema:wow:check-result", "0.2.0"),
        ("schema:wow:check-result", "0.1.1"),
    ] {
        let mut value: Value = serde_json::from_str(CHECK)?;
        value["schema"]["schema_id"] = id.into();
        value["schema"]["version"] = version.into();
        reseal(&mut value)?;
        let decoded: E0CheckResultEnvelope = serde_json::from_value(value.clone())?;
        assert_schema_error(decoded.validate(), "schema")?;
        assert_schema_error(decoded.canonical_bytes(), "schema")?;
        assert_schema_error(draft(&value)?.finalize(), "schema")?;
    }
    Ok(())
}

#[test]
fn envelope_016_error_rejects_wrong_family_and_future_versions_even_when_resealed() -> TestResult {
    for (id, version) in [
        ("schema:wow:check-result", "0.1.0"),
        ("schema:fixture:unrelated", "0.1.0"),
        ("schema:wow:operation-error", "1.0.0"),
        ("schema:wow:operation-error", "0.2.0"),
        ("schema:wow:operation-error", "0.1.1"),
    ] {
        let mut value: Value = serde_json::from_str(ERROR)?;
        value["schema"]["schema_id"] = id.into();
        value["schema"]["version"] = version.into();
        reseal(&mut value)?;
        let decoded: E0OperationErrorEnvelope = serde_json::from_value(value.clone())?;
        assert_schema_error(decoded.validate(), "schema")?;
        assert_schema_error(decoded.canonical_bytes(), "schema")?;
        assert_schema_error(
            E0OperationErrorEnvelope::finalize(
                fixture_field(&value, "schema")?,
                fixture_field(&value, "operation_id")?,
                fixture_field(&value, "error")?,
            ),
            "schema",
        )?;
    }
    Ok(())
}

#[test]
fn envelope_017_rejects_a_resealed_unknown_canonicalization_profile() -> TestResult {
    for source in [CHECK, ERROR] {
        let mut value: Value = serde_json::from_str(source)?;
        value["canonicalization_version"] = "wow-core-json/unreviewed".into();
        reseal(&mut value)?;
        if source == CHECK {
            let decoded: E0CheckResultEnvelope = serde_json::from_value(value)?;
            assert_schema_error(decoded.validate(), "canonicalization_version")?;
        } else {
            let decoded: E0OperationErrorEnvelope = serde_json::from_value(value)?;
            assert_schema_error(decoded.validate(), "canonicalization_version")?;
        }
    }
    Ok(())
}
