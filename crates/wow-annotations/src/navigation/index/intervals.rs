//! Balanced implicit interval tree over immutable, prevalidated generated maps.
use super::{LookupError, check_cancelled};
use std::sync::atomic::AtomicBool;
use wow_reference::native::Span;

struct Entry {
    span: Span,
    ordinal: usize,
    max_end: usize,
}

pub(super) struct Intervals {
    entries: Vec<Entry>,
}

impl Intervals {
    pub(super) fn new(
        spans: impl Iterator<Item = Span>,
        cancelled: &AtomicBool,
    ) -> Result<Self, LookupError> {
        let mut entries = Vec::new();
        for (ordinal, span) in spans.enumerate() {
            check_cancelled(cancelled)?;
            entries.push(Entry {
                span,
                ordinal,
                max_end: span.end,
            });
        }
        entries.sort_unstable_by_key(|entry| (entry.span.start, entry.span.end, entry.ordinal));
        check_cancelled(cancelled)?;
        Self::augment(&mut entries, cancelled)?;
        Ok(Self { entries })
    }

    // Each recursive slice halves; depth is logarithmic in the bounded map count,
    // independent of source nesting. There is no pointer tree or unbounded recursion.
    fn augment(entries: &mut [Entry], cancelled: &AtomicBool) -> Result<usize, LookupError> {
        check_cancelled(cancelled)?;
        if entries.is_empty() {
            return Ok(0);
        }
        let middle = entries.len() / 2;
        let left = Self::augment(&mut entries[..middle], cancelled)?;
        let right = Self::augment(&mut entries[middle + 1..], cancelled)?;
        let end = entries[middle].span.end.max(left).max(right);
        entries[middle].max_end = end;
        Ok(end)
    }

    pub(super) fn visit(
        &self,
        offset: usize,
        cancelled: &AtomicBool,
        mut visitor: impl FnMut(usize, Span) -> Result<(), LookupError>,
    ) -> Result<(), LookupError> {
        Self::visit_slice(&self.entries, offset, cancelled, &mut visitor)
    }

    fn visit_slice(
        entries: &[Entry],
        offset: usize,
        cancelled: &AtomicBool,
        visitor: &mut impl FnMut(usize, Span) -> Result<(), LookupError>,
    ) -> Result<(), LookupError> {
        check_cancelled(cancelled)?;
        if entries.is_empty() || entries[0].span.start > offset {
            return Ok(());
        }
        let middle = entries.len() / 2;
        let entry = &entries[middle];
        if entry.max_end <= offset {
            return Ok(());
        }
        Self::visit_slice(&entries[..middle], offset, cancelled, visitor)?;
        if entry.span.start <= offset && offset < entry.span.end {
            visitor(entry.ordinal, entry.span)?;
        }
        if entry.span.start <= offset {
            Self::visit_slice(&entries[middle + 1..], offset, cancelled, visitor)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_intervals_match_a_linear_scan_for_nested_and_crossing_ranges() -> Result<(), LookupError> {
        let spans = (0..257)
            .map(|ordinal| {
                let start = (ordinal * 71) % 257;
                Span { start, end: start + 1 + (ordinal * 43) % 257 }
            })
            .collect::<Vec<_>>();
        let cancelled = AtomicBool::new(false);
        let index = Intervals::new(spans.iter().copied(), &cancelled)?;
        for offset in 0..=514 {
            let mut actual = Vec::new();
            index.visit(offset, &cancelled, |ordinal, _| {
                actual.push(ordinal);
                Ok(())
            })?;
            actual.sort_unstable();
            let expected = spans.iter().enumerate()
                .filter(|(_, span)| span.start <= offset && offset < span.end)
                .map(|(ordinal, _)| ordinal)
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "offset {offset}");
        }
        Ok(())
    }
}
