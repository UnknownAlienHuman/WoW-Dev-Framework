//! The optional alias lane must use the same reserved type space as native
//! receiver projection, including names outside its supported target profile.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalog;
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_aliases::ingest_aliases;

#[test]
fn catalog_cannot_redefine_reserved_types_or_their_qualified_prefixes()
-> Result<(), Box<dyn std::error::Error>> {
    let cancelled = AtomicBool::new(false);
    let source =
        r#"APIDocumentation:AddDocumentationTable({Name="ObjectAPI",Type="ScriptObject"})"#;
    let documents = [ingest_document(
        &"a".repeat(40),
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &cancelled,
    )?];
    let mut input = String::new();
    for name in ["never", "lightuserdata", "bool", "cstring", "luaIndex"] {
        input.push_str(&format!(
            "---@alias {name} string\n---@alias {name}.Fake string\n"
        ));
    }
    input.push_str("---@alias Independent number\n");
    let catalog = ingest_aliases(
        &"b".repeat(40),
        "Aliases.lua",
        &input,
        &source_digest(input.as_bytes()),
        &cancelled,
    )?;
    let library =
        project_with_alias_catalog(&documents, "Mainline", None, Some(&catalog), &cancelled)?;
    let report = library.aliases.as_ref().ok_or("missing report")?;
    assert_eq!(report.outcomes.len(), 11);
    assert!(report.outcomes[..10].iter().all(|v| v.status != "emitted"));
    assert_eq!(report.outcomes[10].status, "emitted");
    assert_eq!(library.projection, "partial");
    let file = library.files.last().ok_or("missing alias file")?;
    assert_eq!(file.mappings.len(), 1);
    assert!(file.text.contains("---@alias Independent number\n"));
    assert!(!file.text.contains("---@alias never"));
    assert!(!file.text.contains("---@alias lightuserdata"));
    Ok(())
}
