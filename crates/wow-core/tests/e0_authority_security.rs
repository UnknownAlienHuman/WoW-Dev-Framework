use std::error::Error;
use std::fmt::Write as _;

use serde_json::{Map, Value};
use wow_core::{
    CoreError, CoreErrorCode, E0CheckResultEnvelope, E0OperationErrorEnvelope,
    domain_separated_digest,
};

#[test]
fn derived_evidence_cannot_depend_on_candidate_evidence() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    let (derived_index, input_id) = derived_record_and_first_input(&value)?;
    let input_index = evidence_index_by_id(&value, &input_id)?;

    {
        let records = evidence_records_mut(&mut value)?;
        let input = record_object_mut(records, input_index)?;
        input.insert(
            "provenance".to_owned(),
            Value::String("semantic_candidate".to_owned()),
        );
        input.insert(
            "confidence".to_owned(),
            Value::String("candidate".to_owned()),
        );
        input.insert(
            "claim_scope".to_owned(),
            Value::String("candidate_relation".to_owned()),
        );
        input.insert("derivation_input_ids".to_owned(), Value::Array(Vec::new()));
    }
    normalize_evidence_ids(&mut value)?;

    let records = evidence_records(&value)?;
    let derived = record_object(records, derived_index)?;
    assert_eq!(
        derived.get("confidence").and_then(Value::as_str),
        Some("derived")
    );

    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::EvidenceAuthorityViolation);
    Ok(())
}

#[test]
fn proven_evidence_cannot_hide_derivation_inputs() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    let (derived_index, _) = derived_record_and_first_input(&value)?;
    {
        let records = evidence_records_mut(&mut value)?;
        let derived = record_object_mut(records, derived_index)?;
        derived.insert("confidence".to_owned(), Value::String("proven".to_owned()));
    }
    normalize_evidence_ids(&mut value)?;

    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::EvidenceAuthorityViolation);
    Ok(())
}

#[test]
fn proven_runtime_probe_is_scenario_scoped() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    let records = evidence_records_mut(&mut value)?;
    let direct_index = records
        .iter()
        .position(|record| {
            record
                .get("derivation_input_ids")
                .and_then(Value::as_array)
                .is_some_and(Vec::is_empty)
        })
        .ok_or_else(|| std::io::Error::other("direct evidence fixture is missing"))?;
    {
        let record = record_object_mut(records, direct_index)?;
        record.insert(
            "provenance".to_owned(),
            Value::String("runtime_probe".to_owned()),
        );
        record.insert("confidence".to_owned(), Value::String("proven".to_owned()));
        record.insert(
            "claim_scope".to_owned(),
            Value::String("platform_contract".to_owned()),
        );
    }
    normalize_evidence_ids(&mut value)?;

    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::EvidenceAuthorityViolation);
    Ok(())
}

#[test]
fn source_handle_floating_revision_is_rejected_after_id_rederivation() -> Result<(), Box<dyn Error>>
{
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    mutate_first_source_handle(&mut value, |record| {
        record.insert("revision".to_owned(), Value::String("latest".to_owned()));
    })?;

    let envelope: E0CheckResultEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::InvalidSourceHandle);
    Ok(())
}

#[test]
fn source_handle_path_escape_is_rejected_after_id_rederivation() -> Result<(), Box<dyn Error>> {
    let mut value: Value =
        serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    mutate_first_source_handle(&mut value, |record| {
        record.insert(
            "path".to_owned(),
            Value::String("Addon/../Outside.lua".to_owned()),
        );
    })?;

    let error = match serde_json::from_value::<E0CheckResultEnvelope>(value) {
        Ok(_) => {
            return Err(std::io::Error::other(
                "path escape unexpectedly deserialized into a typed envelope",
            )
            .into());
        }
        Err(error) => error,
    };
    assert!(error.to_string().contains("PathEscape"));
    Ok(())
}

#[test]
fn error_reason_argument_cannot_be_a_credential_payload() -> Result<(), Box<dyn Error>> {
    let mut value: Value = serde_json::from_str(include_str!(
        "../examples/e0-generation-mismatch-error.json"
    ))?;
    let arguments = value
        .get_mut("error")
        .and_then(|error| error.get_mut("reason_arguments"))
        .and_then(Value::as_array_mut)
        .ok_or_else(|| std::io::Error::other("error reason arguments are missing"))?;
    let first = arguments
        .first()
        .cloned()
        .ok_or_else(|| std::io::Error::other("error reason arguments are empty"))?;
    let mut sensitive = first;
    let object = sensitive
        .as_object_mut()
        .ok_or_else(|| std::io::Error::other("reason argument is not an object"))?;
    object.insert("name".to_owned(), Value::String("access_token".to_owned()));
    object.insert("value".to_owned(), Value::String("synthetic".to_owned()));
    arguments.push(sensitive);
    arguments.sort_by(|left, right| {
        left.get("name")
            .and_then(Value::as_str)
            .cmp(&right.get("name").and_then(Value::as_str))
    });

    let envelope: E0OperationErrorEnvelope = serde_json::from_value(value)?;
    let error = require_error(envelope.validate())?;
    assert_eq!(error.code(), CoreErrorCode::ContractViolation);
    Ok(())
}

