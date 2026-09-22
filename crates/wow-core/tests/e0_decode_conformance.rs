use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, E0CheckResultEnvelope, E0DecodeLimits, E0OperationErrorEnvelope,
    canonical_json_bytes,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;
const CLEAN: &str = include_str!("../examples/e0-clean-result.json");
const ERROR: &str = include_str!("../examples/e0-generation-mismatch-error.json");
const RESULTS: [&str; 4] = [
    CLEAN,
    include_str!("../examples/e0-findings-result.json"),
    include_str!("../examples/e0-not-evaluated-result.json"),
    include_str!("../examples/e0-conflict-not-evaluated-result.json"),
];

fn limits() -> Result<E0DecodeLimits, wow_core::CoreError> {
    E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)
}

fn check_error(input: &[u8], code: CoreErrorCode) -> TestResult {
    let error = E0CheckResultEnvelope::from_json_slice(input, limits()?)
        .err()
        .ok_or("unexpected successful check decoding")?;
    assert_eq!(error.code(), code);
    error.validate()?;
    Ok(())
}

#[test]
fn decode_001_all_golden_envelopes_retain_exact_canonical_bytes() -> TestResult {
    for input in RESULTS {
        let expected = canonical_json_bytes(&serde_json::from_str::<Value>(input)?)?;
        let result = E0CheckResultEnvelope::from_json_slice(input.as_bytes(), limits()?)?;
        assert_eq!(result.canonical_bytes()?, expected);
        assert_eq!(
            E0CheckResultEnvelope::from_json_slice(&expected, limits()?)?,
            result
        );
    }
    let expected = canonical_json_bytes(&serde_json::from_str::<Value>(ERROR)?)?;
    let result = E0OperationErrorEnvelope::from_json_slice(ERROR.as_bytes(), limits()?)?;
    assert_eq!(result.canonical_bytes()?, expected);
    assert_eq!(
        E0OperationErrorEnvelope::from_json_slice(&expected, limits()?)?,
        result
    );
    Ok(())
}

#[test]
fn decode_002_whitespace_key_order_and_equivalent_escapes_preserve_identity() -> TestResult {
    let expected = E0CheckResultEnvelope::from_json_slice(CLEAN.as_bytes(), limits()?)?;
    let value: Value = serde_json::from_str(CLEAN)?;
    let object = value.as_object().ok_or("fixture object")?;
    let entries = object
        .iter()
        .rev()
        .map(|(key, value)| {
            Ok(format!(
                "{}:{}",
                serde_json::to_string(key)?,
                serde_json::to_string(value)?
            ))
        })
        .collect::<Result<Vec<_>, serde_json::Error>>()?;
    let reversed = format!(" \r\n {{ {} }}\t", entries.join(",\n"));
    for input in [
        CLEAN.replace('\n', "\r\n"),
        reversed,
        CLEAN.replacen("\"operation_id\"", "\"operati\\u006fn_id\"", 1),
    ] {
        assert_eq!(
            E0CheckResultEnvelope::from_json_slice(input.as_bytes(), limits()?)?,
            expected
        );
    }
    Ok(())
}

#[test]
fn decode_003_duplicate_keys_reject_at_every_depth_before_projection() -> TestResult {
    let cases = [
        r#"{"a":1,"a":1}"#,
        r#"{"a":0,"a":1}"#,
        r#"{"a":{"b":0,"b":1}}"#,
        r#"{"a":[{"b":0,"b":1}]}"#,
        r#"{"a":[{"b":[{"x":0,"x":1}]}]}"#,
        r#"{"a":0,"\u0061":1}"#,
        r#"{"😀":0,"\ud83d\ude00":1}"#,
        r#"{"a\"b":0,"a\u0022b":1}"#,
    ];
    for input in cases {
        check_error(input.as_bytes(), CoreErrorCode::DuplicateField)?;
        let error = E0OperationErrorEnvelope::from_json_slice(input.as_bytes(), limits()?)
            .err()
            .ok_or("unexpected successful error decoding")?;
        assert_eq!(error.code(), CoreErrorCode::DuplicateField);
    }
    Ok(())
}

#[test]
fn decode_004_duplicate_optional_nulls_cannot_disappear() -> TestResult {
    // Serde's ordinary Option decoder treats this extra null as None. The
    // unchanged ID and digest therefore do not prove the raw JSON was lossless.
    let mut value: Value = serde_json::from_str(CLEAN)?;
    value["context"]["profile"]["edition_id"] = Value::Null;
    let input = serde_json::to_vec(&value)?;
    let structural: E0CheckResultEnvelope = serde_json::from_slice(&input)?;
    structural.validate()?;
    check_error(&input, CoreErrorCode::CanonicalizationFailure)?;
    // An independent real Option field in the error envelope demonstrates the
    // former bypass without depending on the selected profile's optional fields.
    let mut value: Value = serde_json::from_str(ERROR)?;
    value["error"]["subject_id"] = Value::Null;
    let input = serde_json::to_vec(&value)?;
    let structural: E0OperationErrorEnvelope = serde_json::from_slice(&input)?;
    structural.validate()?;
    let error = E0OperationErrorEnvelope::from_json_slice(&input, limits()?)
        .err()
        .ok_or("null was silently discarded")?;
    assert_eq!(error.code(), CoreErrorCode::CanonicalizationFailure);
    Ok(())
}

