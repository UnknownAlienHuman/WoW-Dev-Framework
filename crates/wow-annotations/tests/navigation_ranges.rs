//! Whole diagnostic spans must not inherit the first member's precision.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project;
use wow_annotations::navigation::{
    LookupError, MappingPrecision, PositionEncoding, SourceLookup, TextPosition, TextRange,
    source_at, source_for_range, source_for_text_range,
};
use wow_reference::native::{DocumentationDocument, Span, ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RAW: &str = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="Read",Documentation={"é 🦀"},Arguments={{Name="id",Type="number"},{Name="mode",Type="string"}},Returns={{Name="ok",Type="bool"}}},
{Name="Write",Arguments={{Name="text",Type="string"}}}
}})"#;

fn document() -> Result<DocumentationDocument> {
    Ok(ingest_document(
        REV,
        "Range.lua",
        RAW,
        &source_digest(RAW.as_bytes()),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn entire_selection_controls_precision_and_empty_ranges_keep_caret_semantics() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first().ok_or("generated file")?;
    let members = file
        .mappings
        .iter()
        .filter(|mapping| mapping.granularity == "parameter")
        .collect::<Vec<_>>();
    let first = members.first().ok_or("first parameter")?.generated;
    let second = members.get(1).ok_or("second parameter")?.generated;
    let third = members.get(2).ok_or("other declaration")?.generated;
    for (span, expected) in [
        (first, MappingPrecision::Member),
        (
            Span {
                start: first.start,
                end: second.end,
            },
            MappingPrecision::Declaration,
        ),
    ] {
        let lookup = source_for_range(&library, &file.path, &file.sha256, span, &cancelled)?;
        let SourceLookup::Mapped {
            precision,
            candidates,
        } = lookup
        else {
            return Err("range did not map".into());
        };
        assert_eq!(precision, expected);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].revision, REV);
    }
    assert!(matches!(
        source_for_range(
            &library,
            &file.path,
            &file.sha256,
            Span {
                start: first.start,
                end: third.end,
            },
            &cancelled,
        )?,
        SourceLookup::Unmapped
    ));
    for offset in [0, first.start, first.end, file.text.len()] {
        let point = source_at(&library, &file.path, &file.sha256, offset, &cancelled)?;
        let range = source_for_range(
            &library,
            &file.path,
            &file.sha256,
            Span {
                start: offset,
                end: offset,
            },
            &cancelled,
        )?;
        assert_eq!(serde_json::to_value(point)?, serde_json::to_value(range)?);
    }
    Ok(())
}

#[test]
fn range_queries_retain_full_validation_and_encoded_endpoint_rules() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first().ok_or("generated file")?;
    let path = file.path.clone();
    let digest = file.sha256.clone();
    let unicode = file.text.find('🦀').ok_or("Unicode text")?;
    let line_start = file.text[..unicode].rfind('\n').map_or(0, |at| at + 1);
    let line = u32::try_from(
        file.text[..unicode]
            .bytes()
            .filter(|b| *b == b'\n')
            .count(),
    )?;
    let column = u32::try_from(file.text[line_start..unicode].encode_utf16().count())?;
    let span = Span {
        start: unicode,
        end: unicode + '🦀'.len_utf8(),
    };
    let encoded = TextRange {
        start: TextPosition {
            line,
            character: column,
        },
        end: TextPosition {
            line,
            character: column + 2,
        },
    };
    let expected = source_for_range(&library, &path, &digest, span, &cancelled)?;
    let actual = source_for_text_range(
        &library,
        &path,
        &digest,
        encoded,
        PositionEncoding::Utf16,
        &cancelled,
    )?;
    assert_eq!(serde_json::to_value(expected)?, serde_json::to_value(actual)?);
    for invalid in [
        Span {
            start: unicode + 1,
            end: span.end,
        },
        Span {
            start: unicode,
            end: unicode + 1,
        },
        Span {
            start: span.end,
            end: span.start,
        },
        Span {
            start: 0,
            end: usize::MAX,
        },
    ] {
        assert_eq!(
            source_for_range(&library, &path, &digest, invalid, &cancelled).err(),
            Some(LookupError::InvalidPosition)
        );
    }
    for invalid in [
        TextRange {
            start: encoded.end,
            end: encoded.start,
        },
        TextRange {
            start: encoded.start,
            end: TextPosition {
                line,
                character: column + 1,
            },
        },
    ] {
        assert_eq!(
            source_for_text_range(
                &library,
                &path,
                &digest,
                invalid,
                PositionEncoding::Utf16,
                &cancelled,
            )
            .err(),
            Some(LookupError::InvalidPosition)
        );
    }
    assert_eq!(
        source_for_range(&library, &path, "wrong", span, &cancelled).err(),
        Some(LookupError::StaleArtifact)
    );
    assert_eq!(
        source_for_range(&library, &path, &digest, span, &AtomicBool::new(true)).err(),
        Some(LookupError::Cancelled)
    );
    // A malformed sibling invalidates even a range over the unmapped header.
    library.files[0]
        .mappings
        .last_mut()
        .ok_or("last mapping")?
        .source
        .sha256 = "wrong".into();
    assert_eq!(
        source_for_range(
            &library,
            &path,
            &digest,
            Span { start: 0, end: 1 },
            &cancelled,
        )
        .err(),
        Some(LookupError::InvalidMapping)
    );
    Ok(())
}

