//! Cross-resource behavior for the production catalog composition API.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{project_with_alias_catalog, project_with_alias_catalogs};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn catalog(path: &str, text: &str) -> Result<AliasDocument> {
    Ok(ingest_alias_catalog(
        REV,
        path,
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn documents() -> Result<Vec<DocumentationDocument>> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Read",Returns={{Name="value",Type="Choice"}}}}})"#;
    Ok(vec![ingest_document(
        REV,
        "API.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?])
}

#[test]
fn cross_file_dependencies_keep_provenance_and_canonical_output() -> Result<()> {
    let docs = documents()?;
    let a = catalog("A.lua", "---@alias Choice LiteralChoice|nil\n")?;
    let b = catalog(
        "B.lua",
        "---@alias LiteralChoice\n---|\"FIRST\"\n---|\"SECOND\"\n",
    )?;
    let cancel = AtomicBool::new(false);
    let first = project_with_alias_catalogs(&docs, "Mainline", None, &[&a, &b], &cancel)?;
    let second = project_with_alias_catalogs(&docs, "Mainline", None, &[&b, &a], &cancel)?;
    assert_eq!(first.projection, "projected_with_sidecars");
    assert_eq!(serde_json::to_value(&first)?, serde_json::to_value(second)?);
    let report = first.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/3");
    assert_eq!(report.source.path(), "A.lua");
    assert_eq!(report.additional_sources[0].path(), "B.lua");
    assert!(report.outcomes.iter().all(|outcome| outcome.status == "emitted"));
    let file = first.files.last().ok_or("file")?;
    assert_eq!(file.mappings.len(), 2);
    assert_eq!(file.mappings[0].source.path, "A.lua");
    assert_eq!(file.mappings[1].source.path, "B.lua");
    let old = project_with_alias_catalog(&docs, "Mainline", None, Some(&b), &cancel)?;
    let one = project_with_alias_catalogs(&docs, "Mainline", None, &[&b], &cancel)?;
    assert_eq!(serde_json::to_value(old)?, serde_json::to_value(one)?);
    assert!(project_with_alias_catalogs(&docs, "Mainline", None, &[&a, &a], &cancel).is_err());
    Ok(())
}

#[test]
fn conflicts_and_cycles_do_not_disappear_at_resource_boundaries() -> Result<()> {
    let docs = documents()?;
    let a = catalog(
        "A.lua",
        "---@alias Duplicate number\n---@alias CycleA CycleB\n---@alias Independent string\n",
    )?;
    let b = catalog(
        "B.lua",
        "---@alias Duplicate string\n---@alias CycleB CycleA\n---@alias Dependent Duplicate\n",
    )?;
    let cancel = AtomicBool::new(false);
    let result = project_with_alias_catalogs(&docs, "Mainline", None, &[&a, &b], &cancel)?;
    assert_eq!(result.projection, "partial");
    let outcomes = &result.aliases.as_ref().ok_or("aliases")?.outcomes;
    assert_eq!(outcomes.iter().filter(|o| o.status == "emitted").count(), 1);
    assert_eq!(outcomes[2].name, "Independent");
    assert_eq!(outcomes[0].status, "duplicate_alias");
    assert_eq!(outcomes[3].status, "duplicate_alias");
    let text = "---@alias Foreign string\n";
    let foreign = ingest_alias_catalog(
        &"b".repeat(40),
        "C.lua",
        text,
        &source_digest(text.as_bytes()),
        &cancel,
    )?;
    assert!(
        project_with_alias_catalogs(&docs, "Mainline", None, &[&a, &foreign], &cancel).is_err()
    );
    Ok(())
}
