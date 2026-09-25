//! Explicit local read-back of source handles from an admitted graph manifest.
//! One file is read and hashed once; source text is emitted only after the whole
//! file matches. This never changes retained evidence or authenticates an origin.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use wow_core::{
    ContentDigest, GenerationContextId, SourceContent, SourceHandle, SourceSpan, SourceSpanKind,
    StableHandleId,
};
use wow_graph::GraphEvidenceCatalog;

use super::ProjectGraphFile;
use crate::disk::{ProjectDiskFile, ProjectInputDirectory};
use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

/// Can only be obtained by complete retained project/graph evidence admission.
/// It is not an inventory supplied by the source-read request.
#[derive(Debug)]
pub struct RetainedProjectSourceManifest {
    context_id: GenerationContextId,
    catalog_digest: Box<str>,
    files: BTreeMap<String, ProjectGraphFile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSourceReadLimits {
    pub max_files: u32,
    pub max_source_handles: u32,
    pub max_file_bytes: u32,
    pub max_read_bytes: u32,
    pub max_excerpt_bytes: u32,
    pub max_total_excerpt_bytes: u32,
}
impl Default for ProjectSourceReadLimits {
    fn default() -> Self {
        Self {
            max_files: 16,
            max_source_handles: 128,
            max_file_bytes: 1024 * 1024,
            max_read_bytes: 8 * 1024 * 1024,
            max_excerpt_bytes: 16 * 1024,
            max_total_excerpt_bytes: 256 * 1024,
        }
    }
}
impl ProjectSourceReadLimits {
    pub fn validate(self) -> ProjectResult<()> {
        if !(1..=256).contains(&self.max_files)
            || !(1..=4096).contains(&self.max_source_handles)
            || !(1..=16 * 1024 * 1024).contains(&self.max_file_bytes)
            || !(1..=64 * 1024 * 1024).contains(&self.max_read_bytes)
            || !(1..=64 * 1024).contains(&self.max_excerpt_bytes)
            || !(1..=4 * 1024 * 1024).contains(&self.max_total_excerpt_bytes)
        {
            return Err(failure(ProjectErrorCode::InvalidBudgetPolicy));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSourceFileStatus {
    Verified,
    ContentMismatch,
    UnavailableOrUnsafe,
    ChangedDuringRead,
    UnsupportedPath,
    NotUtf8,
    FileByteLimit,
    ReadByteLimit,
    FileCountLimit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSourceExcerptStatus {
    Returned,
    SourceNotVerified,
    UnknownSpan,
    InvalidSpan,
    ExcerptByteLimit,
    TotalExcerptByteLimit,
    OutputByteLimit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSourceReadTruncation {
    Files,
    SourceHandles,
    FileBytes,
    ReadBytes,
    ExcerptBytes,
    TotalExcerptBytes,
    OutputBytes,
}
#[derive(Debug, Serialize)]
pub struct ProjectSourceExcerpt {
    source_handle_id: StableHandleId,
    span: SourceSpan,
    status: ProjectSourceExcerptStatus,
    span_verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct ProjectSourceFileRead {
    path: String,
    expected_content_digest: ContentDigest<SourceContent>,
    expected_byte_length: u64,
    status: ProjectSourceFileStatus,
    excerpts: Vec<ProjectSourceExcerpt>,
}
#[derive(Debug, Serialize)]
pub struct ProjectSourceReadReport {
    schema: &'static str,
    context_id: GenerationContextId,
    evidence_catalog_digest: Box<str>,
    limits: ProjectSourceReadLimits,
    requested_source_handles: usize,
    omitted_source_handles: usize,
    attempted_files: u32,
    /// Upper bound reserved before each read, including one overflow-detection
    /// byte. Not a claim about actual OS I/O or successful content bytes.
    reserved_read_bytes: u64,
    returned_excerpt_bytes: u64,
    all_requested_files_verified: bool,
    all_requested_spans_verified: bool,
    files: Vec<ProjectSourceFileRead>,
    truncations: BTreeSet<ProjectSourceReadTruncation>,
    origin_authenticated: bool,
    filesystem_snapshot_acquired: bool,
}
impl ProjectSourceReadReport {
    pub fn truncations(&self) -> &BTreeSet<ProjectSourceReadTruncation> {
        &self.truncations
    }
}

impl RetainedProjectSourceManifest {
    pub(super) fn from_admitted(
        context_id: GenerationContextId,
        catalog_digest: Box<str>,
        files: Vec<ProjectGraphFile>,
    ) -> Self {
        Self {
            context_id,
            catalog_digest,
            files: files
                .into_iter()
                .map(|file| (file.path.clone(), file))
                .collect(),
        }
    }

    /// Read only the selected *admitted* source handles. Every whole-file digest
    /// is checked before any excerpt is retained; mismatch bytes never escape.
    /// No discovery, Library fallback, source execution or historical checkout.
    pub fn read_sources(
        &self,
        directory: &ProjectInputDirectory,
        catalog: &GraphEvidenceCatalog,
        handles: &[&SourceHandle],
        limits: ProjectSourceReadLimits,
        max_output_bytes: usize,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectSourceReadReport> {
        crate::disk::checkpoint(stop)?;
        limits.validate()?;
        if self.context_id != catalog.context().context_id()
            || self.catalog_digest.as_ref() != catalog.digest()
            || handles.len() > 65_536
        {
            return Err(failure(ProjectErrorCode::SourceRegistryInvalid));
        }
        if max_output_bytes > 8 * 1024 * 1024 {
            return Err(failure(ProjectErrorCode::InvalidBudgetPolicy));
        }
        // Validate the full selection before opening any descendant, including
        // handles which will not fit the response's record budget.
        let mut seen = BTreeSet::new();
        let mut paths = BTreeMap::new();
        for handle in handles {
            crate::disk::checkpoint(stop)?;
            let file = self
                .files
                .get(handle.path().as_str())
                .ok_or_else(|| failure(ProjectErrorCode::UndeclaredFile))?;
            let path = handle.path().as_str();
            if paths
                .insert(path.to_lowercase(), path)
                .is_some_and(|previous| previous != path)
            {
                return Err(failure(ProjectErrorCode::FileCaseCollision));
            }
            if !seen.insert(handle.handle_id())
                || catalog.source_handle(&handle.handle_id()) != Some(*handle)
                || *handle.content_digest() != file.content_digest
            {
                return Err(failure(ProjectErrorCode::SourceHandleInvalid));
            }
        }
        let mut ordered = handles.to_vec();
        ordered.sort_by(|left, right| {
            left.path()
                .cmp(right.path())
                .then_with(|| left.handle_id().cmp(&right.handle_id()))
        });
        let mut groups: BTreeMap<&str, Vec<&SourceHandle>> = BTreeMap::new();
        for handle in ordered.iter().take(limits.max_source_handles as usize) {
            groups
                .entry(handle.path().as_str())
                .or_default()
                .push(handle);
        }
        let mut output = ProjectSourceReadReport {
            schema: "wow-project/retained-source-read/1",
            context_id: self.context_id,
            evidence_catalog_digest: self.catalog_digest.clone(),
            limits,
            requested_source_handles: handles.len(),
            omitted_source_handles: handles.len(),
            attempted_files: 0,
            reserved_read_bytes: 0,
            returned_excerpt_bytes: 0,
            all_requested_files_verified: false,
            all_requested_spans_verified: false,
            files: Vec::new(),
            truncations: BTreeSet::new(),
            origin_authenticated: false,
            filesystem_snapshot_acquired: false,
        };
        if handles.len() > limits.max_source_handles as usize {
            output
                .truncations
                .insert(ProjectSourceReadTruncation::SourceHandles);
        }
        // Fixed reserve covers growing counters and the complete truncation
        // vocabulary, independently of the selected files and excerpt bytes.
        let mut remaining = max_output_bytes
            .checked_sub(encoded_len(&output, max_output_bytes)?)
            .and_then(|bytes| bytes.checked_sub(1024))
            .ok_or_else(|| failure(ProjectErrorCode::SourceBudgetExceeded))?;
        for (path, group) in groups {
            crate::disk::checkpoint(stop)?;
            let file = self
                .files
                .get(path)
                .ok_or_else(|| failure(ProjectErrorCode::UndeclaredFile))?;
            let mut record = ProjectSourceFileRead {
                path: file.path.clone(),
                expected_content_digest: file.content_digest,
                expected_byte_length: file.byte_length,
                status: ProjectSourceFileStatus::UnavailableOrUnsafe,
                excerpts: group
                    .iter()
                    .map(|handle| ProjectSourceExcerpt {
                        source_handle_id: handle.handle_id(),
                        span: handle.span(),
                        status: ProjectSourceExcerptStatus::SourceNotVerified,
                        span_verified: false,
                        text: None,
                    })
                    .collect(),
            };
            // Count metadata before reading. Reserve status-name variation and
            // array commas, then charge escaped JSON text separately in O(bytes).
            let metadata = match encoded_len(&record, remaining) {
                Ok(bytes) => bytes.saturating_add(128 + group.len() * 32),
                Err(_) => {
                    output
                        .truncations
                        .insert(ProjectSourceReadTruncation::OutputBytes);
                    break;
                }
            };
            if metadata > remaining {
                output
                    .truncations
                    .insert(ProjectSourceReadTruncation::OutputBytes);
                break;
            }
            remaining -= metadata;
            let source = read_file(directory, file, limits, &mut output, stop);
            match source {
                Ok(text) => {
                    record.status = ProjectSourceFileStatus::Verified;
                    for excerpt in &mut record.excerpts {
                        crate::disk::checkpoint(stop)?;
                        fill_excerpt(excerpt, &text, limits, &mut output, &mut remaining)?;
                    }
                }
                Err(ReadFailure::Outcome(status)) => record.status = status,
                Err(ReadFailure::Fatal(error)) => return Err(error),
            }
            output.omitted_source_handles -= group.len();
            output.files.push(record);
        }
        output.all_requested_files_verified = !handles.is_empty()
            && output.omitted_source_handles == 0
            && output
                .files
                .iter()
                .all(|file| file.status == ProjectSourceFileStatus::Verified);
        output.all_requested_spans_verified = output.all_requested_files_verified
            && output
                .files
                .iter()
                .all(|file| file.excerpts.iter().all(|excerpt| excerpt.span_verified));
        crate::disk::checkpoint(stop)?;
        encoded_len(&output, max_output_bytes)?;
        Ok(output)
    }
}

enum ReadFailure {
    Outcome(ProjectSourceFileStatus),
    Fatal(ProjectError),
}
fn read_file(
    directory: &ProjectInputDirectory,
    file: &ProjectGraphFile,
    limits: ProjectSourceReadLimits,
    output: &mut ProjectSourceReadReport,
    stop: &AtomicBool,
) -> Result<String, ReadFailure> {
    use ProjectSourceFileStatus as Status;
    use ProjectSourceReadTruncation as Truncation;
    let extension = file.path.rsplit('.').next().unwrap_or("");
    if crate::disk::validate_path(&file.path).is_err()
        || !["lua", "xml", "toc"]
            .iter()
            .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        return Err(ReadFailure::Outcome(Status::UnsupportedPath));
    }
    if file.byte_length > u64::from(limits.max_file_bytes) {
        output.truncations.insert(Truncation::FileBytes);
        return Err(ReadFailure::Outcome(Status::FileByteLimit));
    }
    if output.attempted_files == limits.max_files {
        output.truncations.insert(Truncation::Files);
        return Err(ReadFailure::Outcome(Status::FileCountLimit));
    }
    let reserved = file.byte_length + 1;
    if output.reserved_read_bytes + reserved > u64::from(limits.max_read_bytes) {
        output.truncations.insert(Truncation::ReadBytes);
        return Err(ReadFailure::Outcome(Status::ReadByteLimit));
    }
    output.attempted_files += 1;
    output.reserved_read_bytes += reserved;
    let selected =
        ProjectDiskFile::new(&file.path).with_identity(file.content_digest, file.byte_length);
    let bytes = directory
        .read(&selected, file.byte_length as usize, stop)
        .map_err(|error| {
            ReadFailure::Outcome(match error.code() {
                ProjectErrorCode::FileDigestMismatch | ProjectErrorCode::FileLengthMismatch => {
                    Status::ContentMismatch
                }
                ProjectErrorCode::SourceChangedDuringRead => Status::ChangedDuringRead,
                ProjectErrorCode::InvalidFilePath => Status::UnsupportedPath,
                ProjectErrorCode::SourceBudgetExceeded => {
                    output.truncations.insert(Truncation::ReadBytes);
                    Status::ReadByteLimit
                }
                ProjectErrorCode::SourceReadCancelled | ProjectErrorCode::AnalysisCancelled => {
                    return ReadFailure::Fatal(error);
                }
                _ => Status::UnavailableOrUnsafe,
            })
        })?;
    String::from_utf8(bytes).map_err(|_| ReadFailure::Outcome(Status::NotUtf8))
}
fn fill_excerpt(
    excerpt: &mut ProjectSourceExcerpt,
    text: &str,
    limits: ProjectSourceReadLimits,
    output: &mut ProjectSourceReadReport,
    remaining: &mut usize,
) -> ProjectResult<()> {
    use ProjectSourceExcerptStatus as Status;
    use ProjectSourceReadTruncation as Truncation;
    let range = match excerpt.span.kind() {
        SourceSpanKind::WholeFile => Some((0, text.len())),
        SourceSpanKind::ByteRange => excerpt
            .span
            .byte_start()
            .zip(excerpt.span.byte_end())
            .and_then(|(start, end)| {
                Some((usize::try_from(start).ok()?, usize::try_from(end).ok()?))
            }),
        SourceSpanKind::Unknown => {
            excerpt.status = Status::UnknownSpan;
            return Ok(());
        }
    };
    let Some(fragment) = range.and_then(|(start, end)| text.get(start..end)) else {
        excerpt.status = Status::InvalidSpan;
        return Ok(());
    };
    excerpt.span_verified = true;
    if fragment.len() > limits.max_excerpt_bytes as usize {
        excerpt.status = Status::ExcerptByteLimit;
        output.truncations.insert(Truncation::ExcerptBytes);
        return Ok(());
    }
    if output.returned_excerpt_bytes + fragment.len() as u64
        > u64::from(limits.max_total_excerpt_bytes)
    {
        excerpt.status = Status::TotalExcerptByteLimit;
        output.truncations.insert(Truncation::TotalExcerptBytes);
        return Ok(());
    }
    let encoded = encoded_len(&fragment, *remaining)
        .ok()
        .and_then(|n| n.checked_add(8));
    let Some(encoded) = encoded.filter(|n| *n <= *remaining) else {
        excerpt.status = Status::OutputByteLimit;
        output.truncations.insert(Truncation::OutputBytes);
        return Ok(());
    };
    *remaining -= encoded;
    output.returned_excerpt_bytes += fragment.len() as u64;
    excerpt.status = Status::Returned;
    excerpt.text = Some(fragment.to_owned());
    Ok(())
}
fn encoded_len(value: &impl Serialize, limit: usize) -> ProjectResult<usize> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(bytes.len())
                .filter(|n| *n <= self.limit)
                .ok_or_else(|| std::io::Error::other("source read output limit"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value)
        .map_err(|_| failure(ProjectErrorCode::SourceBudgetExceeded))?;
    Ok(counter.bytes)
}
fn failure(code: ProjectErrorCode) -> ProjectError {
    ProjectError::new(code, ProjectPhase::View, "retained source read failed")
}
