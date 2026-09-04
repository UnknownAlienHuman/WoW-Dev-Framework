use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::canonical::canonical_json_bytes;
use crate::error::{CoreError, CoreErrorCode, CoreResult, validation_error};

pub(crate) fn validate_source_handle(record: &crate::SourceHandle) -> CoreResult<()> {
    validate_domain_id(
        record,
        "handle_id",
        "wow-core/source-handle/e0-1",
        "handle:sha256:",
        "validate_source_handle",
    )
}

pub(crate) fn validate_evidence(record: &crate::EvidenceRecord) -> CoreResult<()> {
    validate_domain_id(
        record,
        "evidence_id",
        "wow-core/evidence/e0-1",
        "evidence:sha256:",
        "validate_evidence_record",
    )?;
    let value = object_value(record, "validate_evidence_record")?;
    let provenance = required_string(&value, "provenance", "validate_evidence_record")?;
    let confidence = required_string(&value, "confidence", "validate_evidence_record")?;
    if matches!(provenance, "semantic_candidate" | "model_inference") && confidence != "candidate" {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::EvidenceAuthorityViolation,
            "confidence",
        ));
    }
    let inputs = string_array(&value, "derivation_input_ids", "validate_evidence_record")?;
    if confidence == "derived" && inputs.is_empty() {
        return Err(validation_error(
            "validate_evidence_record",
            CoreErrorCode::DerivedEvidenceMissingInputs,
            "derivation_input_ids",
        ));
    }
    ensure_sorted_unique(&inputs, "validate_evidence_record", "derivation_input_ids")?;
    let handles = string_array(&value, "source_handle_ids", "validate_evidence_record")?;
    ensure_sorted_unique(&handles, "validate_evidence_record", "source_handle_ids")?;
    Ok(())
}

pub(crate) fn validate_conflict(record: &crate::ConflictRecord) -> CoreResult<()> {
    validate_domain_id(
        record,
        "conflict_id",
        "wow-core/conflict/e0-1",
        "conflict:sha256:",
        "validate_conflict_record",
    )?;
    let value = object_value(record, "validate_conflict_record")?;
    let evidence = string_array(&value, "evidence_ids", "validate_conflict_record")?;
    if evidence.len() < 2 {
        return Err(validation_error(
            "validate_conflict_record",
            CoreErrorCode::ConflictScopeEmpty,
            "evidence_ids",
        ));
    }
    ensure_sorted_unique(&evidence, "validate_conflict_record", "evidence_ids")?;
    let affected = value
        .get("affected_refs")
        .and_then(Value::as_array)
        .ok_or_else(|| contract_error("validate_conflict_record", "affected_refs"))?;
    if affected.is_empty() {
        return Err(validation_error(
            "validate_conflict_record",
            CoreErrorCode::ConflictScopeEmpty,
            "affected_refs",
        ));
    }
    Ok(())
}

