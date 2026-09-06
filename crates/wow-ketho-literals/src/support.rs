//! Literal-profile validation and bounded output; never evaluates source text.
use wow_render_contract::{LiteralError, MAX_TEXT_BYTES};
pub(crate) struct Output {
    pub(crate) bytes: String,
    pub(crate) limit: usize,
}
impl Output {
    pub(crate) fn push(&mut self, text: &str) -> Result<(), LiteralError> {
        if self.bytes.len().saturating_add(text.len()) > self.limit {
            return Err(LiteralError::OutputLimit);
        }
        self.bytes.push_str(text);
        Ok(())
    }
}
pub(crate) fn identifier(value: &str) -> Result<(), LiteralError> {
    if value.is_empty() || value.len() > 1024 {
        return Err(LiteralError::InvalidIdentifier);
    }
    let mut chars = value.bytes();
    if !chars
        .next()
        .is_some_and(|b| b == b'_' || b.is_ascii_alphabetic())
        || !chars.all(|b| b == b'_' || b.is_ascii_alphanumeric())
        || matches!(
            value,
            "and"
                | "break"
                | "do"
                | "else"
                | "elseif"
                | "end"
                | "false"
                | "for"
                | "function"
                | "goto"
                | "if"
                | "in"
                | "local"
                | "nil"
                | "not"
                | "or"
                | "repeat"
                | "return"
                | "then"
                | "true"
                | "until"
                | "while"
        )
    {
        return Err(LiteralError::InvalidIdentifier);
    }
    Ok(())
}
pub(crate) fn safe_text(value: &str) -> Result<(), LiteralError> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(LiteralError::InputLimit);
    }
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(LiteralError::UnsafeDocumentation);
    }
    Ok(())
}
