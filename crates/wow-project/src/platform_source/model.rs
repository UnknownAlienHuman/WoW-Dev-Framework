//! Bounded caller assertions for local platform-source byte admission.
//! These DTOs neither attest provenance nor construct admitted source bytes.

use std::{collections::BTreeSet, sync::atomic::AtomicBool};

use serde::{Deserialize, Serialize};
use wow_core::{CanonicalResult, ContentDigest, ProducerId, SourceContent, ToolVersion};

use super::{
    budget, invalid, path,
    profile::{BlizzardUiSourceProfile, PlatformSourceClass, PlatformTarget},
    serialized_size, text, under,
};
use crate::{ProjectResult, disk::checkpoint};

const SCHEMA: &str = "wow-project/platform-source-inventory/1";

/// Exact caller-supplied revision; no moving selector is resolved here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformSourceRevision {
    Git {
        object_format: String,
        commit: String,
        tree: String,
    },
    Archive {
        digest: ContentDigest<SourceContent>,
    },
    Fixture {
        digest: ContentDigest<SourceContent>,
    },
}

/// Asserted origin metadata, not proof of repository membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSourceOrigin {
    pub provider: String,
    pub repository: String,
    pub revision: PlatformSourceRevision,
}

/// Materializer identity and asserted evidence, not security attestation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformMaterializer {
    pub producer: ProducerId,
    pub version: ToolVersion,
    pub configuration_digest: ContentDigest<CanonicalResult>,
    pub report_digest: ContentDigest<SourceContent>,
}

/// Materializer's inventory claim; byte admission cannot verify root closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformInventoryScope {
    DeclaredComplete,
    DeclaredPartial,
}

/// Explicit root accounting for all supplied entry dispositions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformRootInventory {
    pub root: String,
    pub declared_entries: u64,
    pub scope: PlatformInventoryScope,
    pub evidence_digest: ContentDigest<SourceContent>,
}

/// Asserted license state; no variant grants redistribution permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformLicenseState {
    Known,
    Unknown,
    Conflict,
}

/// License evidence inherited by this inventory's entries, for local use only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformLicenseRecord {
    pub state: PlatformLicenseState,
    pub attribution: String,
    pub evidence_digest: ContentDigest<SourceContent>,
}

/// Extension classification, independent of decoding or parser coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformFileKind {
    Lua,
    Toc,
    Xml,
    Schema,
    Unknown,
}

/// Unsupported materialized entry classes; none permits traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformSpecialEntry {
    Symlink,
    Reparse,
    Submodule,
    LfsPointer,
    Unknown,
}

/// Exactly one disposition per entry; omissions remain explicit input records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlatformEntryDisposition {
    Included {
        digest: ContentDigest<SourceContent>,
        byte_length: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        object_id: Option<String>,
    },
    Excluded {
        rule_path: String,
    },
    Unsupported {
        kind: PlatformSpecialEntry,
        evidence_digest: ContentDigest<SourceContent>,
    },
    External {
        evidence_digest: ContentDigest<SourceContent>,
    },
    Conflict {
        reason: String,
    },
    Failed {
        reason: String,
    },
}

/// One snapshot-relative entry, including unknown or unconsumed members.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformInventoryEntry {
    pub path: String,
    pub kind: PlatformFileKind,
    pub disposition: PlatformEntryDisposition,
}

/// Versioned input assertions. Canonicalization performs no source IO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSourceInventory {
    pub schema: String,
    pub profile_digest: ContentDigest<CanonicalResult>,
    pub target: PlatformTarget,
    pub origin: PlatformSourceOrigin,
    pub materializer: PlatformMaterializer,
    pub roots: Vec<PlatformRootInventory>,
    pub entries: Vec<PlatformInventoryEntry>,
    pub license: PlatformLicenseRecord,
    pub compatibility_evidence: ContentDigest<SourceContent>,
}

