use super::{CoreError, CoreErrorCode, CoreResult, validation_error};

// Error labels are metadata, not a channel for caller prose or host paths.
// Keep the operation namespace open while enforcing its canonical spelling.
pub(super) fn validate_metadata(error: &CoreError) -> CoreResult<()> {
    if !crate::MessageCode::parse(&error.operation_id).is_ok_and(|parsed| parsed.was_canonical()) {
        return Err(invalid("error.operation_id"));
    }
    if let Some(field) = &error.field_path
        && !schema_path(field)
    {
        return Err(invalid("error.field_path"));
    }
    if let Some(kind) = error.subject_kind.as_deref() {
        if !matches!(
            kind,
            "identifier"
                | "profile"
                | "context"
                | "handle"
                | "evidence"
                | "conflict"
                | "coverage"
                | "evaluation"
                | "finding"
                | "warning"
                | "budget"
                | "schema"
                | "envelope"
        ) {
            return Err(invalid("error.subject_kind"));
        }
        // Kind-only errors are intentional: the failing input may have no ID.
        if let Some(id) = error.subject_id.as_deref()
            && !subject_identifier(kind, id)
        {
            return Err(invalid("error.subject_id"));
        }
    } else if error.subject_id.is_some() {
        return Err(invalid("error.subject_kind"));
    }
    Ok(())
}

fn invalid(field: &'static str) -> CoreError {
    validation_error(
        "validate_operation_error",
        CoreErrorCode::ContractViolation,
        field,
    )
}

fn schema_path(value: &str) -> bool {
    value.split('.').all(|part| {
        let end = part.find('[').unwrap_or(part.len());
        let (name, mut suffix) = part.split_at(end);
        if !name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return false;
        }
        while let Some(index) = suffix.strip_prefix('[') {
            let Some((digits, tail)) = index.split_once(']') else {
                return false;
            };
            if !digits.bytes().all(|byte| byte.is_ascii_digit()) {
                return false;
            }
            suffix = tail;
        }
        suffix.is_empty()
    })
}

fn subject_identifier(kind: &str, value: &str) -> bool {
    match kind {
        "profile" => value.parse::<crate::ProfileId>().is_ok(),
        "context" => value.parse::<crate::GenerationContextId>().is_ok(),
        "handle" => value.parse::<crate::StableHandleId>().is_ok(),
        "evidence" => value.parse::<crate::EvidenceId>().is_ok(),
        "conflict" => value.parse::<crate::ConflictId>().is_ok(),
        "coverage" => value.parse::<crate::CoverageId>().is_ok(),
        "evaluation" => value.parse::<crate::NotEvaluatedId>().is_ok(),
        "finding" => value.parse::<crate::FindingId>().is_ok(),
        "warning" => value.parse::<crate::WarningId>().is_ok(),
        "schema" => value.parse::<crate::SchemaId>().is_ok(),
        // No separate budget/envelope ID family exists in E0. A caller may
        // identify the owning operation or another existing exact core subject.
        _ => canonical_identifier(value),
    }
}

pub(super) fn canonical_identifier(value: &str) -> bool {
    match value.split_once(':').map(|(family, _)| family) {
        Some("profile") => value.parse::<crate::ProfileId>().is_ok(),
        Some("schema") => value.parse::<crate::SchemaId>().is_ok(),
        Some("entity") => value.parse::<crate::EntityKey>().is_ok(),
        Some("partition") => value.parse::<crate::CoveragePartitionId>().is_ok(),
        Some("context") => value.parse::<crate::GenerationContextId>().is_ok(),
        Some("handle") => value.parse::<crate::StableHandleId>().is_ok(),
        Some("evidence") => value.parse::<crate::EvidenceId>().is_ok(),
        Some("conflict") => value.parse::<crate::ConflictId>().is_ok(),
        Some("coverage") => value.parse::<crate::CoverageId>().is_ok(),
        Some("not-evaluated") => value.parse::<crate::NotEvaluatedId>().is_ok(),
        Some("finding") => value.parse::<crate::FindingId>().is_ok(),
        Some("finding-fingerprint") => value.parse::<crate::FindingFingerprint>().is_ok(),
        Some("root-cause") => value.parse::<crate::RootCauseKey>().is_ok(),
        Some("warning") => value.parse::<crate::WarningId>().is_ok(),
        Some("generation") => {
            value.parse::<crate::ReferenceGenerationId>().is_ok()
                || value.parse::<crate::ProjectGenerationId>().is_ok()
                || value.parse::<crate::ExternalGenerationId>().is_ok()
        }
        None => value.parse::<crate::OperationId>().is_ok(),
        _ => false,
    }
}
