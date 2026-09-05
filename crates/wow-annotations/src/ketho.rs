//! Rust port of `Ketho/vscode-wow-api:luasrc/annotate/init.lua` and the
//! `wowdoc/init.lua` name/argument helpers. See `../THIRD_PARTY_NOTICES.md`.
//!
//! Ketho's field ordering and annotation conventions are preserved. Caller-supplied
//! enum names and resolved widget names replace Lua globals; no build-specific
//! enum/widget inventory is embedded. Unrepresentable input returns an error, not
//! a silently widened type or partially successful artifact. Output is analysis
//! data, never an addon implementation to execute.

use std::collections::BTreeSet;
use std::fmt;

const MAX_NAME_BYTES: usize = 1024;
pub(crate) const MAX_TEXT_BYTES: usize = 64 * 1024;
pub(crate) const MAX_ITEMS: usize = 4096;
pub(crate) const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;

/// Explicit namespace/receiver ownership, rather than a guessed dotted name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Owner {
    Global,
    Namespace(String),
    /// The reference layer supplies the current widget alias, if one is known.
    ScriptObject {
        system_name: String,
        annotation_name: Option<String>,
    },
}

/// Ketho field inputs. Default text is already the selected scalar's display
/// representation, not Lua code; `Some("false")` and `Some("0")` remain present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub type_name: String,
    pub inner_type: Option<String>,
    pub nilable: bool,
    pub default_text: Option<String>,
    pub variadic: bool,
}

/// A callable with ordered arguments and ordered multiple returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    /// `None` and a present empty documentation array have different Ketho output.
    pub documentation: Option<Vec<String>>,
    pub arguments: Vec<Field>,
    pub returns: Vec<Field>,
}

/// Supported named table projections from Ketho's `GetSystem`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Table {
    Structure {
        name: String,
        fields: Vec<Field>,
    },
    /// Only argument-only callbacks are admitted by this initial donor slice.
    /// A reference adapter must reject/report callbacks with unprojected returns.
    Callback {
        name: String,
        arguments: Vec<Field>,
    },
}

/// Ordered declaration model for the renderer, not an authoritative ReferenceView.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct System {
    pub owner: Owner,
    pub functions: Vec<Function>,
    pub tables: Vec<Table>,
}

/// Fixed, non-source-bearing failure classes. No invalid source text is echoed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderError {
    Cancelled,
    InvalidSource,
    InvalidIdentifier,
    UnsupportedType,
    UnsafeDocumentation,
    InvalidVariadic,
    DuplicateName,
    InputLimit,
    OutputLimit,
    UnsupportedLiteral,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "annotation projection cancelled",
            Self::InvalidSource => "annotation sources are inconsistent",
            Self::InvalidIdentifier => "annotation identifier is not representable",
            Self::UnsupportedType => "annotation type is not supported by this renderer profile",
            Self::UnsafeDocumentation => "annotation documentation requires explicit sanitization",
            Self::InvalidVariadic => "variadic declaration is not representable",
            Self::DuplicateName => "duplicate declaration or argument name",
            Self::InputLimit => "annotation input exceeds the renderer limit",
            Self::OutputLimit => "annotation output exceeds the renderer limit",
            Self::UnsupportedLiteral => "literal is not representable in this annotation profile",
        })
    }
}

impl std::error::Error for RenderError {}

/// Declaration ranges measured on the final UTF-8 output. Indices refer to the
/// ordered renderer input; source identities are joined by the reference adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedDeclaration {
    pub table: bool,
    pub index: usize,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedSystem {
    pub text: String,
    pub declarations: Vec<RenderedDeclaration>,
}

/// A pure, bounded Ketho-compatible rendering profile. It has no runtime or IO
/// dependencies. The enum set belongs to the selected input, not a global cache.
#[derive(Clone, Debug)]
pub struct Renderer {
    enum_names: BTreeSet<String>,
    max_output_bytes: usize,
}

impl Renderer {
    pub fn new(enum_names: BTreeSet<String>, max_output_bytes: usize) -> Result<Self, RenderError> {
        if enum_names.len() > MAX_ITEMS || max_output_bytes > MAX_OUTPUT_BYTES {
            return Err(RenderError::InputLimit);
        }
        for name in &enum_names {
            identifier(name)?;
        }
        Ok(Self {
            enum_names,
            max_output_bytes,
        })
    }

    /// Ketho `GetType`: bool/cstring/luaIndex aliases and explicit Enum membership.
    /// Unknown valid named types stay named; they are never replaced by `any`.
    pub fn lower_type(&self, name: &str) -> Result<String, RenderError> {
        let primitive = match name {
            "bool" => Some("boolean"),
            "cstring" => Some("string"),
            "luaIndex" => Some("number"),
            "nil" | "function" => Some(name),
            _ => None,
        };
        if let Some(primitive) = primitive {
            return Ok(primitive.to_owned());
        }
        qualified_identifier(name).map_err(|_| RenderError::UnsupportedType)?;
        if self.enum_names.contains(name) {
            Ok(format!("Enum.{name}"))
        } else {
            Ok(name.to_owned())
        }
    }

