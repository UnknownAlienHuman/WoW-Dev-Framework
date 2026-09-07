//! Explicit open string aliases with bounded literal completion suggestions.
use super::{catalog, location};
use emmylua_parser::{
    LuaAstNode, LuaDocTag, LuaDocTagAlias, LuaDocType, LuaLanguageLevel, LuaParser,
    LuaTypeBinaryOperator, ParserConfig,
};
use std::collections::BTreeSet;

pub(super) fn values(alias: &LuaDocTagAlias, input: &str) -> Option<Vec<String>> {
    let ty = alias.get_type()?;
    if !input.contains('\n') {
        let mut values = Vec::new();
        let mut base = false;
        if !inline(ty, &mut base, &mut values, 0) || !base || values.is_empty() {
            return None;
        }
        let unique = values.iter().collect::<BTreeSet<_>>();
        return (unique.len() == values.len()).then_some(values);
    }
    let LuaDocType::Name(base) = ty else {
        // Do not drop continuation lines from an already populated inline union.
        return None;
    };
    if base.get_generic_param().is_some() || base.get_name_text()?.as_str() != "string" {
        return None;
    }
    let span = location(&base);
    let line_end = input.find('\n')?;
    let suffix = input.get(span.end..line_end)?.trim();
    if input.get(span.start..span.end)? != "string"
        || !(suffix.is_empty() || suffix.starts_with('#'))
    {
        return None;
    }

    // Emmy does not attach continuation types after a named base. Only the
    // AST-confirmed base token is blanked in a bounded parser view. Raw source,
    // syntax status and byte offsets remain unchanged in the resource receipt.
    let mut view = input.to_owned();
    view.replace_range(span.start..span.end, "      ");
    let tree = LuaParser::parse(&view, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return None;
    }
    let chunk = tree.get_chunk_node();
    let mut tags = chunk.syntax().descendants().filter_map(LuaDocTag::cast);
    let LuaDocTag::Alias(projected) = tags.next()? else {
        return None;
    };
    if tags.next().is_some()
        || projected.get_name_token()?.get_name_text() != alias.get_name_token()?.get_name_text()
        || projected.get_generic_decl_list().is_some()
        || projected.get_type_flag().is_some()
    {
        return None;
    }
    let ty @ LuaDocType::MultiLineUnion(_) = projected.get_type()? else {
        return None;
    };
    catalog::string_values(ty)
}

fn inline(ty: LuaDocType, base: &mut bool, values: &mut Vec<String>, depth: usize) -> bool {
    if depth >= 16 {
        return false;
    }
    match ty {
        LuaDocType::Binary(binary)
            if binary
                .get_op_token()
                .is_some_and(|op| op.get_op() == LuaTypeBinaryOperator::Union) =>
        {
            let Some((left, right)) = binary.get_types() else {
                return false;
            };
            inline(left, base, values, depth + 1) && inline(right, base, values, depth + 1)
        }
        LuaDocType::Name(name) if !*base && values.is_empty() => {
            *base = name.get_generic_param().is_none()
                && name.get_name_text().is_some_and(|name| name == "string");
            *base
        }
        ty @ LuaDocType::Literal(_) if *base => {
            let Some(literals) = catalog::string_values(ty) else {
                return false;
            };
            if values.len() + literals.len() > 256 {
                return false;
            }
            values.extend(literals);
            true
        }
        _ => false,
    }
}
