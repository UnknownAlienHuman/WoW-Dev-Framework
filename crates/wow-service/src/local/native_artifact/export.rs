//! One explicit service operation returns bounded artifact bytes. No file writes,
//! implicit selection, analyzer execution or second operation on output failure.
use std::collections::BTreeSet;
use std::sync::atomic::AtomicBool;

use wow_core::{ContentDigest, SourceContent};
use wow_project::ProjectId;

use super::super::input::invalid;
use super::super::{LocalProjectInput, cancelled};
use super::{NATIVE_ARTIFACT_MAX_BYTES, budget, parse_digest, wire};
use crate::{OperationId, ServiceError, ServiceErrorCode, ServiceResult};

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeExportKind {
    Report,
    Artifact,
}

pub struct NativeExportRequest {
    project_id: ProjectId,
    kind: NativeExportKind,
    max_bytes: usize,
    expected_sha256: Option<ContentDigest<SourceContent>>,
    operation_id: OperationId,
}

impl NativeExportRequest {
    pub fn new(
        project_id: String,
        kind: NativeExportKind,
        max_bytes: usize,
        expected_sha256: Option<String>,
    ) -> ServiceResult<Self> {
        let project_id =
            ProjectId::new(project_id).map_err(|_| invalid("invalid native export ProjectId"))?;
        if max_bytes == 0 || max_bytes > NATIVE_ARTIFACT_MAX_BYTES {
            return Err(budget());
        }
        let expected_sha256 = expected_sha256.as_deref().map(parse_digest).transpose()?;
        let id = crate::identity::canonical_digest(
            "local-operation:sha256:",
            &(
                "native.export",
                project_id.as_str(),
                kind,
                max_bytes,
                &expected_sha256,
            ),
        )?;
        Ok(Self {
            project_id,
            kind,
            max_bytes,
            expected_sha256,
            operation_id: OperationId::new(id)?,
        })
    }
}

/// Exact service-approved bytes. Export success certifies only artifact output,
/// never clean findings, source freshness or semantic producer/consumer acceptance.
pub struct NativeExportArtifact {
    bytes: Box<[u8]>,
    sha256: String,
}

impl NativeExportArtifact {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }
}

pub fn execute_native_export(
    input: LocalProjectInput,
    request: &NativeExportRequest,
    stop: &AtomicBool,
) -> ServiceResult<NativeExportArtifact> {
    export(&input, request, stop).map_err(|error| {
        ServiceError::for_operation(error.code(), error.message(), request.operation_id.as_str())
    })
}

fn export(
    input: &LocalProjectInput,
    request: &NativeExportRequest,
    stop: &AtomicBool,
) -> ServiceResult<NativeExportArtifact> {
    cancelled(stop)?;
    if input.bundle.configuration().project_id() != &request.project_id {
        return Err(ServiceError::new(
            ServiceErrorCode::IdentityMismatch,
            "requested native export project is not configured",
        ));
    }
    let report = input.native_source_report().ok_or_else(|| {
        ServiceError::new(
            ServiceErrorCode::ComponentUnavailable,
            "configured input has no native report",
        )
    })?;
    let bytes = match request.kind {
        NativeExportKind::Report => {
            if report.len() > request.max_bytes {
                return Err(budget());
            }
            report.to_vec()
        }
        NativeExportKind::Artifact => {
            wire::reference(&input.reference)?;
            let mut files = Vec::new();
            let mut paths = BTreeSet::new();
            let mut total = 0usize;
            for library in input.bundle.libraries() {
                for file in library.files() {
                    cancelled(stop)?;
                    total = total.checked_add(file.text().len()).ok_or_else(budget)?;
                    if files.len() == 1024
                        || file.text().len() > 1024 * 1024
                        || total > 16 * 1024 * 1024
                    {
                        return Err(budget());
                    }
                    if file.path().contains('/')
                        || !file.path().ends_with(".lua")
                        || !paths.insert(file.path().to_lowercase())
                    {
                        return Err(invalid("native export Library inventory was rejected"));
                    }
                    files.push(wire::LibraryFileOutput {
                        path: file.path(),
                        text: file.text(),
                        sha256: parse_digest(file.content_sha256())?,
                        byte_length: file.byte_len(),
                    });
                }
            }
            if files.is_empty() {
                return Err(invalid("native export has no Library files"));
            }
            files.sort_by(|a, b| a.path.cmp(b.path));
            let report_text = std::str::from_utf8(report)
                .map_err(|_| invalid("native report encoding was rejected"))?;
            // Use the same binding admission as import. Do not emit a cache that
            // this envelope version cannot reopen (for example a newer report).
            wire::report(
                report_text,
                input.bundle.configuration().selected_profile(),
                &input.reference,
                &files,
            )?;
            cancelled(stop)?;
            let output = wire::ArtifactOutput {
                schema: wire::ARTIFACT_SCHEMA,
                profile: input.bundle.configuration().selected_profile(),
                reference_view: &input.reference,
                library_files: files,
                source_report: wire::SourceReportOutput {
                    json: report_text,
                    sha256: wow_reference::native::source_digest(report),
                },
                negative_authority: false,
            };
            super::super::native_input::report_bytes_with_limit(&output, request.max_bytes, stop)?
        }
    };
    cancelled(stop)?;
    let sha256 = wow_reference::native::source_digest(&bytes);
    if request
        .expected_sha256
        .is_some_and(|expected| expected.to_string() != sha256)
    {
        return Err(ServiceError::new(
            ServiceErrorCode::IdentityMismatch,
            "native export digest guard did not match",
        ));
    }
    Ok(NativeExportArtifact {
        bytes: bytes.into_boxed_slice(),
        sha256,
    })
}