    /// Ketho `GetSystem` plus its analysis-only `---@meta _` file header.
    pub fn render(&self, system: &System) -> Result<String, RenderError> {
        Ok(self.render_mapped(system)?.text)
    }

    pub fn render_mapped(&self, system: &System) -> Result<RenderedSystem, RenderError> {
        let mut declarations = Vec::new();
        validate_owner(&system.owner)?;
        if system.functions.len().saturating_add(system.tables.len()) > MAX_ITEMS {
            return Err(RenderError::InputLimit);
        }
        let mut output = Output {
            bytes: String::new(),
            limit: self.max_output_bytes,
        };
        output.push("---@meta _\n")?;
        let mut separated = false;
        let mut function_names = BTreeSet::new();
        if !system.functions.is_empty()
            && let Owner::Namespace(name) = &system.owner
        {
            output.push(name)?;
            output.push(" = {}")?;
            separated = true;
        }
        for (index, function) in system.functions.iter().enumerate() {
            identifier(&function.name)?;
            if !function_names.insert(&function.name) {
                return Err(RenderError::DuplicateName);
            }
            output.separate(&mut separated)?;
            let start = output.bytes.len();
            self.function(&mut output, &system.owner, function)?;
            declarations.push(RenderedDeclaration {
                table: false,
                index,
                start,
                end: output.bytes.len(),
            });
        }
        let mut table_names = BTreeSet::new();
        for (index, table) in system.tables.iter().enumerate() {
            let name = match table {
                Table::Structure { name, .. } | Table::Callback { name, .. } => name,
            };
            identifier(name)?;
            if !table_names.insert(name) {
                return Err(RenderError::DuplicateName);
            }
            output.separate(&mut separated)?;
            let start = output.bytes.len();
            match table {
                Table::Structure { name, fields } => {
                    validate_fields(fields, false)?;
                    output.push("---@class ")?;
                    output.push(name)?;
                    for field in fields {
                        output.push("\n")?;
                        self.field(&mut output, Position::Field, field)?;
                    }
                }
                Table::Callback { name, arguments } => {
                    validate_fields(arguments, false)?;
                    output.push("---@alias ")?;
                    output.push(name)?;
                    output.push(" FunctionContainer|fun(")?;
                    for (index, argument) in arguments.iter().enumerate() {
                        // This is precisely the donor's callback branch. Arrays are
                        // unsupported here rather than quietly dropping InnerType.
                        if argument.inner_type.is_some() {
                            return Err(RenderError::UnsupportedType);
                        }
                        if index != 0 {
                            output.push(", ")?;
                        }
                        output.push(&argument.name)?;
                        if argument.nilable || argument.default_text.is_some() {
                            output.push("?")?;
                        }
                        output.push(": ")?;
                        output.push(&self.lower_type(&argument.type_name)?)?;
                        if let Some(default) = &argument.default_text {
                            safe_text(default)?;
                        }
                    }
                    output.push(")")?;
                }
            }
            declarations.push(RenderedDeclaration {
                table: true,
                index,
                start,
                end: output.bytes.len(),
            });
        }
        Ok(RenderedSystem {
            text: output.bytes,
            declarations,
        })
    }

    fn function(
        &self,
        out: &mut Output,
        owner: &Owner,
        function: &Function,
    ) -> Result<(), RenderError> {
        validate_fields(&function.arguments, true)?;
        validate_fields(&function.returns, true)?;
        if let Some(documentation) = &function.documentation {
            if documentation.len() > MAX_ITEMS {
                return Err(RenderError::InputLimit);
            }
            out.push("---")?;
            for (index, text) in documentation.iter().enumerate() {
                safe_text(text)?;
                // Do not allow source prose to introduce a LuaCATS directive.
                if text.trim_start().starts_with('@') {
                    return Err(RenderError::UnsafeDocumentation);
                }
                if index != 0 {
                    out.push("; ")?;
                }
                out.push(text)?;
            }
            out.push("\n---\n")?;
        }
        out.push("---[Documentation](https://warcraft.wiki.gg/wiki/API_")?;
        let (owner_name, method) = owner_name(owner);
        if let Some(name) = owner_name {
            out.push(name)?;
            out.push(if method { "_" } else { "." })?;
        }
        out.push(&function.name)?;
        out.push(")\n")?;
        for field in &function.arguments {
            self.field(out, Position::Param, field)?;
            out.push("\n")?;
        }
        for field in &function.returns {
            self.field(out, Position::Return, field)?;
            out.push("\n")?;
        }
        out.push("function ")?;
        if let Some(name) = owner_name {
            out.push(name)?;
            out.push(if method { ":" } else { "." })?;
        }
        out.push(&function.name)?;
        out.push("(")?;
        for (index, field) in function.arguments.iter().enumerate() {
            if index != 0 {
                out.push(", ")?;
            }
            out.push(if field.variadic { "..." } else { &field.name })?;
        }
        out.push(") end")
    }

