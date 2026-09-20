//! Executable E0-A path/span cases; all inputs are synthetic and filesystem-free.

use std::error::Error;
use std::fmt::Debug;

use wow_core::{
    ContentDigest, CoreErrorCode, CoreResult, NormalizedSourcePath, SourceContent, SourceHandle,
    SourceHandleBuilder, SourceOriginKind, SourceSpan, SourceSpanKind, canonical_json_string,
};

type TestResult = Result<(), Box<dyn Error>>;

fn assert_error<T: Debug>(
    case: &str,
    result: CoreResult<T>,
    expected: CoreErrorCode,
    field: &str,
) -> TestResult {
    let error = match result {
        Err(error) => error,
        Ok(value) => return Err(format!("{case}: unexpectedly accepted {value:?}").into()),
    };
    assert_eq!(error.code(), expected, "{case}");
    assert_eq!(error.field_path(), Some(field), "{case}");
    error.validate()?;
    Ok(())
}

fn handle(path: &str, span: SourceSpan) -> CoreResult<SourceHandle> {
    SourceHandleBuilder::new(
        SourceOriginKind::Fixture,
        "fixture:e0-source-conformance",
        "fixture:e0-rev1",
        path,
        span,
        ContentDigest::<SourceContent>::from_bytes([7; 32]),
    )
    .build()
}

#[test]
fn path_001_004_015_018_020_023_normalize_and_round_trip() -> TestResult {
    let cases = [
        ("PATH-001", "Core/Init.lua", "Core/Init.lua", true),
        ("PATH-002", r"Core\Init.lua", "Core/Init.lua", false),
        ("PATH-003", "./Core//Init.lua", "Core/Init.lua", false),
        ("PATH-004", "Core/./Init.lua", "Core/Init.lua", false),
        ("PATH-015", "Core/Пример.lua", "Core/Пример.lua", true),
        ("PATH-018", "Core/Init.lua/", "Core/Init.lua", false),
        ("PATH-019", "Core/%2e%2e.lua", "Core/%2e%2e.lua", true),
        ("PATH-020", "Core/part:name.lua", "Core/part:name.lua", true),
    ];
    for (case, input, expected, was_canonical) in cases {
        let parsed = NormalizedSourcePath::parse(input)?;
        assert_eq!(parsed.value().as_str(), expected, "{case}");
        assert_eq!(parsed.was_canonical(), was_canonical, "{case}");
        let again = NormalizedSourcePath::parse(parsed.value().as_str())?;
        assert!(again.was_canonical(), "PATH-023 / {case}");
        assert_eq!(again.value(), parsed.value(), "PATH-023 / {case}");
        let json = serde_json::to_string(parsed.value())?;
        let decoded: NormalizedSourcePath = serde_json::from_str(&json)?;
        assert_eq!(&decoded, parsed.value(), "PATH-023 / {case}");
        if !was_canonical {
            assert!(input.parse::<NormalizedSourcePath>().is_err(), "{case}");
            let json = serde_json::to_string(input)?;
            assert!(
                serde_json::from_str::<NormalizedSourcePath>(&json).is_err(),
                "{case}"
            );
        }
    }
    Ok(())
}

#[test]
fn path_005_014_reject_escapes_host_roots_and_controls() -> TestResult {
    use CoreErrorCode::{AbsolutePathForbidden, InvalidSourcePath, PathEscape};
    let cases = [
        ("PATH-005", "Core/../Init.lua", PathEscape),
        ("PATH-006", "../Init.lua", PathEscape),
        ("PATH-007", "/Core/Init.lua", AbsolutePathForbidden),
        ("PATH-008", r"C:\Core\Init.lua", AbsolutePathForbidden),
        ("PATH-008", "C:Init.lua", AbsolutePathForbidden),
        (
            "PATH-009",
            r"\\server\share\Init.lua",
            AbsolutePathForbidden,
        ),
        ("PATH-010", r"\\?\C:\Init.lua", AbsolutePathForbidden),
        ("PATH-010", r"\\.\device", AbsolutePathForbidden),
        ("PATH-011", "file://host/Init.lua", AbsolutePathForbidden),
        ("PATH-011", "https://host/Init.lua", AbsolutePathForbidden),
        ("PATH-012", "", InvalidSourcePath),
        ("PATH-013", ".", InvalidSourcePath),
        ("PATH-013", "././", InvalidSourcePath),
        ("PATH-014", "Core/Init\0.lua", InvalidSourcePath),
        ("PATH-014", "Core/Init\n.lua", InvalidSourcePath),
        ("PATH-014", "Core/\u{85}Init.lua", InvalidSourcePath),
    ];
    for (case, input, code) in cases {
        assert_error(case, NormalizedSourcePath::parse(input), code, "candidate")?;
    }
    Ok(())
}