#[test]
fn decode_005_raw_numbers_reject_before_feature_dependent_normalization() -> TestResult {
    for token in [
        "-0",
        "-1",
        "+1",
        "01",
        "1.0",
        "1e0",
        "1E+0",
        "18446744073709551616",
    ] {
        let input = format!("{{\"x\":{token}}}");
        check_error(input.as_bytes(), CoreErrorCode::CanonicalizationFailure)?;
    }
    // Integer zero is a legitimate value, but lexical -0 is not. This applies
    // even when arbitrary_precision would normalize it during deserialization.
    let value: Value = serde_json::from_str(CLEAN)?;
    let compact = serde_json::to_string(&value)?;
    let input = compact.replacen("\"warnings\":0", "\"warnings\":-0", 1);
    assert_ne!(input, compact);
    check_error(input.as_bytes(), CoreErrorCode::CanonicalizationFailure)?;
    Ok(())
}

#[test]
fn decode_006_input_byte_limit_is_external_and_checked_first() -> TestResult {
    let exact = E0DecodeLimits::new(CLEAN.len(), 64, 100_000, 64 * 1024)?;
    E0CheckResultEnvelope::from_json_slice(CLEAN.as_bytes(), exact)?;
    let smaller = E0DecodeLimits::new(CLEAN.len() - 1, 64, 100_000, 64 * 1024)?;
    let error = E0CheckResultEnvelope::from_json_slice(CLEAN.as_bytes(), smaller)
        .err()
        .ok_or("input byte limit ignored")?;
    assert_eq!(error.code(), CoreErrorCode::BudgetExceeded);
    assert_eq!(error.field_path(), Some("input.bytes"));
    // The byte limit is applied even before UTF-8 validation.
    let tiny = E0DecodeLimits::new(1, 64, 100, 100)?;
    let error = E0OperationErrorEnvelope::from_json_slice(&[255, 255], tiny)
        .err()
        .ok_or("oversized invalid input accepted")?;
    assert_eq!(error.field_path(), Some("input.bytes"));
    Ok(())
}

#[test]
fn decode_007_token_limit_counts_keys_containers_and_scalars_exactly() -> TestResult {
    // Four tokens: object, key, array, integer. Grammar/schema are left to Serde.
    let input = br#"{"x":[0]}"#;
    for (tokens, code) in [
        (4, CoreErrorCode::ContractViolation),
        (3, CoreErrorCode::BudgetExceeded),
    ] {
        let limit = E0DecodeLimits::new(100, 64, tokens, 100)?;
        let error = E0CheckResultEnvelope::from_json_slice(input, limit)
            .err()
            .ok_or("not an envelope")?;
        assert_eq!(error.code(), code);
        if tokens == 3 {
            assert_eq!(error.field_path(), Some("input.tokens"));
        }
    }
    Ok(())
}

#[test]
fn decode_008_depth_limit_precedes_recursive_serde_work() -> TestResult {
    for depth in [1, 2, 16, 64] {
        let limit = E0DecodeLimits::new(1000, depth, 1000, 100)?;
        let at_limit = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        let error = E0CheckResultEnvelope::from_json_slice(at_limit.as_bytes(), limit)
            .err()
            .ok_or("not an envelope")?;
        assert_eq!(error.code(), CoreErrorCode::ContractViolation);
        let over_limit = format!("[{at_limit}]");
        let error = E0CheckResultEnvelope::from_json_slice(over_limit.as_bytes(), limit)
            .err()
            .ok_or("depth limit ignored")?;
        assert_eq!(error.code(), CoreErrorCode::BudgetExceeded);
        assert_eq!(error.field_path(), Some("input.nesting_depth"));
    }
    Ok(())
}

#[test]
fn decode_009_raw_string_limit_includes_escape_spelling_and_keys() -> TestResult {
    for input in [r#"{"x":"\u0061"}"#, r#"{"\u0061":0}"#] {
        for (size, code) in [
            (6, CoreErrorCode::ContractViolation),
            (5, CoreErrorCode::BudgetExceeded),
        ] {
            let limit = E0DecodeLimits::new(100, 64, 100, size)?;
            let error = E0CheckResultEnvelope::from_json_slice(input.as_bytes(), limit)
                .err()
                .ok_or("not an envelope")?;
            assert_eq!(error.code(), code);
            if size == 5 {
                assert_eq!(error.field_path(), Some("input.string_bytes"));
            }
        }
    }
    Ok(())
}

