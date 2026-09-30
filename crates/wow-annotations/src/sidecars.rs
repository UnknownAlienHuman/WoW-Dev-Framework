//! Canonical standalone source-map and projection-loss sidecars for native annotations.
//!
//! The native renderer remains the semantic owner. This module projects its final
//! bytes, source links, explicit issues and declared limitations into independently
//! verifiable artifacts. It does not infer runtime truth or repair omitted semantics.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::canonical_json_bytes;
use wow_reference::native_constants::ScalarError;

use crate::artifact::AnnotationArtifact;
use crate::native::{NativeLibrary, SourceLink};

pub const NATIVE_SOURCE_MAP_SCHEMA: &str = "wow-annotations/native-source-map/1";
pub const NATIVE_PROJECTION_LOSS_SCHEMA: &str = "wow-annotations/native-projection-loss/1";
const SUPPORTED_SOURCE_MAP_PROFILE: &str = "wow-native-field-maps/1";
const MAX_SIDECAR_BYTES: usize = 64 * 1024 * 1024;
const MAX_RECORDS: usize = 262_144;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeSidecarErrorCode {
    InvalidArtifact,
    InvalidIdentity,
    InvalidSourceMap,
    InvalidLossReport,
    InputLimit,
    SerializationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSidecarError {
    code: NativeSidecarErrorCode,
    message: Box<str>,
}

impl NativeSidecarError {
    fn new(code: NativeSidecarErrorCode, message: impl Into<Box<str>>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> NativeSidecarErrorCode {
        self.code
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for NativeSidecarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for NativeSidecarError {}

pub type NativeSidecarResult<T> = Result<T, NativeSidecarError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeProjectionClosure {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeLossCategory {
    UnrepresentableType,
    ConsumerSyntaxGap,
    UnknownReferenceField,
    UnsupportedReferenceFact,
    ConditionalOrRuntimeRestrictionGap,
    DocumentationSanitizedOrTruncated,
    InvalidIdentifierRendering,
    SourceConflictOrPartial,
    BudgetTruncation,
    DeferredCapability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeLossSeverity {
    Informational,
    Advisory,
    BlockingForDeclaredCapability,
    BlockingForReleaseReadyArtifact,
}

impl NativeLossSeverity {
    const fn blocks_release(self) -> bool {
        matches!(
            self,
            Self::BlockingForDeclaredCapability | Self::BlockingForReleaseReadyArtifact
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSidecarSpan {
    start: u64,
    end: u64,
}

impl NativeSidecarSpan {
    fn from_usize(start: usize, end: usize) -> NativeSidecarResult<Self> {
        let start = u64::try_from(start).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native sidecar span exceeds the reviewed integer range",
            )
        })?;
        let end = u64::try_from(end).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native sidecar span exceeds the reviewed integer range",
            )
        })?;
        if start > end {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidIdentity,
                "native sidecar span is reversed",
            ));
        }
        Ok(Self { start, end })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSidecarSource {
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<Box<str>>,
    path: Box<str>,
    sha256: Box<str>,
    span: NativeSidecarSpan,
}

impl NativeSidecarSource {
    fn from_link(link: &SourceLink) -> NativeSidecarResult<Self> {
        let source = Self {
            scope: link.scope.map(Into::into),
            path: link.path.clone().into_boxed_str(),
            sha256: link.sha256.clone().into_boxed_str(),
            span: NativeSidecarSpan::from_usize(link.span.start, link.span.end)?,
        };
        source.validate()?;
        Ok(source)
    }

    fn validate(&self) -> NativeSidecarResult<()> {
        if self.path.is_empty()
            || self.path.len() > 4096
            || self.path.starts_with('/')
            || self.path.contains('\\')
            || self
                .path
                .split('/')
                .any(|component| component.is_empty() || component == "." || component == "..")
            || !valid_sha256(&self.sha256)
            || self.span.start > self.span.end
            || self.scope.as_ref().is_some_and(|scope| {
                scope.is_empty()
                    || scope.len() > 128
                    || !scope.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_')
                    })
            })
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidIdentity,
                "native sidecar source identity is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSourceMapEntry {
    mapping_id: Box<str>,
    granularity: Box<str>,
    generated: NativeSidecarSpan,
    source: NativeSidecarSource,
}

impl NativeSourceMapEntry {
    fn build(
        granularity: &str,
        generated_start: usize,
        generated_end: usize,
        source: &SourceLink,
    ) -> NativeSidecarResult<Self> {
        if granularity.is_empty() || granularity.len() > 128 {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map granularity is invalid",
            ));
        }
        let generated = NativeSidecarSpan::from_usize(generated_start, generated_end)?;
        let source = NativeSidecarSource::from_link(source)?;
        let mapping_id = source_mapping_id(granularity, generated, &source)?;
        Ok(Self {
            mapping_id,
            granularity: granularity.into(),
            generated,
            source,
        })
    }

    fn validate(&self, file_length: u64) -> NativeSidecarResult<()> {
        self.source.validate()?;
        if self.granularity.is_empty()
            || self.granularity.len() > 128
            || self.generated.start >= self.generated.end
            || self.generated.end > file_length
            || self.mapping_id
                != source_mapping_id(&self.granularity, self.generated, &self.source)?
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map entry is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSourceMapFile {
    file_map_id: Box<str>,
    path: Box<str>,
    sha256: Box<str>,
    byte_length: u64,
    mappings: Box<[NativeSourceMapEntry]>,
}

impl NativeSourceMapFile {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    #[must_use]
    pub const fn byte_length(&self) -> u64 {
        self.byte_length
    }

    #[must_use]
    pub fn mappings(&self) -> &[NativeSourceMapEntry] {
        &self.mappings
    }

    fn validate(&self) -> NativeSidecarResult<()> {
        validate_generated_path(&self.path)?;
        if !valid_sha256(&self.sha256) || self.byte_length == 0 {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map file identity is invalid",
            ));
        }
        for mapping in &self.mappings {
            mapping.validate(self.byte_length)?;
        }
        if !strictly_sorted_unique(self.mappings.iter().map(|item| item.mapping_id.as_ref()))
            || self.file_map_id != source_file_id(self)?
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map file order or identity is invalid",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSourceMap {
    schema: Box<str>,
    source_map_id: Box<str>,
    annotation_artifact_id: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    library_schema: Box<str>,
    source_map_profile: Box<str>,
    revision: Box<str>,
    projection: Box<str>,
    coverage: NativeProjectionClosure,
    mapping_count: u64,
    unmapped_files: Box<[Box<str>]>,
    files: Box<[NativeSourceMapFile]>,
}

impl NativeSourceMap {
    fn build(
        artifact: &AnnotationArtifact,
        library: &NativeLibrary<'_>,
    ) -> NativeSidecarResult<Self> {
        if library.source_map_profile != SUPPORTED_SOURCE_MAP_PROFILE {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map profile is unsupported",
            ));
        }
        let mut files = Vec::with_capacity(library.files.len());
        let mut mapping_count = 0u64;
        let mut unmapped_files = Vec::new();
        for file in &library.files {
            validate_generated_path(&file.path)?;
            let byte_length = u64::try_from(file.text.len()).map_err(|_| {
                NativeSidecarError::new(
                    NativeSidecarErrorCode::InputLimit,
                    "native generated file length exceeds the reviewed integer range",
                )
            })?;
            if file.text.is_empty() || sha256(file.text.as_bytes()) != file.sha256 {
                return Err(NativeSidecarError::new(
                    NativeSidecarErrorCode::InvalidSourceMap,
                    "native generated file does not match its owner digest",
                ));
            }
            let mut mappings = file
                .mappings
                .iter()
                .map(|mapping| {
                    NativeSourceMapEntry::build(
                        mapping.granularity,
                        mapping.generated.start,
                        mapping.generated.end,
                        &mapping.source,
                    )
                })
                .collect::<NativeSidecarResult<Vec<_>>>()?;
            mappings.sort_by(|left, right| left.mapping_id.cmp(&right.mapping_id));
            if mappings.is_empty() {
                unmapped_files.push(file.path.clone().into_boxed_str());
            }
            mapping_count = mapping_count
                .checked_add(u64::try_from(mappings.len()).map_err(|_| {
                    NativeSidecarError::new(
                        NativeSidecarErrorCode::InputLimit,
                        "native source-map entry count exceeds the reviewed integer range",
                    )
                })?)
                .ok_or_else(|| {
                    NativeSidecarError::new(
                        NativeSidecarErrorCode::InputLimit,
                        "native source-map entry count overflow",
                    )
                })?;
            let mut mapped_file = NativeSourceMapFile {
                file_map_id: "pending".into(),
                path: file.path.clone().into_boxed_str(),
                sha256: file.sha256.clone().into_boxed_str(),
                byte_length,
                mappings: mappings.into_boxed_slice(),
            };
            mapped_file.file_map_id = source_file_id(&mapped_file)?;
            files.push(mapped_file);
        }
        let mapping_count_within_bounds =
            usize::try_from(mapping_count).is_ok_and(|count| count <= MAX_RECORDS);
        if files.is_empty() || files.len() > MAX_RECORDS || !mapping_count_within_bounds {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native source-map inventory exceeds the reviewed bounds",
            ));
        }
        files.sort_by(|left, right| left.path.cmp(&right.path));
        unmapped_files.sort();
        let coverage = NativeProjectionClosure::Partial;
        let mut source_map = Self {
            schema: NATIVE_SOURCE_MAP_SCHEMA.into(),
            source_map_id: "pending".into(),
            annotation_artifact_id: artifact.artifact_id().into(),
            producer_id: artifact.producer_id().into(),
            producer_version: artifact.producer_version().into(),
            profile_id: artifact.profile_id().into(),
            source_generation_id: artifact.source_generation_id().into(),
            library_schema: library.schema.into(),
            source_map_profile: library.source_map_profile.into(),
            revision: library.revision.into(),
            projection: library.projection.into(),
            coverage,
            mapping_count,
            unmapped_files: unmapped_files.into_boxed_slice(),
            files: files.into_boxed_slice(),
        };
        source_map.source_map_id = source_map_id(&source_map)?;
        source_map.validate()?;
        Ok(source_map)
    }

    pub fn from_canonical_slice(bytes: &[u8]) -> NativeSidecarResult<Self> {
        let value: Self = serde_json::from_slice(bytes).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map bytes failed strict decoding",
            )
        })?;
        value.validate()?;
        if value.canonical_bytes()?.as_ref() != bytes {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map bytes are not canonical",
            ));
        }
        Ok(value)
    }

    pub fn validate(&self) -> NativeSidecarResult<()> {
        if self.schema.as_ref() != NATIVE_SOURCE_MAP_SCHEMA
            || self.source_map_profile.as_ref() != SUPPORTED_SOURCE_MAP_PROFILE
            || !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.producer_id)
            || !valid_component(&self.producer_version)
            || !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
            || !valid_component(&self.library_schema)
            || !valid_component(&self.revision)
            || !matches!(
                self.projection.as_ref(),
                "projected_with_sidecars" | "partial"
            )
            || self.files.is_empty()
            || self.files.len() > MAX_RECORDS
            || !usize::try_from(self.mapping_count).is_ok_and(|count| count <= MAX_RECORDS)
            || self.coverage != NativeProjectionClosure::Partial
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map header or bounds are invalid",
            ));
        }
        for file in &self.files {
            file.validate()?;
        }
        if !strictly_sorted_unique(self.files.iter().map(|item| item.path.as_ref()))
            || !strictly_sorted_unique(self.unmapped_files.iter().map(|item| item.as_ref()))
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map file inventory is not canonical",
            ));
        }
        let expected_unmapped = self
            .files
            .iter()
            .filter(|file| file.mappings.is_empty())
            .map(|file| file.path.as_ref())
            .collect::<Vec<_>>();
        let observed_unmapped = self
            .unmapped_files
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        let expected_count = self.files.iter().try_fold(0u64, |total, file| {
            total.checked_add(u64::try_from(file.mappings.len()).ok()?)
        });
        if expected_unmapped != observed_unmapped
            || expected_count != Some(self.mapping_count)
            || self.source_map_id != source_map_id(self)?
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidSourceMap,
                "native source-map closure or identity does not match",
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> NativeSidecarResult<Box<[u8]>> {
        let bytes = canonical_json_bytes(self).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::SerializationFailed,
                "native source-map cannot be canonicalized",
            )
        })?;
        if bytes.len() > MAX_SIDECAR_BYTES {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native source-map exceeds the reviewed byte budget",
            ));
        }
        Ok(bytes.into_boxed_slice())
    }

    #[must_use]
    pub fn source_map_id(&self) -> &str {
        &self.source_map_id
    }

    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub const fn coverage(&self) -> NativeProjectionClosure {
        self.coverage
    }

    #[must_use]
    pub fn files(&self) -> &[NativeSourceMapFile] {
        &self.files
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectionLossRecord {
    loss_id: Box<str>,
    category: NativeLossCategory,
    severity: NativeLossSeverity,
    code: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<Box<str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<NativeSidecarSource>,
}

impl NativeProjectionLossRecord {
    fn build(
        category: NativeLossCategory,
        severity: NativeLossSeverity,
        code: impl Into<Box<str>>,
        subject: Option<Box<str>>,
        source: Option<NativeSidecarSource>,
    ) -> NativeSidecarResult<Self> {
        let code = code.into();
        if code.is_empty()
            || code.len() > 512
            || subject.as_ref().is_some_and(|item| item.len() > 8192)
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss record exceeds the reviewed bounds",
            ));
        }
        let loss_id = projection_loss_id(
            category,
            severity,
            &code,
            subject.as_deref(),
            source.as_ref(),
        )?;
        Ok(Self {
            loss_id,
            category,
            severity,
            code,
            subject,
            source,
        })
    }

    fn validate(&self) -> NativeSidecarResult<()> {
        if self.code.is_empty()
            || self.code.len() > 512
            || self.subject.as_ref().is_some_and(|item| item.len() > 8192)
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss record is invalid",
            ));
        }
        if let Some(source) = &self.source {
            source.validate()?;
        }
        if self.loss_id
            != projection_loss_id(
                self.category,
                self.severity,
                &self.code,
                self.subject.as_deref(),
                self.source.as_ref(),
            )?
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss identity does not match",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProjectionLossReport {
    schema: Box<str>,
    report_id: Box<str>,
    annotation_artifact_id: Box<str>,
    producer_id: Box<str>,
    producer_version: Box<str>,
    profile_id: Box<str>,
    source_generation_id: Box<str>,
    library_schema: Box<str>,
    revision: Box<str>,
    projection: Box<str>,
    disclosure: NativeProjectionClosure,
    blocking_for_release_ready: u64,
    records: Box<[NativeProjectionLossRecord]>,
}

