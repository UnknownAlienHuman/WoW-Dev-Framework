use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    canonical_json, ContentDigest, CoreError, CoreErrorCode, CoreResult, EntityKey,
    ProfileId, ReferenceGenerationId, RepositoryId, RevisionId, StableHandleId,
};

const MAX_SOURCE_PATH_LEN: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ByteSpan {
    pub start: u64,
    pub end: u64,
}

impl ByteSpan {
    pub fn new(start: u64, end: u64) -> CoreResult<Self> {
        if start > end {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "build_source_handle",
                "byte span start exceeds end",
            ));
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LinePosition {
    pub line: u32,
    pub column: u32,
}

impl LinePosition {
    pub fn new(line: u32, column: u32) -> CoreResult<Self> {
        if line == 0 {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "build_source_handle",
                "line numbers are one-based",
            ));
        }
        Ok(Self { line, column })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineSpan {
    pub start: LinePosition,
    pub end: LinePosition,
}

impl LineSpan {
    pub fn new(start: LinePosition, end: LinePosition) -> CoreResult<Self> {
        if start > end {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "build_source_handle",
                "line span start exceeds end",
            ));
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceOwner {
    Repository {
        repository: RepositoryId,
        revision: RevisionId,
    },
    ReferencePack {
        profile: ProfileId,
        reference_generation: ReferenceGenerationId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct SourceHandle {
    id: StableHandleId,
    owner: SourceOwner,
    path: String,
    byte_span: Option<ByteSpan>,
    line_span: Option<LineSpan>,
    digest: ContentDigest,
    symbol: Option<EntityKey>,
}

#[derive(Debug, Clone)]
pub struct SourceHandleInput {
    pub owner: SourceOwner,
    pub path: String,
    pub byte_span: Option<ByteSpan>,
    pub line_span: Option<LineSpan>,
    pub digest: ContentDigest,
    pub symbol: Option<EntityKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceHandleIdentity {
    owner: SourceOwner,
    path: String,
    byte_span: Option<ByteSpan>,
    line_span: Option<LineSpan>,
    digest: ContentDigest,
    symbol: Option<EntityKey>,
}

#[derive(Debug, Deserialize)]
struct SourceHandleWire {
    id: StableHandleId,
    owner: SourceOwner,
    path: String,
    byte_span: Option<ByteSpan>,
    line_span: Option<LineSpan>,
    digest: ContentDigest,
    symbol: Option<EntityKey>,
}

pub fn normalize_source_path(path: impl AsRef<str>) -> CoreResult<String> {
    let path = path.as_ref().replace('\\', "/");
    if path.is_empty()
        || path.len() > MAX_SOURCE_PATH_LEN
        || path.starts_with('/')
        || path.starts_with("//")
        || (path.len() >= 2
            && path.as_bytes()[0].is_ascii_alphabetic()
            && path.as_bytes()[1] == b':')
    {
        return Err(CoreError::new(
            CoreErrorCode::InvalidSourceHandle,
            "normalize_source_path",
            "path must be bounded and repository-relative",
        ));
    }

    let mut normalized = Vec::new();
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "normalize_source_path",
                "parent traversal is forbidden",
            ));
        }
        if segment.chars().any(char::is_control) || segment.contains(':') {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "normalize_source_path",
                "path contains a control or device-like component",
            ));
        }
        normalized.push(segment);
    }

    if normalized.is_empty() {
        return Err(CoreError::new(
            CoreErrorCode::InvalidSourceHandle,
            "normalize_source_path",
            "path resolves to an empty value",
        ));
    }
    Ok(normalized.join("/"))
}

pub fn build_source_handle(input: SourceHandleInput) -> CoreResult<SourceHandle> {
    let path = normalize_source_path(input.path)?;
    if let Some(span) = input.byte_span {
        ByteSpan::new(span.start, span.end)?;
    }
    if let Some(span) = input.line_span {
        LinePosition::new(span.start.line, span.start.column)?;
        LinePosition::new(span.end.line, span.end.column)?;
        LineSpan::new(span.start, span.end)?;
    }
    validate_owner(&input.owner)?;

    let identity = SourceHandleIdentity {
        owner: input.owner,
        path,
        byte_span: input.byte_span,
        line_span: input.line_span,
        digest: input.digest,
        symbol: input.symbol,
    };
    let identity_json = canonical_json(&identity)?;
    let identity_digest = ContentDigest::from_bytes(identity_json.as_bytes());
    let id = StableHandleId::parse(format!(
        "source:{}",
        identity_digest
            .canonical_string()
            .trim_start_matches("sha256:")
    ))?;

    Ok(SourceHandle {
        id,
        owner: identity.owner,
        path: identity.path,
        byte_span: identity.byte_span,
        line_span: identity.line_span,
        digest: identity.digest,
        symbol: identity.symbol,
    })
}

fn validate_owner(owner: &SourceOwner) -> CoreResult<()> {
    match owner {
        SourceOwner::Repository { revision, .. } => {
            RevisionId::parse_exact(revision.as_str())?;
        }
        SourceOwner::ReferencePack { profile, .. } => {
            ProfileId::parse_exact(profile.as_str())?;
        }
    }
    Ok(())
}

pub fn verify_source_handle_content(
    handle: &SourceHandle,
    supplied_digest: ContentDigest,
) -> CoreResult<()> {
    if handle.digest == supplied_digest {
        return Ok(());
    }
    Err(CoreError::new(
        CoreErrorCode::DigestMismatch,
        "verify_source_handle_content",
        format!("expected {}, received {}", handle.digest, supplied_digest),
    ))
}

impl SourceHandle {
    pub fn validate(&self) -> CoreResult<()> {
        let rebuilt = build_source_handle(SourceHandleInput {
            owner: self.owner.clone(),
            path: self.path.clone(),
            byte_span: self.byte_span,
            line_span: self.line_span,
            digest: self.digest,
            symbol: self.symbol.clone(),
        })?;
        if rebuilt.id != self.id {
            return Err(CoreError::new(
                CoreErrorCode::InvalidSourceHandle,
                "validate_source_handle",
                "stable handle ID does not match its canonical identity",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn id(&self) -> &StableHandleId {
        &self.id
    }

    #[must_use]
    pub fn owner(&self) -> &SourceOwner {
        &self.owner
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn byte_span(&self) -> Option<ByteSpan> {
        self.byte_span
    }

    #[must_use]
    pub fn line_span(&self) -> Option<LineSpan> {
        self.line_span
    }

    #[must_use]
    pub fn digest(&self) -> ContentDigest {
        self.digest
    }

    #[must_use]
    pub fn symbol(&self) -> Option<&EntityKey> {
        self.symbol.as_ref()
    }
}

impl<'de> Deserialize<'de> for SourceHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SourceHandleWire::deserialize(deserializer)?;
        let handle = SourceHandle {
            id: wire.id,
            owner: wire.owner,
            path: wire.path,
            byte_span: wire.byte_span,
            line_span: wire.line_span,
            digest: wire.digest,
            symbol: wire.symbol,
        };
        handle.validate().map_err(D::Error::custom)?;
        Ok(handle)
    }
}
