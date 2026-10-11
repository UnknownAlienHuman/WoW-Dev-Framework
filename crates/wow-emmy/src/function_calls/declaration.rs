//! Selected syntactic declaration observations from already-held Main closures.
//! These are header ingredients, not semantic types, returns or whole-body shape.
use std::io::Write;
use std::sync::atomic::AtomicBool;

use emmylua_parser::{LuaAst, LuaAstNode, LuaAstToken, LuaClosureExpr, LuaTokenKind, LuaVarExpr};
use serde::{Deserialize, Serialize};
use wow_core::SourceSpan;

use super::{FunctionCallBudget, SourceFunctionKind, invalid, resource_budget, visit};
use crate::LuaWorkspaceFile;
use crate::bindings::checkpoint;
use crate::references::{EmmyMemberCallResult, ast_span};

/// A separate selected recipe; existing function-call report profiles are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PhysicalFunctionDeclarationRecipe {
    #[serde(rename = "wow-emmy/physical-function-declaration-syntax/1")]
    DeclaredSyntaxV1,
}
impl<'de> Deserialize<'de> for PhysicalFunctionDeclarationRecipe {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let recipe = String::deserialize(deserializer)?;
        match recipe.as_str() {
            "wow-emmy/physical-function-declaration-syntax/1" => Ok(Self::DeclaredSyntaxV1),
            _ => Err(serde::de::Error::custom(
                "unsupported physical-function declaration recipe",
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationForm {
    LocalFunction,
    NamedFunction,
    /// Includes assigned/argument/table/returned closures, without inferred names.
    ClosureExpression,
}

/// Order and exact token spans are retained; implicit self is not inserted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceDeclaredParameter {
    Name { name: String, span: SourceSpan },
    Vararg { span: SourceSpan },
}
impl SourceDeclaredParameter {
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::Name { span, .. } | Self::Vararg { span } => *span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationUnavailable {
    ChunkHasNoDeclaration,
    MissingParent,
    MissingDeclarationName,
    MissingParameterList,
    MissingParameterToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionDeclarationUnsupported {
    ExtendedClosure,
    ExtendedDeclaration,
    NamedIndexForm,
    VarargPlacement,
}

/// Serialize-only output held by an immutable, snapshot-bound function report.
/// Declaration form is not body equivalence or a complete callable signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceFunctionDeclaration {
    Observed {
        form: SourceFunctionDeclarationForm,
        colon_definition: bool,
        /// Exact declaration-parent span; comparison recipes exclude this binding.
        declaration_span: SourceSpan,
        parameters: Vec<SourceDeclaredParameter>,
    },
    Unavailable {
        reason: SourceFunctionDeclarationUnavailable,
    },
    Unsupported {
        reason: SourceFunctionDeclarationUnsupported,
    },
}
impl SourceFunctionDeclaration {
    pub(super) const fn chunk() -> Self {
        Self::Unavailable {
            reason: SourceFunctionDeclarationUnavailable::ChunkHasNoDeclaration,
        }
    }

    pub(super) fn validate(
        &self,
        kind: SourceFunctionKind,
        function_span: SourceSpan,
        stop: Option<&AtomicBool>,
    ) -> EmmyMemberCallResult<()> {
        if let Some(stop) = stop {
            checkpoint(stop)?;
        }
        if kind == SourceFunctionKind::Chunk {
            return if self == &Self::chunk() {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        match self {
            Self::Unavailable {
                reason: SourceFunctionDeclarationUnavailable::ChunkHasNoDeclaration,
            } => Err(invalid()),
            Self::Unavailable { .. } | Self::Unsupported { .. } => Ok(()),
            Self::Observed {
                form,
                colon_definition,
                declaration_span,
                parameters,
            } => {
                if *colon_definition && *form != SourceFunctionDeclarationForm::NamedFunction {
                    return Err(invalid());
                }
                function_span.validate().map_err(|_| invalid())?;
                let (Some(start), Some(end)) =
                    (function_span.byte_start(), function_span.byte_end())
                else {
                    return Err(invalid());
                };
                declaration_span.validate().map_err(|_| invalid())?;
                let (Some(declaration_start), Some(declaration_end)) =
                    (declaration_span.byte_start(), declaration_span.byte_end())
                else {
                    return Err(invalid());
                };
                if start >= end
                    || declaration_start > start
                    || declaration_end < end
                    || declaration_start >= declaration_end
                    || *form == SourceFunctionDeclarationForm::ClosureExpression
                        && *declaration_span != function_span
                {
                    return Err(invalid());
                }
                let mut previous_end = start;
                let mut vararg = false;
                for parameter in parameters {
                    if let Some(stop) = stop {
                        checkpoint(stop)?;
                    }
                    let span = parameter.span();
                    span.validate().map_err(|_| invalid())?;
                    let (Some(a), Some(b)) = (span.byte_start(), span.byte_end()) else {
                        return Err(invalid());
                    };
                    if a < previous_end || b > end || a >= b || vararg {
                        return Err(invalid());
                    }
                    match parameter {
                        SourceDeclaredParameter::Name { name, .. } => {
                            if name.is_empty() || (b - a) as u128 != name.len() as u128 {
                                return Err(invalid());
                            }
                        }
                        SourceDeclaredParameter::Vararg { .. } => {
                            if b - a != 3 {
                                return Err(invalid());
                            }
                            vararg = true;
                        }
                    }
                    previous_end = b;
                }
                Ok(())
            }
        }
    }
}

/// Early retained-payload bound shares the existing report-byte ceiling.
/// Final framing and combined output are still counted by the report owner.
/// Dropped temporary observations remain conservatively charged.
pub(super) struct DeclarationBudget {
    used: u64,
    limit: u64,
}
impl DeclarationBudget {
    pub(super) const fn new(limit: u64) -> Self {
        Self { used: 0, limit }
    }

    fn ensure(&self, bytes: usize) -> EmmyMemberCallResult<()> {
        let attempted = u128::from(self.used) + bytes as u128;
        if attempted > u128::from(self.limit) {
            return Err(resource_budget(
                "declaration bytes",
                attempted,
                self.limit,
                None,
            ));
        }
        Ok(())
    }

    pub(super) fn charge<T: Serialize>(
        &mut self,
        value: &T,
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<()> {
        checkpoint(stop)?;
        struct Count<'a> {
            budget: &'a mut DeclarationBudget,
            stop: &'a AtomicBool,
            attempted: u128,
        }
        impl Write for Count<'_> {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                if self.stop.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err(std::io::Error::other("declaration cancelled"));
                }
                self.attempted = u128::from(self.budget.used) + bytes.len() as u128;
                if self.attempted > u128::from(self.budget.limit) {
                    return Err(std::io::Error::other("declaration byte budget"));
                }
                self.budget.used = u64::try_from(self.attempted)
                    .map_err(|_| std::io::Error::other("declaration byte budget"))?;
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut count = Count {
            attempted: u128::from(self.used),
            budget: self,
            stop,
        };
        let result = serde_json::to_writer(&mut count, value);
        checkpoint(stop)?;
        result.map_err(|_| {
            resource_budget(
                "declaration bytes",
                count.attempted,
                count.budget.limit,
                None,
            )
        })
    }

    fn outcome(
        &mut self,
        observation: SourceFunctionDeclaration,
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<SourceFunctionDeclaration> {
        self.charge(&observation, stop)?;
        Ok(observation)
    }
}

pub(super) fn observe(
    closure: &LuaClosureExpr,
    file: &LuaWorkspaceFile,
    visits: &mut u64,
    limits: FunctionCallBudget,
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SourceFunctionDeclaration> {
    use SourceFunctionDeclarationUnavailable as U;
    use SourceFunctionDeclarationUnsupported as S;
    checkpoint(stop)?;
    visit(visits, limits.max_ast_visits())?;
    if closure.is_short_closure() || closure.has_do_block() {
        return bytes.outcome(
            SourceFunctionDeclaration::Unsupported {
                reason: S::ExtendedClosure,
            },
            stop,
        );
    }
    let Some(parent) = closure.get_parent::<LuaAst>() else {
        return bytes.outcome(
            SourceFunctionDeclaration::Unavailable {
                reason: U::MissingParent,
            },
            stop,
        );
    };
    let (form, colon_definition, declaration_span) = match parent {
        LuaAst::LuaLocalFuncStat(statement) => {
            if statement.is_const() {
                return bytes.outcome(
                    SourceFunctionDeclaration::Unsupported {
                        reason: S::ExtendedDeclaration,
                    },
                    stop,
                );
            }
            if statement.get_local_name().is_none() {
                return bytes.outcome(
                    SourceFunctionDeclaration::Unavailable {
                        reason: U::MissingDeclarationName,
                    },
                    stop,
                );
            }
            (
                SourceFunctionDeclarationForm::LocalFunction,
                false,
                ast_span(file, statement.get_range())?,
            )
        }
        LuaAst::LuaFuncStat(statement) => {
            if statement.is_global() {
                return bytes.outcome(
                    SourceFunctionDeclaration::Unsupported {
                        reason: S::ExtendedDeclaration,
                    },
                    stop,
                );
            }
            let Some(name) = statement.get_func_name() else {
                return bytes.outcome(
                    SourceFunctionDeclaration::Unavailable {
                        reason: U::MissingDeclarationName,
                    },
                    stop,
                );
            };
            let colon = match name {
                LuaVarExpr::NameExpr(name) => {
                    if name.get_name_token().is_none() {
                        return bytes.outcome(
                            SourceFunctionDeclaration::Unavailable {
                                reason: U::MissingDeclarationName,
                            },
                            stop,
                        );
                    }
                    false
                }
                LuaVarExpr::IndexExpr(index) => {
                    let Some(token) = index.get_index_token() else {
                        return bytes.outcome(
                            SourceFunctionDeclaration::Unavailable {
                                reason: U::MissingDeclarationName,
                            },
                            stop,
                        );
                    };
                    if !token.is_dot() && !token.is_colon() {
                        return bytes.outcome(
                            SourceFunctionDeclaration::Unsupported {
                                reason: S::NamedIndexForm,
                            },
                            stop,
                        );
                    }
                    token.is_colon()
                }
            };
            (
                SourceFunctionDeclarationForm::NamedFunction,
                colon,
                ast_span(file, statement.get_range())?,
            )
        }
        _ => (
            SourceFunctionDeclarationForm::ClosureExpression,
            false,
            ast_span(file, closure.get_range())?,
        ),
    };
    let Some(list) = closure.get_params_list() else {
        return bytes.outcome(
            SourceFunctionDeclaration::Unavailable {
                reason: U::MissingParameterList,
            },
            stop,
        );
    };
    let mut observation = SourceFunctionDeclaration::Observed {
        form,
        colon_definition,
        declaration_span,
        parameters: Vec::new(),
    };
    bytes.charge(&observation, stop)?;
    let SourceFunctionDeclaration::Observed { parameters, .. } = &mut observation else {
        return Err(invalid());
    };
    let mut vararg = false;
    for parameter in list.get_params() {
        checkpoint(stop)?;
        visit(visits, limits.max_ast_visits())?;
        if vararg {
            return bytes.outcome(
                SourceFunctionDeclaration::Unsupported {
                    reason: S::VarargPlacement,
                },
                stop,
            );
        }
        let token = match (parameter.is_dots(), parameter.get_name_token()) {
            (true, None) => {
                vararg = true;
                let Some(dots) = parameter.token_by_kind(LuaTokenKind::TkDots) else {
                    return bytes.outcome(
                        SourceFunctionDeclaration::Unavailable {
                            reason: U::MissingParameterToken,
                        },
                        stop,
                    );
                };
                SourceDeclaredParameter::Vararg {
                    span: ast_span(file, dots.get_range())?,
                }
            }
            (false, Some(name)) => {
                let text = name.get_name_text();
                if text.is_empty() {
                    return bytes.outcome(
                        SourceFunctionDeclaration::Unavailable {
                            reason: U::MissingParameterToken,
                        },
                        stop,
                    );
                }
                // Bound the one candidate string before allocating, then count its
                // actual escaped JSON and span before retaining the vector row.
                bytes.ensure(text.len())?;
                SourceDeclaredParameter::Name {
                    name: text.to_owned(),
                    span: ast_span(file, name.get_range())?,
                }
            }
            _ => {
                return bytes.outcome(
                    SourceFunctionDeclaration::Unavailable {
                        reason: U::MissingParameterToken,
                    },
                    stop,
                );
            }
        };
        bytes.charge(&token, stop)?;
        if !parameters.is_empty() {
            bytes.ensure(1)?;
            bytes.used += 1;
        }
        parameters.push(token);
    }
    checkpoint(stop)?;
    observation.validate(
        SourceFunctionKind::Closure,
        ast_span(file, closure.get_range())?,
        Some(stop),
    )?;
    Ok(observation)
}
