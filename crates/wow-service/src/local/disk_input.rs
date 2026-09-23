//! Resolve only the explicit files in a local configuration. File acquisition is
//! owned by wow-project; profile, reference and analyzer semantics stay with owners.
use std::path::Path;
use std::sync::atomic::AtomicBool;

use serde::Deserialize;
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity};
use wow_project::disk::{ProjectDiskFile, ProjectInputDirectory};
use wow_project::{
    ProjectError, ProjectErrorCode, ProjectFileRole, ProjectId, ProjectSourceOriginId,
    ProjectWorkspaceId,
};
use wow_reference::ReferenceView;

use super::LocalProjectInput;
use super::input::{AnalyzerInput, LOCAL_INPUT_SCHEMA, ProjectMetadata, invalid};
use crate::{ServiceError, ServiceErrorCode, ServiceResult};

pub const LOCAL_FILES_SCHEMA: &str = "wow-service/local-project-files/1";

#[derive(Deserialize)]
struct SchemaSelector {
    schema: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiskInput {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    profile: ProjectDiskFile,
    reference_view: ProjectDiskFile,
    analyzer: DiskAnalyzer,
    main: DiskInventory,
    library: DiskInventory,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiskAnalyzer {
    compatibility_report: ProjectDiskFile,
    accepted_pin_id: String,
    configuration_digest: ContentDigest<CanonicalResult>,
    contract_id: String,
    fixture_contract_id: String,
    library_contract_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DiskInventory {
    root: String,
    files: Vec<ProjectDiskFile>,
}

impl LocalProjectInput {
    /// Read one explicit config and its declared inputs. The selected config's
    /// parent is registered once; all manifest paths are relative to that handle.
    /// Inline inputs remain supported; neither schema scans a directory or writes.
    pub fn from_config_path(path: &Path, stop: &AtomicBool) -> ServiceResult<Self> {
        super::cancelled(stop)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| invalid("configuration must name a UTF-8 JSON file"))?;
        let directory = ProjectInputDirectory::open(parent).map_err(acquisition_error)?;
        let bytes = directory
            .read_configuration(name, stop)
            .map_err(acquisition_error)?;
        let selector: SchemaSelector = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("invalid local input schema selector"))?;
        let result = match selector.schema.as_str() {
            LOCAL_INPUT_SCHEMA => Self::from_json_slice(&bytes),
            LOCAL_FILES_SCHEMA => Self::from_disk_manifest(&bytes, &directory, stop),
            _ => Err(invalid("unsupported local input schema")),
        }?;
        super::cancelled(stop)?;
        Ok(result)
    }

    fn from_disk_manifest(
        bytes: &[u8],
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        let input: DiskInput =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid local file manifest"))?;
        if input.schema != LOCAL_FILES_SCHEMA {
            return Err(invalid("unsupported local file manifest"));
        }
        let profile: ProfileIdentity = serde_json::from_slice(
            &directory
                .read_json_artifact(&input.profile, stop)
                .map_err(acquisition_error)?,
        )
        .map_err(|_| invalid("profile artifact was rejected"))?;
        profile
            .validate()
            .map_err(|_| invalid("profile artifact was rejected"))?;
        let reference: ReferenceView = serde_json::from_slice(
            &directory
                .read_json_artifact(&input.reference_view, stop)
                .map_err(acquisition_error)?,
        )
        .map_err(|_| invalid("reference artifact was rejected"))?;
        reference
            .validate()
            .map_err(|_| invalid("reference artifact was rejected"))?;
        let report = String::from_utf8(
            directory
                .read_json_artifact(&input.analyzer.compatibility_report, stop)
                .map_err(acquisition_error)?,
        )
        .map_err(|_| invalid("analyzer report must contain UTF-8"))?;
        let main = directory
            .read_lua_inventory(
                &input.main.root,
                &input.main.files,
                ProjectFileRole::FirstPartyMain,
                stop,
            )
            .map_err(acquisition_error)?;
        let library = directory
            .read_lua_inventory(
                &input.library.root,
                &input.library.files,
                ProjectFileRole::Library,
                stop,
            )
            .map_err(acquisition_error)?;
        super::cancelled(stop)?;
        Self::assemble(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile,
                analyzer: AnalyzerInput {
                    compatibility_report_json: report,
                    accepted_pin_id: input.analyzer.accepted_pin_id,
                    configuration_digest: input.analyzer.configuration_digest,
                    contract_id: input.analyzer.contract_id,
                    fixture_contract_id: input.analyzer.fixture_contract_id,
                    library_contract_id: input.analyzer.library_contract_id,
                },
            },
            reference,
            main,
            library,
        )
    }
}

fn acquisition_error(error: ProjectError) -> ServiceError {
    let code = match error.code() {
        ProjectErrorCode::SourceReadCancelled => ServiceErrorCode::Cancelled,
        ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        _ => ServiceErrorCode::InvalidConfiguration,
    };
    // The project reader emits only fixed messages; never forward OS error prose.
    ServiceError::new(code, error.message())
}
