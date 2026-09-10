//! Ketho GlobalColors resources become inert, source-bound global declarations.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalogs;
use wow_annotations::navigation::{GeneratedLookup, NavigationIndex, SourceFile, SourceLookup};
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog};

const SOURCE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn catalog(path: &str, raw: &str) -> Result<AliasDocument, Box<dyn std::error::Error>> {
    Ok(ingest_alias_catalog(
        DONOR_REVISION,
        path,
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn project(
    colors: &str,
    include_color_type: bool,
) -> Result<wow_annotations::native::NativeLibrary<'static>, Box<dyn std::error::Error>> {
    let source = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Read"}}})"#;
    let documents = Box::leak(Box::new([ingest_document(
        SOURCE_REVISION,
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?]));
    let colors = Box::leak(Box::new(catalog("GlobalColors.lua", colors)?));
    let mut catalogs = vec![colors as &AliasDocument];
    if include_color_type {
        let types = Box::leak(Box::new(catalog(
            "Types.lua",
            "---@alias colorRGBA table\n",
        )?));
        catalogs.push(types);
    }
    Ok(project_with_alias_catalogs(
        documents,
        "Mainline",
        None,
        &catalogs,
        &AtomicBool::new(false),
    )?)
}

#[test]
fn exact_colors_emit_inert_globals_and_bidirectional_maps() -> Result<(), Box<dyn std::error::Error>>
{
    let raw = "---@meta _\nFIRST_COLOR = CreateColor(0.000, 0.500, 1.000, 1.000)\nSECOND_COLOR=CreateColor(1,0,0,1)\n";
    let library = project(raw, true)?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/9");
    assert_eq!(report.global_color_outcomes.len(), 2);
    assert!(
        report
            .global_color_outcomes
            .iter()
            .all(|outcome| outcome.status == "emitted")
    );
    assert!(report.unresolved_global_color_types.is_empty());
    let file = library.files.last().ok_or("external output")?;
    assert!(file.text.contains("---@type colorRGBA\nFIRST_COLOR = nil"));
    assert!(file.text.contains("---@type colorRGBA\nSECOND_COLOR = nil"));
    assert!(!file.text.contains("CreateColor("));
    let color_maps = file
        .mappings
        .iter()
        .filter(|mapping| mapping.source.path == "GlobalColors.lua")
        .collect::<Vec<_>>();
    assert_eq!(color_maps.len(), 2);
    let index = NavigationIndex::new(&library, &AtomicBool::new(false))?;
    for mapping in &color_maps {
        let SourceLookup::Mapped { candidates, .. } = index.source_at(
            &file.path,
            &file.sha256,
            mapping.generated.start,
            &AtomicBool::new(false),
        )?
        else {
            return Err("generated color did not navigate".into());
        };
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].revision, DONOR_REVISION);
        assert_eq!(candidates[0].source.path, "GlobalColors.lua");
    }
    let document = std::iter::once(report.source)
        .chain(report.additional_sources.iter().copied())
        .find(|document| document.path() == "GlobalColors.lua")
        .ok_or("color resource")?;
    let source = index.bind_source(
        SourceFile {
            scope: Some("annotation_alias_catalog"),
            revision: DONOR_REVISION,
            path: document.path(),
            sha256: document.sha256(),
        },
        raw,
        &AtomicBool::new(false),
    )?;
    let fact = &document.global_colors()[0];
    assert!(matches!(
        source.generated_at(fact.span.start, &AtomicBool::new(false))?,
        GeneratedLookup::Mapped { .. }
    ));
    Ok(())
}

#[test]
fn duplicates_are_explicit_while_independent_colors_survive()
-> Result<(), Box<dyn std::error::Error>> {
    let raw = "DUPLICATE_COLOR = CreateColor(0, 0, 0, 1)\nDUPLICATE_COLOR = CreateColor(1, 1, 1, 1)\nUNIQUE_COLOR = CreateColor(0, 1, 0, 1)\n";
    let library = project(raw, true)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/9");
    assert_eq!(
        report.global_color_outcomes[0].status,
        "duplicate_global_color"
    );
    assert_eq!(
        report.global_color_outcomes[1].status,
        "duplicate_global_color"
    );
    assert_eq!(report.global_color_outcomes[2].status, "emitted");
    let text = &library.files.last().ok_or("external output")?.text;
    assert!(!text.contains("DUPLICATE_COLOR = nil"));
    assert!(text.contains("UNIQUE_COLOR = nil"));
    assert_eq!(
        library
            .issues
            .iter()
            .filter(|issue| issue.code == "duplicate_global_color")
            .count(),
        2
    );
    Ok(())
}

#[test]
fn missing_color_type_retains_globals_but_keeps_projection_partial()
-> Result<(), Box<dyn std::error::Error>> {
    let raw = "SYNTHETIC_COLOR = CreateColor(0, 0, 0, 1)\n";
    let library = project(raw, false)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.global_color_outcomes[0].status, "emitted");
    assert_eq!(report.unresolved_global_color_types.len(), 1);
    assert!(
        library
            .files
            .last()
            .ok_or("external output")?
            .text
            .contains("---@type colorRGBA\nSYNTHETIC_COLOR = nil")
    );
    assert!(library.issues.iter().any(|issue| {
        issue.code == "unresolved_global_color_type"
            && issue.source.scope == Some("annotation_alias_catalog")
    }));
    Ok(())
}
