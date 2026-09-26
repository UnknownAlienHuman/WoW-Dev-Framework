//! Admit a complete xtask source-manifest document, then capture only its exact
//! version/selected-TOC/Lua closure. No Git, network, scanning or source execution.
mod model;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{ContentDigest, SourceContent};

use super::{
    DISK_INVENTORY_MAX_BYTES, DISK_SOURCE_MAX_BYTES, PinnedLuaSource, ProjectDiskFile,
    ProjectInputDirectory, checkpoint, failure, validate_path,
};
use crate::load::{DocumentTocSelection, LoadSource, TocLoadContext, document_toc};
use crate::{ProjectError, ProjectErrorCode, ProjectResult};
use model::{Member, SourceManifest};

const MAX_MANIFEST_BYTES: usize = 64 * 1024 * 1024;
const MAX_MANIFEST_FILES: u64 = 200_000;
const MAX_MEMBER_BYTES: u64 = 32 * 1024 * 1024;
const MAX_SELECTED_BYTES: u64 = 256 * 1024 * 1024;

/// One caller-selected manifest and TOC; no floating source selector is resolved here.
pub struct ManifestedLuaRequest<'a> {
    pub root: &'a str,
    /// Relative to the registered configuration directory, not the source root.
    pub manifest: &'a ProjectDiskFile,
    /// Exact root-relative path in the manifest, not an inferred addon variant.
    pub toc: &'a str,
    pub revision: &'a str,
    /// Exact version.txt value required by the caller's selected profile.
    pub version: &'a str,
    pub interface: u64,
    pub load_context: Option<&'a TocLoadContext>,
}

/// Bounded evidence of manifest admission and consumed content, not Git membership
/// authentication, full-mirror verification or complete semantic coverage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceManifestReceipt {
    pub schema: &'static str,
    pub manifest: LoadSource,
    pub manifest_sha256: String,
    pub source_revision: String,
    pub source_selector: String,
    pub source_version: String,
    pub git_object_format: String,
    pub declared_tracked_files: u64,
    pub declared_included_files: u64,
    pub declared_included_bytes: u64,
    pub verified_source_files: usize,
    pub version_file: LoadSource,
    pub toc: DocumentTocSelection,
    pub git_membership: &'static str,
    pub unconsumed_source_bytes: &'static str,
    pub negative_authority: bool,
}

impl SourceManifestReceipt {
    #[must_use]
    pub fn selected_toc(&self) -> &LoadSource {
        &self.toc.source
    }
    #[must_use]
    pub fn selected_file_count(&self) -> usize {
        self.toc.source_order.len()
    }
}

pub struct ManifestedLuaSources {
    sources: Vec<PinnedLuaSource>,
    receipt: SourceManifestReceipt,
}
impl ManifestedLuaSources {
    #[must_use]
    pub fn into_parts(self) -> (Vec<PinnedLuaSource>, SourceManifestReceipt) {
        (self.sources, self.receipt)
    }
}

