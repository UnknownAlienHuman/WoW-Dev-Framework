//! Sparse line checkpoints over the same immutable bytes as the map index.
use super::super::{
    LookupError, PositionEncoding, TextPosition, TextRange, check_cancelled, positions,
};
use std::sync::atomic::AtomicBool;
use wow_reference::native::Span;

const LINE_STRIDE: u32 = 64;

/// One u32 byte offset per 64 logical lines, plus the initial zero offset.
/// Sparse storage preserves the existing input limits without allocating one
/// entry per newline. The borrowed text cannot be replaced after preparation.
pub(super) struct Lines<'a> {
    text: &'a str,
    starts: Box<[u32]>,
}

impl<'a> Lines<'a> {
    pub(super) fn new(text: &'a str, cancelled: &AtomicBool) -> Result<Self, LookupError> {
        check_cancelled(cancelled)?;
        if text.len() > crate::ketho::MAX_OUTPUT_BYTES {
            return Err(LookupError::InputLimit);
        }
        let mut starts = vec![0];
        let mut line = 0u32;
        let mut cursor = 0usize;
        let mut next_check = 0usize;
        let bytes = text.as_bytes();
        while cursor < bytes.len() {
            if cursor >= next_check {
                check_cancelled(cancelled)?;
                next_check = cursor.saturating_add(4096);
            }
            let width = positions::line_break_width(bytes, cursor);
            cursor += width.max(1);
            if width != 0 {
                line += 1;
                if line.is_multiple_of(LINE_STRIDE) {
                    starts.push(u32::try_from(cursor).map_err(|_| LookupError::InputLimit)?);
                }
            }
        }
        check_cancelled(cancelled)?;
        Ok(Self {
            text,
            starts: starts.into_boxed_slice(),
        })
    }

