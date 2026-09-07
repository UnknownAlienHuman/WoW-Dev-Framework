//! Lossless UTF-8 string admission around Emmy's token decoder, not a Lua parser.
use super::{NativeErrorCode, Result, Span, at};
use emmylua_parser::{LuaSyntaxToken, string_token_value};

/// The flag records use of the extended string profile without changing receipts
/// for inputs already admitted by the original evaluator.
pub(super) fn decode(token: &LuaSyntaxToken, location: Span) -> Result<(String, bool)> {
    let raw = token.text();
    let long = raw.starts_with('[');
    let extended = if long {
        raw.contains('\r')
    } else {
        short_profile(raw, location)?
    };
    let mut value =
        string_token_value(token).map_err(|_| at(NativeErrorCode::UnsupportedString, location))?;
    if long && extended {
        // Emmy already removes the optional first newline. Its current decoder
        // consumes only LF from an initial LFCR pair. Reconcile only that exact
        // legacy result; a decoder which consumes both bytes is accepted too.
        let boundary = raw.bytes().skip(1).take_while(|byte| *byte == b'=').count() + 2;
        if raw
            .get(boundary..)
            .is_some_and(|body| body.starts_with("\n\r"))
        {
            let body = raw
                .get(boundary + 2..raw.len().saturating_sub(boundary))
                .ok_or_else(|| at(NativeErrorCode::UnsupportedString, location))?;
            if value.strip_prefix('\r') == Some(body) {
                value.remove(0);
            } else if value != body {
                return Err(at(NativeErrorCode::UnsupportedString, location));
            }
        }
        value = normalize_newlines(&value);
    }
    Ok((value, extended))
}

fn short_profile(raw: &str, location: Span) -> Result<bool> {
    let mut chars = raw.chars().peekable();
    let mut extended = false;
    while let Some(character) = chars.next() {
        if character != '\\' {
            continue;
        }
        let escape = chars
            .next()
            .ok_or_else(|| at(NativeErrorCode::UnsupportedString, location))?;
        if escape.is_ascii_digit() {
            let mut number = escape as u16 - u16::from(b'0');
            for _ in 0..2 {
                let Some(digit) = chars.peek().copied().filter(char::is_ascii_digit) else {
                    break;
                };
                let _ = chars.next();
                number = number * 10 + digit as u16 - u16::from(b'0');
            }
            // For ASCII, Emmy's character conversion is byte-identical to Lua.
            // Larger bytes must not become UTF-8 code points or disappear on
            // overflow. They still require a separate byte-valued raw contract.
            if number > 127 {
                return Err(at(NativeErrorCode::UnsupportedString, location));
            }
            extended = true;
        } else if matches!(escape, 'x' | 'u' | '\r')
            || (escape == '\n' && chars.peek() == Some(&'\r'))
        {
            // Hex/Unicode forms stay outside Lua 5.1. Quoted physical CR pairs
            // cannot be normalized after decoding without changing explicit \r.
            return Err(at(NativeErrorCode::UnsupportedString, location));
        }
    }
    Ok(extended)
}

fn normalize_newlines(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if matches!(character, '\r' | '\n') {
            if chars
                .peek()
                .is_some_and(|next| matches!(next, '\r' | '\n') && *next != character)
            {
                let _ = chars.next();
            }
            output.push('\n');
        } else {
            output.push(character);
        }
    }
    output
}
