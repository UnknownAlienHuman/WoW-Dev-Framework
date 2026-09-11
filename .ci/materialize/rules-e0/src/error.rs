use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleErrorCode {
    InvalidIdentity,
    InvalidSourceLocation,
    DuplicateFact,
    DanglingRelation,
    SourceMismatch,
    ContradictoryEvidence,
    InputLimitExceeded,
    CanonicalizationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleError {
    code: RuleErrorCode,
    message: Box<str>,
    subject_id: Option<Box<str>>,
}

impl RuleError {
    pub(crate) fn new(
        code: RuleErrorCode,
        message: impl Into<Box<str>>,
        subject_id: Option<&str>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            subject_id: subject_id.map(Into::into),
        }
    }

    #[must_use]
    pub const fn code(&self) -> RuleErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn subject_id(&self) -> Option<&str> {
        self.subject_id.as_deref()
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.subject_id() {
            Some(subject) => write!(formatter, "{} ({subject})", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for RuleError {}

pub type RuleResult<T> = Result<T, RuleError>;
