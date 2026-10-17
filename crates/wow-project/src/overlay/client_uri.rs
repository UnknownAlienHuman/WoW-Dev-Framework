//! Bounded local-file URI projection. No filesystem or URL acquisition.
use super::{OverlayErrorCode, OverlayResult, error};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OverlayUriPolicy {
    StrictCanonical,
    LspLocalFileV1,
}
impl OverlayUriPolicy {
    pub const fn id(self) -> &'static str {
        match self {
            Self::StrictCanonical => "strict-canonical-file-uri/1",
            Self::LspLocalFileV1 => "lsp-local-file-uri/1",
        }
    }
}
/// Private canonical URI is returned only to the calling owner consumer, never
/// automatically serialized in public workspace/diagnostic metadata.
pub struct OverlayResolvedDocument {
    pub relative_path: Box<str>,
    pub canonical_uri: Box<str>,
}

pub(super) fn decode_path(encoded: &str, max: usize) -> OverlayResult<String> {
    if encoded.len() > 65536 || max > 65536 {
        return Err(error(OverlayErrorCode::InvalidPath));
    }
    let mut decoded = Vec::with_capacity(encoded.len().min(max));
    let mut cursor = encoded.bytes();
    while let Some(byte) = cursor.next() {
        let byte = if byte == b'%' {
            let high = cursor
                .next()
                .and_then(hex)
                .ok_or_else(|| error(OverlayErrorCode::InvalidPath))?;
            let low = cursor
                .next()
                .and_then(hex)
                .ok_or_else(|| error(OverlayErrorCode::InvalidPath))?;
            let byte = (high << 4) | low;
            if matches!(byte, b'/' | b'\\') {
                return Err(error(OverlayErrorCode::InvalidPath));
            }
            byte
        } else {
            if !raw_path_byte(byte) {
                return Err(error(OverlayErrorCode::InvalidPath));
            }
            byte
        };
        if decoded.len() >= max {
            return Err(error(OverlayErrorCode::InvalidPath));
        }
        decoded.push(byte);
    }
    let value = String::from_utf8(decoded).map_err(|_| error(OverlayErrorCode::InvalidPath))?;
    if value.chars().any(char::is_control) || value.contains('\\') {
        return Err(error(OverlayErrorCode::InvalidPath));
    }
    Ok(value)
}
fn raw_path_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'.'
                | b'_'
                | b'~'
                | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'@'
                | b'/'
        )
}
fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

/// A decoded lexical absolute path with leading slash and normalized Windows
/// drive letter. All other component bytes remain exact, including case.
pub(super) fn decode_local_file_uri(uri: &str) -> OverlayResult<String> {
    if uri.len() > 65536
        || !uri
            .get(..7)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("file://"))
    {
        return Err(error(OverlayErrorCode::InvalidPath));
    }
    let encoded = &uri[7..];
    // Nonempty authorities, UNC and duplicate leading separators are refused.
    if !encoded.starts_with('/') || encoded.starts_with("//") {
        return Err(error(OverlayErrorCode::InvalidPath));
    }
    let mut path = decode_path(encoded, 65536)?;
    let start = if cfg!(windows) {
        let drive = path.as_bytes();
        if drive.len() < 4
            || drive[0] != b'/'
            || !drive[1].is_ascii_alphabetic()
            || drive[2] != b':'
            || drive[3] != b'/'
        {
            return Err(error(OverlayErrorCode::InvalidPath));
        }
        4
    } else {
        1
    };
    let mut components = path[start..].split('/').peekable();
    while let Some(component) = components.next() {
        if component.is_empty() && components.peek().is_none() {
            continue;
        }
        if component.is_empty() || matches!(component, "." | "..") {
            return Err(error(OverlayErrorCode::InvalidPath));
        }
        crate::disk::validate_path(component).map_err(|_| error(OverlayErrorCode::InvalidPath))?;
    }
    if cfg!(windows) {
        let upper = path.as_bytes()[1].to_ascii_uppercase();
        path.replace_range(1..2, &char::from(upper).to_string());
    }
    Ok(path)
}

#[cfg(test)]
mod tests;
