//! Strict byte framing for the selected LSP stdio profile.
use std::io::{BufRead, ErrorKind, Write};

#[derive(Clone, Copy)]
pub(super) struct Limits {
    pub header_bytes: usize,
    pub body_bytes: usize,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FrameError {
    Io,
    InvalidHeader,
    TooLarge,
    Truncated,
}

pub(super) fn read_frame<R: BufRead>(
    reader: &mut R,
    frame: &mut Vec<u8>,
    limits: Limits,
) -> Result<bool, FrameError> {
    frame.clear();
    if limits.header_bytes == 0
        || limits.header_bytes > 65536
        || limits.body_bytes == 0
        || limits.body_bytes > 16 * 1024 * 1024
    {
        return Err(FrameError::TooLarge);
    }
    let mut header = Vec::new();
    header
        .try_reserve_exact(limits.header_bytes)
        .map_err(|_| FrameError::TooLarge)?;
    loop {
        let available = reader.fill_buf().map_err(|_| FrameError::Io)?;
        if available.is_empty() {
            return if header.is_empty() {
                Ok(false)
            } else {
                Err(FrameError::Truncated)
            };
        }
        let byte = available[0];
        reader.consume(1);
        if header.len() >= limits.header_bytes {
            return Err(FrameError::TooLarge);
        }
        if !byte.is_ascii()
            || (byte.is_ascii_control() && !matches!(byte, b'\r' | b'\n' | b'\t'))
            || (byte == b'\n' && header.last() != Some(&b'\r'))
            || (header.last() == Some(&b'\r') && byte != b'\n')
        {
            return Err(FrameError::InvalidHeader);
        }
        header.push(byte);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header =
        std::str::from_utf8(&header[..header.len() - 4]).map_err(|_| FrameError::InvalidHeader)?;
    let mut length = None;
    let mut content_type = false;
    for line in header.split("\r\n") {
        let (name, value) = line.split_once(':').ok_or(FrameError::InvalidHeader)?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("Content-Length") && length.is_none() {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(FrameError::InvalidHeader);
            }
            let selected: usize = value.parse().map_err(|_| FrameError::TooLarge)?;
            if selected > limits.body_bytes {
                return Err(FrameError::TooLarge);
            }
            length = Some(selected);
        } else if name.eq_ignore_ascii_case("Content-Type") && !content_type {
            validate_content_type(value)?;
            content_type = true;
        } else {
            return Err(FrameError::InvalidHeader);
        }
    }
    let length = length.ok_or(FrameError::InvalidHeader)?;
    frame
        .try_reserve_exact(length)
        .map_err(|_| FrameError::TooLarge)?;
    frame.resize(length, 0);
    reader.read_exact(frame).map_err(|error| {
        if error.kind() == ErrorKind::UnexpectedEof {
            FrameError::Truncated
        } else {
            FrameError::Io
        }
    })?;
    Ok(true)
}
fn validate_content_type(value: &str) -> Result<(), FrameError> {
    let mut parts = value.split(';');
    if !parts.next().is_some_and(|mime| {
        mime.trim()
            .eq_ignore_ascii_case("application/vscode-jsonrpc")
    }) {
        return Err(FrameError::InvalidHeader);
    }
    let mut charset_seen = false;
    for parameter in parts {
        let (name, value) = parameter
            .trim()
            .split_once('=')
            .ok_or(FrameError::InvalidHeader)?;
        if charset_seen || !name.trim().eq_ignore_ascii_case("charset") {
            return Err(FrameError::InvalidHeader);
        }
        let value = value.trim();
        let value = if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            &value[1..value.len() - 1]
        } else {
            value
        };
        if !value.eq_ignore_ascii_case("utf-8") && !value.eq_ignore_ascii_case("utf8") {
            return Err(FrameError::InvalidHeader);
        }
        charset_seen = true;
    }
    Ok(())
}
pub(super) fn write_frame<W: Write>(
    output: &mut W,
    payload: &[u8],
    max_bytes: usize,
) -> Result<(), FrameError> {
    if payload.len() > max_bytes {
        return Err(FrameError::TooLarge);
    }
    let header = format!("Content-Length: {}\r\n\r\n", payload.len());
    output
        .write_all(header.as_bytes())
        .and_then(|()| output.write_all(payload))
        .and_then(|()| output.flush())
        .map_err(|_| FrameError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiline_utf8_uses_byte_length_and_preserves_adjacent_frames() -> Result<(), FrameError> {
        let payload = "{\n\"text\":\"😀\"\n}".as_bytes();
        let mut output = Vec::new();
        write_frame(&mut output, payload, 256)?;
        write_frame(&mut output, b"{}", 256)?;
        let mut input = output.as_slice();
        let mut frame = Vec::new();
        let limits = Limits {
            header_bytes: 128,
            body_bytes: 256,
        };
        assert!(read_frame(&mut input, &mut frame, limits)?);
        assert_eq!(frame, payload);
        assert!(read_frame(&mut input, &mut frame, limits)?);
        assert_eq!(frame, b"{}");
        assert!(!read_frame(&mut input, &mut frame, limits)?);
        Ok(())
    }
    #[test]
    fn malformed_truncated_and_oversized_frames_refuse_without_partial_output() {
        let limits = Limits {
            header_bytes: 128,
            body_bytes: 4,
        };
        for (input, expected) in [
            (
                b"Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
                FrameError::InvalidHeader,
            ),
            (
                b"Content-Length: 9\r\n\r\n".as_slice(),
                FrameError::TooLarge,
            ),
            (
                b"Content-Length: 4\r\n\r\nabc".as_slice(),
                FrameError::Truncated,
            ),
            (
                b"Content-Length: 2\n\n{}".as_slice(),
                FrameError::InvalidHeader,
            ),
        ] {
            let mut reader = input;
            assert_eq!(
                read_frame(&mut reader, &mut Vec::new(), limits),
                Err(expected)
            );
        }
        let mut output = Vec::new();
        assert_eq!(
            write_frame(&mut output, b"12345", 4),
            Err(FrameError::TooLarge)
        );
        assert!(output.is_empty());
    }
}