#[test]
fn path_016_017_preserve_unicode_form_and_case() -> TestResult {
    for (case, first, second) in [
        ("PATH-016", "Core/é.lua", "Core/e\u{301}.lua"),
        ("PATH-017", "Core/Foo.lua", "Core/foo.lua"),
    ] {
        let first = NormalizedSourcePath::parse(first)?.into_value();
        let second = NormalizedSourcePath::parse(second)?.into_value();
        assert_ne!(first, second, "{case}");
        assert_ne!(
            serde_json::to_string(&first)?,
            serde_json::to_string(&second)?,
            "{case}"
        );
    }
    Ok(())
}

#[test]
fn path_021_checks_the_input_bound_before_normalization() -> TestResult {
    let exact = "a".repeat(16_384);
    let parsed = NormalizedSourcePath::parse(&exact)?;
    assert_eq!(parsed.value().as_str(), exact);
    assert!(parsed.was_canonical());
    for input in [
        format!("{exact}a"),
        format!("{}Core.lua", "./".repeat(8_192)),
    ] {
        assert_error(
            "PATH-021",
            NormalizedSourcePath::parse(&input),
            CoreErrorCode::InvalidSourcePath,
            "candidate",
        )?;
    }
    Ok(())
}

#[test]
fn path_024_normalization_cannot_expose_a_drive_prefix() -> TestResult {
    // Exhaustive mutations over both drive cases and all prefix forms below.
    // Before the fix, ./C:/Init.lua became a typed absolute path which could not
    // even deserialize its own JSON. Check both the value and handle boundaries.
    for drive in (b'A'..=b'Z').chain(b'a'..=b'z') {
        for prefix in ["./", ".\\", "././", ".\\.\\", ".//", "./.\\"] {
            for suffix in [":/Init.lua", ":\\Init.lua", ":Init.lua", ":"] {
                let input = format!("{prefix}{}{suffix}", char::from(drive));
                assert_error(
                    &input,
                    NormalizedSourcePath::parse(&input),
                    CoreErrorCode::AbsolutePathForbidden,
                    "candidate",
                )?;
                assert_error(
                    &input,
                    handle(&input, SourceSpan::whole_file()),
                    CoreErrorCode::AbsolutePathForbidden,
                    "candidate",
                )?;
            }
        }
    }
    Ok(())
}

#[test]
fn path_024_component_mutations_never_accept_parent_traversal() -> TestResult {
    for prefix in ["", "./", "Core/", "Core\\", "Core//./"] {
        for separator in ["/", "\\", "//", "\\\\"] {
            let input = format!("{prefix}..{separator}Init.lua");
            assert_error(
                &input,
                NormalizedSourcePath::parse(&input),
                CoreErrorCode::PathEscape,
                "candidate",
            )?;
        }
    }
    Ok(())
}

#[test]
fn handle_equivalent_paths_have_one_round_trippable_identity() -> TestResult {
    let expected = handle("Core/Init.lua", SourceSpan::whole_file())?;
    for input in ["Core/Init.lua", "./Core//Init.lua", r"Core\Init.lua"] {
        let actual = handle(input, SourceSpan::whole_file())?;
        actual.validate()?;
        assert_eq!(actual, expected, "{input}");
        let bytes = serde_json::to_vec(&actual)?;
        let decoded: SourceHandle = serde_json::from_slice(&bytes)?;
        decoded.validate()?;
        assert_eq!(decoded, actual, "{input}");
    }
    Ok(())
}

