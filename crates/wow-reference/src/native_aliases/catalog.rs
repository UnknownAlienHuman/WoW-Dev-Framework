//! Lexer-bounded comment grouping and Emmy-owned closed string union extraction.
use super::{Result, error};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{LuaAstToken, LuaDocType, LuaLiteralToken, LuaTypeBinaryOperator};
use std::collections::BTreeSet;

const MAX_VALUES: usize = 256;

pub(super) fn groups(input: &str, comments: impl Iterator<Item = Span>) -> Result<Vec<Span>> {
    let mut groups: Vec<Span> = Vec::new();
    let mut continuations = 0;
    for span in comments {
        let text = input
            .get(span.start..span.end)
            .ok_or_else(|| error(NativeErrorCode::Syntax))?;
        if text.starts_with("---|") {
            let previous = groups
                .last_mut()
                .ok_or_else(|| error(NativeErrorCode::UnsupportedExpression))?;
            let gap = &input[previous.end..span.start];
            if gap.bytes().filter(|&b| b == b'\n').count() != 1
                || !gap.chars().all(char::is_whitespace)
            {
                return Err(error(NativeErrorCode::UnsupportedExpression));
            }
            continuations += 1;
            if continuations > MAX_VALUES || span.end - previous.start > 64 * 1024 {
                return Err(error(NativeErrorCode::Limit));
            }
            previous.end = span.end;
        } else {
            continuations = 0;
            groups.push(span);
        }
    }
    Ok(groups)
}

pub(super) fn string_values(ty: LuaDocType) -> Option<Vec<String>> {
    let mut values = Vec::new();
    if !collect(ty, &mut values, 0) || values.is_empty() {
        return None;
    }
    let unique = values.iter().collect::<BTreeSet<_>>();
    (unique.len() == values.len()).then_some(values)
}

fn collect(ty: LuaDocType, values: &mut Vec<String>, depth: usize) -> bool {
    if depth >= 16 || values.len() >= MAX_VALUES {
        return false;
    }
    match ty {
        LuaDocType::Literal(literal) => {
            let Some(LuaLiteralToken::String(token)) = literal.get_literal() else {
                return false;
            };
            // This profile admits printable ASCII without escape decoding. Lua
            // byte escapes, Unicode and mixed/open unions stay explicitly unsupported.
            let raw = token.get_text();
            let quote = raw.as_bytes().first().copied();
            if !matches!(quote, Some(b'\'' | b'"')) || raw.as_bytes().last().copied() != quote {
                return false;
            }
            let Some(value) = raw.get(1..raw.len().saturating_sub(1)) else {
                return false;
            };
            if value.len() > 128
                || !value
                    .bytes()
                    .all(|b| (b' '..=b'~').contains(&b) && b != b'"' && b != b'\\')
            {
                return false;
            }
            values.push(value.to_owned());
            true
        }
        LuaDocType::Binary(binary)
            if binary
                .get_op_token()
                .is_some_and(|op| op.get_op() == LuaTypeBinaryOperator::Union) =>
        {
            let Some((left, right)) = binary.get_types() else {
                return false;
            };
            collect(left, values, depth + 1) && collect(right, values, depth + 1)
        }
        LuaDocType::MultiLineUnion(union) => {
            for field in union.get_fields() {
                let Some(ty) = field.get_type() else {
                    return false;
                };
                if !collect(ty, values, depth + 1) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}