#[test]
fn indexed_ranges_match_one_shot_queries_and_keep_equal_candidates() -> Result<()> {
    use std::collections::BTreeSet;
    use wow_annotations::navigation::NavigationIndex;

    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first_mut().ok_or("generated file")?;
    // An equal mapping is intentionally retained, not resolved by first-wins.
    let duplicate = file
        .mappings
        .iter()
        .find(|mapping| mapping.granularity == "parameter")
        .ok_or("parameter map")?
        .clone();
    file.mappings.push(duplicate.clone());
    let mut boundaries = BTreeSet::from([0, file.text.len()]);
    for mapping in &file.mappings {
        boundaries.extend([mapping.generated.start, mapping.generated.end]);
    }
    let file = library.files.first().ok_or("generated file")?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    for &start in &boundaries {
        for &end in &boundaries {
            let range = Span { start, end };
            let expected = source_for_range(&library, &file.path, &file.sha256, range, &cancelled);
            let actual = index.source_for_range(&file.path, &file.sha256, range, &cancelled);
            match (actual, expected) {
                (Ok(actual), Ok(expected)) => {
                    assert_eq!(serde_json::to_value(actual)?, serde_json::to_value(expected)?);
                }
                (Err(actual), Err(expected)) => assert_eq!(actual, expected),
                _ => return Err("indexed range result differs".into()),
            }
        }
    }
    let SourceLookup::Mapped { candidates, .. } = index.source_for_range(
        &file.path,
        &file.sha256,
        duplicate.generated,
        &cancelled,
    )? else {
        return Err("duplicate member did not map".into());
    };
    assert_eq!(candidates.len(), 2);
    let unicode = file.text.find('🦀').ok_or("Unicode text")?;
    let line_start = file.text[..unicode].rfind('\n').map_or(0, |at| at + 1);
    let line = u32::try_from(file.text[..unicode].bytes().filter(|b| *b == b'\n').count())?;
    for encoding in [PositionEncoding::Utf8, PositionEncoding::Utf16, PositionEncoding::Utf32] {
        let prefix = &file.text[line_start..unicode];
        let (column, width) = match encoding {
            PositionEncoding::Utf8 => (prefix.len(), '🦀'.len_utf8()),
            PositionEncoding::Utf16 => (prefix.encode_utf16().count(), '🦀'.len_utf16()),
            PositionEncoding::Utf32 => (prefix.chars().count(), 1),
        };
        let range = TextRange {
            start: TextPosition { line, character: u32::try_from(column)? },
            end: TextPosition { line, character: u32::try_from(column + width)? },
        };
        let expected = source_for_text_range(
            &library, &file.path, &file.sha256, range, encoding, &cancelled,
        )?;
        let actual = index.source_for_text_range(
            &file.path, &file.sha256, range, encoding, &cancelled,
        )?;
        assert_eq!(serde_json::to_value(actual)?, serde_json::to_value(expected)?);
    }
    assert_eq!(
        index.source_for_range(&file.path, "stale", duplicate.generated, &cancelled).err(),
        Some(LookupError::StaleArtifact)
    );
    assert_eq!(
        index.source_for_range(
            &file.path, &file.sha256, duplicate.generated, &AtomicBool::new(true),
        ).err(),
        Some(LookupError::Cancelled)
    );
    for range in [
        Span { start: unicode + 1, end: unicode + 4 },
        Span { start: 0, end: usize::MAX },
    ] {
        assert_eq!(
            index.source_for_range(&file.path, &file.sha256, range, &cancelled).err(),
            Some(LookupError::InvalidPosition)
        );
    }
    Ok(())
}