impl PlatformSourceInventory {
    /// Bound the envelope before typed decoding, then validate every assertion.
    pub fn from_json(
        bytes: &[u8],
        profile: &BlizzardUiSourceProfile,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        checkpoint(stop)?;
        profile.validate()?;
        if bytes.len() > profile.limits().max_manifest_bytes {
            return Err(budget("platform inventory envelope exceeds its limit"));
        }
        let decoded = serde_json::from_slice::<Self>(bytes);
        checkpoint(stop)?;
        decoded
            .map_err(|_| invalid("invalid or ambiguous platform source inventory"))?
            .canonicalize(profile, stop)
    }

    /// Validate direct Rust inputs and canonicalize order before member IO.
    /// The returned value is still an input DTO, not an admission capability.
    pub fn canonicalize(
        mut self,
        profile: &BlizzardUiSourceProfile,
        stop: &AtomicBool,
    ) -> ProjectResult<Self> {
        checkpoint(stop)?;
        profile.validate()?;
        checkpoint(stop)?;
        let limits = profile.limits();
        if self.entries.len() > limits.max_entries {
            return Err(budget("platform inventory has too many entries"));
        }
        if self.roots.len() != profile.roots().len() {
            return Err(invalid("platform inventory roots differ from the profile"));
        }
        serialized_size(&self, limits.max_manifest_bytes)?;
        checkpoint(stop)?;
        if self.schema != SCHEMA
            || self.profile_digest != profile.digest()
            || &self.target != profile.target()
        {
            return Err(invalid(
                "platform inventory schema or target binding differs",
            ));
        }
        self.target.validate()?;
        text(&self.origin.provider)?;
        text(&self.origin.repository)?;
        validate_revision(&self.origin.revision, profile.source_class())?;
        text(&self.license.attribution)?;

        let max_entries = u64::try_from(limits.max_entries)
            .map_err(|_| budget("platform entry budget is not representable"))?;
        let mut root_names = BTreeSet::new();
        let mut declared_entries = 0_u64;
        for root in &self.roots {
            checkpoint(stop)?;
            path(&root.root)?;
            if !root_names.insert(root.root.as_str())
                || !profile.roots().iter().any(|item| item.root == root.root)
            {
                return Err(invalid(
                    "platform inventory has an unexpected or repeated root",
                ));
            }
            declared_entries = declared_entries
                .checked_add(root.declared_entries)
                .filter(|total| *total <= max_entries)
                .ok_or_else(|| budget("platform root accounting exceeds the entry limit"))?;
        }
        if usize::try_from(declared_entries).ok() != Some(self.entries.len()) {
            return Err(invalid(
                "platform declared entry count differs from inventory",
            ));
        }

        let exclusion_paths: BTreeSet<_> = profile
            .exclusions()
            .iter()
            .map(|item| item.path.to_ascii_lowercase())
            .collect();
        let mut actual_counts = vec![0_u64; self.roots.len()];
        let mut folded_paths = BTreeSet::new();
        let mut leaves = BTreeSet::new();
        let mut total_bytes = 0_u64;
        for entry in &self.entries {
            checkpoint(stop)?;
            path(&entry.path)?;
            let folded = entry.path.to_ascii_lowercase();
            if !folded_paths.insert(folded.clone()) {
                return Err(invalid("platform inventory paths collide ignoring case"));
            }
            if entry.kind != classify(&folded) {
                return Err(invalid("platform entry kind disagrees with its extension"));
            }
            let mut memberships = self
                .roots
                .iter()
                .enumerate()
                .filter(|(_, root)| under(&entry.path, &root.root));
            let root_index = memberships
                .next()
                .map(|(index, _)| index)
                .ok_or_else(|| invalid("platform entry has no configured root"))?;
            if memberships.next().is_some() {
                return Err(invalid("platform entry belongs to more than one root"));
            }
            actual_counts[root_index] = actual_counts[root_index]
                .checked_add(1)
                .ok_or_else(|| budget("platform root entry accounting overflow"))?;

            match &entry.disposition {
                PlatformEntryDisposition::Included {
                    byte_length,
                    object_id,
                    ..
                } => {
                    if exclusion_paths.contains(&folded) {
                        return Err(invalid("included platform member matches an exclusion"));
                    }
                    if *byte_length > limits.max_file_bytes {
                        return Err(budget("platform member exceeds its raw byte limit"));
                    }
                    total_bytes = total_bytes
                        .checked_add(*byte_length)
                        .filter(|total| *total <= limits.max_total_bytes)
                        .ok_or_else(|| budget("platform inventory exceeds its total byte limit"))?;
                    if let Some(object_id) = object_id {
                        let PlatformSourceRevision::Git { object_format, .. } =
                            &self.origin.revision
                        else {
                            return Err(invalid("platform object ID requires a Git object format"));
                        };
                        if !hex(object_id, object_length(object_format)?) {
                            return Err(invalid("invalid platform member Git object ID"));
                        }
                    }
                    leaves.insert(folded);
                }
                PlatformEntryDisposition::Excluded { rule_path } => {
                    path(rule_path)?;
                    if rule_path != &entry.path
                        || !profile
                            .exclusions()
                            .iter()
                            .any(|item| item.path == *rule_path)
                    {
                        return Err(invalid("platform exclusion is not an exact reviewed rule"));
                    }
                    leaves.insert(folded);
                }
                PlatformEntryDisposition::Conflict { reason }
                | PlatformEntryDisposition::Failed { reason } => text(reason)?,
                PlatformEntryDisposition::Unsupported { .. }
                | PlatformEntryDisposition::External { .. } => {}
            }
        }
        for (root, actual) in self.roots.iter().zip(actual_counts) {
            checkpoint(stop)?;
            if root.declared_entries != actual {
                return Err(invalid(
                    "platform root count differs from its entry accounting",
                ));
            }
        }
        for selected in &folded_paths {
            checkpoint(stop)?;
            for (boundary, _) in selected.match_indices('/') {
                if leaves.contains(&selected[..boundary]) {
                    return Err(invalid(
                        "platform file leaf is an ancestor of another entry",
                    ));
                }
            }
        }

        checkpoint(stop)?;
        self.roots.sort_by(|left, right| left.root.cmp(&right.root));
        checkpoint(stop)?;
        self.entries
            .sort_by(|left, right| left.path.cmp(&right.path));
        checkpoint(stop)?;
        Ok(self)
    }
}

