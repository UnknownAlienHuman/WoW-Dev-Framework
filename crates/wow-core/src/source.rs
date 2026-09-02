use std::fmt;
use std::str::FromStr;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::digest::{
    ContentDigest, ProjectGenerationId, ReferenceGenerationId, SourceContent, StableHandleId,
};
use crate::error::{CoreError, CoreErrorCode, CoreResult, validation_error};
use crate::ids::{EntityKey, Parsed};

const MAX_EXACT_INTEGER: u64 = 9_007_199_254_740_991;
const MAX_PATH_BYTES: usize = 16_384;
const MAX_ORIGIN_BYTES: usize = 1_024;

/// Canonical repository- or artifact-relative UTF-8 path.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedSourcePath(String);

impl NormalizedSourcePath {
    /// Normalizes a source path without touching a filesystem.
    pub fn parse(candidate: &str) -> CoreResult<Parsed<Self>> {
        const OPERATION: &str = "normalize_source_path";
        if candidate.is_empty() || candidate.len() > MAX_PATH_BYTES {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidSourcePath,
                "candidate",
            )
            .with_argument("length", candidate.len().to_string()));
        }
        if candidate.chars().any(char::is_control) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidSourcePath,
                "candidate",
            ));
        }
        if is_absolute_or_host_path(candidate) {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::AbsolutePathForbidden,
                "candidate",
            ));
        }

        let slash_normalized = candidate.replace('\\', "/");
        let mut components = Vec::new();
        for component in slash_normalized.split('/') {
            match component {
                "" | "." => {}
                ".." => {
                    return Err(validation_error(
                        OPERATION,
                        CoreErrorCode::PathEscape,
                        "candidate",
                    ));
                }
                value => components.push(value),
            }
        }
        if components.is_empty() {
            return Err(validation_error(
                OPERATION,
                CoreErrorCode::InvalidSourcePath,
                "candidate",
            ));
        }

        let canonical = components.join("/");
        Ok(Parsed::new(Self(canonical.clone()), canonical == candidate))
    }

    /// Canonical slash-separated path.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for NormalizedSourcePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("NormalizedSourcePath")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for NormalizedSourcePath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for NormalizedSourcePath {
    type Err = CoreError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed = Self::parse(input)?;
        if parsed.was_canonical() {
            Ok(parsed.into_value())
        } else {
            Err(validation_error(
                "normalize_source_path",
                CoreErrorCode::InvalidSourcePath,
                "candidate",
            )
            .with_argument("reason", "noncanonical"))
        }
    }
}

impl Serialize for NormalizedSourcePath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NormalizedSourcePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(D::Error::custom)
    }
}

/// Canonical source-span state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceSpanKind {
    Unknown,
    WholeFile,
    ByteRange,
}

/// Zero-based end-exclusive UTF-8 byte span.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    kind: SourceSpanKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    byte_end: Option<u64>,
}

impl SourceSpan {
    /// Unknown source span. It is never interpreted as whole-file proof.
    #[must_use]
    pub const fn unknown() -> Self {
        Self {
            kind: SourceSpanKind::Unknown,
            byte_start: None,
            byte_end: None,
        }
    }

    /// Whole-file span.
    #[must_use]
    pub const fn whole_file() -> Self {
        Self {
            kind: SourceSpanKind::WholeFile,
            byte_start: None,
            byte_end: None,
        }
    }

    /// Constructs a validated byte range `[start, end)`.
    pub fn byte_range(start: u64, end: u64) -> CoreResult<Self> {
        if end < start || start > MAX_EXACT_INTEGER || end > MAX_EXACT_INTEGER {
            return Err(validation_error(
                "validate_source_span",
                CoreErrorCode::InvalidSourceSpan,
                "span",
            ));
        }
        Ok(Self {
            kind: SourceSpanKind::ByteRange,
            byte_start: Some(start),
            byte_end: Some(end),
        })
    }