impl NativeProjectionLossReport {
    fn build(
        artifact: &AnnotationArtifact,
        library: &NativeLibrary<'_>,
    ) -> NativeSidecarResult<Self> {
        let mut records = Vec::new();
        for issue in &library.issues {
            let (category, severity) = classify_issue(&issue.code);
            records.push(NativeProjectionLossRecord::build(
                category,
                severity,
                issue.code.clone().into_boxed_str(),
                None,
                Some(NativeSidecarSource::from_link(&issue.source)?),
            )?);
        }
        for sidecar in &library.metadata_sidecars {
            records.push(NativeProjectionLossRecord::build(
                NativeLossCategory::UnknownReferenceField,
                NativeLossSeverity::Advisory,
                "metadata_sidecar",
                Some(sidecar.field.clone().into_boxed_str()),
                Some(NativeSidecarSource::from_link(&sidecar.source)?),
            )?);
        }
        for resolution in &library.scalar_resolutions {
            if let Some(error) = resolution.result.as_ref().err().copied() {
                records.push(NativeProjectionLossRecord::build(
                    scalar_category(error),
                    NativeLossSeverity::BlockingForDeclaredCapability,
                    scalar_code(error),
                    None,
                    Some(NativeSidecarSource::from_link(&resolution.source)?),
                )?);
            }
        }
        for projection in &library.name_projections {
            records.push(NativeProjectionLossRecord::build(
                NativeLossCategory::InvalidIdentifierRendering,
                NativeLossSeverity::Advisory,
                projection.rule,
                Some(format!("{}=>{}", projection.original, projection.rendered).into_boxed_str()),
                Some(NativeSidecarSource::from_link(&projection.source)?),
            )?);
        }
        if library
            .corrections
            .as_ref()
            .is_some_and(|report| report.has_blockers())
        {
            records.push(NativeProjectionLossRecord::build(
                NativeLossCategory::SourceConflictOrPartial,
                NativeLossSeverity::BlockingForReleaseReadyArtifact,
                "correction_report_has_blockers",
                None,
                None,
            )?);
        }
        for (index, limitation) in library.limitations.iter().enumerate() {
            records.push(NativeProjectionLossRecord::build(
                NativeLossCategory::DeferredCapability,
                NativeLossSeverity::BlockingForReleaseReadyArtifact,
                format!("declared_limitation_{index}"),
                Some((*limitation).into()),
                None,
            )?);
        }
        if library.projection == "partial"
            && !records
                .iter()
                .any(|record| record.severity.blocks_release())
        {
            records.push(NativeProjectionLossRecord::build(
                NativeLossCategory::SourceConflictOrPartial,
                NativeLossSeverity::BlockingForReleaseReadyArtifact,
                "projection_marked_partial",
                None,
                None,
            )?);
        }
        if records.len() > MAX_RECORDS {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native projection-loss inventory exceeds the reviewed bounds",
            ));
        }
        records.sort_by(|left, right| left.loss_id.cmp(&right.loss_id));
        let blocking_for_release_ready = u64::try_from(
            records
                .iter()
                .filter(|record| record.severity.blocks_release())
                .count(),
        )
        .map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native projection-loss count exceeds the reviewed integer range",
            )
        })?;
        let mut report = Self {
            schema: NATIVE_PROJECTION_LOSS_SCHEMA.into(),
            report_id: "pending".into(),
            annotation_artifact_id: artifact.artifact_id().into(),
            producer_id: artifact.producer_id().into(),
            producer_version: artifact.producer_version().into(),
            profile_id: artifact.profile_id().into(),
            source_generation_id: artifact.source_generation_id().into(),
            library_schema: library.schema.into(),
            revision: library.revision.into(),
            projection: library.projection.into(),
            disclosure: NativeProjectionClosure::Complete,
            blocking_for_release_ready,
            records: records.into_boxed_slice(),
        };
        report.report_id = projection_loss_report_id(&report)?;
        report.validate()?;
        Ok(report)
    }

    pub fn from_canonical_slice(bytes: &[u8]) -> NativeSidecarResult<Self> {
        let value: Self = serde_json::from_slice(bytes).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss bytes failed strict decoding",
            )
        })?;
        value.validate()?;
        if value.canonical_bytes()?.as_ref() != bytes {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss bytes are not canonical",
            ));
        }
        Ok(value)
    }

    pub fn validate(&self) -> NativeSidecarResult<()> {
        if self.schema.as_ref() != NATIVE_PROJECTION_LOSS_SCHEMA
            || !valid_component(&self.annotation_artifact_id)
            || !valid_component(&self.producer_id)
            || !valid_component(&self.producer_version)
            || !valid_component(&self.profile_id)
            || !valid_component(&self.source_generation_id)
            || !valid_component(&self.library_schema)
            || !valid_component(&self.revision)
            || !matches!(
                self.projection.as_ref(),
                "projected_with_sidecars" | "partial"
            )
            || self.records.len() > MAX_RECORDS
            || self.disclosure != NativeProjectionClosure::Complete
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss header or bounds are invalid",
            ));
        }
        for record in &self.records {
            record.validate()?;
        }
        let expected_blocking = u64::try_from(
            self.records
                .iter()
                .filter(|record| record.severity.blocks_release())
                .count(),
        )
        .map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native projection-loss count exceeds the reviewed integer range",
            )
        })?;
        if !strictly_sorted_unique(self.records.iter().map(|item| item.loss_id.as_ref()))
            || self.blocking_for_release_ready != expected_blocking
            || self.report_id != projection_loss_report_id(self)?
        {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidLossReport,
                "native projection-loss closure or identity does not match",
            ));
        }
        Ok(())
    }

    pub fn canonical_bytes(&self) -> NativeSidecarResult<Box<[u8]>> {
        let bytes = canonical_json_bytes(self).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::SerializationFailed,
                "native projection-loss report cannot be canonicalized",
            )
        })?;
        if bytes.len() > MAX_SIDECAR_BYTES {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InputLimit,
                "native projection-loss report exceeds the reviewed byte budget",
            ));
        }
        Ok(bytes.into_boxed_slice())
    }

    #[must_use]
    pub fn report_id(&self) -> &str {
        &self.report_id
    }

    #[must_use]
    pub fn annotation_artifact_id(&self) -> &str {
        &self.annotation_artifact_id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }

    #[must_use]
    pub fn source_generation_id(&self) -> &str {
        &self.source_generation_id
    }

    #[must_use]
    pub const fn disclosure(&self) -> NativeProjectionClosure {
        self.disclosure
    }

    #[must_use]
    pub const fn blocking_for_release_ready(&self) -> u64 {
        self.blocking_for_release_ready
    }

    #[must_use]
    pub fn records(&self) -> &[NativeProjectionLossRecord] {
        &self.records
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeProjectionSidecars {
    source_map: NativeSourceMap,
    source_map_bytes: Box<[u8]>,
    loss_report: NativeProjectionLossReport,
    loss_report_bytes: Box<[u8]>,
}

impl NativeProjectionSidecars {
    pub fn from_library(
        artifact: &AnnotationArtifact,
        library: &NativeLibrary<'_>,
    ) -> NativeSidecarResult<Self> {
        artifact.validate().map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidArtifact,
                "annotation artifact is invalid",
            )
        })?;
        let payload = canonical_json_bytes(library).map_err(|_| {
            NativeSidecarError::new(
                NativeSidecarErrorCode::SerializationFailed,
                "native annotation library cannot be canonicalized",
            )
        })?;
        if sha256(&payload) != artifact.payload_sha256() {
            return Err(NativeSidecarError::new(
                NativeSidecarErrorCode::InvalidArtifact,
                "annotation artifact does not bind the supplied native library",
            ));
        }
        let source_map = NativeSourceMap::build(artifact, library)?;
        let source_map_bytes = source_map.canonical_bytes()?;
        let loss_report = NativeProjectionLossReport::build(artifact, library)?;
        let loss_report_bytes = loss_report.canonical_bytes()?;
        Ok(Self {
            source_map,
            source_map_bytes,
            loss_report,
            loss_report_bytes,
        })
    }

    #[must_use]
    pub fn source_map(&self) -> &NativeSourceMap {
        &self.source_map
    }

    #[must_use]
    pub fn source_map_bytes(&self) -> &[u8] {
        &self.source_map_bytes
    }

    #[must_use]
    pub fn loss_report(&self) -> &NativeProjectionLossReport {
        &self.loss_report
    }

    #[must_use]
    pub fn loss_report_bytes(&self) -> &[u8] {
        &self.loss_report_bytes
    }
}

