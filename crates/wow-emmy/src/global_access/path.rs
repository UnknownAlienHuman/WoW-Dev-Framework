//! Literal path projection from Emmy's AST, never from reparsed source text.
use super::{AccessBudget, GlobalAccessKind, MAX_PATH_DEPTH, budget, invalid};
use crate::references::EmmyMemberCallResult;
use emmylua_parser::{
    LuaAssignStat, LuaAstNode, LuaAstToken, LuaExpr, LuaFuncStat, LuaIndexExpr, LuaIndexKey,
    LuaLiteralToken, LuaNameExpr, LuaNumberToken, LuaParenExpr, LuaSyntaxKind, LuaSyntaxNode,
};
use serde::Serialize;

/// Distinct Lua key domains. In particular, `[1]`, `["1"]`, and `[true]` do
/// not identify the same slot. Integers use a cross-Lua-version exact range.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum GlobalAccessKey {
    String(String),
    Integer(#[serde(serialize_with = "serialize_integer")] i64),
    Boolean(bool),
}
// Core semantic JSON forbids negative JSON numbers. Preserve the signed key
// domain as a canonical decimal string rather than widening core contracts.
fn serialize_integer<S: serde::Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}
const MAX_EXACT_INTEGER: i64 = 9_007_199_254_740_991;
impl GlobalAccessKey {
    pub fn text_bytes(&self) -> usize {
        match self {
            Self::String(value) => value.len(),
            Self::Integer(_) => 20,
            Self::Boolean(_) => 5,
        }
    }
    pub(super) fn valid(&self) -> bool {
        match self {
            Self::String(value) => value.len() <= 1024,
            Self::Integer(value) => (-MAX_EXACT_INTEGER..=MAX_EXACT_INTEGER).contains(value),
            Self::Boolean(_) => true,
        }
    }
}

pub(super) struct SourcePath {
    pub root: LuaNameExpr,
    pub keys: Vec<GlobalAccessKey>,
    pub complete: bool,
}

