//! Read-only navigation from final annotation bytes to exact source descriptors.
//!
//! This consumes an existing native generation; it does not parse Lua, execute
//! an analyzer, authenticate an external artifact, or infer API absence. The
//! caller must retain the generation and provide the digest of the viewed file.
use crate::native::{AnnotationFile, NativeLibrary, SourceLink};
use serde::Serialize;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::{Span, source_digest};

mod positions;
mod sources;
pub use positions::{PositionEncoding, TextPosition, source_at_position};

const MAX_FILES: usize = 4096;
const MAX_MAPPINGS: usize = 131_072;
const MAX_CANDIDATES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingPrecision {
    Member,
    Declaration,
    WholeFile,
}

/// All source identities are borrowed from the selected generation. External
/// alias revisions stay separate from the enclosing Blizzard revision.
#[derive(Clone, Debug, Serialize)]
pub struct SourceLocation<'a> {
    pub generated: Span,
    pub granularity: &'static str,
    pub revision: &'a str,
    pub source: &'a SourceLink,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SourceLookup<'a> {
    /// No stored map covers this byte. This never means the source API is absent.
    Unmapped,
    Mapped {
        precision: MappingPrecision,
        /// Equally precise matches are retained, not resolved by first-wins.
        candidates: Vec<SourceLocation<'a>>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupError {
    Cancelled,
    InputLimit,
    UnknownGeneratedFile,
    StaleArtifact,
    InvalidPosition,
    InvalidMapping,
    UnsupportedProfile,
}

impl fmt::Display for LookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "annotation navigation was cancelled",
            Self::InputLimit => "annotation navigation input exceeds its bound",
            Self::UnknownGeneratedFile => "generated file is not in this library",
            Self::StaleArtifact => "generated file bytes do not match the requested digest",
            Self::InvalidPosition => "position is not a valid code-point boundary in the file",
            Self::InvalidMapping => "annotation source-map identity or range is invalid",
            Self::UnsupportedProfile => "annotation navigation profile is unsupported",
        })
    }
}

impl std::error::Error for LookupError {}

/// Resolve a zero-based UTF-8 byte offset in a generated annotation file.
///
/// Half-open member ranges take precedence over declaration and whole-file
/// ranges. Within the same precision, the smallest range wins; all ties remain.
/// EOF and unmapped headers return Unmapped. Mid-codepoint positions, stale
/// bytes, unknown map kinds and invalid links fail instead of guessing. Existing
/// partial generations are usable, but a match adds no runtime/absence authority.
pub fn source_at<'a>(
    library: &'a NativeLibrary<'_>,
    generated_path: &str,
    expected_sha256: &str,
    byte_offset: usize,
    cancelled: &AtomicBool,
) -> Result<SourceLookup<'a>, LookupError> {
    let file = viewed_file(library, generated_path, expected_sha256, cancelled)?;
    source_in_file(library, file, byte_offset, cancelled)
}

/// Bind both navigation entry points to the same immutable viewed-file bytes.
fn viewed_file<'a>(
    library: &'a NativeLibrary<'_>,
    generated_path: &str,
    expected_sha256: &str,
    cancelled: &AtomicBool,
) -> Result<&'a AnnotationFile, LookupError> {
    check_cancelled(cancelled)?;
    if !matches!(
        library.schema,
        "wow-native-annotation-library/3"
            | "wow-native-annotation-library/4"
            | "wow-native-annotation-library/5"
            | "wow-native-annotation-library/6"
    ) {
        return Err(LookupError::UnsupportedProfile);
    }
    if library.source_map_profile != "wow-native-field-maps/1" {
        return Err(LookupError::UnsupportedProfile);
    }
    if library.negative_authority {
        return Err(LookupError::InvalidMapping);
    }
    if library.files.len() > MAX_FILES {
        return Err(LookupError::InputLimit);
    }
    let mut files = library
        .files
        .iter()
        .filter(|file| file.path == generated_path);
    let file = files.next().ok_or(LookupError::UnknownGeneratedFile)?;
    if files.next().is_some() {
        return Err(LookupError::InvalidMapping);
    }
    if file.text.len() > crate::ketho::MAX_OUTPUT_BYTES || file.mappings.len() > MAX_MAPPINGS {
        return Err(LookupError::InputLimit);
    }
    if file.sha256 != expected_sha256 || source_digest(file.text.as_bytes()) != expected_sha256 {
        return Err(LookupError::StaleArtifact);
    }
    check_cancelled(cancelled)?;
    Ok(file)
}

fn source_in_file<'a>(
    library: &'a NativeLibrary<'_>,
    file: &'a AnnotationFile,
    byte_offset: usize,
    cancelled: &AtomicBool,
) -> Result<SourceLookup<'a>, LookupError> {
    check_cancelled(cancelled)?;
    if byte_offset > file.text.len() || !file.text.is_char_boundary(byte_offset) {
        return Err(LookupError::InvalidPosition);
    }
    let sources = sources::Sources::new(library, cancelled)?;
    let mut best = None;
    // Validate the complete selected map set before returning a local match.
    // Otherwise an invalid sibling could become an apparently clean Unmapped.
    for mapping in &file.mappings {
        check_cancelled(cancelled)?;
        let rank = rank(mapping.granularity)?;
        let span = mapping.generated;
        if span.start >= span.end || file.text.get(span.start..span.end).is_none() {
            return Err(LookupError::InvalidMapping);
        }
        sources.revision(&mapping.source)?;
        if span.start <= byte_offset && byte_offset < span.end {
            let key = (rank, span.end - span.start);
            if best.is_none_or(|previous| key < previous) {
                best = Some(key);
            }
        }
    }
    let Some(best) = best else {
        check_cancelled(cancelled)?;
        return Ok(SourceLookup::Unmapped);
    };
    let mut candidates = Vec::new();
    for mapping in &file.mappings {
        check_cancelled(cancelled)?;
        let span = mapping.generated;
        if span.start <= byte_offset
            && byte_offset < span.end
            && (rank(mapping.granularity)?, span.end - span.start) == best
        {
            if candidates.len() >= MAX_CANDIDATES {
                return Err(LookupError::InputLimit);
            }
            candidates.push(SourceLocation {
                generated: span,
                granularity: mapping.granularity,
                revision: sources.revision(&mapping.source)?,
                source: &mapping.source,
            });
        }
    }
    check_cancelled(cancelled)?;
    Ok(SourceLookup::Mapped {
        precision: match best.0 {
            0 => MappingPrecision::Member,
            1 => MappingPrecision::Declaration,
            _ => MappingPrecision::WholeFile,
        },
        candidates,
    })
}

fn rank(granularity: &str) -> Result<u8, LookupError> {
    match granularity {
        "parameter" | "return" | "field" => Ok(0),
        "declaration" => Ok(1),
        "literal_file" => Ok(2),
        _ => Err(LookupError::UnsupportedProfile),
    }
}

fn check_cancelled(cancelled: &AtomicBool) -> Result<(), LookupError> {
    if cancelled.load(Ordering::Relaxed) {
        Err(LookupError::Cancelled)
    } else {
        Ok(())
    }
}
