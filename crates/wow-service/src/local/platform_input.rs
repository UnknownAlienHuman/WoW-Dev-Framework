//! Data-only platform transport. Source, package/Main, XML and graph authority
//! comes from retained native owners, never from a decoded owner receipt.
use std::{sync::Arc, sync::atomic::AtomicBool};

use serde::Deserialize;
use wow_project::{
    PlatformGraphProfile, ProjectFileRole, ProjectId, ProjectSourceOriginId, ProjectWorkspaceId,
    disk::{ManifestedPlatformInput, ProjectDiskFile, ProjectInputDirectory},
    load::{ProjectPackageInput, TocLoadContext},
    platform_source::{BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest},
};
use wow_reference::ReferenceView;

use super::{
    LocalProjectInput,
    disk_input::{DiskAnalyzer, DiskInventory, acquisition_error},
    input::{LocalProjectLoad, ProjectMetadata, invalid},
};
use crate::ServiceResult;

pub const LOCAL_PLATFORM_SCHEMA: &str = "wow-service/local-platform-source/1";

#[cfg(test)]
mod tests;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformInput {
    schema: String,
    project_id: ProjectId,
    workspace_id: ProjectWorkspaceId,
    source_origin_id: ProjectSourceOriginId,
    logical_root: String,
    source_profile: BlizzardUiSourceProfileRequest,
    source: ManifestedPlatformInput,
    reference_view: ProjectDiskFile,
    analyzer: DiskAnalyzer,
    packages: Vec<ProjectPackageInput>,
    #[serde(default)]
    load_context: Option<TocLoadContext>,
    graph_profile: PlatformGraphProfile,
    #[serde(default)]
    library: Option<DiskInventory>,
}

impl LocalProjectInput {
    pub(super) fn from_platform_manifest(
        bytes: &[u8],
        directory: &ProjectInputDirectory,
        stop: &AtomicBool,
    ) -> ServiceResult<Self> {
        super::cancelled(stop)?;
        let input: PlatformInput =
            serde_json::from_slice(bytes).map_err(|_| invalid("invalid platform source input"))?;
        if input.schema != LOCAL_PLATFORM_SCHEMA {
            return Err(invalid("unsupported platform source input schema"));
        }
        let profile =
            BlizzardUiSourceProfile::new(input.source_profile).map_err(acquisition_error)?;
        let source = Arc::new(
            directory
                .admit_platform_source_manifest(&profile, &input.source, stop)
                .map_err(acquisition_error)?
                .into_parts()
                .0,
        );
        let packages = Arc::new(
            source
                .specialize_packages(&input.packages, input.load_context.as_ref(), stop)
                .map_err(acquisition_error)?,
        );
        let reference: ReferenceView = serde_json::from_slice(
            &directory
                .read_json_artifact(&input.reference_view, stop)
                .map_err(acquisition_error)?,
        )
        .map_err(|_| invalid("reference artifact was rejected"))?;
        let analyzer = input.analyzer.read(directory, stop)?;
        let libraries = match input.library {
            Some(library) => directory
                .read_lua_inventory(
                    &library.root,
                    &library.files,
                    ProjectFileRole::Library,
                    stop,
                )
                .map_err(acquisition_error)?,
            None => Vec::new(),
        };
        super::cancelled(stop)?;
        Self::assemble_with_platform(
            ProjectMetadata {
                project_id: input.project_id,
                workspace_id: input.workspace_id,
                source_origin_id: input.source_origin_id,
                logical_root: input.logical_root,
                profile: profile.target().reference_profile.clone(),
                analyzer,
            },
            reference,
            packages.files().to_vec(),
            libraries,
            LocalProjectLoad::PackageUniverse {
                load_plan: packages.load_plan().clone(),
                main_plan: packages.main_plan().clone(),
            },
            Some((packages, input.graph_profile)),
        )
    }
}
