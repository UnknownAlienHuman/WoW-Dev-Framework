//! Bounded named-declaration syntax cache over already held native closures.
//! This cache is excluded from legacy reports and cannot be deserialized.
use super::{SourceFunctionDeclarationForm, SourceFunctionKind, invalid};
use crate::{
    LuaWorkspaceFile,
    bindings::checkpoint,
    references::{EmmyMemberCallResult, ast_span},
};
use emmylua_parser::{
    LuaAst, LuaAstNode, LuaAstToken, LuaClosureExpr, LuaExpr, LuaIndexKey, LuaNameToken, LuaVarExpr,
};
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use wow_core::SourceSpan;

const MAX_COMPONENTS: usize = 128;
const MAX_NAME_BYTES: usize = 4096;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionBindingState {
    Observed,
    Chunk,
    AnonymousClosure,
    UnsupportedSyntax,
    MissingSyntax,
    OverCapacity,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct NamePart {
    name: String,
    span: SourceSpan,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceFunctionBindingName {
    producer_profile: &'static str,
    form: SourceFunctionDeclarationForm,
    qualified_name: String,
    short_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiver: Option<String>,
    colon_definition: bool,
    name_span: SourceSpan,
    declaration_span: SourceSpan,
    parts: Vec<NamePart>,
}
impl SourceFunctionBindingName {
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
    pub fn short_name(&self) -> &str {
        &self.short_name
    }
    pub fn receiver(&self) -> Option<&str> {
        self.receiver.as_deref()
    }
    pub const fn colon_definition(&self) -> bool {
        self.colon_definition
    }
    pub const fn form(&self) -> SourceFunctionDeclarationForm {
        self.form
    }
    pub const fn name_span(&self) -> SourceSpan {
        self.name_span
    }
    pub const fn declaration_span(&self) -> SourceSpan {
        self.declaration_span
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BindingNameObservation {
    state: SourceFunctionBindingState,
    name: Option<SourceFunctionBindingName>,
}
impl BindingNameObservation {
    pub(super) const fn chunk() -> Self {
        Self::unavailable(SourceFunctionBindingState::Chunk)
    }
    const fn unavailable(state: SourceFunctionBindingState) -> Self {
        Self { state, name: None }
    }
    pub(super) const fn state(&self) -> SourceFunctionBindingState {
        self.state
    }
    pub(super) fn name(&self) -> Option<&SourceFunctionBindingName> {
        self.name.as_ref()
    }
    pub(super) fn validate(
        &self,
        kind: SourceFunctionKind,
        function_span: SourceSpan,
        stop: Option<&AtomicBool>,
    ) -> EmmyMemberCallResult<()> {
        if (kind == SourceFunctionKind::Chunk) != (self.state == SourceFunctionBindingState::Chunk)
            || self.name.is_some() != (self.state == SourceFunctionBindingState::Observed)
        {
            return Err(invalid());
        }
        let Some(name) = &self.name else {
            return Ok(());
        };
        if name.parts.is_empty()
            || name.parts.len() > MAX_COMPONENTS
            || name.qualified_name.len() > MAX_NAME_BYTES
            || name.form == SourceFunctionDeclarationForm::ClosureExpression
            || name.form == SourceFunctionDeclarationForm::LocalFunction
                && (name.parts.len() != 1 || name.colon_definition)
        {
            return Err(invalid());
        }
        let (start, end) = bounds(name.name_span)?;
        let (decl_start, decl_end) = bounds(name.declaration_span)?;
        let (function_start, function_end) = bounds(function_span)?;
        if start >= end
            || decl_start > start
            || decl_end < end
            || decl_start > function_start
            || decl_end < function_end
        {
            return Err(invalid());
        }
        let mut previous = start;
        let mut qualified = String::new();
        for (index, part) in name.parts.iter().enumerate() {
            if let Some(stop) = stop {
                checkpoint(stop)?;
            }
            let (a, b) = bounds(part.span)?;
            if a < previous || b > end || a >= b || b - a != part.name.len() as u64 {
                return Err(invalid());
            }
            previous = b;
            if index > 0 {
                qualified.push(if name.colon_definition && index + 1 == name.parts.len() {
                    ':'
                } else {
                    '.'
                });
            }
            qualified.push_str(&part.name);
        }
        let last = name.parts.last().ok_or_else(invalid)?;
        let receiver = if name.parts.len() > 1 {
            Some(
                name.parts[..name.parts.len() - 1]
                    .iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join("."),
            )
        } else {
            None
        };
        if qualified != name.qualified_name
            || last.name != name.short_name
            || receiver != name.receiver
        {
            return Err(invalid());
        }
        Ok(())
    }
}
fn bounds(span: SourceSpan) -> EmmyMemberCallResult<(u64, u64)> {
    span.validate().map_err(|_| invalid())?;
    span.byte_start().zip(span.byte_end()).ok_or_else(invalid)
}
fn push(
    parts: &mut Vec<NamePart>,
    bytes: &mut usize,
    token: LuaNameToken,
    file: &LuaWorkspaceFile,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<bool> {
    checkpoint(stop)?;
    let text = token.get_name_text();
    let Some(total) = bytes
        .checked_add(text.len())
        .and_then(|n| n.checked_add(usize::from(!parts.is_empty())))
    else {
        return Ok(false);
    };
    if parts.len() >= MAX_COMPONENTS || total > MAX_NAME_BYTES {
        return Ok(false);
    }
    let span = ast_span(file, token.get_range())?;
    parts.push(NamePart {
        name: text.to_owned(),
        span,
    });
    *bytes = total;
    Ok(true)
}
pub(super) fn observe(
    closure: &LuaClosureExpr,
    file: &LuaWorkspaceFile,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<BindingNameObservation> {
    use SourceFunctionBindingState as S;
    checkpoint(stop)?;
    if closure.is_short_closure() || closure.has_do_block() {
        return Ok(BindingNameObservation::unavailable(S::UnsupportedSyntax));
    }
    let Some(parent) = closure.get_parent::<LuaAst>() else {
        return Ok(BindingNameObservation::unavailable(S::MissingSyntax));
    };
    let mut parts = Vec::new();
    let mut bytes = 0;
    let mut colon_definition = false;
    let (form, declaration_span, name_span) = match parent {
        LuaAst::LuaLocalFuncStat(statement) => {
            if statement.is_const() {
                return Ok(BindingNameObservation::unavailable(S::UnsupportedSyntax));
            }
            let Some(token) = statement.get_local_name().and_then(|n| n.get_name_token()) else {
                return Ok(BindingNameObservation::unavailable(S::MissingSyntax));
            };
            let span = ast_span(file, token.get_range())?;
            if !push(&mut parts, &mut bytes, token, file, stop)? {
                return Ok(BindingNameObservation::unavailable(S::OverCapacity));
            }
            (
                SourceFunctionDeclarationForm::LocalFunction,
                ast_span(file, statement.get_range())?,
                span,
            )
        }
        LuaAst::LuaFuncStat(statement) => {
            if statement.is_global() {
                return Ok(BindingNameObservation::unavailable(S::UnsupportedSyntax));
            }
            let Some(mut current) = statement.get_func_name() else {
                return Ok(BindingNameObservation::unavailable(S::MissingSyntax));
            };
            let name_span = ast_span(file, current.get_range())?;
            loop {
                checkpoint(stop)?;
                if parts.len() >= MAX_COMPONENTS {
                    return Ok(BindingNameObservation::unavailable(S::OverCapacity));
                }
                match current {
                    LuaVarExpr::NameExpr(name) => {
                        let Some(token) = name.get_name_token() else {
                            return Ok(BindingNameObservation::unavailable(S::MissingSyntax));
                        };
                        if !push(&mut parts, &mut bytes, token, file, stop)? {
                            return Ok(BindingNameObservation::unavailable(S::OverCapacity));
                        }
                        break;
                    }
                    LuaVarExpr::IndexExpr(index) => {
                        let Some(separator) = index.get_index_token() else {
                            return Ok(BindingNameObservation::unavailable(S::MissingSyntax));
                        };
                        if index.is_safe_index()
                            || !(separator.is_dot() || separator.is_colon())
                            || separator.is_colon() && !parts.is_empty()
                        {
                            return Ok(BindingNameObservation::unavailable(S::UnsupportedSyntax));
                        }
                        if parts.is_empty() {
                            colon_definition = separator.is_colon();
                        }
                        let Some(LuaIndexKey::Name(token)) = index.get_index_key() else {
                            return Ok(BindingNameObservation::unavailable(S::UnsupportedSyntax));
                        };
                        if !push(&mut parts, &mut bytes, token, file, stop)? {
                            return Ok(BindingNameObservation::unavailable(S::OverCapacity));
                        }
                        current = match index.get_prefix_expr() {
                            Some(LuaExpr::NameExpr(name)) => LuaVarExpr::NameExpr(name),
                            Some(LuaExpr::IndexExpr(index)) => LuaVarExpr::IndexExpr(index),
                            _ => {
                                return Ok(BindingNameObservation::unavailable(
                                    S::UnsupportedSyntax,
                                ));
                            }
                        };
                    }
                }
            }
            parts.reverse();
            (
                SourceFunctionDeclarationForm::NamedFunction,
                ast_span(file, statement.get_range())?,
                name_span,
            )
        }
        _ => return Ok(BindingNameObservation::unavailable(S::AnonymousClosure)),
    };
    checkpoint(stop)?;
    let last = parts.last().ok_or_else(invalid)?;
    let short_name = last.name.clone();
    let receiver = if parts.len() > 1 {
        Some(
            parts[..parts.len() - 1]
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join("."),
        )
    } else {
        None
    };
    let mut qualified_name = String::with_capacity(bytes);
    for (index, part) in parts.iter().enumerate() {
        checkpoint(stop)?;
        if index > 0 {
            qualified_name.push(if colon_definition && index + 1 == parts.len() {
                ':'
            } else {
                '.'
            });
        }
        qualified_name.push_str(&part.name);
    }
    let result = BindingNameObservation {
        state: S::Observed,
        name: Some(SourceFunctionBindingName {
            producer_profile: "wow-emmy/named-function-binding-syntax/1",
            form,
            qualified_name,
            short_name,
            receiver,
            colon_definition,
            name_span,
            declaration_span,
            parts,
        }),
    };
    result.validate(
        SourceFunctionKind::Closure,
        ast_span(file, closure.get_range())?,
        Some(stop),
    )?;
    Ok(result)
}