    /// Validates a materialized span.
    pub fn validate(&self) -> CoreResult<()> {
        match (self.kind, self.byte_start, self.byte_end) {
            (SourceSpanKind::Unknown | SourceSpanKind::WholeFile, None, None) => Ok(()),
            (SourceSpanKind::ByteRange, Some(start), Some(end))
                if start <= end && end <= MAX_EXACT_INTEGER =>
            {
                Ok(())
            }
            (SourceSpanKind::ByteRange, Some(_), Some(_)) => Err(validation_error(
                "validate_source_span",
                CoreErrorCode::InvalidSourceSpan,
                "span",
            )),
            _ => Err(validation_error(
                "validate_source_span",
                CoreErrorCode::SpanStateConflict,
                "span",
            )),
        }
    }

    /// Span kind.
    #[must_use]
    pub const fn kind(&self) -> SourceSpanKind {
        self.kind
    }

    /// Optional range start.
    #[must_use]
    pub const fn byte_start(&self) -> Option<u64> {
        self.byte_start
    }

    /// Optional range end.
    #[must_use]
    pub const fn byte_end(&self) -> Option<u64> {
        self.byte_end
    }
}

/// Source registry class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceOriginKind {
    Repository,
    ReferencePack,
    GeneratedArtifact,
    Fixture,
}

/// Immutable source artifact/span identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceHandle {
    handle_id: StableHandleId,
    origin_kind: SourceOriginKind,
    origin_id: String,
    revision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference_generation: Option<ReferenceGenerationId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project_generation: Option<ProjectGenerationId>,
    path: NormalizedSourcePath,
    span: SourceSpan,
    content_digest: ContentDigest<SourceContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entity_key: Option<EntityKey>,
}

impl SourceHandle {
    /// Stable content-derived handle ID.
    #[must_use]
    pub const fn handle_id(&self) -> StableHandleId {
        self.handle_id
    }

    /// Origin class.
    #[must_use]
    pub const fn origin_kind(&self) -> SourceOriginKind {
        self.origin_kind
    }

    /// Opaque registered origin identity.
    #[must_use]
    pub fn origin_id(&self) -> &str {
        &self.origin_id
    }

    /// Immutable revision or artifact identity.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Canonical path.
    #[must_use]
    pub const fn path(&self) -> &NormalizedSourcePath {
        &self.path
    }

    /// Canonical source span.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    /// Content digest.
    #[must_use]
    pub const fn content_digest(&self) -> &ContentDigest<SourceContent> {
        &self.content_digest
    }

    /// Compares two handles without implying lineage or replacement.
    #[must_use]
    pub fn compare(&self, other: &Self) -> SourceHandleComparison {
        if self == other {
            return SourceHandleComparison::Identical;
        }
        if self.same_except_span(other) {
            return SourceHandleComparison::SameFileDifferentSpan;
        }
        if self.origin_kind == other.origin_kind
            && self.origin_id == other.origin_id
            && self.path == other.path
            && self.revision != other.revision
        {
            return SourceHandleComparison::SameOriginPathDifferentRevision;
        }
        if self.origin_kind == other.origin_kind
            && self.origin_id == other.origin_id
            && self.revision == other.revision
            && self.path == other.path
            && self.content_digest != other.content_digest
        {
            return SourceHandleComparison::SameOriginRevisionPathDifferentContent;
        }
        SourceHandleComparison::Unrelated
    }

    fn same_except_span(&self, other: &Self) -> bool {
        self.origin_kind == other.origin_kind
            && self.origin_id == other.origin_id
            && self.revision == other.revision
            && self.reference_generation == other.reference_generation
            && self.project_generation == other.project_generation
            && self.path == other.path
            && self.content_digest == other.content_digest
            && self.entity_key == other.entity_key
            && self.span != other.span
    }
}

/// Explicit comparison category with no lineage meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceHandleComparison {
    Identical,
    SameFileDifferentSpan,
    SameOriginPathDifferentRevision,
    SameOriginRevisionPathDifferentContent,
    Unrelated,
}

