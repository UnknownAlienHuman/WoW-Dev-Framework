//! Error metadata admission, safe failure projection and schema-version decoding.
use std::error::Error;

use serde_json::{Value, json};
use wow_core::{
    CanonicalResult, ContentDigest, CoreError, CoreErrorCode, CoreResult, E0DecodeLimits,
    E0OperationErrorEnvelope, ErrorArgumentKind, ErrorCategory, RetryClass, SchemaVersionEntry,
    SourceContent, ToolVersion, canonical_json_bytes, domain_separated_digest,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const GOLDEN: &str = include_str!("../examples/e0-generation-mismatch-error.json");

fn base() -> CoreError {
    CoreError::new(
        CoreErrorCode::GenerationMismatch,
        ErrorCategory::Mismatch,
        "wow.check",
        RetryClass::AfterInputChange,
    )
}

fn finish(error: CoreError) -> CoreResult<E0OperationErrorEnvelope> {
    E0OperationErrorEnvelope::finalize(
        SchemaVersionEntry::new("schema:wow:operation-error".parse()?, "0.1.0".parse()?),
        "wow.check".parse()?,
        error,
    )
}

fn limits() -> CoreResult<E0DecodeLimits> {
    E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)
}

fn rejected<T>(result: CoreResult<T>, code: CoreErrorCode, field: &str) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(_) => return Err(format!("expected {code:?} at {field}").into()),
    };
    assert_eq!(error.code(), code);
    assert_eq!(error.field_path(), Some(field));
    error.validate()?;
    assert!(!serde_json::to_string(&error)?.contains("PRIVATE_MARKER"));
    Ok(())
}

fn reject_error(error: CoreError, field: &str) -> TestResult {
    rejected(error.validate(), CoreErrorCode::ContractViolation, field)?;
    let decoded: CoreError = serde_json::from_value(serde_json::to_value(&error)?)?;
    rejected(decoded.validate(), CoreErrorCode::ContractViolation, field)?;
    rejected(finish(error), CoreErrorCode::ContractViolation, field)
}

// Seal untyped data independently of error admission/finalization.
fn reseal(value: &mut Value) -> TestResult {
    let mut projection = value.clone();
    projection
        .as_object_mut()
        .ok_or("object")?
        .remove("canonical_digest");
    let hash = domain_separated_digest("wow-core/result/e0-1", &projection)?;
    value["canonical_digest"] = ContentDigest::<CanonicalResult>::from_bytes(hash)
        .to_string()
        .into();
    Ok(())
}

#[test]
fn error_001_committed_golden_is_unchanged_through_all_entrypoints() -> TestResult {
    let value: Value = serde_json::from_str(GOLDEN)?;
    let expected = canonical_json_bytes(&value)?;
    let error: CoreError = serde_json::from_value(value["error"].clone())?;
    error.validate()?;
    assert_eq!(finish(error)?.canonical_bytes()?, expected);
    let envelope = E0OperationErrorEnvelope::from_json_slice(GOLDEN.as_bytes(), limits()?)?;
    assert_eq!(envelope.canonical_bytes()?, expected);
    assert_eq!(
        E0OperationErrorEnvelope::from_json_slice(&expected, limits()?)?,
        envelope
    );
    Ok(())
}

#[test]
fn error_002_operations_are_metadata_not_paths_or_prose() -> TestResult {
    for name in [
        "wow.check",
        "validate_operation_error",
        "fixture.custom_operation",
    ] {
        let error = CoreError::new(
            CoreErrorCode::InvalidIdentifier,
            ErrorCategory::Validation,
            name,
            RetryClass::AfterInputChange,
        );
        error.validate()?;
        assert_eq!(error.operation_id(), name);
        assert_eq!(
            serde_json::to_value(finish(error)?)?["error"]["operation_id"],
            name
        );
    }
    for name in [
        "",
        "Wow.Check",
        " wow.check",
        "wow.check ",
        "wow..check",
        "check input",
        "/PRIVATE_MARKER",
        "C:\\PRIVATE_MARKER",
        "https://PRIVATE_MARKER",
        "wow.check\n",
    ] {
        reject_error(
            CoreError::new(
                CoreErrorCode::InvalidIdentifier,
                ErrorCategory::Validation,
                name,
                RetryClass::AfterInputChange,
            ),
            "error.operation_id",
        )?;
    }
    Ok(())
}

