//! Explicit external structures supplement, but never replace, native source types.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalogs;
use wow_annotations::navigation::{
    GeneratedLookup, MappingPrecision, NavigationIndex, SourceFile, SourceLookup,
};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_aliases::{AliasDocument, ingest_alias_catalog, ingest_aliases};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn document() -> Result<DocumentationDocument> {
    let raw = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Read",Returns={{Name="result",Type="ExternalRecord"}}}},Tables={{Name="NativeRecord",Type="Structure",Fields={{Name="id",Type="number"}}}}})"#;
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
fn classes_arrays_nullable_fields_and_cross_file_aliases_reach_native_output() -> Result<()> {
    let docs = [document()?];
    let aliases = catalog(
        "Aliases.lua",
        "---@alias ID number\n---@alias RecordAlias ExternalRecord\n",
    )?;
    let raw = "\u{feff}---@meta _\r\n---@class ExternalRecord\r\n---@field points number[]\r\n---@field label string?\r\n---@field id ID\r\n---@class RecursiveRecord\r\n---@field child RecursiveRecord?\r\n";
    let structures = catalog("Structures.lua", raw)?;
    assert_eq!(structures.structures().len(), 2);
    assert!(structures.aliases().is_empty());
    let cancelled = AtomicBool::new(false);
    let library = project_with_alias_catalogs(
        &docs,
        "Mainline",
        None,
        &[&structures, &aliases],
        &cancelled,
    )?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let report = library.aliases.as_ref().ok_or("report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/6");
    assert!(
        report
            .outcomes
            .iter()
            .all(|outcome| outcome.status == "emitted")
    );
    assert!(
        report
            .structure_outcomes
            .iter()
            .all(|outcome| outcome.status == "emitted")
    );
    let file = library.files.last().ok_or("overlay")?;
    assert!(file.text.contains("---@alias RecordAlias ExternalRecord\n"));
    assert!(file.text.contains("---@class ExternalRecord\n---@field points number[]\n---@field label string?\n---@field id ID\n"));
    assert!(file.text.contains("---@field child RecursiveRecord?"));
    let map = file
        .mappings
        .iter()
        .find(|map| map.source.path == "Structures.lua")
        .ok_or("map")?;
    assert!(
        raw.get(map.source.span.start..map.source.span.end)
            .ok_or("source span")?
            .contains("---@field id ID")
    );
    let index = NavigationIndex::new(&library, &cancelled)?;
    let SourceLookup::Mapped { candidates, .. } =
        index.source_at(&file.path, &file.sha256, map.generated.start, &cancelled)?
    else {
        return Err("external class did not map".into());
    };
    assert_eq!(candidates[0].revision, DONOR);
    assert_eq!(candidates[0].source.scope, Some("annotation_alias_catalog"));
    let reversed = project_with_alias_catalogs(
        &docs,
        "Mainline",
        None,
        &[&aliases, &structures],
        &cancelled,
    )?;
    assert_eq!(
        serde_json::to_value(&library)?,
        serde_json::to_value(reversed)?
    );
    Ok(())
}

#[test]
fn unsupported_classes_do_not_erase_independent_supported_structures() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    for bad in [
        "---@class Broken: Parent\n---@field id number\n",
        "---@class Broken<T>\n---@field id T\n",
        "---@class Broken\n---@field private id number\n",
        "---@class Broken\n---@field [string] number\n",
        "---@class Broken\n---@field id number[][]\n",
        "---@class Broken\n---@field callback fun():number\n",
        "---@class Broken\n---@field id number\n---@field id string\n",
    ] {
        let raw = format!("{bad}---@class ExternalRecord\n---@field id number\n");
        let resource = catalog("Types.lua", &raw)?;
        let library =
            project_with_alias_catalogs(&docs, "Mainline", None, &[&resource], &cancelled)?;
        assert_eq!(library.projection, "partial", "{bad}");
        let report = library.aliases.as_ref().ok_or("report")?;
        assert_ne!(report.structure_outcomes[0].status, "emitted");
        assert_eq!(report.structure_outcomes[1].status, "emitted");
        let output = &library.files.last().ok_or("overlay")?.text;
        assert!(output.contains("---@class ExternalRecord"));
        assert!(!output.contains("---@class Broken"));
    }
    Ok(())
}

#[test]
fn unresolved_fields_are_retained_without_widening_and_keep_the_overlay_partial() -> Result<()> {
    let docs = [document()?];
    let resource = catalog(
        "Types.lua",
        "---@class ExternalRecord\n---@field value MissingType?\n",
    )?;
    let cancelled = AtomicBool::new(false);
    let library = project_with_alias_catalogs(&docs, "Mainline", None, &[&resource], &cancelled)?;
    assert_eq!(library.projection, "partial");
    assert!(
        library
            .files
            .last()
            .ok_or("overlay")?
            .text
            .contains("---@field value MissingType?")
    );
    let report = library.aliases.as_ref().ok_or("report")?;
    assert_eq!(report.structure_outcomes[0].status, "emitted");
    assert_eq!(report.unresolved_structure_fields.len(), 1);
    assert!(
        library
            .issues
            .iter()
            .any(|issue| issue.code == "unresolved_structure_field_type")
    );
    assert!(!library.negative_authority);
    Ok(())
}

