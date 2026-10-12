//! Re-admission of raw manifested documentation selection inputs. Reuses the
//! manifest validator and TOC parser; archived receipt labels cannot mint an owner.
use super::{
    MAX_MANIFEST_BYTES, ManifestRoute, ManifestedLuaRequest, SourceManifest, SourceManifestReceipt,
    budget, invalid,
};
use crate::{
    ProjectResult,
    disk::{
        DISK_INVENTORY_MAX_BYTES, DISK_INVENTORY_MAX_FILES, DISK_SOURCE_MAX_BYTES, PinnedLuaSource,
        ProjectDiskFile, checkpoint, validate_path,
    },
    load::{TocLoadContext, document_toc},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt,
    sync::{Arc, atomic::AtomicBool},
};

/// Strict input data for the same manifested generated-API selection owner.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestedLuaReplayInput {
    manifest: ProjectDiskFile,
    manifest_text: String,
    toc_path: String,
    toc_text: String,
    version_text: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present_context"
    )]
    load_context: Option<TocLoadContext>,
}

fn present_context<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<TocLoadContext>, D::Error> {
    TocLoadContext::deserialize(deserializer).map(Some)
}

impl fmt::Debug for ManifestedLuaReplayInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ManifestedLuaReplayInput")
            .field("manifest_bytes", &self.manifest_text.len())
            .field("toc_bytes", &self.toc_text.len())
            .field("version_bytes", &self.version_text.len())
            .finish_non_exhaustive()
    }
}

impl ManifestedLuaReplayInput {
    pub(super) fn capture(
        request: &ManifestedLuaRequest<'_>,
        manifest: Vec<u8>,
        toc: Vec<u8>,
        version: Vec<u8>,
    ) -> ProjectResult<Arc<Self>> {
        Ok(Arc::new(Self {
            manifest: request.manifest.clone(),
            manifest_text: String::from_utf8(manifest)
                .map_err(|_| invalid("manifest input is not UTF-8"))?,
            toc_path: request.toc.to_owned(),
            toc_text: String::from_utf8(toc).map_err(|_| invalid("TOC input is not UTF-8"))?,
            version_text: String::from_utf8(version)
                .map_err(|_| invalid("version input is not UTF-8"))?,
            load_context: request.load_context.cloned(),
        }))
    }
    pub(super) fn manifest_path(&self) -> &str {
        self.manifest.path()
    }
    pub(super) fn manifest_bytes(&self) -> &[u8] {
        self.manifest_text.as_bytes()
    }
    pub(super) fn version_bytes(&self) -> &[u8] {
        self.version_text.as_bytes()
    }

    /// Re-run source admission against actual captured Lua inputs, without disk
    /// reads, temporary files, decoded receipts or semantic output recovery.
    pub fn rebuild(
        &self,
        revision: &str,
        version: &str,
        interface: u64,
        sources: &[PinnedLuaSource],
        stop: &AtomicBool,
    ) -> ProjectResult<SourceManifestReceipt> {
        checkpoint(stop)?;
        self.manifest.validate()?;
        validate_path(&self.toc_path)?;
        if self.manifest.content_digest.is_none() || !self.manifest.path().ends_with(".json") {
            return Err(invalid("source manifest requires pinned JSON bytes"));
        }
        if self.manifest_text.len() > MAX_MANIFEST_BYTES
            || self.toc_text.len() > DISK_SOURCE_MAX_BYTES
            || self.version_text.len() > DISK_SOURCE_MAX_BYTES
            || sources.is_empty()
            || sources.len() > DISK_INVENTORY_MAX_FILES
        {
            return Err(budget());
        }
        self.manifest.verify(self.manifest_bytes())?;
        let manifest = SourceManifest::admit(
            self.manifest_bytes(),
            revision,
            version,
            ManifestRoute::GeneratedApi,
            stop,
        )?;
        let toc_member = manifest.member(&self.toc_path)?;
        if toc_member.kind != "toc"
            || !self
                .toc_path
                .split('/')
                .any(|p| p == "Blizzard_APIDocumentationGenerated")
        {
            return Err(invalid(
                "source TOC must belong to the generated documentation corpus",
            ));
        }
        toc_member.disk_file()?.verify(self.toc_text.as_bytes())?;
        manifest
            .member("version.txt")?
            .disk_file()?
            .verify(self.version_bytes())?;
        if self.version_text.trim() != manifest.source.version {
            return Err(invalid("source version bytes disagree with the manifest"));
        }
        let toc = document_toc::select(
            &self.toc_path,
            &self.toc_text,
            interface,
            self.load_context.as_ref(),
            stop,
        )?;
        let mut selected = toc
            .source_order
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        selected.sort_unstable();
        if sources.len() != selected.len() {
            return Err(invalid("retained documentation input set changed"));
        }
        let mut total = self
            .toc_text
            .len()
            .checked_add(self.version_text.len())
            .ok_or_else(budget)?;
        for (path, file) in selected.iter().zip(sources) {
            checkpoint(stop)?;
            let member = manifest.member(path)?;
            if member.kind != "generated_api" || file.path() != *path {
                return Err(invalid("retained documentation input set changed"));
            }
            if file.text().len() > DISK_SOURCE_MAX_BYTES {
                return Err(budget());
            }
            member.disk_file()?.verify(file.text().as_bytes())?;
            total = total.checked_add(file.text().len()).ok_or_else(budget)?;
            if total > DISK_INVENTORY_MAX_BYTES {
                return Err(budget());
            }
        }
        let generated = manifest
            .files
            .iter()
            .filter(|file| file.kind == "generated_api")
            .map(|file| file.path.as_str())
            .collect::<BTreeSet<_>>();
        let selected = selected.into_iter().collect::<BTreeSet<_>>();
        let complete = !generated.is_empty()
            && generated.len() == toc.source_order.len()
            && generated == selected;
        let count = u64::try_from(generated.len()).map_err(|_| budget())?;
        checkpoint(stop)?;
        Ok(Arc::new(self.clone()).receipt(manifest, toc, complete, count, sources.len()))
    }
}
