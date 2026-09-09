//! Explicit, annotation-only alias resources for the Ketho port.
//!
//! This is NOT Blizzard reference truth or an automatic correction source.
//! EmmyLua parses the input; no Lua statement, diagnostic directive, namespace
//! change or loader is admitted. The original resource and all declarations
//! survive in the report, including unsupported type forms and duplicates.

use std::sync::atomic::{AtomicBool, Ordering};

use emmylua_parser::{
    LexerConfig, LuaAstNode, LuaDocTag, LuaDocType, LuaLanguageLevel, LuaLexer, LuaParser,
    LuaTokenKind, LuaTypeBinaryOperator, ParserConfig, Reader,
};
use serde::Serialize;

use crate::native::{NativeError, NativeErrorCode, Span, source_digest};

mod catalog;
mod open_strings;
mod structures;
pub use structures::{StructureFact, StructureField, StructureFieldType};

const MAX_BYTES: usize = 256 * 1024;
const MAX_ALIASES: usize = 4096;
const MAX_TERMS: usize = 16;

/// One observed alias. Unsupported forms have neither terms nor string values;
/// they are retained, never replaced with `any`.
#[derive(Clone, Debug, Serialize)]
pub struct AliasFact {
    pub name: String,
    pub terms: Option<Vec<String>>,
    /// Literal values, disjoint from named/primitive `terms`; closed unless `string_base` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_values: Option<Vec<String>>,
    /// An explicit open base with literal completion hints, not a closed whitelist.
    /// Absent for the retained named and closed-literal resource profiles.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_base: Option<&'static str>,
    pub syntax_error: bool,
    pub span: Span,
}