#[test]
fn native_collisions_and_alias_class_duplicates_do_not_replace_definitions() -> Result<()> {
    let docs = [document()?];
    let resource = catalog(
        "Types.lua",
        "---@class NativeRecord\n---@field poison string\n---@class Duplicate\n---@field id number\n---@alias Duplicate string\n---@class ExternalRecord\n---@field id number\n",
    )?;
    let cancelled = AtomicBool::new(false);
    let library = project_with_alias_catalogs(&docs, "Mainline", None, &[&resource], &cancelled)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("report")?;
    assert_eq!(report.structure_outcomes[0].status, "source_name_conflict");
    assert_eq!(report.structure_outcomes[1].status, "duplicate_structure");
    assert_eq!(report.structure_outcomes[2].status, "emitted");
    assert_eq!(report.outcomes[0].status, "duplicate_alias");
    let overlay = &library.files.last().ok_or("overlay")?.text;
    assert!(!overlay.contains("NativeRecord"));
    assert!(!overlay.contains("Duplicate"));
    Ok(())
}

#[test]
fn directives_execution_and_orphan_fields_reject_and_legacy_bytes_stay_unchanged() -> Result<()> {
    for raw in [
        "---@field id number\n",
        "---@class Record\n---@alias ID number\n---@field id number\n",
        "---@class Record\n---@diagnostic disable\n",
        "---@class Record\nlocal Record = {}\n",
        "---@class Record\nreturn {}\n",
    ] {
        assert!(catalog("Types.lua", raw).is_err());
    }
    let raw = "---@alias ID number|nil\n";
    let original = ingest_aliases(
        DONOR,
        "Types.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        serde_json::to_value(original)?,
        serde_json::to_value(catalog("Types.lua", raw)?)?
    );
    let raw = "---@class Record\n---@field id number\n";
    assert!(
        ingest_aliases(
            DONOR,
            "Types.lua",
            raw,
            &source_digest(raw.as_bytes()),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn external_field_maps_bind_exact_members_without_colliding_with_native_paths() -> Result<()> {
    let docs = [document()?];
    // Deliberately use the same path as the independent Blizzard input.
    let raw = "\u{feff}---@meta _\r\n---@class ExternalRecord\r\n---@field id number\r\n---@field label string?\r\n---@class OtherRecord\r\n---@field id number\r\n";
    let resource = catalog("API.lua", raw)?;
    let cancelled = AtomicBool::new(false);
    let library = project_with_alias_catalogs(&docs, "Mainline", None, &[&resource], &cancelled)?;
    let file = library.files.last().ok_or("overlay")?;
    let index = NavigationIndex::new(&library, &cancelled)?;
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
    let fields = resource
        .structures()
        .iter()
        .flat_map(|structure| &structure.fields)
        .collect::<Vec<_>>();
    let maps = file
        .mappings
        .iter()
        .filter(|map| map.granularity == "field")
        .collect::<Vec<_>>();
    assert_eq!(maps.len(), fields.len());
    assert_eq!(maps.len(), 3);
    for map in maps {
        let field = fields
            .iter()
            .find(|field| field.span == map.source.span)
            .ok_or("exact source field")?;
        let generated = file
            .text
            .get(map.generated.start..map.generated.end)
            .ok_or("generated member bytes")?;
        let original = raw
            .get(field.span.start..field.span.end)
            .ok_or("source member bytes")?;
        assert_eq!(generated, original);
        let SourceLookup::Mapped {
            precision,
            candidates,
        } = index.source_for_range(&file.path, &file.sha256, map.generated, &cancelled)?
        else {
            return Err("field range did not map".into());
        };
        assert_eq!(precision, MappingPrecision::Member);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].revision, DONOR);
        assert_eq!(candidates[0].source.sha256, resource.sha256());
        assert_eq!(candidates[0].source.span, field.span);
        let GeneratedLookup::Mapped { candidates } =
            source.generated_for_range(field.span, &cancelled)?
        else {
            return Err("source field range did not map".into());
        };
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].generated, map.generated);
    }
    let first_member = file
        .mappings
        .iter()
        .position(|map| map.granularity == "field")
        .ok_or("first member")?;
    assert!(
        file.mappings[..first_member]
            .iter()
            .all(|map| map.granularity == "declaration")
    );
    assert!(
        file.mappings[first_member..]
            .iter()
            .all(|map| map.granularity == "field")
    );
    Ok(())
}
