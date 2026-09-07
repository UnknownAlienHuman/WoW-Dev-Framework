//! String forms are decoded through the production source owner, not Lua execution.
use std::sync::atomic::AtomicBool;
use wow_reference::native::{DocumentationDocument, RawKind, ingest_document, source_digest};
use wow_reference::native_model::object;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn read(source: &str) -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        "Strings.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn decimal_ascii_escapes_preserve_values_keys_and_exact_spans() -> Result<()> {
    let source = r#"APIDocumentation:AddDocumentationTable({["\078ame"]="\065\66\0674\0\127Я",literal="\\255",quote="\034"})"#;
    let document = read(source)?;
    let fields = object(&document.registrations()[0].value)?;
    let name = fields["Name"];
    assert_eq!(name.kind, RawKind::String("ABC4\0\u{7f}Я".into()));
    assert_eq!(fields["literal"].kind, RawKind::String("\\255".into()));
    assert_eq!(fields["quote"].kind, RawKind::String("\"".into()));
    assert_eq!(
        source.get(name.span.start..name.span.end),
        Some(r#""\065\66\0674\0\127Я""#)
    );
    assert_eq!(document.sha256(), source_digest(source.as_bytes()));
    assert_eq!(
        serde_json::to_value(document)?["evaluator"],
        "ketho-apidoc-declarative/3"
    );
    Ok(())
}

#[test]
fn long_string_newline_pairs_normalize_once_and_keep_literal_backslashes() -> Result<()> {
    for first in ["", "\n", "\r", "\r\n", "\n\r"] {
        for separator in ["\n", "\r", "\r\n", "\n\r"] {
            let token = format!("[==[{first}Я{separator}\\065{separator}text]==]");
            let source = format!("APIDocumentation:AddDocumentationTable({{value={token}}})");
            let document = read(&source)?;
            let fields = object(&document.registrations()[0].value)?;
            let value = fields["value"];
            assert_eq!(value.kind, RawKind::String("Я\n\\065\ntext".into()));
            assert_eq!(
                source.get(value.span.start..value.span.end),
                Some(token.as_str())
            );
        }
    }
    for (raw, expected) in [
        ("\r\n\r", "\n\n"),
        ("\n\r\n", "\n\n"),
        ("\r\r", "\n\n"),
    ] {
        let source = format!("APIDocumentation:AddDocumentationTable({{value=[[a{raw}b]]}})");
        let document = read(&source)?;
        assert_eq!(
            object(&document.registrations()[0].value)?["value"].kind,
            RawKind::String(format!("a{expected}b"))
        );
    }
    Ok(())
}

#[test]
fn non_ascii_byte_escapes_and_overflow_never_change_or_disappear() {
    for literal in [
        r#""\128""#,
        r#""\255""#,
        r#""\256""#,
        r#""\999""#,
        r#""\x41""#,
        r#""\u{41}""#,
    ] {
        for source in [
            format!("APIDocumentation:AddDocumentationTable({{value={literal}}})"),
            format!("APIDocumentation:AddDocumentationTable({{[{literal}]=1}})"),
        ] {
            assert!(read(&source).is_err(), "{literal}");
        }
    }
}

#[test]
fn previously_supported_strings_keep_the_original_evaluator_profile() -> Result<()> {
    let source = r#"APIDocumentation:AddDocumentationTable({value="Я\n\r\t\\255",long=[[\255]]})"#;
    let document = read(source)?;
    let fields = object(&document.registrations()[0].value)?;
    assert_eq!(fields["value"].kind, RawKind::String("Я\n\r\t\\255".into()));
    assert_eq!(fields["long"].kind, RawKind::String("\\255".into()));
    assert_eq!(
        serde_json::to_value(document)?["evaluator"],
        "ketho-apidoc-declarative/2"
    );
    Ok(())
}