fn source_mapping_id(
    granularity: &str,
    generated: NativeSidecarSpan,
    source: &NativeSidecarSource,
) -> NativeSidecarResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        granularity: &'a str,
        generated: NativeSidecarSpan,
        source: &'a NativeSidecarSource,
    }
    content_id(
        "native-source-mapping",
        &Identity {
            schema: "wow-annotations/native-source-mapping/1",
            granularity,
            generated,
            source,
        },
    )
}

fn source_file_id(file: &NativeSourceMapFile) -> NativeSidecarResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        path: &'a str,
        sha256: &'a str,
        byte_length: u64,
        mappings: &'a [NativeSourceMapEntry],
    }
    content_id(
        "native-source-map-file",
        &Identity {
            schema: "wow-annotations/native-source-map-file/1",
            path: &file.path,
            sha256: &file.sha256,
            byte_length: file.byte_length,
            mappings: &file.mappings,
        },
    )
}

fn source_map_id(value: &NativeSourceMap) -> NativeSidecarResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        annotation_artifact_id: &'a str,
        producer_id: &'a str,
        producer_version: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        library_schema: &'a str,
        source_map_profile: &'a str,
        revision: &'a str,
        projection: &'a str,
        coverage: NativeProjectionClosure,
        mapping_count: u64,
        unmapped_files: &'a [Box<str>],
        files: &'a [NativeSourceMapFile],
    }
    content_id(
        "native-source-map",
        &Identity {
            schema: NATIVE_SOURCE_MAP_SCHEMA,
            annotation_artifact_id: &value.annotation_artifact_id,
            producer_id: &value.producer_id,
            producer_version: &value.producer_version,
            profile_id: &value.profile_id,
            source_generation_id: &value.source_generation_id,
            library_schema: &value.library_schema,
            source_map_profile: &value.source_map_profile,
            revision: &value.revision,
            projection: &value.projection,
            coverage: value.coverage,
            mapping_count: value.mapping_count,
            unmapped_files: &value.unmapped_files,
            files: &value.files,
        },
    )
}

