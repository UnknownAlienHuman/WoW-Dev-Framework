//! Thin request conversion for explicit-list or manifested-TOC native acquisition.
//! File/manifest/load validation remains in wow-project, not in the service.
use std::sync::atomic::AtomicBool;

use serde::Deserialize;
use wow_project::disk::{
    ManifestedLuaRequest, ManifestedPlatformInput, PinnedLuaSource, ProjectDiskFile,
    ProjectInputDirectory, SourceManifestReceipt,
};
use wow_project::load::TocLoadContext;

use super::disk_input::acquisition_error;
use super::input::invalid;
use crate::ServiceResult;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NativeSourceInput {
    root: String,
    #[serde(default)]
    files: Option<Vec<ProjectDiskFile>>,
    #[serde(default)]
    manifest: Option<ProjectDiskFile>,
    #[serde(default)]
    toc: Option<String>,
    #[serde(default)]
    load_context: Option<TocLoadContext>,
}

impl NativeSourceInput {
    /// Require the native callable input to acquire the platform request's
    /// exact manifested source. Pinned-file lists are not this capability.
    pub(super) fn validate_platform_manifest(
        &self,
        source: &ManifestedPlatformInput,
    ) -> ServiceResult<()> {
        if self.root != source.source_root
            || self.manifest.as_ref() != Some(&source.manifest)
            || self.files.is_some()
            || self.toc.is_none()
        {
            return Err(invalid(
                "native Reference must use the exact platform source manifest",
            ));
        }
        Ok(())
    }

    pub(super) fn read(
        &self,
        directory: &ProjectInputDirectory,
        revision: &str,
        version: &str,
        interface: u64,
        stop: &AtomicBool,
    ) -> ServiceResult<(Vec<PinnedLuaSource>, Option<SourceManifestReceipt>)> {
        match (&self.files, &self.manifest, &self.toc, &self.load_context) {
            (Some(files), None, None, None) => Ok((
                directory
                    .read_pinned_lua_sources(&self.root, files, stop)
                    .map_err(acquisition_error)?,
                None,
            )),
            (None, Some(manifest), Some(toc), context) => {
                let (sources, receipt) = directory
                    .read_manifested_lua_sources(
                        &ManifestedLuaRequest {
                            root: &self.root,
                            manifest,
                            toc,
                            revision,
                            version,
                            interface,
                            load_context: context.as_ref(),
                        },
                        stop,
                    )
                    .map_err(acquisition_error)?
                    .into_parts();
                Ok((sources, Some(receipt)))
            }
            _ => Err(invalid(
                "native source must select either pinned files or one manifest and TOC",
            )),
        }
    }
}
