//! Sparse line checkpoints over the same immutable bytes as the map index.
use super::super::{LookupError, PositionEncoding, TextPosition, check_cancelled, positions};
use std::sync::atomic::AtomicBool;

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
    fn checkpoints_preserve_all_coordinate_rules_across_multiple_blocks()
    -> Result<(), LookupError> {
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
    fn sparse_storage_keeps_trailing_empty_line_and_checks_cancellation()
    -> Result<(), LookupError> {
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