#[test]
fn error_003_field_paths_are_schema_coordinates() -> TestResult {
    for field in [
        "context",
        "context.project_generation",
        "findings[0].evidence_ids[]",
        "matrix[12][3].value",
        "items[].value",
    ] {
        let error = base().at_field(field);
        error.validate()?;
        assert_eq!(error.field_path(), Some(field));
    }
    for field in [
        "",
        ".value",
        "value.",
        "value..id",
        "items[x]",
        "items[-1]",
        "items[",
        "items[0]tail",
        "items[]]",
        "items.0",
        "/PRIVATE_MARKER",
        "C:/PRIVATE_MARKER",
        "context value",
        "context\n",
        "Context.value",
    ] {
        reject_error(base().at_field(field), "error.field_path")?;
    }
    base().at_field("a".repeat(512)).validate()?;
    reject_error(base().at_field("a".repeat(513)), "error.field_path")?;
    Ok(())
}

#[test]
fn error_004_subject_kind_without_id_is_valid_but_orphan_id_is_not() -> TestResult {
    for kind in [
        "identifier",
        "profile",
        "context",
        "handle",
        "evidence",
        "conflict",
        "coverage",
        "evaluation",
        "finding",
        "warning",
        "budget",
        "schema",
        "envelope",
    ] {
        let mut value = serde_json::to_value(base())?;
        value["subject_kind"] = kind.into();
        let error: CoreError = serde_json::from_value(value)?;
        error.validate()?;
        assert_eq!(
            serde_json::to_value(finish(error)?)?["error"]["subject_kind"],
            kind
        );
    }
    let mut value = serde_json::to_value(base())?;
    value["subject_id"] = format!("context:sha256:{}", "12".repeat(32)).into();
    reject_error(serde_json::from_value(value)?, "error.subject_kind")?;
    for kind in [
        "",
        "Context",
        "unregistered",
        "PRIVATE_MARKER/path",
        "context ",
    ] {
        let mut value = serde_json::to_value(base())?;
        value["subject_kind"] = kind.into();
        reject_error(serde_json::from_value(value)?, "error.subject_kind")?;
    }
    Ok(())
}

#[test]
fn error_005_subject_ids_match_their_declared_family() -> TestResult {
    for (kind, prefix) in [
        ("context", "context"),
        ("handle", "handle"),
        ("evidence", "evidence"),
        ("conflict", "conflict"),
        ("coverage", "coverage"),
        ("evaluation", "not-evaluated"),
        ("finding", "finding"),
        ("warning", "warning"),
    ] {
        let id = format!("{prefix}:sha256:{}", "12".repeat(32));
        let error = base().with_subject(kind, &id);
        error.validate()?;
        assert_eq!(
            serde_json::to_value(finish(error)?)?["error"]["subject_id"],
            id
        );
        reject_error(
            base().with_subject(kind, "profile:fixture:wrong-family"),
            "error.subject_id",
        )?;
        reject_error(
            base().with_subject(kind, format!("{prefix}:sha256:abcd")),
            "error.subject_id",
        )?;
    }
    for (kind, id) in [
        ("profile", "profile:fixture:test"),
        ("schema", "schema:wow:check-result"),
        ("identifier", "entity:api:Foo"),
        ("budget", "wow.check"),
        ("envelope", "wow.check"),
    ] {
        let error = base().with_subject(kind, id);
        error.validate()?;
        assert_eq!(
            serde_json::to_value(finish(error)?)?["error"]["subject_id"],
            id
        );
    }
    for id in [
        "",
        "PRIVATE_MARKER words",
        "C:/PRIVATE_MARKER",
        "profile:FIXTURE:test",
    ] {
        reject_error(base().with_subject("identifier", id), "error.subject_id")?;
    }
    Ok(())
}

#[test]
fn error_006_scalar_arguments_are_exact_and_not_coerced() -> TestResult {
    for (kind, values) in [
        (
            ErrorArgumentKind::Integer,
            vec!["0", "1", "18446744073709551615"],
        ),
        (ErrorArgumentKind::Boolean, vec!["true", "false"]),
        (
            ErrorArgumentKind::Text,
            vec!["not available", "точное значение"],
        ),
    ] {
        for value in values {
            let error = base().with_typed_argument("value", kind, value);
            error.validate()?;
            let wire = serde_json::to_value(finish(error)?)?;
            assert_eq!(wire["error"]["reason_arguments"][0]["value"], value);
        }
    }
    for (kind, values) in [
        (
            ErrorArgumentKind::Integer,
            vec!["-0", "01", "+1", "-1", "1.0", "1e1", "18446744073709551616"],
        ),
        (ErrorArgumentKind::Boolean, vec!["1", "True", " true"]),
        (
            ErrorArgumentKind::Digest,
            vec!["sha1:abcd", "sha256:abcd", "SHA256:abcd"],
        ),
    ] {
        for value in values {
            reject_error(
                base().with_typed_argument("value", kind, value),
                "error.reason_arguments.value",
            )?;
        }
    }
    let digest = format!("sha256:{}", "ab".repeat(32));
    base()
        .with_typed_argument("value", ErrorArgumentKind::Digest, &digest)
        .validate()?;
    reject_error(
        base().with_typed_argument("value", ErrorArgumentKind::Digest, digest.to_uppercase()),
        "error.reason_arguments.value",
    )?;
    Ok(())
}

