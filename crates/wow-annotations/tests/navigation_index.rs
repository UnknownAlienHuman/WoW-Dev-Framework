//! Prepared queries retain the existing one-shot semantics and exact source joins.
use std::sync::atomic::{AtomicBool, Ordering};
use wow_annotations::native::{project, project_with_alias_catalog};
use wow_annotations::navigation::{
    GeneratedLookup, LookupError, NavigationIndex, PositionEncoding, SourceLookup, TextPosition,
    source_at, source_at_position,
};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RAW: &str = r#"local args = {{Name="value",Type="number"}}
APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="First",Documentation={"é 🦀"},Arguments=args,Returns={{Name="result",Type="string"}}},
{Name="Second",Arguments=args}
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
fn prepared_queries_match_one_shot_queries_at_every_generated_boundary() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    // Overlaps and identical map ties must retain the original serialized order.
    let file = library.files.first_mut().ok_or("file")?;
    let member = file
        .mappings
        .iter()
        .find(|map| map.granularity == "parameter")
        .ok_or("member")?
        .clone();
    file.mappings.insert(0, member);
    file.mappings.reverse();
    let index = NavigationIndex::new(&library, &cancelled)?;
    for file in &library.files {
        for offset in 0..=file.text.len() {
            if !file.text.is_char_boundary(offset) {
                assert_eq!(
                    index
                        .source_at(&file.path, &file.sha256, offset, &cancelled)
                        .err(),
                    Some(LookupError::InvalidPosition)
                );
                continue;
            }
            assert_eq!(
                serde_json::to_value(index.source_at(
                    &file.path,
                    &file.sha256,
                    offset,
                    &cancelled
                )?)?,
                serde_json::to_value(source_at(
                    &library,
                    &file.path,
                    &file.sha256,
                    offset,
                    &cancelled
                )?)?,
                "byte {offset}"
            );
        }
        for encoding in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            for line in [0, 1, 2, 5, 7] {
                let position = TextPosition { line, character: 0 };
                let expected = source_at_position(
                    &library,
                    &file.path,
                    &file.sha256,
                    position,
                    encoding,
                    &cancelled,
                );
                let actual = index.source_at_position(
                    &file.path,
                    &file.sha256,
                    position,
                    encoding,
                    &cancelled,
                );
                match (actual, expected) {
                    (Ok(actual), Ok(expected)) => {
                        assert_eq!(
                            serde_json::to_value(actual)?,
                            serde_json::to_value(expected)?
                        );
                    }
                    (Err(actual), Err(expected)) => assert_eq!(actual, expected),
                    _ => return Err("encoded query mismatch".into()),
                }
            }
        }
        assert_eq!(
            index
                .source_at(&file.path, "sha256:stale", 0, &cancelled)
                .err(),
            Some(LookupError::StaleArtifact)
        );
    }
    cancelled.store(true, Ordering::Relaxed);
    assert_eq!(
        index.source_at("missing", "stale", 0, &cancelled).err(),
        Some(LookupError::Cancelled)
    );
    assert_eq!(
        NavigationIndex::new(&library, &cancelled).err(),
        Some(LookupError::Cancelled)
    );
    Ok(())
}

#[test]
fn reverse_query_returns_each_use_of_one_shared_source_descriptor() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let file = library.files.first().ok_or("file")?;
    let member = file
        .mappings
        .iter()
        .find(|map| map.granularity == "parameter")
        .ok_or("member")?;
    let GeneratedLookup::Mapped { candidates } =
        index.generated_for(REV, &member.source, &cancelled)?
    else {
        return Err("source did not map".into());
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates[0].generated.start < candidates[1].generated.start);
    for candidate in candidates {
        assert_eq!(candidate.source_revision, REV);
        let SourceLookup::Mapped { candidates, .. } = index.source_at(
            candidate.path,
            candidate.sha256,
            candidate.generated.start,
            &cancelled,
        )?
        else {
            return Err("reverse result did not map back".into());
        };
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.source.span == member.source.span)
        );
    }
    assert_eq!(
        index.generated_for(DONOR, &member.source, &cancelled).err(),
        Some(LookupError::StaleArtifact)
    );
    let mut unrecorded = member.source.clone();
    unrecorded.span.start += 1;
    assert!(matches!(
        index.generated_for(REV, &unrecorded, &cancelled)?,
        GeneratedLookup::Unmapped
    ));
    unrecorded.sha256 = source_digest(b"changed");
    assert_eq!(
        index.generated_for(REV, &unrecorded, &cancelled).err(),
        Some(LookupError::InvalidMapping)
    );
    Ok(())
}

#[test]
fn reverse_catalog_queries_keep_independent_revision_and_scope() -> Result<()> {
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
    let library = project_with_alias_catalog(&docs, "Mainline", None, Some(&catalog), &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let alias = library
        .files
        .last()
        .ok_or("alias file")?
        .mappings
        .first()
        .ok_or("alias map")?;
    let GeneratedLookup::Mapped { candidates } =
        index.generated_for(DONOR, &alias.source, &cancelled)?
    else {
        return Err("catalog source did not map".into());
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source_revision, DONOR);
    assert_eq!(candidates[0].source.scope, Some("annotation_alias_catalog"));
    assert_eq!(
        index.generated_for(REV, &alias.source, &cancelled).err(),
        Some(LookupError::StaleArtifact)
    );
    let mut wrong_scope = alias.source.clone();
    wrong_scope.scope = None;
    assert_eq!(
        index.generated_for(REV, &wrong_scope, &cancelled).err(),
        Some(LookupError::InvalidMapping)
    );
    Ok(())
}

#[test]
fn preparation_rejects_inconsistent_files_and_maps_before_any_query() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    for mutation in 0..5 {
        let mut library = project(&docs, "Mainline", &cancelled)?;
        let file = library.files.first_mut().ok_or("file")?;
        let expected = match mutation {
            0 => {
                file.text.push(' ');
                LookupError::StaleArtifact
            }
            1 => {
                file.mappings[0].generated.end = usize::MAX;
                LookupError::InvalidMapping
            }
            2 => {
                file.mappings[0].source.sha256 = source_digest(b"wrong");
                LookupError::InvalidMapping
            }
            3 => {
                file.mappings[0].granularity = "unknown";
                LookupError::UnsupportedProfile
            }
            _ => {
                let duplicate = file.clone();
                library.files.push(duplicate);
                LookupError::InvalidMapping
            }
        };
        assert_eq!(
            NavigationIndex::new(&library, &cancelled).err(),
            Some(expected)
        );
    }
    Ok(())
}