fn projection_loss_id(
    category: NativeLossCategory,
    severity: NativeLossSeverity,
    code: &str,
    subject: Option<&str>,
    source: Option<&NativeSidecarSource>,
) -> NativeSidecarResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        category: NativeLossCategory,
        severity: NativeLossSeverity,
        code: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        source: Option<&'a NativeSidecarSource>,
    }
    content_id(
        "native-projection-loss",
        &Identity {
            schema: "wow-annotations/native-projection-loss-record/1",
            category,
            severity,
            code,
            subject,
            source,
        },
    )
}

fn projection_loss_report_id(value: &NativeProjectionLossReport) -> NativeSidecarResult<Box<str>> {
    #[derive(Serialize)]
    struct Identity<'a> {
        schema: &'static str,
        annotation_artifact_id: &'a str,
        producer_id: &'a str,
        producer_version: &'a str,
        profile_id: &'a str,
        source_generation_id: &'a str,
        library_schema: &'a str,
        revision: &'a str,
        projection: &'a str,
        disclosure: NativeProjectionClosure,
        blocking_for_release_ready: u64,
        records: &'a [NativeProjectionLossRecord],
    }
    content_id(
        "native-projection-loss-report",
        &Identity {
            schema: NATIVE_PROJECTION_LOSS_SCHEMA,
            annotation_artifact_id: &value.annotation_artifact_id,
            producer_id: &value.producer_id,
            producer_version: &value.producer_version,
            profile_id: &value.profile_id,
            source_generation_id: &value.source_generation_id,
            library_schema: &value.library_schema,
            revision: &value.revision,
            projection: &value.projection,
            disclosure: value.disclosure,
            blocking_for_release_ready: value.blocking_for_release_ready,
            records: &value.records,
        },
    )
}