#[test]
fn error_007_path_arguments_preserve_canonical_repository_spelling() -> TestResult {
    for path in [
        "Core/Init.lua",
        "é/файл.lua",
        "e\u{301}/file.lua",
        "Core/name:part.lua",
        "literal%2Fname.lua",
    ] {
        let error = base().with_typed_argument("path", ErrorArgumentKind::Path, path);
        error.validate()?;
        let wire = serde_json::to_value(finish(error)?)?;
        assert_eq!(wire["error"]["reason_arguments"][0]["value"], path);
    }
    Ok(())
}

#[test]
fn error_008_path_arguments_reject_normalization_and_drive_bypasses() -> TestResult {
    for path in [
        ".",
        "./Core/Init.lua",
        "Core//Init.lua",
        "Core\\Init.lua",
        "Core/./Init.lua",
        "Core/../Init.lua",
        "../Init.lua",
        "/PRIVATE_MARKER",
        "C:/PRIVATE_MARKER",
        "C:PRIVATE_MARKER",
        "./C:/PRIVATE_MARKER",
        ".//C:PRIVATE_MARKER",
        "\\\\server\\share",
        "//server/share",
        "file://PRIVATE_MARKER",
        "https://PRIVATE_MARKER",
        "Core/",
        "./C:\\PRIVATE_MARKER",
    ] {
        reject_error(
            base().with_typed_argument("path", ErrorArgumentKind::Path, path),
            "error.reason_arguments.value",
        )?;
    }
    Ok(())
}

#[test]
fn error_009_identifiers_use_the_existing_core_family_parsers() -> TestResult {
    let mut ids = vec![
        "wow.check".to_owned(),
        "profile:fixture:test".to_owned(),
        "schema:wow:check-result".to_owned(),
        "entity:api.function:Foo%2FBar".to_owned(),
        "partition:project.file:Core%2FInit.lua".to_owned(),
    ];
    for prefix in [
        "context",
        "handle",
        "evidence",
        "conflict",
        "coverage",
        "not-evaluated",
        "finding",
        "finding-fingerprint",
        "root-cause",
        "warning",
        "generation:reference",
        "generation:project",
        "generation:external:fixture",
    ] {
        ids.push(format!("{prefix}:sha256:{}", "34".repeat(32)));
    }
    for id in ids {
        let error = base().with_typed_argument("id", ErrorArgumentKind::Identifier, &id);
        error.validate()?;
        assert_eq!(
            serde_json::to_value(finish(error)?)?["error"]["reason_arguments"][0]["value"],
            id
        );
    }
    for id in [
        "words",
        "Wow.Check",
        "wow.latest",
        "profile:FIXTURE:test",
        "entity:api:A%2fB",
        "context:sha256:abcd",
        "/PRIVATE_MARKER",
        "C:/PRIVATE_MARKER",
        "https://PRIVATE_MARKER",
        "new:family",
    ] {
        reject_error(
            base().with_typed_argument("id", ErrorArgumentKind::Identifier, id),
            "error.reason_arguments.value",
        )?;
    }
    Ok(())
}

#[test]
fn error_010_argument_bounds_and_sensitive_names_remain_enforced() -> TestResult {
    base()
        .with_argument("a".repeat(63), "x".repeat(4096))
        .validate()?;
    reject_error(
        base().with_argument("a".repeat(64), "value"),
        "error.reason_arguments.name",
    )?;
    reject_error(
        base().with_argument("value", "x".repeat(4097)),
        "error.reason_arguments.value",
    )?;
    for name in [
        "",
        "Name",
        "access_token",
        "provider_access_token",
        "api_key",
        "password",
    ] {
        reject_error(
            base().with_argument(name, "PRIVATE_MARKER"),
            "error.reason_arguments.name",
        )?;
    }
    for value in [
        "",
        "value\n",
        "Bearer PRIVATE_MARKER",
        "ghp_PRIVATE_MARKER",
        "sk-PRIVATE_MARKER",
    ] {
        reject_error(
            base().with_argument("value", value),
            "error.reason_arguments.value",
        )?;
    }
    Ok(())
}

