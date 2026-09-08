//! Prepared bidirectional queries over one immutably borrowed native generation.
use super::{
    LookupError, MAX_CANDIDATES, MappingPrecision, PositionEncoding, SourceLocation,
    SourceLookup, TextPosition, check_cancelled, positions, rank, sources,
    validate_file, validate_library,
};
use crate::native::{AnnotationFile, NativeLibrary, SourceLink};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use wow_reference::native::Span;

mod intervals;
mod reverse;

const MAX_INDEX_BYTES: usize = 64 * 1024 * 1024;
const MAX_INDEX_MAPPINGS: usize = 262_144;

/// An exact occurrence of the requested source descriptor in generated output.
#[derive(Clone, Debug, Serialize)]
pub struct GeneratedLocation<'a> {
    pub path: &'a str,
    pub sha256: &'a str,
    pub generated: Span,
    pub granularity: &'static str,
    pub source_revision: &'a str,
    pub source: &'a SourceLink,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GeneratedLookup<'a> {
    /// No exact descriptor map is retained; not source/API absence evidence.
    Unmapped,
    Mapped {
        /// All occurrences, ordered by generated path/range. Ties are preserved.
        candidates: Vec<GeneratedLocation<'a>>,
    },
}

struct IndexedFile<'a> {
    file: &'a AnnotationFile,
    intervals: intervals::Intervals,
}

/// Validates all generated files and map links once, then serves repeated queries.
///
/// The immutable borrow prevents editing a generation underneath this index.
/// Rebuild for a replacement generation. No global cache, IO, source discovery,
/// negative authority or promotion of an untrusted JSON artifact is introduced.
/// Preparation admits at most 64 MiB of generated text and 262,144 mappings in
/// total, in addition to the existing per-file and source-identity bounds.
pub struct NavigationIndex<'a> {
    files: BTreeMap<&'a str, IndexedFile<'a>>,
    sources: sources::Sources<'a>,
    reverse: reverse::Reverse<'a>,
}

impl<'a> NavigationIndex<'a> {
    pub fn new(
        library: &'a NativeLibrary<'_>,
        cancelled: &AtomicBool,
    ) -> Result<Self, LookupError> {
        validate_library(library, cancelled)?;
        let sources = sources::Sources::new(library, cancelled)?;
        let mut files = BTreeMap::new();
        let mut reverse = reverse::Reverse::default();
        let mut bytes = 0usize;
        let mut mappings = 0usize;
        for file in &library.files {
            check_cancelled(cancelled)?;
            bytes = bytes
                .checked_add(file.text.len())
                .ok_or(LookupError::InputLimit)?;
            mappings = mappings
                .checked_add(file.mappings.len())
                .ok_or(LookupError::InputLimit)?;
            if bytes > MAX_INDEX_BYTES || mappings > MAX_INDEX_MAPPINGS {
                return Err(LookupError::InputLimit);
            }
            if files.contains_key(file.path.as_str()) {
                return Err(LookupError::InvalidMapping);
            }
            validate_file(file, &file.sha256, cancelled)?;
            for mapping in &file.mappings {
                check_cancelled(cancelled)?;
                rank(mapping.granularity)?;
                let span = mapping.generated;
                if span.start >= span.end || file.text.get(span.start..span.end).is_none() {
                    return Err(LookupError::InvalidMapping);
                }
                let revision = sources.revision(&mapping.source)?;
                reverse.insert(GeneratedLocation {
                    path: &file.path,
                    sha256: &file.sha256,
                    generated: span,
                    granularity: mapping.granularity,
                    source_revision: revision,
                    source: &mapping.source,
                })?;
            }
            let intervals = intervals::Intervals::new(
                file.mappings.iter().map(|mapping| mapping.generated),
                cancelled,
            )?;
            files.insert(file.path.as_str(), IndexedFile { file, intervals });
        }
        reverse.order(cancelled)?;
        check_cancelled(cancelled)?;
        Ok(Self {
            files,
            sources,
            reverse,
        })
    }