pub(crate) fn validate_coverage(record: &crate::CoverageRecord) -> CoreResult<()> {
    validate_domain_id(
        record,
        "coverage_id",
        "wow-core/coverage/e0-1",
        "coverage:sha256:",
        "validate_coverage_record",
    )?;
    let value = object_value(record, "validate_coverage_record")?;
    let status = required_string(&value, "status", "validate_coverage_record")?;
    let missing = string_array(&value, "missing_input_ids", "validate_coverage_record")?;
    let conflicts = string_array(&value, "conflict_ids", "validate_coverage_record")?;
    let truncation = value
        .get("truncation_refs")
        .and_then(Value::as_array)
        .ok_or_else(|| contract_error("validate_coverage_record", "truncation_refs"))?;
    let failure = value.get("failure_code");
    ensure_sorted_unique(&missing, "validate_coverage_record", "missing_input_ids")?;
    ensure_sorted_unique(&conflicts, "validate_coverage_record", "conflict_ids")?;

    let valid = match status {
        "complete" => missing.is_empty() && truncation.is_empty() && failure.is_none(),
        "partial" => !missing.is_empty() || !truncation.is_empty() || !conflicts.is_empty(),
        "unknown" => !missing.is_empty() && failure.is_none(),
        "failed" => failure.and_then(Value::as_str).is_some(),
        "not_applicable" => {
            missing.is_empty() && conflicts.is_empty() && truncation.is_empty() && failure.is_none()
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(validation_error(
            "validate_coverage_record",
            CoreErrorCode::CoverageConflict,
            "status",
        ))
    }
}

pub(crate) fn validate_not_evaluated(record: &crate::NotEvaluatedRecord) -> CoreResult<()> {
    validate_domain_id(
        record,
        "not_evaluated_id",
        "wow-core/not-evaluated/e0-1",
        "not-evaluated:sha256:",
        "validate_not_evaluated_record",
    )?;
    let value = object_value(record, "validate_not_evaluated_record")?;
    let capabilities = string_array(
        &value,
        "blocking_capability_ids",
        "validate_not_evaluated_record",
    )?;
    if capabilities.is_empty() {
        return Err(validation_error(
            "validate_not_evaluated_record",
            CoreErrorCode::CoverageRecordMissing,
            "blocking_capability_ids",
        ));
    }
    ensure_sorted_unique(
        &capabilities,
        "validate_not_evaluated_record",
        "blocking_capability_ids",
    )?;
    Ok(())
}

pub(crate) fn validate_warning(record: &crate::WarningRecord) -> CoreResult<()> {
    validate_domain_id(
        record,
        "warning_id",
        "wow-core/warning/e0-1",
        "warning:sha256:",
        "validate_warning_record",
    )
}

pub(crate) fn validate_finding(record: &crate::Finding) -> CoreResult<()> {
    let value = object_value(record, "bind_finding_to_context")?;
    let supplied_fingerprint =
        required_string(&value, "fingerprint", "derive_finding_fingerprint")?;
    let supplied_finding_id = required_string(&value, "finding_id", "bind_finding_to_context")?;

    let arguments = value
        .get("message_arguments")
        .and_then(Value::as_array)
        .ok_or_else(|| contract_error("derive_finding_fingerprint", "message_arguments"))?;
    let identity_arguments = arguments
        .iter()
        .filter(|argument| {
            argument
                .get("identity_relevant")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .cloned()
        .collect::<Vec<_>>();

    let mut projection = Map::new();
    copy_required(
        &value,
        &mut projection,
        "finding_code",
        "derive_finding_fingerprint",
    )?;
    projection.insert(
        "identity_message_arguments".to_owned(),
        Value::Array(identity_arguments),
    );
    copy_required(
        &value,
        &mut projection,
        "primary_source_handle_id",
        "derive_finding_fingerprint",
    )?;
    copy_optional(&value, &mut projection, "root_cause_key");
    copy_required(
        &value,
        &mut projection,
        "rule_id",
        "derive_finding_fingerprint",
    )?;
    copy_required(
        &value,
        &mut projection,
        "rule_version",
        "derive_finding_fingerprint",
    )?;
    copy_optional(&value, &mut projection, "subject_entity_key");

    let expected_fingerprint = typed_domain_id(
        "wow-core/finding-fingerprint/e0-1",
        &Value::Object(projection),
        "finding-fingerprint:sha256:",
    )?;
    if supplied_fingerprint != expected_fingerprint {
        return Err(validation_error(
            "derive_finding_fingerprint",
            CoreErrorCode::CanonicalDigestMismatch,
            "fingerprint",
        ));
    }

    let mut binding = Map::new();
    copy_required(
        &value,
        &mut binding,
        "context_id",
        "bind_finding_to_context",
    )?;
    binding.insert(
        "finding_fingerprint".to_owned(),
        Value::String(supplied_fingerprint.to_owned()),
    );
    let expected_finding_id = typed_domain_id(
        "wow-core/finding/e0-1",
        &Value::Object(binding),
        "finding:sha256:",
    )?;
    if supplied_finding_id != expected_finding_id {
        return Err(validation_error(
            "bind_finding_to_context",
            CoreErrorCode::CanonicalDigestMismatch,
            "finding_id",
        ));
    }
    Ok(())
}

fn validate_domain_id<T: Serialize>(
    record: &T,
    id_field: &'static str,
    domain: &'static str,
    prefix: &'static str,
    operation: &'static str,
) -> CoreResult<()> {
    let mut value = object_value(record, operation)?;
    let supplied = required_string(&value, id_field, operation)?.to_owned();
    value
        .as_object_mut()
        .ok_or_else(|| contract_error(operation, id_field))?
        .remove(id_field);
    let expected = typed_domain_id(domain, &value, prefix)?;
    if supplied == expected {
        Ok(())
    } else {
        Err(validation_error(
            operation,
            CoreErrorCode::CanonicalDigestMismatch,
            id_field,
        ))
    }
}

fn typed_domain_id(domain: &str, value: &Value, prefix: &str) -> CoreResult<String> {
    let digest = crate::domain_separated_digest(domain, value)?;
    let mut output = String::with_capacity(prefix.len() + 64);
    output.push_str(prefix);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}")
            .map_err(|_| contract_error("derive_typed_digest_id", "digest"))?;
    }
    Ok(output)
}

fn object_value<T: Serialize>(record: &T, operation: &'static str) -> CoreResult<Value> {
    let value = serde_json::to_value(record).map_err(|error| {
        contract_error(operation, "record").with_argument("reason", error.to_string())
    })?;
    if value.is_object() {
        Ok(value)
    } else {
        Err(contract_error(operation, "record"))
    }
}

fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
    operation: &'static str,
) -> CoreResult<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| contract_error(operation, field))
}

fn string_array(
    value: &Value,
    field: &'static str,
    operation: &'static str,
) -> CoreResult<Vec<String>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| contract_error(operation, field))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| contract_error(operation, field))
        })
        .collect()
}

fn ensure_sorted_unique(
    values: &[String],
    operation: &'static str,
    field: &'static str,
) -> CoreResult<()> {
    for pair in values.windows(2) {
        if pair[0] >= pair[1] {
            return Err(validation_error(
                operation,
                CoreErrorCode::ResultDuplicateId,
                field,
            ));
        }
    }
    Ok(())
}

fn copy_required(
    source: &Value,
    target: &mut Map<String, Value>,
    field: &'static str,
    operation: &'static str,
) -> CoreResult<()> {
    let value = source
        .get(field)
        .cloned()
        .ok_or_else(|| contract_error(operation, field))?;
    target.insert(field.to_owned(), value);
    Ok(())
}

fn copy_optional(source: &Value, target: &mut Map<String, Value>, field: &'static str) {
    if let Some(value) = source.get(field) {
        target.insert(field.to_owned(), value.clone());
    }
}

fn contract_error(operation: &'static str, field: &'static str) -> CoreError {
    validation_error(operation, CoreErrorCode::ContractViolation, field)
}

#[allow(dead_code)]
fn canonical_bytes(value: &Value) -> CoreResult<Vec<u8>> {
    canonical_json_bytes(value)
}

#[allow(dead_code)]
fn unique_strings(values: &[String]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
