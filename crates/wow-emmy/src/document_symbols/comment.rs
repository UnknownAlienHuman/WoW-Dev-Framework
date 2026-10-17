// Adapted from EmmyLuaLs/emmylua-analyzer-rust aaaca68425d9362876228649b0b8d92f07654daa.
// Copyright(c)2024-2026CppCXY. MIT; see ../../DOCUMENT_SYMBOLS_DONOR_LICENSE.
use super::DocumentSymbolKind as SymbolKind;
use emmylua_parser::{LuaAstNode, LuaComment, LuaSyntaxId, LuaTokenKind};
use rowan::NodeOrToken;

use super::builder::{DocumentSymbolBuilder, LuaSymbol};

pub(super) fn build_doc_region_symbol(
    builder: &mut DocumentSymbolBuilder,
    comment: LuaComment,
    parent_id: LuaSyntaxId,
) -> Option<LuaSyntaxId> {
    let mut region_token = None;
    for child in comment.syntax().children_with_tokens() {
        builder.checkpoint()?;
        if let NodeOrToken::Token(token) = child
            && token.kind() == LuaTokenKind::TkDocRegion.into()
        {
            region_token = Some(token);
            break;
        }
    }

    let region_token = region_token?;

    let description = comment
        .get_description()
        .map(|desc| desc.get_description_text())
        .map(|text| {
            text.lines()
                .next()
                .map(|line| line.trim())
                .unwrap_or_default()
                .to_string()
        })
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| "region".to_string());

    let range = comment.get_range();
    let selection_range = region_token.text_range();
    let symbol = LuaSymbol::with_selection_range(
        description,
        None,
        SymbolKind::NAMESPACE,
        range,
        selection_range,
    );

    let symbol_id = builder.add_node_symbol(comment.syntax().clone(), symbol, Some(parent_id));

    Some(symbol_id)
}
