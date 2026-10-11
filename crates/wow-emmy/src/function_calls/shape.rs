//! Full ordered body syntax from an already-held native closure. No source rescan.
use std::sync::atomic::AtomicBool;

use emmylua_parser::{LuaAstNode, LuaClosureExpr, LuaSyntaxKind, LuaTokenKind};
use rowan::{NodeOrToken, WalkEvent};
use serde::Serialize;

use super::declaration::DeclarationBudget;
use super::{
    FunctionCallBudget, SourceFunctionDeclaration, SourceFunctionKind, invalid, resource_budget,
    visit,
};
use crate::bindings::checkpoint;
use crate::references::EmmyMemberCallResult;

/// Owned stable tags. Explicit values are the comparison encoding, never upstream ordinals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u16)]
pub enum FunctionShapeNode {
    Block = 1,
    EmptyStatement = 2,
    LocalStatement = 3,
    LocalFunction = 4,
    IfStatement = 5,
    ElseIfClause = 6,
    ElseClause = 7,
    WhileStatement = 8,
    DoStatement = 9,
    ForStatement = 10,
    ForRangeStatement = 11,
    RepeatStatement = 12,
    NamedFunction = 13,
    LabelStatement = 14,
    BreakStatement = 15,
    ReturnStatement = 16,
    GotoStatement = 17,
    CallStatement = 18,
    AssignStatement = 19,
    ParenthesizedExpression = 20,
    LiteralExpression = 21,
    ClosureExpression = 22,
    UnaryExpression = 23,
    BinaryExpression = 24,
    TableArray = 25,
    TableObject = 26,
    TableEmpty = 27,
    CallExpression = 28,
    IndexExpression = 29,
    NameExpression = 30,
    LocalName = 31,
    ParameterName = 32,
    ParameterList = 33,
    CallArgumentList = 34,
    TableFieldAssignment = 35,
    TableFieldValue = 36,
}
impl FunctionShapeNode {
    pub const fn encoding_tag(self) -> u16 {
        self as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[repr(u16)]
pub enum FunctionShapeToken {
    And = 1,
    Break = 2,
    Do = 3,
    Else = 4,
    ElseIf = 5,
    End = 6,
    For = 7,
    Function = 8,
    Goto = 9,
    If = 10,
    In = 11,
    Local = 12,
    Not = 13,
    Or = 14,
    Repeat = 15,
    Return = 16,
    Then = 17,
    Until = 18,
    While = 19,
    Plus = 20,
    Minus = 21,
    Multiply = 22,
    Divide = 23,
    IntegerDivide = 24,
    Dot = 25,
    Concatenate = 26,
    Vararg = 27,
    Comma = 28,
    Assign = 29,
    Equal = 30,
    GreaterEqual = 31,
    LessEqual = 32,
    NotEqual = 33,
    ShiftLeft = 34,
    ShiftRight = 35,
    Less = 36,
    Greater = 37,
    Modulo = 38,
    Power = 39,
    Length = 40,
    BitAnd = 41,
    BitOr = 42,
    BitXor = 43,
    Colon = 44,
    DoubleColon = 45,
    Semicolon = 46,
    LeftBracket = 47,
    RightBracket = 48,
    LeftParen = 49,
    RightParen = 50,
    LeftBrace = 51,
    RightBrace = 52,
    Name = 53,
    Number = 54,
    String = 55,
    Boolean = 56,
    Nil = 57,
}
impl FunctionShapeToken {
    pub const fn encoding_tag(self) -> u16 {
        self as u16
    }
}

/// Balanced depth-first events retain the entire body, including nested closures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "tag", rename_all = "snake_case")]
pub enum FunctionShapeEvent {
    Enter(FunctionShapeNode),
    Leave(FunctionShapeNode),
    Token(FunctionShapeToken),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionShapeUnavailable {
    ChunkHasNoDeclaration,
    MissingDeclaration,
    MissingBody,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceFunctionShapeUnsupported {
    UnsupportedDeclaration,
    UnsupportedSyntax,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum SourceFunctionShape {
    Observed {
        events: Vec<FunctionShapeEvent>,
    },
    Unavailable {
        reason: SourceFunctionShapeUnavailable,
    },
    Unsupported {
        reason: SourceFunctionShapeUnsupported,
    },
}
impl SourceFunctionShape {
    pub(super) const fn chunk() -> Self {
        Self::Unavailable {
            reason: SourceFunctionShapeUnavailable::ChunkHasNoDeclaration,
        }
    }

    pub(super) fn validate(
        &self,
        kind: SourceFunctionKind,
        limits: FunctionCallBudget,
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
                reason: SourceFunctionShapeUnavailable::ChunkHasNoDeclaration,
            } => Err(invalid()),
            Self::Unavailable { .. } | Self::Unsupported { .. } => Ok(()),
            Self::Observed { events } => {
                if events.first() != Some(&FunctionShapeEvent::Enter(FunctionShapeNode::Block))
                    || events.last() != Some(&FunctionShapeEvent::Leave(FunctionShapeNode::Block))
                    || events.len() as u128 > u128::from(limits.max_ast_visits())
                {
                    return Err(invalid());
                }
                let mut stack = Vec::new();
                for (index, event) in events.iter().enumerate() {
                    if let Some(stop) = stop {
                        checkpoint(stop)?;
                    }
                    match event {
                        FunctionShapeEvent::Enter(tag) => {
                            if stack.len() >= super::MAX_SCOPE_DEPTH {
                                return Err(invalid());
                            }
                            stack.push(*tag);
                        }
                        FunctionShapeEvent::Leave(tag) if stack.pop() == Some(*tag) => {}
                        FunctionShapeEvent::Token(_) if !stack.is_empty() => {}
                        _ => return Err(invalid()),
                    }
                    if stack.is_empty() && index + 1 != events.len() {
                        return Err(invalid());
                    }
                }
                if stack.is_empty() {
                    Ok(())
                } else {
                    Err(invalid())
                }
            }
        }
    }
}

pub(super) fn observe(
    closure: &LuaClosureExpr,
    declaration: &SourceFunctionDeclaration,
    visits: &mut u64,
    limits: FunctionCallBudget,
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SourceFunctionShape> {
    checkpoint(stop)?;
    let early = match declaration {
        SourceFunctionDeclaration::Observed { .. } => None,
        SourceFunctionDeclaration::Unavailable { .. } => Some(SourceFunctionShape::Unavailable {
            reason: SourceFunctionShapeUnavailable::MissingDeclaration,
        }),
        SourceFunctionDeclaration::Unsupported { .. } => Some(SourceFunctionShape::Unsupported {
            reason: SourceFunctionShapeUnsupported::UnsupportedDeclaration,
        }),
    };
    if let Some(outcome) = early {
        bytes.charge(&outcome, stop)?;
        return Ok(outcome);
    }
    let Some(body) = closure.get_block() else {
        // The pinned ordinary closure grammar omits Block precisely when the
        // parameter list is followed by its closing end. Main parse health is
        // checked by the report owner; normalize that native empty-body form.
        if closure.token_by_kind(LuaTokenKind::TkEnd).is_some() {
            visit(visits, limits.max_ast_visits())?;
            visit(visits, limits.max_ast_visits())?;
            let events = [
                FunctionShapeEvent::Enter(FunctionShapeNode::Block),
                FunctionShapeEvent::Leave(FunctionShapeNode::Block),
            ];
            bytes.charge(&SourceFunctionShape::Observed { events: Vec::new() }, stop)?;
            for event in &events {
                bytes.charge(event, stop)?;
            }
            bytes.charge(&0_u8, stop)?;
            let outcome = SourceFunctionShape::Observed {
                events: events.to_vec(),
            };
            outcome.validate(SourceFunctionKind::Closure, limits, Some(stop))?;
            return Ok(outcome);
        }
        let outcome = SourceFunctionShape::Unavailable {
            reason: SourceFunctionShapeUnavailable::MissingBody,
        };
        bytes.charge(&outcome, stop)?;
        return Ok(outcome);
    };
    let mut observation = SourceFunctionShape::Observed { events: Vec::new() };
    bytes.charge(&observation, stop)?;
    let SourceFunctionShape::Observed { events } = &mut observation else {
        return Err(invalid());
    };
    let mut walk = body.syntax().preorder_with_tokens();
    let mut depth = 0_u64;
    while let Some(event) = walk.next() {
        checkpoint(stop)?;
        // Includes skipped/trivia work and repeated nested-body work in the one
        // native report visit ledger. No uncharged per-closure traversal.
        visit(visits, limits.max_ast_visits())?;
        let normalized = match event {
            WalkEvent::Enter(NodeOrToken::Node(node)) => {
                let kind: LuaSyntaxKind = node.kind().into();
                if kind == LuaSyntaxKind::Comment {
                    walk.skip_subtree();
                    continue;
                }
                if let Some(tag) = node_tag(kind) {
                    depth = depth.checked_add(1).ok_or_else(invalid)?;
                    if u128::from(depth) > super::MAX_SCOPE_DEPTH as u128 {
                        return Err(resource_budget(
                            "shape depth",
                            u128::from(depth),
                            super::MAX_SCOPE_DEPTH as u64,
                            None,
                        ));
                    }
                    Some(FunctionShapeEvent::Enter(tag))
                } else {
                    return unsupported(bytes, stop);
                }
            }
            WalkEvent::Leave(NodeOrToken::Node(node)) => {
                let kind: LuaSyntaxKind = node.kind().into();
                if kind == LuaSyntaxKind::Comment {
                    continue;
                }
                depth = depth.checked_sub(1).ok_or_else(invalid)?;
                Some(FunctionShapeEvent::Leave(
                    node_tag(kind).ok_or_else(invalid)?,
                ))
            }
            WalkEvent::Enter(NodeOrToken::Token(token)) => {
                let kind: LuaTokenKind = token.kind().into();
                if matches!(
                    kind,
                    LuaTokenKind::TkWhitespace
                        | LuaTokenKind::TkEndOfLine
                        | LuaTokenKind::TkShortComment
                        | LuaTokenKind::TkLongComment
                ) {
                    continue;
                }
                if let Some(tag) = token_tag(kind) {
                    Some(FunctionShapeEvent::Token(tag))
                } else {
                    return unsupported(bytes, stop);
                }
            }
            WalkEvent::Leave(NodeOrToken::Token(_)) => None,
        };
        if let Some(event) = normalized {
            bytes.charge(&event, stop)?;
            if !events.is_empty() {
                bytes.charge(&0_u8, stop)?;
            } // one array separator byte
            events.push(event);
        }
    }
    if depth != 0 {
        return Err(invalid());
    }
    observation.validate(SourceFunctionKind::Closure, limits, Some(stop))?;
    Ok(observation)
}
fn unsupported(
    bytes: &mut DeclarationBudget,
    stop: &AtomicBool,
) -> EmmyMemberCallResult<SourceFunctionShape> {
    let outcome = SourceFunctionShape::Unsupported {
        reason: SourceFunctionShapeUnsupported::UnsupportedSyntax,
    };
    bytes.charge(&outcome, stop)?;
    Ok(outcome)
}
fn node_tag(kind: LuaSyntaxKind) -> Option<FunctionShapeNode> {
    use LuaSyntaxKind as K;
    Some(match kind {
        K::Block => FunctionShapeNode::Block,
        K::EmptyStat => FunctionShapeNode::EmptyStatement,
        K::LocalStat => FunctionShapeNode::LocalStatement,
        K::LocalFuncStat => FunctionShapeNode::LocalFunction,
        K::IfStat => FunctionShapeNode::IfStatement,
        K::ElseIfClauseStat => FunctionShapeNode::ElseIfClause,
        K::ElseClauseStat => FunctionShapeNode::ElseClause,
        K::WhileStat => FunctionShapeNode::WhileStatement,
        K::DoStat => FunctionShapeNode::DoStatement,
        K::ForStat => FunctionShapeNode::ForStatement,
        K::ForRangeStat => FunctionShapeNode::ForRangeStatement,
        K::RepeatStat => FunctionShapeNode::RepeatStatement,
        K::FuncStat => FunctionShapeNode::NamedFunction,
        K::LabelStat => FunctionShapeNode::LabelStatement,
        K::BreakStat => FunctionShapeNode::BreakStatement,
        K::ReturnStat => FunctionShapeNode::ReturnStatement,
        K::GotoStat => FunctionShapeNode::GotoStatement,
        K::CallExprStat => FunctionShapeNode::CallStatement,
        K::AssignStat => FunctionShapeNode::AssignStatement,
        K::ParenExpr => FunctionShapeNode::ParenthesizedExpression,
        K::LiteralExpr => FunctionShapeNode::LiteralExpression,
        K::ClosureExpr => FunctionShapeNode::ClosureExpression,
        K::UnaryExpr => FunctionShapeNode::UnaryExpression,
        K::BinaryExpr => FunctionShapeNode::BinaryExpression,
        K::TableArrayExpr => FunctionShapeNode::TableArray,
        K::TableObjectExpr => FunctionShapeNode::TableObject,
        K::TableEmptyExpr => FunctionShapeNode::TableEmpty,
        K::CallExpr
        | K::RequireCallExpr
        | K::ErrorCallExpr
        | K::AssertCallExpr
        | K::TypeCallExpr
        | K::SetmetatableCallExpr => FunctionShapeNode::CallExpression,
        K::IndexExpr => FunctionShapeNode::IndexExpression,
        K::NameExpr => FunctionShapeNode::NameExpression,
        K::LocalName => FunctionShapeNode::LocalName,
        K::ParamName => FunctionShapeNode::ParameterName,
        K::ParamList => FunctionShapeNode::ParameterList,
        K::CallArgList => FunctionShapeNode::CallArgumentList,
        K::TableFieldAssign => FunctionShapeNode::TableFieldAssignment,
        K::TableFieldValue => FunctionShapeNode::TableFieldValue,
        _ => return None,
    })
}
fn token_tag(kind: LuaTokenKind) -> Option<FunctionShapeToken> {
    use LuaTokenKind as K;
    Some(match kind {
        K::TkAnd => FunctionShapeToken::And,
        K::TkBreak => FunctionShapeToken::Break,
        K::TkDo => FunctionShapeToken::Do,
        K::TkElse => FunctionShapeToken::Else,
        K::TkElseIf => FunctionShapeToken::ElseIf,
        K::TkEnd => FunctionShapeToken::End,
        K::TkFor => FunctionShapeToken::For,
        K::TkFunction => FunctionShapeToken::Function,
        K::TkGoto => FunctionShapeToken::Goto,
        K::TkIf => FunctionShapeToken::If,
        K::TkIn => FunctionShapeToken::In,
        K::TkLocal => FunctionShapeToken::Local,
        K::TkNot => FunctionShapeToken::Not,
        K::TkOr => FunctionShapeToken::Or,
        K::TkRepeat => FunctionShapeToken::Repeat,
        K::TkReturn => FunctionShapeToken::Return,
        K::TkThen => FunctionShapeToken::Then,
        K::TkUntil => FunctionShapeToken::Until,
        K::TkWhile => FunctionShapeToken::While,
        K::TkPlus => FunctionShapeToken::Plus,
        K::TkMinus => FunctionShapeToken::Minus,
        K::TkMul => FunctionShapeToken::Multiply,
        K::TkDiv => FunctionShapeToken::Divide,
        K::TkIDiv => FunctionShapeToken::IntegerDivide,
        K::TkDot => FunctionShapeToken::Dot,
        K::TkConcat => FunctionShapeToken::Concatenate,
        K::TkDots => FunctionShapeToken::Vararg,
        K::TkComma => FunctionShapeToken::Comma,
        K::TkAssign => FunctionShapeToken::Assign,
        K::TkEq => FunctionShapeToken::Equal,
        K::TkGe => FunctionShapeToken::GreaterEqual,
        K::TkLe => FunctionShapeToken::LessEqual,
        K::TkNe => FunctionShapeToken::NotEqual,
        K::TkShl => FunctionShapeToken::ShiftLeft,
        K::TkShr => FunctionShapeToken::ShiftRight,
        K::TkLt => FunctionShapeToken::Less,
        K::TkGt => FunctionShapeToken::Greater,
        K::TkMod => FunctionShapeToken::Modulo,
        K::TkPow => FunctionShapeToken::Power,
        K::TkLen => FunctionShapeToken::Length,
        K::TkBitAnd => FunctionShapeToken::BitAnd,
        K::TkBitOr => FunctionShapeToken::BitOr,
        K::TkBitXor => FunctionShapeToken::BitXor,
        K::TkColon => FunctionShapeToken::Colon,
        K::TkDbColon => FunctionShapeToken::DoubleColon,
        K::TkSemicolon => FunctionShapeToken::Semicolon,
        K::TkLeftBracket => FunctionShapeToken::LeftBracket,
        K::TkRightBracket => FunctionShapeToken::RightBracket,
        K::TkLeftParen => FunctionShapeToken::LeftParen,
        K::TkRightParen => FunctionShapeToken::RightParen,
        K::TkLeftBrace => FunctionShapeToken::LeftBrace,
        K::TkRightBrace => FunctionShapeToken::RightBrace,
        K::TkName => FunctionShapeToken::Name,
        K::TkInt | K::TkFloat => FunctionShapeToken::Number,
        K::TkString | K::TkLongString => FunctionShapeToken::String,
        K::TkTrue | K::TkFalse => FunctionShapeToken::Boolean,
        K::TkNil => FunctionShapeToken::Nil,
        _ => return None,
    })
}