/// Accept only a name, literal indices and transparent parentheses. A dynamic
/// index retains its root but never claims that the known prefix is the slot.
pub(super) fn from_expr(
    mut expr: LuaExpr,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<Option<SourcePath>> {
    let mut reversed = Vec::new();
    for _ in 0..MAX_PATH_DEPTH {
        limits.step()?;
        match expr {
            LuaExpr::NameExpr(root) => {
                reversed.reverse();
                let complete = reversed.iter().all(Option::is_some);
                let keys = reversed
                    .into_iter()
                    .take_while(Option::is_some)
                    .flatten()
                    .collect();
                return Ok(Some(SourcePath {
                    root,
                    keys,
                    complete,
                }));
            }
            LuaExpr::IndexExpr(index) => {
                reversed.push(index_key(&index, limits)?);
                expr = index.get_prefix_expr().ok_or_else(invalid)?;
            }
            LuaExpr::ParenExpr(paren) => expr = paren.get_expr().ok_or_else(invalid)?,
            _ => return Ok(None),
        }
    }
    Err(budget())
}

pub(super) fn maximal(
    name: &LuaNameExpr,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<(LuaSyntaxNode, SourcePath)> {
    let mut current = name.syntax().clone();
    for _ in 0..MAX_PATH_DEPTH {
        limits.step()?;
        let Some(parent) = current.parent() else {
            return finish(current, limits);
        };
        let extends = LuaIndexExpr::cast(parent.clone()).is_some_and(|index| {
            index
                .get_prefix_expr()
                .is_some_and(|prefix| prefix.syntax() == &current)
        }) || LuaParenExpr::cast(parent.clone()).is_some_and(|paren| {
            paren
                .get_expr()
                .is_some_and(|inner| inner.syntax() == &current)
        });
        if !extends {
            return finish(current, limits);
        }
        current = parent;
    }
    Err(budget())
}
fn finish(
    current: LuaSyntaxNode,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<(LuaSyntaxNode, SourcePath)> {
    let expr = LuaExpr::cast(current.clone()).ok_or_else(invalid)?;
    let path = from_expr(expr, limits)?.ok_or_else(invalid)?;
    Ok((current, path))
}

fn index_key(
    index: &LuaIndexExpr,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<Option<GlobalAccessKey>> {
    if index.syntax().kind() == LuaSyntaxKind::SafeIndexExpr.into() {
        return Ok(None);
    }
    let key = match index.get_index_key() {
        Some(LuaIndexKey::Name(token)) => {
            Some(GlobalAccessKey::String(token.get_name_text().into()))
        }
        Some(LuaIndexKey::String(token)) => Some(GlobalAccessKey::String(token.get_value())),
        Some(LuaIndexKey::Integer(token)) => integer(&token, false),
        Some(LuaIndexKey::Expr(expr)) => literal(expr, limits)?,
        _ => None,
    };
    if let Some(key) = &key {
        if !key.valid() {
            return Err(budget());
        }
        limits.text(key.text_bytes() + 32)?;
    }
    Ok(key)
}
fn literal(
    mut expr: LuaExpr,
    limits: &mut AccessBudget,
) -> EmmyMemberCallResult<Option<GlobalAccessKey>> {
    let mut negative = false;
    for _ in 0..MAX_PATH_DEPTH {
        limits.step()?;
        match expr {
            LuaExpr::ParenExpr(paren) => expr = paren.get_expr().ok_or_else(invalid)?,
            LuaExpr::UnaryExpr(unary)
                if !negative
                    && unary
                        .get_op_token()
                        .is_some_and(|op| op.syntax().text() == "-") =>
            {
                negative = true;
                expr = unary.get_expr().ok_or_else(invalid)?;
            }
            LuaExpr::LiteralExpr(value) => {
                return Ok(match value.get_literal() {
                    Some(LuaLiteralToken::Number(number)) => integer(&number, negative),
                    Some(LuaLiteralToken::String(string)) if !negative => {
                        Some(GlobalAccessKey::String(string.get_value()))
                    }
                    Some(LuaLiteralToken::Bool(value)) if !negative => {
                        Some(GlobalAccessKey::Boolean(value.is_true()))
                    }
                    _ => None,
                });
            }
            _ => return Ok(None),
        }
    }
    Err(budget())
}
/// Upstream get_number_value defaults malformed tokens to zero and can reinterpret
/// overflowing hex as signed. Admit only bounded decimal/hex integer spellings;
/// never use that fallback to invent a key. Fractions/exponents stay unsupported.
fn integer(token: &LuaNumberToken, negative: bool) -> Option<GlobalAccessKey> {
    if !token.is_int() {
        return None;
    }
    let text = token.syntax().text();
    if text.len() > 32 {
        return None;
    }
    let value = if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        if hex.is_empty() || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        i64::from_str_radix(hex, 16).ok()?
    } else {
        if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        text.parse::<i64>().ok()?
    };
    if value > MAX_EXACT_INTEGER {
        return None;
    }
    Some(GlobalAccessKey::Integer(if negative {
        -value
    } else {
        value
    }))
}

pub(super) fn access_kind(current: &LuaSyntaxNode) -> GlobalAccessKind {
    if let Some(parent) = current.parent() {
        if let Some(assignment) = LuaAssignStat::cast(parent.clone()) {
            let (vars, _) = assignment.get_var_and_expr_list();
            if vars.iter().any(|var| var.syntax() == current) {
                return if assignment
                    .get_assign_op()
                    .is_some_and(|op| op.syntax().text() == "=")
                {
                    GlobalAccessKind::Write
                } else {
                    GlobalAccessKind::UnsupportedAssignment
                };
            }
        } else if let Some(statement) = LuaFuncStat::cast(parent)
            && statement
                .get_func_name()
                .is_some_and(|var| var.syntax() == current)
        {
            return GlobalAccessKind::Write;
        }
    }
    GlobalAccessKind::Read
}