fn content_id(prefix: &str, value: &impl Serialize) -> NativeSidecarResult<Box<str>> {
    let bytes = canonical_json_bytes(value).map_err(|_| {
        NativeSidecarError::new(
            NativeSidecarErrorCode::SerializationFailed,
            "native sidecar identity cannot be canonicalized",
        )
    })?;
    Ok(format!("{prefix}:sha256:{}", hex(&Sha256::digest(bytes))).into_boxed_str())
}

fn classify_issue(code: &str) -> (NativeLossCategory, NativeLossSeverity) {
    if code == "environment_not_selected" {
        return (
            NativeLossCategory::DeferredCapability,
            NativeLossSeverity::Informational,
        );
    }
    let category = if code.contains("documentation") {
        NativeLossCategory::DocumentationSanitizedOrTruncated
    } else if code.contains("identifier") || code.contains("name_") {
        NativeLossCategory::InvalidIdentifierRendering
    } else if code.starts_with("renderer_") {
        NativeLossCategory::ConsumerSyntaxGap
    } else if code.starts_with("normalization_")
        || code.contains("conflict")
        || code.starts_with("duplicate_")
    {
        NativeLossCategory::SourceConflictOrPartial
    } else if code.contains("restriction") || code.contains("runtime") {
        NativeLossCategory::ConditionalOrRuntimeRestrictionGap
    } else if code.contains("limit") || code.contains("budget") {
        NativeLossCategory::BudgetTruncation
    } else if code.contains("type") {
        NativeLossCategory::UnrepresentableType
    } else {
        NativeLossCategory::UnsupportedReferenceFact
    };
    (
        category,
        NativeLossSeverity::BlockingForReleaseReadyArtifact,
    )
}