#[test]
fn error_011_argument_count_duplicates_and_wire_order_are_not_repaired() -> TestResult {
    let mut error = base();
    for n in 0..64 {
        error = error.with_argument(format!("arg_{n:02}"), "value");
    }
    error.validate()?;
    rejected(
        error.clone().with_argument("arg_64", "value").validate(),
        CoreErrorCode::BudgetExceeded,
        "error.reason_arguments",
    )?;
    let mut value = serde_json::to_value(&error)?;
    value["reason_arguments"]
        .as_array_mut()
        .ok_or("array")?
        .reverse();
    let decoded: CoreError = serde_json::from_value(value)?;
    rejected(
        decoded.validate(),
        CoreErrorCode::InvalidMessageArgument,
        "error.reason_arguments",
    )?;
    let duplicate = base()
        .with_argument("reason", "one")
        .with_argument("reason", "two");
    rejected(
        duplicate.validate(),
        CoreErrorCode::InvalidMessageArgument,
        "error.reason_arguments",
    )?;
    Ok(())
}

#[test]
fn error_012_cause_codes_are_ordered_unique_and_closed() -> TestResult {
    let error = base()
        .with_cause(CoreErrorCode::DigestMismatch)
        .with_cause(CoreErrorCode::InvalidIdentifier);
    error.validate()?;
    let mut value = serde_json::to_value(error)?;
    assert_eq!(
        value["cause_codes"],
        json!(["invalid_identifier", "digest_mismatch"])
    );
    value["cause_codes"] = json!(["digest_mismatch", "invalid_identifier"]);
    reject_error(serde_json::from_value(value.clone())?, "error.cause_codes")?;
    value["cause_codes"] = json!(["digest_mismatch", "digest_mismatch"]);
    reject_error(serde_json::from_value(value.clone())?, "error.cause_codes")?;
    value["cause_codes"] = json!(["PRIVATE_MARKER"]);
    assert!(serde_json::from_value::<CoreError>(value).is_err());
    Ok(())
}

#[test]
fn error_013_equivalent_construction_orders_have_identical_bytes() -> TestResult {
    let expected = finish(
        base()
            .with_argument("a", "first")
            .with_argument("b", "second")
            .with_cause(CoreErrorCode::InvalidIdentifier)
            .with_cause(CoreErrorCode::DigestMismatch),
    )?
    .canonical_bytes()?;
    let other = finish(
        base()
            .with_cause(CoreErrorCode::DigestMismatch)
            .with_argument("b", "second")
            .with_argument("a", "first")
            .with_cause(CoreErrorCode::InvalidIdentifier)
            .with_argument("a", "first"),
    )?
    .canonical_bytes()?;
    assert_eq!(other, expected);
    let value: Value = serde_json::from_slice(&other)?;
    assert_eq!(
        value["error"]["reason_arguments"],
        json!([
            {"name":"a", "kind":"text", "value":"first"},
            {"name":"b", "kind":"text", "value":"second"}
        ])
    );
    Ok(())
}

#[test]
fn error_014_digest_failures_do_not_echo_the_algorithm_candidate() -> TestResult {
    for algorithm in [
        "sha1".to_owned(),
        "Bearer PRIVATE_MARKER".to_owned(),
        "PRIVATE_MARKER\n".to_owned(),
        format!("PRIVATE_MARKER{}", "x".repeat(8192)),
    ] {
        let error = match ContentDigest::<SourceContent>::parse(&format!("{algorithm}:abcd")) {
            Err(error) => error,
            Ok(_) => return Err("unsupported digest accepted".into()),
        };
        assert_eq!(error.code(), CoreErrorCode::UnsupportedDigestAlgorithm);
        assert_eq!(error.field_path(), Some("candidate.algorithm"));
        error.validate()?;
        let wire = serde_json::to_value(&error)?;
        assert_eq!(
            wire["reason_arguments"],
            json!([
                {"name":"algorithm_length", "kind":"integer", "value":algorithm.len().to_string()}
            ])
        );
        assert!(!serde_json::to_string(&wire)?.contains("PRIVATE_MARKER"));
        finish(error)?.validate()?;
    }
    Ok(())
}

