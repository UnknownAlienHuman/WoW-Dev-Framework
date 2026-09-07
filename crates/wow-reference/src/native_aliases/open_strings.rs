//! Ketho open string aliases: explicit base plus literal completion suggestions.
use super::{catalog, location};
use emmylua_parser::{
    LuaAstNode, LuaDocTag, LuaDocTagAlias, LuaDocType, LuaLanguageLevel, LuaParser, ParserConfig,
};

pub(super) fn values(alias: &LuaDocTagAlias, input: &str) -> Option<Vec<String>> {
    let LuaDocType::Name(base) = alias.get_type()? else {
        return None;
    };
    if base.get_generic_param().is_some() || base.get_name_text()?.as_str() != "string" {
        return None;
    }
    let span = location(&base);
    let line_end = input.find('\n')?;
    if span.end > line_end
        || input.get(span.start..span.end)? != "string"
        || !input.get(span.end..line_end)?.trim().is_empty()
    {
        return None;
    }

    // Emmy parses a leading named type but does not attach its continuation
    // comments as a MultiLineUnion. Parse the already bounded group a second
    // time with only the AST-confirmed base token blanked. Byte offsets stay
    // unchanged. Neither source bytes nor the original syntax status is changed;
    // invalid literal hints never fall back to a bare string alias.
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