const fn scalar_category(error: ScalarError) -> NativeLossCategory {
    match error {
        ScalarError::Conflict | ScalarError::Cycle | ScalarError::InvalidSource => {
            NativeLossCategory::SourceConflictOrPartial
        }
        ScalarError::Limit | ScalarError::Cancelled => NativeLossCategory::BudgetTruncation,
        ScalarError::UnsupportedValue
        | ScalarError::UnresolvedReference
        | ScalarError::NonIntegralArithmetic
        | ScalarError::OutOfRange => NativeLossCategory::UnrepresentableType,
    }
}

const fn scalar_code(error: ScalarError) -> &'static str {
    match error {
        ScalarError::InvalidSource => "scalar_invalid_source",
        ScalarError::UnresolvedReference => "scalar_unresolved_reference",
        ScalarError::Conflict => "scalar_conflict",
        ScalarError::Cycle => "scalar_cycle",
        ScalarError::UnsupportedValue => "scalar_unsupported_value",
        ScalarError::NonIntegralArithmetic => "scalar_non_integral_arithmetic",
        ScalarError::OutOfRange => "scalar_out_of_range",
        ScalarError::Limit => "scalar_limit",
        ScalarError::Cancelled => "scalar_cancelled",
    }
}

fn validate_generated_path(path: &str) -> NativeSidecarResult<()> {
    if path.is_empty()
        || path.len() > 4096
        || path.starts_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(NativeSidecarError::new(
            NativeSidecarErrorCode::InvalidIdentity,
            "native generated annotation path is invalid",
        ));
    }
    Ok(())
}

fn valid_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 8192
        && !value
            .bytes()
            .any(|byte| byte == 0 || byte.is_ascii_control())
}

fn valid_sha256(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    })
}

fn strictly_sorted_unique<'a>(values: impl Iterator<Item = &'a str>) -> bool {
    let mut previous: Option<&str> = None;
    for value in values {
        if previous.is_some_and(|prior| prior >= value) {
            return false;
        }
        previous = Some(value);
    }
    true
}

fn sha256(bytes: &[u8]) -> String {
    format!("sha256:{}", hex(&Sha256::digest(bytes)))
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}
