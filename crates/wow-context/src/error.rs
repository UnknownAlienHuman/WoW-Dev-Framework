use std::fmt;

use serde::Serialize;

pub type ContextResult<T> = Result<T, ContextError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextErrorCode {
    InvalidBudget,
    UniverseMismatch,
    RootInvalid,
    MissingRequiredCustody,
    SupportMismatch,
    MinimumRequiredContentExceedsBudget,
    Cancelled,
    NativeOwnerRefused,
    EncodingInvalid,
}

/// Diagnostics contain no source-controlled strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ContextError {
    code: ContextErrorCode,
}
impl ContextError {
    pub(crate) const fn new(code: ContextErrorCode) -> Self {
        Self { code }
    }
    #[must_use]
    pub const fn code(self) -> ContextErrorCode {
        self.code
    }
}
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "context owner refused ({:?})", self.code)
    }
}
impl std::error::Error for ContextError {}

pub(crate) fn graph_error(error: wow_graph::GraphError) -> ContextError {
    ContextError::new(match error.code() {
        wow_graph::GraphErrorCode::Cancelled => ContextErrorCode::Cancelled,
        wow_graph::GraphErrorCode::BudgetExceeded => {
            ContextErrorCode::MinimumRequiredContentExceedsBudget
        }
        _ => ContextErrorCode::NativeOwnerRefused,
    })
}
