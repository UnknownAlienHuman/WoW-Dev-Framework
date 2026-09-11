use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphErrorCode {
    IdentifierInvalid,
    LimitsInvalid,
    BudgetExceeded,
    NodeDuplicate,
    EdgeDuplicate,
    CoverageDuplicate,
    EndpointMissing,
    GenerationMismatch,
    UniverseMismatch,
    SelfEdgeInvalid,
    EvidenceInvalid,
    QueryInvalid,
    SnapshotInvalid,
    SnapshotIdentityMismatch,
    PublicationKeyInvalid,
    ArtifactKindMismatch,
    ArtifactSchemaMismatch,
    ArtifactDecodeFailed,
    StoreFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphError {
    code: GraphErrorCode,
    message: Box<str>,
}

impl GraphError {
    pub(crate) fn new(code: GraphErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> GraphErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GraphError {}

impl From<wow_store::StoreError> for GraphError {
    fn from(source: wow_store::StoreError) -> Self {
        Self::new(GraphErrorCode::StoreFailure, source.message())
    }
}

pub type GraphResult<T> = Result<T, GraphError>;
