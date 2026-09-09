//! Namespace-only Ketho resources. Emmy parses exact empty-table assignments;
//! no statement is executed and no general Lua expression is admitted.
use super::{Result, error, location};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaDocTag, LuaExpr, LuaLanguageLevel, LuaParser, LuaStat, LuaVarExpr,
    ParserConfig,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug, Serialize)]
pub struct NamespaceFact {
    pub name: String,
    pub span: Span,
}

pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<NamespaceFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let chunk = tree.get_chunk_node();
    for tag in chunk.syntax().descendants().filter_map(LuaDocTag::cast) {
        match tag {
            LuaDocTag::Meta(meta)
                if meta
                    .get_name_token()
                    .is_none_or(|token| token.get_name_text() == "_") => {}
            _ => return Err(error(NativeErrorCode::UnsupportedStatement)),
        }
    }
    let block = chunk
        .get_block()
        .ok_or_else(|| error(NativeErrorCode::Syntax))?;
    let mut result = Vec::new();
    for stat in block.get_stats() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(error(NativeErrorCode::Cancelled));
        }
        let LuaStat::AssignStat(assign) = stat else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if assign
            .get_assign_op()
            .is_none_or(|operator| operator.get_text() != "=")
        {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let (variables, values) = assign.get_var_and_expr_list();
        let mut variables = variables.into_iter();
        let Some(LuaVarExpr::NameExpr(variable)) = variables.next() else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if variables.next().is_some() || values.len() != 1 {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let name = variable
            .get_name_text()
            .filter(|name| valid_name(name))
            .ok_or_else(|| error(NativeErrorCode::InvalidRegistration))?;
        let Some(LuaExpr::TableExpr(table)) = values.into_iter().next() else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if !table.is_empty() || table.get_fields().next().is_some() {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        result.push(NamespaceFact {
            name,
            span: location(&assign),
        });
    }
    if result.is_empty() {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    Ok(result)
}

fn valid_name(name: &str) -> bool {
    name.len() <= 1024
        && name.starts_with("C_")
        && name
            .bytes()
            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}
