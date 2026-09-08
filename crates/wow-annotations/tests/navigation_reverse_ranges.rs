//! Retained source-selection cases through the reconciled direct public API.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::{project, project_with_alias_catalog};
use wow_annotations::navigation::{
    GeneratedLocation, GeneratedLookup, LookupError, NavigationIndex, PositionEncoding,
    SourceFile, SourceText, TextPosition, TextRange,
};
use wow_reference::native::{DocumentationDocument, Span, ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const RAW: &str = r#"local shared = {{Name="value",Type="number"}}
APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="First",Documentation={"é 🦀"},Arguments=shared},
{Name="Second",Arguments=shared},
{Name="Pair",Arguments={{Name="left",Type="number"},{Name="right",Type="string"}}}
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

fn view(document: &DocumentationDocument) -> SourceText<'_> {
    SourceText {
        scope: None,
        revision: document.revision(),
        path: document.path(),
        sha256: document.sha256(),
        text: RAW,
    }
}

fn locations<'a>(lookup: GeneratedLookup<'a>) -> Result<Vec<GeneratedLocation<'a>>> {
    match lookup {
        GeneratedLookup::Mapped { candidates } => Ok(candidates),
        GeneratedLookup::Unmapped => Err("expected mapped source selection".into()),
    }
}

#[test]
fn shared_field_selections_keep_all_ties_and_the_exact_lookup_contract() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first_mut().ok_or("file")?;
    let member = file
        .mappings
        .iter()
        .find(|map| map.granularity == "parameter")
        .ok_or("member")?
        .clone();
    file.mappings.push(member.clone());
    let index = NavigationIndex::new(&library, &cancelled)?;
    let selected = Span {
        start: member.source.span.start + 1,
        end: member.source.span.end - 1,
    };
    let candidates = locations(index.generated_for_range(view(&docs[0]), selected, &cancelled)?)?;
    assert_eq!(candidates.len(), 3);
    for candidate in &candidates {
        assert_eq!(candidate.source.span, member.source.span);
        assert_eq!(candidate.source_revision, REV);
        assert_eq!(candidate.granularity, "parameter");
    }
    assert!(candidates.windows(2).all(|pair| {
        (pair[0].path, pair[0].generated.start, pair[0].generated.end)
            <= (pair[1].path, pair[1].generated.start, pair[1].generated.end)
    }));
    let mut exact = member.source.clone();
    exact.span = selected;
    assert!(matches!(
        index.generated_for(REV, &exact, &cancelled)?,
        GeneratedLookup::Unmapped
    ));
    assert_eq!(
        locations(index.generated_for(REV, &member.source, &cancelled)?)?.len(),
        3
    );
    Ok(())
}

#[test]
fn crossing_members_selects_a_declaration_without_joining_unrelated_ones() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let left = RAW.find("{Name=\"left\"").ok_or("left field")?;
    let right = RAW.find("Type=\"string\"").ok_or("right field")?;
    let candidates = locations(index.generated_for_range(
        view(&docs[0]),
        Span {
            start: left,
            end: right + "Type=\"string\"".len(),
        },
        &cancelled,
    )?)?;
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].granularity, "declaration");
    let source = candidates[0].source.span;
    let descriptor = RAW.get(source.start..source.end).ok_or("span")?;
    assert!(descriptor.contains("Pair"));
    let first = RAW.find("{Name=\"First\"").ok_or("first declaration")?;
    let second = RAW.find("{Name=\"Second\"").ok_or("second declaration")?;
    for selected in [
        Span {
            start: first,
            end: second + 1,
        },
        Span { start: 0, end: 0 },
        Span {
            start: RAW.len(),
            end: RAW.len(),
        },
    ] {
        assert!(matches!(
            index.generated_for_range(view(&docs[0]), selected, &cancelled)?,
            GeneratedLookup::Unmapped
        ));
    }
    Ok(())
}