fn derived_record_and_first_input(value: &Value) -> Result<(usize, String), Box<dyn Error>> {
    let records = evidence_records(value)?;
    for (index, record) in records.iter().enumerate() {
        if record.get("confidence").and_then(Value::as_str) == Some("derived")
            && let Some(input) = record
                .get("derivation_input_ids")
                .and_then(Value::as_array)
                .and_then(|inputs| inputs.first())
                .and_then(Value::as_str)
        {
            return Ok((index, input.to_owned()));
        }
    }
    Err(std::io::Error::other("derived evidence fixture is missing").into())
}

fn evidence_index_by_id(value: &Value, id: &str) -> Result<usize, Box<dyn Error>> {
    evidence_records(value)?
        .iter()
        .position(|record| record.get("evidence_id").and_then(Value::as_str) == Some(id))
        .ok_or_else(|| std::io::Error::other("evidence input fixture is missing").into())
}

fn evidence_records(value: &Value) -> Result<&Vec<Value>, Box<dyn Error>> {
    value
        .get("evidence_records")
        .and_then(Value::as_array)
        .ok_or_else(|| std::io::Error::other("evidence records are missing").into())
}

fn evidence_records_mut(value: &mut Value) -> Result<&mut Vec<Value>, Box<dyn Error>> {
    value
        .get_mut("evidence_records")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| std::io::Error::other("evidence records are missing").into())
}

fn record_object(records: &[Value], index: usize) -> Result<&Map<String, Value>, Box<dyn Error>> {
    records
        .get(index)
        .and_then(Value::as_object)
        .ok_or_else(|| std::io::Error::other("evidence record is not an object").into())
}

fn record_object_mut(
    records: &mut [Value],
    index: usize,
) -> Result<&mut Map<String, Value>, Box<dyn Error>> {
    records
        .get_mut(index)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| std::io::Error::other("evidence record is not an object").into())
}

fn normalize_evidence_ids(value: &mut Value) -> Result<(), Box<dyn Error>> {
    let count = evidence_records(value)?.len();
    for _ in 0..=count {
        let mut changed = false;
        for index in 0..count {
            let replacement = {
                let records = evidence_records_mut(value)?;
                let record = record_object_mut(records, index)?;
                let old = record
                    .get("evidence_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| std::io::Error::other("evidence_id is missing"))?
                    .to_owned();
                let new = rederive_record_id(
                    record,
                    "evidence_id",
                    "wow-core/evidence/e0-1",
                    "evidence:sha256:",
                )?;
                (old != new).then_some((old, new))
            };
            if let Some((old, new)) = replacement {
                replace_string(value, &old, &new);
                changed = true;
            }
        }
        if !changed {
            return Ok(());
        }
    }
    Err(std::io::Error::other("evidence IDs did not converge").into())
}

fn mutate_first_source_handle(
    value: &mut Value,
    mutation: impl FnOnce(&mut Map<String, Value>),
) -> Result<(), Box<dyn Error>> {
    let handles = value
        .get_mut("source_handles")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| std::io::Error::other("source handles are missing"))?;
    let first = handles
        .first_mut()
        .and_then(Value::as_object_mut)
        .ok_or_else(|| std::io::Error::other("source handles are empty"))?;
    let old = first
        .get("handle_id")
        .and_then(Value::as_str)
        .ok_or_else(|| std::io::Error::other("handle_id is missing"))?
        .to_owned();
    mutation(first);
    let new = rederive_record_id(
        first,
        "handle_id",
        "wow-core/source-handle/e0-1",
        "handle:sha256:",
    )?;
    replace_string(value, &old, &new);
    Ok(())
}

fn rederive_record_id(
    object: &mut Map<String, Value>,
    id_field: &str,
    domain: &str,
    prefix: &str,
) -> Result<String, Box<dyn Error>> {
    object.remove(id_field);
    let digest = domain_separated_digest(domain, &Value::Object(object.clone()))?;
    let mut id = String::with_capacity(prefix.len() + 64);
    id.push_str(prefix);
    for byte in digest {
        write!(&mut id, "{byte:02x}")?;
    }
    object.insert(id_field.to_owned(), Value::String(id.clone()));
    Ok(id)
}

fn replace_string(value: &mut Value, old: &str, new: &str) {
    match value {
        Value::String(current) if current == old => *current = new.to_owned(),
        Value::Array(items) => {
            for item in items {
                replace_string(item, old, new);
            }
        }
        Value::Object(object) => {
            for item in object.values_mut() {
                replace_string(item, old, new);
            }
        }
        _ => {}
    }
}

fn require_error<T>(result: Result<T, CoreError>) -> Result<CoreError, Box<dyn Error>> {
    match result {
        Ok(_) => Err(std::io::Error::other("tampered value unexpectedly validated").into()),
        Err(error) => Ok(error),
    }
}
