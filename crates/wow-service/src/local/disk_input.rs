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
pub const LOCAL_TOC_SCHEMA: &str = "wow-service/local-project-toc/1";

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
    main: MainInventory,
    library: DiskInventory,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiskAnalyzer {
    compatibility_report: ProjectDiskFile,
    accepted_pin_id: String,
    configuration_digest: ContentDigest<CanonicalResult>,
    contract_id: String,
    fixture_contract_id: String,
    library_contract_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct DiskInventory {
    pub(super) root: String,
    pub(super) files: Vec<ProjectDiskFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct MainInventory {
    root: String,
    #[serde(default)]
    files: Option<Vec<ProjectDiskFile>>,
    #[serde(default)]
    toc: Option<ProjectDiskFile>,
    #[serde(default)]
    load_context: Option<wow_project::load::TocLoadContext>,
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
            super::native_input::LOCAL_NATIVE_SCHEMA => {
                Self::from_native_manifest(&bytes, &directory, stop)
            }
            super::native_artifact::LOCAL_NATIVE_ARTIFACT_SCHEMA => {
                Self::from_native_artifact(&bytes, &directory, stop)
            }
            LOCAL_FILES_SCHEMA | LOCAL_TOC_SCHEMA => {
                Self::from_disk_manifest(&bytes, &directory, stop)
            }
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
        let toc_mode = input.schema == LOCAL_TOC_SCHEMA;
        match (input.schema.as_str(), &input.main.files, &input.main.toc) {
            (LOCAL_FILES_SCHEMA, Some(_), None) | (LOCAL_TOC_SCHEMA, None, Some(_)) => {}
            _ => {
                return Err(invalid(
                    "manifest must select either explicit files or one TOC, never both",
                ));
            }
        }
        if !toc_mode && input.main.load_context.is_some() {
            return Err(invalid("load_context is only valid for selected TOC input"));
        }
        if let Some(context) = &input.main.load_context {
            context.validate().map_err(acquisition_error)?;
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
        let analyzer = input.analyzer.read(directory, stop)?;
        let (main, load_plan) = input.main.read(directory, &profile, stop)?;
        let library = directory
            .read_lua_inventory(
                &input.library.root,
                &input.library.files,
                ProjectFileRole::Library,
                stop,
            )
            .map_err(acquisition_error)?;
        super::cancelled(stop)?;
        Self::assemble_with_load_plan(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile,
                analyzer,
            },
            reference,
            main,
            library,
            load_plan,
        )
    }
}

pub(super) fn acquisition_error(error: ProjectError) -> ServiceError {
    let code = match error.code() {
        ProjectErrorCode::SourceReadCancelled => ServiceErrorCode::Cancelled,
        ProjectErrorCode::SourceBudgetExceeded => ServiceErrorCode::BudgetExceeded,
        ProjectErrorCode::PackageTargetExcluded => ServiceErrorCode::ProjectTargetExcluded,
        ProjectErrorCode::PackageTargetUnresolved => ServiceErrorCode::ProjectTargetUnresolved,
        _ => ServiceErrorCode::InvalidConfiguration,
    };
    // The project reader emits only fixed messages; never forward OS error prose.
    ServiceError::new(code, error.message())
}

impl DiskAnalyzer {
    pub(super) fn read(
        self,
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<AnalyzerInput> {
        let report = String::from_utf8(
            directory
                .read_json_artifact(&self.compatibility_report, stop)
                .map_err(acquisition_error)?,
        )
        .map_err(|_| invalid("analyzer report must contain UTF-8"))?;
        Ok(AnalyzerInput {
            compatibility_report_json: report,
            accepted_pin_id: self.accepted_pin_id,
            configuration_digest: self.configuration_digest,
            contract_id: self.contract_id,
            fixture_contract_id: self.fixture_contract_id,
            library_contract_id: self.library_contract_id,
        })
    }
}

impl MainInventory {
    pub(super) fn read(
        &self,
        directory: &ProjectInputDirectory,
        profile: &ProfileIdentity,
        stop: &AtomicBool,
    ) -> ServiceResult<(
        Vec<wow_project::ProjectInputFile>,
        Option<wow_project::load::ProjectLoadPlan>,
    )> {
        match (&self.files, &self.toc) {
            (Some(files), None) if self.load_context.is_none() => Ok((
                directory
                    .read_lua_inventory(&self.root, files, ProjectFileRole::FirstPartyMain, stop)
                    .map_err(acquisition_error)?,
                None,
            )),
            (None, Some(toc)) => {
                if let Some(context) = &self.load_context {
                    context.validate().map_err(acquisition_error)?;
                }
                let (files, plan) = directory
                    .read_toc_project_with_context(
                        &self.root,
                        toc,
                        profile,
                        self.load_context.as_ref(),
                        stop,
                    )
                    .map_err(acquisition_error)?
                    .into_parts();
                Ok((files, Some(plan)))
            }
            _ => Err(invalid(
                "Main must select explicit Lua files or one TOC; load_context requires TOC",
            )),
        }
    }
}