#[test]
fn direct_and_bound_queries_agree_for_all_encodings_and_reject_invalid_boundaries() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let source = view(&docs[0]);
    let bound = index.bind_source(
        SourceFile {
            scope: source.scope,
            revision: source.revision,
            path: source.path,
            sha256: source.sha256,
        },
        source.text,
        &cancelled,
    )?;
    let offset = RAW.find('🦀').ok_or("Unicode source")?;
    let prefix = RAW[..offset].rsplit('\n').next().ok_or("line")?;
    let line = u32::try_from(RAW[..offset].bytes().filter(|b| *b == b'\n').count())?;
    let selected = Span {
        start: offset,
        end: offset + '🦀'.len_utf8(),
    };
    let expected = serde_json::to_value(bound.generated_for_range(selected, &cancelled)?)?;
    let direct = index.generated_for_range(source, selected, &cancelled)?;
    assert_eq!(serde_json::to_value(direct)?, expected);
    for encoding in [
        PositionEncoding::Utf8,
        PositionEncoding::Utf16,
        PositionEncoding::Utf32,
    ] {
        let (character, width) = match encoding {
            PositionEncoding::Utf8 => (prefix.len(), '🦀'.len_utf8()),
            PositionEncoding::Utf16 => (prefix.encode_utf16().count(), '🦀'.len_utf16()),
            PositionEncoding::Utf32 => (prefix.chars().count(), 1),
        };
        let range = TextRange {
            start: TextPosition {
                line,
                character: u32::try_from(character)?,
            },
            end: TextPosition {
                line,
                character: u32::try_from(character + width)?,
            },
        };
        let actual = index.generated_for_text_range(source, range, encoding, &cancelled)?;
        assert_eq!(serde_json::to_value(actual)?, expected);
        let prepared = bound.generated_for_text_range(range, encoding, &cancelled)?;
        assert_eq!(serde_json::to_value(prepared)?, expected);
        let reversed = TextRange {
            start: range.end,
            end: range.start,
        };
        let rejected = index.generated_for_text_range(source, reversed, encoding, &cancelled);
        assert_eq!(rejected.err(), Some(LookupError::InvalidPosition));
    }
    for range in [
        Span {
            start: offset + 1,
            end: offset + 4,
        },
        Span {
            start: offset,
            end: offset + 1,
        },
        Span {
            start: offset + 4,
            end: offset,
        },
        Span {
            start: 0,
            end: usize::MAX,
        },
    ] {
        let rejected = index.generated_for_range(source, range, &cancelled);
        assert_eq!(rejected.err(), Some(LookupError::InvalidPosition));
    }
    let range = TextRange {
        start: TextPosition {
            line,
            character: u32::try_from(prefix.encode_utf16().count() + 1)?,
        },
        end: TextPosition {
            line,
            character: u32::try_from(prefix.encode_utf16().count() + 2)?,
        },
    };
    let rejected =
        index.generated_for_text_range(source, range, PositionEncoding::Utf16, &cancelled);
    assert_eq!(rejected.err(), Some(LookupError::InvalidPosition));
    Ok(())
}

#[test]
fn stale_sources_and_cancellation_cannot_produce_unmapped_success() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let original = view(&docs[0]);
    let changed = RAW.replacen("First", "Other", 1);
    let cursor = Span { start: 0, end: 0 };
    for source in [
        SourceText {
            revision: DONOR,
            ..original
        },
        SourceText {
            sha256: "sha256:stale",
            ..original
        },
        SourceText {
            text: &changed,
            ..original
        },
        SourceText {
            text: "",
            ..original
        },
    ] {
        let rejected = index.generated_for_range(source, cursor, &cancelled);
        assert_eq!(rejected.err(), Some(LookupError::StaleArtifact));
    }
    let unknown = SourceText {
        scope: Some("unknown"),
        ..original
    };
    let rejected = index.generated_for_range(unknown, cursor, &cancelled);
    assert_eq!(rejected.err(), Some(LookupError::UnsupportedProfile));
    let rejected = index.generated_for_range(original, cursor, &AtomicBool::new(true));
    assert_eq!(rejected.err(), Some(LookupError::Cancelled));
    Ok(())
}

#[test]
fn identical_paths_keep_independent_catalog_revisions() -> Result<()> {
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
    let source = SourceText {
        scope: Some("annotation_alias_catalog"),
        revision: DONOR,
        path: "Types.lua",
        sha256: catalog.sha256(),
        text: raw,
    };
    let start = raw.find("Quantity").ok_or("alias")?;
    let range = Span {
        start,
        end: start + "Quantity".len(),
    };
    let candidates = locations(index.generated_for_range(source, range, &cancelled)?)?;
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].source_revision, DONOR);
    assert_eq!(candidates[0].source.scope, Some("annotation_alias_catalog"));
    let wrong_scope = SourceText {
        scope: None,
        ..source
    };
    let rejected = index.generated_for_range(wrong_scope, range, &cancelled);
    assert_eq!(rejected.err(), Some(LookupError::StaleArtifact));
    Ok(())
}

#[test]
fn supplied_source_text_rejects_invalid_nonmatching_sibling_boundaries() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let mut library = project(&docs, "Mainline", &cancelled)?;
    let file = library.files.first_mut().ok_or("file")?;
    let mut invalid = file.mappings.first().ok_or("mapping")?.clone();
    let offset = RAW.find('🦀').ok_or("Unicode source")?;
    invalid.source.span = Span {
        start: offset + 1,
        end: offset + 4,
    };
    file.mappings.push(invalid);
    let index = NavigationIndex::new(&library, &cancelled)?;
    let cursor = Span { start: 0, end: 0 };
    let rejected = index.generated_for_range(view(&docs[0]), cursor, &cancelled);
    assert_eq!(rejected.err(), Some(LookupError::InvalidMapping));
    Ok(())
}

#[test]
fn returned_locations_do_not_borrow_the_temporary_source_buffer() -> Result<()> {
    let docs = [document()?];
    let cancelled = AtomicBool::new(false);
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let locations = {
        let owned = RAW.to_owned();
        let source = SourceText {
            text: &owned,
            ..view(&docs[0])
        };
        let start = RAW.find("value").ok_or("shared field")?;
        let range = Span {
            start,
            end: start + "value".len(),
        };
        locations(index.generated_for_range(source, range, &cancelled)?)?
    };
    assert_eq!(locations.len(), 2);
    assert!(locations.iter().all(|item| item.source_revision == REV));
    Ok(())
}