    fn field(
        &self,
        out: &mut Output,
        position: Position,
        field: &Field,
    ) -> Result<(), RenderError> {
        let mut type_name =
            self.lower_type(field.inner_type.as_deref().unwrap_or(&field.type_name))?;
        if field.inner_type.is_some() {
            type_name.push_str("[]");
        }
        let optional = field.nilable || field.default_text.is_some();
        let name = if field.variadic { "..." } else { &field.name };
        match position {
            Position::Param => {
                out.push("---@param ")?;
                out.push(name)?;
                if optional {
                    out.push("?")?;
                }
                out.push(" ")?;
                out.push(&type_name)?;
            }
            Position::Field => {
                out.push("---@field ")?;
                out.push(name)?;
                out.push(" ")?;
                out.push(&type_name)?;
                if optional {
                    out.push("?")?;
                }
            }
            Position::Return => {
                out.push("---@return ")?;
                out.push(&type_name)?;
                if optional {
                    out.push("?")?;
                }
                out.push(" ")?;
                out.push(name)?;
            }
        }
        if field.variadic {
            out.push(" ")?;
            out.push(&field.name)?;
        }
        if let Some(default) = &field.default_text {
            safe_text(default)?;
            out.push(" Default = ")?;
            out.push(default)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Position {
    Field,
    Param,
    Return,
}

pub(crate) struct Output {
    pub(crate) bytes: String,
    pub(crate) limit: usize,
}

impl Output {
    pub(crate) fn push(&mut self, value: &str) -> Result<(), RenderError> {
        if self.bytes.len().saturating_add(value.len()) > self.limit {
            return Err(RenderError::OutputLimit);
        }
        self.bytes.push_str(value);
        Ok(())
    }

    fn separate(&mut self, separated: &mut bool) -> Result<(), RenderError> {
        if *separated {
            self.push("\n\n")?;
        }
        *separated = true;
        Ok(())
    }
}

fn owner_name(owner: &Owner) -> (Option<&str>, bool) {
    match owner {
        Owner::Global => (None, false),
        Owner::Namespace(name) => (Some(name), false),
        Owner::ScriptObject {
            system_name,
            annotation_name,
        } => (
            Some(annotation_name.as_deref().unwrap_or(system_name)),
            true,
        ),
    }
}

fn validate_owner(owner: &Owner) -> Result<(), RenderError> {
    match owner {
        Owner::Global => Ok(()),
        // Blizzard namespace declarations are one identifier, not assignments
        // into arbitrary caller-controlled expressions or undeclared parents.
        Owner::Namespace(name) => identifier(name),
        Owner::ScriptObject {
            system_name,
            annotation_name,
        } => {
            identifier(system_name)?;
            if let Some(name) = annotation_name {
                identifier(name)?;
            }
            Ok(())
        }
    }
}

fn validate_fields(fields: &[Field], allow_variadic: bool) -> Result<(), RenderError> {
    if fields.len() > MAX_ITEMS {
        return Err(RenderError::InputLimit);
    }
    let mut names = BTreeSet::new();
    for (index, field) in fields.iter().enumerate() {
        identifier(&field.name)?;
        if !names.insert(&field.name) {
            return Err(RenderError::DuplicateName);
        }
        if field.variadic && (!allow_variadic || index + 1 != fields.len()) {
            return Err(RenderError::InvalidVariadic);
        }
    }
    Ok(())
}

fn qualified_identifier(value: &str) -> Result<(), RenderError> {
    if value.len() > MAX_NAME_BYTES {
        return Err(RenderError::InputLimit);
    }
    for part in value.split('.') {
        identifier(part)?;
    }
    Ok(())
}

pub(crate) fn identifier(value: &str) -> Result<(), RenderError> {
    if value.is_empty() || value.len() > MAX_NAME_BYTES {
        return Err(RenderError::InvalidIdentifier);
    }
    let mut chars = value.bytes();
    if !chars
        .next()
        .is_some_and(|b| b == b'_' || b.is_ascii_alphabetic())
        || !chars.all(|b| b == b'_' || b.is_ascii_alphanumeric())
        || is_keyword(value)
    {
        return Err(RenderError::InvalidIdentifier);
    }
    Ok(())
}

pub(crate) fn is_keyword(value: &str) -> bool {
    matches!(
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
}

pub(crate) fn safe_text(value: &str) -> Result<(), RenderError> {
    if value.len() > MAX_TEXT_BYTES {
        return Err(RenderError::InputLimit);
    }
    if value
        .chars()
        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(RenderError::UnsafeDocumentation);
    }
    Ok(())
}
