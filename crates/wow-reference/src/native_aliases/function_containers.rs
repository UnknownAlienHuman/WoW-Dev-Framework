//! Static Ketho FunctionContainer declarations parsed without executing Lua.
use super::{Result, collect_terms, error, location};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{
    LuaAstNode, LuaCommentOwner, LuaDocTag, LuaExpr, LuaIndexKey, LuaLanguageLevel, LuaParser,
    LuaStat, LuaVarExpr, ParserConfig,
};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

/// One explicitly declared static class and its empty colon-method stubs.
#[derive(Clone, Debug, Serialize)]
pub struct FunctionContainerFact {
    pub name: String,
    pub methods: Vec<FunctionContainerMethod>,
    pub header_span: Span,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionContainerMethod {
    pub name: String,
    pub returns: Vec<FunctionContainerReturn>,
    pub span: Span,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionContainerReturn {
    pub terms: Vec<String>,
    pub span: Span,
}

pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<FunctionContainerFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let chunk = tree.get_chunk_node();
    let mut class_tags = 0usize;
    let mut return_tags = 0usize;
    for tag in chunk.syntax().descendants().filter_map(LuaDocTag::cast) {
        match tag {
            LuaDocTag::Meta(meta)
                if meta
                    .get_name_token()
                    .is_none_or(|token| token.get_name_text() == "_") => {}
            LuaDocTag::Class(_) => class_tags += 1,
            LuaDocTag::Return(_) => return_tags += 1,
            _ => return Err(error(NativeErrorCode::UnsupportedStatement)),
        }
    }
    if class_tags != 1 {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    let block = chunk
        .get_block()
        .ok_or_else(|| error(NativeErrorCode::Syntax))?;
    let mut stats = block.get_stats();
    let Some(LuaStat::LocalStat(local)) = stats.next() else {
        return Err(error(NativeErrorCode::UnsupportedStatement));
    };
    let comment = local
        .get_left_comment()
        .ok_or_else(|| error(NativeErrorCode::UnsupportedStatement))?;
    let mut classes = comment
        .syntax()
        .descendants()
        .filter_map(LuaDocTag::cast)
        .filter_map(|tag| match tag {
            LuaDocTag::Class(class) => Some(class),
            _ => None,
        });
    let class = classes
        .next()
        .ok_or_else(|| error(NativeErrorCode::UnsupportedStatement))?;
    if classes.next().is_some()
        || class.get_generic_decl().is_some()
        || class.get_supers().is_some()
        || class.get_type_flag().is_some()
        || local.get_attrib().is_some()
        || local.is_const()
    {
        return Err(error(NativeErrorCode::UnsupportedStatement));
    }
    let name = class
        .get_name_token()
        .ok_or_else(|| error(NativeErrorCode::Syntax))?
        .get_name_text()
        .to_owned();
    let mut names = local.get_local_name_list();
    let local_name = names
        .next()
        .and_then(|name| name.get_name_token())
        .ok_or_else(|| error(NativeErrorCode::Syntax))?;
    if names.next().is_some() || local_name.get_name_text() != name {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    let mut values = local.get_value_exprs();
    let Some(LuaExpr::TableExpr(table)) = values.next() else {
        return Err(error(NativeErrorCode::UnsupportedStatement));
    };
    if values.next().is_some() || !table.is_empty() || table.get_fields().next().is_some() {
        return Err(error(NativeErrorCode::UnsupportedStatement));
    }
    let header_span = Span {
        start: location(&comment).start,
        end: location(&local).end,
    };
    let mut methods = Vec::new();
    let mut consumed_returns = 0usize;
    for stat in stats {
        if cancelled.load(Ordering::Relaxed) {
            return Err(error(NativeErrorCode::Cancelled));
        }
        let LuaStat::FuncStat(function) = stat else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if function.is_global() {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let LuaVarExpr::IndexExpr(index) = function
            .get_func_name()
            .ok_or_else(|| error(NativeErrorCode::Syntax))?
        else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if !index
            .get_index_token()
            .is_some_and(|token| token.is_colon())
        {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let Some(LuaExpr::NameExpr(receiver)) = index.get_prefix_expr() else {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        };
        if receiver.get_name_text().as_deref() != Some(name.as_str()) {
            return Err(error(NativeErrorCode::InvalidRegistration));
        }
        let method_name = match index.get_index_key() {
            Some(LuaIndexKey::Name(method)) => method.get_name_text().to_owned(),
            _ => return Err(error(NativeErrorCode::UnsupportedStatement)),
        };
        let closure = function
            .get_closure()
            .ok_or_else(|| error(NativeErrorCode::Syntax))?;
        if closure.is_short_closure()
            || closure.has_do_block()
            || closure
                .get_params_list()
                .is_none_or(|parameters| parameters.get_params().next().is_some())
            || closure
                .get_block()
                .is_none_or(|body| body.get_stats().next().is_some())
        {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let function_span = location(&function);
        let mut method_span = function_span;
        let mut returns = Vec::new();
        if let Some(documentation) = function.get_left_comment() {
            method_span.start = location(&documentation).start;
            for tag in documentation
                .syntax()
                .descendants()
                .filter_map(LuaDocTag::cast)
            {
                let LuaDocTag::Return(return_tag) = tag else {
                    return Err(error(NativeErrorCode::UnsupportedStatement));
                };
                consumed_returns += 1;
                if returns.len() >= 16 {
                    return Err(error(NativeErrorCode::Limit));
                }
                let mut information = return_tag.get_info_list().into_iter();
                let Some((return_type, None)) = information.next() else {
                    return Err(error(NativeErrorCode::UnsupportedExpression));
                };
                if information.next().is_some() {
                    return Err(error(NativeErrorCode::UnsupportedExpression));
                }
                let mut terms = Vec::new();
                if !collect_terms(return_type, &mut terms, 0) || terms.is_empty() {
                    return Err(error(NativeErrorCode::UnsupportedExpression));
                }
                returns.push(FunctionContainerReturn {
                    terms,
                    span: location(&return_tag),
                });
            }
        }
        methods.push(FunctionContainerMethod {
            name: method_name,
            returns,
            span: method_span,
        });
        if methods.len() > super::MAX_ALIASES {
            return Err(error(NativeErrorCode::Limit));
        }
    }
    if methods.is_empty() || consumed_returns != return_tags {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    let span = Span {
        start: header_span.start,
        end: methods
            .last()
            .map_or(header_span.end, |method| method.span.end),
    };
    Ok(vec![FunctionContainerFact {
        name,
        methods,
        header_span,
        span,
    }])
}
