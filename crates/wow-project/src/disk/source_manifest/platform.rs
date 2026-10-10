//! Bridge the existing manifest decoder to the existing platform byte owner.
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_core::{ContentDigest, SourceContent};

use super::{MAX_MANIFEST_BYTES, budget, invalid, read_manifest};
use crate::{
    ProjectResult,
    disk::{DISK_SOURCE_MAX_BYTES, ProjectDiskFile, ProjectInputDirectory, checkpoint},
    identity::source_digest,
    load::LoadSource,
    platform_source::{
        AdmittedPlatformSource, BlizzardUiSourceProfile, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformMaterializer, PlatformRootInventory, PlatformSourceInventory, PlatformSourceOrigin,
        PlatformSourceRevision, serialized_size, under,
    },
};

/// Exact local manifest selection and caller assertions. The manifest is pinned
/// relative to the configuration directory; its members are source-root-relative.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestedPlatformInput {
    pub source_root: String,
    pub manifest: ProjectDiskFile,
    pub source_revision: String,
    pub source_version: String,
    pub origin: PlatformSourceOrigin,
    pub materializer: PlatformMaterializer,
    pub license: PlatformLicenseRecord,
    pub compatibility_evidence: ContentDigest<SourceContent>,
}

/// Decoder and version-byte evidence. Out-of-profile members are declarations,
/// not consumed bytes; xtask's unlisted extension exclusions have no path census.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlatformSourceManifestReceipt {
    schema: &'static str,
    manifest: LoadSource,
    manifest_sha256: String,
    source_revision: String,
    source_selector: String,
    source_version: String,
    acquisition: String,
    declared_tracked_files: u64,
    declared_selected_files: u64,
    declared_selected_bytes: u64,
    declared_unlisted_extension_exclusions: u64,
    declared_generated_api_files: u64,
    version_file: LoadSource,
    outside_profile_members: Vec<LoadSource>,
    git_membership: &'static str,
    root_completeness: &'static str,
    outside_profile_bytes: &'static str,
    negative_authority: bool,
}

pub struct ManifestedPlatformSource {
    source: AdmittedPlatformSource,
    receipt: PlatformSourceManifestReceipt,
}
impl ManifestedPlatformSource {
    pub fn into_parts(self) -> (AdmittedPlatformSource, PlatformSourceManifestReceipt) {
        (self.source, self.receipt)
    }
}

