//! Exact positions over the original immutable UTF8 bytes. No normalization.
use super::{
    OverlayBudget, OverlayEncoding, OverlayErrorCode, OverlayPosition, OverlayResult, error,
};
use std::sync::Arc;

struct Line {
    start: usize,
    end: usize,
}
pub(super) struct OverlayLineIndex {
    content: Arc<str>,
    lines: Vec<Line>,
}
impl OverlayLineIndex {
    pub(super) fn build(
        content: Arc<str>,
        max_lines: usize,
        budget: &mut OverlayBudget<'_>,
    ) -> OverlayResult<Self> {
        let mut lines = Vec::new();
        let mut start = 0;
        let mut cursor = 0;
        let bytes = content.as_bytes();
        let mut push = |start, end, budget: &mut OverlayBudget<'_>| -> OverlayResult<()> {
            budget.work(1)?;
            if lines.len() >= max_lines {
                return Err(error(OverlayErrorCode::Limit));
            }
            if lines.len() == lines.capacity() {
                let additional = lines.capacity().max(1).min(max_lines - lines.len());
                budget.work(
                    lines
                        .len()
                        .checked_add(additional)
                        .and_then(|n| n.checked_mul(std::mem::size_of::<Line>()))
                        .ok_or_else(|| error(OverlayErrorCode::Limit))?,
                )?;
                lines
                    .try_reserve_exact(additional)
                    .map_err(|_| error(OverlayErrorCode::Limit))?;
            }
            lines.push(Line { start, end });
            Ok(())
        };
        while cursor < bytes.len() {
            budget.work(1)?;
            match bytes[cursor] {
                b'\r' => {
                    push(start, cursor, budget)?;
                    cursor += 1;
                    if bytes.get(cursor) == Some(&b'\n') {
                        budget.work(1)?;
                        cursor += 1;
                    }
                    start = cursor;
                }
                b'\n' => {
                    push(start, cursor, budget)?;
                    cursor += 1;
                    start = cursor;
                }
                _ => cursor += 1,
            }
        }
        push(start, cursor, budget)?;
        Ok(Self { content, lines })
    }
    pub(super) fn line_count(&self) -> usize {
        self.lines.len()
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.lines
            .capacity()
            .saturating_mul(std::mem::size_of::<Line>())
    }
    pub(super) fn byte_offset(
        &self,
        position: OverlayPosition,
        encoding: OverlayEncoding,
        budget: &mut OverlayBudget<'_>,
    ) -> OverlayResult<usize> {
        budget.work(1)?;
        let line = self
            .lines
            .get(position.line as usize)
            .ok_or_else(|| error(OverlayErrorCode::InvalidPosition))?;
        let column = position.character as usize;
        match encoding {
            OverlayEncoding::Utf8 => {
                let offset = line
                    .start
                    .checked_add(column)
                    .filter(|offset| *offset <= line.end)
                    .ok_or_else(|| error(OverlayErrorCode::InvalidPosition))?;
                if !self.content.is_char_boundary(offset) {
                    return Err(error(OverlayErrorCode::InvalidEncodingBoundary));
                }
                Ok(offset)
            }
            OverlayEncoding::Utf16 => {
                let mut units = 0;
                for (offset, ch) in self.content[line.start..line.end].char_indices() {
                    budget.work(ch.len_utf8())?;
                    if units == column {
                        return Ok(line.start + offset);
                    }
                    let next = units + ch.len_utf16();
                    if column < next {
                        return Err(error(OverlayErrorCode::InvalidEncodingBoundary));
                    }
                    units = next;
                }
                if units == column {
                    Ok(line.end)
                } else {
                    Err(error(OverlayErrorCode::InvalidPosition))
                }
            }
        }
    }
    pub(super) fn position(
        &self,
        offset: usize,
        encoding: OverlayEncoding,
        budget: &mut OverlayBudget<'_>,
    ) -> OverlayResult<OverlayPosition> {
        budget.work(usize::BITS as usize)?;
        if offset > self.content.len() {
            return Err(error(OverlayErrorCode::InvalidPosition));
        }
        if !self.content.is_char_boundary(offset) {
            return Err(error(OverlayErrorCode::InvalidEncodingBoundary));
        }
        let index = self
            .lines
            .partition_point(|line| line.start <= offset)
            .checked_sub(1)
            .ok_or_else(|| error(OverlayErrorCode::InvalidPosition))?;
        let line = &self.lines[index];
        if offset > line.end {
            return Err(error(OverlayErrorCode::InvalidPosition));
        }
        let character = match encoding {
            OverlayEncoding::Utf8 => offset - line.start,
            OverlayEncoding::Utf16 => {
                let mut units = 0_usize;
                for ch in self.content[line.start..offset].chars() {
                    budget.work(ch.len_utf8())?;
                    units += ch.len_utf16();
                }
                units
            }
        };
        Ok(OverlayPosition {
            line: u32::try_from(index).map_err(|_| error(OverlayErrorCode::Limit))?,
            character: u32::try_from(character).map_err(|_| error(OverlayErrorCode::Limit))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    #[test]
    fn unicode_and_crlf_roundtrip_refuses_split_codepoints_and_line_endings() -> OverlayResult<()> {
        let stop = AtomicBool::new(false);
        let mut work = OverlayBudget::new(65536, &stop);
        let index = OverlayLineIndex::build(Arc::from("a😀b\r\n中\n"), 16, &mut work)?;
        assert_eq!(index.line_count(), 3);
        for (position, offset) in [
            (
                OverlayPosition {
                    line: 0,
                    character: 1,
                },
                1,
            ),
            (
                OverlayPosition {
                    line: 0,
                    character: 3,
                },
                5,
            ),
            (
                OverlayPosition {
                    line: 1,
                    character: 1,
                },
                11,
            ),
            (
                OverlayPosition {
                    line: 2,
                    character: 0,
                },
                12,
            ),
        ] {
            assert_eq!(
                index.byte_offset(position, OverlayEncoding::Utf16, &mut work)?,
                offset
            );
            assert_eq!(
                index.position(offset, OverlayEncoding::Utf16, &mut work)?,
                position
            );
        }
        assert!(
            index
                .byte_offset(
                    OverlayPosition {
                        line: 0,
                        character: 2
                    },
                    OverlayEncoding::Utf16,
                    &mut work
                )
                .is_err()
        );
        assert!(
            index
                .byte_offset(
                    OverlayPosition {
                        line: 0,
                        character: 2
                    },
                    OverlayEncoding::Utf8,
                    &mut work
                )
                .is_err()
        );
        assert!(
            index
                .position(7, OverlayEncoding::Utf16, &mut work)
                .is_err()
        );
        Ok(())
    }
}
