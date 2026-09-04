use std::error::Error;

use serde_json::Value;
use wow_core::{
    BudgetLimits, BudgetUsage, CoreErrorCode, E0CheckResultEnvelope, MessageArgument,
    MessageArgumentKind, SchemaCompatibility, SchemaId, ToolVersion, TruncationEntry,
    accumulate_budget_usage, canonical_result_order, classify_truncation, validate_budget,
    validate_message_arguments, validate_schema_version,
};

#[test]
fn message_arguments_require_unique_canonical_order() -> Result<(), Box<dyn Error>> {
    let alpha = MessageArgument::new("alpha", MessageArgumentKind::Text, "value", true)?;
    let zeta = MessageArgument::new("zeta", MessageArgumentKind::Boolean, "true", false)?;

    validate_message_arguments(&[alpha.clone(), zeta.clone()])?;
    let error = require_error(
        validate_message_arguments(&[zeta, alpha]),
        "order must be rejected",
    )?;
    assert_eq!(error.code(), CoreErrorCode::InvalidMessageArgument);

    let float = require_error(
        MessageArgument::new("count", MessageArgumentKind::Integer, "1.5", true),
        "floating-point message argument must be rejected",
    )?;
    assert_eq!(float.code(), CoreErrorCode::InvalidMessageArgument);
    Ok(())
}

#[test]
fn budget_validation_and_usage_are_checked() -> Result<(), Box<dyn Error>> {
    let limits = valid_limits();
    validate_budget(&limits)?;

    let invalid = BudgetLimits {
        max_findings: 0,
        ..limits
    };
    let error = require_error(validate_budget(&invalid), "zero limit must be rejected")?;
    assert_eq!(error.code(), CoreErrorCode::BudgetInvalid);

    let left = BudgetUsage {
        findings: u64::MAX,
        ..BudgetUsage::default()
    };
    let right = BudgetUsage {
        findings: 1,
        ..BudgetUsage::default()
    };
    let error = require_error(
        accumulate_budget_usage(left, right),
        "usage overflow must fail",
    )?;
    assert_eq!(error.code(), CoreErrorCode::UsageOverflow);
    Ok(())
}

#[test]
fn truncation_never_uses_zero_as_unknown() -> Result<(), Box<dyn Error>> {
    let reason = "wow.core.output_limited".parse()?;
    let unknown = TruncationEntry::new("findings", Vec::new(), None, true, reason)?;
    let state = classify_truncation(vec![unknown])?;
    assert!(state.is_truncated());

    let invalid = require_error(
        TruncationEntry::new(
            "findings",
            Vec::new(),
            Some(0),
            true,
            "wow.core.output_limited".parse()?,
        ),
        "known and unknown omission counts are mutually exclusive",
    )?;
    assert_eq!(invalid.code(), CoreErrorCode::ContractViolation);
    Ok(())
}

#[test]
fn schema_compatibility_is_exact_major_and_not_future_minor() -> Result<(), Box<dyn Error>> {
    let schema: SchemaId = "schema:wow:check-result".parse()?;
    let supported: ToolVersion = "0.2.0".parse()?;
    assert_eq!(
        validate_schema_version(&schema, &"0.2.0".parse()?, &schema, &supported)?,
        SchemaCompatibility::ExactSupported
    );
    assert_eq!(
        validate_schema_version(&schema, &"0.1.0".parse()?, &schema, &supported)?,
        SchemaCompatibility::CompatibleSupported
    );
    let future = require_error(
        validate_schema_version(&schema, &"0.3.0".parse()?, &schema, &supported),
        "future minor version must be rejected",
    )?;
    assert_eq!(future.code(), CoreErrorCode::SchemaVersionUnsupported);
    Ok(())
}

#[test]
fn canonical_reordering_preserves_exact_clean_result() -> Result<(), Box<dyn Error>> {
    let envelope: E0CheckResultEnvelope =
        serde_json::from_str(include_str!("../examples/e0-clean-result.json"))?;
    let original = envelope.canonical_bytes()?;
    let reordered = canonical_result_order(envelope)?;
    assert_eq!(original, reordered.canonical_bytes()?);
    Ok(())
}

#[test]
fn canonical_digest_tampering_is_rejected() -> Result<(), Box<dyn Error>> {
    let mut value: Value = serde_json::from_str(include_str!("../examples/e0-clean-result.json"))?;
    set_string_field(
        &mut value,
        "canonical_digest",
        format!("sha256:{}", "0".repeat(64)),
    )?;
    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate(), "tampered canonical digest must fail")?;
    assert_eq!(error.code(), CoreErrorCode::CanonicalDigestMismatch);
    Ok(())
}

#[test]
fn complete_status_cannot_hide_not_evaluated() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-not-evaluated-result.json"))?;
    set_string_field(&mut value, "status", "complete".to_owned())?;
    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate(), "complete must not hide NotEvaluated")?;
    assert_eq!(error.code(), CoreErrorCode::ResultStatusViolation);
    Ok(())
}

#[test]
fn strict_envelope_rejects_unknown_and_duplicate_fields() -> Result<(), Box<dyn Error>> {
    let mut value: Value = serde_json::from_str(include_str!("../examples/e0-clean-result.json"))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| std::io::Error::other("fixture is not an object"))?;
    object.insert("timestamp".to_owned(), Value::String("volatile".to_owned()));
    assert!(serde_json::from_value::<E0CheckResultEnvelope>(value).is_err());

    let source = include_str!("../examples/e0-clean-result.json");
    let duplicate = source.replacen('{', "{\"status\":\"complete\",", 1);
    assert!(serde_json::from_str::<E0CheckResultEnvelope>(&duplicate).is_err());
    Ok(())
}

#[test]
fn presentation_field_inside_finding_is_rejected() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    let findings = value
        .get_mut("findings")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| std::io::Error::other("findings fixture is not an array"))?;
    let first = findings
        .first_mut()
        .and_then(Value::as_object_mut)
        .ok_or_else(|| std::io::Error::other("findings fixture is empty"))?;
    first.insert(
        "rendered_message".to_owned(),
        Value::String("presentation only".to_owned()),
    );
    assert!(serde_json::from_value::<E0CheckResultEnvelope>(value).is_err());
    Ok(())
}

fn set_string_field(
    value: &mut Value,
    field: &str,
    replacement: String,
) -> Result<(), Box<dyn Error>> {
    let object = value
        .as_object_mut()
        .ok_or_else(|| std::io::Error::other("fixture is not an object"))?;
    object.insert(field.to_owned(), Value::String(replacement));
    Ok(())
}

fn require_error<T>(
    result: Result<T, wow_core::CoreError>,
    message: &'static str,
) -> Result<wow_core::CoreError, Box<dyn Error>> {
    match result {
        Ok(_) => Err(std::io::Error::other(message).into()),
        Err(error) => Ok(error),
    }
}

const fn valid_limits() -> BudgetLimits {
    BudgetLimits {
        max_coverage_records: 256,
        max_capability_summaries: 128,
        max_source_handles: 256,
        max_evidence_records: 256,
        max_conflicts: 128,
        max_findings: 128,
        max_not_evaluated: 128,
        max_warnings: 128,
        max_output_bytes: 524_288,
    }
}
