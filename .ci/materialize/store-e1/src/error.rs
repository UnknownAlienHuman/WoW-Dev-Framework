use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreErrorCode {
    InvalidIdentity,
    InvalidPath,
    InputLimitExceeded,
    IoFailed,
    ObjectMissing,
    ObjectCorrupt,
    SnapshotCorrupt,
    RefCorrupt,
    RefConflict,
    LockFailed,
    CanonicalizationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    code: StoreErrorCode,
    message: Box<str>,
    subject: Option<Box<str>>,
}

impl StoreError {
    pub(crate) fn new(
        code: StoreErrorCode,
        message: impl Into<Box<str>>,
        subject: Option<&str>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            subject: subject.map(Into::into),
        }
    }

    #[must_use]
    pub const fn code(&self) -> StoreErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.subject() {
            Some(subject) => write!(formatter, "{} ({subject})", self.message),
            None => formatter.write_str(&self.message),
        }
    }
}

impl std::error::Error for StoreError {}

pub type StoreResult<T> = Result<T, StoreError>;