/// Builder for immutable source handles.
#[derive(Debug, Clone)]
pub struct SourceHandleBuilder {
    origin_kind: SourceOriginKind,
    origin_id: String,
    revision: String,
    reference_generation: Option<ReferenceGenerationId>,
    project_generation: Option<ProjectGenerationId>,
    path_candidate: String,
    span: SourceSpan,
    content_digest: ContentDigest<SourceContent>,
    entity_key: Option<EntityKey>,
}

impl SourceHandleBuilder {
    /// Starts a source handle from mandatory fields.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        origin_kind: SourceOriginKind,
        origin_id: impl Into<String>,
        revision: impl Into<String>,
        path_candidate: impl Into<String>,
        span: SourceSpan,
        content_digest: ContentDigest<SourceContent>,
    ) -> Self {
        Self {
            origin_kind,
            origin_id: origin_id.into(),
            revision: revision.into(),
            reference_generation: None,
            project_generation: None,
            path_candidate: path_candidate.into(),
            span,
            content_digest,
            entity_key: None,
        }
    }

    /// Binds a reference generation.
    #[must_use]
    pub const fn reference_generation(mut self, generation: ReferenceGenerationId) -> Self {
        self.reference_generation = Some(generation);
        self
    }

    /// Binds a project generation.
    #[must_use]
    pub const fn project_generation(mut self, generation: ProjectGenerationId) -> Self {
        self.project_generation = Some(generation);
        self
    }

    /// Associates an exact entity key.
    #[must_use]
    pub fn entity_key(mut self, entity_key: EntityKey) -> Self {
        self.entity_key = Some(entity_key);
        self
    }

    /// Validates and derives the stable handle ID.
    pub fn build(self) -> CoreResult<SourceHandle> {
        validate_origin_text(&self.origin_id, "origin_id")?;
        validate_origin_text(&self.revision, "revision")?;
        if is_floating_revision(&self.revision) {
            return Err(validation_error(
                "build_source_handle",
                CoreErrorCode::InvalidSourceHandle,
                "revision",
            ));
        }
        if self.path_candidate.ends_with('/') || self.path_candidate.ends_with('\\') {
            return Err(validation_error(
                "build_source_handle",
                CoreErrorCode::InvalidSourcePath,
                "path",
            )
            .with_argument("reason", "file_path_has_trailing_separator"));
        }
        let path = NormalizedSourcePath::parse(&self.path_candidate)?.into_value();
        self.span.validate()?;
        validate_origin_generation_matrix(
            self.origin_kind,
            self.reference_generation,
            self.project_generation,
        )?;

        #[derive(Serialize)]
        struct IdentityProjection<'a> {
            content_digest: &'a ContentDigest<SourceContent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            entity_key: Option<&'a EntityKey>,
            origin_id: &'a str,
            origin_kind: SourceOriginKind,
            path: &'a NormalizedSourcePath,
            #[serde(skip_serializing_if = "Option::is_none")]
            project_generation: Option<ProjectGenerationId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_generation: Option<ReferenceGenerationId>,
            revision: &'a str,
            span: SourceSpan,
        }

        let projection = IdentityProjection {
            content_digest: &self.content_digest,
            entity_key: self.entity_key.as_ref(),
            origin_id: &self.origin_id,
            origin_kind: self.origin_kind,
            path: &path,
            project_generation: self.project_generation,
            reference_generation: self.reference_generation,
            revision: &self.revision,
            span: self.span,
        };
        let handle_id = StableHandleId::derive(&projection)?;

        Ok(SourceHandle {
            handle_id,
            origin_kind: self.origin_kind,
            origin_id: self.origin_id,
            revision: self.revision,
            reference_generation: self.reference_generation,
            project_generation: self.project_generation,
            path,
            span: self.span,
            content_digest: self.content_digest,
            entity_key: self.entity_key,
        })
    }
}

