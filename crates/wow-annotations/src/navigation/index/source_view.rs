//! Source cursor/range queries bound to independently verified immutable text.
use super::{
    GeneratedLocation, GeneratedLookup, LookupError, MAX_CANDIDATES, MAX_INDEX_MAPPINGS,
    NavigationIndex, PositionEncoding, TextPosition, TextRange, check_cancelled, intervals, lines,
    ranges, rank,
};
use crate::navigation::SourceFile;
use std::sync::atomic::AtomicBool;
use wow_reference::native::Span;

/// A source view bound to one prepared generation and the exact viewed bytes.
///
/// Borrowing both prevents either text or map identities changing underneath a
/// query. Build a new view after an edit. Source spans select stored descriptors;
/// generated offsets are never interpolated from source columns or token names.
pub struct SourceNavigation<'view, 'generation> {
    text: &'view str,
    lines: lines::Lines<'view>,
    locations: Vec<&'view GeneratedLocation<'generation>>,
    intervals: intervals::Intervals,
}

impl<'generation> NavigationIndex<'generation> {
    /// Bind a source buffer by scope, revision, path, digest and exact contents.
    ///
    /// Only the requested file gets a source-side interval/line index. A valid
    /// source without emitted maps can be bound, but its queries are Unmapped.
    /// No source is fetched, parsed or executed, and nothing is cached globally.
    pub fn bind_source<'view>(
        &'view self,
        file: SourceFile<'_>,
        text: &'view str,
        cancelled: &AtomicBool,
    ) -> Result<SourceNavigation<'view, 'generation>, LookupError> {
        self.sources.validate_text(file, text, cancelled)?;
        let mut locations = Vec::new();
        if let Some(descriptors) = self.reverse.descriptors(file.scope, file.path)? {
            for occurrences in descriptors.values() {
                for location in occurrences {
                    check_cancelled(cancelled)?;
                    let span = location.source.span;
                    // The generation records source byte lengths. Now that exact
                    // text is available, also reject boundaries inside UTF-8.
                    if text.get(span.start..span.end).is_none() {
                        return Err(LookupError::InvalidMapping);
                    }
                    if locations.len() >= MAX_INDEX_MAPPINGS {
                        return Err(LookupError::InputLimit);
                    }
                    locations.push(location);
                }
            }
        }
        let intervals = intervals::Intervals::new(
            locations.iter().map(|location| location.source.span),
            cancelled,
        )?;
        let lines = lines::Lines::new(text, cancelled)?;
        check_cancelled(cancelled)?;
        Ok(SourceNavigation {
            text,
            lines,
            locations,
            intervals,
        })
    }
}

impl<'generation> SourceNavigation<'_, 'generation> {
    /// Find every generated occurrence of the most precise source descriptor
    /// containing this UTF-8 cursor. Exclusive ends and EOF are not clamped.
    pub fn generated_at(
        &self,
        byte_offset: usize,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        self.generated_for_range(
            Span {
                start: byte_offset,
                end: byte_offset,
            },
            cancelled,
        )
    }

    /// Resolve an explicitly encoded source position using sparse line checkpoints.
    pub fn generated_at_position(
        &self,
        position: TextPosition,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        let offset = self.lines.byte_offset(position, encoding, cancelled)?;
        self.generated_at(offset, cancelled)
    }

    /// Require one source map to contain the whole selection. Rank by member,
    /// declaration, whole-file, then smallest source span. All tied occurrences
    /// survive in generated path/range order, including shared local descriptors.
    /// Unmapped is not evidence that all selected bytes or an API are absent.
    pub fn generated_for_range(
        &self,
        selected: Span,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        check_cancelled(cancelled)?;
        if self.text.get(selected.start..selected.end).is_none() {
            return Err(LookupError::InvalidPosition);
        }
        let mut best = None;
        self.intervals
            .visit(selected.start, cancelled, |ordinal, span| {
                if ranges::covers(span, selected) {
                    let key = (
                        rank(self.locations[ordinal].granularity)?,
                        span.end - span.start,
                    );
                    if best.is_none_or(|previous| key < previous) {
                        best = Some(key);
                    }
                }
                Ok(())
            })?;
        let Some(best) = best else {
            check_cancelled(cancelled)?;
            return Ok(GeneratedLookup::Unmapped);
        };
        let mut candidates = Vec::new();
        // Count final best matches, not earlier lower-precision candidates.
        self.intervals
            .visit(selected.start, cancelled, |ordinal, span| {
                let location = self.locations[ordinal];
                if ranges::covers(span, selected)
                    && (rank(location.granularity)?, span.end - span.start) == best
                {
                    if candidates.len() >= MAX_CANDIDATES {
                        return Err(LookupError::InputLimit);
                    }
                    candidates.push(location.clone());
                }
                Ok(())
            })?;
        candidates.sort_by_key(|location| {
            (
                location.path,
                location.generated.start,
                location.generated.end,
                location.granularity,
            )
        });
        check_cancelled(cancelled)?;
        Ok(GeneratedLookup::Mapped { candidates })
    }

    /// Convert both source endpoints against this same bound buffer. Invalid
    /// UTF-8/UTF-16 positions and reversed ranges reject without clamping.
    pub fn generated_for_text_range(
        &self,
        range: TextRange,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<GeneratedLookup<'generation>, LookupError> {
        let selected = ranges::byte_span_with(range, cancelled, |position| {
            self.lines.byte_offset(position, encoding, cancelled)
        })?;
        self.generated_for_range(selected, cancelled)
    }
}