    /// Inverse coordinate conversion for a stored byte range. The unrepresentable
    /// boundary between CR and LF rejects instead of moving to another byte.
    pub(super) fn text_range(
        &self,
        range: Span,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<TextRange, LookupError> {
        check_cancelled(cancelled)?;
        if self.text.get(range.start..range.end).is_none() {
            return Err(LookupError::InvalidPosition);
        }
        let start = self.text_position(range.start, encoding, cancelled)?;
        let end = if range.start == range.end {
            start
        } else {
            self.text_position(range.end, encoding, cancelled)?
        };
        check_cancelled(cancelled)?;
        Ok(TextRange { start, end })
    }

    fn text_position(
        &self,
        offset: usize,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<TextPosition, LookupError> {
        check_cancelled(cancelled)?;
        let offset_u32 = u32::try_from(offset).map_err(|_| LookupError::InvalidPosition)?;
        let checkpoint = self
            .starts
            .partition_point(|start| *start <= offset_u32)
            .checked_sub(1)
            .ok_or(LookupError::InvalidMapping)?;
        let mut cursor = usize::try_from(self.starts[checkpoint])
            .map_err(|_| LookupError::InvalidMapping)?;
        let mut line = u32::try_from(checkpoint)
            .ok()
            .and_then(|index| index.checked_mul(LINE_STRIDE))
            .ok_or(LookupError::InvalidPosition)?;
        let mut line_start = cursor;
        let mut next_check = cursor;
        let bytes = self.text.as_bytes();
        while cursor < offset {
            if cursor >= next_check {
                check_cancelled(cancelled)?;
                next_check = cursor.saturating_add(4096);
            }
            let width = positions::line_break_width(bytes, cursor);
            cursor += width.max(1);
            if cursor > offset {
                return Err(LookupError::InvalidPosition);
            }
            if width != 0 {
                line = line.checked_add(1).ok_or(LookupError::InvalidPosition)?;
                line_start = cursor;
            }
        }
        let prefix = self
            .text
            .get(line_start..offset)
            .ok_or(LookupError::InvalidPosition)?;
        let mut character = 0u32;
        for ch in prefix.chars() {
            check_cancelled(cancelled)?;
            let width = match encoding {
                PositionEncoding::Utf8 => ch.len_utf8(),
                PositionEncoding::Utf16 => ch.len_utf16(),
                PositionEncoding::Utf32 => 1,
            };
            character = character
                .checked_add(u32::try_from(width).map_err(|_| LookupError::InvalidPosition)?)
                .ok_or(LookupError::InvalidPosition)?;
        }
        check_cancelled(cancelled)?;
        Ok(TextPosition { line, character })
    }

    pub(super) fn byte_offset(
        &self,
        position: TextPosition,
        encoding: PositionEncoding,
        cancelled: &AtomicBool,
    ) -> Result<usize, LookupError> {
        check_cancelled(cancelled)?;
        let checkpoint = usize::try_from(position.line / LINE_STRIDE)
            .map_err(|_| LookupError::InvalidPosition)?;
        let start = *self
            .starts
            .get(checkpoint)
            .ok_or(LookupError::InvalidPosition)?;
        let start = usize::try_from(start).map_err(|_| LookupError::InvalidPosition)?;
        let tail = self.text.get(start..).ok_or(LookupError::InvalidMapping)?;
        // Reuse strict Unicode/column handling and scan at most 63 preceding
        // line delimiters, rather than scanning from the start of the file.
        let relative = positions::byte_offset(
            tail,
            TextPosition {
                line: position.line % LINE_STRIDE,
                ..position
            },
            encoding,
            cancelled,
        )?;
        start
            .checked_add(relative)
            .ok_or(LookupError::InvalidPosition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkpoints_preserve_all_coordinate_rules_across_multiple_blocks() -> Result<(), LookupError>
    {
        let cancelled = AtomicBool::new(false);
        let mut text = String::new();
        for line in 0..200 {
            text.push_str("aé🦀e\u{301}");
            text.push_str(["\n", "\r\n", "\r", "\n\r"][line % 4]);
        }
        for text in ["", "x\n", "\r\n", text.as_str()] {
            let indexed = Lines::new(text, &cancelled)?;
            for encoding in [
                PositionEncoding::Utf8,
                PositionEncoding::Utf16,
                PositionEncoding::Utf32,
            ] {
                for line in (0..=260).chain([u32::MAX]) {
                    for character in (0..=12).chain([u32::MAX]) {
                        let position = TextPosition { line, character };
                        assert_eq!(
                            indexed.byte_offset(position, encoding, &cancelled),
                            positions::byte_offset(text, position, encoding, &cancelled)
                        );
                    }
                }
            }
        }
        Ok(())
    }

    #[test]
    fn inverse_coordinates_round_trip_all_boundaries_across_checkpoints() -> Result<(), LookupError>
    {
        let cancelled = AtomicBool::new(false);
        let mut text = String::new();
        for line in 0..140 {
            text.push_str("aé🦀e\u{301}");
            text.push_str(["\n", "\r\n", "\r", "\n\r"][line % 4]);
        }
        for text in ["", "x\n", "\r\n", text.as_str()] {
            let indexed = Lines::new(text, &cancelled)?;
            for encoding in [
                PositionEncoding::Utf8,
                PositionEncoding::Utf16,
                PositionEncoding::Utf32,
            ] {
                for offset in 0..=text.len() {
                    let span = Span {
                        start: offset,
                        end: offset,
                    };
                    let result = indexed.text_range(span, encoding, &cancelled);
                    let split_crlf = offset > 0
                        && text.as_bytes().get(offset - 1) == Some(&b'\r')
                        && text.as_bytes().get(offset) == Some(&b'\n');
                    if !text.is_char_boundary(offset) || split_crlf {
                        assert_eq!(result, Err(LookupError::InvalidPosition));
                        continue;
                    }
                    let converted = result?;
                    assert_eq!(converted.start, converted.end);
                    assert_eq!(
                        positions::byte_offset(text, converted.start, encoding, &cancelled)?,
                        offset
                    );
                    assert_eq!(
                        indexed.byte_offset(converted.end, encoding, &cancelled)?,
                        offset
                    );
                }
            }
        }
        Ok(())
    }

    #[test]
    fn inverse_ranges_preserve_newline_edges_and_reject_unrepresentable_ends()
    -> Result<(), LookupError> {
        let cancelled = AtomicBool::new(false);
        let text = "aé🦀\r\nz\r";
        let indexed = Lines::new(text, &cancelled)?;
        for (encoding, eol) in [
            (PositionEncoding::Utf8, 7),
            (PositionEncoding::Utf16, 4),
            (PositionEncoding::Utf32, 3),
        ] {
            let converted = indexed.text_range(Span { start: 7, end: 9 }, encoding, &cancelled)?;
            assert_eq!(
                converted.start,
                TextPosition {
                    line: 0,
                    character: eol,
                }
            );
            assert_eq!(
                converted.end,
                TextPosition {
                    line: 1,
                    character: 0,
                }
            );
            let all = indexed.text_range(
                Span {
                    start: 0,
                    end: text.len(),
                },
                encoding,
                &cancelled,
            )?;
            assert_eq!(
                all.end,
                TextPosition {
                    line: 2,
                    character: 0,
                }
            );
            for invalid in [
                Span { start: 8, end: 8 },
                Span { start: 0, end: 8 },
                Span { start: 8, end: 9 },
                Span { start: 2, end: 3 },
                Span { start: 9, end: 7 },
                Span {
                    start: 0,
                    end: usize::MAX,
                },
            ] {
                assert_eq!(
                    indexed.text_range(invalid, encoding, &cancelled),
                    Err(LookupError::InvalidPosition)
                );
            }
            assert_eq!(
                indexed.text_range(Span { start: 0, end: 0 }, encoding, &AtomicBool::new(true)),
                Err(LookupError::Cancelled)
            );
        }
        Ok(())
    }

    #[test]
    fn sparse_storage_keeps_trailing_empty_line_and_checks_cancellation() -> Result<(), LookupError>
    {
        let cancelled = AtomicBool::new(false);
        let text = "\r\n".repeat(4096);
        let indexed = Lines::new(&text, &cancelled)?;
        assert_eq!(indexed.starts.len(), 65);
        let position = TextPosition {
            line: 4096,
            character: 0,
        };
        assert_eq!(
            indexed.byte_offset(position, PositionEncoding::Utf16, &cancelled)?,
            text.len()
        );
        assert!(matches!(
            Lines::new(&text, &AtomicBool::new(true)),
            Err(LookupError::Cancelled)
        ));
        assert_eq!(
            indexed.byte_offset(position, PositionEncoding::Utf16, &AtomicBool::new(true)),
            Err(LookupError::Cancelled)
        );
        Ok(())
    }
}
