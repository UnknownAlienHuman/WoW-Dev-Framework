//! Convert actual navigation results without losing their bound byte locations.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project;
use wow_annotations::navigation::{LookupError, NavigationIndex, PositionEncoding, SourceFile};
use wow_reference::native::{Span, ingest_document, source_digest};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_and_generated_result_ranges_round_trip_through_public_queries() -> Result<()> {
    let cancelled = AtomicBool::new(false);
    let raw = "APIDocumentation:AddDocumentationTable({Name=\"Probe\",Type=\"System\",\r\nNamespace=\"C_Probe\",Functions={{Name=\"Read\",Documentation={\"é 🦀\"},\rArguments={{Name=\"value\",Type=\"number\"}},Returns={{Name=\"ok\",Type=\"bool\"}}}}})";
    let docs = [ingest_document(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "Types.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &cancelled,
    )?];
    let library = project(&docs, "Mainline", &cancelled)?;
    let index = NavigationIndex::new(&library, &cancelled)?;
    let source = index.bind_source(
        SourceFile {
            scope: None,
            revision: docs[0].revision(),
            path: docs[0].path(),
            sha256: docs[0].sha256(),
        },
        raw,
        &cancelled,
    )?;
    let file = library.files.first().ok_or("generated file")?;
    assert!(!file.mappings.is_empty());
    for encoding in [
        PositionEncoding::Utf8,
        PositionEncoding::Utf16,
        PositionEncoding::Utf32,
    ] {
        for map in &file.mappings {
            let generated = index.generated_text_range(
                &file.path,
                &file.sha256,
                map.generated,
                encoding,
                &cancelled,
            )?;
            let by_text = index.source_for_text_range(
                &file.path,
                &file.sha256,
                generated,
                encoding,
                &cancelled,
            )?;
            let by_bytes = index.source_for_range(
                &file.path,
                &file.sha256,
                map.generated,
                &cancelled,
            )?;
            assert_eq!(
                serde_json::to_value(by_text)?,
                serde_json::to_value(by_bytes)?
            );
            let original = source.source_text_range(map.source.span, encoding, &cancelled)?;
            let by_text = source.generated_for_text_range(original, encoding, &cancelled)?;
            let by_bytes = source.generated_for_range(map.source.span, &cancelled)?;
            assert_eq!(
                serde_json::to_value(by_text)?,
                serde_json::to_value(by_bytes)?
            );
        }
        let invalid = Span {
            start: 0,
            end: usize::MAX,
        };
        assert_eq!(
            index.generated_text_range(&file.path, "sha256:stale", invalid, encoding, &cancelled),
            Err(LookupError::StaleArtifact)
        );
        assert_eq!(
            index.generated_text_range("unknown.lua", &file.sha256, invalid, encoding, &cancelled),
            Err(LookupError::UnknownGeneratedFile)
        );
        assert_eq!(
            index.generated_text_range(&file.path, &file.sha256, invalid, encoding, &cancelled),
            Err(LookupError::InvalidPosition)
        );
        assert_eq!(
            source.source_text_range(invalid, encoding, &cancelled),
            Err(LookupError::InvalidPosition)
        );
        let stopped = AtomicBool::new(true);
        assert_eq!(
            index.generated_text_range(&file.path, &file.sha256, invalid, encoding, &stopped),
            Err(LookupError::Cancelled)
        );
        assert_eq!(
            source.source_text_range(invalid, encoding, &stopped),
            Err(LookupError::Cancelled)
        );
    }
    Ok(())
}