impl ProjectInputDirectory {
    /// Derive entries only from the admitted manifest. Verify all Included bytes
    /// through the platform owner and version.txt through the same root handle.
    /// No manifest, caller revision or materializer assertion attests Git membership.
    pub fn admit_platform_source_manifest(
        &self,
        profile: &BlizzardUiSourceProfile,
        input: &ManifestedPlatformInput,
        stop: &AtomicBool,
    ) -> ProjectResult<ManifestedPlatformSource> {
        checkpoint(stop)?;
        profile.validate()?;
        let (manifest, bytes) = read_manifest(
            self,
            &input.manifest,
            &input.source_revision,
            &input.source_version,
            stop,
        )?;
        let manifest_digest = source_digest(&bytes);
        let git_members = match &input.origin.revision {
            PlatformSourceRevision::Git {
                object_format,
                commit,
                ..
            } if object_format == &manifest.source.git_object_format
                && commit == &manifest.source.revision =>
            {
                true
            }
            PlatformSourceRevision::Fixture { digest } if *digest == manifest_digest => false,
            _ => {
                return Err(invalid(
                    "platform origin differs from the source manifest identity",
                ));
            }
        };
        let mut roots = profile
            .roots()
            .iter()
            .map(|root| PlatformRootInventory {
                root: root.root.clone(),
                declared_entries: 0,
                scope: PlatformInventoryScope::DeclaredPartial,
                evidence_digest: manifest_digest,
            })
            .collect::<Vec<_>>();
        let mut entries = Vec::new();
        let mut outside_profile_members = Vec::new();
        for member in &manifest.files {
            checkpoint(stop)?;
            if member.path == "version.txt" {
                continue;
            }
            let content_digest = member.disk_file()?.content_digest.ok_or_else(budget)?;
            let Some(root) = roots
                .iter_mut()
                .find(|root| under(&member.path, &root.root))
            else {
                outside_profile_members
                    .try_reserve(1)
                    .map_err(|_| budget())?;
                outside_profile_members.push(LoadSource {
                    path: member.path.clone(),
                    content_digest,
                    byte_length: member.bytes,
                });
                continue;
            };
            if entries.len() >= profile.limits().max_entries {
                return Err(budget());
            }
            root.declared_entries = root.declared_entries.checked_add(1).ok_or_else(budget)?;
            let disposition = if profile
                .exclusions()
                .iter()
                .any(|item| item.path == member.path)
            {
                PlatformEntryDisposition::Excluded {
                    rule_path: member.path.clone(),
                }
            } else {
                PlatformEntryDisposition::Included {
                    digest: content_digest,
                    byte_length: member.bytes,
                    object_id: git_members.then(|| member.git_blob_id.clone()),
                }
            };
            let kind = match member.kind.as_str() {
                "lua" | "generated_api" => PlatformFileKind::Lua,
                "toc" => PlatformFileKind::Toc,
                "xml" => PlatformFileKind::Xml,
                "schema" => PlatformFileKind::Schema,
                _ => return Err(invalid("unexpected admitted platform manifest kind")),
            };
            entries.try_reserve(1).map_err(|_| budget())?;
            entries.push(PlatformInventoryEntry {
                path: member.path.clone(),
                kind,
                disposition,
            });
        }
        let inventory = PlatformSourceInventory {
            schema: "wow-project/platform-source-inventory/1".into(),
            profile_digest: profile.digest(),
            target: profile.target().clone(),
            origin: input.origin.clone(),
            materializer: input.materializer.clone(),
            roots,
            entries,
            license: input.license.clone(),
            compatibility_evidence: input.compatibility_evidence,
        }
        .canonicalize(profile, stop)?;
        let directory = self.subdirectory(&input.source_root)?;
        let version = manifest.member("version.txt")?;
        let version_bytes = directory.read(&version.disk_file()?, DISK_SOURCE_MAX_BYTES, stop)?;
        if std::str::from_utf8(&version_bytes)
            .map_err(|_| invalid("source version file is not UTF-8"))?
            .trim()
            != manifest.source.version
        {
            return Err(invalid("source version bytes disagree with the manifest"));
        }
        let receipt = PlatformSourceManifestReceipt {
            schema: "wow-project/platform-source-manifest-admission/1",
            manifest: LoadSource {
                path: input.manifest.path.clone(),
                content_digest: manifest_digest,
                byte_length: bytes.len() as u64,
            },
            manifest_sha256: manifest.manifest_sha256,
            source_revision: manifest.source.revision,
            source_selector: manifest.source.selector,
            source_version: manifest.source.version,
            acquisition: manifest.source.acquisition,
            declared_tracked_files: manifest.coverage.tracked_files,
            declared_selected_files: manifest.coverage.included_files,
            declared_selected_bytes: manifest.coverage.included_bytes,
            declared_unlisted_extension_exclusions: manifest.coverage.excluded_files,
            declared_generated_api_files: manifest.coverage.kind_generated_api.unwrap_or(0),
            version_file: LoadSource {
                path: "version.txt".into(),
                content_digest: source_digest(&version_bytes),
                byte_length: version_bytes.len() as u64,
            },
            outside_profile_members,
            git_membership: "not_attested",
            root_completeness: "not_attested",
            outside_profile_bytes: "not_verified",
            negative_authority: false,
        };
        serialized_size(&receipt, MAX_MANIFEST_BYTES)?;
        let source = directory.admit_platform_source(profile, inventory, stop)?;
        checkpoint(stop)?;
        Ok(ManifestedPlatformSource { source, receipt })
    }
}
