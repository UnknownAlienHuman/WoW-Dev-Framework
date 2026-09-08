//! Source-side cursor queries preserve exact generation identity and every use.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{project, project_with_alias_catalog};
use wow_annotations::navigation::{
    GeneratedLookup, LookupError, NavigationIndex, PositionEncoding, SourceFile, TextPosition,
    TextRange,
};
use wow_reference::native::{DocumentationDocument, Span, ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RAW: &str = r#"local args = {{Name="value",Type="number"}}
APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="First",Documentation={"é 🦀"},Arguments=args,Returns={{Name="result",Type="string"}}},
{Name="Second",Arguments=args}
}})"#;

fn read(raw: &str, path: &str) -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        path,
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

fn identity(document: &DocumentationDocument) -> SourceFile<'_> {
    SourceFile {
        scope: None,
        revision: document.revision(),
        path: document.path(),
        sha256: document.sha256(),
    }
}

#[test]
fn source_cursor_and_selection_find_every_shared_descriptor_occurrence() -> Result<()> {
    let docs = [read(RAW, "Types.lua")?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let source = index.bind_source(identity(&docs[0]), RAW, &cancelled)?;
    let file = library.files.first().ok_or("file")?;
    let member = file
        .mappings
        .iter()
        .find(|m| m.granularity == "parameter")
        .ok_or("member")?;
    let expected = serde_json::to_value(index.generated_for(REV, &member.source, &cancelled)?)?;
    let actual = source.generated_at(member.source.span.start + 1, &cancelled)?;
    let GeneratedLookup::Mapped { candidates } = &actual else {
        return Err("shared field did not map".into());
    };
    assert_eq!(candidates.len(), 2);
    assert_eq!(serde_json::to_value(actual)?, expected);
    assert_eq!(
        serde_json::to_value(source.generated_for_range(member.source.span, &cancelled)?)?,
        expected
    );
    let declarations = file
        .mappings
        .iter()
        .filter(|m| m.granularity == "declaration")
        .collect::<Vec<_>>();
    let first = declarations.first().ok_or("first")?;
    let second = declarations.get(1).ok_or("second")?;
    let GeneratedLookup::Mapped { candidates } =
        source.generated_for_range(first.source.span, &cancelled)?
    else {
        return Err("whole declaration did not map".into());
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].generated, first.generated);
    let crossing = Span {
        start: first.source.span.start,
        end: second.source.span.end,
    };
    assert!(matches!(
        source.generated_for_range(crossing, &cancelled)?,
        GeneratedLookup::Unmapped
    ));
    for offset in [0, member.source.span.end, RAW.len()] {
        assert!(matches!(
            source.generated_at(offset, &cancelled)?,
            GeneratedLookup::Unmapped
        ));
    }
    Ok(())
}

#[test]
fn encoded_source_queries_preserve_unicode_and_invalid_endpoint_rules() -> Result<()> {
    let cancelled = AtomicBool::new(false);
    for newline in ["\n", "\r\n", "\r"] {
        let raw = RAW.replace('\n', newline);
        let docs = [read(&raw, "Types.lua")?];
        let library = project(&docs, "Mainline", &cancelled)?;
        let index = NavigationIndex::new(&library, &cancelled)?;
        let source = index.bind_source(identity(&docs[0]), &raw, &cancelled)?;
        let offset = raw.find('🦀').ok_or("unicode")?;
        let line_start = raw[..offset].rfind(['\n', '\r']).map_or(0, |at| at + 1);
        let prefix = &raw[line_start..offset];
        let bytes = Span {
            start: offset,
            end: offset + '🦀'.len_utf8(),
        };
        let expected = serde_json::to_value(source.generated_for_range(bytes, &cancelled)?)?;
        for encoding in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            let (column, width) = match encoding {
                PositionEncoding::Utf8 => (prefix.len(), '🦀'.len_utf8()),
                PositionEncoding::Utf16 => (prefix.encode_utf16().count(), '🦀'.len_utf16()),
                PositionEncoding::Utf32 => (prefix.chars().count(), 1),
            };
            let start = TextPosition {
                line: 2,
                character: u32::try_from(column)?,
            };
            let end = TextPosition {
                line: 2,
                character: u32::try_from(column + width)?,
            };
            let result =
                source.generated_for_text_range(TextRange { start, end }, encoding, &cancelled)?;
            assert_eq!(serde_json::to_value(result)?, expected);
            assert_eq!(
                serde_json::to_value(source.generated_at_position(start, encoding, &cancelled)?)?,
                serde_json::to_value(source.generated_at(offset, &cancelled)?)?
            );
            let reversed = TextRange {
                start: end,
                end: start,
            };
            assert_eq!(
                source
                    .generated_for_text_range(reversed, encoding, &cancelled)
                    .err(),
                Some(LookupError::InvalidPosition)
            );
            if width > 1 {
                let interior = TextPosition {
                    character: start.character + 1,
                    ..start
                };
                assert_eq!(
                    source
                        .generated_at_position(interior, encoding, &cancelled)
                        .err(),
                    Some(LookupError::InvalidPosition)
                );
            }
        }
        for invalid in [
            Span {
                start: offset + 1,
                end: offset + 2,
            },
            Span {
                start: raw.len(),
                end: 0,
            },
            Span {
                start: 0,
                end: usize::MAX,
            },
        ] {
            assert_eq!(
                source.generated_for_range(invalid, &cancelled).err(),
                Some(LookupError::InvalidPosition)
            );
        }
        assert_eq!(
            source.generated_at(0, &AtomicBool::new(true)).err(),
            Some(LookupError::Cancelled)
        );
    }
    Ok(())
}

