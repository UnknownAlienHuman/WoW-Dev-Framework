//! Exact numeric value conversion, never a second Lua parser or float evaluator.
use std::sync::atomic::AtomicBool;
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_constants::{
    ScalarCatalog, ScalarError, ScalarValue, exact_integer_magnitude,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn decimal_and_hex_magnitudes_share_exact_range_checks() {
    for (text, value) in [
        ("1.0", 1),
        ("1e3", 1000),
        ("0.5e1", 5),
        ("1.50E+1", 15),
        ("1000e-3", 1),
        ("000001.000", 1),
        ("0.000e-300", 0),
        ("0x10", 16),
        ("9007199254740991.0", 9_007_199_254_740_991),
    ] {
        assert_eq!(exact_integer_magnitude(text), Ok(value), "{text}");
    }
    for text in [
        "1.5",
        "1.00000000000000001",
        "1e-999",
        "1e999",
        "9007199254740992.0",
        "0x20000000000000",
        "+1",
        "-1",
        "",
        ".",
        "1e",
        "1e+",
        "1..0",
        "0x",
        " 1",
        "1e1e1",
    ] {
        assert!(exact_integer_magnitude(text).is_err(), "{text}");
    }
    assert_eq!(
        exact_integer_magnitude(&"0".repeat(2049)),
        Err(ScalarError::Limit)
    );
}

#[test]
fn exact_decimal_operands_resolve_without_rounding_or_losing_source() -> Result<()> {
    use ScalarError::{NonIntegralArithmetic, OutOfRange};
    for (expression, expected) in [
        ("1.0 + 2", Ok("3")),
        ("1e3 - 0.5e1", Ok("995")),
        ("0x10 + 1.50e1", Ok("31")),
        ("-2.0 + 0.1e1", Ok("-1")),
        ("9007199254740991.0 - 1e0", Ok("9007199254740990")),
        ("9007199254740991.0 + 1e0", Err(OutOfRange)),
        ("9007199254740992.0 - 1", Err(OutOfRange)),
        ("1.00000000000000001 + 1", Err(NonIntegralArithmetic)),
        ("1.5 + 0.5", Err(NonIntegralArithmetic)),
        ("1e-400 + 1", Err(NonIntegralArithmetic)),
        ("-0.0 + 1", Err(NonIntegralArithmetic)),
        ("\"1.0\" + 1", Err(NonIntegralArithmetic)),
    ] {
        let text = format!("APIDocumentation:AddDocumentationTable({{Value={expression}}})");
        let document = ingest_document(
            REV,
            "Value.lua",
            &text,
            &source_digest(text.as_bytes()),
            &AtomicBool::new(false),
        )?;
        let before = serde_json::to_string(&document)?;
        let fields = document.registrations()[0].value.fields().ok_or("fields")?;
        let value = &fields.first().ok_or("value")?.value;
        let catalog = ScalarCatalog::new(REV, &[])?;
        let actual = catalog.resolve(&document, value, None, &AtomicBool::new(false));
        assert_eq!(
            actual.map(|resolved| resolved.value),
            expected.map(|number| ScalarValue::Number(number.into())),
            "{expression}"
        );
        assert_eq!(serde_json::to_string(&document)?, before);
    }
    Ok(())
}
