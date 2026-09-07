//! The final parameter/return fragments retain their exact source-field identity.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project;
use wow_reference::native::{ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn local_fields_renamed_returns_and_callback_components_keep_exact_maps() -> Result<()> {
    let raw = r#"local argument={Name="value",Type="number"}
local result={Name="end",Type="string"}
APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="Rejected",Arguments={{Name="end",Type="number"}}},
{Name="Read",Documentation={"Пример"},Arguments={argument},Returns={result}}
},Tables={
{Name="Record",Type="Structure",Fields={{Name="label",Type="string"}}},
{Name="Callback",Type="CallbackType",Arguments={argument},Returns={{Name="ok",Type="bool"}}}
}})"#;
    let documents = [ingest_document(
        &"a".repeat(40),
        "Members.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?];
    let library = project(&documents, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.source_map_profile, "wow-native-field-maps/1");
    assert_eq!(library.projection, "partial");
    let file = library.files.first().ok_or("generated file")?;
    let expected = [
        (
            "parameter",
            "---@param value number",
            r#"{Name="value",Type="number"}"#,
        ),
        (
            "return",
            "---@return string",
            r#"{Name="end",Type="string"}"#,
        ),
        (
            "field",
            "---@field label string",
            r#"{Name="label",Type="string"}"#,
        ),
        (
            "parameter",
            "value: number",
            r#"{Name="value",Type="number"}"#,
        ),
        ("return", "boolean", r#"{Name="ok",Type="bool"}"#),
    ];
    let members = file
        .mappings
        .iter()
        .filter(|mapping| mapping.granularity != "declaration")
        .collect::<Vec<_>>();
    assert_eq!(members.len(), expected.len());
    for (mapping, (role, prefix, source)) in members.iter().zip(expected) {
        assert_eq!(mapping.granularity, role);
        assert_eq!(mapping.source.sha256, documents[0].sha256());
        assert_eq!(mapping.source.path, "Members.lua");
        assert!(mapping.source.scope.is_none());
        assert_eq!(
            raw.get(mapping.source.span.start..mapping.source.span.end),
            Some(source)
        );
        assert!(
            file.text
                .get(mapping.generated.start..mapping.generated.end)
                .is_some_and(|fragment| fragment.starts_with(prefix))
        );
    }
    let declaration = file.mappings.first().ok_or("function declaration")?;
    assert!(members[0].source.span.end < declaration.source.span.start);
    assert!(members[1].source.span.end < declaration.source.span.start);
    assert!(!file.text.contains("Rejected"));
    assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
    Ok(())
}