#[test]
fn span_001_004_011_013_014_have_exact_wire_states() -> TestResult {
    let cases = [
        ("SPAN-001", SourceSpan::unknown(), r#"{"kind":"unknown"}"#),
        (
            "SPAN-002",
            SourceSpan::whole_file(),
            r#"{"kind":"whole_file"}"#,
        ),
        (
            "SPAN-003",
            SourceSpan::byte_range(0, 0)?,
            r#"{"byte_end":0,"byte_start":0,"kind":"byte_range"}"#,
        ),
        (
            "SPAN-004",
            SourceSpan::byte_range(0, 1)?,
            r#"{"byte_end":1,"byte_start":0,"kind":"byte_range"}"#,
        ),
        (
            "SPAN-011",
            SourceSpan::byte_range(9_007_199_254_740_991, 9_007_199_254_740_991)?,
            r#"{"byte_end":9007199254740991,"byte_start":9007199254740991,"kind":"byte_range"}"#,
        ),
    ];
    for (case, span, expected) in cases {
        span.validate()?;
        assert_eq!(canonical_json_string(&span)?, expected, "{case}");
        let decoded: SourceSpan = serde_json::from_str(expected)?;
        decoded.validate()?;
        assert_eq!(decoded, span, "SPAN-013 / {case}");
    }
    assert_ne!(SourceSpan::unknown(), SourceSpan::whole_file(), "SPAN-014");
    assert_eq!(SourceSpan::unknown().kind(), SourceSpanKind::Unknown);
    assert_eq!(SourceSpan::unknown().byte_start(), None);
    assert_eq!(SourceSpan::unknown().byte_end(), None);
    Ok(())
}

#[test]
fn span_005_012_reject_inversion_and_inexact_integer_offsets() -> TestResult {
    for (case, start, end) in [
        ("SPAN-005", 2, 1),
        ("SPAN-012", 0, 9_007_199_254_740_992),
        ("SPAN-012", 9_007_199_254_740_992, 9_007_199_254_740_992),
        ("SPAN-012", u64::MAX, u64::MAX),
    ] {
        assert_error(
            case,
            SourceSpan::byte_range(start, end),
            CoreErrorCode::InvalidSourceSpan,
            "span",
        )?;
        let wire = serde_json::json!({"kind":"byte_range","byte_start":start,"byte_end":end});
        let decoded: SourceSpan = serde_json::from_value(wire)?;
        assert_error(
            case,
            decoded.validate(),
            CoreErrorCode::InvalidSourceSpan,
            "span",
        )?;
    }
    Ok(())
}

#[test]
fn span_006_009_reject_negative_and_presentation_fields_at_decode() {
    for (case, json) in [
        (
            "SPAN-006",
            r#"{"kind":"byte_range","byte_start":-1,"byte_end":0}"#,
        ),
        ("SPAN-009", r#"{"kind":"unknown","line":1}"#),
        ("SPAN-009", r#"{"kind":"whole_file","column":1}"#),
    ] {
        assert!(serde_json::from_str::<SourceSpan>(json).is_err(), "{case}");
    }
}

#[test]
fn span_007_008_reject_mixed_and_incomplete_states() -> TestResult {
    for (case, json) in [
        (
            "SPAN-007",
            r#"{"kind":"unknown","byte_start":0,"byte_end":0}"#,
        ),
        (
            "SPAN-008",
            r#"{"kind":"whole_file","byte_start":0,"byte_end":1}"#,
        ),
        ("SPAN-007", r#"{"kind":"byte_range","byte_start":0}"#),
        ("SPAN-008", r#"{"kind":"byte_range","byte_end":1}"#),
    ] {
        let span: SourceSpan = serde_json::from_str(json)?;
        assert_error(
            case,
            span.validate(),
            CoreErrorCode::SpanStateConflict,
            "span",
        )?;
    }
    Ok(())
}

#[test]
fn span_010_changes_handle_identity_without_conflating_span_kinds() -> TestResult {
    let spans = [
        SourceSpan::unknown(),
        SourceSpan::whole_file(),
        SourceSpan::byte_range(0, 2)?,
        SourceSpan::byte_range(1, 2)?,
    ];
    let mut ids = std::collections::BTreeSet::new();
    for span in spans {
        assert!(
            ids.insert(handle("Core/Init.lua", span)?.handle_id()),
            "SPAN-010"
        );
    }
    assert_eq!(ids.len(), spans.len());
    Ok(())
}