#[test]
fn decode_010_invalid_limits_reject_without_defaults_or_clamping() -> TestResult {
    let ceilings = [64 * 1024 * 1024, 64, 1_000_000, 1024 * 1024];
    E0DecodeLimits::new(ceilings[0], ceilings[1], ceilings[2], ceilings[3])?;
    for index in 0..4 {
        for bad in [0, ceilings[index] + 1, usize::MAX] {
            let mut values = ceilings;
            values[index] = bad;
            let error = E0DecodeLimits::new(values[0], values[1], values[2], values[3])
                .err()
                .ok_or("invalid limit accepted")?;
            assert_eq!(error.code(), CoreErrorCode::BudgetInvalid);
            error.validate()?;
        }
    }
    Ok(())
}

#[test]
fn decode_011_invalid_utf8_bom_syntax_and_trailing_documents_reject() -> TestResult {
    for input in [
        b"".as_slice(),
        b" ",
        b"\xef\xbb\xbf{}",
        b"\xff",
        br#"{"x":"\ud800"}"#,
        br#"{"x":"\udc00"}"#,
        br#"{"x":"\q"}"#,
        b"{]",
        b"[}",
        b"{",
        b"]",
        br#"{"x":truefalse}"#,
        br#"{"x":0,}"#,
        br#"{"x" 0}"#,
        b"{\"x\":\"unclosed",
    ] {
        check_error(input, CoreErrorCode::ContractViolation)?;
    }
    for trailing in ["{}", "true", "[]", "0"] {
        check_error(
            format!("{CLEAN}{trailing}").as_bytes(),
            CoreErrorCode::ContractViolation,
        )?;
    }
    Ok(())
}

#[test]
fn decode_012_unknown_fields_missing_fields_and_wrong_envelope_kinds_reject() -> TestResult {
    for pointer in ["", "/context/profile", "/budget", "/budget/truncation"] {
        let mut value: Value = serde_json::from_str(CLEAN)?;
        value.pointer_mut(pointer).ok_or("fixture path")?["unknown_member"] = json!(true);
        check_error(
            &serde_json::to_vec(&value)?,
            CoreErrorCode::ContractViolation,
        )?;
    }
    let mut value: Value = serde_json::from_str(CLEAN)?;
    value
        .as_object_mut()
        .ok_or("fixture object")?
        .remove("schema");
    check_error(
        &serde_json::to_vec(&value)?,
        CoreErrorCode::ContractViolation,
    )?;
    check_error(ERROR.as_bytes(), CoreErrorCode::ContractViolation)?;
    assert!(E0OperationErrorEnvelope::from_json_slice(CLEAN.as_bytes(), limits()?).is_err());
    Ok(())
}

#[test]
fn decode_013_semantic_schema_and_digest_errors_are_not_hidden() -> TestResult {
    let mut value: Value = serde_json::from_str(CLEAN)?;
    value["schema"]["version"] = json!("99.0.0");
    check_error(
        &serde_json::to_vec(&value)?,
        CoreErrorCode::SchemaVersionUnsupported,
    )?;
    let mut value: Value = serde_json::from_str(CLEAN)?;
    value["canonical_digest"] = json!(format!("sha256:{}", "00".repeat(32)));
    check_error(
        &serde_json::to_vec(&value)?,
        CoreErrorCode::CanonicalDigestMismatch,
    )?;
    let mut value: Value = serde_json::from_str(ERROR)?;
    value["canonical_digest"] = json!(format!("sha256:{}", "00".repeat(32)));
    let error = E0OperationErrorEnvelope::from_json_slice(&serde_json::to_vec(&value)?, limits()?)
        .err()
        .ok_or("invalid error digest accepted")?;
    assert_eq!(error.code(), CoreErrorCode::CanonicalDigestMismatch);
    Ok(())
}

#[test]
fn decode_014_rejection_never_echoes_caller_keys_values_or_parser_prose() -> TestResult {
    const SECRET: &str = "private-marker-ghp_123456789";
    for input in [
        format!("{{\"{SECRET}\":0,\"{SECRET}\":1}}"),
        format!("{{\"schema\":\"{SECRET}\"}}"),
        format!("{{\"{SECRET}\":\"\\q\"}}"),
    ] {
        let error = E0CheckResultEnvelope::from_json_slice(input.as_bytes(), limits()?)
            .err()
            .ok_or("invalid input accepted")?;
        error.validate()?;
        for rendered in [
            serde_json::to_string(&error)?,
            format!("{error}"),
            format!("{error:?}"),
        ] {
            assert!(!rendered.contains(SECRET));
        }
    }
    Ok(())
}

#[test]
fn decode_015_quoted_structure_and_independent_object_keys_do_not_collide() -> TestResult {
    for input in [
        r#"{"a":[{"x":0},{"x":1}]}"#,
        r#"{"a":"\"x\":1,{[]}\\","b":0}"#,
        r#"{"a":"😀é\ud83d\ude00","b":0}"#,
        r#"{"a":18446744073709551615,"b":true,"c":false}"#,
    ] {
        // Passing preflight reaches the concrete schema and fails there, not
        // at duplicate/numeric/budget admission. Also verify the syntax oracle.
        let _: Value = serde_json::from_str(input)?;
        check_error(input.as_bytes(), CoreErrorCode::ContractViolation)?;
    }
    Ok(())
}
