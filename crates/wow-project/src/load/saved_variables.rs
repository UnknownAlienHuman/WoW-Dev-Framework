//! TOC declarations only. SavedVariables data files are never opened.
use super::budget;
use crate::ProjectResult;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocSavedVariableScope {
    Account,
    Character,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocSavedVariableState {
    Declared,
    UnsupportedIdentifier,
}

/// Ordinal within a single directive. The containing LoadRecord supplies its
/// exact source span, raw digest, directive order and selection context.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocSavedVariable {
    pub ordinal: u32,
    pub name: String,
    pub scope: TocSavedVariableScope,
    pub state: TocSavedVariableState,
}

pub(super) fn parse(key: &str, value: &str) -> ProjectResult<Vec<TocSavedVariable>> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    let scope = if key == "savedvariablespercharacter" {
        TocSavedVariableScope::Character
    } else {
        TocSavedVariableScope::Account
    };
    let mut declarations = Vec::new();
    for name in value.split(',').map(str::trim) {
        if declarations.len() >= 1024 || name.len() > 1024 {
            return Err(budget());
        }
        let valid = name
            .as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
            && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            && !matches!(
                name,
                "_G" | "_ENV"
                    | "and"
                    | "break"
                    | "do"
                    | "else"
                    | "elseif"
                    | "end"
                    | "false"
                    | "for"
                    | "function"
                    | "if"
                    | "in"
                    | "local"
                    | "nil"
                    | "not"
                    | "or"
                    | "repeat"
                    | "return"
                    | "then"
                    | "true"
                    | "until"
                    | "while"
            );
        declarations.push(TocSavedVariable {
            ordinal: declarations.len() as u32,
            name: name.to_owned(),
            scope,
            state: if valid {
                TocSavedVariableState::Declared
            } else {
                TocSavedVariableState::UnsupportedIdentifier
            },
        });
    }
    Ok(declarations)
}