fn validate_revision(
    revision: &PlatformSourceRevision,
    source_class: PlatformSourceClass,
) -> ProjectResult<()> {
    match (source_class, revision) {
        (
            PlatformSourceClass::VendorUiSourceMirror,
            PlatformSourceRevision::Git {
                object_format,
                commit,
                tree,
            },
        ) => {
            let length = object_length(object_format)?;
            if !hex(commit, length)
                || !hex(tree, length)
                || commit.bytes().all(|byte| byte == b'0')
                || tree.bytes().all(|byte| byte == b'0')
            {
                return Err(invalid(
                    "platform Git revision requires exact nonzero commit and tree",
                ));
            }
            Ok(())
        }
        (PlatformSourceClass::VendorUiSourceMirror, PlatformSourceRevision::Archive { .. })
        | (PlatformSourceClass::SyntheticFixture, PlatformSourceRevision::Fixture { .. }) => Ok(()),
        _ => Err(invalid(
            "platform revision and source evidence class differ",
        )),
    }
}

fn object_length(format: &str) -> ProjectResult<usize> {
    match format {
        "sha1" => Ok(40),
        "sha256" => Ok(64),
        _ => Err(invalid("unsupported platform Git object format")),
    }
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn classify(path: &str) -> PlatformFileKind {
    if path.ends_with(".lua") {
        PlatformFileKind::Lua
    } else if path.ends_with(".toc") {
        PlatformFileKind::Toc
    } else if path.ends_with(".xml") {
        PlatformFileKind::Xml
    } else if path.ends_with(".xsd") {
        PlatformFileKind::Schema
    } else {
        PlatformFileKind::Unknown
    }
}
