//! Explicit empty namespace resources supplement, but never replace, native source.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalogs;
use wow_annotations::navigation::{NavigationIndex, SourceFile, SourceLookup};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog, ingest_aliases};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn document() -> Result<DocumentationDocument> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Name="Known",Type="System",Namespace="C_Known",Functions={{Name="Read"}}})"#;
    Ok(ingest_document(
        REV,
        "API.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn catalog(path: &str, raw: &str) -> Result<AliasDocument> {
    Ok(ingest_alias_catalog(
        DONOR,
        path,
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn exact_namespace_assignments_project_with_independent_maps_and_conflicts() -> Result<()> {
    let docs = [document()?];
    let raw = "---@meta _\n-- exact empty namespaces\nC_Missing = {} -- absent from generated docs\nC_Known = {} -- source collision\nC_Other = {}\n";
    let resource = catalog("Namespace.lua", raw)?;
    assert!(resource.aliases().is_empty());
    assert!(resource.structures().is_empty());
    assert_eq!(resource.namespaces().len(), 3);
    let cancelled = AtomicBool::new(false);
    let library = project_with_alias_catalogs(&docs, "Mainline", None, &[&resource], &cancelled)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/7");
    assert_eq!(report.source.revision(), DONOR);
    assert_eq!(report.namespace_outcomes.len(), 3);
    assert_eq!(report.namespace_outcomes[0].status, "emitted");
    assert_eq!(report.namespace_outcomes[1].status, "source_name_conflict");
    assert_eq!(report.namespace_outcomes[2].status, "emitted");
    let file = library.files.last().ok_or("overlay")?;
    let missing = file
        .text
        .find("C_Missing = {}")
        .ok_or("missing namespace")?;
    let other = file.text.find("C_Other = {}").ok_or("other namespace")?;
    assert!(missing < other);
    assert!(!file.text.contains("C_Known = {}"));
    let map = file
        .mappings
        .iter()
        .find(|map| {
            file.text
                .get(map.generated.start..map.generated.end)
                .is_some_and(|text| text == "C_Missing = {}")
        })
        .ok_or("namespace map")?;
    assert_eq!(map.source.scope, Some("annotation_alias_catalog"));
    assert_eq!(map.source.path, "Namespace.lua");
    assert_eq!(map.source.sha256, resource.sha256());
    assert_eq!(
        raw.get(map.source.span.start..map.source.span.end),
        Some("C_Missing = {}")
    );
    let index = NavigationIndex::new(&library, &cancelled)?;
    let SourceLookup::Mapped { candidates, .. } =
        index.source_at(&file.path, &file.sha256, map.generated.start, &cancelled)?
    else {
        return Err("namespace did not navigate".into());
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].revision, DONOR);
    let source = index.bind_source(
        SourceFile {
            scope: Some("annotation_alias_catalog"),
            revision: DONOR,
            path: resource.path(),
            sha256: resource.sha256(),
        },
        raw,
        &cancelled,
    )?;
    assert!(matches!(
        source.generated_at(map.source.span.start, &cancelled)?,
        wow_annotations::navigation::GeneratedLookup::Mapped { .. }
    ));
    Ok(())
}

#[test]
fn namespace_resources_are_order_independent_and_duplicates_remain_partial() -> Result<()> {
    let docs = [document()?];
    let first = catalog("A.lua", "C_Alpha = {}\n")?;
    let second = catalog("B.lua", "C_Beta = {}\n")?;
    let cancelled = AtomicBool::new(false);
    let left =
        project_with_alias_catalogs(&docs, "Mainline", None, &[&first, &second], &cancelled)?;
    let right =
        project_with_alias_catalogs(&docs, "Mainline", None, &[&second, &first], &cancelled)?;
    assert_eq!(serde_json::to_value(left)?, serde_json::to_value(right)?);

    let duplicate = catalog("B.lua", "C_Alpha = {}\n")?;
    let library =
        project_with_alias_catalogs(&docs, "Mainline", None, &[&first, &duplicate], &cancelled)?;
    assert_eq!(library.projection, "partial");
    let outcomes = &library.aliases.as_ref().ok_or("report")?.namespace_outcomes;
    assert_eq!(outcomes.len(), 2);
    assert!(
        outcomes
            .iter()
            .all(|outcome| outcome.status == "duplicate_namespace")
    );
    assert!(
        !library
            .files
            .last()
            .ok_or("native output")?
            .text
            .contains("C_Alpha = {}")
    );
    Ok(())
}

#[test]
fn namespace_profile_rejects_general_lua_and_mixed_annotation_resources() -> Result<()> {
    for raw in [
        "Other = {}\n",
        "C_Bad = { value = 1 }\n",
        "C_Bad = make()\n",
        "C_Bad.member = {}\n",
        "local C_Bad = {}\n",
        "C_A, C_B = {}, {}\n",
        "do C_Bad = {} end\n",
        "C_Good = {}\nos.execute('bad')\n",
        "---@alias Alias string\nC_Good = {}\n",
        "---@class Record\nC_Good = {}\n",
    ] {
        assert!(catalog("Namespace.lua", raw).is_err(), "{raw}");
    }
    let raw = "C_Good = {}\n";
    assert!(
        ingest_aliases(
            DONOR,
            "Namespace.lua",
            raw,
            &source_digest(raw.as_bytes()),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    Ok(())
}
