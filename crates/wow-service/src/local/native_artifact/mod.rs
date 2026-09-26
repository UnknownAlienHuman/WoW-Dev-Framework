//! Explicit prebuilt native input. The report is retained producer evidence,
//! not a re-executed source projection or serialized analyzer session.
mod export;
mod wire;

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wow_core::{CanonicalResult, ContentDigest, ProfileId, ProfileKind, SourceContent};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_project::{
    ProjectFileRole, ProjectId, ProjectInputFile, ProjectLanguageKind, ProjectSourceOriginId,
    ProjectWorkspaceId,
};

use super::disk_input::{DiskAnalyzer, MainInventory, acquisition_error};
use super::input::{ProjectMetadata, invalid};
use super::{LocalProjectInput, cancelled};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub use export::{
    NativeExportArtifact, NativeExportKind, NativeExportRequest, execute_native_export,
};
pub use wire::ARTIFACT_SCHEMA as NATIVE_ARTIFACT_SCHEMA;
pub const LOCAL_NATIVE_ARTIFACT_SCHEMA: &str = "wow-service/local-project-native-artifact/1";
pub const NATIVE_ARTIFACT_MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    expected_profile_id: ProfileId,
    artifact: ProjectDiskFile,
    analyzer: DiskAnalyzer,
    main: MainInventory,
}

/// Current admission/binding receipt. Historical analyzer IDs inside the retained
/// source report are not substituted for this operation's analyzer declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NativeArtifactReceipt {
    pub schema: &'static str,
    pub artifact_sha256: String,
    pub artifact_bytes: usize,
    pub report_sha256: String,
    pub report_bytes: usize,
    pub library_files: usize,
    pub library_bytes: usize,
    pub analyzer_input_configuration: ContentDigest<CanonicalResult>,
    pub analyzer_bound_configuration: ContentDigest<CanonicalResult>,
    pub provenance: &'static str,
    pub source_verification: &'static str,
    pub producer_verification: &'static str,
    pub source_maps_verification: &'static str,
    pub semantic_consumer_acceptance: &'static str,
    pub negative_authority: bool,
}

pub(super) struct NativeArtifactEvidence {
    pub receipt: NativeArtifactReceipt,
    pub report: Box<[u8]>,
}

impl LocalProjectInput {
    #[must_use]
    pub fn native_artifact_receipt(&self) -> Option<&NativeArtifactReceipt> {
        self.native_artifact
            .as_ref()
            .map(|evidence| &evidence.receipt)
    }