impl ProjectInputDirectory {
    /// Verify the manifest's wire shape, self-digest, coverage arithmetic and
    /// entire member index before opening its source root. Only TOC-selected
    /// generated Lua documents plus TOC/version bytes are subsequently consumed.
    pub fn read_manifested_lua_sources(
        &self,
        request: &ManifestedLuaRequest<'_>,
        stop: &AtomicBool,
    ) -> ProjectResult<ManifestedLuaSources> {
        checkpoint(stop)?;
        request.manifest.validate()?;
        validate_path(request.toc)?;
        if request.manifest.content_digest.is_none() || !request.manifest.path.ends_with(".json") {
            return Err(invalid("source manifest requires pinned JSON bytes"));
        }
        let bytes = self.read(request.manifest, MAX_MANIFEST_BYTES, stop)?;
        let manifest = SourceManifest::admit(&bytes, request, stop)?;
        let selected_toc = manifest.member(request.toc)?;
        if selected_toc.kind != "toc"
            || !request
                .toc
                .split('/')
                .any(|p| p == "Blizzard_APIDocumentationGenerated")
        {
            return Err(invalid(
                "source TOC must belong to the generated documentation corpus",
            ));
        }
        let version = manifest.member("version.txt")?;
        // Retain one root handle for all consumed source files. A renamed root
        // cannot redirect a later Lua acquisition to another directory.
        let directory = self.subdirectory(request.root)?;
        let toc_bytes = directory.read(&selected_toc.disk_file()?, DISK_SOURCE_MAX_BYTES, stop)?;
        let version_bytes = directory.read(&version.disk_file()?, DISK_SOURCE_MAX_BYTES, stop)?;
        let version_text = std::str::from_utf8(&version_bytes)
            .map_err(|_| invalid("source version file is not UTF-8"))?;
        if version_text.trim() != manifest.source.version {
            return Err(invalid("source version bytes disagree with the manifest"));
        }
        let toc_text =
            std::str::from_utf8(&toc_bytes).map_err(|_| invalid("source TOC is not UTF-8"))?;
        let toc = document_toc::select(
            request.toc,
            toc_text,
            request.interface,
            request.load_context,
            stop,
        )?;
        let mut selected = Vec::with_capacity(toc.source_order.len());
        let mut total = toc_bytes.len().saturating_add(version_bytes.len()) as u64;
        for path in &toc.source_order {
            checkpoint(stop)?;
            let member = manifest.member(path)?;
            if member.kind != "generated_api" {
                return Err(invalid(
                    "TOC-selected Lua input is not a generated API manifest member",
                ));
            }
            total = total.checked_add(member.bytes).ok_or_else(budget)?;
            if total > DISK_INVENTORY_MAX_BYTES as u64 {
                return Err(budget());
            }
            selected.push(member.disk_file()?);
        }
        let sources = directory.read_pinned_lua_sources(".", &selected, stop)?;
        checkpoint(stop)?;
        let receipt = SourceManifestReceipt {
            schema: "wow-project/source-manifest-admission/1",
            manifest: LoadSource {
                path: request.manifest.path.clone(),
                content_digest: crate::identity::source_digest(&bytes),
                byte_length: bytes.len() as u64,
            },
            manifest_sha256: manifest.manifest_sha256,
            source_revision: manifest.source.revision,
            source_selector: manifest.source.selector,
            source_version: manifest.source.version,
            git_object_format: manifest.source.git_object_format,
            declared_tracked_files: manifest.coverage.tracked_files,
            declared_included_files: manifest.coverage.included_files,
            declared_included_bytes: manifest.coverage.included_bytes,
            verified_source_files: sources.len() + 2,
            version_file: LoadSource {
                path: "version.txt".to_owned(),
                content_digest: crate::identity::source_digest(&version_bytes),
                byte_length: version_bytes.len() as u64,
            },
            toc,
            git_membership: "not_attested",
            unconsumed_source_bytes: "not_verified",
            negative_authority: false,
        };
        Ok(ManifestedLuaSources { sources, receipt })
    }
}

