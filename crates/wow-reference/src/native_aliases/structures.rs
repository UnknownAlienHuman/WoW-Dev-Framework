//! Comment-only Ketho classes. Emmy owns names and type syntax, never Lua execution.
use super::{MAX_ALIASES, Result, collect_terms, error};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{LuaDocFieldKey, LuaDocTagClass, LuaDocTagField, LuaDocType};
use serde::Serialize;

/// Explicit external structure, not a declaration imported into ReferenceView.
#[derive(Clone, Debug, Serialize)]
pub struct StructureFact {
    pub name: String,
    pub fields: Vec<StructureField>,
    pub header_supported: bool,
    pub syntax_error: bool,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct StructureField {
    /// Unsupported indexer/string/number keys remain in the raw resource.
    pub name: Option<String>,
    pub field_type: Option<StructureFieldType>,
    pub span: Span,
}

/// Named/primitive union, optionally an array and/or nullable as a whole.
/// Nullable elements, nested arrays, functions, generics and indexers are not
/// coerced into this profile. Unsupported fields prevent their class emission.
#[derive(Clone, Debug, Serialize)]
pub struct StructureFieldType {
    pub terms: Vec<String>,
    pub array: bool,
    pub nullable: bool,
}

pub(super) fn class(
    tag: &LuaDocTagClass,
    span: Span,
    syntax_error: bool,
) -> Result<StructureFact> {
    let name = tag
        .get_name_token()
        .ok_or_else(|| error(NativeErrorCode::Syntax))?
        .get_name_text()
        .to_owned();
    Ok(StructureFact {
        name,
        fields: Vec::new(),
        header_supported: tag.get_generic_decl().is_none()
            && tag.get_supers().is_none()
            && tag.get_type_flag().is_none(),
        syntax_error,
        span,
    })
}

pub(super) fn field(
    target: &mut StructureFact,
    tag: &LuaDocTagField,
    span: Span,
    syntax_error: bool,
    count: &mut usize,
) -> Result<()> {
    *count += 1;
    if *count > MAX_ALIASES {
        return Err(error(NativeErrorCode::Limit));
    }
    let name = match tag.get_field_key() {
        Some(LuaDocFieldKey::Name(name)) => Some(name.get_name_text().to_owned()),
        _ => None,
    };
    let field_type = if !syntax_error
        && !tag.is_nullable()
        && tag.get_visibility_token().is_none()
        && tag.get_type_flag().is_none()
    {
        tag.get_type().and_then(field_type)
    } else {
        None
    };
    target.fields.push(StructureField {
        name,
        field_type,
        span,
    });
    target.span.end = span.end;
    target.syntax_error |= syntax_error;
    Ok(())
}

fn field_type(ty: LuaDocType) -> Option<StructureFieldType> {
    let (ty, nullable) = match ty {
        LuaDocType::Nullable(ty) => (ty.get_type()?, true),
        ty => (ty, false),
    };
    let (ty, array) = match ty {
        LuaDocType::Array(ty) => (ty.get_type()?, true),
        ty => (ty, false),
    };
    let mut terms = Vec::new();
    if !collect_terms(ty, &mut terms, 0) || terms.is_empty() {
        return None;
    }
    Some(StructureFieldType {
        terms,
        array,
        nullable,
    })
}
