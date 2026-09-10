use lsp_types::{Position, Range};
use wow_core::SourceSpan;

use super::EmmySyntaxErrorCode;

pub(super) fn convert(text: &str, range: Range) -> Result<SourceSpan, EmmySyntaxErrorCode> {
    let start = scalar_position_to_byte(text, range.start)?;
    let end = scalar_position_to_byte(text, range.end)?;
    if start > end || !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return Err(EmmySyntaxErrorCode::CoordinateConversionFailed);
    }
    SourceSpan::byte_range(
        u64::try_from(start).map_err(|_| EmmySyntaxErrorCode::CoordinateConversionFailed)?,
        u64::try_from(end).map_err(|_| EmmySyntaxErrorCode::CoordinateConversionFailed)?,
    )
    .map_err(|_| EmmySyntaxErrorCode::CoordinateConversionFailed)
}

/// The pinned upstream LineIndex counts Unicode scalar values in its LSP-shaped
/// character field. Preserve that observed contract, then publish UTF-8 bytes.
fn scalar_position_to_byte(text: &str, position: Position) -> Result<usize, EmmySyntaxErrorCode> {
    let requested_line = usize::try_from(position.line)
        .map_err(|_| EmmySyntaxErrorCode::CoordinateConversionFailed)?;
    let requested_column = usize::try_from(position.character)
        .map_err(|_| EmmySyntaxErrorCode::CoordinateConversionFailed)?;
    let mut line = 0usize;
    let mut line_start = 0usize;
    for (offset, byte) in text.bytes().enumerate() {
        if line == requested_line {
            break;
        }
        if byte == b'\n' {
            line += 1;
            line_start = offset + 1;
        }
    }
    if line != requested_line {
        return Err(EmmySyntaxErrorCode::CoordinateConversionFailed);
    }
    let line_end = text[line_start..]
        .find('\n')
        .map_or(text.len(), |offset| line_start + offset);
    let line_text = text
        .get(line_start..line_end)
        .ok_or(EmmySyntaxErrorCode::CoordinateConversionFailed)?;
    for (column, (relative, _)) in line_text.char_indices().enumerate() {
        if column == requested_column {
            return Ok(line_start + relative);
        }
    }
    if line_text.chars().count() == requested_column {
        Ok(line_end)
    } else {
        Err(EmmySyntaxErrorCode::CoordinateConversionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_positions_preserve_multibyte_and_crlf_bytes() {
        let text = "é🦀x\r\nnext";
        for (line, character, expected) in [
            (0, 0, 0),
            (0, 1, "é".len()),
            (0, 3, "é🦀x".len()),
            (0, 4, "é🦀x\r".len()),
            (1, 0, "é🦀x\r\n".len()),
        ] {
            assert_eq!(
                scalar_position_to_byte(text, Position { line, character }),
                Ok(expected)
            );
        }
    }
}