#[test]
fn source_binding_rejects_stale_text_revision_scope_and_invalid_source_boundaries() -> Result<()> {
    let docs = [read(RAW, "Types.lua")?];
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
    let id = SourceFile {
        scope: Some("annotation_alias_catalog"),
        revision: DONOR,
        path: catalog.path(),
        sha256: catalog.sha256(),
    };
    let source = index.bind_source(id, raw, &cancelled)?;
    let offset = raw.find("Quantity").ok_or("alias")?;
    let GeneratedLookup::Mapped { candidates } = source.generated_at(offset, &cancelled)? else {
        return Err("alias did not map".into());
    };
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source_revision, DONOR);
    assert_eq!(candidates[0].source.scope, id.scope);
    for bad in [
        SourceFile {
            revision: REV,
            ..id
        },
        SourceFile { scope: None, ..id },
        SourceFile {
            sha256: "sha256:stale",
            ..id
        },
    ] {
        assert!(matches!(
            index.bind_source(bad, raw, &cancelled),
            Err(LookupError::StaleArtifact)
        ));
    }
    let changed = raw.replace("number", "string");
    assert!(matches!(
        index.bind_source(id, &changed, &cancelled),
        Err(LookupError::StaleArtifact)
    ));
    assert!(matches!(
        index.bind_source(
            SourceFile {
                scope: Some("other"),
                ..id
            },
            raw,
            &cancelled
        ),
        Err(LookupError::UnsupportedProfile)
    ));
    assert!(matches!(
        index.bind_source(id, raw, &AtomicBool::new(true)),
        Err(LookupError::Cancelled)
    ));
    let mut invalid = project(&docs, "Mainline", &cancelled)?;
    let map = invalid
        .files
        .first_mut()
        .ok_or("file")?
        .mappings
        .first_mut()
        .ok_or("map")?;
    let offset = RAW.find('🦀').ok_or("unicode")?;
    map.source.span = Span {
        start: offset + 1,
        end: offset + 2,
    };
    let invalid = NavigationIndex::new(&invalid, &cancelled)?;
    assert!(matches!(
        invalid.bind_source(identity(&docs[0]), RAW, &cancelled),
        Err(LookupError::InvalidMapping)
    ));
    Ok(())
}

#[test]
fn a_valid_source_with_no_emitted_maps_binds_without_claiming_absence() -> Result<()> {
    let raw = "APIDocumentation:AddDocumentationTable({Name=\"Empty\",Type=\"System\"})";
    let docs = [read(raw, "Empty.lua")?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let source = index.bind_source(identity(&docs[0]), raw, &cancelled)?;
    assert!(matches!(
        source.generated_at(0, &cancelled)?,
        GeneratedLookup::Unmapped
    ));
    let range = Span {
        start: 0,
        end: raw.len(),
    };
    assert!(matches!(
        source.generated_for_range(range, &cancelled)?,
        GeneratedLookup::Unmapped
    ));
    Ok(())
}
