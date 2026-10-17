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

fn validate_limits(limits: Limits) -> Result<(), FrameError> {
    if limits.header_bytes == 0
        || limits.header_bytes > 65536
        || limits.body_bytes == 0
        || limits.body_bytes > 16 * 1024 * 1024
    {
        return Err(FrameError::TooLarge);
    }
    Ok(())
}
fn header_byte(previous: Option<u8>, byte: u8) -> Result<(), FrameError> {
    if !byte.is_ascii()
        || (byte.is_ascii_control() && !matches!(byte, b'\r' | b'\n' | b'\t'))
        || (byte == b'\n' && previous != Some(b'\r'))
        || (previous == Some(b'\r') && byte != b'\n')
    {
        return Err(FrameError::InvalidHeader);
    }
    Ok(())
}

/// One partial frame plus at most one bounded input chunk. Callers drain all
/// available frames before pushing another chunk; there is no hidden frame queue.
pub(super) struct FrameDecoder {
    limits: Limits,
    buffer: Vec<u8>,
    scanned: usize,
    body: Option<(usize, usize)>,
}
impl FrameDecoder {
    pub(super) fn new(limits: Limits) -> Result<Self, FrameError> {
        validate_limits(limits)?;
        Ok(Self {
            limits,
            buffer: Vec::new(),
            scanned: 0,
            body: None,
        })
    }
    pub(super) fn push(&mut self, bytes: &[u8]) -> Result<(), FrameError> {
        let maximum = self.limits.header_bytes + self.limits.body_bytes + 8192;
        if bytes.len() > 8192
            || self
                .buffer
                .len()
                .checked_add(bytes.len())
                .is_none_or(|length| length > maximum)
        {
            return Err(FrameError::TooLarge);
        }
        let needed = self.buffer.len() + bytes.len();
        if needed > self.buffer.capacity() {
            let target = self
                .buffer
                .capacity()
                .max(8192)
                .saturating_mul(2)
                .min(maximum)
                .max(needed);
            self.buffer
                .try_reserve_exact(target - self.buffer.len())
                .map_err(|_| FrameError::TooLarge)?;
        }
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }
    pub(super) fn next_frame(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        if self.body.is_none() {
            while self.scanned < self.buffer.len() {
                if self.scanned >= self.limits.header_bytes {
                    return Err(FrameError::TooLarge);
                }
                header_byte(
                    self.scanned.checked_sub(1).map(|n| self.buffer[n]),
                    self.buffer[self.scanned],
                )?;
                self.scanned += 1;
                if self.buffer[..self.scanned].ends_with(b"\r\n\r\n") {
                    let length = parse_header(&self.buffer[..self.scanned], self.limits)?;
                    self.body = Some((self.scanned, length));
                    break;
                }
            }
        }
        let Some((header, length)) = self.body else {
            return Ok(None);
        };
        let end = header.checked_add(length).ok_or(FrameError::TooLarge)?;
        if self.buffer.len() < end {
            return Ok(None);
        }
        let mut frame = Vec::new();
        frame
            .try_reserve_exact(length)
            .map_err(|_| FrameError::TooLarge)?;
        frame.extend_from_slice(&self.buffer[header..end]);
        self.buffer.drain(..end);
        self.scanned = 0;
        self.body = None;
        Ok(Some(frame))
    }
    /// Call after draining next_frame at EOF. A retained prefix is truncated.
    pub(super) fn finish(&self) -> Result<(), FrameError> {
        if self.buffer.is_empty() {
            Ok(())
        } else {
            Err(FrameError::Truncated)
        }
    }
}

pub(super) fn read_frame<R: BufRead>(
    reader: &mut R,
    frame: &mut Vec<u8>,
    limits: Limits,
) -> Result<bool, FrameError> {
    frame.clear();
    validate_limits(limits)?;
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
        header_byte(header.last().copied(), byte)?;
        header.push(byte);
        if header.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let length = parse_header(&header, limits)?;
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
fn parse_header(header: &[u8], limits: Limits) -> Result<usize, FrameError> {
    let header = header
        .strip_suffix(b"\r\n\r\n")
        .ok_or(FrameError::InvalidHeader)?;
    let header = std::str::from_utf8(header).map_err(|_| FrameError::InvalidHeader)?;
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
    length.ok_or(FrameError::InvalidHeader)
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
    fn decoder_keeps_split_utf8_and_adjacent_frames_exact() -> Result<(), FrameError> {
        let mut wire = Vec::new();
        write_frame(&mut wire, "{\"text\":\"😀\"}".as_bytes(), 256)?;
        write_frame(&mut wire, b"{}", 256)?;
        let mut decoder = FrameDecoder::new(Limits {
            header_bytes: 128,
            body_bytes: 256,
        })?;
        let mut frames = Vec::new();
        for chunk in wire.chunks(1) {
            decoder.push(chunk)?;
            while let Some(frame) = decoder.next_frame()? {
                frames.push(frame);
            }
        }
        decoder.finish()?;
        assert_eq!(
            frames,
            vec!["{\"text\":\"😀\"}".as_bytes().to_vec(), b"{}".to_vec()]
        );
        // The same coalesced bytes pass through the same header parser.
        let mut decoder = FrameDecoder::new(Limits {
            header_bytes: 128,
            body_bytes: 256,
        })?;
        decoder.push(&wire)?;
        assert_eq!(decoder.next_frame()?, Some(frames[0].clone()));
        assert_eq!(decoder.next_frame()?, Some(frames[1].clone()));
        assert_eq!(decoder.next_frame()?, None);
        decoder.finish()
    }
    #[test]
    fn decoder_rejects_partial_oversized_and_duplicate_headers() -> Result<(), FrameError> {
        let limits = Limits {
            header_bytes: 128,
            body_bytes: 4,
        };
        for partial in [
            b"Content-Length: ".as_slice(),
            b"Content-Length: 4\r\n\r\nabc".as_slice(),
        ] {
            let mut decoder = FrameDecoder::new(limits)?;
            decoder.push(partial)?;
            assert_eq!(decoder.next_frame()?, None);
            assert_eq!(decoder.finish(), Err(FrameError::Truncated));
        }
        for (bytes, expected) in [
            (
                b"Content-Length: 9\r\n\r\n".as_slice(),
                FrameError::TooLarge,
            ),
            (
                b"Content-Length: 2\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
                FrameError::InvalidHeader,
            ),
            (
                b"Content-Length: 2\n\n{}".as_slice(),
                FrameError::InvalidHeader,
            ),
        ] {
            let mut decoder = FrameDecoder::new(limits)?;
            decoder.push(bytes)?;
            assert_eq!(decoder.next_frame(), Err(expected));
        }
        let mut decoder = FrameDecoder::new(limits)?;
        assert_eq!(decoder.push(&[0; 8193]), Err(FrameError::TooLarge));
        Ok(())
    }
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
