//! Native, non-executing APIDocumentation input lane for the Ketho Rust port.
//!
//! EmmyLua owns lexing and syntax. This module admits only declarative tables,
//! immutable local bindings and the exact registration call used by the donor.
//! Raw fields (including duplicates, nil and unknown metadata) are retained.
//! This is an in-memory source boundary, not a persistent ReferenceStore.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use emmylua_parser::{
    LexerConfig, LuaAstNode, LuaAstToken, LuaExpr, LuaIndexKey, LuaLanguageLevel, LuaLexer,
    LuaLiteralToken, LuaParser, LuaStat, LuaTokenKind, ParserConfig, Reader, string_token_value,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

const MAX_BYTES: usize = 1024 * 1024;
const MAX_NODES: usize = 65_536;
const MAX_DEPTH: usize = 48;
const MAX_TOKENS: usize = 262_144;
const MAX_BINDINGS: usize = 128;
const MAX_REGISTRATIONS: usize = 128;

/// Half-open UTF-8 byte range in the exact source, including its BOM if present.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

/// Source-owned table keys. Source order and duplicates remain observable.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum RawKey {
    Name(String),
    Index(u64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RawField {
    pub key: RawKey,
    pub value: RawValue,
    pub span: Span,
}

/// Numbers retain their original lexeme, not a rounded JSON/f64 approximation.
/// Symbolic Enum/Constants paths remain unresolved references, never invented values.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum RawKind {
    Nil,
    Boolean(bool),
    Number(String),
    String(String),
    Reference(Vec<String>),
    /// An unevaluated bare global name observed in a data value. It is never
    /// looked up in a host/Lua environment or treated as nil/a known constant.
    UnresolvedName(String),
    /// Only the corpus-required additive forms are admitted; the original
    /// operands and spans remain raw. The reference resolver owns evaluation.
    BinaryExpression {
        op: AdditiveOp,
        left: Box<RawValue>,
        right: Box<RawValue>,
    },
    Table(Vec<RawField>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum AdditiveOp {
    Add,
    Subtract,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RawValue {
    pub kind: RawKind,
    pub span: Span,
}

impl RawValue {
    pub fn fields(&self) -> Option<&[RawField]> {
        match &self.kind {
            RawKind::Table(fields) => Some(fields),
            _ => None,
        }
    }

    fn weight(&self) -> (usize, usize) {
        match &self.kind {
            RawKind::Table(fields) => fields.iter().fold((1, 0), |(n, b), f| {
                let (cn, cb) = f.value.weight();
                let key_bytes = match &f.key {
                    RawKey::Name(s) => s.len(),
                    _ => 0,
                };
                (n + cn + 1, b + cb + key_bytes)
            }),
            RawKind::String(s) | RawKind::Number(s) | RawKind::UnresolvedName(s) => (1, s.len()),
            RawKind::Reference(parts) => (1, parts.iter().map(String::len).sum()),
            RawKind::BinaryExpression { left, right, .. } => {
                let (ln, lb) = left.weight();
                let (rn, rb) = right.weight();
                (1 + ln + rn, lb + rb)
            }
            _ => (1, 0),
        }
    }
}

/// One exact APIDocumentation registration in source order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Registration {
    pub ordinal: usize,
    pub span: Span,
    pub value: RawValue,
}

/// Validated file identity and immutable registrations. No parser AST escapes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DocumentationDocument {
    revision: String,
    source_bytes: usize,
    evaluator: &'static str,
    path: String,
    sha256: String,
    registrations: Vec<Registration>,
}

impl DocumentationDocument {
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn registrations(&self) -> &[Registration] {
        &self.registrations
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum NativeErrorCode {
    InvalidIdentity,
    DigestMismatch,
    InvalidEncoding,
    Syntax,
    UnsupportedStatement,
    UnsupportedExpression,
    UnsupportedKey,
    UnsupportedString,
    UnknownBinding,
    InvalidRegistration,
    Limit,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NativeError {
    pub code: NativeErrorCode,
    pub span: Option<Span>,
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "native documentation input rejected: {:?}", self.code)
    }
}
impl std::error::Error for NativeError {}

type Result<T> = std::result::Result<T, NativeError>;
fn fail(code: NativeErrorCode) -> NativeError {
    NativeError { code, span: None }
}
fn at(code: NativeErrorCode, span: Span) -> NativeError {
    NativeError {
        code,
        span: Some(span),
    }
}
fn span(node: &impl LuaAstNode) -> Span {
    let range = node.syntax().text_range();
    Span {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

/// SHA-256 in the same explicit identifier form used by source manifests.
pub fn source_digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut text = String::from("sha256:");
    for byte in hash {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

/// Read an exact caller-selected file. No IO, source execution, fallback or discovery.
/// Any unsupported statement invalidates this file; callers retain other files as partial.
pub fn ingest_document(
    revision: &str,
    path: &str,
    text: &str,
    expected_sha256: &str,
    cancelled: &AtomicBool,
) -> Result<DocumentationDocument> {
    if !matches!(revision.len(), 40 | 64)
        || !revision
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || path.is_empty()
        || path.len() > 4096
        || path.contains(['\\', ':'])
        || path.chars().any(char::is_control)
        || path
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
    {
        return Err(fail(NativeErrorCode::InvalidIdentity));
    }
    if text.len() > MAX_BYTES {
        return Err(fail(NativeErrorCode::Limit));
    }
    if cancelled.load(Ordering::Relaxed) {
        return Err(fail(NativeErrorCode::Cancelled));
    }
    if text.contains('\0') {
        return Err(fail(NativeErrorCode::InvalidEncoding));
    }
    let digest = source_digest(text.as_bytes());
    if digest != expected_sha256 {
        return Err(fail(NativeErrorCode::DigestMismatch));
    }
    // Preserve byte offsets while accepting UTF-8 BOM files.
    let input = text
        .strip_prefix('\u{feff}')
        .map(|rest| format!("   {rest}"));
    let input = input.as_deref().unwrap_or(text);
    preflight(input)?;
    let mut config = ParserConfig::with_level(LuaLanguageLevel::Lua51);
    config.enable_emmylua_doc = false; // source comments are inert data, not annotation input
    let tree = LuaParser::parse(input, config);
    if !tree.get_errors().is_empty() {
        return Err(fail(NativeErrorCode::Syntax));
    }
    let block = tree
        .get_chunk_node()
        .get_block()
        .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
    let mut evaluator = Evaluator {
        bindings: BTreeMap::new(),
        remaining: MAX_NODES,
        bytes: MAX_BYTES * 4,
        cancelled,
    };
    let mut registrations = Vec::new();
    for statement in block.get_stats() {
        evaluator.tick(1, 0)?;
        let location = span(&statement);
        match statement {
            LuaStat::LocalStat(local) => {
                let names = local.get_local_name_list().collect::<Vec<_>>();
                let values = local.get_value_exprs().collect::<Vec<_>>();
                if names.len() != 1 || values.len() != 1 || names[0].get_attrib().is_some() {
                    return Err(at(NativeErrorCode::UnsupportedStatement, location));
                }
                let name = names[0]
                    .get_name_token()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?
                    .get_name_text()
                    .to_owned();
                if matches!(name.as_str(), "APIDocumentation" | "Enum" | "Constants")
                    || evaluator.bindings.contains_key(&name)
                    || evaluator.bindings.len() >= MAX_BINDINGS
                {
                    return Err(at(NativeErrorCode::UnsupportedStatement, location));
                }
                let value = evaluator.value(values[0].clone(), 0)?;
                evaluator.bindings.insert(name, value);
            }
            LuaStat::CallExprStat(stat) => {
                let call = stat
                    .get_call_expr()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                let prefix = call
                    .get_prefix_expr()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                let LuaExpr::IndexExpr(index) = prefix else {
                    return Err(at(NativeErrorCode::InvalidRegistration, location));
                };
                let receiver = index.get_prefix_expr();
                let method = index.get_name_token().map(|t| t.get_name_text().to_owned());
                if !call.is_colon_call()
                    || method.as_deref() != Some("AddDocumentationTable")
                    || !matches!(receiver, Some(LuaExpr::NameExpr(ref n)) if n.get_name_text().as_deref() == Some("APIDocumentation"))
                {
                    return Err(at(NativeErrorCode::InvalidRegistration, location));
                }
                let arguments = call
                    .get_args_list()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?
                    .get_args()
                    .collect::<Vec<_>>();
                if arguments.len() != 1 {
                    return Err(at(NativeErrorCode::InvalidRegistration, location));
                }
                let value = evaluator.value(arguments[0].clone(), 0)?;
                if !matches!(value.kind, RawKind::Table(_)) {
                    return Err(at(NativeErrorCode::InvalidRegistration, location));
                }
                if registrations.len() >= MAX_REGISTRATIONS {
                    return Err(fail(NativeErrorCode::Limit));
                }
                registrations.push(Registration {
                    ordinal: registrations.len(),
                    span: location,
                    value,
                });
            }
            LuaStat::EmptyStat(_) => {}
            _ => return Err(at(NativeErrorCode::UnsupportedStatement, location)),
        }
    }
    if registrations.is_empty() {
        return Err(fail(NativeErrorCode::InvalidRegistration));
    }
    evaluator.tick(0, 0)?;
    Ok(DocumentationDocument {
        revision: revision.to_owned(),
        source_bytes: text.len(),
        evaluator: "ketho-apidoc-declarative/2",
        path: path.to_owned(),
        sha256: digest,
        registrations,
    })
}

// This is a budget/allow-list pass over Emmy tokens, not a second Lua lexer/parser.
// Delimiter and chain limits bound recursion before the upstream parser sees input.
fn preflight(text: &str) -> Result<()> {
    use LuaTokenKind::*;
    let mut errors = Vec::new();
    let tokens = LuaLexer::new(
        Reader::new(text),
        LexerConfig::new(LuaLanguageLevel::Lua51),
        Some(&mut errors),
    )
    .tokenize();
    if !errors.is_empty() {
        return Err(fail(NativeErrorCode::Syntax));
    }
    if tokens.len() > MAX_TOKENS {
        return Err(fail(NativeErrorCode::Limit));
    }
    let mut depth = 0usize;
    let mut chain = 0usize;
    for token in tokens {
        if matches!(
            token.kind,
            TkWhitespace | TkEndOfLine | TkShortComment | TkLongComment
        ) {
            continue;
        }
        match token.kind {
            TkLeftParen | TkLeftBrace | TkLeftBracket => {
                depth += 1;
                chain = 0;
            }
            TkRightParen | TkRightBrace | TkRightBracket => {
                depth = depth.saturating_sub(1);
                chain = 0;
            }
            TkAssign | TkComma | TkSemicolon => chain = 0,
            TkLocal | TkName | TkString | TkLongString | TkInt | TkFloat | TkNil | TkTrue
            | TkFalse | TkDot | TkColon | TkMinus | TkPlus => chain += 1,
            _ => return Err(fail(NativeErrorCode::UnsupportedExpression)),
        }
        if depth > MAX_DEPTH || chain > MAX_DEPTH {
            return Err(fail(NativeErrorCode::Limit));
        }
    }
    Ok(())
}

struct Evaluator<'a> {
    bindings: BTreeMap<String, RawValue>,
    remaining: usize,
    bytes: usize,
    cancelled: &'a AtomicBool,
}
impl Evaluator<'_> {
    fn tick(&mut self, nodes: usize, bytes: usize) -> Result<()> {
        if self.cancelled.load(Ordering::Relaxed) {
            return Err(fail(NativeErrorCode::Cancelled));
        }
        self.remaining = self
            .remaining
            .checked_sub(nodes)
            .ok_or_else(|| fail(NativeErrorCode::Limit))?;
        self.bytes = self
            .bytes
            .checked_sub(bytes)
            .ok_or_else(|| fail(NativeErrorCode::Limit))?;
        Ok(())
    }
    fn value(&mut self, expression: LuaExpr, depth: usize) -> Result<RawValue> {
        self.tick(1, 0)?;
        if depth > MAX_DEPTH {
            return Err(fail(NativeErrorCode::Limit));
        }
        let location = span(&expression);
        let kind = match expression {
            LuaExpr::LiteralExpr(literal) => match literal
                .get_literal()
                .ok_or_else(|| fail(NativeErrorCode::Syntax))?
            {
                LuaLiteralToken::String(token) => {
                    let value = decode_string(token.syntax(), location)?;
                    self.tick(0, value.len())?;
                    RawKind::String(value)
                }
                LuaLiteralToken::Number(token) if !token.is_complex() => {
                    RawKind::Number(token.syntax().text().to_owned())
                }
                LuaLiteralToken::Bool(token) => RawKind::Boolean(token.syntax().text() == "true"),
                LuaLiteralToken::Nil(_) => RawKind::Nil,
                _ => return Err(at(NativeErrorCode::UnsupportedExpression, location)),
            },
            LuaExpr::TableExpr(table) => {
                let mut fields = Vec::new();
                let mut ordinal = 0u64;
                for field in table.get_fields() {
                    self.tick(1, 0)?;
                    let key = if field.is_value_field() {
                        ordinal += 1;
                        RawKey::Index(ordinal)
                    } else {
                        match field
                            .get_field_key()
                            .ok_or_else(|| at(NativeErrorCode::UnsupportedKey, span(&field)))?
                        {
                            LuaIndexKey::Name(name) => {
                                RawKey::Name(name.get_name_text().to_owned())
                            }
                            LuaIndexKey::String(token) => {
                                RawKey::Name(decode_string(token.syntax(), span(&field))?)
                            }
                            LuaIndexKey::Integer(token) => {
                                RawKey::Index(token.syntax().text().parse::<u64>().map_err(
                                    |_| at(NativeErrorCode::UnsupportedKey, span(&field)),
                                )?)
                            }
                            _ => return Err(at(NativeErrorCode::UnsupportedKey, span(&field))),
                        }
                    };
                    let value = self.value(
                        field
                            .get_value_expr()
                            .ok_or_else(|| fail(NativeErrorCode::Syntax))?,
                        depth + 1,
                    )?;
                    fields.push(RawField {
                        key,
                        value,
                        span: span(&field),
                    });
                }
                RawKind::Table(fields)
            }
            LuaExpr::NameExpr(name) => {
                let name = name
                    .get_name_text()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                let Some(bound) = self.bindings.get(&name) else {
                    // A raw unresolved leaf does not authorize a value, a host
                    // lookup or registration. General calls/indexing still fail.
                    if depth == 0 {
                        return Err(at(NativeErrorCode::UnknownBinding, location));
                    }
                    self.tick(0, name.len())?;
                    return Ok(RawValue {
                        kind: RawKind::UnresolvedName(name),
                        span: location,
                    });
                };
                let weight = bound.weight();
                self.tick(weight.0, weight.1)?;
                return self
                    .bindings
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| fail(NativeErrorCode::UnknownBinding));
            }
            LuaExpr::ParenExpr(paren) => {
                return self.value(
                    paren
                        .get_expr()
                        .ok_or_else(|| fail(NativeErrorCode::Syntax))?,
                    depth + 1,
                );
            }
            LuaExpr::BinaryExpr(binary) => {
                let op = match binary
                    .get_op_token()
                    .map(|t| t.syntax().text().to_owned())
                    .as_deref()
                {
                    Some("+") => AdditiveOp::Add,
                    Some("-") => AdditiveOp::Subtract,
                    _ => return Err(at(NativeErrorCode::UnsupportedExpression, location)),
                };
                let (left, right) = binary
                    .get_exprs()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                let left = self.value(left, depth + 1)?;
                let right = self.value(right, depth + 1)?;
                RawKind::BinaryExpression {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            }
            LuaExpr::UnaryExpr(unary) => {
                let op = unary
                    .get_op_token()
                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                if op.syntax().text() != "-" {
                    return Err(at(NativeErrorCode::UnsupportedExpression, location));
                }
                let value = self.value(
                    unary
                        .get_expr()
                        .ok_or_else(|| fail(NativeErrorCode::Syntax))?,
                    depth + 1,
                )?;
                let RawKind::Number(number) = value.kind else {
                    return Err(at(NativeErrorCode::UnsupportedExpression, location));
                };
                if number.starts_with('-') {
                    return Err(at(NativeErrorCode::UnsupportedExpression, location));
                }
                RawKind::Number(format!("-{number}"))
            }
            LuaExpr::IndexExpr(index) => {
                // Retain symbolic generated Enum/Constants metadata without evaluating globals.
                let mut parts = Vec::new();
                let mut current = LuaExpr::IndexExpr(index);
                loop {
                    if parts.len() >= MAX_DEPTH {
                        return Err(fail(NativeErrorCode::Limit));
                    }
                    match current {
                        LuaExpr::IndexExpr(index) if !index.is_safe_index() => {
                            if index
                                .get_index_token()
                                .is_none_or(|t| t.syntax().text() != ".")
                            {
                                return Err(at(NativeErrorCode::UnsupportedExpression, location));
                            }
                            parts.push(
                                index
                                    .get_name_token()
                                    .ok_or_else(|| fail(NativeErrorCode::Syntax))?
                                    .get_name_text()
                                    .to_owned(),
                            );
                            current = index
                                .get_prefix_expr()
                                .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                        }
                        LuaExpr::NameExpr(name) => {
                            let root = name
                                .get_name_text()
                                .ok_or_else(|| fail(NativeErrorCode::Syntax))?;
                            if !matches!(root.as_str(), "Enum" | "Constants") {
                                return Err(at(NativeErrorCode::UnknownBinding, location));
                            }
                            parts.push(root);
                            break;
                        }
                        _ => return Err(at(NativeErrorCode::UnsupportedExpression, location)),
                    }
                }
                parts.reverse();
                RawKind::Reference(parts)
            }
            _ => return Err(at(NativeErrorCode::UnsupportedExpression, location)),
        };
        Ok(RawValue {
            kind,
            span: location,
        })
    }
}

// A conservative byte-string capability guard around the upstream decoder, not
// a second decoder. Numeric/hex/Unicode escapes require a byte-valued raw lane.
// Reject them rather than accepting upstream char-casts that change Lua bytes.
fn decode_string(token: &emmylua_parser::LuaSyntaxToken, location: Span) -> Result<String> {
    if token.text().starts_with(['\'', '"']) {
        let mut chars = token.text().chars();
        while let Some(c) = chars.next() {
            if c == '\\'
                && let Some(escape) = chars.next()
                && (escape.is_ascii_digit() || matches!(escape, 'x' | 'u'))
            {
                return Err(at(NativeErrorCode::UnsupportedString, location));
            }
        }
    }
    let value =
        string_token_value(token).map_err(|_| at(NativeErrorCode::UnsupportedString, location))?;
    // Lua normalizes physical newline sequences in long strings. The upstream
    // returns those bytes unchanged, so this lane reports them unsupported.
    if token.text().starts_with('[') && token.text().contains('\r') {
        return Err(at(NativeErrorCode::UnsupportedString, location));
    }
    Ok(value)
}
