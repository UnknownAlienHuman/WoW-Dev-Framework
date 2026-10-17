// Adapted from EmmyLuaLs/emmylua-analyzer-rust aaaca68425d9362876228649b0b8d92f07654daa.
// Copyright(c)2024-2026CppCXY. MIT; see ../../DOCUMENT_SYMBOLS_DONOR_LICENSE.
use super::builder::DocumentSymbolBuilder;
use super::comment::build_doc_region_symbol;
use super::expr::{build_closure_expr_symbol, build_table_symbol};
use super::stats::{
    IfSymbolContext, build_assign_stat_symbol, build_do_stat_symbol, build_for_range_stat_symbol,
    build_for_stat_symbol, build_func_stat_symbol, build_if_stat_symbol,
    build_local_func_stat_symbol, build_local_stat_symbol,
};
use emmylua_parser::{
    LuaAstNode, LuaBlock, LuaChunk, LuaComment, LuaExpr, LuaSingleArgExpr, LuaStat, LuaSyntaxId,
    LuaSyntaxNode,
};
pub(super) fn process_chunk(
    builder: &mut DocumentSymbolBuilder,
    chunk: &LuaChunk,
    parent_id: LuaSyntaxId,
) -> Option<()> {
    for node in chunk.syntax().children() {
        builder.checkpoint()?;
        match node {
            comment if LuaComment::can_cast(comment.kind().into()) => {
                let comment = LuaComment::cast(comment.clone())?;
                process_comment(builder, &comment, parent_id);
            }
            block if LuaBlock::can_cast(block.kind().into()) => {
                let block = LuaBlock::cast(block.clone())?;
                process_block(builder, block, parent_id)?;
            }
            _ => {}
        }
    }

    Some(())
}

fn process_comment(
    builder: &mut DocumentSymbolBuilder,
    comment: &LuaComment,
    parent_id: LuaSyntaxId,
) {
    build_doc_region_symbol(builder, comment.clone(), parent_id);
}

fn process_block(
    builder: &mut DocumentSymbolBuilder,
    block: LuaBlock,
    parent_id: LuaSyntaxId,
) -> Option<()> {
    for child in block.syntax().children() {
        builder.checkpoint()?;
        match child {
            comment if LuaComment::can_cast(comment.kind().into()) => {
                let comment = LuaComment::cast(comment.clone())?;
                process_comment(builder, &comment, parent_id);
            }
            stat if LuaStat::can_cast(stat.kind().into()) => {
                let stat = LuaStat::cast(stat.clone())?;
                process_stat(builder, stat, parent_id)?;
            }
            _ => {}
        }
    }

    Some(())
}

