//! End-to-end checks of the production generated-to-source query.
use std::sync::atomic::{AtomicBool, Ordering};
use wow_annotations::native::{project, project_with_alias_catalog};
use wow_annotations::navigation::{LookupError, MappingPrecision, SourceLookup, source_at};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RAW: &str = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="Read",Documentation={"é 🦀"},Arguments={{Name="value",Type="number"}},Returns={{Name="result",Type="string"}}}
}})"#;

fn document() -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        "Types.lua",
        RAW,
        &source_digest(RAW.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn member_navigation_checks_bytes_boundaries_profiles_and_cancellation() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first().ok_or("file")?;
    let path = file.path.clone();
    let digest = file.sha256.clone();
    let offset = file.text.find("---@param value").ok_or("parameter")?;
    let lookup = source_at(&library, &path, &digest, offset, &cancelled)?;
    let SourceLookup::Mapped {
        precision,
        candidates,
    } = lookup
    else {
        return Err("member did not map".into());
    };
    assert_eq!(precision, MappingPrecision::Member);
    assert_eq!(candidates.len(), 1);
    let source = &candidates[0];
    assert_eq!(source.revision, REV);
    assert_eq!(source.granularity, "parameter");
    assert_eq!(
        RAW.get(source.source.span.start..source.source.span.end),
        Some(r#"{Name="value",Type="number"}"#)
    );
    let unicode = file.text.find('é').ok_or("UTF-8 documentation")?;
    assert_eq!(
        source_at(&library, &path, &digest, unicode + 1, &cancelled).err(),
        Some(LookupError::InvalidPosition)
    );
    assert!(matches!(
        source_at(&library, &path, &digest, file.text.len(), &cancelled)?,
        SourceLookup::Unmapped
    ));
    assert_eq!(
        source_at(&library, &path, "sha256:stale", offset, &cancelled).err(),
        Some(LookupError::StaleArtifact)
    );
    cancelled.store(true, Ordering::Relaxed);
    assert_eq!(
        source_at(&library, &path, &digest, offset, &cancelled).err(),
        Some(LookupError::Cancelled)
    );
    cancelled.store(false, Ordering::Relaxed);
    library.source_map_profile = "unknown";
    assert_eq!(
        source_at(&library, &path, &digest, offset, &cancelled).err(),
        Some(LookupError::UnsupportedProfile)
    );
    Ok(())
}

#[test]
fn same_named_catalog_and_blizzard_files_keep_independent_revisions() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let raw = "---@alias Quantity number\n";
    let catalog = ingest_alias_catalog(
        DONOR,
        "Types.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &cancelled,
    )?;
    let mut library =
        project_with_alias_catalog(&docs, "Mainline", None, Some(&catalog), &cancelled)?;
    let file = library.files.last_mut().ok_or("alias file")?;
    let path = file.path.clone();
    let digest = file.sha256.clone();
    let mapping = file.mappings.first().ok_or("alias map")?.clone();
    file.mappings.push(mapping.clone());
    let lookup = source_at(
        &library,
        &path,
        &digest,
        mapping.generated.start,
        &cancelled,
    )?;
    let SourceLookup::Mapped {
        precision,
        candidates,
    } = lookup
    else {
        return Err("alias did not map".into());
    };
    assert_eq!(precision, MappingPrecision::Declaration);
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().all(|candidate| {
        candidate.revision == DONOR
            && candidate.source.path == "Types.lua"
            && candidate.source.scope == Some("annotation_alias_catalog")
    }));
    let corrupted = &mut library.files.last_mut().ok_or("alias file")?.mappings[1];
    corrupted.source.sha256 = source_digest(b"changed");
    assert_eq!(
        source_at(&library, &path, &digest, 0, &cancelled).err(),
        Some(LookupError::InvalidMapping)
    );
    Ok(())
}
