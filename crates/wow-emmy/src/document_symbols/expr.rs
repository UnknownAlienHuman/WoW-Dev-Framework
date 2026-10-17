// Adapted from EmmyLuaLs/emmylua-analyzer-rust aaaca68425d9362876228649b0b8d92f07654daa.
// Copyright(c)2024-2026CppCXY. MIT; see ../../DOCUMENT_SYMBOLS_DONOR_LICENSE.
use super::DocumentSymbolKind as SymbolKind;
use emmylua_code_analysis::LuaDeclId;
use emmylua_parser::{
    LuaAstNode, LuaClosureExpr, LuaIndexKey, LuaSyntaxId, LuaSyntaxKind, LuaTableExpr,
};

use super::builder::{DocumentSymbolBuilder, LuaSymbol};

pub(super) fn build_closure_expr_symbol(
    builder: &mut DocumentSymbolBuilder,
    closure: LuaClosureExpr,
    parent_id: LuaSyntaxId,
) -> Option<LuaSyntaxId> {
    let parent_kind = closure.syntax().parent().map(|parent| parent.kind().into());
    let convert_parent_to_function = matches!(
        parent_kind,
        Some(LuaSyntaxKind::TableFieldAssign | LuaSyntaxKind::TableFieldValue)
    );
    let needs_own_symbol = match parent_kind {
        Some(LuaSyntaxKind::LocalFuncStat | LuaSyntaxKind::FuncStat) => false,
        Some(_) if convert_parent_to_function => false,
        _ => true,
    };

    let param_list = closure.get_params_list()?;
    let limits = builder.limits();
    let mut params = Vec::new();
    for param in param_list.get_params() {
        builder.checkpoint()?;
        if params.len() >= limits.max_symbols {
            return builder.limit();
        }
        params.push(param);
    }
    let mut detail_text = String::from("(");
    let mut named = 0usize;
    for param in &params {
        builder.checkpoint()?;
        let name = if param.is_dots() {
            "...".to_string()
        } else {
            param
                .get_name_token()
                .map(|token| token.get_name_text().to_string())
                .unwrap_or_default()
        };
        if name.is_empty() {
            continue;
        }
        let separator = if named == 0 { 0 } else { 2 };
        if name.len() > limits.max_name_bytes
            || detail_text
                .len()
                .checked_add(name.len() + separator + 1)
                .is_none_or(|n| n > limits.max_detail_bytes)
        {
            return builder.limit();
        }
        if named != 0 {
            detail_text.push_str(", ");
        }
        detail_text.push_str(&name);
        named += 1;
    }
    detail_text.push(')');
    let detail = Some(detail_text);
    let mut effective_parent = parent_id;

    if needs_own_symbol {
        let symbol = LuaSymbol::new(
            "closure".to_string(),
            detail.clone(),
            SymbolKind::MODULE,
            closure.get_range(),
        );

        effective_parent =
            builder.add_node_symbol(closure.syntax().clone(), symbol, Some(parent_id));
    } else if convert_parent_to_function {
        let detail_clone = detail.clone();
        builder.with_symbol_mut(&parent_id, |symbol| {
            symbol.set_kind(SymbolKind::FUNCTION);
            symbol.set_detail(detail_clone);
        })?;
    }

    let file_id = builder.get_file_id();
    for param in params {
        builder.checkpoint()?;
        let decl_id = LuaDeclId::new(file_id, param.get_position());
        let decl = builder.get_decl(&decl_id)?;
        let typ = builder.get_type(decl_id.into());
        let desc = builder.get_symbol_kind_and_detail(Some(&typ));
        let symbol = LuaSymbol::new(
            decl.get_name().to_string(),
            desc.1,
            desc.0,
            decl.get_range(),
        );

        builder.add_node_symbol(param.syntax().clone(), symbol, Some(effective_parent));
    }

    Some(effective_parent)
}

pub(super) fn build_table_symbol(
    builder: &mut DocumentSymbolBuilder,
    table: LuaTableExpr,
    parent_id: LuaSyntaxId,
    inline_to_parent: bool,
) -> Option<LuaSyntaxId> {
    let table_id = if inline_to_parent {
        parent_id
    } else {
        let symbol = LuaSymbol::new(
            "table".to_string(),
            None,
            SymbolKind::STRUCT,
            table.get_range(),
        );

        builder.add_node_symbol(table.syntax().clone(), symbol, Some(parent_id))
    };

    if table.is_object() {
        for field in table.get_fields() {
            builder.checkpoint()?;
            let key = field.get_field_key()?;
            let str_key = match key {
                LuaIndexKey::String(key) => key.get_value(),
                LuaIndexKey::Name(key) => key.get_name_text().to_string(),
                LuaIndexKey::Integer(i) => i.get_number_value().to_string(),
                _ => continue,
            };

            let symbol = LuaSymbol::new(str_key, None, SymbolKind::FIELD, field.get_range());

            builder.add_node_symbol(field.syntax().clone(), symbol, Some(table_id));
        }
    }

    Some(table_id)
}
