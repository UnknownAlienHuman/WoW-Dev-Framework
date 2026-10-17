//! Explicit session-private workspaces and immutable unsaved Lua overlays.
//! These snapshots have no published Project/Reference/Graph base authority.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};
use wow_core::{CanonicalResult, ContentDigest, SourceContent};

mod client_uri;
mod coordinates;
pub use client_uri::{OverlayResolvedDocument, OverlayUriPolicy};
mod state;
#[cfg(test)]
mod tests;
mod workspace;
pub use state::{OverlayAnalyzerInput, OverlaySnapshotLease};
pub use workspace::OverlayWorkspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayEncoding {
    Utf8,
    Utf16,
}
impl<'de> Deserialize<'de> for OverlayEncoding {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match String::deserialize(deserializer)?.as_str() {
            "utf8" => Ok(Self::Utf8),
            "utf16" => Ok(Self::Utf16),
            _ => Err(serde::de::Error::custom("unsupported overlay encoding")),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayPosition {
    pub line: u32,
    pub character: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayRange {
    pub start: OverlayPosition,
    pub end: OverlayPosition,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayEdit {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub range: Option<OverlayRange>,
    pub text: String,
}
/// Explicit document input; URI is validated against the registered root.
pub struct OverlayOpenRequest<'a> {
    pub relative_path: &'a str,
    pub uri: &'a str,
    pub client_version: u64,
    pub content: String,
}
/// An atomic sequence interpreted against each preceding intermediate state.
pub struct OverlayChangeRequest<'a> {
    pub document_id: &'a str,
    pub expected_snapshot: &'a str,
    pub expected_content: ContentDigest<SourceContent>,
    pub client_version: u64,
    pub encoding: OverlayEncoding,
    pub edits: &'a [OverlayEdit],
}
fn present<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayLimits {
    pub max_documents: usize,
    pub max_document_bytes: usize,
    pub max_private_bytes: usize,
    pub max_live_snapshots: usize,
    pub max_leases: usize,
    pub max_lines: usize,
    pub max_edits: usize,
    pub max_work: usize,
}
impl OverlayLimits {
    pub fn validate(self) -> OverlayResult<()> {
        if self.max_documents == 0
            || self.max_documents > 256
            || self.max_document_bytes == 0
            || self.max_document_bytes > 16 * 1024 * 1024
            || self.max_private_bytes < self.max_document_bytes
            || self.max_private_bytes > 256 * 1024 * 1024
            || self.max_live_snapshots == 0
            || self.max_live_snapshots > 4096
            || self.max_leases == 0
            || self.max_leases > 4096
            || self.max_lines == 0
            || self.max_lines > 1_048_576
            || self.max_edits == 0
            || self.max_edits > 4096
            || self.max_work == 0
            || self.max_work > 1024 * 1024 * 1024
        {
            return Err(error(OverlayErrorCode::Limit));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlayErrorCode {
    InvalidRoot,
    InvalidPath,
    ScopeMismatch,
    Closed,
    Busy,
    DocumentMissing,
    Conflict,
    ResynchronizationRequired,
    InvalidPosition,
    InvalidEncodingBoundary,
    InvalidRange,
    Limit,
    Cancelled,
    Analyzer,
    SourceFailure,
    Canonicalization,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverlayError {
    code: OverlayErrorCode,
}
impl OverlayError {
    pub const fn code(&self) -> OverlayErrorCode {
        self.code
    }
}
impl fmt::Display for OverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "private overlay operation refused ({:?})", self.code)
    }
}
impl std::error::Error for OverlayError {}
pub type OverlayResult<T> = Result<T, OverlayError>;
fn error(code: OverlayErrorCode) -> OverlayError {
    OverlayError { code }
}

pub(super) struct OverlayBudget<'a> {
    remaining: usize,
    stop: &'a AtomicBool,
}
impl<'a> OverlayBudget<'a> {
    pub(super) fn new(max_work: usize, stop: &'a AtomicBool) -> Self {
        Self {
            remaining: max_work,
            stop,
        }
    }
    pub(super) fn work(&mut self, amount: usize) -> OverlayResult<()> {
        if self.stop.load(Ordering::Acquire) {
            return Err(error(OverlayErrorCode::Cancelled));
        }
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or_else(|| error(OverlayErrorCode::Limit))?;
        Ok(())
    }
}
static NEXT_SCOPE: AtomicU64 = AtomicU64::new(1);
static NEXT_REGISTRATION: AtomicU64 = AtomicU64::new(1);
fn next(counter: &AtomicU64) -> OverlayResult<u64> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
            value.checked_add(1)
        })
        .map_err(|_| error(OverlayErrorCode::Limit))
}
#[derive(Debug)]
struct Scope {
    id: u64,
}
/// Opaque process-local scope; serialized IDs cannot construct this token.
#[derive(Debug, Clone)]
pub struct OverlaySessionScope(Arc<Scope>);
impl OverlaySessionScope {
    pub fn begin() -> OverlayResult<Self> {
        Ok(Self(Arc::new(Scope {
            id: next(&NEXT_SCOPE)?,
        })))
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverlayWorkspaceDescriptor {
    pub schema: Box<str>,
    pub registration_id: Box<str>,
    pub session_scope: u64,
    pub workspace_id: Box<str>,
    pub state: Box<str>,
    pub path_profile: Box<str>,
    pub base_state: Box<str>,
    pub privacy: Box<str>,
    pub limits: OverlayLimits,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverlaySnapshotDescriptor {
    pub schema: Box<str>,
    pub snapshot_id: Box<str>,
    pub registration_id: Box<str>,
    pub document_id: Box<str>,
    pub relative_path: Box<str>,
    pub client_version: u64,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: usize,
    pub line_count: usize,
    pub base_state: Box<str>,
    pub synchronization_profile: Box<str>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlaySaveState {
    ContentAcknowledgedOnly,
    DiskReconciled,
    ConflictWithDisk,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OverlaySaveObservation {
    pub snapshot_id: Box<str>,
    pub client_version: u64,
    pub state: OverlaySaveState,
}

fn digest_bytes(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}
fn identity<T: Serialize>(domain: &str, value: &T) -> OverlayResult<Box<str>> {
    let hash = wow_core::domain_separated_digest(domain, value)
        .map_err(|_| error(OverlayErrorCode::Canonicalization))?;
    Ok(format!(
        "{domain}:{}",
        ContentDigest::<CanonicalResult>::from_bytes(hash)
    )
    .into())
}
