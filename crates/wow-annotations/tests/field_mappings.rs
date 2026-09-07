//! Field ranges are joined by admitted input position, never by rendered spelling.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{NativeLibrary, project};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn document(raw: &str) -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        "Fields.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn check(library: &NativeLibrary<'_>, raw: &str, expected: &[(&str, &str, &str)]) -> Result<()> {
    let mut actual = Vec::new();
    for file in &library.files {
        assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
        for map in file
            .mappings
            .iter()
            .filter(|m| m.granularity != "declaration")
        {
            let generated = file
                .text
                .get(map.generated.start..map.generated.end)
                .ok_or("output span")?;
            let source = raw
                .get(map.source.span.start..map.source.span.end)
                .ok_or("source span")?;
            assert_eq!(map.source.path, "Fields.lua");
            assert_eq!(map.source.sha256, source_digest(raw.as_bytes()));
            assert!(file.mappings.iter().any(|parent| {
                parent.granularity == "declaration"
                    && parent.generated.start <= map.generated.start
                    && map.generated.end <= parent.generated.end
                    && parent.source.span.start <= map.source.span.start
                    && map.source.span.end <= parent.source.span.end
            }));
            actual.push((map.granularity, generated, source));
        }
    }
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn fields_keep_exact_utf8_spans_after_filtered_declarations_and_return_renaming() -> Result<()> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="Bad",Arguments={{Name="end",Type="number"}}},
{Name="Read",Documentation={"é 🦀"},Arguments={{Name="id",Type="number"},{Name="items",Type="table",InnerType="number",Nilable=true}},Returns={{Name="end",Type="bool"}}}
},Tables={{Name="Record",Type="Structure",Fields={{Name="text",Type="string"}}}})"#;
    let docs = [document(raw)?];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "partial");
    assert_eq!(library.issues.len(), 1);
    check(
        &library,
        raw,
        &[
            (
                "parameter",
                "---@param id number",
                r#"{Name="id",Type="number"}"#,
            ),
            (
                "parameter",
                "---@param items? number[]",
                r#"{Name="items",Type="table",InnerType="number",Nilable=true}"#,
            ),
            (
                "return",
                "---@return boolean __wow_return_1",
                r#"{Name="end",Type="bool"}"#,
            ),
            (
                "field",
                "---@field text string",
                r#"{Name="text",Type="string"}"#,
            ),
        ],
    )
}

#[test]
fn callbacks_map_both_profiles_and_terminal_argument_packs() -> Result<()> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Tables={
{Name="Plain",Type="CallbackType",Arguments={{Name="id",Type="number"}}},
{Name="Rich",Type="CallbackType",Arguments={{Name="values",Type="table",InnerType="number"},{Name="rest",Type="string",StrideIndex=1}},Returns={{Name="ok",Type="bool"},{Name="list",Type="table",InnerType="string",Nilable=true}}}
}})"#;
    let docs = [document(raw)?];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "projected_with_sidecars");
    check(
        &library,
        raw,
        &[
            (
                "parameter",
                "id: number",
                r#"{Name="id",Type="number"}"#,
            ),
            (
                "parameter",
                "values: number[]",
                r#"{Name="values",Type="table",InnerType="number"}"#,
            ),
            (
                "parameter",
                "...: string",
                r#"{Name="rest",Type="string",StrideIndex=1}"#,
            ),
            (
                "return",
                "boolean",
                r#"{Name="ok",Type="bool"}"#,
            ),
            (
                "return",
                "string[]?",
                r#"{Name="list",Type="table",InnerType="string",Nilable=true}"#,
            ),
        ],
    )
}