fn process_stat(
    builder: &mut DocumentSymbolBuilder,
    stat: LuaStat,
    parent_id: LuaSyntaxId,
) -> Option<()> {
    match stat {
        LuaStat::LocalStat(local_stat) => {
            let bindings = build_local_stat_symbol(builder, local_stat, parent_id)?;
            for binding in bindings {
                builder.checkpoint()?;
                if let Some(expr) = binding.value_expr {
                    process_expr(builder, expr, binding.symbol_id, true)?;
                }
            }
        }
        LuaStat::AssignStat(assign_stat) => {
            let bindings = build_assign_stat_symbol(builder, assign_stat.clone(), parent_id)?;
            for binding in bindings {
                builder.checkpoint()?;
                if let Some(expr) = binding.value_expr {
                    process_expr(builder, expr, binding.symbol_id, true)?;
                }
            }
        }
        LuaStat::FuncStat(func_stat) => {
            let func_id = build_func_stat_symbol(builder, func_stat.clone(), parent_id)?;
            if let Some(closure) = func_stat.get_closure() {
                let scope_parent = build_closure_expr_symbol(builder, closure.clone(), func_id)?;
                if let Some(block) = closure.get_block() {
                    process_block(builder, block, scope_parent)?;
                }
            }
        }
        LuaStat::LocalFuncStat(local_func) => {
            let func_id = build_local_func_stat_symbol(builder, local_func.clone(), parent_id)?;
            if let Some(closure) = local_func.get_closure() {
                let scope_parent = build_closure_expr_symbol(builder, closure.clone(), func_id)?;
                if let Some(block) = closure.get_block() {
                    process_block(builder, block, scope_parent)?;
                }
            }
        }
        LuaStat::ForStat(for_stat) => {
            let for_id = build_for_stat_symbol(builder, for_stat.clone(), parent_id)?;
            process_exprs(builder, for_stat.syntax(), for_id)?;
            if let Some(block) = for_stat.get_block() {
                process_block(builder, block, for_id)?;
            }
        }
        LuaStat::ForRangeStat(for_range_stat) => {
            let for_range_id =
                build_for_range_stat_symbol(builder, for_range_stat.clone(), parent_id)?;
            process_exprs(builder, for_range_stat.syntax(), for_range_id)?;
            if let Some(block) = for_range_stat.get_block() {
                process_block(builder, block, for_range_id)?;
            }
        }
        LuaStat::IfStat(if_stat) => {
            let ctx = build_if_stat_symbol(builder, if_stat.clone(), parent_id)?;
            if let Some(condition) = if_stat.get_condition_expr() {
                process_expr(builder, condition, ctx.if_id, false)?;
            }
            if let Some(block) = if_stat.get_block() {
                process_block(builder, block, ctx.if_id)?;
            }
            process_if_clauses(builder, ctx)?;
        }
        LuaStat::WhileStat(while_stat) => {
            if let Some(condition) = while_stat.get_condition_expr() {
                process_expr(builder, condition, parent_id, false)?;
            }
            if let Some(block) = while_stat.get_block() {
                process_block(builder, block, parent_id)?;
            }
        }
        LuaStat::RepeatStat(repeat_stat) => {
            if let Some(block) = repeat_stat.get_block() {
                process_block(builder, block, parent_id)?;
            }
            if let Some(condition) = repeat_stat.get_condition_expr() {
                process_expr(builder, condition, parent_id, false)?;
            }
        }
        LuaStat::DoStat(do_stat) => {
            let do_id = build_do_stat_symbol(builder, do_stat.clone(), parent_id)?;
            if let Some(block) = do_stat.get_block() {
                process_block(builder, block, do_id)?;
            }
        }
        LuaStat::CallExprStat(call_stat) => {
            process_exprs(builder, call_stat.syntax(), parent_id)?;
        }
        LuaStat::ReturnStat(return_stat) => {
            process_exprs(builder, return_stat.syntax(), parent_id)?;
        }
        LuaStat::GlobalStat(global_stat) => {
            process_exprs(builder, global_stat.syntax(), parent_id)?;
        }
        LuaStat::GotoStat(_)
        | LuaStat::BreakStat(_)
        | LuaStat::ContinueStat(_)
        | LuaStat::LabelStat(_)
        | LuaStat::EmptyStat(_) => {}
    }

    Some(())
}

fn process_if_clauses(builder: &mut DocumentSymbolBuilder, ctx: IfSymbolContext) -> Option<()> {
    for (clause, clause_id) in ctx.clause_symbols {
        builder.checkpoint()?;
        if let Some(condition) = clause.get_condition_expr() {
            process_expr(builder, condition, clause_id, false)?;
        }
        if let Some(block) = clause.get_block() {
            process_block(builder, block, clause_id)?;
        }
    }

    Some(())
}

fn process_exprs(
    builder: &mut DocumentSymbolBuilder,
    syntax: &LuaSyntaxNode,
    parent_id: LuaSyntaxId,
) -> Option<()> {
    for child in syntax.children() {
        builder.checkpoint()?;
        match child {
            expr if LuaExpr::can_cast(expr.kind().into()) => {
                let expr = LuaExpr::cast(expr.clone())?;
                process_expr(builder, expr, parent_id, false)?;
            }
            _ => {}
        }
    }
    Some(())
}