fn validate_origin_generation_matrix(
    origin_kind: SourceOriginKind,
    reference_generation: Option<ReferenceGenerationId>,
    project_generation: Option<ProjectGenerationId>,
) -> CoreResult<()> {
    let valid = match origin_kind {
        SourceOriginKind::Repository => {
            reference_generation.is_none() && project_generation.is_none()
        }
        SourceOriginKind::ReferencePack => {
            reference_generation.is_some() && project_generation.is_none()
        }
        SourceOriginKind::GeneratedArtifact => {
            reference_generation.is_some() || project_generation.is_some()
        }
        SourceOriginKind::Fixture => true,
    };
    if valid {
        Ok(())
    } else {
        Err(validation_error(
            "build_source_handle",
            CoreErrorCode::InvalidSourceHandle,
            "origin_kind",
        ))
    }
}

fn validate_origin_text(value: &str, field: &'static str) -> CoreResult<()> {
    if value.is_empty()
        || value.len() > MAX_ORIGIN_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
        || contains_credentials(value)
    {
        Err(validation_error(
            "build_source_handle",
            CoreErrorCode::InvalidSourceHandle,
            field,
        ))
    } else {
        Ok(())
    }
}

fn contains_credentials(value: &str) -> bool {
    value
        .split_once("://")
        .and_then(|(_, authority_and_path)| authority_and_path.split('/').next())
        .is_some_and(|authority| authority.contains('@'))
}

fn is_absolute_or_host_path(candidate: &str) -> bool {
    let bytes = candidate.as_bytes();
    candidate.starts_with('/')
        || candidate.starts_with('\\')
        || candidate.contains("://")
        || bytes.get(1) == Some(&b':') && bytes.first().is_some_and(u8::is_ascii_alphabetic)
}

fn is_floating_revision(revision: &str) -> bool {
    matches!(
        revision.to_ascii_lowercase().as_str(),
        "main" | "master" | "current" | "latest" | "live" | "head" | "default" | "auto"
    )
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::digest::{ContentDigest, ProjectGenerationId, ReferenceGenerationId, SourceContent};
    use crate::ids::EntityKey;

    use super::{NormalizedSourcePath, SourceHandleBuilder, SourceOriginKind, SourceSpan};

    #[test]
    fn windows_separator_is_normalized() {
        let path = NormalizedSourcePath::parse("Core\\Init.lua");
        assert_eq!(
            path.as_ref().map(|value| value.value().as_str()),
            Ok("Core/Init.lua")
        );
        assert_eq!(path.as_ref().map(|value| value.was_canonical()), Ok(false));
    }

    #[test]
    fn traversal_is_rejected() {
        assert!(NormalizedSourcePath::parse("Core/../Init.lua").is_err());
    }

    #[test]
    fn empty_range_is_valid() {
        assert!(SourceSpan::byte_range(0, 0).is_ok());
    }

    #[test]
    fn fixture_source_handle_matches_normative_hash_vector() -> crate::CoreResult<()> {
        let reference = ReferenceGenerationId::from_str(
            "generation:reference:sha256:8e56faa8b5c7efae0e0c8468c48101ed8d2cfef206b40e6e96106993096d2786",
        )?;
        let project = ProjectGenerationId::from_str(
            "generation:project:sha256:e606bd7594eb932275741dbe51afbd46ecdf59388f370b71eaa1422dd0259463",
        )?;
        let entity = EntityKey::from_str("entity:api:C_Fixture.Missing")?;
        let handle = SourceHandleBuilder::new(
            SourceOriginKind::Fixture,
            "fixture:e0-addon",
            "fixture:e0-rev1",
            "Addon/Core.lua",
            SourceSpan::byte_range(20, 31)?,
            ContentDigest::<SourceContent>::from_bytes([2_u8; 32]),
        )
        .reference_generation(reference)
        .project_generation(project)
        .entity_key(entity)
        .build()?;

        assert_eq!(
            handle.handle_id().to_string(),
            "handle:sha256:a78c656122272558469d3b3bff87aeba8537211ee51c2bf7e0c9b24fa1502664"
        );
        Ok(())
    }
}