impl SourceManifest {
    fn admit(
        bytes: &[u8],
        request: &ManifestedLuaRequest<'_>,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        checkpoint(stop)?;
        // Do not decode through Value first: it would erase repeated member names.
        let manifest: Self = serde_json::from_slice(bytes)
            .map_err(|_| invalid("invalid or ambiguous source manifest"))?;
        if manifest.schema_version != 1
            || manifest.source.source_id != "blizzard-ui"
            || manifest.source.acquisition != "local_git_object_database"
            || manifest.source.revision != request.revision
            || manifest.source.version != request.version
            || !label(&manifest.source.version)
            || !label(&manifest.source.selector)
            || manifest.selection.extensions != [".lua", ".toc", ".xml", ".xsd"]
            || manifest.selection.version_path != "version.txt"
            || manifest.selection.non_regular_entries != "reject"
            || manifest.selection.working_tree != "ignored"
        {
            return Err(invalid(
                "source manifest schema, selection or profile identity mismatch",
            ));
        }
        let oid_length = match manifest.source.git_object_format.as_str() {
            "sha1" => 40,
            "sha256" => 64,
            _ => return Err(invalid("unsupported source Git object format")),
        };
        if !hex(&manifest.source.revision, oid_length) || !hex(&manifest.manifest_sha256, 64) {
            return Err(invalid("invalid source manifest identity"));
        }
        manifest.validate_members(oid_length, stop)?;
        checkpoint(stop)?;
        // xtask v1 seals sorted compact JSON without manifest_sha256. Preserve
        // original field presence; the typed pass above has already rejected
        // duplicates/unknown fields. Core canonicalization also rejects nulls.
        let mut value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|_| invalid("source manifest cannot be decoded"))?;
        value
            .as_object_mut()
            .ok_or_else(|| invalid("source manifest must be an object"))?
            .remove("manifest_sha256");
        let canonical = wow_core::canonical_json_bytes(&value)
            .map_err(|_| invalid("source manifest is outside the canonical wire subset"))?;
        if format!("{:x}", Sha256::digest(&canonical)) != manifest.manifest_sha256 {
            return Err(failure(
                ProjectErrorCode::FileDigestMismatch,
                "source manifest self-digest mismatch",
            ));
        }
        checkpoint(stop)?;
        Ok(manifest)
    }

    fn validate_members(&self, oid_length: usize, stop: &AtomicBool) -> ProjectResult<()> {
        if self.files.is_empty()
            || self.files.len() as u64 > MAX_MANIFEST_FILES
            || self.coverage.tracked_files > MAX_MANIFEST_FILES
        {
            return Err(budget());
        }
        let mut previous: Option<&str> = None;
        let mut folded = BTreeSet::new();
        let mut kinds = BTreeMap::new();
        let mut total = 0u64;
        for member in &self.files {
            checkpoint(stop)?;
            validate_path(&member.path)?;
            if previous.is_some_and(|prior| prior.as_bytes() >= member.path.as_bytes())
                || !folded.insert(member.path.to_lowercase())
                || member.git_blob_algorithm != self.source.git_object_format
                || !hex(&member.git_blob_id, oid_length)
                || !hex(&member.content_sha256, 64)
                || member.kind != classify(&member.path)?
            {
                return Err(invalid(
                    "source manifest member identity, order or kind is invalid",
                ));
            }
            total = total.checked_add(member.bytes).ok_or_else(budget)?;
            if member.bytes > MAX_MEMBER_BYTES || total > MAX_SELECTED_BYTES {
                return Err(budget());
            }
            *kinds.entry(member.kind.as_str()).or_insert(0u64) += 1;
            previous = Some(&member.path);
        }
        let coverage = &self.coverage;
        if coverage.included_files != self.files.len() as u64
            || coverage.included_bytes != total
            || coverage.included_files.checked_add(coverage.excluded_files)
                != Some(coverage.tracked_files)
            || kinds.get("version") != Some(&1)
        {
            return Err(invalid(
                "source manifest coverage differs from its member inventory",
            ));
        }
        for (kind, count) in [
            ("version", coverage.kind_version),
            ("generated_api", coverage.kind_generated_api),
            ("lua", coverage.kind_lua),
            ("toc", coverage.kind_toc),
            ("xml", coverage.kind_xml),
            ("schema", coverage.kind_schema),
        ] {
            if count != kinds.get(kind).copied() {
                return Err(invalid(
                    "source manifest kind coverage differs from its members",
                ));
            }
        }
        Ok(())
    }

    fn member(&self, path: &str) -> ProjectResult<&Member> {
        self.files
            .binary_search_by(|member| member.path.as_str().cmp(path))
            .map(|index| &self.files[index])
            .map_err(|_| {
                failure(
                    ProjectErrorCode::UndeclaredFile,
                    "consumed source is absent from the manifest",
                )
            })
    }
}
impl Member {
    fn disk_file(&self) -> ProjectResult<ProjectDiskFile> {
        let digest: ContentDigest<SourceContent> = format!("sha256:{}", self.content_sha256)
            .parse()
            .map_err(|_| invalid("source manifest content digest is invalid"))?;
        Ok(ProjectDiskFile::new(&self.path).with_identity(digest, self.bytes))
    }
}
fn classify(path: &str) -> ProjectResult<&'static str> {
    let lower = path.to_ascii_lowercase();
    Ok(if path == "version.txt" {
        "version"
    } else if lower.ends_with(".lua") {
        if path
            .split('/')
            .any(|p| p == "Blizzard_APIDocumentationGenerated")
        {
            "generated_api"
        } else {
            "lua"
        }
    } else if lower.ends_with(".toc") {
        "toc"
    } else if lower.ends_with(".xml") {
        "xml"
    } else if lower.ends_with(".xsd") {
        "schema"
    } else {
        return Err(invalid("source manifest contains an unselected file kind"));
    })
}
fn label(value: &str) -> bool {
    !value.is_empty() && value.len() <= 4096 && !value.chars().any(char::is_control)
}
fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn invalid(message: &'static str) -> ProjectError {
    failure(ProjectErrorCode::InvalidInputInventory, message)
}
fn budget() -> ProjectError {
    failure(
        ProjectErrorCode::SourceBudgetExceeded,
        "source manifest exceeds its declared admission budget",
    )
}