#[test]
fn error_015_digest_purpose_failure_keeps_only_known_labels() -> TestResult {
    let error = wow_core::digest::digest_purpose_mismatch("source_content", "canonical_result");
    error.validate()?;
    let wire = serde_json::to_value(error)?;
    assert_eq!(wire["reason_arguments"][0]["value"], "canonical_result");
    assert_eq!(wire["reason_arguments"][1]["value"], "source_content");
    let error =
        wow_core::digest::digest_purpose_mismatch("PRIVATE_MARKER", "Bearer PRIVATE_MARKER");
    error.validate()?;
    assert!(!serde_json::to_string(&error)?.contains("PRIVATE_MARKER"));
    assert_eq!(error.code(), CoreErrorCode::DigestPurposeMismatch);
    finish(error)?.validate()?;
    Ok(())
}

#[test]
fn error_016_resealed_raw_envelopes_cannot_hide_invalid_metadata() -> TestResult {
    let original: Value = serde_json::from_str(GOLDEN)?;
    for (field, bad, path) in [
        (
            "operation_id",
            json!("C:/PRIVATE_MARKER"),
            "error.operation_id",
        ),
        ("field_path", json!("../PRIVATE_MARKER"), "error.field_path"),
        (
            "subject_kind",
            json!("PRIVATE_MARKER"),
            "error.subject_kind",
        ),
        (
            "subject_id",
            json!("profile:fixture:wrong-family"),
            "error.subject_id",
        ),
        (
            "reason_arguments",
            json!([{"name":"path","kind":"path","value":"./C:/PRIVATE_MARKER"}]),
            "error.reason_arguments.value",
        ),
    ] {
        let mut value = original.clone();
        value["error"][field] = bad;
        reseal(&mut value)?;
        let envelope: E0OperationErrorEnvelope = serde_json::from_value(value.clone())?;
        rejected(envelope.validate(), CoreErrorCode::ContractViolation, path)?;
        rejected(
            E0OperationErrorEnvelope::from_json_slice(&serde_json::to_vec(&value)?, limits()?),
            CoreErrorCode::ContractViolation,
            path,
        )?;
    }
    Ok(())
}

#[test]
fn error_017_unknown_error_fields_and_variants_remain_structurally_rejected() -> TestResult {
    for (field, value) in [
        ("code", json!("not_evaluated")),
        ("category", json!("success")),
        ("retry_class", json!("automatically")),
        ("findings", json!([])),
        ("timestamp", json!(1)),
    ] {
        let mut wire = serde_json::to_value(base())?;
        wire[field] = value;
        assert!(serde_json::from_value::<CoreError>(wire).is_err());
    }
    Ok(())
}

#[test]
fn error_018_tool_versions_reuse_strict_parse_on_wire_input() -> TestResult {
    // This token is valid SemVer, but forbidden by the core identity policy.
    // An underscore here would only test the upstream syntax rejection.
    assert_eq!(
        semver::Version::parse("1.2.3+private-marker")?
            .build
            .as_str(),
        "private-marker"
    );
    for version in ["0.1.0", "1.2.3", "2.0.0-alpha.1"] {
        let parsed = ToolVersion::parse(version)?;
        let decoded: ToolVersion = serde_json::from_value(json!(version))?;
        assert_eq!(decoded, parsed);
        assert_eq!(serde_json::to_value(decoded)?, json!(version));
    }
    for version in [
        "1.2.3+private-marker",
        "01.2.3",
        "1.2",
        " 1.2.3",
        "1.2.3 ",
        "PRIVATE_MARKER",
    ] {
        assert!(ToolVersion::parse(version).is_err());
        assert!(serde_json::from_value::<ToolVersion>(json!(version)).is_err());
    }
    let mut wire: Value = serde_json::from_str(GOLDEN)?;
    wire["schema"]["version"] = "0.1.0+private-marker".into();
    rejected(
        E0OperationErrorEnvelope::from_json_slice(&serde_json::to_vec(&wire)?, limits()?),
        CoreErrorCode::ContractViolation,
        "input",
    )?;
    Ok(())
}

#[test]
fn error_019_semver_parse_failures_use_fixed_safe_reasons() -> TestResult {
    for input in [
        "PRIVATE_MARKER",
        "1.2.PRIVATE_MARKER",
        "Bearer PRIVATE_MARKER",
        "1.2.3\nPRIVATE_MARKER",
    ] {
        let error = match ToolVersion::parse(input) {
            Err(error) => error,
            Ok(_) => return Err("malformed version accepted".into()),
        };
        assert_eq!(error.code(), CoreErrorCode::InvalidIdentifier);
        error.validate()?;
        let wire = serde_json::to_value(&error)?;
        assert_eq!(
            wire["reason_arguments"][0]["value"],
            "invalid_semantic_version"
        );
        assert!(!serde_json::to_string(&error)?.contains("PRIVATE_MARKER"));
        finish(error)?.validate()?;
    }
    Ok(())
}
