use super::support::{TestResult, assert_error};
use serde_json::json;
use wow_core::{
    CoreErrorCode, MessageArgument, MessageArgumentKind, canonical_json_bytes,
    validate_message_arguments,
};

#[test]
fn message_arg_001_003_004_006_canonical_scalar_kinds_and_strict_wire() -> TestResult {
    for (kind, values) in [
        (
            MessageArgumentKind::Text,
            vec!["".to_owned(), "Русский text".to_owned(), "x".repeat(4096)],
        ),
        (
            MessageArgumentKind::Integer,
            vec![
                "0".to_owned(),
                "1".to_owned(),
                "9007199254740991".to_owned(),
            ],
        ),
        (
            MessageArgumentKind::Boolean,
            vec!["true".to_owned(), "false".to_owned()],
        ),
        (
            MessageArgumentKind::Identifier,
            vec!["C_Fixture.Call".to_owned()],
        ),
        (
            MessageArgumentKind::Digest,
            vec![format!("sha256:{}", "ab".repeat(32))],
        ),
    ] {
        for value in values {
            let arg = MessageArgument::new("value", kind, &value, true)?;
            validate_message_arguments(std::slice::from_ref(&arg))?;
            let decoded: MessageArgument = serde_json::from_slice(&canonical_json_bytes(&arg)?)?;
            assert_eq!(arg, decoded);
            assert_eq!(arg.value(), value);
        }
    }
    for value in [
        json!({"nested":1}),
        json!([1]),
        json!(1.5),
        json!(true),
        json!(null),
    ] {
        assert!(
            serde_json::from_value::<MessageArgument>(
                json!({"name":"value","kind":"integer","value":value,"identity_relevant":true})
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn message_arg_007_rejects_noncanonical_decimal_even_after_decoding() -> TestResult {
    for value in [
        "+1",
        "01",
        "000",
        "-0",
        "-1",
        "1.0",
        "1e1",
        "",
        " 1",
        "1 ",
        "9007199254740992",
        "18446744073709551616",
    ] {
        assert_error(
            MessageArgument::new("count", MessageArgumentKind::Integer, value, true),
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        )?;
        let arg: MessageArgument = serde_json::from_value(
            json!({"name":"count","kind":"integer","value":value,"identity_relevant":false}),
        )?;
        assert_error(
            validate_message_arguments(&[arg]),
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        )?;
    }
    Ok(())
}

#[test]
fn message_arg_008_paths_use_the_canonical_source_path_boundary() -> TestResult {
    for value in [
        "C:/file.lua",
        "C:file.lua",
        "./C:/file.lua",
        r"\\server\file.lua",
        r"a\b.lua",
        "/root/file.lua",
        "a/../b.lua",
        "./a.lua",
        "a//b.lua",
        "a/./b.lua",
        "",
        ".",
    ] {
        assert_error(
            MessageArgument::new("source", MessageArgumentKind::Path, value, true),
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        )?;
        let arg: MessageArgument = serde_json::from_value(
            json!({"name":"source","kind":"path","value":value,"identity_relevant":true}),
        )?;
        assert_error(
            validate_message_arguments(&[arg]),
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        )?;
    }
    for value in ["Addon/file..lua", "Addon/файл.lua", "a/b.lua"] {
        let arg = MessageArgument::new("source", MessageArgumentKind::Path, value, true)?;
        assert_eq!(arg.value(), value);
        validate_message_arguments(&[arg])?;
    }
    Ok(())
}

#[test]
fn message_arg_002_005_009_names_counts_order_and_value_bounds() -> TestResult {
    let arg = MessageArgument::new("alpha", MessageArgumentKind::Text, "a", false)?;
    let zeta = MessageArgument::new("zeta", MessageArgumentKind::Text, "z", false)?;
    for args in [vec![arg.clone(), arg.clone()], vec![zeta, arg]] {
        assert_error(
            validate_message_arguments(&args),
            CoreErrorCode::InvalidMessageArgument,
            "arguments",
        )?;
    }
    let args = (0..128)
        .map(|i| MessageArgument::new(format!("key{i:03}"), MessageArgumentKind::Text, "", true))
        .collect::<wow_core::CoreResult<Vec<_>>>()?;
    validate_message_arguments(&args)?;
    let mut extra = args;
    extra.push(MessageArgument::new(
        "zeta",
        MessageArgumentKind::Text,
        "",
        true,
    )?);
    assert_error(
        validate_message_arguments(&extra),
        CoreErrorCode::InvalidMessageArgument,
        "arguments",
    )?;
    for value in [
        "x".repeat(4097),
        "é".repeat(2049),
        "text\ncontrol".to_owned(),
    ] {
        assert_error(
            MessageArgument::new("text", MessageArgumentKind::Text, &value, false),
            CoreErrorCode::InvalidMessageArgument,
            "arguments.value",
        )?;
    }
    assert_error(
        MessageArgument::new("Bad", MessageArgumentKind::Text, "safe", true),
        CoreErrorCode::InvalidIdentifier,
        "arguments.name",
    )?;
    Ok(())
}
