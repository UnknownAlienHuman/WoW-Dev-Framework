//! Numeric source-to-output coverage for the existing native and literal owners.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{NativeLibrary, project};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn document(path: &str, text: &str) -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        path,
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn output(library: &NativeLibrary<'_>) -> String {
    library
        .files
        .iter()
        .map(|file| file.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn fractions_scientific_constants_and_signed_zero_reach_generated_bytes() -> Result<()> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Limits",Type="Constants",Values={
{Name="Fraction",Type="number",Value=1.5},{Name="Step",Type="number",Value=0.5},
{Name="Exponent",Type="number",Value=2.5e-1},{Name="Integral",Type="number",Value=1.5e1},
{Name="DecimalInteger",Type="number",Value=2.0},{Name="FractionalTenth",Type="number",Value=0.1},
{Name="NegativeZero",Type="number",Value=-0.0},{Name="IntegerZero",Type="number",Value=-0},
{Name="SignedFraction",Type="number",Value=-1.25},{Name="Hex",Type="number",Value=0x10}
}}}})"#;
    let docs = [document("Values.lua", raw)?];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let text = output(&library);
    for expected in [
        "Fraction = 1.5,",
        "Step = 0.5,",
        "Exponent = 0.25,",
        "Integral = 15,",
        "DecimalInteger = 2,",
        "FractionalTenth = 0.1,",
        "NegativeZero = -0.0,",
        "IntegerZero = -0.0,",
        "SignedFraction = -1.25,",
        "Hex = 16,",
    ] {
        assert!(text.contains(expected), "{expected}");
    }
    assert!(!library.negative_authority);
    assert!(serde_json::to_string(&library.scalar_resolutions)?.contains("2.5e-1"));
    for file in &library.files {
        assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
        for map in &file.mappings {
            assert_eq!(map.source.sha256, source_digest(raw.as_bytes()));
            assert!(
                raw.get(map.source.span.start..map.source.span.end)
                    .is_some()
            );
        }
    }
    Ok(())
}

#[test]
fn fractional_references_keep_transitive_source_evidence() -> Result<()> {
    let docs = [
        document(
            "A.lua",
            "APIDocumentation:AddDocumentationTable({Tables={{Name='Base',Type='Constants',Values={{Name='Step',Type='number',Value=1.5}}}}})",
        )?,
        document(
            "B.lua",
            "APIDocumentation:AddDocumentationTable({Tables={{Name='Derived',Type='Constants',Values={{Name='Step',Type='number',Value=Constants.Base.Step}}}}})",
        )?,
    ];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "projected_with_sidecars");
    assert_eq!(output(&library).matches("Step = 1.5,").count(), 2);
    let record = library
        .scalar_resolutions
        .iter()
        .find(|record| record.source.path == "B.lua")
        .ok_or("reference")?;
    let resolved = record.result.as_ref().map_err(|_| "unresolved")?;
    assert_eq!(resolved.evidence.len(), 2);
    assert!(resolved.evidence.iter().any(|item| item.path == "A.lua"));
    assert!(resolved.evidence.iter().any(|item| item.path == "B.lua"));
    Ok(())
}

#[test]
fn overflow_underflow_and_rounding_do_not_erase_valid_constant_siblings() -> Result<()> {
    for rejected in [
        "1e999",
        "-1e999",
        "1e-999",
        "-1e-999",
        "9007199254740993",
        "9007199254740993.0",
        "9007199254740991.1",
        "1.00000000000000001",
        "0x20000000000000",
        "0.25+0.5",
    ] {
        let raw = format!(
            "APIDocumentation:AddDocumentationTable({{Tables={{{{Name='Limits',Type='Constants',Values={{{{Name='Good',Type='number',Value=1.5}},{{Name='Bad',Type='number',Value={rejected}}}}}}}}}})",
        );
        let docs = [document("Values.lua", &raw)?];
        let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
        assert_eq!(library.projection, "partial", "{rejected}");
        let text = output(&library);
        assert!(text.contains("Good = 1.5,"), "{rejected}");
        assert!(!text.contains("Bad ="), "{rejected}");
        assert_eq!(library.issues.len(), 1, "{rejected}");
    }
    Ok(())
}

#[test]
fn fractional_enum_does_not_gain_constant_float_semantics() -> Result<()> {
    let docs = [document(
        "Values.lua",
        "APIDocumentation:AddDocumentationTable({Tables={{Name='Mode',Type='Enumeration',Fields={{Name='Good',EnumValue=1},{Name='Bad',EnumValue=1.5}}},{Name='Limits',Type='Constants',Values={{Name='Good',Type='number',Value=1.5}}}}})",
    )?];
    let library = project(&docs, "Mainline", &AtomicBool::new(false))?;
    assert_eq!(library.projection, "partial");
    assert_eq!(library.issues.len(), 1);
    let text = output(&library);
    assert!(text.contains("Good = 1,"));
    assert!(text.contains("Good = 1.5,"));
    assert!(!text.contains("Bad ="));
    Ok(())
}
