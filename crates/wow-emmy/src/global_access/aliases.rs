//! Same-file lexical alias index. This is source provenance, not a runtime heap.
use super::{AccessBudget, SourceAliasHop, budget, invalid, path};
use crate::LuaWorkspaceFile;
use crate::bindings::checkpoint;
use crate::references::{EmmyMemberCallResult, ast_span};
use emmylua_code_analysis::{LuaDeclId, LuaSemanticDeclId, SemanticDeclLevel, SemanticModel};
use emmylua_parser::{LuaAst, LuaAstNode, LuaNameExpr, LuaVarExpr};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use wow_core::SourceSpan;

const MAX_ALIAS_DEFINITIONS: usize = 16_384;

pub(super) struct AliasCandidate {
    pub path: path::SourcePath,
    pub hop: SourceAliasHop,
}
pub(crate) struct AliasIndex {
    candidates: HashMap<LuaDeclId, AliasCandidate>,
    /// Any syntactic rebinding, even in a branch or a nested closure, blocks
    /// inference for the entire binding. No control-flow dominance is invented.
    writes: HashMap<LuaDeclId, SourceSpan>,
}
impl AliasIndex {
    pub(crate) fn collect(
        model: &SemanticModel<'_>,
        file: &LuaWorkspaceFile,
        limits: &mut AccessBudget,
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<Self> {
        let mut result = Self {
            candidates: HashMap::new(),
            writes: HashMap::new(),
        };
        for ast in model.get_root().descendants::<LuaAst>() {
            checkpoint(stop)?;
            limits.step()?;
            match ast {
                LuaAst::LuaLocalStat(statement) => {
                    // Zip only explicit initializer expressions. A trailing call
                    // or vararg never creates guessed aliases for extra locals.
                    for (name, expr) in statement
                        .get_local_name_list()
                        .zip(statement.get_value_exprs())
                    {
                        limits.step()?;
                        let Some(LuaSemanticDeclId::LuaDecl(id)) = model
                            .find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
                        else {
                            continue;
                        };
                        let declaration = model
                            .get_db()
                            .get_decl_index()
                            .get_decl(&id)
                            .ok_or_else(invalid)?;
                        if !declaration.is_local() || declaration.is_param() {
                            continue;
                        }
                        let initializer_span = ast_span(file, expr.get_range())?;
                        let Some(path) = path::from_expr(expr, limits)? else {
                            continue;
                        };
                        if result.candidates.len() >= MAX_ALIAS_DEFINITIONS {
                            return Err(budget());
                        }
                        let name = declaration.get_name().to_owned();
                        if name.len() > 1024 {
                            return Err(budget());
                        }
                        limits.text(name.len() + 256)?;
                        let hop = SourceAliasHop {
                            name,
                            declaration_span: ast_span(file, declaration.get_range())?,
                            initializer_span,
                            statement_span: ast_span(file, statement.get_range())?,
                            first_reassignment: None,
                        };
                        if result
                            .candidates
                            .insert(id, AliasCandidate { path, hop })
                            .is_some()
                        {
                            return Err(invalid());
                        }
                    }
                }
                LuaAst::LuaAssignStat(statement) => {
                    for var in statement.get_var_and_expr_list().0 {
                        limits.step()?;
                        if let LuaVarExpr::NameExpr(name) = var {
                            result.record_write(model, file, &name)?;
                        }
                    }
                }
                LuaAst::LuaFuncStat(statement) => {
                    if let Some(LuaVarExpr::NameExpr(name)) = statement.get_func_name() {
                        result.record_write(model, file, &name)?;
                    }
                }
                _ => {}
            }
        }
        Ok(result)
    }
    fn record_write(
        &mut self,
        model: &SemanticModel<'_>,
        file: &LuaWorkspaceFile,
        name: &LuaNameExpr,
    ) -> EmmyMemberCallResult<()> {
        if let Some(LuaSemanticDeclId::LuaDecl(id)) =
            model.find_decl(name.syntax().clone().into(), SemanticDeclLevel::default())
        {
            let declaration = model
                .get_db()
                .get_decl_index()
                .get_decl(&id)
                .ok_or_else(invalid)?;
            if declaration.is_local() && !declaration.is_param() {
                if self.writes.len() >= MAX_ALIAS_DEFINITIONS && !self.writes.contains_key(&id) {
                    return Err(budget());
                }
                self.writes
                    .entry(id)
                    .or_insert(ast_span(file, name.get_range())?);
            }
        }
        Ok(())
    }
    pub(super) fn get(&self, id: &LuaDeclId) -> Option<&AliasCandidate> {
        self.candidates.get(id)
    }
    pub(super) fn first_write(&self, id: &LuaDeclId) -> Option<SourceSpan> {
        self.writes.get(id).copied()
    }
}
