//! Frozen scalar-value Levenshtein, bounded by lengths, band and charged cells.
use crate::{SearchError, SearchResult};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum DistanceResult {
    Within(u32),
    Outside,
    Exhausted,
}

pub(super) fn normalize(value: &str, ascii_fold: bool) -> Vec<char> {
    value
        .chars()
        .map(|ch| {
            if ascii_fold {
                ch.to_ascii_lowercase()
            } else {
                ch
            }
        })
        .collect()
}

pub(super) fn bounded_distance(
    query: &[char],
    candidate: &[char],
    max_distance: u32,
    remaining_cells: &mut u64,
    stop: &AtomicBool,
) -> SearchResult<DistanceResult> {
    checkpoint(stop)?;
    if query.len() > 128 || candidate.len() > 132 || max_distance > 4 {
        return Err(SearchError::InvalidProfile);
    }
    let band = max_distance as usize;
    if query.len().abs_diff(candidate.len()) > band {
        return Ok(DistanceResult::Outside);
    }
    if query.is_empty() {
        return Ok(DistanceResult::Within(candidate.len() as u32));
    }
    if candidate.is_empty() {
        return Ok(DistanceResult::Within(query.len() as u32));
    }
    let sentinel = max_distance + 1;
    let mut previous = (0..=candidate.len())
        .map(|n| (n as u32).min(sentinel))
        .collect::<Vec<_>>();
    let mut current = vec![sentinel; candidate.len() + 1];
    for (row, ch) in query.iter().enumerate() {
        checkpoint(stop)?;
        let row = row + 1;
        current.fill(sentinel);
        current[0] = (row as u32).min(sentinel);
        let start = row.saturating_sub(band).max(1);
        let end = (row + band).min(candidate.len());
        let mut best = current[0];
        for column in start..=end {
            checkpoint(stop)?;
            if *remaining_cells == 0 {
                return Ok(DistanceResult::Exhausted);
            }
            *remaining_cells -= 1;
            let substitution = previous[column - 1] + u32::from(*ch != candidate[column - 1]);
            current[column] = (previous[column] + 1)
                .min(current[column - 1] + 1)
                .min(substitution)
                .min(sentinel);
            best = best.min(current[column]);
        }
        if best > max_distance {
            return Ok(DistanceResult::Outside);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let distance = previous[candidate.len()];
    Ok(if distance <= max_distance {
        DistanceResult::Within(distance)
    } else {
        DistanceResult::Outside
    })
}
fn checkpoint(stop: &AtomicBool) -> SearchResult<()> {
    if stop.load(Ordering::Acquire) {
        Err(SearchError::Cancelled)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn compare(a: &str, b: &str, k: u32, cells: &mut u64) -> SearchResult<DistanceResult> {
        bounded_distance(
            &normalize(a, false),
            &normalize(b, false),
            k,
            cells,
            &AtomicBool::new(false),
        )
    }
    #[test]
    fn scalar_edits_and_frozen_ascii_normalization() -> SearchResult<()> {
        let mut cells = 10000;
        assert_eq!(
            compare("Read", "Reed", 1, &mut cells)?,
            DistanceResult::Within(1)
        );
        assert_eq!(
            compare("Read", "Reads", 1, &mut cells)?,
            DistanceResult::Within(1)
        );
        assert_eq!(
            compare("Read", "Re", 2, &mut cells)?,
            DistanceResult::Within(2)
        );
        assert_eq!(
            compare("Read", "road", 1, &mut cells)?,
            DistanceResult::Outside
        );
        assert_eq!(normalize("RÉAD", true), vec!['r', 'É', 'a', 'd']);
        assert_ne!(normalize("é", true), normalize("e\u{301}", true));
        assert_eq!(
            compare("é", "e\u{301}", 2, &mut cells)?,
            DistanceResult::Within(2)
        );
        assert_eq!(
            bounded_distance(
                &normalize("READ", true),
                &normalize("read", true),
                0,
                &mut cells,
                &AtomicBool::new(false)
            )?,
            DistanceResult::Within(0)
        );
        Ok(())
    }
    #[test]
    fn charged_band_exhaustion_lower_bound_and_cancellation() -> SearchResult<()> {
        let mut cells = 4;
        assert_eq!(
            compare("Read", "Read", 0, &mut cells)?,
            DistanceResult::Within(0)
        );
        assert_eq!(cells, 0);
        let mut cells = 3;
        assert_eq!(
            compare("Read", "Read", 0, &mut cells)?,
            DistanceResult::Exhausted
        );
        assert_eq!(cells, 0);
        let mut cells = 0;
        assert_eq!(
            compare("Read", "MuchLonger", 1, &mut cells)?,
            DistanceResult::Outside
        );
        assert_eq!(cells, 0);
        assert!(matches!(
            bounded_distance(&['a'], &['a'], 0, &mut cells, &AtomicBool::new(true)),
            Err(SearchError::Cancelled)
        ));
        assert!(matches!(
            compare(&"a".repeat(129), "a", 1, &mut cells),
            Err(SearchError::InvalidProfile)
        ));
        Ok(())
    }
}