fn process_expr(
    builder: &mut DocumentSymbolBuilder,
    expr: LuaExpr,
    parent_id: LuaSyntaxId,
    inline_table_to_parent: bool,
) -> Option<()> {
    match expr {
        LuaExpr::TableExpr(table) => {
            if !inline_table_to_parent {
                if table.is_object() {
                    for field in table.get_fields() {
                        builder.checkpoint()?;
                        if let Some(value_expr) = field.get_value_expr() {
                            process_expr(builder, value_expr, parent_id, false)?;
                        }
                    }
                }
                return Some(());
            }
            let table_id =
                build_table_symbol(builder, table.clone(), parent_id, inline_table_to_parent)?;
            for field in table.get_fields() {
                builder.checkpoint()?;
                if let Some(value_expr) = field.get_value_expr() {
                    let field_id =
                        LuaSyntaxId::new(field.syntax().kind(), field.syntax().text_range());
                    let next_parent = if builder.contains_symbol(&field_id) {
                        field_id
                    } else {
                        table_id
                    };
                    process_expr(builder, value_expr, next_parent, true)?;
                }
            }
        }
        LuaExpr::ClosureExpr(closure) => {
            if !inline_table_to_parent {
                return Some(());
            }
            let scope_parent = build_closure_expr_symbol(builder, closure.clone(), parent_id)?;
            if let Some(block) = closure.get_block() {
                process_block(builder, block, scope_parent)?;
            }
        }
        LuaExpr::BinaryExpr(binary) => {
            if let Some((left, right)) = binary.get_exprs() {
                process_expr(builder, left, parent_id, inline_table_to_parent)?;
                process_expr(builder, right, parent_id, inline_table_to_parent)?;
            }
        }
        LuaExpr::UnaryExpr(unary) => {
            if let Some(inner) = unary.get_expr() {
                process_expr(builder, inner, parent_id, inline_table_to_parent)?;
            }
        }
        LuaExpr::ParenExpr(paren) => {
            if let Some(inner) = paren.get_expr() {
                process_expr(builder, inner, parent_id, inline_table_to_parent)?;
            }
        }
        LuaExpr::CallExpr(call) => {
            if let Some(prefix) = call.get_prefix_expr() {
                process_expr(builder, prefix, parent_id, inline_table_to_parent)?;
            }
            if let Some(args) = call.get_args_list() {
                let single_arg_no_parens = args.is_single_arg_no_parens();
                if single_arg_no_parens {
                    if let Some(LuaSingleArgExpr::TableExpr(table)) = args.get_single_arg_expr() {
                        process_expr(
                            builder,
                            LuaExpr::TableExpr(table),
                            parent_id,
                            inline_table_to_parent,
                        )?;
                    }
                } else {
                    let mut visited = 0usize;
                    let max_symbols = builder.limits().max_symbols;
                    for arg in args.get_args() {
                        visited += 1;
                        if visited > max_symbols {
                            return builder.limit();
                        }
                        builder.checkpoint()?;
                        process_expr(builder, arg, parent_id, inline_table_to_parent)?;
                    }
                }
            }
        }
        LuaExpr::IndexExpr(index_expr) => {
            if let Some(prefix) = index_expr.get_prefix_expr() {
                process_expr(builder, prefix, parent_id, inline_table_to_parent)?;
            }
        }
        LuaExpr::NameExpr(_) | LuaExpr::LiteralExpr(_) => {}
        LuaExpr::TernaryExpr(ternary_expr) => {
            if let Some((condition, true_expr, false_expr)) = ternary_expr.get_exprs() {
                process_expr(builder, condition, parent_id, inline_table_to_parent)?;
                process_expr(builder, true_expr, parent_id, inline_table_to_parent)?;
                process_expr(builder, false_expr, parent_id, inline_table_to_parent)?;
            }
        }
    }

    Some(())
}