    pub(super) fn from_native_artifact(
        bytes: &[u8],
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        cancelled(stop)?;
        let input: Input = serde_json::from_slice(bytes)
            .map_err(|_| invalid("invalid prebuilt native project configuration"))?;
        if input.schema != LOCAL_NATIVE_ARTIFACT_SCHEMA {
            return Err(invalid("unsupported prebuilt native project schema"));
        }
        let bytes = directory
            .read_native_artifact(&input.artifact, stop)
            .map_err(acquisition_error)?;
        let artifact_sha256 = wow_reference::native::source_digest(&bytes);
        let artifact_bytes = bytes.len();
        let artifact: wire::Artifact = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("invalid prebuilt native artifact"))?;
        drop(bytes);
        cancelled(stop)?;
        artifact
            .profile
            .validate()
            .map_err(|_| invalid("native artifact profile was rejected"))?;
        if artifact.schema != NATIVE_ARTIFACT_SCHEMA
            || artifact.negative_authority
            || artifact.profile.profile_kind() != ProfileKind::Release
            || artifact.profile.profile_id() != &input.expected_profile_id
            || input.expected_profile_id.as_str() == wow_rules::FIXTURE_PROFILE_ID
        {
            return Err(invalid(
                "native artifact schema, profile or authority mismatch",
            ));
        }
        wire::reference(&artifact.reference_view)?;
        if artifact.source_report.json.len() > NATIVE_ARTIFACT_MAX_BYTES
            || artifact.source_report.sha256.to_string()
                != wow_reference::native::source_digest(artifact.source_report.json.as_bytes())
        {
            return Err(invalid("native artifact report digest or size mismatch"));
        }
        let mut total = 0usize;
        let mut paths = BTreeSet::new();
        if artifact.library_files.is_empty() {
            return Err(invalid("native artifact has no Library files"));
        }
        for file in &artifact.library_files {
            cancelled(stop)?;
            total = total.checked_add(file.text.len()).ok_or_else(budget)?;
            if file.text.len() > 1024 * 1024 || total > 16 * 1024 * 1024 {
                return Err(budget());
            }
            if file.path.contains('/')
                || !file.path.ends_with(".lua")
                || !paths.insert(file.path.to_lowercase())
                || file.byte_length != file.text.len() as u64
                || file.sha256.to_string()
                    != wow_reference::native::source_digest(file.text.as_bytes())
            {
                return Err(invalid("native artifact Library path or content mismatch"));
            }
        }
        let report_files = artifact
            .library_files
            .iter()
            .map(|file| wire::LibraryFileOutput {
                path: &file.path,
                text: &file.text,
                sha256: file.sha256,
                byte_length: file.byte_length,
            })
            .collect::<Vec<_>>();
        wire::report(
            &artifact.source_report.json,
            &artifact.profile,
            &artifact.reference_view,
            &report_files,
        )?;
        drop(report_files);
        cancelled(stop)?;
        let mut analyzer = input.analyzer.read(directory, stop)?;
        let original_configuration = analyzer.configuration_digest;
        let binding = wow_core::canonical_json_bytes(&(
            "wow-service/native-artifact-analyzer-binding/1",
            original_configuration,
            &artifact_sha256,
            &artifact.profile,
            artifact.reference_view.self_digest(),
        ))
        .map_err(|_| invalid("native artifact analyzer binding was rejected"))?;
        analyzer.configuration_digest = ContentDigest::from_bytes(Sha256::digest(&binding).into());
        let receipt = NativeArtifactReceipt {
            schema: "wow-service/native-artifact-receipt/1",
            artifact_sha256,
            artifact_bytes,
            report_sha256: artifact.source_report.sha256.to_string(),
            report_bytes: artifact.source_report.json.len(),
            library_files: artifact.library_files.len(),
            library_bytes: total,
            analyzer_input_configuration: original_configuration,
            analyzer_bound_configuration: analyzer.configuration_digest,
            provenance: "explicit_digest_pinned_prebuilt_artifact",
            source_verification: "not_reacquired",
            producer_verification: "not_reexecuted",
            source_maps_verification: "retained_not_revalidated",
            semantic_consumer_acceptance: "not_evaluated",
            negative_authority: false,
        };
        let libraries = artifact
            .library_files
            .into_iter()
            .map(|file| {
                ProjectInputFile::declared_with_identity(
                    file.path,
                    file.text.into_bytes(),
                    ProjectLanguageKind::Lua,
                    ProjectFileRole::Library,
                    file.sha256,
                    file.byte_length,
                    None::<String>,
                )
                .map_err(|_| invalid("native artifact Library file was rejected by its owner"))
            })
            .collect::<ServiceResult<Vec<_>>>()?;
        let (main, load_plan) = input.main.read(directory, &artifact.profile, stop)?;
        let mut assembled = Self::assemble_with_load_plan(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile: artifact.profile,
                analyzer,
            },
            artifact.reference_view,
            main,
            libraries,
            load_plan,
        )?;
        assembled.native_artifact = Some(Arc::new(NativeArtifactEvidence {
            receipt,
            report: artifact.source_report.json.into_bytes().into_boxed_slice(),
        }));
        cancelled(stop)?;
        Ok(assembled)
    }
}

fn budget() -> ServiceError {
    ServiceError::new(
        ServiceErrorCode::BudgetExceeded,
        "native artifact exceeds its byte budget",
    )
}

fn parse_digest(value: &str) -> ServiceResult<ContentDigest<SourceContent>> {
    let digest: ContentDigest<SourceContent> = value
        .parse()
        .map_err(|_| invalid("invalid exact native artifact digest"))?;
    if digest.to_string() != value {
        return Err(invalid(
            "native artifact digest must use canonical spelling",
        ));
    }
    Ok(digest)
}