/// Immutable external resource. Its own revision is independent of the selected
/// Blizzard revision. Raw bytes are kept as UTF-8 text for review and provenance.
#[derive(Clone, Debug, Serialize)]
pub struct AliasDocument {
    schema: &'static str,
    revision: String,
    path: String,
    sha256: String,
    source_bytes: usize,
    text: String,
    aliases: Vec<AliasFact>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    structures: Vec<StructureFact>,
}
impl AliasDocument {
    pub fn revision(&self) -> &str {
        &self.revision
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn aliases(&self) -> &[AliasFact] {
        &self.aliases
    }
    pub fn structures(&self) -> &[StructureFact] {
        &self.structures
    }
}

type Result<T> = std::result::Result<T, NativeError>;
fn error(code: NativeErrorCode) -> NativeError {
    NativeError { code, span: None }
}
fn location(node: &impl LuaAstNode) -> Span {
    let range = node.syntax().text_range();
    Span {
        start: u32::from(range.start()) as usize,
        end: u32::from(range.end()) as usize,
    }
}

/// Retained single-comment named/primitive-union profile. Existing callers keep
/// their admission rules and v1 bytes; the Git driver uses `ingest_alias_catalog`.
pub fn ingest_aliases(
    revision: &str,
    path: &str,
    text: &str,
    expected_sha256: &str,
    cancelled: &AtomicBool,
) -> Result<AliasDocument> {
    ingest(revision, path, text, expected_sha256, cancelled, false)
}

/// Admit named aliases and bounded Ketho string-enum resources. An explicit
/// leading `string` base remains open while its literal hints are preserved.
/// Comment-only classes with named fields are also admitted as an external
/// structure profile; they never replace Blizzard facts or imply inheritance.
/// Only contiguous continuation comments join an alias. Emmy owns type parsing;
/// malformed declarations cannot consume their independently parsed siblings.
pub fn ingest_alias_catalog(
    revision: &str,
    path: &str,
    text: &str,
    expected_sha256: &str,
    cancelled: &AtomicBool,
) -> Result<AliasDocument> {
    ingest(revision, path, text, expected_sha256, cancelled, true)
}

fn ingest(
    revision: &str,
    path: &str,
    text: &str,
    expected_sha256: &str,
    cancelled: &AtomicBool,
    string_enums: bool,
) -> Result<AliasDocument> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
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
        return Err(error(NativeErrorCode::InvalidIdentity));
    }
    if text.len() > MAX_BYTES {
        return Err(error(NativeErrorCode::Limit));
    }
    if text.contains('\0') {
        return Err(error(NativeErrorCode::InvalidEncoding));
    }
    let digest = source_digest(text.as_bytes());
    if digest != expected_sha256 {
        return Err(error(NativeErrorCode::DigestMismatch));
    }
    // Replace the three-byte UTF-8 BOM with three spaces, preserving spans.
    let input = text
        .strip_prefix('\u{feff}')
        .map(|rest| format!("   {rest}"));
    let input = input.as_deref().unwrap_or(text);
    // Bound line complexity before invoking the upstream doc parser. The
    // catalog also bounds continuation groups before parsing their type AST.
    for line in input.lines() {
        if line.len() > 2048 || line.bytes().filter(|b| b"([{<?|".contains(b)).count() > 48 {
            return Err(error(NativeErrorCode::Limit));
        }
        if !string_enums && line.trim_start().starts_with("---|") {
            return Err(error(NativeErrorCode::UnsupportedExpression));
        }
    }
    let mut errors = Vec::new();
    let tokens = LuaLexer::new(
        Reader::new(input),
        LexerConfig::new(LuaLanguageLevel::Lua51),
        Some(&mut errors),
    )
    .tokenize();
    if !errors.is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    if tokens.iter().any(|t| {
        !matches!(
            t.kind,
            LuaTokenKind::TkShortComment
                | LuaTokenKind::TkWhitespace
                | LuaTokenKind::TkEndOfLine
                | LuaTokenKind::TkEof
        )
    }) {
        return Err(error(NativeErrorCode::UnsupportedStatement));
    }
    let comments = tokens
        .iter()
        .filter(|t| t.kind == LuaTokenKind::TkShortComment)
        .map(|t| Span {
            start: t.range.start_offset,
            end: t.range.end_offset(),
        });
    let groups = if string_enums {
        catalog::groups(input, comments)?
    } else {
        comments.collect()
    };
    let mut aliases = Vec::new();
    let mut structures = Vec::new();
    let mut active_structure = None;
    let mut field_count = 0usize;
    for Span { start, end } in groups {
        if cancelled.load(Ordering::Relaxed) {
            return Err(error(NativeErrorCode::Cancelled));
        }
        let comment = input
            .get(start..end)
            .ok_or_else(|| error(NativeErrorCode::Syntax))?;
        let tree = LuaParser::parse(comment, ParserConfig::with_level(LuaLanguageLevel::Lua51));
        let syntax_error = !tree.get_errors().is_empty();
        let multiline = comment.contains('\n');
        let mut observed_alias = false;
        let mut observed_structure = false;
        for tag in tree
            .get_chunk_node()
            .syntax()
            .descendants()
            .filter_map(LuaDocTag::cast)
        {
            match tag {
                LuaDocTag::Class(class) if string_enums && !multiline => {
                    observed_structure = true;
                    structures.push(structures::class(
                        &class,
                        Span { start, end },
                        syntax_error,
                    )?);
                    active_structure = Some(structures.len() - 1);
                }
                LuaDocTag::Field(field) if string_enums && !multiline => {
                    observed_structure = true;
                    let target = active_structure
                        .and_then(|index| structures.get_mut(index))
                        .ok_or_else(|| error(NativeErrorCode::UnsupportedStatement))?;
                    structures::field(
                        target,
                        &field,
                        Span { start, end },
                        syntax_error,
                        &mut field_count,
                    )?;
                }
                LuaDocTag::Alias(alias) => {
                    active_structure = None;
                    if observed_alias || aliases.len() >= MAX_ALIASES {
                        return Err(error(NativeErrorCode::Limit));
                    }
                    observed_alias = true;
                    let name = alias
                        .get_name_token()
                        .ok_or_else(|| error(NativeErrorCode::Syntax))?
                        .get_name_text()
                        .to_owned();
                    let plain = !syntax_error
                        && alias.get_generic_decl_list().is_none()
                        && alias.get_type_flag().is_none();
                    let mut terms = Vec::new();
                    let supported = plain
                        && !multiline
                        && alias
                            .get_type()
                            .is_some_and(|ty| collect_terms(ty, &mut terms, 0));
                    let mut string_values = if string_enums && plain {
                        alias.get_type().and_then(catalog::string_values)
                    } else {
                        None
                    };
                    let string_base = if string_enums && plain && string_values.is_none() {
                        string_values = open_strings::values(&alias, comment);
                        string_values.as_ref().map(|_| "string")
                    } else {
                        None
                    };
                    let span = if syntax_error || string_base.is_some() {
                        Span { start, end }
                    } else {
                        let local = location(&alias);
                        Span {
                            start: start + local.start,
                            end: start + local.end,
                        }
                    };
                    aliases.push(AliasFact {
                        name,
                        terms: supported.then_some(terms),
                        string_values,
                        string_base,
                        syntax_error,
                        span,
                    });
                }
                LuaDocTag::Meta(meta)
                    if !syntax_error
                        && meta
                            .get_name_token()
                            .is_none_or(|t| t.get_name_text() == "_") =>
                {
                    active_structure = None;
                }
                _ => return Err(error(NativeErrorCode::UnsupportedStatement)),
            }
        }
        if multiline && !observed_alias {
            return Err(error(NativeErrorCode::UnsupportedExpression));
        }
        if aliases.len() + structures.len() > MAX_ALIASES {
            return Err(error(NativeErrorCode::Limit));
        }
        if syntax_error && !observed_alias && !observed_structure {
            return Err(error(NativeErrorCode::Syntax));
        }
    }
    if aliases.is_empty() && structures.is_empty() {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    Ok(AliasDocument {
        schema: if !structures.is_empty() {
            "wow-native-alias-resource/4"
        } else if aliases.iter().any(|alias| alias.string_base.is_some()) {
            "wow-native-alias-resource/3"
        } else if aliases.iter().any(|alias| alias.string_values.is_some()) {
            "wow-native-alias-resource/2"
        } else {
            "wow-native-alias-resource/1"
        },
        revision: revision.into(),
        path: path.into(),
        sha256: digest,
        source_bytes: text.len(),
        text: text.into(),
        aliases,
        structures,
    })
}

fn collect_terms(ty: LuaDocType, terms: &mut Vec<String>, depth: usize) -> bool {
    if depth >= MAX_TERMS || terms.len() >= MAX_TERMS {
        return false;
    }
    match ty {
        LuaDocType::Name(name) if name.get_generic_param().is_none() => {
            let Some(name) = name.get_name_text() else {
                return false;
            };
            terms.push(name);
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
            collect_terms(left, terms, depth + 1) && collect_terms(right, terms, depth + 1)
        }
        _ => false,
    }
}
