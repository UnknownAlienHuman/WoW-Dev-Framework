//! Static Ketho global colors parsed without executing `CreateColor` calls.
use super::{Result, error, location};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaDocTag, LuaExpr, LuaLanguageLevel, LuaLiteralToken, LuaParser,
    LuaStat, LuaVarExpr, ParserConfig,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

/// One exact global assignment. Components retain source number lexemes; they
/// are evidence only and are never evaluated or copied into executable output.
#[derive(Clone, Debug, Serialize)]
pub struct GlobalColorFact {
    pub name: String,
    pub components: [String; 4],
    pub span: Span,
}

pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<GlobalColorFact>> {
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
            .ok_or_else(|| error(NativeErrorCode::Syntax))?;
        let Some(LuaExpr::CallExpr(call)) = values.into_iter().next() else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        let Some(LuaExpr::NameExpr(callee)) = call.get_prefix_expr() else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if callee.get_name_text().as_deref() != Some("CreateColor")
            || call.is_colon_call()
            || call.has_safe_navigation()
            || call.get_call_generic_type_list().is_some()
        {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let arguments = call
            .get_args_list()
            .ok_or_else(|| error(NativeErrorCode::Syntax))?;
        if arguments.is_single_arg_no_parens() {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let mut components = Vec::new();
        for argument in arguments.get_args() {
            let LuaExpr::LiteralExpr(literal) = argument else {
                return Err(error(NativeErrorCode::UnsupportedExpression));
            };
            let Some(LuaLiteralToken::Number(number)) = literal.get_literal() else {
                return Err(error(NativeErrorCode::UnsupportedExpression));
            };
            if number.is_complex() || number.syntax().text().len() > 64 {
                return Err(error(NativeErrorCode::UnsupportedExpression));
            }
            components.push(number.syntax().text().to_owned());
        }
        let components: [String; 4] = components
            .try_into()
            .map_err(|_| error(NativeErrorCode::InvalidRegistration))?;
        result.push(GlobalColorFact {
            name,
            components,
            span: location(&assign),
        });
        if result.len() > super::MAX_ALIASES {
            return Err(error(NativeErrorCode::Limit));
        }
    }
    if result.is_empty() {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    Ok(result)
}