    /// Same byte positions, precedence and candidate order as the one-shot query.
    /// The viewed digest is still required, but immutable bytes are not rehashed.
    pub fn source_at(
        &self,
        generated_path: &str,
        expected_sha256: &str,
        byte_offset: usize,
        cancelled: &AtomicBool,
    ) -> Result<SourceLookup<'a>, LookupError> {
        let file = self.viewed_file(generated_path, expected_sha256, cancelled)?;
        self.lookup(file, byte_offset, cancelled)
    }

    /// Uses the existing strict coordinate conversion without changing encodings
    /// or newline policy. Conversion still scans the selected file's line prefix.
    pub fn source_at_position(
        &self,
        generated_path: &str,
        expected_sha256: &str,
        position: TextPosition,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<SourceLookup<'a>, LookupError> {
        let file = self.viewed_file(generated_path, expected_sha256, cancelled)?;
        let offset = positions::byte_offset(&file.file.text, position, encoding, cancelled)?;
        self.lookup(file, offset, cancelled)
    }

    /// Find all generated occurrences of one exact source descriptor.
    ///
    /// Scope, revision, path, digest and half-open source range are mandatory.
    /// This is exact range equality, not an overlap/reference-search heuristic.
    /// A local descriptor reused by several callables returns every occurrence.
    /// Valid but unrecorded ranges yield Unmapped, never negative authority.
    pub fn generated_for(
        &self,
        source_revision: &str,
        source: &SourceLink,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'a>, LookupError> {
        check_cancelled(cancelled)?;
        if self.sources.revision(source)? != source_revision {
            return Err(LookupError::StaleArtifact);
        }
        let matches = self.reverse.get(source)?;
        if matches.len() > MAX_CANDIDATES {
            return Err(LookupError::InputLimit);
        }
        let mut candidates = Vec::with_capacity(matches.len());
        for location in matches {
            check_cancelled(cancelled)?;
            candidates.push(location.clone());
        }
        check_cancelled(cancelled)?;
        if candidates.is_empty() {
            Ok(GeneratedLookup::Unmapped)
        } else {
            Ok(GeneratedLookup::Mapped { candidates })
        }
    }

    fn viewed_file(
        &self,
        path: &str,
        expected_sha256: &str,
        cancelled: &AtomicBool,
    ) -> Result<&IndexedFile<'a>, LookupError> {
        check_cancelled(cancelled)?;
        let file = self
            .files
            .get(path)
            .ok_or(LookupError::UnknownGeneratedFile)?;
        if file.file.sha256 != expected_sha256 {
            return Err(LookupError::StaleArtifact);
        }
        Ok(file)
    }

    fn lookup(
        &self,
        indexed: &IndexedFile<'a>,
        offset: usize,
        cancelled: &AtomicBool,
    ) -> Result<SourceLookup<'a>, LookupError> {
        let file = indexed.file;
        if offset > file.text.len() || !file.text.is_char_boundary(offset) {
            return Err(LookupError::InvalidPosition);
        }
        let mut best = None;
        indexed.intervals.visit(offset, cancelled, |ordinal, span| {
            let key = (
                rank(file.mappings[ordinal].granularity)?,
                span.end - span.start,
            );
            if best.is_none_or(|previous| key < previous) {
                best = Some(key);
            }
            Ok(())
        })?;
        let Some(best) = best else {
            return Ok(SourceLookup::Unmapped);
        };
        let mut ordinals = Vec::new();
        // Count only final best matches. An earlier, less precise tie set must
        // not exhaust the candidate budget before a more precise match is found.
        indexed.intervals.visit(offset, cancelled, |ordinal, span| {
            if (rank(file.mappings[ordinal].granularity)?, span.end - span.start) == best {
                if ordinals.len() >= MAX_CANDIDATES {
                    return Err(LookupError::InputLimit);
                }
                ordinals.push(ordinal);
            }
            Ok(())
        })?;
        ordinals.sort_unstable();
        let mut candidates = Vec::with_capacity(ordinals.len());
        for ordinal in ordinals {
            check_cancelled(cancelled)?;
            let mapping = &file.mappings[ordinal];
            candidates.push(SourceLocation {
                generated: mapping.generated,
                granularity: mapping.granularity,
                revision: self.sources.revision(&mapping.source)?,
                source: &mapping.source,
            });
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
}
